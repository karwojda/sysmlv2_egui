//! The Standard Notation: how nodes and edges are drawn as scene primitives.

use crate::model::{Element, ElementKind, Model};
use crate::scene::{Depicts, Fill, Font, Item, Point, Rect, Shape, TextAnchor};

pub(crate) const KEYWORD_FONT: Font = Font { size: 12.0, bold: false, italic: false };
pub(crate) const NAME_FONT: Font = Font { size: 14.0, bold: true, italic: false };
/// Usage titles are not bold, unlike definition names.
pub(crate) const USAGE_NAME_FONT: Font = Font { size: 14.0, bold: false, italic: false };
const COMPARTMENT_TITLE_FONT: Font = Font { size: 11.0, bold: false, italic: true };
const ROW_FONT: Font = Font { size: 12.0, bold: false, italic: false };
pub(crate) const LABEL_FONT: Font = Font { size: 11.0, bold: false, italic: false };
pub(crate) const PADDING: f32 = 10.0;
pub(crate) const LINE_GAP: f32 = 4.0;
pub(crate) const MIN_NODE_WIDTH: f32 = 120.0;
pub(crate) const USAGE_CORNER_RADIUS: f32 = 10.0;
const ARROW_LENGTH: f32 = 14.0;
const ARROW_HALF_WIDTH: f32 = 8.0;
const DIAMOND_LENGTH: f32 = 18.0;
const DIAMOND_HALF_WIDTH: f32 = 6.0;
pub(crate) const LABEL_OFFSET: f32 = 6.0;
/// How far apart edge ends on the same side of a node must be.
const END_SEPARATION: f32 = 14.0;
/// How far edges keep from nodes they pass, and how a detour searches.
const OBSTACLE_CLEARANCE: f32 = 4.0;
const DETOUR_STEP: f32 = 24.0;
const DETOUR_STEPS: usize = 16;
/// How far outside an obstacle's corner a corner bend sits.
const CORNER_CLEARANCE: f32 = 14.0;
/// Sideways distance between parallel edges joining the same two nodes.
const PARALLEL_SPREAD: f32 = 36.0;
const COLON_DOT_RADIUS: f32 = 1.8;
/// How far the defined-by colon sits before the triangle, and how far
/// apart its two dots are.
const COLON_GAP: f32 = 5.0;
const COLON_HALF_SPREAD: f32 = 4.0;

/// Text width estimated from character count, so layout never depends on
/// the fonts installed on the machine doing the Export.
pub(crate) fn text_width(text: &str, font: Font) -> f32 {
    let average = if font.bold { 0.62 } else { 0.56 };
    text.chars().count() as f32 * font.size * average
}

/// The area `shape` covers, with text estimated like node sizes are.
pub fn extent(shape: &Shape) -> Rect {
    let around = |points: &[Point], pad: f32| {
        let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min) - pad;
        let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) - pad;
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max) + pad;
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max) + pad;
        Rect { x: min_x, y: min_y, width: max_x - min_x, height: max_y - min_y }
    };
    match shape {
        Shape::Rect { rect, .. } => *rect,
        Shape::Polyline { points, .. } | Shape::Polygon { points, .. } => around(points, 0.0),
        Shape::Circle { center, radius, .. } => around(&[*center], *radius),
        Shape::Text { position, text, anchor, font } => {
            let width = text_width(text, *font);
            let x = match anchor {
                TextAnchor::Start => position.x,
                TextAnchor::Middle => position.x - width / 2.0,
                TextAnchor::End => position.x - width,
            };
            Rect { x, y: position.y - font.size, width, height: font.size * 1.3 }
        }
    }
}

