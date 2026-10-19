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
    let secs = std::env::var("SCREENSHOT_AFTER_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(6);
    Duration::from_secs(secs)
}

struct Screenshot {
    app: ViewerApp,
    out: PathBuf,
    started: Instant,
    resized: bool,
    requested: bool,
    saved: Arc<AtomicBool>,
}

impl eframe::App for Screenshot {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // SCREENSHOT_ADAPTER=1 prints which graphics adapter wgpu chose.
        if !self.resized
            && std::env::var_os("SCREENSHOT_ADAPTER").is_some()
            && let Some(state) = frame.wgpu_render_state()
        {
            let info = state.adapter.get_info();
            eprintln!(
                "wgpu adapter: {} ({:?}, {:?})",
                info.name, info.backend, info.device_type
            );
        }
        self.app.ui(ui, frame);
        let ctx = ui.ctx().clone();
        // SCREENSHOT_RESIZE=WxH resizes the window halfway through the wait.
        if !self.resized && self.started.elapsed() > settle() / 2 {
            self.resized = true;
            let size = std::env::var("SCREENSHOT_RESIZE").ok().and_then(|s| {
                let (w, h) = s.split_once('x')?;
                Some(egui::vec2(w.parse().ok()?, h.parse().ok()?))
            });
            if let Some(size) = size {
                ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(size));
            }
            // SCREENSHOT_MOVE=X,Y moves it too, as dragging a left or top edge does.
            let position = std::env::var("SCREENSHOT_MOVE").ok().and_then(|s| {
                let (x, y) = s.split_once(',')?;
                Some(egui::pos2(x.parse().ok()?, y.parse().ok()?))
            });
            if let Some(position) = position {
                ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(position));
            }
            if std::env::var_os("SCREENSHOT_MAXIMIZE").is_some() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
            }
        }
        if !self.requested && self.started.elapsed() > settle() {
            self.requested = true;
            let (out, saved, repaint) = (self.out.clone(), self.saved.clone(), ctx.clone());
            let callback = egui::ScreenshotCallback::new(move |image| {
                let [w, h] = image.size;
                let bytes: Vec<u8> = image.pixels.iter().flat_map(|p| p.to_array()).collect();
                image::save_buffer(&out, &bytes, w as u32, h as u32, image::ColorType::Rgba8)
                    .unwrap();
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
    let root = args
        .next()
        .map(|r| viewer_core::QualifiedName::new(r.to_string_lossy()));
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1200.0, 800.0]),
        // SCREENSHOT_RENDERER=glow draws with OpenGL instead of wgpu.
        renderer: if std::env::var("SCREENSHOT_RENDERER").as_deref() == Ok("glow") {
            eframe::Renderer::Glow
        } else {
            eframe::Renderer::Wgpu
        },
        ..Default::default()
    };
    eframe::run_native(
        "SysML v2 Viewer",
        options,
        Box::new(move |cc| {
            let app = ViewerApp::new(
                &cc.egui_ctx,
                Box::new(OpenSysmlBackend::new()),
                Some(folder),
                root,
            );
            // SCREENSHOT_INTERCONNECTION=1 opens the root's Interconnection Diagram.
            let app = if std::env::var_os("SCREENSHOT_INTERCONNECTION").is_some() {
                app.prefer_interconnection()
            } else {
                app
            };
            // SCREENSHOT_STYLE=1 shows the Style panel instead of Properties.
            let app = if std::env::var_os("SCREENSHOT_STYLE").is_some() {
                app.show_style_panel()
            } else {
                app
            };
            Ok(Box::new(Screenshot {
                app,
                out,
                started: Instant::now(),
                resized: false,
                requested: false,
                saved: Default::default(),
            }))
        }),
    )
}
