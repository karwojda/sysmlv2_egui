//! The headless command-line tool: Exports saved Diagrams, and the Model's
//! views, to SVG.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{ArgGroup, Parser, Subcommand};
use viewer_core::model::{Diagnostic, Severity};
use viewer_core::style::{self, Style};
use viewer_core::view::{self, ViewError};
use viewer_core::{
    DiagramFile, DiagramSource, Model, ModelBackend, QualifiedName, diagram_file, svg,
};
use viewer_opensysml::ServiceConfig;

pub const EXIT_OK: u8 = 0;
pub const EXIT_FAILURE: u8 = 1;

#[derive(Parser)]
#[command(name = "sysml-viewer", about = "Export SysML v2 Diagrams to SVG")]
struct Cli {
    #[command(subcommand)]
    command: Command,
    #[command(flatten)]
    service: ServiceArgs,
}

/// How to reach sysml-grpc; flags win over the environment variables.
#[derive(clap::Args)]
pub struct ServiceArgs {
    /// The sysml-grpc binary to start (default: $OPENSYSML_GRPC_BINARY, else
    /// a cached or downloaded release).
    #[arg(long, global = true, value_name = "PATH")]
    pub sysml_binary: Option<PathBuf>,
    /// Use a sysml-grpc service already running at HOST:PORT instead of
    /// starting one (default: $OPENSYSML_SERVICE).
    #[arg(long, global = true, value_name = "HOST:PORT")]
    pub sysml_service: Option<String>,
}

impl ServiceArgs {
    /// These settings over the environment's.
    pub fn config(&self) -> ServiceConfig {
        let env = ServiceConfig::from_env();
        ServiceConfig {
            binary: self.sysml_binary.clone().or(env.binary),
            service: self.sysml_service.clone().or(env.service),
        }
    }
}

/// The service settings in `args`, for building the backend before
/// [`run`]; if `args` don't parse, the environment's (and `run` reports the
/// problem).
pub fn service_config(args: &[OsString]) -> ServiceConfig {
    Cli::try_parse_from(args)
        .map(|cli| cli.service.config())
        .unwrap_or_else(|_| ServiceConfig::from_env())
}

#[derive(Subcommand)]
enum Command {
    /// Export saved Diagrams, or the Model's views, to SVG.
    #[command(group(ArgGroup::new("which").required(true).args(["diagram", "view", "all"])))]
    Export {
        /// The Model folder.
        model_folder: PathBuf,
        /// The name of the Diagram File to Export.
        #[arg(long, requires = "out")]
        diagram: Option<String>,
        /// The view to Export, by simple or qualified name.
        #[arg(long, requires = "out")]
        view: Option<String>,
        /// The SVG file to write (with --diagram or --view).
        #[arg(long, conflicts_with = "all")]
        out: Option<PathBuf>,
        /// Export every Diagram File, as <out-dir>/<diagram-name>.svg, and
        /// every view without one, as <out-dir>/<view-name>.svg.
        #[arg(long, requires = "out_dir")]
        all: bool,
        /// The folder to write SVG files into (with --all).
        #[arg(long, conflicts_with_all = ["diagram", "view"])]
        out_dir: Option<PathBuf>,
    },
}

/// Runs the tool with `args` (program name first), loading Models through
/// `backend`. Messages go to `stderr`; the result is the process exit code.
pub fn run(
    args: impl IntoIterator<Item = OsString>,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> u8 {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        // Help and version go to stdout; usage errors to `stderr`.
        Err(error) if error.use_stderr() => {
            let _ = write!(stderr, "{error}");
            return error.exit_code() as u8;
        }
        Err(error) => {
            let _ = error.print();
            return error.exit_code() as u8;
        }
    };
    let result = match cli.command {
        Command::Export {
            model_folder,
            diagram: Some(name),
            out: Some(out),
            ..
        } => export_one(&model_folder, &name, &out, backend, stderr),
        Command::Export {
            model_folder,
            view: Some(name),
            out: Some(out),
            ..
        } => export_view(&model_folder, &name, &out, backend, stderr),
        Command::Export {
            model_folder,
            out_dir: Some(out_dir),
            ..
        } => export_all(&model_folder, &out_dir, backend, stderr),
        Command::Export { .. } => {
            unreachable!("clap enforces --diagram or --view with --out, or --all with --out-dir")
        }
    };
    match result {
        Ok(()) => EXIT_OK,
        Err(error) => {
            let _ = writeln!(stderr, "error: {error}");
            EXIT_FAILURE
        }
    }
}

