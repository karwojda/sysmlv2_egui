//! Dev tool: runs the viewer on a Model folder, waits, saves a screenshot
//! of the window as PNG and exits.
//! `cargo run -p viewer-app --example screenshot -- <model-folder> <out.png> [root-element]`

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use eframe::egui;
use viewer_app::ViewerApp;
use viewer_opensysml::OpenSysmlBackend;

/// How long to wait before the screenshot; `SCREENSHOT_AFTER_SECS` overrides.
fn settle() -> Duration {
    let secs = std::env::var("SCREENSHOT_AFTER_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(6);
    Duration::from_secs(secs)
}

struct Screenshot {
    app: ViewerApp,
    out: PathBuf,
    started: Instant,
    requested: bool,
    saved: Arc<AtomicBool>,
}

impl eframe::App for Screenshot {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        self.app.ui(ui, frame);
        let ctx = ui.ctx().clone();
        if !self.requested && self.started.elapsed() > settle() {
            self.requested = true;
            let (out, saved, repaint) = (self.out.clone(), self.saved.clone(), ctx.clone());
            let callback = egui::ScreenshotCallback::new(move |image| {
                let [w, h] = image.size;
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(&out, &bytes, w as u32, h as u32, image::ColorType::Rgba8).unwrap();
                saved.store(true, Ordering::SeqCst);
                repaint.request_repaint();
            });
            ctx.send_viewport_cmd(egui::ViewportCommand::Screenshot(callback));
        }
        if self.saved.load(Ordering::SeqCst) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        ctx.request_repaint_after(Duration::from_millis(100));
    }
}

fn main() -> eframe::Result {
    let mut args = std::env::args_os().skip(1);
    let folder = PathBuf::from(args.next().expect("model folder"));
    let out = PathBuf::from(args.next().expect("output png"));
    let root = args.next().map(|r| viewer_core::QualifiedName::new(r.to_string_lossy()));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SysML v2 Viewer",
        options,
        Box::new(move |cc| {
            let app = ViewerApp::new(&cc.egui_ctx, Box::new(OpenSysmlBackend::new()), Some(folder), root);
            // SCREENSHOT_INTERCONNECTION=1 opens the root's Interconnection Diagram.
            let app = if std::env::var_os("SCREENSHOT_INTERCONNECTION").is_some() { app.prefer_interconnection() } else { app };
            // SCREENSHOT_STYLE=1 shows the Style panel instead of Properties.
            let app = if std::env::var_os("SCREENSHOT_STYLE").is_some() { app.show_style_panel() } else { app };
            Ok(Box::new(Screenshot { app, out, started: Instant::now(), requested: false, saved: Default::default() }))
        }),
    )
}