/// What a node shows: a header, then titled compartments of feature rows.
pub struct NodeContent {
    keyword: String,
    title: String,
    usage: bool,
    compartments: Vec<(&'static str, Vec<String>)>,
}

/// The compartments a node lists, in order, with the kinds of owned
/// feature each one holds.
const COMPARTMENTS: [(&str, &[ElementKind]); 4] = [
    ("attributes", &[ElementKind::AttributeUsage]),
    ("ports", &[ElementKind::PortUsage]),
    ("parts", &[ElementKind::PartUsage]),
    ("connections", &[ElementKind::ConnectionUsage, ElementKind::InterfaceUsage]),
];

/// What `element`'s node shows; without `compartments` (collapsed), only
/// its header.
pub fn node_content(element: &Element, model: &Model, compartments: bool) -> NodeContent {
    let usage = !element.kind.is_definition();
    let compartments = COMPARTMENTS
        .iter()
        .filter(|_| compartments)
        .filter_map(|(title, kinds)| {
            let rows: Vec<String> = model
                .owned_members(&element.qualified_name)
                .filter(|m| kinds.contains(&m.kind))
                .map(feature_row)
                .collect();
            (!rows.is_empty()).then_some((*title, rows))
        })
        .collect();
    NodeContent {
        keyword: format!("«{}»", element.kind.keyword()),
        title: if usage { usage_label(element) } else { element.name.clone() },
        usage,
        compartments,
    }
}

/// `name:Type [multiplicity]`, with `~` for a conjugated type.
pub(crate) fn usage_label(element: &Element) -> String {
    let mut label = element.name.clone();
    if let Some(definition) = &element.typed_by {
        let conjugate = if element.conjugated { "~" } else { "" };
        label.push_str(&format!(":{conjugate}{}", definition.simple_name()));
    }
    if let Some(multiplicity) = &element.multiplicity {
        label.push_str(&format!(" {multiplicity}"));
    }
    label
}

/// A compartment row: ports lead with their direction, connections trail
/// with their ends.
fn feature_row(feature: &Element) -> String {
    let mut row = String::new();
    if let Some(direction) = feature.direction {
        row.push_str(direction.keyword());
        row.push(' ');
    }
    row.push_str(&usage_label(feature));
    if feature.kind.is_connection() && !feature.ends.is_empty() {
        let ends: Vec<String> = feature.ends.iter().map(ToString::to_string).collect();
        row.push_str(&format!(" ({})", ends.join(" → ")));
    }
    row
}

fn title_font(content: &NodeContent) -> Font {
    if content.usage { USAGE_NAME_FONT } else { NAME_FONT }
}

fn header_height() -> f32 {
    2.0 * PADDING + KEYWORD_FONT.size + LINE_GAP + NAME_FONT.size
}

fn compartment_height(rows: usize) -> f32 {
    PADDING + COMPARTMENT_TITLE_FONT.size + rows as f32 * (LINE_GAP + ROW_FONT.size) + PADDING
}

pub fn node_size(content: &NodeContent) -> (f32, f32) {
    let mut widest = text_width(&content.keyword, KEYWORD_FONT).max(text_width(&content.title, title_font(content)));
    for (title, rows) in &content.compartments {
        widest = widest.max(text_width(title, COMPARTMENT_TITLE_FONT));
        for row in rows {
            widest = widest.max(text_width(row, ROW_FONT));
        }
    }
    let width = (widest + 2.0 * PADDING).max(MIN_NODE_WIDTH).ceil();
    let height = header_height()
        + content
            .compartments
            .iter()
            .map(|(_, rows)| compartment_height(rows.len()))
            .sum::<f32>();
    (width, height.ceil())
}

/// A node: square corners for definitions, rounded for usages; the keyword
/// in guillemets above the bold title; then each compartment under a
/// separator, with an italic title and left-aligned rows.
pub fn node(element: &Element, content: &NodeContent, rect: Rect) -> Vec<Item> {
    let depicts = Depicts::Element(element.qualified_name.clone());
    let corner_radius = if content.usage { USAGE_CORNER_RADIUS } else { 0.0 };
    let keyword_baseline = rect.y + PADDING + KEYWORD_FONT.size;
    let mut shapes = vec![
        Shape::Rect { rect, fill: Fill::Background, corner_radius },
        Shape::Text {
            position: Point::new(rect.center_x(), keyword_baseline),
            text: content.keyword.clone(),
            anchor: TextAnchor::Middle,
            font: KEYWORD_FONT,
        },
        Shape::Text {
            position: Point::new(rect.center_x(), keyword_baseline + LINE_GAP + NAME_FONT.size),
            text: content.title.clone(),
            anchor: TextAnchor::Middle,
            font: title_font(content),
        },
    ];
    let mut top = rect.y + header_height();
    for (title, rows) in &content.compartments {
        shapes.push(Shape::Polyline {
            points: vec![Point::new(rect.x, top), Point::new(rect.right(), top)],
            dashed: false,
        });
        let mut baseline = top + PADDING + COMPARTMENT_TITLE_FONT.size;
        shapes.push(Shape::Text {
            position: Point::new(rect.center_x(), baseline),
            text: (*title).to_owned(),
            anchor: TextAnchor::Middle,
            font: COMPARTMENT_TITLE_FONT,
        });
        for row in rows {
            baseline += LINE_GAP + ROW_FONT.size;
            shapes.push(Shape::Text {
                position: Point::new(rect.x + PADDING, baseline),
                text: row.clone(),
                anchor: TextAnchor::Start,
                font: ROW_FONT,
            });
        }
        top += compartment_height(rows.len());
    }
    tag(depicts, shapes)
}

/// Specialization: a solid line from the specific node ending in a hollow
/// triangle on the general node.
pub fn specialization(depicts: Depicts, path: Vec<Point>) -> Vec<Item> {
    let (before, end) = last_segment(&path);
    let mut line = path.clone();
    *line.last_mut().expect("a route has two points") = back_from(end, before, ARROW_LENGTH);
    tag(depicts, vec![Shape::Polyline { points: line, dashed: false }, triangle(end, before)])
}

/// Defined by: a solid line from the usage ending in a hollow triangle on
/// its definition, with a colon across the line just before the triangle.
pub fn defined_by(depicts: Depicts, path: Vec<Point>) -> Vec<Item> {
    let (before, end) = last_segment(&path);
    let base = back_from(end, before, ARROW_LENGTH);
    let colon = back_from(base, before, COLON_GAP);
    let (nx, ny) = normal(before, end, COLON_HALF_SPREAD);
    let dot = |sign: f32| Shape::Circle {
        center: Point::new(colon.x + sign * nx, colon.y + sign * ny),
        radius: COLON_DOT_RADIUS,
        fill: Fill::Stroke,
    };
    let mut line = path.clone();
    *line.last_mut().expect("a route has two points") = base;
    tag(depicts, vec![Shape::Polyline { points: line, dashed: false }, triangle(end, before), dot(1.0), dot(-1.0)])
}

/// Composition: a solid line from the owner, starting in a filled diamond,
/// to the part's definition, labelled with the part's name and multiplicity
/// near the definition.
pub fn composition(depicts: Depicts, usage: &Element, path: Vec<Point>) -> Vec<Item> {
    let (start, after) = (path[0], path[1]);
    let diamond_back = back_from(start, after, DIAMOND_LENGTH);
    let middle = back_from(start, after, DIAMOND_LENGTH / 2.0);
    let (nx, ny) = normal(start, after, DIAMOND_HALF_WIDTH);
    let mut label = usage.name.clone();
    if let Some(multiplicity) = &usage.multiplicity {
        label.push_str(&format!(" {multiplicity}"));
    }
    let (before, end) = last_segment(&path);
    let near_end = back_from(end, before, 2.0 * LABEL_OFFSET + LABEL_FONT.size);
    let mut line = path.clone();
    line[0] = diamond_back;
    tag(depicts, vec![
        Shape::Polyline { points: line, dashed: false },
        Shape::Polygon {
            points: vec![
                start,
                Point::new(middle.x + nx, middle.y + ny),
                diamond_back,
                Point::new(middle.x - nx, middle.y - ny),
            ],
            fill: Fill::Stroke,
        },
        Shape::Text {
            position: Point::new(near_end.x + LABEL_OFFSET, near_end.y),
            text: label,
            anchor: TextAnchor::Start,
            font: LABEL_FONT,
        },
    ])
}

/// A connection or interface: a plain solid line between the usages it
/// joins, labelled with its name and type. A straight one is labelled above
/// its middle; a bent one beside its middle bend, on the outer side, so
/// fanned-out parallel connections keep their labels apart.
pub fn connection(depicts: Depicts, element: &Element, path: Vec<Point>) -> Vec<Item> {
    let (first, last) = (path[0], path[path.len() - 1]);
    let chord_middle = Point::new((first.x + last.x) / 2.0, (first.y + last.y) / 2.0);
    let (position, anchor) = if path.len() % 2 == 1 {
        let bend = path[path.len() / 2];
        let (dx, dy) = (bend.x - chord_middle.x, bend.y - chord_middle.y);
        if dx.abs() < 0.5 && dy.abs() < 0.5 {
            (Point::new(bend.x, bend.y - LABEL_OFFSET), TextAnchor::Middle)
        } else if dx.abs() >= dy.abs() {
            let side = dx.signum() * LABEL_OFFSET;
            let anchor = if dx > 0.0 { TextAnchor::Start } else { TextAnchor::End };
            (Point::new(bend.x + side, bend.y + LABEL_FONT.size / 3.0), anchor)
        } else if dy > 0.0 {
            (Point::new(bend.x, bend.y + LABEL_OFFSET + LABEL_FONT.size), TextAnchor::Middle)
        } else {
            (Point::new(bend.x, bend.y - LABEL_OFFSET), TextAnchor::Middle)
        }
    } else {
        let segment = (path.len() - 1) / 2;
        let (a, b) = (path[segment], path[segment + 1]);
        (Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0 - LABEL_OFFSET), TextAnchor::Middle)
    };
    tag(depicts, vec![
        Shape::Polyline { points: path, dashed: false },
        Shape::Text { position, text: usage_label(element), anchor, font: LABEL_FONT },
    ])
}

