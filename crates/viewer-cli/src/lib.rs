//! The headless command-line tool: Exports saved Diagrams, and the Model's
//! views, to SVG.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{ArgGroup, Parser, Subcommand};
use viewer_core::constraint::{self, ConstraintStatus};
use viewer_core::model::ClaimResult;
use viewer_core::model::{Diagnostic, Severity};
use viewer_core::satisfaction::{self, SatisfactionStatus};
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
    /// Report every requirement's Satisfaction Status, checked against the
    /// Model's values. Fails if any requirement is violated.
    Requirements {
        /// The Model folder.
        model_folder: PathBuf,
        /// Also fail on requirements that are undecided or not claimed.
        #[arg(long)]
        strict: bool,
    },
    /// Report every requirement's status and every asserted constraint's
    /// result, checked against the Model's values. Fails if anything is
    /// violated: one gate for CI.
    Check {
        /// The Model folder.
        model_folder: PathBuf,
        /// Also fail on anything undecided or not claimed.
        #[arg(long)]
        strict: bool,
    },
}

/// Runs the tool with `args` (program name first), loading Models through
/// `backend`. Reports go to standard output, messages to `stderr`; the
/// result is the process exit code.
pub fn run(
    args: impl IntoIterator<Item = OsString>,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> u8 {
    run_with_output(args, backend, &mut std::io::stdout(), stderr)
}

/// [`run`], with reports written to `stdout`.
pub fn run_with_output(
    args: impl IntoIterator<Item = OsString>,
    backend: &mut dyn ModelBackend,
    stdout: &mut dyn Write,
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
        Command::Requirements {
            model_folder,
            strict,
        } => requirements(&model_folder, strict, backend, stdout, stderr),
        Command::Check {
            model_folder,
            strict,
        } => check(&model_folder, strict, backend, stdout, stderr),
    };
    match result {
        Ok(()) => EXIT_OK,
        Err(error) => {
            let _ = writeln!(stderr, "error: {error}");
            EXIT_FAILURE
        }
    }
}

/// Prints one line per requirement usage, in qualified-name order: its
/// mark and status, its name, and each claim on it with its result. Fails
/// on any Violated requirement; with `strict`, on any not Satisfied.
fn requirements(
    model_folder: &Path,
    strict: bool,
    backend: &mut dyn ModelBackend,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let model = load_model_for(model_folder, backend, stderr, "no requirement was checked")?;
    let counts = requirements_report(&model, model_folder, stdout);
    match counts.failing(strict) {
        0 => Ok(()),
        n => Err(Failure(counts.failure(n, strict))),
    }
}

/// The `requirements` report, then one line per Constraint Result. Fails on
/// any violated requirement or constraint; with `strict`, on anything not
/// satisfied or not holding.
fn check(
    model_folder: &Path,
    strict: bool,
    backend: &mut dyn ModelBackend,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let model = load_model_for(model_folder, backend, stderr, "nothing was checked")?;
    let requirements = requirements_report(&model, model_folder, stdout);
    let constraints = constraints_report(&model, model_folder, stdout);
    let failures: Vec<String> = [
        (
            requirements.failing(strict),
            requirements.failure(requirements.failing(strict), strict),
        ),
        (
            constraints.failing(strict),
            constraints.failure(constraints.failing(strict), strict),
        ),
    ]
    .into_iter()
    .filter(|(n, _)| *n > 0)
    .map(|(_, message)| message)
    .collect();
    if failures.is_empty() {
        Ok(())
    } else {
        Err(Failure(failures.join("; ")))
    }
}

/// How a report came out: how many lines it had of each kind.
struct Counts {
    /// What a line is about, singular and plural: `requirement(s)`.
    noun: (&'static str, &'static str),
    /// What failing means, leniently and strictly: `violated`, `not satisfied`.
    fails: (&'static str, &'static str),
    violated: usize,
    /// Undecided, and for requirements also not claimed.
    unsettled: usize,
}

impl Counts {
    fn failing(&self, strict: bool) -> usize {
        if strict {
            self.violated + self.unsettled
        } else {
            self.violated
        }
    }

    fn failure(&self, n: usize, strict: bool) -> String {
        let (noun, verb) = if n == 1 {
            (self.noun.0, "is")
        } else {
            (self.noun.1, "are")
        };
        let what = if strict { self.fails.1 } else { self.fails.0 };
        format!("{n} {noun} {verb} {what}")
    }
}

