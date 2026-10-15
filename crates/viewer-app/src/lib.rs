//! The desktop viewer: opens a Model folder and shows its Browser Diagram
//! and diagnostics.

pub mod loader;

use std::path::{Path, PathBuf};

use eframe::egui;
use viewer_core::browser::{self, BrowserEntry};
use viewer_core::model::{Diagnostic, Severity};
use viewer_core::{Model, ModelBackend};

use loader::Loader;

pub struct ViewerApp {
    loader: Loader,
    folder_input: String,
    loading: Option<PathBuf>,
    folder: Option<PathBuf>,
    model: Option<Model>,
    error: Option<String>,
    filter: String,
}

impl ViewerApp {
    /// Creates the app; with `folder`, starts loading it straight away.
    pub fn new(ctx: &egui::Context, backend: Box<dyn ModelBackend + Send>, folder: Option<PathBuf>) -> Self {
        let repaint = ctx.clone();
        let mut app = Self {
            loader: Loader::new(backend, move || repaint.request_repaint()),
            folder_input: folder.as_ref().map(|f| f.display().to_string()).unwrap_or_default(),
            loading: None,
            folder: None,
            model: None,
            error: None,
            filter: String::new(),
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
                    self.folder = Some(loaded.folder);
                    self.model = Some(model);
                }
                Err(error) => self.error = Some(error.to_string()),
            }
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
        egui::ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
            if entries.is_empty() {
                ui.weak(if filtering { "Nothing matches the filter." } else { "The Model is empty." });
            }
            for entry in &entries {
                browser_entry(ui, entry, 0, filtering);
            }
        });
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
}

/// Entries with children are collapsible. Top-level entries start open, and
/// while filtering every entry is open so matches are visible.
fn browser_entry(ui: &mut egui::Ui, entry: &BrowserEntry, depth: usize, filtering: bool) {
    let mut label = egui::RichText::new(format!("«{}» {}", entry.keyword, entry.name));
    if entry.unsupported {
        label = label.italics().weak();
    }
    if entry.children.is_empty() {
        // Line up with the text of collapsible entries, past their arrow.
        ui.horizontal(|ui| {
            ui.add_space(ui.spacing().icon_width + ui.spacing().icon_spacing);
            ui.label(label);
        });
    } else {
        egui::CollapsingHeader::new(label)
            .id_salt(entry.element.as_str())
            .default_open(depth == 0)
            .open(filtering.then_some(true))
            .show(ui, |ui| {
                for child in &entry.children {
                    browser_entry(ui, child, depth + 1, filtering);
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
        egui::CentralPanel::default().show(ui, |ui| {
            ui.centered_and_justified(|ui| {
                ui.weak("Diagrams will open here.");
            });
        });
    }
}