fn export_one(
    model_folder: &Path,
    name: &str,
    out: &Path,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let diagram = read_diagram(model_folder, name)?;
    let model = load_model(model_folder, backend, stderr)?;
    let style = load_style(model_folder, stderr);
    render_to(&model, &diagram, style.as_ref(), out)
}

/// Exports the view `name` (simple or qualified), with the Layout of a
/// saved Diagram File of it if there is one.
fn export_view(
    model_folder: &Path,
    name: &str,
    out: &Path,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let model = load_model(model_folder, backend, stderr)?;
    let view = find_view(&model, name)?;
    if let Err(ViewError::Unsupported { reason, .. }) = view::kind(&model, &view) {
        return Err(Failure(format!("view `{view}` can't be drawn: {reason}")));
    }
    let source = DiagramSource::View { view: view.clone() };
    let saved = diagram_file::names_in(model_folder)
        .unwrap_or_default()
        .iter()
        .filter_map(|name| read_diagram(model_folder, name).ok())
        .find(|file| file.source == source);
    let diagram = saved.unwrap_or_else(|| DiagramFile::of(view::file_name(&view), source));
    let style = load_style(model_folder, stderr);
    render_to(&model, &diagram, style.as_ref(), out)
}

/// The view a command-line name means: a qualified name, else the one view
/// with that simple name.
fn find_view(model: &Model, name: &str) -> Result<QualifiedName, Failure> {
    let views: Vec<&QualifiedName> = view::views(model).map(|v| &v.qualified_name).collect();
    if let Some(view) = views.iter().find(|v| v.as_str() == name) {
        return Ok((*view).clone());
    }
    match views
        .iter()
        .filter(|v| v.simple_name() == name)
        .collect::<Vec<_>>()
        .as_slice()
    {
        [view] => Ok((**view).clone()),
        [] => Err(Failure(format!("the Model has no view named `{name}`"))),
        many => {
            let names: Vec<String> = many.iter().map(|v| format!("`{v}`")).collect();
            Err(Failure(format!(
                "`{name}` names several views ({}); use the qualified name",
                names.join(", ")
            )))
        }
    }
}

