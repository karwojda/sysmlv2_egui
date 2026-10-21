use std::path::PathBuf;

use clap::Parser;
use eframe::egui;
use viewer_app::ViewerApp;
use viewer_core::QualifiedName;
use viewer_opensysml::{BINARY_VARIABLE, ServiceConfig};

/// View SysML v2 Models as Diagrams.
#[derive(Parser)]
#[command(name = "sysml-viewer-gui")]
struct Args {
    /// The Model folder to open.
    model_folder: Option<PathBuf>,
    /// An element whose Diagram to open once the Model has loaded: a General
    /// Diagram for a package or definition, an Interconnection Diagram for a
    /// part.
    root: Option<String>,
    /// Open an Interconnection Diagram of the root, even if it's a part def.
    #[arg(long)]
    interconnection: bool,
    /// The sysml-grpc binary to start (default: $OPENSYSML_GRPC_BINARY, else
    /// a cached or downloaded release).
    #[arg(long, value_name = "PATH")]
    sysml_binary: Option<PathBuf>,
    /// Use a sysml-grpc service already running at HOST:PORT instead of
    /// starting one (default: $OPENSYSML_SERVICE).
    #[arg(long, value_name = "HOST:PORT")]
    sysml_service: Option<String>,
    /// How the window is drawn: `wgpu` (Vulkan or GL through wgpu), or
    /// `glow` (OpenGL), which can behave better under WSLg.
    #[arg(long, value_enum, default_value = "wgpu")]
    renderer: Renderer,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Renderer {
    Wgpu,
    Glow,
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
        renderer: match args.renderer {
            Renderer::Wgpu => eframe::Renderer::Wgpu,
            Renderer::Glow => eframe::Renderer::Glow,
        },
        ..Default::default()
    };
    let (folder, root, interconnection) = (
        args.model_folder,
        args.root.map(QualifiedName::new),
        args.interconnection,
    );
    eframe::run_native(
        "SysML v2 Viewer",
        options,
        Box::new(|cc| {
            let app = ViewerApp::new(
                &cc.egui_ctx,
                Box::new(viewer_app::backend(config)),
                folder,
                root,
            );
            Ok(Box::new(if interconnection {
                app.prefer_interconnection()
            } else {
                app
            }))
        }),
    )
}
