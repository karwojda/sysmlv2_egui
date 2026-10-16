//! Interconnection Diagrams: a part's parts inside its Frame, wired port to
//! port, laid out top to bottom (`in` ports on top, `out` ports below).
//!
//! Everything is a box: the Frame is the root's box, whose children are its
//! parts, and the root's own ports are ports on the Frame's border. Each box
//! is laid out from the inside out.

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

/// Space inside a box with parts, around them and above them for its title.
const BOX_PADDING: f32 = 30.0;
const TITLE_BAND: f32 = 2.0 * PADDING + KEYWORD_FONT.size + LINE_GAP + USAGE_NAME_FONT.size;
/// Room above and below each part for its port labels.
const LABEL_ROOM: f32 = LABEL_FONT.size + 2.0 * LABEL_OFFSET;
const PORT_SIZE: f32 = 10.0;
/// Space each port takes along an edge beyond its label.
const PORT_SLOT_PADDING: f32 = 24.0;
/// The least distance between two ports on one side.
const PORT_SPACING: f32 = 20.0;
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

    fn vertical(self) -> bool {
        matches!(self, Side::Top | Side::Bottom)
    }
}

/// A port on a box's border.
struct PortSlot<'a> {
    port: &'a Element,
    side: Side,
    /// Distance from the start of its side (left or top end).
    offset: f32,
}

/// One end of a wire, as seen from the box that owns the wire.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum End {
    /// A port of child `child`, or (`None`) of the owning box itself.
    Port { child: Option<usize>, port: usize },
    /// Child `child` itself, without a port.
    Part { child: usize },
}

impl End {
    fn child(self) -> Option<usize> {
        match self {
            End::Port { child, .. } => child,
            End::Part { child } => Some(child),
        }
    }
}

/// A connection or interface drawn inside the box that owns it.
struct Wire<'a> {
    connection: &'a Element,
    a: End,
    b: End,
}

/// A part's box (or the Frame), with the parts drawn inside it.
struct BoxNode<'a> {
    element: &'a Element,
    ports: Vec<PortSlot<'a>>,
    children: Vec<BoxNode<'a>>,
    wires: Vec<Wire<'a>>,
    /// Width and height, then the absolute rect once placed.
    rect: Rect,
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

    let mut frame = open_box(model, root_element, Some(definition));
    size(&mut frame);
    place(&mut frame, crate::diagram::MARGIN, crate::diagram::MARGIN);
    place_own_ports(&mut frame);
    place_child_ports(&mut frame);

    let mut items = Vec::new();
    let mut rects = Vec::new();
    draw_boxes(&frame, true, &mut items, &mut rects);
    draw_connectors(&frame, &mut items);
    draw_ports(&frame, &mut items);
    Ok(page(items, &rects))
}

/// A box showing its definition's parts, each closed, and the wires
/// between them.
fn open_box<'a>(model: &'a Model, element: &'a Element, definition: Option<&'a Element>) -> BoxNode<'a> {
    let mut node = closed_box(model, element, definition);
    let Some(definition) = definition else { return node };
    node.children = model
        .owned_members(&definition.qualified_name)
        .filter(|e| e.kind == ElementKind::PartUsage)
        .map(|usage| closed_box(model, usage, usage.typed_by.as_ref().and_then(|d| model.element(d))))
        .collect();
    node.wires = wires(model, definition, &node);
    node
}

/// A box with only its ports: its definition's, then any it declares
/// itself.
fn closed_box<'a>(model: &'a Model, element: &'a Element, definition: Option<&'a Element>) -> BoxNode<'a> {
    let own = (definition.map(|d| &d.qualified_name) != Some(&element.qualified_name))
        .then(|| model.owned_members(&element.qualified_name))
        .into_iter()
        .flatten();
    let ports = definition
        .into_iter()
        .flat_map(|d| model.owned_members(&d.qualified_name))
        .chain(own)
        .filter(|e| e.kind == ElementKind::PortUsage)
        .map(|port| PortSlot { port, side: Side::Bottom, offset: 0.0 })
        .collect();
    BoxNode { element, ports, children: Vec::new(), wires: Vec::new(), rect: Rect { x: 0.0, y: 0.0, width: 0.0, height: 0.0 } }
}