/// Exports every Diagram File, and every view not saved as one, carrying on
/// past failures so that each one is reported, and fails if any did.
/// Unsupported Views are skipped with a warning. Two Exports that would
/// share a file fail the whole run before anything is written.
fn export_all(
    model_folder: &Path,
    out_dir: &Path,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let names = diagram_file::names_in(model_folder).map_err(|e| {
        Failure(format!(
            "cannot list Diagram Files in {}: {e}",
            model_folder.display()
        ))
    })?;
    let model = load_model(model_folder, backend, stderr)?;

    // What to write, by output name, and what each output came from.
    let mut jobs: Vec<(String, Result<DiagramFile, Failure>)> = Vec::new();
    let mut origin: BTreeMap<String, String> = BTreeMap::new();
    for name in &names {
        origin.insert(name.clone(), format!("Diagram File `{name}`"));
        jobs.push((name.clone(), read_diagram(model_folder, name)));
    }
    let saved_views: Vec<DiagramSource> = jobs
        .iter()
        .filter_map(|(_, file)| file.as_ref().ok())
        .map(|f| f.source.clone())
        .collect();
    for element in view::views(&model) {
        let view = &element.qualified_name;
        if let Err(ViewError::Unsupported { reason, .. }) = view::kind(&model, view) {
            let _ = writeln!(stderr, "warning: view `{view}` is not Exported: {reason}");
            continue;
        }
        let source = DiagramSource::View { view: view.clone() };
        if saved_views.contains(&source) {
            continue;
        }
        let name = view::file_name(view);
        if let Some(other) = origin.insert(name.clone(), format!("view `{view}`")) {
            return Err(Failure(format!(
                "{other} and view `{view}` would both be Exported as {name}.svg; nothing was Exported"
            )));
        }
        jobs.push((name.clone(), Ok(DiagramFile::of(name, source))));
    }
    if jobs.is_empty() {
        let _ = writeln!(
            stderr,
            "no Diagram Files or views in {}; nothing to Export",
            model_folder.display()
        );
        return Ok(());
    }

    let style = load_style(model_folder, stderr);
    let mut failed = 0;
    for (name, diagram) in &jobs {
        let out = out_dir.join(format!("{name}.svg"));
        let result = match diagram {
            Ok(diagram) => render_to(&model, diagram, style.as_ref(), &out),
            Err(error) => Err(Failure(error.0.clone())),
        };
        if let Err(error) = result {
            let _ = writeln!(stderr, "error: {error}");
            failed += 1;
        }
    }
    if failed > 0 {
        return Err(Failure(format!(
            "{failed} of {} Diagrams failed to Export",
            jobs.len()
        )));
    }
    Ok(())
}

fn read_diagram(model_folder: &Path, name: &str) -> Result<DiagramFile, Failure> {
    let path = diagram_file::path_for(model_folder, name);
    let text = std::fs::read_to_string(&path)
        .map_err(|e| Failure(format!("cannot read Diagram File {}: {e}", path.display())))?;
    DiagramFile::parse(&text).map_err(|e| Failure(format!("{}: {e}", path.display())))
}

/// Loads the Model and prints its diagnostics; a Model with errors fails.
fn load_model(
    model_folder: &Path,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> Result<Model, Failure> {
    let model = backend
        .load(model_folder)
        .map_err(|e| Failure(e.to_string()))?;
    for diagnostic in model.diagnostics() {
        let _ = writeln!(stderr, "{}", DiagnosticLine(diagnostic));
    }
    if model.has_errors() {
        return Err(Failure(
            "the Model has errors; nothing was Exported".to_owned(),
        ));
    }
    Ok(model)
}

/// The Model Style, if it has one; a broken Style file is reported and
/// Diagrams are drawn without it.
fn load_style(model_folder: &Path, stderr: &mut dyn Write) -> Option<Style> {
    style::load(model_folder).unwrap_or_else(|error| {
        let _ = writeln!(
            stderr,
            "warning: {error}; Exporting without the Model Style"
        );
        None
    })
}

fn render_to(
    model: &Model,
    diagram: &DiagramFile,
    style: Option<&Style>,
    out: &Path,
) -> Result<(), Failure> {
    let mut scene = diagram
        .source
        .build_scene(model, &diagram.layout)
        .map_err(|e| Failure(format!("Diagram `{}`: {e}", diagram.name)))?;
    style::apply_styles(&mut scene, model, style, &diagram.style);
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)
            .map_err(|e| Failure(format!("cannot create {}: {e}", parent.display())))?;
    }
    std::fs::write(out, svg::render(&scene))
        .map_err(|e| Failure(format!("cannot write {}: {e}", out.display())))
}

#[derive(Debug)]
struct Failure(String);

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// `file:line: error: message`, the form editors and CI logs link up.
struct DiagnosticLine<'a>(&'a Diagnostic);

impl fmt::Display for DiagnosticLine<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(location) = &self.0.location {
            write!(f, "{}:{}: ", location.file.display(), location.line)?;
        }
        let severity = match self.0.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
        };
        write!(f, "{severity}: {}", self.0.message)
    }
}
