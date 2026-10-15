//! Open Diagrams and the egui renderer that paints their scenes. Painting
//! adds no notation of its own: it draws the scene's primitives, the same
//! ones the SVG renderer writes, on a white page like the Export.

use std::path::Path;

use eframe::egui::{self, Align2, Color32, FontId, Pos2, Stroke, StrokeKind, Vec2};
use viewer_core::scene::{Fill, Point, Shape, TextAnchor};
use viewer_core::editing::{self, Edit};
use viewer_core::{
    DiagramError, DiagramFile, DiagramKind, Layout, Model, QualifiedName, Scene, build_scene_with_layout, diagram_file,
    svg,
};

const STROKE: Color32 = Color32::BLACK;
const PAGE: Color32 = Color32::WHITE;
const HIGHLIGHT: Color32 = Color32::from_rgb(0x1f, 0x6f, 0xeb);
const MIN_ZOOM: f32 = 0.1;
const MAX_ZOOM: f32 = 8.0;
/// Text sits on its baseline; egui anchors by box, so drop it by about the
/// font's descent.
const DESCENT: f32 = 0.22;

/// A Diagram open in the viewer: what it shows, the user's Layout, and how
/// it is viewed. `name` is set once it has been saved as a Diagram File;
/// until then it is an ad hoc Diagram that leaves no file behind.
pub struct OpenDiagram {
    pub kind: DiagramKind,
    pub root: QualifiedName,
    pub layout: Layout,
    pub name: Option<String>,
    /// Layout changes not yet saved.
    pub dirty: bool,
    pub scene: Result<Scene, DiagramError>,
    pub view: View,
    /// Fit the Diagram to the canvas on the next paint.
    pub needs_fit: bool,
}

impl OpenDiagram {
    pub fn open(model: &Model, kind: DiagramKind, root: QualifiedName) -> Self {
        Self::with_layout(model, kind, root, Layout::default(), None)
    }

    /// Opens a saved Diagram File.
    pub fn open_saved(model: &Model, file: DiagramFile) -> Self {
        Self::with_layout(model, file.kind, file.root, file.layout, Some(file.name))
    }

    fn with_layout(model: &Model, kind: DiagramKind, root: QualifiedName, layout: Layout, name: Option<String>) -> Self {
        Self {
            scene: build_scene_with_layout(model, kind, &root, &layout),
            kind,
            root,
            layout,
            name,
            dirty: false,
            view: View::default(),
            needs_fit: true,
        }
    }

    /// Rebuilds the scene from a newly loaded Model, keeping the Layout
    /// and the view.
    pub fn refresh(&mut self, model: &Model) {
        self.scene = build_scene_with_layout(model, self.kind, &self.root, &self.layout);
    }

    /// Applies a Layout edit and redraws.
    pub fn edit(&mut self, model: &Model, edit: &Edit) {
        let Ok(scene) = &self.scene else { return };
        let before = self.layout.clone();
        editing::apply(&mut self.layout, model, scene, edit);
        if self.layout != before {
            self.dirty = true;
            self.refresh(model);
        }
    }

    /// The tab title: the saved name, or the root for an ad hoc Diagram,
    /// marked while there are unsaved changes.
    pub fn title(&self) -> String {
        let title = match (&self.name, self.kind) {
            (Some(name), _) => name.clone(),
            (None, DiagramKind::General) => format!("{} (General)", self.root.simple_name()),
        };
        if self.dirty { format!("{title} •") } else { title }
    }

    /// Saves as the Diagram File `name` in `model_folder`, refusing to
    /// overwrite a different saved Diagram of that name.
    pub fn save_as(&mut self, model_folder: &Path, name: &str) -> Result<(), String> {
        check_name(name)?;
        let path = diagram_file::path_for(model_folder, name);
        if path.exists() && self.name.as_deref() != Some(name) {
            return Err(format!("a Diagram named `{name}` already exists"));
        }
        let mut file = DiagramFile::new(name, self.kind, self.root.clone());
        file.layout = self.layout.clone();
        std::fs::create_dir_all(path.parent().expect("a Diagram File is in a folder")).map_err(|e| e.to_string())?;
        std::fs::write(&path, file.to_text()).map_err(|e| e.to_string())?;
        self.name = Some(name.to_owned());
        self.dirty = false;
        Ok(())
    }

