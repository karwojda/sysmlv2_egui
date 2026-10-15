use std::path::PathBuf;

use clap::Parser;
use eframe::egui;
use viewer_app::ViewerApp;
use viewer_core::QualifiedName;
use viewer_opensysml::{BINARY_VARIABLE, OpenSysmlBackend, ServiceConfig};

/// View SysML v2 Models as Diagrams.
#[derive(Parser)]
#[command(name = "sysml-viewer-gui")]
struct Args {
    /// The Model folder to open.
    model_folder: Option<PathBuf>,
    /// An element whose General Diagram to open once the Model has loaded.
    root: Option<String>,
    /// The sysml-grpc binary to start (default: $OPENSYSML_GRPC_BINARY, else
    /// a cached or downloaded release).
    #[arg(long, value_name = "PATH")]
    sysml_binary: Option<PathBuf>,
    /// Use a sysml-grpc service already running at HOST:PORT instead of
    /// starting one (default: $OPENSYSML_SERVICE).
    #[arg(long, value_name = "HOST:PORT")]
    sysml_service: Option<String>,
}

fn main() -> eframe::Result {
    let args = Args::parse();
    let env = ServiceConfig::from_env();
    let config = ServiceConfig {
        binary: args.sysml_binary.or(env.binary),
        service: args.sysml_service.or(env.service),
    };
    if let Some(binary) = &config.binary {
        // The opensysml client reads the binary from the environment only.
        // SAFETY: no other thread has started yet.
        unsafe { std::env::set_var(BINARY_VARIABLE, binary) };
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    let (folder, root) = (args.model_folder, args.root.map(QualifiedName::new));
    eframe::run_native(
        "SysML v2 Viewer",
        options,
        Box::new(|cc| {
            Ok(Box::new(ViewerApp::new(&cc.egui_ctx, Box::new(OpenSysmlBackend::with_config(config)), folder, root)))
        }),
    )
}