/// Prints the requirements report and counts its lines.
fn requirements_report(model: &Model, model_folder: &Path, stdout: &mut dyn Write) -> Counts {
    let mut counts = Counts {
        noun: ("requirement", "requirements"),
        fails: ("violated", "not satisfied"),
        violated: 0,
        unsettled: 0,
    };
    let requirements: Vec<(&QualifiedName, SatisfactionStatus)> = model
        .elements()
        .filter_map(|e| {
            Some((
                &e.qualified_name,
                satisfaction::status(model, &e.qualified_name)?,
            ))
        })
        .collect();
    if requirements.is_empty() {
        let _ = writeln!(
            stdout,
            "no requirement usages in {}",
            model_folder.display()
        );
        return counts;
    }
    let width = requirements
        .iter()
        .map(|(name, _)| name.as_str().chars().count())
        .max()
        .unwrap_or(0);
    for (name, status) in &requirements {
        let claims: Vec<String> = satisfaction::claims_on(model, name)
            .map(|claim| {
                let satisfier = claim
                    .satisfier
                    .as_ref()
                    .map_or("?".to_owned(), ToString::to_string);
                let mark = satisfaction::claim_status(claim).mark();
                match &claim.claim_result {
                    Some(ClaimResult::Violated { condition }) => {
                        format!("by {satisfier} {mark} {condition}")
                    }
                    Some(ClaimResult::Undecided { reason } | ClaimResult::Unchecked { reason }) => {
                        format!("by {satisfier} {mark} {reason}")
                    }
                    _ => format!("by {satisfier} {mark}"),
                }
            })
            .collect();
        let nested = model
            .element(name)
            .map_or(0, |e| satisfaction::nested(model, e).count());
        let details = match (claims.is_empty(), nested) {
            (true, 0) => "nothing claims to satisfy it".to_owned(),
            (true, n) => format!("rolled up from {n} nested requirements"),
            (false, 0) => claims.join("; "),
            (false, n) => format!(
                "{}; rolled up from {n} nested requirements",
                claims.join("; ")
            ),
        };
        let _ = writeln!(
            stdout,
            "{} {:<11} {:<width$}  {details}",
            status.mark(),
            status.label(),
            name.as_str()
        );
    }
    let count =
        |wanted: SatisfactionStatus| requirements.iter().filter(|(_, s)| *s == wanted).count();
    let (satisfied, violated, undecided, unclaimed) = (
        count(SatisfactionStatus::Satisfied),
        count(SatisfactionStatus::Violated),
        count(SatisfactionStatus::Undecided),
        count(SatisfactionStatus::NotClaimed),
    );
    let _ = writeln!(
        stdout,
        "{} requirements: {satisfied} satisfied, {violated} violated, {undecided} undecided, {unclaimed} not claimed",
        requirements.len()
    );
    counts.violated = violated;
    counts.unsettled = undecided + unclaimed;
    counts
}

/// Prints one line per Constraint Result, by constraint and then part, and
/// counts them.
fn constraints_report(model: &Model, model_folder: &Path, stdout: &mut dyn Write) -> Counts {
    let mut counts = Counts {
        noun: ("constraint", "constraints"),
        fails: ("violated", "not holding"),
        violated: 0,
        unsettled: 0,
    };
    let mut results: Vec<(&QualifiedName, &viewer_core::model::ConstraintCheck)> = model
        .elements()
        .filter(|e| e.is_asserted)
        .flat_map(|c| c.checks.iter().map(move |check| (&c.qualified_name, check)))
        .collect();
    results.sort_by(|a, b| (a.0, &a.1.part).cmp(&(b.0, &b.1.part)));
    if results.is_empty() {
        let _ = writeln!(
            stdout,
            "no checked constraints in {}",
            model_folder.display()
        );
        return counts;
    }
    let width = results
        .iter()
        .map(|(name, _)| name.as_str().chars().count())
        .max()
        .unwrap_or(0);
    let (mut holding, mut undecided) = (0, 0);
    for (name, check) in &results {
        let status = constraint::check_status(&check.result);
        match status {
            ConstraintStatus::Holds => holding += 1,
            ConstraintStatus::Violated => counts.violated += 1,
            ConstraintStatus::Undecided => undecided += 1,
        }
        let detail = match &check.result {
            ClaimResult::Holds => String::new(),
            ClaimResult::Violated { condition } => format!(": {condition}"),
            ClaimResult::Undecided { reason } | ClaimResult::Unchecked { reason } => {
                format!(": {reason}")
            }
        };
        let _ = writeln!(
            stdout,
            "{} {:<11} {:<width$}  on {}{detail}",
            status.mark(),
            status.label(),
            name.as_str(),
            check.part
        );
    }
    let _ = writeln!(
        stdout,
        "{} constraint results: {holding} hold, {} violated, {undecided} undecided",
        results.len(),
        counts.violated
    );
    counts.unsettled = undecided;
    counts
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

/// Loads the Model and prints its diagnostics; a Model with errors fails,
/// saying nothing was Exported.
fn load_model(
    model_folder: &Path,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> Result<Model, Failure> {
    load_model_for(model_folder, backend, stderr, "nothing was Exported")
}

/// [`load_model`], with what a Model with errors stops (`outcome`).
fn load_model_for(
    model_folder: &Path,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
    outcome: &str,
) -> Result<Model, Failure> {
    let model = backend
        .load(model_folder)
        .map_err(|e| Failure(e.to_string()))?;
    for diagnostic in model.diagnostics() {
        let _ = writeln!(stderr, "{}", DiagnosticLine(diagnostic));
    }
    if model.has_errors() {
        return Err(Failure(format!("the Model has errors; {outcome}")));
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
