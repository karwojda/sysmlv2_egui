use std::process::ExitCode;

use viewer_opensysml::OpenSysmlBackend;

fn main() -> ExitCode {
    let mut backend = OpenSysmlBackend::new();
    let code = viewer_cli::run(std::env::args_os(), &mut backend, &mut std::io::stderr());
    // Dropping the backend stops the sidecar before the process exits.
    drop(backend);
    ExitCode::from(code)
}
