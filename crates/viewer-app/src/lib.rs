//! The desktop viewer: opens a Model folder, shows its Browser Diagram and
//! diagnostics, and opens General Diagrams on a canvas.

pub mod canvas;
pub mod loader;

use std::path::{Path, PathBuf};

use eframe::egui;
use viewer_core::browser::{self, BrowserEntry};
use viewer_core::model::{Diagnostic, Severity};
use viewer_core::{DiagramKind, Model, ModelBackend, QualifiedName};

use canvas::{OpenDiagram, View};
use loader::Loader;

pub struct ViewerApp {
    loader: Loader,
    folder_input: String,
    loading: Option<PathBuf>,
    folder: Option<PathBuf>,
    model: Option<Model>,
    error: Option<String>,
    filter: String,
    selected: Option<QualifiedName>,
    diagrams: Vec<OpenDiagram>,
    active: Option<usize>,
    /// A Diagram to open once the Model has loaded.
    pending_root: Option<QualifiedName>,
    export_path: String,
    status: Option<String>,
}

enum BrowserAction {
    Select(QualifiedName),
    OpenGeneral(QualifiedName),
}

impl ViewerApp {
    /// Creates the app; with `folder`, starts loading it straight away, and
    /// with `root` too, opens a General Diagram of it once loaded.
    pub fn new(
        ctx: &egui::Context,
        backend: Box<dyn ModelBackend + Send>,
        folder: Option<PathBuf>,
        root: Option<QualifiedName>,
    ) -> Self {
        let repaint = ctx.clone();
        let mut app = Self {
            loader: Loader::new(backend, move || repaint.request_repaint()),
            folder_input: folder.as_ref().map(|f| f.display().to_string()).unwrap_or_default(),
            loading: None,
            folder: None,
            model: None,
            error: None,
            filter: String::new(),
            selected: None,
            diagrams: Vec::new(),
            active: None,
            pending_root: root,
            export_path: String::new(),
            status: None,
        };
        if let Some(folder) = folder {
            app.open(folder);
        }
        app
    }

    fn open(&mut self, folder: PathBuf) {
        self.error = None;
        self.loading = Some(folder.clone());
        self.loader.load(folder);
    }

    fn collect_loads(&mut self) {
        while let Some(loaded) = self.loader.poll() {
            if self.loading.as_ref() == Some(&loaded.folder) {
                self.loading = None;
            }
            match loaded.result {
                Ok(model) => {
                    if self.folder.as_ref() != Some(&loaded.folder) {
                        self.diagrams.clear();
                        self.active = None;
                        self.selected = None;
                    }
                    for diagram in &mut self.diagrams {
                        diagram.refresh(&model);
                    }
                    self.folder = Some(loaded.folder);
                    self.model = Some(model);
                    if let Some(root) = self.pending_root.take() {
                        self.open_general(root);
                    }
                }
                Err(error) => self.error = Some(error.to_string()),
            }
        }
    }

    /// Opens a General Diagram of `root`, or switches to it if already open.
    fn open_general(&mut self, root: QualifiedName) {
        let Some(model) = &self.model else { return };
        if let Some(index) = self.diagrams.iter().position(|d| d.root == root) {
            self.activate(index);
            return;
        }
        self.diagrams.push(OpenDiagram::open(model, DiagramKind::General, root));
        self.activate(self.diagrams.len() - 1);
    }

    fn activate(&mut self, index: usize) {
        self.active = Some(index);
        let name = self.diagrams[index].root.simple_name().to_owned();
        let folder = self.folder.clone().unwrap_or_default();
        self.export_path = folder.join(format!("{name}.svg")).display().to_string();
        self.status = None;
    }

    fn close(&mut self, index: usize) {
        self.diagrams.remove(index);
        self.active = match self.active {
            _ if self.diagrams.is_empty() => None,
            Some(a) if a >= index && a > 0 => Some(a - 1),
            other => other,
        };
        if let Some(active) = self.active {
            self.activate(active);
        }
    }

