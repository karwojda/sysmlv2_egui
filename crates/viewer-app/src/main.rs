use std::path::PathBuf;

use eframe::egui;
use viewer_app::ViewerApp;
use viewer_opensysml::OpenSysmlBackend;

/// `sysml-viewer-gui [model-folder]`
fn main() -> eframe::Result {
    let folder = std::env::args_os().nth(1).map(PathBuf::from);
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SysML v2 Viewer",
        options,
        Box::new(|cc| Ok(Box::new(ViewerApp::new(&cc.egui_ctx, Box::new(OpenSysmlBackend::new()), folder)))),
    )
}
