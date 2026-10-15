//! The Standard Notation: how nodes and edges are drawn as scene primitives.

use crate::model::Element;
use crate::scene::{Depicts, Fill, Font, Item, Point, Rect, Shape, TextAnchor};

const KEYWORD_FONT: Font = Font { size: 12.0, bold: false };
const NAME_FONT: Font = Font { size: 14.0, bold: true };
const PADDING: f32 = 10.0;
const LINE_GAP: f32 = 4.0;
const MIN_NODE_WIDTH: f32 = 120.0;
const ARROW_LENGTH: f32 = 14.0;
const ARROW_HALF_WIDTH: f32 = 8.0;

/// Text width estimated from character count, so layout never depends on
/// the fonts installed on the machine doing the Export.
fn text_width(text: &str, font: Font) -> f32 {
    let average = if font.bold { 0.62 } else { 0.56 };
    text.chars().count() as f32 * font.size * average
}

fn keyword_label(element: &Element) -> String {
    format!("«{}»", element.kind.keyword())
}

pub fn node_size(element: &Element) -> (f32, f32) {
    let content = text_width(&keyword_label(element), KEYWORD_FONT)
        .max(text_width(&element.name, NAME_FONT));
    let width = (content + 2.0 * PADDING).max(MIN_NODE_WIDTH).ceil();
    let height = (2.0 * PADDING + KEYWORD_FONT.size + LINE_GAP + NAME_FONT.size).ceil();
    (width, height)
}

/// A definition node: a square-cornered box with the keyword in guillemets
/// above the bold name.
pub fn node(element: &Element, rect: Rect) -> Vec<Item> {
    let depicts = Depicts::Element(element.qualified_name.clone());
    let keyword_baseline = rect.y + PADDING + KEYWORD_FONT.size;
    let name_baseline = keyword_baseline + LINE_GAP + NAME_FONT.size;
    [
        Shape::Rect { rect, fill: Fill::Background },
        Shape::Text {
            position: Point::new(rect.center_x(), keyword_baseline),
            text: keyword_label(element),
            anchor: TextAnchor::Middle,
            font: KEYWORD_FONT,
        },
        Shape::Text {
            position: Point::new(rect.center_x(), name_baseline),
            text: element.name.clone(),
            anchor: TextAnchor::Middle,
            font: NAME_FONT,
        },
    ]
    .into_iter()
    .map(|shape| Item { depicts: depicts.clone(), shape })
    .collect()
}

/// A specialization edge from the specific node up to the general node,
/// ending in a hollow triangle on the general node's border.
pub fn specialization(depicts: Depicts, specific: Rect, general: Rect) -> Vec<Item> {
    let (start, tip) = if specific.y >= general.bottom() {
        (
            Point::new(specific.center_x(), specific.y),
            Point::new(general.center_x(), general.bottom()),
        )
    } else {
        (
            Point::new(specific.center_x(), specific.bottom()),
            Point::new(general.center_x(), general.y),
        )
    };
    let (dx, dy) = (tip.x - start.x, tip.y - start.y);
    let length = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    let (ux, uy) = (dx / length, dy / length);
    let base = Point::new(tip.x - ux * ARROW_LENGTH, tip.y - uy * ARROW_LENGTH);
    let (nx, ny) = (-uy * ARROW_HALF_WIDTH, ux * ARROW_HALF_WIDTH);
    [
        Shape::Polyline { points: vec![start, base] },
        Shape::Polygon {
            points: vec![
                tip,
                Point::new(base.x + nx, base.y + ny),
                Point::new(base.x - nx, base.y - ny),
            ],
            fill: Fill::Background,
        },
    ]
    .into_iter()
    .map(|shape| Item { depicts: depicts.clone(), shape })
    .collect()
}