    fn top_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Model folder:");
            let input = ui.add(egui::TextEdit::singleline(&mut self.folder_input).desired_width(420.0));
            let submitted = input.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
            if (ui.button("Open").clicked() || submitted) && !self.folder_input.trim().is_empty() {
                self.open(PathBuf::from(self.folder_input.trim()));
            }
            if let Some(folder) = &self.loading {
                ui.spinner();
                ui.label(format!("Loading {}…", folder.display()));
            } else if let Some(folder) = &self.folder {
                ui.weak(folder.display().to_string());
            }
        });
        if let Some(error) = &self.error {
            ui.colored_label(ui.visuals().error_fg_color, error);
        }
    }

    fn browser(&mut self, ui: &mut egui::Ui) {
        ui.heading("Browser");
        ui.add(egui::TextEdit::singleline(&mut self.filter).hint_text("Filter by name"));
        ui.separator();
        let Some(model) = &self.model else {
            ui.weak("Open a Model folder to browse it.");
            return;
        };
        let entries = browser::tree(model, &self.filter);
        let filtering = !self.filter.trim().is_empty();
        let mut actions = Vec::new();
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            if entries.is_empty() {
                ui.weak(if filtering { "Nothing matches the filter." } else { "The Model is empty." });
            }
            let context = EntryContext { model, selected: self.selected.as_ref(), filtering };
            for entry in &entries {
                browser_entry(ui, entry, 0, &context, &mut actions);
            }
        });
        for action in actions {
            match action {
                BrowserAction::Select(element) => self.selected = Some(element),
                BrowserAction::OpenGeneral(root) => self.open_general(root),
            }
        }
    }

    fn diagnostics(&self, ui: &mut egui::Ui) {
        let diagnostics = self.model.as_ref().map(Model::diagnostics).unwrap_or_default();
        ui.label(format!("Diagnostics ({})", diagnostics.len()));
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            for diagnostic in diagnostics {
                diagnostic_line(ui, diagnostic, self.folder.as_deref());
            }
        });
    }

    fn diagram_area(&mut self, ui: &mut egui::Ui) {
        let Some(active) = self.active else {
            ui.centered_and_justified(|ui| {
                ui.weak("Double-click a package or definition in the Browser to open its General Diagram.");
            });
            return;
        };
        self.tabs(ui);
        if self.active != Some(active) {
            return;
        }
        self.toolbar(ui, active);
        ui.separator();
        self.canvas(ui, active);
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        let mut activate = None;
        let mut close = None;
        ui.horizontal_wrapped(|ui| {
            for (index, diagram) in self.diagrams.iter().enumerate() {
                if ui.selectable_label(self.active == Some(index), diagram.title()).clicked() {
                    activate = Some(index);
                }
                if ui.small_button("×").on_hover_text("Close").clicked() {
                    close = Some(index);
                }
                ui.add_space(8.0);
            }
        });
        if let Some(index) = activate {
            self.activate(index);
        }
        if let Some(index) = close {
            self.close(index);
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, active: usize) {
        ui.horizontal(|ui| {
            if ui.button("Fit").clicked() {
                self.diagrams[active].needs_fit = true;
            }
            ui.label(format!("{:.0}%", self.diagrams[active].view.zoom * 100.0));
            ui.separator();
            ui.add(egui::TextEdit::singleline(&mut self.export_path).desired_width(360.0));
            if ui.button("Export SVG").clicked() {
                let path = PathBuf::from(self.export_path.trim());
                self.status = Some(match self.diagrams[active].export_svg(&path) {
                    Ok(()) => format!("Exported to {}", path.display()),
                    Err(error) => format!("Export failed: {error}"),
                });
            }
            if let Some(status) = &self.status {
                ui.weak(status);
            }
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui, active: usize) {
        let diagram = &mut self.diagrams[active];
        let scene = match &diagram.scene {
            Ok(scene) => scene,
            Err(error) => {
                ui.colored_label(ui.visuals().error_fg_color, error.to_string());
                return;
            }
        };
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let origin = response.rect.min;
        if diagram.needs_fit {
            diagram.view = View::fit(egui::vec2(scene.width, scene.height), response.rect.size());
            diagram.needs_fit = false;
        }
        if response.dragged() {
            diagram.view.pan += response.drag_delta();
        }
        if let Some(pointer) = response.hover_pos() {
            let (zoom, scroll) = ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta));
            if zoom != 1.0 {
                diagram.view.zoom_at(origin, pointer, zoom);
            } else {
                diagram.view.pan += scroll;
            }
        }
        let hovered = response
            .hover_pos()
            .and_then(|p| scene.element_at(diagram.view.to_scene(origin, p)))
            .cloned();
        if response.clicked() {
            self.selected = hovered.clone();
        }
        let highlight = hovered.as_ref().or(self.selected.as_ref());
        canvas::paint(&painter.with_clip_rect(response.rect), scene, diagram.view, origin, highlight);
        if let Some(element) = &hovered {
            response.on_hover_text_at_pointer(element.as_str());
        }
    }
}

