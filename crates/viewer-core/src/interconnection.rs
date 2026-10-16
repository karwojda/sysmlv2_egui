//! Interconnection Diagrams: a part's parts inside its Frame, wired port to
//! port, laid out top to bottom (`in` ports on top, `out` ports below).

use std::collections::BTreeMap;

use crate::diagram::{DiagramError, page};
use crate::diagram_file::Layout;
use crate::layout;
use crate::model::{Direction, Element, ElementKind, Model, QualifiedName};
use crate::notation::{
    self, KEYWORD_FONT, LABEL_FONT, LABEL_OFFSET, LINE_GAP, MIN_NODE_WIDTH, PADDING, USAGE_CORNER_RADIUS,
    USAGE_NAME_FONT, text_width,
};
use crate::scene::{Depicts, Fill, Item, Point, Rect, Scene, Shape, TextAnchor};

/// Space inside the Frame around its parts, and above them for its title.
const FRAME_PADDING: f32 = 30.0;
const FRAME_TITLE: f32 = 2.0 * PADDING + KEYWORD_FONT.size + LINE_GAP + USAGE_NAME_FONT.size;
/// Room above and below each part for its port labels.
const LABEL_ROOM: f32 = LABEL_FONT.size + 2.0 * LABEL_OFFSET;
const PORT_SIZE: f32 = 10.0;
/// Space each port takes along an edge beyond its label.
const PORT_SLOT_PADDING: f32 = 24.0;
/// How far a connector runs straight out of a port before turning.
const CONNECTOR_STUB: f32 = 16.0;
/// How close two ports must be across the connector to count as in line.
const ALIGNED: f32 = 2.0;
/// Vertical distance between stacked connector labels.
const LABEL_STACK: f32 = LABEL_FONT.size * 1.3 + 2.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
    fn outward(self) -> (f32, f32) {
        match self {
            Side::Top => (0.0, -1.0),
            Side::Bottom => (0.0, 1.0),
            Side::Left => (-1.0, 0.0),
            Side::Right => (1.0, 0.0),
        }
    }
}

/// A port on a part's border.
struct PortSlot<'a> {
    port: &'a Element,
    side: Side,
    /// Distance from the start of its side (left or top end).
    offset: f32,
}

/// A part drawn as a closed box inside the Frame.
struct PartBox<'a> {
    usage: &'a Element,
    ports: Vec<PortSlot<'a>>,
    rect: Rect,
}

/// A connection between ports of two parts.
struct Wire<'a> {
    connection: &'a Element,
    a: (usize, usize),
    b: (usize, usize),
}

