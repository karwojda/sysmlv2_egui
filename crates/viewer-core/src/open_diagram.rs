//! The Open Diagram: one Diagram being viewed or adjusted, with its Layout
//! and Style, their unsaved changes and undo history, and its Selection.
//! The GUI and the command line both draw, edit, save and Export Diagrams
//! through it; how it is shown on screen is theirs.

use std::path::Path;
use std::sync::Arc;

use crate::diagram::{DiagramError, DiagramKind, DiagramSource};
use crate::diagram_file::{self, DiagramFile, DiagramStyle, Layout};
use crate::editing::{self, Edit};
use crate::model::{Model, QualifiedName};
use crate::scene::{Rect, Scene};
use crate::style::{self, Style};
use crate::svg;

/// How many undo steps an Open Diagram keeps; older ones are dropped.
pub const UNDO_STEPS: usize = 200;

/// Why saving or exporting an Open Diagram failed.
#[derive(Debug, thiserror::Error)]
pub enum OpenDiagramError {
    #[error("`{0}` is not a valid Diagram name: use letters, digits, `-`, `_` and `.`")]
    NameNotAllowed(String),
    #[error("a Diagram named `{0}` already exists")]
    NameTaken(String),
    /// The Diagram can't be drawn, so there is nothing to Export.
    #[error(transparent)]
    NotDrawn(DiagramError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// A Diagram open for viewing or adjusting: what it shows, the user's
/// Layout and Style, and the Scene drawn from them. `name` is set once it
/// has been saved as a Diagram File; until then it is an ad hoc Diagram
/// that leaves no file behind.
pub struct OpenDiagram {
    /// The Model it is drawn from; every Scene is built from this one.
    model: Arc<Model>,
    source: DiagramSource,
    /// The Diagram Kind it was last drawn as; a view's can change with the
    /// Model, and is unknown while the view can't be drawn.
    kind: Option<DiagramKind>,
    layout: Layout,
    /// The Diagram's own Style, saved with it.
    style: DiagramStyle,
    /// The Model Style it is drawn with, as last loaded.
    model_style: Option<Style>,
    name: Option<String>,
    /// Layout changes not yet saved.
    dirty: bool,
    /// The Layout and Style as last saved (or as opened), to tell whether
    /// undoing has brought the Diagram back to them.
    saved: (Layout, DiagramStyle),
    /// Layouts to go back to, oldest first, and to go forward to again.
    undo: Vec<Layout>,
    redo: Vec<Layout>,
    /// While a drag is under way: whether it has recorded its undo step
    /// yet. A whole drag is one step, though it edits on every frame.
    drag_step: Option<bool>,
    selection: Selection,
    scene: Result<Scene, DiagramError>,
}

impl OpenDiagram {
    /// Opens the Diagram `source` shows, drawn from `model` with
    /// `model_style`, with no Layout adjustments yet.
    pub fn open(model: Arc<Model>, model_style: Option<Style>, source: DiagramSource) -> Self {
        Self::with_layout(
            model,
            model_style,
            source,
            Layout::default(),
            DiagramStyle::default(),
            None,
        )
    }

    /// Opens a saved Diagram File.
    pub fn open_saved(model: Arc<Model>, model_style: Option<Style>, file: DiagramFile) -> Self {
        Self::with_layout(
            model,
            model_style,
            file.source,
            file.layout,
            file.style,
            Some(file.name),
        )
    }

    fn with_layout(
        model: Arc<Model>,
        model_style: Option<Style>,
        source: DiagramSource,
        layout: Layout,
        style: DiagramStyle,
        name: Option<String>,
    ) -> Self {
        let mut diagram = Self {
            model,
            scene: Ok(Scene::default()),
            kind: None,
            source,
            saved: (layout.clone(), style.clone()),
            layout,
            style,
            model_style,
            name,
            dirty: false,
            undo: Vec::new(),
            redo: Vec::new(),
            drag_step: None,
            selection: Selection::default(),
        };
        diagram.redraw();
        diagram
    }

    /// Draws from a newly loaded Model and Model Style, keeping the Layout,
    /// its undo history, and the Selection's nodes that are still drawn.
    pub fn reload(&mut self, model: Arc<Model>, model_style: Option<Style>) {
        self.model = model;
        self.model_style = model_style;
        self.redraw();
    }

    /// Changes the Diagram's own Style, as an unsaved change.
    pub fn set_style(&mut self, style: DiagramStyle) {
        if style != self.style {
            self.style = style;
            self.dirty = true;
            self.redraw();
        }
    }

    /// Rebuilds the scene from the Model, Layout and Styles, keeping the
    /// Selection's nodes that are still drawn.
    fn redraw(&mut self) {
        let model = &*self.model;
        self.scene = self.source.build_scene(model, &self.layout);
        match &mut self.scene {
            Ok(scene) => {
                style::apply_styles(scene, model, self.model_style.as_ref(), &self.style);
                self.selection.keep_drawn(scene);
            }
            Err(_) => self.selection.clear(),
        }
        self.kind = self.source.kind(model);
    }

    /// What the Diagram shows: a Diagram Kind and root, or a `view`.
    pub fn source(&self) -> &DiagramSource {
        &self.source
    }

    /// The Diagram Kind it was last drawn as; a view's can change with the
    /// Model, and is unknown while the view can't be drawn.
    pub fn kind(&self) -> Option<DiagramKind> {
        self.kind
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// The Diagram's own Style, saved with it.
    pub fn style(&self) -> &DiagramStyle {
        &self.style
    }

    /// The Scene as last drawn, or why it can't be drawn.
    pub fn scene(&self) -> Result<&Scene, &DiagramError> {
        self.scene.as_ref()
    }

    /// The name it is saved under, once saved as a Diagram File.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Whether it has Layout or Style changes not yet saved.
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// A plain click: selects `key` alone, or nothing if it can't be
    /// selected.
    pub fn select_only(&mut self, key: &QualifiedName) {
        if let Ok(scene) = &self.scene {
            self.selection.only(scene, key);
        }
    }

    /// A Ctrl- or Shift-click: adds `key` to the Selection, or takes it out
    /// if it was in. A node in another box than the rest starts a new
    /// Selection.
    pub fn toggle_selected(&mut self, key: &QualifiedName) {
        if let Ok(scene) = &self.scene {
            self.selection.toggle(scene, key);
        }
    }

    /// A selection box: selects the outermost nodes lying wholly inside
    /// `area`, in scene coordinates.
    pub fn select_in(&mut self, area: Rect) {
        if let Ok(scene) = &self.scene {
            self.selection.boxed(scene, area);
        }
    }

    pub fn clear_selection(&mut self) {
        self.selection.clear();
    }

    /// Whether this is the Interconnection Diagram rooted at `element`, so
    /// opening that again would go nowhere.
    pub fn is_interconnection_of(&self, element: &QualifiedName) -> bool {
        matches!(&self.source, DiagramSource::Tool { kind: DiagramKind::Interconnection, root } if root == element)
    }

    /// The name a Diagram File or Export of it gets unless the user picks
    /// another: the saved name, else the root's or view's.
    pub fn default_name(&self) -> String {
        match (&self.name, &self.source) {
            (Some(name), _) => name.clone(),
            (None, DiagramSource::Tool { root, .. }) => root.simple_name().to_lowercase(),
            (None, DiagramSource::View { view }) => crate::view::file_name(view),
        }
    }

    /// Applies a Layout edit and redraws. Each edit that changes the
    /// Layout is an undo step, unless it is part of a drag.
    pub fn edit(&mut self, edit: &Edit) {
        let Ok(scene) = &self.scene else { return };
        let before = self.layout.clone();
        let (source, model) = (&self.source, &*self.model);
        let known = |key: &QualifiedName| source.layout_key_exists(model, key);
        editing::apply_with(&mut self.layout, &known, scene, edit);
        if *edit == Edit::Reset {
            self.selection.clear();
        }
        if self.layout != before {
            if self.drag_step != Some(true) {
                self.undo.push(before);
                if self.undo.len() > UNDO_STEPS {
                    self.undo.remove(0);
                }
                self.redo.clear();
                if self.drag_step.is_some() {
                    self.drag_step = Some(true);
                }
            }
            self.dirty = true;
            self.redraw();
        }
    }

    /// Starts a drag: its edits, until [`Self::end_drag`], are one undo
    /// step.
    pub fn begin_drag(&mut self) {
        self.drag_step = Some(false);
    }

    pub fn end_drag(&mut self) {
        self.drag_step = None;
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    /// Goes back to the Layout before the last undo step, and redraws.
    pub fn undo(&mut self) {
        if let Some(layout) = self.undo.pop() {
            self.redo.push(std::mem::replace(&mut self.layout, layout));
            self.restored();
        }
    }

    /// Goes forward again to the Layout the last undo left.
    pub fn redo(&mut self) {
        if let Some(layout) = self.redo.pop() {
            self.undo.push(std::mem::replace(&mut self.layout, layout));
            self.restored();
        }
    }

    /// After undo or redo: unsaved unless back at the saved Diagram.
    fn restored(&mut self) {
        self.dirty = (&self.layout, &self.style) != (&self.saved.0, &self.saved.1);
        self.redraw();
    }

    /// Saves as the Diagram File `name` in `model_folder`, refusing to
    /// overwrite a different saved Diagram of that name.
    pub fn save_as(&mut self, model_folder: &Path, name: &str) -> Result<(), OpenDiagramError> {
        check_name(name)?;
        let path = diagram_file::path_for(model_folder, name);
        if path.exists() && self.name.as_deref() != Some(name) {
            return Err(OpenDiagramError::NameTaken(name.to_owned()));
        }
        let mut file = DiagramFile::of(name, self.source.clone());
        file.layout = self.layout.clone();
        file.style = self.style.clone();
        std::fs::create_dir_all(path.parent().expect("a Diagram File is in a folder"))?;
        std::fs::write(&path, file.to_text())?;
        self.name = Some(name.to_owned());
        self.dirty = false;
        self.saved = (self.layout.clone(), self.style.clone());
        Ok(())
    }

    /// Writes the Export: the SVG of the Diagram as it is drawn.
    pub fn export_svg(&self, path: &Path) -> Result<(), OpenDiagramError> {
        let scene = self
            .scene
            .as_ref()
            .map_err(|e| OpenDiagramError::NotDrawn(e.clone()))?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, svg::render(scene))?;
        Ok(())
    }
}

/// The nodes of one Diagram picked to move or arrange together, by node
/// key (see [`crate::scene::Depicts::node_key`]), in the order they were
/// picked. They are all drawn in the same box; the Interconnection Frame
/// is never in one.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Selection(Vec<QualifiedName>);

impl Selection {
    pub fn keys(&self) -> &[QualifiedName] {
        &self.0
    }

    pub fn contains(&self, key: &QualifiedName) -> bool {
        self.0.contains(key)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn clear(&mut self) {
        self.0.clear();
    }

    /// A plain click: `key` alone, or nothing if it can't be selected.
    fn only(&mut self, scene: &Scene, key: &QualifiedName) {
        self.0.clear();
        if scene.movable(key) {
            self.0.push(key.clone());
        }
    }

    /// A Ctrl- or Shift-click: adds `key`, or takes it out if it was in.
    /// A node in another box than the rest starts a new Selection.
    fn toggle(&mut self, scene: &Scene, key: &QualifiedName) {
        if let Some(index) = self.0.iter().position(|k| k == key) {
            self.0.remove(index);
        } else if self
            .0
            .first()
            .is_some_and(|k| scene.container_of(k) != scene.container_of(key))
        {
            self.only(scene, key);
        } else if scene.movable(key) {
            self.0.push(key.clone());
        }
    }

    /// A selection box: the nodes lying wholly inside `area` (in scene
    /// coordinates). Nodes inside a box that is in the area move with it,
    /// so only the outermost ones count; where several boxes' nodes are
    /// equally outermost, those of the first in paint order.
    fn boxed(&mut self, scene: &Scene, area: Rect) {
        let inside = |r: Rect| {
            r.x >= area.x
                && r.y >= area.y
                && r.right() <= area.right()
                && r.bottom() <= area.bottom()
        };
        let candidates: Vec<&QualifiedName> = scene
            .elements()
            .into_iter()
            .filter(|k| scene.movable(k) && scene.node_bounds(k).is_some_and(inside))
            .collect();
        let depth = |k: &QualifiedName| scene.container_of(k).map_or(0, |c| c.split('.').count());
        let outermost = candidates.iter().map(|k| depth(k)).min();
        let first = candidates
            .iter()
            .find(|k| Some(depth(k)) == outermost)
            .map(|k| scene.container_of(k));
        self.0 = candidates
            .into_iter()
            .filter(|k| Some(scene.container_of(k)) == first)
            .cloned()
            .collect();
    }

    /// Drops the nodes that are no longer drawn, such as hidden ones.
    fn keep_drawn(&mut self, scene: &Scene) {
        let drawn = scene.elements();
        self.0.retain(|k| drawn.contains(&k));
    }
}

/// Diagram names become file names, so they are limited to letters, digits,
/// `-`, `_` and `.`, and may not start with `.`.
pub fn check_name(name: &str) -> Result<(), OpenDiagramError> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.');
    if name.is_empty() || name.starts_with('.') || !name.chars().all(allowed) {
        return Err(OpenDiagramError::NameNotAllowed(name.to_owned()));
    }
    Ok(())
}