/// Bends that fan out `count` parallel edges between `a` and `b`: edge
/// `index` bends at the middle, offset sideways from the straight line.
/// The middle edge of an odd count stays straight.
pub fn fan_bends(a: Rect, b: Rect, index: usize, count: usize) -> Vec<Point> {
    let offset = (index as f32 - (count as f32 - 1.0) / 2.0) * PARALLEL_SPREAD;
    if offset == 0.0 {
        return Vec::new();
    }
    let (ca, cb) = (center(a), center(b));
    let (nx, ny) = normal(ca, cb, offset);
    vec![Point::new((ca.x + cb.x) / 2.0 + nx, (ca.y + cb.y) / 2.0 + ny)]
}

/// The path of an edge from `from` to `to` through `bends`: it leaves
/// `from`'s border heading for the first bend (or `to`'s center) and enters
/// `to`'s border from the last bend (or `from`'s center).
pub fn route(from: Rect, to: Rect, bends: &[Point]) -> Vec<Point> {
    let first_target = bends.first().copied().unwrap_or(center(to));
    let last_source = bends.last().copied().unwrap_or(center(from));
    let mut path = vec![border_point(from, first_target)];
    path.extend_from_slice(bends);
    path.push(border_point(to, last_source));
    path
}

/// A bend that takes the straight edge from `from` to `to` around the
/// rects in `obstacles` it would otherwise cross. Candidates are points
/// sideways from the middle and points just off each obstacle's corners;
/// the shortest that crosses nothing wins, else the one crossing fewest, as
/// long as it improves on the straight edge. Empty if the straight edge is
/// already clear or nothing improves on it.
pub fn detour(from: Rect, to: Rect, obstacles: &[Rect]) -> Vec<Point> {
    let crossings = |path: &[Point]| {
        obstacles
            .iter()
            .filter(|r| path.windows(2).any(|s| crosses(s[0], s[1], **r)))
            .count()
    };
    let length = |path: &[Point]| path.windows(2).map(|s| distance(s[0], s[1])).sum::<f32>();
    let straight = crossings(&route(from, to, &[]));
    if straight == 0 {
        return Vec::new();
    }

    let (a, b) = (center(from), center(to));
    let middle = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    let mut candidates = Vec::new();
    for step in 1..=DETOUR_STEPS {
        for side in [1.0, -1.0] {
            let (nx, ny) = normal(a, b, side * step as f32 * DETOUR_STEP);
            candidates.push(Point::new(middle.x + nx, middle.y + ny));
        }
    }
    let off = CORNER_CLEARANCE;
    for r in obstacles {
        candidates.extend([
            Point::new(r.x - off, r.y - off),
            Point::new(r.right() + off, r.y - off),
            Point::new(r.x - off, r.bottom() + off),
            Point::new(r.right() + off, r.bottom() + off),
        ]);
    }
    // Bends inside the two end nodes would make no sense.
    candidates.retain(|c| !from.contains(*c) && !to.contains(*c));

    let best = |options: Vec<Vec<Point>>| {
        options
            .into_iter()
            .map(|bends| {
                let path = route(from, to, &bends);
                (crossings(&path), length(&path), bends)
            })
            .filter(|(count, _, _)| *count < straight)
            .min_by(|x, y| x.0.cmp(&y.0).then(x.1.total_cmp(&y.1)))
    };
    let single = best(candidates.iter().map(|c| vec![*c]).collect());
    if let Some((0, _, bends)) = single {
        return bends;
    }
    // A pocket can need two bends: out of it, then around what blocks it.
    let pairs = candidates
        .iter()
        .flat_map(|a| candidates.iter().filter(move |b| *b != a).map(move |b| vec![*a, *b]))
        .collect();
    match (single, best(pairs)) {
        (_, Some((0, _, bends))) => bends,
        (Some((_, _, bends)), _) => bends,
        (None, Some((_, _, bends))) => bends,
        (None, None) => Vec::new(),
    }
}