/// The definition's connections and interfaces whose two ends resolve to
/// this box's parts, their ports, or the box's own ports.
fn wires<'a>(model: &'a Model, definition: &'a Element, node: &BoxNode<'a>) -> Vec<Wire<'a>> {
    let child_index = |name: &QualifiedName| node.children.iter().position(|c| &c.element.qualified_name == name);
    let own_port = |name: &QualifiedName| node.ports.iter().position(|s| &s.port.qualified_name == name);
    let end = |chain: &[QualifiedName]| -> Option<End> {
        match chain {
            [only] => match (child_index(only), own_port(only)) {
                (Some(child), _) => Some(End::Part { child }),
                (None, Some(port)) => Some(End::Port { child: None, port }),
                _ => None,
            },
            [part, port] => {
                let child = child_index(part)?;
                let port = node.children[child].ports.iter().position(|s| &s.port.qualified_name == port)?;
                Some(End::Port { child: Some(child), port })
            }
            _ => None,
        }
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

fn port_direction(node: &BoxNode, end: End) -> Option<Direction> {
    match end {
        End::Port { child: Some(c), port } => node.children[c].ports[port].port.direction,
        End::Port { child: None, port } => node.ports[port].port.direction,
        End::Part { .. } => None,
    }
}

/// Whether the wire's flow runs from its second end to its first. An end
/// at the box's own `in` port is where flow enters, so it counts as an
/// `out` end inside, and the other way round.
fn flows_from_b(node: &BoxNode, wire: &Wire) -> bool {
    let inward = |end: End, direction: Option<Direction>| match (end, direction) {
        (End::Port { child: None, .. }, Some(Direction::In)) => Some(Direction::Out),
        (End::Port { child: None, .. }, Some(Direction::Out)) => Some(Direction::In),
        (_, d) => d,
    };
    let a = inward(wire.a, port_direction(node, wire.a));
    let b = inward(wire.b, port_direction(node, wire.b));
    a == Some(Direction::In) || b == Some(Direction::Out)
}

fn port_label(port: &Element) -> String {
    let mut label = port.name.clone();
    if let Some(definition) = &port.typed_by {
        let conjugate = if port.conjugated { "~" } else { "" };
        label.push_str(&format!(" : {conjugate}{}", definition.simple_name()));
    }
    label
}

fn title(element: &Element) -> String {
    if element.kind == ElementKind::PartUsage { notation::usage_label(element) } else { element.name.clone() }
}

/// Width needed for the labels of a box's `in` (top) or `out` (bottom)
/// ports side by side.
fn edge_room(node: &BoxNode, direction: Direction) -> f32 {
    let labels: Vec<f32> = node
        .ports
        .iter()
        .filter(|s| s.port.direction == Some(direction))
        .map(|s| text_width(&port_label(s.port), LABEL_FONT) + PORT_SLOT_PADDING)
        .collect();
    let widest = labels.iter().copied().fold(0.0, f32::max);
    widest * (labels.len() as f32 + 1.0)
}

/// Extra room below child `c` for the labels of wires leaving it that
/// stack because they share their ends.
fn label_stack_below(node: &BoxNode, c: usize) -> f32 {
    let mut counts: BTreeMap<(End, End), usize> = BTreeMap::new();
    for wire in &node.wires {
        let (from, to) = if flows_from_b(node, wire) { (wire.b, wire.a) } else { (wire.a, wire.b) };
        if from.child() == Some(c) {
            *counts.entry((from, to)).or_default() += 1;
        }
    }
    let deepest = counts.values().copied().max().unwrap_or(1).max(1);
    (deepest - 1) as f32 * LABEL_STACK
}

/// Sizes a box from the inside out, placing its children relative to its
/// top-left corner.
fn size(node: &mut BoxNode) {
    let title_width = text_width(&title(node.element), USAGE_NAME_FONT).max(text_width("«part def»", KEYWORD_FONT));
    let ports_width = edge_room(node, Direction::In).max(edge_room(node, Direction::Out));
    if node.children.is_empty() {
        let width = (title_width + 2.0 * PADDING).max(MIN_NODE_WIDTH).max(ports_width);
        let height = 2.0 * PADDING + KEYWORD_FONT.size + LINE_GAP + USAGE_NAME_FONT.size;
        node.rect.width = width.ceil();
        node.rect.height = height.ceil();
        return;
    }
    for child in &mut node.children {
        size(child);
    }
    // Layers top to bottom from each wire's `out` end to its `in` end, with
    // room for port labels above and below every child, and below it for
    // stacked labels of wires leaving it.
    let below: Vec<f32> = (0..node.children.len()).map(|c| LABEL_ROOM + label_stack_below(node, c)).collect();
    let sizes: Vec<(f32, f32)> = node
        .children
        .iter()
        .zip(&below)
        .map(|(c, below)| (c.rect.width, c.rect.height + LABEL_ROOM + below))
        .collect();
    let edges: Vec<(usize, usize)> = node
        .wires
        .iter()
        .filter_map(|w| match (w.a.child(), w.b.child()) {
            (Some(a), Some(b)) if a != b => Some(if flows_from_b(node, w) { (b, a) } else { (a, b) }),
            _ => None,
        })
        .collect();
    let content = Point::new(BOX_PADDING, TITLE_BAND + BOX_PADDING);
    for ((child, placed), below) in node.children.iter_mut().zip(layout::layered(&sizes, &edges)).zip(&below) {
        child.rect.x = placed.x + content.x;
        child.rect.y = placed.y + content.y + LABEL_ROOM;
        let _ = below;
    }
    let right = node.children.iter().map(|c| c.rect.right()).fold(content.x, f32::max);
    let bottom = node.children.iter().zip(&below).map(|(c, b)| c.rect.bottom() + b).fold(content.y, f32::max);
    node.rect.width = (right + BOX_PADDING).max(title_width + 2.0 * PADDING).max(ports_width).max(MIN_NODE_WIDTH).ceil();
    node.rect.height = (bottom + BOX_PADDING).ceil();
}

/// Turns child positions relative to their box into absolute ones.
fn place(node: &mut BoxNode, x: f32, y: f32) {
    node.rect.x = x;
    node.rect.y = y;
    for child in &mut node.children {
        let (cx, cy) = (child.rect.x + x, child.rect.y + y);
        place(child, cx, cy);
    }
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

fn by_direction(direction: Option<Direction>) -> Option<Side> {
    match direction {
        Some(Direction::In) => Some(Side::Top),
        Some(Direction::Out) => Some(Side::Bottom),
        _ => None,
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

/// Spaces the given ports evenly along their side, in the order of
/// `along` (where each port's peer is along that side).
fn spread(slots: &mut [PortSlot], rect: Rect, along: &dyn Fn(usize) -> f32) {
    for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
        let mut on_side: Vec<usize> = (0..slots.len()).filter(|&q| slots[q].side == side).collect();
        on_side.sort_by(|a, b| along(*a).total_cmp(&along(*b)).then(a.cmp(b)));
        let length = if side.vertical() { rect.width } else { rect.height };
        let count = on_side.len() as f32;
        for (k, q) in on_side.into_iter().enumerate() {
            slots[q].offset = (length * (k as f32 + 1.0) / (count + 1.0)).round();
        }
    }
}

/// The Frame's own ports: by direction, or facing their inner peer; each
/// lined up with that peer where it can be, so its wire runs straight.
fn place_own_ports(node: &mut BoxNode) {
    let rect = node.rect;
    let mut peers: BTreeMap<usize, Point> = BTreeMap::new();
    for wire in &node.wires {
        for (this, other) in [(wire.a, wire.b), (wire.b, wire.a)] {
            if let (End::Port { child: None, port }, Some(c)) = (this, other.child()) {
                peers.insert(port, center(node.children[c].rect));
            }
        }
    }
    for (q, slot) in node.ports.iter_mut().enumerate() {
        slot.side = match (by_direction(slot.port.direction), peers.get(&q)) {
            (Some(side), _) => side,
            (None, Some(peer)) => facing(center(rect), *peer),
            (None, None) => Side::Bottom,
        };
    }
    let sides: Vec<Side> = node.ports.iter().map(|s| s.side).collect();
    spread(&mut node.ports, rect, &|q| match peers.get(&q) {
        Some(peer) if sides[q].vertical() => peer.x,
        Some(peer) => peer.y,
        None => f32::MAX,
    });
}

/// Puts each child's ports on a side (`in` top, `out` bottom, undirected
/// facing the other end, else bottom) and spaces them in the order of what
/// they connect to. A child port wired to the box's own port lines up with
/// it.
fn place_child_ports(node: &mut BoxNode) {
    let own_points: Vec<Point> = node.ports.iter().map(|s| port_point(node.rect, s)).collect();
    let mut peers: BTreeMap<(usize, usize), Point> = BTreeMap::new();
    for wire in &node.wires {
        for (this, other) in [(wire.a, wire.b), (wire.b, wire.a)] {
            if let End::Port { child: Some(c), port } = this {
                let peer = match other {
                    End::Port { child: Some(o), .. } | End::Part { child: o } => center(node.children[o].rect),
                    End::Port { child: None, port: own } => own_points[own],
                };
                peers.insert((c, port), peer);
            }
        }
    }
    for (c, child) in node.children.iter_mut().enumerate() {
        let here = center(child.rect);
        for (q, slot) in child.ports.iter_mut().enumerate() {
            slot.side = match (by_direction(slot.port.direction), peers.get(&(c, q))) {
                (Some(side), _) => side,
                (None, Some(peer)) => facing(here, *peer),
                (None, None) => Side::Bottom,
            };
        }
        let sides: Vec<Side> = child.ports.iter().map(|s| s.side).collect();
        let rect = child.rect;
        spread(&mut child.ports, rect, &|q| match peers.get(&(c, q)) {
            Some(peer) if sides[q].vertical() => peer.x,
            Some(peer) => peer.y,
            None => f32::MAX,
        });
    }
    align_own_ports(node);
}

/// Moves each of the box's own ports in line with the child port it is
/// wired to, keeping ports on one side apart.
fn align_own_ports(node: &mut BoxNode) {
    let rect = node.rect;
    let mut targets: BTreeMap<usize, f32> = BTreeMap::new();
    for wire in &node.wires {
        for (this, other) in [(wire.a, wire.b), (wire.b, wire.a)] {
            if let (End::Port { child: None, port }, End::Port { child: Some(c), port: q }) = (this, other) {
                let point = port_point(node.children[c].rect, &node.children[c].ports[q]);
                let along = if node.ports[port].side.vertical() { point.x - rect.x } else { point.y - rect.y };
                targets.insert(port, along);
            }
        }
    }
    for side in [Side::Top, Side::Bottom, Side::Left, Side::Right] {
        let mut on_side: Vec<usize> = (0..node.ports.len()).filter(|&q| node.ports[q].side == side).collect();
        on_side.sort_by(|a, b| {
            let key = |q: &usize| targets.get(q).copied().unwrap_or(node.ports[*q].offset);
            key(a).total_cmp(&key(b))
        });
        let length = if side.vertical() { rect.width } else { rect.height };
        let mut previous = f32::NEG_INFINITY;
        for q in on_side {
            let wanted = targets.get(&q).copied().unwrap_or(node.ports[q].offset);
            let offset = wanted.max(previous + PORT_SPACING).clamp(PORT_SPACING, length - PORT_SPACING);
            node.ports[q].offset = offset.round();
            previous = offset;
        }
    }
}

fn texts(depicts: &Depicts, rect: Rect, keyword: &str, title: String) -> Vec<Item> {
    let keyword_baseline = rect.y + PADDING + KEYWORD_FONT.size;
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

/// Boxes from the outside in: each a rounded box with its keyword above
/// its title (the Frame's corners rounder).
fn draw_boxes(node: &BoxNode, frame: bool, items: &mut Vec<Item>, rects: &mut Vec<Rect>) {
    let depicts = Depicts::Element(node.element.qualified_name.clone());
    let radius = if frame { 2.0 * USAGE_CORNER_RADIUS } else { USAGE_CORNER_RADIUS };
    items.push(Item { depicts: depicts.clone(), shape: Shape::Rect { rect: node.rect, fill: Fill::Background, corner_radius: radius } });
    let keyword = format!("«{}»", node.element.kind.keyword());
    items.extend(texts(&depicts, node.rect, &keyword, title(node.element)));
    rects.push(node.rect);
    for child in &node.children {
        draw_boxes(child, false, items, rects);
    }
}

/// Each port as a small square on its box's border, labelled beside it,
/// outside the box.
fn draw_ports(node: &BoxNode, items: &mut Vec<Item>) {
    for slot in &node.ports {
        let depicts = Depicts::Port { part: node.element.qualified_name.clone(), port: slot.port.qualified_name.clone() };
        let p = port_point(node.rect, slot);
        let square = Rect { x: p.x - PORT_SIZE / 2.0, y: p.y - PORT_SIZE / 2.0, width: PORT_SIZE, height: PORT_SIZE };
        let beside = PORT_SIZE / 2.0 + 3.0;
        let (position, anchor) = match slot.side {
            Side::Top => (Point::new(p.x + beside, p.y - LABEL_OFFSET), TextAnchor::Start),
            Side::Bottom => (Point::new(p.x + beside, p.y + LABEL_OFFSET + LABEL_FONT.size), TextAnchor::Start),
            Side::Left => (Point::new(p.x - beside, p.y - LABEL_OFFSET), TextAnchor::End),
            Side::Right => (Point::new(p.x + beside, p.y - LABEL_OFFSET), TextAnchor::Start),
        };
        items.push(Item { depicts: depicts.clone(), shape: Shape::Rect { rect: square, fill: Fill::Background, corner_radius: 0.0 } });
        items.push(Item { depicts, shape: Shape::Text { position, text: port_label(slot.port), anchor, font: LABEL_FONT } });
    }
    for child in &node.children {
        draw_ports(child, items);
    }
}

/// Where a wire end meets its box, and the direction the wire leaves in.
fn end_point(node: &BoxNode, end: End, toward: Point) -> (Point, Side, bool) {
    match end {
        End::Port { child: Some(c), port } => {
            let child = &node.children[c];
            let slot = &child.ports[port];
            (port_point(child.rect, slot), slot.side, false)
        }
        // A wire leaves the box's own port inward.
        End::Port { child: None, port } => {
            let slot = &node.ports[port];
            (port_point(node.rect, slot), slot.side, true)
        }
        End::Part { child } => {
            let rect = node.children[child].rect;
            let side = facing(center(rect), toward);
            let c = center(rect);
            let point = match side {
                Side::Top => Point::new(c.x, rect.y),
                Side::Bottom => Point::new(c.x, rect.bottom()),
                Side::Left => Point::new(rect.x, c.y),
                Side::Right => Point::new(rect.right(), c.y),
            };
            (point, side, false)
        }
    }
}

fn rough_point(node: &BoxNode, end: End) -> Point {
    match end.child() {
        Some(c) => center(node.children[c].rect),
        None => match end {
            End::Port { port, .. } => port_point(node.rect, &node.ports[port]),
            End::Part { .. } => center(node.rect),
        },
    }
}

/// Right-angled connectors, box by box: straight out of each end, then
/// across the gap between them. Connectors over the same ends stack their
/// labels, centred on the connector.
fn draw_connectors(node: &BoxNode, items: &mut Vec<Item>) {
    let mut seen: Vec<(End, End)> = Vec::new();
    for wire in &node.wires {
        let leave = |end: End, other: End| {
            let (point, side, inward) = end_point(node, end, rough_point(node, other));
            let (dx, dy) = side.outward();
            let sign = if inward { -1.0 } else { 1.0 };
            let stub = Point::new(point.x + sign * dx * CONNECTOR_STUB, point.y + sign * dy * CONNECTOR_STUB);
            (point, stub, side)
        };
        let (a, a1, side_a) = leave(wire.a, wire.b);
        let (b, b1, side_b) = leave(wire.b, wire.a);
        let points = if side_a.vertical() && side_b.vertical() && (a.x - b.x).abs() < ALIGNED {
            vec![a, Point::new(a.x, b.y)]
        } else if !side_a.vertical() && !side_b.vertical() && (a.y - b.y).abs() < ALIGNED {
            vec![a, Point::new(b.x, a.y)]
        } else if side_a.vertical() && side_b.vertical() {
            let middle = (a1.y + b1.y) / 2.0;
            vec![a, a1, Point::new(a1.x, middle), Point::new(b1.x, middle), b1, b]
        } else {
            let middle = (a1.x + b1.x) / 2.0;
            vec![a, a1, Point::new(middle, a1.y), Point::new(middle, b1.y), b1, b]
        };

        let pair = (wire.a.min(wire.b), wire.a.max(wire.b));
        let index = seen.iter().filter(|p| **p == pair).count() as f32;
        seen.push(pair);
        let count = node.wires.iter().filter(|w| (w.a.min(w.b), w.a.max(w.b)) == pair).count() as f32;
        let stacked = index - (count - 1.0) / 2.0;
        // The label sits beside the middle of the connector's longest run:
        // beside a vertical run, above a horizontal one.
        let (from, to) = points
            .windows(2)
            .map(|s| (s[0], s[1]))
            .max_by(|x, y| run_length(*x).total_cmp(&run_length(*y)))
            .expect("a connector has segments");
        let middle = Point::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
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
    for child in &node.children {
        draw_connectors(child, items);
    }
}

fn run_length((a, b): (Point, Point)) -> f32 {
    (b.x - a.x).abs() + (b.y - a.y).abs()
}