struct EntryContext<'a> {
    model: &'a Model,
    selected: Option<&'a QualifiedName>,
    filtering: bool,
}

/// Entries with children are collapsible. Top-level entries start open, and
/// while filtering every entry is open so matches are visible. A click
/// selects; a double-click on a package or definition opens its General
/// Diagram, as does its context menu.
fn browser_entry(
    ui: &mut egui::Ui,
    entry: &BrowserEntry,
    depth: usize,
    context: &EntryContext,
    actions: &mut Vec<BrowserAction>,
) {
    let mut label = egui::RichText::new(format!("«{}» {}", entry.keyword, entry.name));
    if entry.unsupported {
        label = label.italics().weak();
    }
    let selected = context.selected == Some(&entry.element);
    let response = if entry.children.is_empty() {
        // Line up with the text of collapsible entries, past their arrow.
        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().icon_width + ui.spacing().icon_spacing);
            ui.selectable_label(selected, label)
        })
        .inner
    } else {
        let label = if selected { label.background_color(ui.visuals().selection.bg_fill) } else { label };
        egui::CollapsingHeader::new(label)
            .id_salt(entry.element.as_str())
            .default_open(depth == 0)
            .open(context.filtering.then_some(true))
            .show(ui, |ui| {
                for child in &entry.children {
                    browser_entry(ui, child, depth + 1, context, actions);
                }
            })
            .header_response
    };

    let openable = context
        .model
        .element(&entry.element)
        .is_some_and(|e| DiagramKind::General.accepts_root(e));
    if response.clicked() {
        actions.push(BrowserAction::Select(entry.element.clone()));
    }
    if openable && response.double_clicked() {
        actions.push(BrowserAction::OpenGeneral(entry.element.clone()));
    }
    if openable {
        response.context_menu(|ui| {
            if ui.button("Open General Diagram").clicked() {
                actions.push(BrowserAction::OpenGeneral(entry.element.clone()));
                ui.close();
            }
        });
    }
}

/// `file:line: severity: message`, with the file relative to the Model
/// folder. Line 0 means the whole file.
fn diagnostic_line(ui: &mut egui::Ui, diagnostic: &Diagnostic, folder: Option<&Path>) {
    let (severity, color) = match diagnostic.severity {
        Severity::Error => ("error", ui.visuals().error_fg_color),
        Severity::Warning => ("warning", ui.visuals().warn_fg_color),
    };
    let location = diagnostic
        .location
        .as_ref()
        .map(|l| {
            let file = folder.and_then(|f| l.file.strip_prefix(f).ok()).unwrap_or(&l.file);
            match l.line {
                0 => format!("{}: ", file.display()),
                line => format!("{}:{line}: ", file.display()),
            }
        })
        .unwrap_or_default();
    ui.colored_label(color, format!("{location}{severity}: {}", diagnostic.message));
}

impl eframe::App for ViewerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.collect_loads();
        egui::Panel::top("top-bar").show(ui, |ui| self.top_bar(ui));
        egui::Panel::bottom("diagnostics")
            .resizable(true)
            .default_size(120.0)
            .show(ui, |ui| self.diagnostics(ui));
        egui::Panel::left("browser")
            .resizable(true)
            .default_size(300.0)
            .show(ui, |ui| self.browser(ui));
        egui::CentralPanel::default().show(ui, |ui| self.diagram_area(ui));
    }
}