/// Builds the Interconnection Diagram of `root`, a part definition or a
/// part usage typed by one.
pub fn build(model: &Model, root: &QualifiedName, _layout: &Layout) -> Result<Scene, DiagramError> {
    let unsuitable = || DiagramError::UnsuitableRoot {
        root: root.clone(),
        kind: crate::DiagramKind::Interconnection.description(),
    };
    let root_element = model.element(root).ok_or_else(|| DiagramError::RootNotFound(root.clone()))?;
    let definition = match root_element.kind {
        ElementKind::PartDefinition => root_element,
        ElementKind::PartUsage => root_element
            .typed_by
            .as_ref()
            .and_then(|d| model.element(d))
            .filter(|d| d.kind == ElementKind::PartDefinition)
            .ok_or_else(unsuitable)?,
        _ => return Err(unsuitable()),
    };

    let mut parts: Vec<PartBox> = model
        .owned_members(&definition.qualified_name)
        .filter(|e| e.kind == ElementKind::PartUsage)
        .map(|usage| PartBox { usage, ports: ports_of(model, usage), rect: Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 } })
        .collect();
    let wires = wires(model, definition, &parts);

    // Sizes, then layers top to bottom from each wire's `out` end to its
    // `in` end, with room for port labels above and below every part, and
    // below it for the stacked labels of connectors leaving it.
    let below: Vec<f32> = (0..parts.len()).map(|p| LABEL_ROOM + label_stack_below(&parts, &wires, p)).collect();
    let sizes: Vec<(f32, f32)> = parts
        .iter()
        .enumerate()
        .map(|(p, part)| {
            let (w, h) = part_size(part);
            (w, h + LABEL_ROOM + below[p])
        })
        .collect();
    let edges: Vec<(usize, usize)> = wires
        .iter()
        .filter(|w| w.a.0 != w.b.0)
        .map(|w| if flows_from_b(&parts, w) { (w.b.0, w.a.0) } else { (w.a.0, w.b.0) })
        .collect();
    let title_width = text_width(&frame_title(root_element), USAGE_NAME_FONT) + 2.0 * PADDING;
    let origin = Point::new(notation_margin() + FRAME_PADDING, notation_margin() + FRAME_TITLE + FRAME_PADDING);
    for ((part, placed), below) in parts.iter_mut().zip(layout::layered(&sizes, &edges)).zip(&below) {
        part.rect = Rect {
            x: placed.x + origin.x,
            y: placed.y + origin.y + LABEL_ROOM,
            width: placed.width,
            height: placed.height - LABEL_ROOM - below,
        };
    }
    let content_right = parts.iter().map(|p| p.rect.right()).fold(origin.x, f32::max);
    let content_bottom =
        parts.iter().zip(&below).map(|(p, below)| p.rect.bottom() + below).fold(origin.y, f32::max);
    let frame = Rect {
        x: notation_margin(),
        y: notation_margin(),
        width: (content_right + FRAME_PADDING - notation_margin()).max(title_width).max(MIN_NODE_WIDTH),
        height: content_bottom + FRAME_PADDING - notation_margin(),
    };

    place_ports(&mut parts, &wires);

    let mut items = Vec::new();
    items.extend(frame_box(root_element, frame));
    for part in &parts {
        items.extend(part_box(part));
    }
    items.extend(connectors(&parts, &wires));
    for part in &parts {
        items.extend(ports(part));
    }
    let mut rects: Vec<Rect> = parts.iter().map(|p| p.rect).collect();
    rects.push(frame);
    Ok(page(items, &rects))
}

fn notation_margin() -> f32 {
    crate::diagram::MARGIN
}

/// The ports a part shows: its type's, then any it declares itself.
fn ports_of<'a>(model: &'a Model, usage: &'a Element) -> Vec<PortSlot<'a>> {
    let typed = usage.typed_by.iter().flat_map(|d| model.owned_members(d));
    typed
        .chain(model.owned_members(&usage.qualified_name))
        .filter(|e| e.kind == ElementKind::PortUsage)
        .map(|port| PortSlot { port, side: Side::Bottom, offset: 0.0 })
        .collect()
}

/// The definition's connections and interfaces that join a port of one of
/// its parts to a port of another (or the same) part.
fn wires<'a>(model: &'a Model, definition: &'a Element, parts: &[PartBox<'a>]) -> Vec<Wire<'a>> {
    let end = |chain: &[QualifiedName]| -> Option<(usize, usize)> {
        let [part, port] = chain else { return None };
        let p = parts.iter().position(|b| &b.usage.qualified_name == part)?;
        let q = parts[p].ports.iter().position(|s| &s.port.qualified_name == port)?;
        Some((p, q))
    };
    model
        .owned_members(&definition.qualified_name)
        .filter(|e| e.kind.is_connection())
        .filter_map(|connection| match &connection.ends[..] {
            [a, b] => Some(Wire { connection, a: end(&a.path)?, b: end(&b.path)? }),
            _ => None,
        })
        .collect()
}