fn distance(a: Point, b: Point) -> f32 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

/// Whether the segment from `a` to `b` passes through `rect`, kept a few
/// units clear of its border (Liang–Barsky clipping).
fn crosses(a: Point, b: Point, rect: Rect) -> bool {
    let (x0, y0) = (rect.x - OBSTACLE_CLEARANCE, rect.y - OBSTACLE_CLEARANCE);
    let (x1, y1) = (rect.right() + OBSTACLE_CLEARANCE, rect.bottom() + OBSTACLE_CLEARANCE);
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (mut t0, mut t1) = (0.0_f32, 1.0_f32);
    for (p, q) in [(-dx, a.x - x0), (dx, x1 - a.x), (-dy, a.y - y0), (dy, y1 - a.y)] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

/// Spreads edge ends that land close together on the same side of `rect`,
/// so their markers don't stack. `ends` are the points to adjust; each is
/// moved along its side only.
pub fn spread_ends(rect: Rect, ends: &mut [&mut Point]) {
    let side_of = |p: &Point| {
        if (p.y - rect.y).abs() < 0.5 {
            0
        } else if (p.y - rect.bottom()).abs() < 0.5 {
            1
        } else if (p.x - rect.x).abs() < 0.5 {
            2
        } else {
            3
        }
    };
    for side in 0..4 {
        let horizontal = side < 2;
        let along = |p: &Point| if horizontal { p.x } else { p.y };
        let (low, high) = if horizontal { (rect.x, rect.right()) } else { (rect.y, rect.bottom()) };
        let mut on_side: Vec<&mut &mut Point> = ends.iter_mut().filter(|p| side_of(p) == side).collect();
        on_side.sort_by(|a, b| along(a).total_cmp(&along(b)));
        for i in 1..on_side.len() {
            let previous = along(on_side[i - 1]);
            let point = &mut on_side[i];
            if along(point) - previous < END_SEPARATION {
                let moved = (previous + END_SEPARATION).min(high).max(low);
                if horizontal { point.x = moved } else { point.y = moved }
            }
        }
    }
}

fn last_segment(path: &[Point]) -> (Point, Point) {
    (path[path.len() - 2], path[path.len() - 1])
}

fn tag(depicts: Depicts, shapes: Vec<Shape>) -> Vec<Item> {
    shapes
        .into_iter()
        .map(|shape| Item { depicts: depicts.clone(), shape })
        .collect()
}

fn center(rect: Rect) -> Point {
    Point::new(rect.center_x(), rect.y + rect.height / 2.0)
}

/// Where the ray from the center of `rect` toward `toward` crosses its
/// border. The coordinate on the crossed side is exact, so edges meet
/// borders precisely.
fn border_point(rect: Rect, toward: Point) -> Point {
    let c = center(rect);
    let (dx, dy) = (toward.x - c.x, toward.y - c.y);
    let (half_w, half_h) = (rect.width / 2.0, rect.height / 2.0);
    if dx == 0.0 && dy == 0.0 {
        return c;
    }
    if dx.abs() * half_h <= dy.abs() * half_w {
        let y = if dy > 0.0 { rect.bottom() } else { rect.y };
        Point::new(c.x + dx * (half_h / dy.abs()), y)
    } else {
        let x = if dx > 0.0 { rect.right() } else { rect.x };
        Point::new(x, c.y + dy * (half_w / dx.abs()))
    }
}

/// The point `distance` back from `tip` toward `from`.
fn back_from(tip: Point, from: Point, distance: f32) -> Point {
    let (dx, dy) = (from.x - tip.x, from.y - tip.y);
    let length = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    Point::new(tip.x + dx / length * distance, tip.y + dy / length * distance)
}

/// A vector of `length` perpendicular to the line from `a` to `b`.
fn normal(a: Point, b: Point, length: f32) -> (f32, f32) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let norm = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    (-dy / norm * length, dx / norm * length)
}

/// A hollow triangle with its tip at `tip`, pointing away from `from`.
fn triangle(tip: Point, from: Point) -> Shape {
    let base = back_from(tip, from, ARROW_LENGTH);
    let (nx, ny) = normal(from, tip, ARROW_HALF_WIDTH);
    Shape::Polygon {
        points: vec![tip, Point::new(base.x + nx, base.y + ny), Point::new(base.x - nx, base.y - ny)],
        fill: Fill::Background,
    }
}
