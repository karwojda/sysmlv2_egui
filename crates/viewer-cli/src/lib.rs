//! The headless command-line tool: Exports saved Diagrams of a Model to SVG.

use std::ffi::OsString;
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};
use viewer_core::model::{Diagnostic, Severity};
use viewer_core::{DiagramFile, ModelBackend, build_scene, diagram_file, svg};

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
    /// Export a saved Diagram of a Model to SVG.
    Export {
        /// The Model folder.
        model_folder: PathBuf,
        /// The name of the Diagram File to Export.
        #[arg(long)]
        diagram: String,
        /// The SVG file to write.
        #[arg(long)]
        out: PathBuf,
    },
}

/// Runs the tool with `args` (program name first), loading Models through
/// `backend`. Messages go to `stderr`; the result is the process exit code.
pub fn run(args: impl IntoIterator<Item = OsString>, backend: &mut dyn ModelBackend, stderr: &mut dyn Write) -> u8 {
    let cli = match Cli::try_parse_from(args) {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return error.exit_code() as u8;
        }
    };
    let result = match cli.command {
        Command::Export { model_folder, diagram, out } => export(&model_folder, &diagram, &out, backend, stderr),
    };
    match result {
        Ok(()) => EXIT_OK,
        Err(error) => {
            let _ = writeln!(stderr, "error: {error}");
            EXIT_FAILURE
        }
    }
}

fn export(
    model_folder: &Path,
    name: &str,
    out: &Path,
    backend: &mut dyn ModelBackend,
    stderr: &mut dyn Write,
) -> Result<(), Failure> {
    let file_path = diagram_file::path_for(model_folder, name);
    let text = std::fs::read_to_string(&file_path)
        .map_err(|e| Failure(format!("cannot read Diagram File {}: {e}", file_path.display())))?;
    let diagram =
        DiagramFile::parse(&text).map_err(|e| Failure(format!("{}: {e}", file_path.display())))?;

    let model = backend.load(model_folder).map_err(|e| Failure(e.to_string()))?;
    for diagnostic in model.diagnostics() {
        let _ = writeln!(stderr, "{}", DiagnosticLine(diagnostic));
    }
    if model.has_errors() {
        return Err(Failure("the Model has errors; nothing was Exported".to_owned()));
    }

    let scene = build_scene(&model, diagram.kind, &diagram.root)
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