/// Extra room below part `p` for the labels of connectors leaving it that
/// stack because they share their ports.
fn label_stack_below(parts: &[PartBox], wires: &[Wire], p: usize) -> f32 {
    let mut counts: BTreeMap<((usize, usize), (usize, usize)), usize> = BTreeMap::new();
    for wire in wires {
        let (from, to) = if flows_from_b(parts, wire) { (wire.b, wire.a) } else { (wire.a, wire.b) };
        if from.0 == p {
            *counts.entry((from, to)).or_default() += 1;
        }
    }
    let deepest = counts.values().copied().max().unwrap_or(1).max(1);
    (deepest - 1) as f32 * LABEL_STACK
}

fn direction(parts: &[PartBox], (part, port): (usize, usize)) -> Option<Direction> {
    parts[part].ports[port].port.direction
}

/// Whether the wire's flow runs from its second end to its first.
fn flows_from_b(parts: &[PartBox], wire: &Wire) -> bool {
    direction(parts, wire.a) == Some(Direction::In) || direction(parts, wire.b) == Some(Direction::Out)
}

fn port_label(port: &Element) -> String {
    let mut label = port.name.clone();
    if let Some(definition) = &port.typed_by {
        let conjugate = if port.conjugated { "~" } else { "" };
        label.push_str(&format!(" : {conjugate}{}", definition.simple_name()));
    }
    label
}

fn part_title(usage: &Element) -> String {
    notation::usage_label(usage)
}

fn frame_title(root: &Element) -> String {
    if root.kind == ElementKind::PartUsage { notation::usage_label(root) } else { root.name.clone() }
}

/// A closed part's size: wide enough for its title and for the labels of
/// the ports on its top and bottom edges side by side.
fn part_size(part: &PartBox) -> (f32, f32) {
    let title = text_width(&part_title(part.usage), USAGE_NAME_FONT).max(text_width("«part»", KEYWORD_FONT));
    let edge = |direction: Direction| {
        let labels: Vec<f32> = part
            .ports
            .iter()
            .filter(|s| s.port.direction == Some(direction))
            .map(|s| text_width(&port_label(s.port), LABEL_FONT) + PORT_SLOT_PADDING)
            .collect();
        let widest = labels.iter().copied().fold(0.0, f32::max);
        widest * (labels.len() as f32 + 1.0)
    };
    let width = (title + 2.0 * PADDING).max(MIN_NODE_WIDTH).max(edge(Direction::In)).max(edge(Direction::Out));
    let height = 2.0 * PADDING + KEYWORD_FONT.size + LINE_GAP + USAGE_NAME_FONT.size;
    (width.ceil(), height.ceil())
}

fn center(rect: Rect) -> Point {
    Point::new(rect.center_x(), rect.y + rect.height / 2.0)
}

fn facing(from: Point, to: Point) -> Side {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dy.abs() >= dx.abs() {
        if dy >= 0.0 { Side::Bottom } else { Side::Top }
    } else if dx >= 0.0 {
        Side::Right
    } else {
        Side::Left
    }
}

/// Puts each port on a side (`in` top, `out` bottom, undirected facing the
/// other end, else bottom) and spaces each side's ports evenly in the order
/// of the parts they connect to.
fn place_ports(parts: &mut [PartBox], wires: &[Wire]) {
    let mut peers: BTreeMap<(usize, usize), Point> = BTreeMap::new();
    for wire in wires {
        peers.insert(wire.a, center(parts[wire.b.0].rect));
        peers.insert(wire.b, center(parts[wire.a.0].rect));
    }
    for (p, part) in parts.iter_mut().enumerate() {
        let here = center(part.rect);
        for (q, slot) in part.ports.iter_mut().enumerate() {
            slot.side = match (slot.port.direction, peers.get(&(p, q))) {
                (Some(Direction::In), _) => Side::Top,
                (Some(Direction::Out), _) => Side::Bottom,
                (_, Some(peer)) => facing(here, *peer),
                (_, None) => Side::Bottom,
            };
        }
        let rect = part.rect;
        for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
            let mut on_side: Vec<usize> = (0..part.ports.len()).filter(|&q| part.ports[q].side == side).collect();
            let along = |q: &usize| match (peers.get(&(p, *q)), side) {
                (Some(peer), Side::Top | Side::Bottom) => peer.x,
                (Some(peer), _) => peer.y,
                (None, _) => f32::MAX,
            };
            on_side.sort_by(|a, b| along(a).total_cmp(&along(b)).then(a.cmp(b)));
            let length = if matches!(side, Side::Top | Side::Bottom) { rect.width } else { rect.height };
            let count = on_side.len() as f32;
            for (k, q) in on_side.into_iter().enumerate() {
                part.ports[q].offset = (length * (k as f32 + 1.0) / (count + 1.0)).round();
            }
        }
    }
}

