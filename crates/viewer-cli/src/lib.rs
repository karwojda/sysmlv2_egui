//! The headless command-line tool: Exports saved Diagrams of a Model to SVG.

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{ArgGroup, Parser, Subcommand};
use viewer_core::model::{Diagnostic, Severity};
use viewer_core::{DiagramFile, Model, ModelBackend, build_scene_with_layout, diagram_file, svg};

pub const EXIT_OK: u8 = 0;
pub const EXIT_FAILURE: u8 = 1;

#[derive(Parser)]
#[command(name = "sysml-viewer", about = "Export SysML v2 Diagrams to SVG")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Export saved Diagrams of a Model to SVG.
    #[command(group(ArgGroup::new("which").required(true).args(["diagram", "all"])))]
    Export {
        /// The Model folder.
        model_folder: PathBuf,
        /// The name of the Diagram File to Export.
        #[arg(long, requires = "out")]
        diagram: Option<String>,
        /// The SVG file to write (with --diagram).
        #[arg(long, conflicts_with = "all")]
        out: Option<PathBuf>,
        /// Export every Diagram File, as <out-dir>/<diagram-name>.svg.
        #[arg(long, requires = "out_dir")]
        all: bool,
        /// The folder to write SVG files into (with --all).
        #[arg(long, conflicts_with = "diagram")]
        out_dir: Option<PathBuf>,
    },
}

/// Runs the tool with `args` (program name first), loading Models through
/// `backend`. Messages go to `stderr`; the result is the process exit code.
pub fn run(args: impl IntoIterator<Item = OsString>, backend: &mut dyn ModelBackend, stderr: &mut dyn Write) -> u8 {
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
        Command::Export { model_folder, diagram: Some(name), out: Some(out), .. } => {
            export_one(&model_folder, &name, &out, backend, stderr)
        }
        Command::Export { model_folder, out_dir: Some(out_dir), .. } => {
            export_all(&model_folder, &out_dir, backend, stderr)
        }
        Command::Export { .. } => unreachable!("clap enforces --diagram with --out or --all with --out-dir"),
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
    render_to(&model, &diagram, out)
}

/// Exports every Diagram File, carrying on past failures so that each one is
/// reported, and fails if any did.
fn export_all(
    model_folder: &Path,
    out_dir: &Path,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let names = diagram_file::names_in(model_folder)
        .map_err(|e| Failure(format!("cannot list Diagram Files in {}: {e}", model_folder.display())))?;
    if names.is_empty() {
        let _ = writeln!(stderr, "no Diagram Files in {}; nothing to Export", model_folder.display());
        return Ok(());
    }
    let model = load_model(model_folder, backend, stderr)?;

    let mut failed = 0;
    for name in &names {
        let out = out_dir.join(format!("{name}.svg"));
        if let Err(error) = read_diagram(model_folder, name).and_then(|diagram| render_to(&model, &diagram, &out)) {
            let _ = writeln!(stderr, "error: {error}");
            failed += 1;
        }
    }
    if failed > 0 {
        return Err(Failure(format!("{failed} of {} Diagrams failed to Export", names.len())));
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
fn load_model(model_folder: &Path, backend: &mut dyn ModelBackend, stderr: &mut dyn Write) -> Result<Model, Failure> {
    let model = backend.load(model_folder).map_err(|e| Failure(e.to_string()))?;
    for diagnostic in model.diagnostics() {
        let _ = writeln!(stderr, "{}", DiagnosticLine(diagnostic));
    }
    if model.has_errors() {
        return Err(Failure("the Model has errors; nothing was Exported".to_owned()));
    }
    Ok(model)
}

fn render_to(model: &Model, diagram: &DiagramFile, out: &Path) -> Result<(), Failure> {
    let scene = build_scene_with_layout(model, diagram.kind, &diagram.root, &diagram.layout)
        .map_err(|e| Failure(format!("Diagram `{}`: {e}", diagram.name)))?;
    if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| Failure(format!("cannot create {}: {e}", parent.display())))?;
    }
    std::fs::write(out, svg::render(&scene)).map_err(|e| Failure(format!("cannot write {}: {e}", out.display())))
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
