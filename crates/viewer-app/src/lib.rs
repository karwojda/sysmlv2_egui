//! The desktop viewer: opens a Model folder, shows its Browser Diagram and
//! diagnostics, and opens General Diagrams on a canvas where their Layout
//! can be adjusted and saved.

pub mod canvas;
pub mod loader;

use std::path::{Path, PathBuf};

use eframe::egui;
use viewer_core::browser::{self, BrowserEntry};
use viewer_core::editing::Edit;
use viewer_core::model::{Diagnostic, Severity};
use viewer_core::{DiagramFile, DiagramKind, Model, ModelBackend, QualifiedName, diagram_file};

use canvas::{Grab, OpenDiagram, RESIZE_HANDLE, View, grab_at};
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
    saved_names: Vec<String>,
    diagrams: Vec<OpenDiagram>,
    active: Option<usize>,
    /// A Diagram to open once the Model has loaded.
    pending_root: Option<QualifiedName>,
    export_path: String,
    save_name: String,
    status: Option<String>,
    drag: Option<Grab>,
    /// The element a context menu on the canvas was opened for.
    context_element: Option<QualifiedName>,
    confirm: Option<Confirm>,
    quit_confirmed: bool,
}

/// A question about discarding unsaved changes.
enum Confirm {
    CloseTab(usize),
    Quit,
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
            saved_names: Vec::new(),
            diagrams: Vec::new(),
            active: None,
            pending_root: root,
            export_path: String::new(),
            save_name: String::new(),
            status: None,
            drag: None,
            context_element: None,
            confirm: None,
            quit_confirmed: false,
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
                    self.saved_names = diagram_file::names_in(&loaded.folder).unwrap_or_default();
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

    /// Opens a General Diagram of `root`, or switches to one already open.
    fn open_general(&mut self, root: QualifiedName) {
        let Some(model) = &self.model else { return };
        if let Some(index) = self.diagrams.iter().position(|d| d.root == root) {
            self.activate(index);
            return;
        }
        self.diagrams.push(OpenDiagram::open(model, DiagramKind::General, root));
        self.activate(self.diagrams.len() - 1);
    }

    /// Opens the saved Diagram File `name`, or switches to it if open.
    fn open_saved(&mut self, name: &str) {
        if let Some(index) = self.diagrams.iter().position(|d| d.name.as_deref() == Some(name)) {
            self.activate(index);
            return;
        }
        let (Some(model), Some(folder)) = (&self.model, &self.folder) else { return };
        let path = diagram_file::path_for(folder, name);
        let file = std::fs::read_to_string(&path)
            .map_err(|e| e.to_string())
            .and_then(|text| DiagramFile::parse(&text).map_err(|e| e.to_string()));
        match file {
            Ok(file) => {
                self.diagrams.push(OpenDiagram::open_saved(model, file));
                self.activate(self.diagrams.len() - 1);
            }
            Err(error) => self.status = Some(format!("Cannot open {}: {error}", path.display())),
        }
    }