fn port_point(rect: Rect, slot: &PortSlot) -> Point {
    match slot.side {
        Side::Top => Point::new(rect.x + slot.offset, rect.y),
        Side::Bottom => Point::new(rect.x + slot.offset, rect.bottom()),
        Side::Left => Point::new(rect.x, rect.y + slot.offset),
        Side::Right => Point::new(rect.right(), rect.y + slot.offset),
    }
}

fn texts(depicts: &Depicts, rect: Rect, keyword: &str, title: String, top: f32) -> Vec<Item> {
    let keyword_baseline = top + PADDING + KEYWORD_FONT.size;
    [
        Shape::Text {
            position: Point::new(rect.center_x(), keyword_baseline),
            text: keyword.to_owned(),
            anchor: TextAnchor::Middle,
            font: KEYWORD_FONT,
        },
        Shape::Text {
            position: Point::new(rect.center_x(), keyword_baseline + LINE_GAP + USAGE_NAME_FONT.size),
            text: title,
            anchor: TextAnchor::Middle,
            font: USAGE_NAME_FONT,
        },
    ]
    .into_iter()
    .map(|shape| Item { depicts: depicts.clone(), shape })
    .collect()
}

/// The Frame: a rounded box with its title at the top.
fn frame_box(root: &Element, rect: Rect) -> Vec<Item> {
    let depicts = Depicts::Element(root.qualified_name.clone());
    let keyword = format!("«{}»", root.kind.keyword());
    let mut items = vec![Item {
        depicts: depicts.clone(),
        shape: Shape::Rect { rect, fill: Fill::Background, corner_radius: 2.0 * USAGE_CORNER_RADIUS },
    }];
    items.extend(texts(&depicts, rect, &keyword, frame_title(root), rect.y));
    items
}

/// A closed part: a rounded box with its keyword above its title.
fn part_box(part: &PartBox) -> Vec<Item> {
    let depicts = Depicts::Element(part.usage.qualified_name.clone());
    let mut items = vec![Item {
        depicts: depicts.clone(),
        shape: Shape::Rect { rect: part.rect, fill: Fill::Background, corner_radius: USAGE_CORNER_RADIUS },
    }];
    items.extend(texts(&depicts, part.rect, "«part»", part_title(part.usage), part.rect.y));
    items
}

/// Each port as a small square on the border, labelled beside it, outside
/// the box.
fn ports(part: &PartBox) -> Vec<Item> {
    let mut items = Vec::new();
    for slot in &part.ports {
        let depicts = Depicts::Port { part: part.usage.qualified_name.clone(), port: slot.port.qualified_name.clone() };
        let p = port_point(part.rect, slot);
        let square = Rect { x: p.x - PORT_SIZE / 2.0, y: p.y - PORT_SIZE / 2.0, width: PORT_SIZE, height: PORT_SIZE };
        let beside = PORT_SIZE / 2.0 + 3.0;
        let (position, anchor) = match slot.side {
            Side::Top => (Point::new(p.x + beside, p.y - LABEL_OFFSET), TextAnchor::Start),
            Side::Bottom => (Point::new(p.x + beside, p.y + LABEL_OFFSET + LABEL_FONT.size), TextAnchor::Start),
            Side::Left => (Point::new(p.x - beside, p.y - LABEL_OFFSET), TextAnchor::End),
            Side::Right => (Point::new(p.x + beside, p.y - LABEL_OFFSET), TextAnchor::Start),
        };
        items.push(Item { depicts: depicts.clone(), shape: Shape::Rect { rect: square, fill: Fill::Background, corner_radius: 0.0 } });
        items.push(Item {
            depicts,
            shape: Shape::Text { position, text: port_label(slot.port), anchor, font: LABEL_FONT },
        });
    }
    items
}

