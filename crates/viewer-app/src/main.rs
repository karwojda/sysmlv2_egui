use std::path::PathBuf;

use eframe::egui;
use viewer_app::ViewerApp;
use viewer_core::QualifiedName;
use viewer_opensysml::OpenSysmlBackend;

/// `sysml-viewer-gui [model-folder [root-element]]`: with a root element,
/// opens its General Diagram once the Model has loaded.
fn main() -> eframe::Result {
    let folder = std::env::args_os().nth(1).map(PathBuf::from);
    let root = std::env::args().nth(2).map(QualifiedName::new);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SysML v2 Viewer",
        options,
        Box::new(|cc| Ok(Box::new(ViewerApp::new(&cc.egui_ctx, Box::new(OpenSysmlBackend::new()), folder, root)))),
    )
}
