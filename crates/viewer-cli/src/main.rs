use std::process::ExitCode;

use viewer_opensysml::{BINARY_VARIABLE, OpenSysmlBackend};

fn main() -> ExitCode {
    let args: Vec<_> = std::env::args_os().collect();
    let config = viewer_cli::service_config(&args);
    if let Some(binary) = &config.binary {
        // The opensysml client reads the binary from the environment only.
        // SAFETY: nothing else runs yet, so no thread reads the environment.
        unsafe { std::env::set_var(BINARY_VARIABLE, binary) };
    }
    let mut backend = OpenSysmlBackend::with_config(config);
    let code = viewer_cli::run(args, &mut backend, &mut std::io::stderr());
    // Dropping the backend stops the sidecar before the process exits.
    drop(backend);
    ExitCode::from(code)
}