    /// Writes the Export: the same SVG the command line writes for this
    /// Diagram.
    pub fn export_svg(&self, path: &Path) -> Result<(), String> {
        let scene = self.scene.as_ref().map_err(|e| e.to_string())?;
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        std::fs::write(path, svg::render(scene)).map_err(|e| e.to_string())
    }
}

/// Diagram names become file names, so they are limited to letters, digits,
/// `-`, `_` and `.`, and may not start with `.`.
pub fn check_name(name: &str) -> Result<(), String> {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.');
    if name.is_empty() || name.starts_with('.') || !name.chars().all(allowed) {
        return Err(format!("`{name}` is not a valid Diagram name: use letters, digits, `-`, `_` and `.`"));
    }
    Ok(())
}

/// Pan and zoom: a scene point `p` is drawn at `origin + pan + p * zoom`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub pan: Vec2,
    pub zoom: f32,
}

impl Default for View {
    fn default() -> Self {
        Self { pan: Vec2::ZERO, zoom: 1.0 }
    }
}

impl View {
    pub fn to_screen(&self, origin: Pos2, p: Point) -> Pos2 {
        origin + self.pan + Vec2::new(p.x, p.y) * self.zoom
    }

    pub fn to_scene(&self, origin: Pos2, pos: Pos2) -> Point {
        let v = (pos - origin - self.pan) / self.zoom;
        Point::new(v.x, v.y)
    }

    /// Zooms by `factor`, keeping the scene point under `pointer` in place.
    pub fn zoom_at(&mut self, origin: Pos2, pointer: Pos2, factor: f32) {
        let anchor = self.to_scene(origin, pointer);
        self.zoom = (self.zoom * factor).clamp(MIN_ZOOM, MAX_ZOOM);
        self.pan = pointer - origin - Vec2::new(anchor.x, anchor.y) * self.zoom;
    }

    /// Fits a page at `top_left` of `size` into `available`, centered,
    /// never enlarging.
    pub fn fit(top_left: Vec2, size: Vec2, available: Vec2) -> Self {
        let zoom = (available.x / size.x).min(available.y / size.y).clamp(MIN_ZOOM, 1.0);
        Self { pan: (available - size * zoom) / 2.0 - top_left * zoom, zoom }
    }
}