/// Right-angled connectors: straight out of each port, then across the gap
/// between them. Connectors over the same pair of ports stack their labels.
fn connectors(parts: &[PartBox], wires: &[Wire]) -> Vec<Item> {
    let mut items = Vec::new();
    let mut seen: Vec<((usize, usize), (usize, usize))> = Vec::new();
    for wire in wires {
        let end = |(p, q): (usize, usize)| {
            let slot = &parts[p].ports[q];
            let point = port_point(parts[p].rect, slot);
            let (dx, dy) = slot.side.outward();
            (point, Point::new(point.x + dx * CONNECTOR_STUB, point.y + dy * CONNECTOR_STUB), slot.side)
        };
        let (a, a1, side_a) = end(wire.a);
        let (b, b1, side_b) = end(wire.b);
        let vertical = |s: Side| matches!(s, Side::Top | Side::Bottom);
        // Ports almost in line get a straight connector, not a tiny jog.
        let points = if vertical(side_a) && vertical(side_b) && (a.x - b.x).abs() < ALIGNED {
            vec![a, Point::new(a.x, b.y)]
        } else if !vertical(side_a) && !vertical(side_b) && (a.y - b.y).abs() < ALIGNED {
            vec![a, Point::new(b.x, a.y)]
        } else if vertical(side_a) && vertical(side_b) {
            let middle = (a1.y + b1.y) / 2.0;
            vec![a, a1, Point::new(a1.x, middle), Point::new(b1.x, middle), b1, b]
        } else {
            let middle = (a1.x + b1.x) / 2.0;
            vec![a, a1, Point::new(middle, a1.y), Point::new(middle, b1.y), b1, b]
        };
        // Labels of connectors over the same ports stack, centred on the
        // connector's middle.
        let pair = (wire.a.min(wire.b), wire.a.max(wire.b));
        let index = seen.iter().filter(|p| **p == pair).count() as f32;
        seen.push(pair);
        let count = wires.iter().filter(|w| (w.a.min(w.b), w.a.max(w.b)) == pair).count() as f32;
        let stacked = index - (count - 1.0) / 2.0;
        // The label sits beside the middle of the connector's longest run.
        let (from, to) = points
            .windows(2)
            .map(|s| (s[0], s[1]))
            .max_by(|x, y| seg_len(*x).total_cmp(&seg_len(*y)))
            .expect("a connector has segments");
        let middle = Point::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
        // Beside a vertical run; above a horizontal one, stacking upward.
        let (position, anchor) = if from.x == to.x {
            (Point::new(middle.x + LABEL_OFFSET, middle.y + LABEL_FONT.size / 3.0 + stacked * LABEL_STACK), TextAnchor::Start)
        } else {
            (Point::new(middle.x, middle.y - LABEL_OFFSET + stacked * LABEL_STACK - (count - 1.0) / 2.0 * LABEL_STACK), TextAnchor::Middle)
        };
        let depicts = Depicts::Connection(wire.connection.qualified_name.clone());
        items.push(Item { depicts: depicts.clone(), shape: Shape::Polyline { points, dashed: false } });
        items.push(Item {
            depicts,
            shape: Shape::Text { position, text: notation::usage_label(wire.connection), anchor, font: LABEL_FONT },
        });
    }
    items
}

fn seg_len((a, b): (Point, Point)) -> f32 {
    (b.x - a.x).abs() + (b.y - a.y).abs()
}