    fn activate(&mut self, index: usize) {
        self.active = Some(index);
        let diagram = &self.diagrams[index];
        let file_name = diagram.name.clone().unwrap_or_else(|| diagram.root.simple_name().to_owned());
        let folder = self.folder.clone().unwrap_or_default();
        self.export_path = folder.join(format!("{file_name}.svg")).display().to_string();
        self.save_name = diagram.name.clone().unwrap_or_else(|| diagram.root.simple_name().to_lowercase());
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

    fn edit_active(&mut self, edit: Edit) {
        let (Some(active), Some(model)) = (self.active, &self.model) else { return };
        self.diagrams[active].edit(model, &edit);
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

    fn left_panel(&mut self, ui: &mut egui::Ui) {
        if self.model.is_some() {
            let mut open = None;
            egui::CollapsingHeader::new(format!("Saved Diagrams ({})", self.saved_names.len()))
                .default_open(true)
                .show(ui, |ui| {
                    if self.saved_names.is_empty() {
                        ui.weak("None yet. Use “Save as” on an open Diagram.");
                    }
                    for name in &self.saved_names {
                        if ui.selectable_label(false, name).on_hover_text("Open").clicked() {
                            open = Some(name.clone());
                        }
                    }
                });
            if let Some(name) = open {
                self.open_saved(&name);
            }
            ui.separator();
        }
        self.browser(ui);
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
        if self.active.is_none() {
            ui.centered_and_justified(|ui| {
                ui.weak("Double-click a package or definition in the Browser to open its General Diagram.");
            });
            return;
        }
        self.tabs(ui);
        let Some(active) = self.active else { return };
        self.toolbar(ui, active);
        ui.separator();
        self.canvas(ui, active);
    }

    fn tabs(&mut self, ui: &mut egui::Ui) {
        let mut activate = None;
        let mut close = None;
        ui.horizontal_wrapped(|ui| {
            for (index, diagram) in self.diagrams.iter().enumerate() {
                let tab = ui.selectable_label(self.active == Some(index), diagram.title());
                let tab = if diagram.dirty { tab.on_hover_text("Unsaved changes") } else { tab };
                if tab.clicked() {
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
            if self.diagrams[index].dirty {
                self.confirm = Some(Confirm::CloseTab(index));
            } else {
                self.close(index);
            }
        }
    }

    fn toolbar(&mut self, ui: &mut egui::Ui, active: usize) {
        let mut edits = Vec::new();
        ui.horizontal(|ui| {
            if ui.button("Fit").clicked() {
                self.diagrams[active].needs_fit = true;
            }
            ui.label(format!("{:.0}%", self.diagrams[active].view.zoom * 100.0));
            ui.separator();

            let diagram = &self.diagrams[active];
            if diagram.name.is_some() && ui.add_enabled(diagram.dirty, egui::Button::new("Save")).clicked() {
                let name = diagram.name.clone().expect("checked above");
                self.save_active(active, &name);
            }
            ui.add(egui::TextEdit::singleline(&mut self.save_name).desired_width(120.0).hint_text("name"));
            if ui.button("Save as").clicked() {
                let name = self.save_name.trim().to_owned();
                self.save_active(active, &name);
            }
            ui.separator();

            let hidden: Vec<QualifiedName> = self.diagrams[active].layout.hidden.iter().cloned().collect();
            ui.add_enabled_ui(!hidden.is_empty(), |ui| {
                ui.menu_button(format!("Hidden ({})", hidden.len()), |ui| {
                    for element in &hidden {
                        if ui.button(format!("Show {}", element.simple_name())).clicked() {
                            edits.push(Edit::Show(element.clone()));
                        }
                    }
                    ui.separator();
                    if ui.button("Show all").clicked() {
                        edits.push(Edit::ShowAll);
                    }
                });
            });
            if ui.button("Reset layout").on_hover_text("Back to the automatic arrangement").clicked() {
                edits.push(Edit::Reset);
            }
            ui.separator();

            ui.add(egui::TextEdit::singleline(&mut self.export_path).desired_width(260.0));
            if ui.button("Export SVG").clicked() {
                let path = PathBuf::from(self.export_path.trim());
                self.status = Some(match self.diagrams[active].export_svg(&path) {
                    Ok(()) => format!("Exported to {}", path.display()),
                    Err(error) => format!("Export failed: {error}"),
                });
            }
        });
        if let Some(status) = &self.status {
            ui.weak(status);
        }
        for edit in edits {
            self.edit_active(edit);
        }
    }

    fn save_active(&mut self, active: usize, name: &str) {
        let Some(folder) = self.folder.clone() else { return };
        self.status = Some(match self.diagrams[active].save_as(&folder, name) {
            Ok(()) => {
                self.saved_names = diagram_file::names_in(&folder).unwrap_or_default();
                format!("Saved {}", diagram_file::path_for(&folder, name).display())
            }
            Err(error) => format!("Not saved: {error}"),
        });
    }

    fn canvas(&mut self, ui: &mut egui::Ui, active: usize) {
        let (response, painter) = ui.allocate_painter(ui.available_size(), egui::Sense::click_and_drag());
        let origin = response.rect.min;
        let diagram = &mut self.diagrams[active];
        let scene = match &diagram.scene {
            Ok(scene) => scene,
            Err(error) => {
                painter.text(origin + egui::vec2(8.0, 8.0), egui::Align2::LEFT_TOP, error.to_string(), egui::FontId::default(), ui.visuals().error_fg_color);
                return;
            }
        };
        if diagram.needs_fit {
            diagram.view =
                View::fit(egui::vec2(scene.x, scene.y), egui::vec2(scene.width, scene.height), response.rect.size());
            diagram.needs_fit = false;
        }
        let view = diagram.view;
        let under = |p: egui::Pos2| scene.element_at(view.to_scene(origin, p)).cloned();

        // Decide what a drag does from where the press began (not where
        // the pointer is once the drag is recognised): resize from a
        // node's corner handle, move a node, or pan the view.
        if response.drag_started_by(egui::PointerButton::Primary)
            && let Some(start) = ui.input(|i| i.pointer.press_origin()).or(response.interact_pointer_pos())
        {
            self.drag = Some(grab_at(scene, view, origin, start));
        }
        let mut edit = None;
        if response.dragged() {
            let delta = response.drag_delta();
            match &self.drag {
                Some(Grab::Move(element)) => {
                    edit = Some(Edit::Move { element: element.clone(), dx: delta.x / view.zoom, dy: delta.y / view.zoom });
                }
                Some(Grab::Resize(element)) => {
                    if let Some(b) = scene.node_bounds(element) {
                        edit = Some(Edit::Resize {
                            element: element.clone(),
                            width: b.width + delta.x / view.zoom,
                            height: b.height + delta.y / view.zoom,
                        });
                    }
                }
                Some(Grab::Pan) | None => diagram.view.pan += delta,
            }
        }
        if response.drag_stopped() {
            self.drag = None;
        }
        if let Some(pointer) = response.hover_pos() {
            let (zoom, scroll) = ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta));
            if zoom != 1.0 {
                diagram.view.zoom_at(origin, pointer, zoom);
            } else {
                diagram.view.pan += scroll;
            }
        }

        // Over a node, or over the part of its handle outside it.
        let hovered = response.hover_pos().and_then(|p| match grab_at(scene, view, origin, p) {
            Grab::Resize(element) | Grab::Move(element) => Some(element),
            Grab::Pan => None,
        });
        if response.clicked() {
            self.selected = hovered.clone();
        }
        if response.secondary_clicked() {
            self.context_element = response.interact_pointer_pos().and_then(under);
        }
        let highlight = hovered.as_ref().or(self.selected.as_ref());
        let clipped = painter.with_clip_rect(response.rect);
        canvas::paint(&clipped, scene, view, origin, highlight);
        if let Some(bounds) = highlight.and_then(|e| scene.node_bounds(e)) {
            let corner = view.to_screen(origin, viewer_core::scene::Point::new(bounds.right(), bounds.bottom()));
            let handle = egui::Rect::from_center_size(corner, egui::vec2(RESIZE_HANDLE, RESIZE_HANDLE) * 0.8);
            clipped.rect_filled(handle, 1.0, egui::Color32::from_rgb(0x1f, 0x6f, 0xeb));
        }
        if let Some(pointer) = response.hover_pos() {
            let cursor = match (&self.drag, grab_at(scene, view, origin, pointer)) {
                (Some(Grab::Resize(_)), _) | (None, Grab::Resize(_)) => egui::CursorIcon::ResizeNwSe,
                (Some(Grab::Move(_)), _) => egui::CursorIcon::Grabbing,
                (None, Grab::Move(_)) => egui::CursorIcon::Grab,
                _ => egui::CursorIcon::Default,
            };
            ui.ctx().set_cursor_icon(cursor);
        }

        let collapsed = self.context_element.as_ref().is_some_and(|e| diagram.layout.collapsed.contains(e));
        let context_element = self.context_element.clone();
        response.context_menu(|ui| match &context_element {
            Some(element) => {
                ui.label(egui::RichText::new(element.as_str()).weak());
                if ui.button("Hide").clicked() {
                    edit = Some(Edit::Hide(element.clone()));
                    ui.close();
                }
                let (label, toggle) = if collapsed {
                    ("Expand", Edit::Expand(element.clone()))
                } else {
                    ("Collapse", Edit::Collapse(element.clone()))
                };
                if ui.button(label).clicked() {
                    edit = Some(toggle);
                    ui.close();
                }
            }
            None => {
                if ui.button("Show all hidden").clicked() {
                    edit = Some(Edit::ShowAll);
                    ui.close();
                }
                if ui.button("Reset layout").clicked() {
                    edit = Some(Edit::Reset);
                    ui.close();
                }
            }
        });
        if let Some(element) = hovered.filter(|_| self.drag.is_none()) {
            response.on_hover_text_at_pointer(element.as_str());
        }
        if let Some(edit) = edit {
            self.edit_active(edit);
        }
    }

    /// Asks before discarding unsaved changes, when closing a tab or the
    /// window.
    fn confirmations(&mut self, ctx: &egui::Context) {
        if ctx.input(|i| i.viewport().close_requested())
            && !self.quit_confirmed
            && self.diagrams.iter().any(|d| d.dirty)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm = Some(Confirm::Quit);
        }
        let Some(confirm) = &self.confirm else { return };
        let question = match confirm {
            Confirm::CloseTab(index) => format!("Discard unsaved changes to “{}”?", self.diagrams[*index].title()),
            Confirm::Quit => {
                let names: Vec<String> = self.diagrams.iter().filter(|d| d.dirty).map(|d| d.title()).collect();
                format!("Discard unsaved changes to {} and quit?", names.join(", "))
            }
        };
        let mut answer = None;
        egui::Modal::new(egui::Id::unique("confirm-discard")).show(ctx, |ui| {
            ui.label(question);
            ui.horizontal(|ui| {
                if ui.button("Discard").clicked() {
                    answer = Some(true);
                }
                if ui.button("Cancel").clicked() {
                    answer = Some(false);
                }
            });
        });
        match (answer, self.confirm.take()) {
            (Some(true), Some(Confirm::CloseTab(index))) => self.close(index),
            (Some(true), Some(Confirm::Quit)) => {
                self.quit_confirmed = true;
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            (Some(_), _) => {}
            (None, pending) => self.confirm = pending,
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
        let ctx = ui.ctx().clone();
        self.confirmations(&ctx);
        egui::Panel::top("top-bar").show(ui, |ui| self.top_bar(ui));
        egui::Panel::bottom("diagnostics")
            .resizable(true)
            .default_size(120.0)
            .show(ui, |ui| self.diagnostics(ui));
        egui::Panel::left("browser")
            .resizable(true)
            .default_size(300.0)
            .show(ui, |ui| self.left_panel(ui));
        egui::CentralPanel::default().show(ui, |ui| self.diagram_area(ui));
    }
}