/// Paints `scene` with its top-left at `origin + view.pan`, outlining the
/// node of `highlight`.
pub fn paint(painter: &egui::Painter, scene: &Scene, view: View, origin: Pos2, highlight: Option<&QualifiedName>) {
    let z = view.zoom;
    let at = |p: Point| view.to_screen(origin, p);
    let stroke = Stroke::new(z.max(0.5), STROKE);
    let fill = |f: Fill| match f {
        Fill::None => Color32::TRANSPARENT,
        Fill::Background => PAGE,
        Fill::Stroke => STROKE,
    };

    let page = egui::Rect::from_min_size(at(Point::new(scene.x, scene.y)), Vec2::new(scene.width, scene.height) * z);
    painter.rect_filled(page, 0.0, PAGE);

    for item in &scene.items {
        match &item.shape {
            Shape::Rect { rect, fill: f, corner_radius } => {
                let r = egui::Rect::from_min_size(at(Point::new(rect.x, rect.y)), Vec2::new(rect.width, rect.height) * z);
                painter.rect(r, corner_radius * z, fill(*f), stroke, StrokeKind::Middle);
            }
            Shape::Polyline { points, dashed } => {
                let points: Vec<Pos2> = points.iter().map(|p| at(*p)).collect();
                if *dashed {
                    painter.extend(egui::Shape::dashed_line(&points, stroke, 6.0 * z, 4.0 * z));
                } else {
                    painter.line(points, stroke);
                }
            }
            Shape::Polygon { points, fill: f } => {
                let points: Vec<Pos2> = points.iter().map(|p| at(*p)).collect();
                painter.add(egui::Shape::convex_polygon(points, fill(*f), stroke));
            }
            Shape::Circle { center, radius, fill: f } => {
                painter.circle_filled(at(*center), radius * z, fill(*f));
            }
            Shape::Text { position, text, anchor, font } => {
                let size = font.size * z;
                let pos = at(*position) + Vec2::new(0.0, size * DESCENT);
                let align = match anchor {
                    TextAnchor::Start => Align2::LEFT_BOTTOM,
                    TextAnchor::Middle => Align2::CENTER_BOTTOM,
                    TextAnchor::End => Align2::RIGHT_BOTTOM,
                };
                let mut job = egui::text::LayoutJob::single_section(
                    text.clone(),
                    egui::TextFormat { font_id: FontId::proportional(size), color: STROKE, italics: font.italic, ..Default::default() },
                );
                job.wrap.max_width = f32::INFINITY;
                let galley = painter.layout_job(job);
                let rect = align.anchor_size(pos, galley.size());
                painter.galley(rect.min, galley.clone(), STROKE);
                // The default font has no bold face; overprint to embolden.
                if font.bold {
                    painter.galley(rect.min + Vec2::new(0.6 * z.max(0.5), 0.0), galley, STROKE);
                }
            }
        }
    }

    if let Some(bounds) = highlight.and_then(|e| scene.node_bounds(e)) {
        let r = egui::Rect::from_min_size(at(Point::new(bounds.x, bounds.y)), Vec2::new(bounds.width, bounds.height) * z);
        painter.rect_stroke(r, 2.0, Stroke::new(2.0, HIGHLIGHT), StrokeKind::Outside);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zooming_keeps_the_point_under_the_pointer_in_place() {
        let origin = Pos2::new(100.0, 50.0);
        let pointer = Pos2::new(400.0, 300.0);
        let mut view = View { pan: Vec2::new(20.0, -10.0), zoom: 1.5 };
        let before = view.to_scene(origin, pointer);

        view.zoom_at(origin, pointer, 1.7);

        let after = view.to_scene(origin, pointer);
        assert!((before.x - after.x).abs() < 1e-3 && (before.y - after.y).abs() < 1e-3);
        assert!((view.zoom - 2.55).abs() < 1e-4);
    }

    #[test]
    fn screen_and_scene_coordinates_round_trip() {
        let view = View { pan: Vec2::new(12.0, 34.0), zoom: 0.75 };
        let origin = Pos2::new(5.0, 6.0);

        let p = view.to_scene(origin, view.to_screen(origin, Point::new(80.0, 120.0)));

        assert!((p.x - 80.0).abs() < 1e-4 && (p.y - 120.0).abs() < 1e-4);
    }

    #[test]
    fn fitting_centers_the_scene_and_never_enlarges_it() {
        let small = View::fit(Vec2::ZERO, Vec2::new(200.0, 100.0), Vec2::new(1000.0, 500.0));
        assert_eq!(small.zoom, 1.0);
        assert_eq!(small.pan, Vec2::new(400.0, 200.0));

        let large = View::fit(Vec2::ZERO, Vec2::new(2000.0, 500.0), Vec2::new(1000.0, 500.0));
        assert_eq!(large.zoom, 0.5);
        assert_eq!(large.pan, Vec2::new(0.0, 125.0));

        let shifted = View::fit(Vec2::new(-40.0, -10.0), Vec2::new(200.0, 100.0), Vec2::new(1000.0, 500.0));
        assert_eq!(shifted.pan, Vec2::new(440.0, 210.0), "the page's top-left lands where an unshifted one would");
    }

    #[test]
    fn zoom_stays_within_limits() {
        let mut view = View::default();
        view.zoom_at(Pos2::ZERO, Pos2::ZERO, 1000.0);
        assert_eq!(view.zoom, MAX_ZOOM);
        view.zoom_at(Pos2::ZERO, Pos2::ZERO, 1e-6);
        assert_eq!(view.zoom, MIN_ZOOM);
    }
}
