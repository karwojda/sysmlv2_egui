//! Interconnection Diagrams: a part's parts inside its Frame, wired port to
//! port, laid out top to bottom (`in` ports on top, `out` ports below).
//!
//! Everything is a box: the Frame is the root's box, whose children are its
//! parts, and the root's own ports are ports on the Frame's border. An
//! Opened Part is a box with children of its own. Each box is laid out from
//! the inside out.

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
/// How many lines up or down a connector label may move to clear others.
const LABEL_TRIES: usize = 6;
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

/// One end of a wire, as seen from the box that owns the wire. `path` leads
/// from that box down through its children to the box the end meets; an
/// empty path is the owning box itself.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum End {
    Port { path: Vec<usize>, port: usize },
    /// The box at `path` itself (never empty): a wire to a part without a
    /// port, or into a closed part whose inside isn't drawn.
    Part { path: Vec<usize> },
}

impl End {
    fn path(&self) -> &[usize] {
        match self {
            End::Port { path, .. } | End::Part { path } => path,
        }
    }

    /// The owning box's child this end lies in, if any.
    fn child(&self) -> Option<usize> {
        self.path().first().copied()
    }

    fn is_own_port(&self) -> bool {
        matches!(self, End::Port { path, .. } if path.is_empty())
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

impl<'a> BoxNode<'a> {
    fn at(&self, path: &[usize]) -> &BoxNode<'a> {
        path.iter().fold(self, |node, &c| &node.children[c])
    }
}

/// Builds the Interconnection Diagram of `root`, a part definition or a
/// part usage typed by one, with the parts in `layout.opened` opened.
pub fn build(model: &Model, root: &QualifiedName, layout: &Layout) -> Result<Scene, DiagramError> {
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

    let mut frame = open_box(model, root_element, definition, "", layout);
    size(&mut frame);
    place(&mut frame, crate::diagram::MARGIN, crate::diagram::MARGIN);
    place_own_ports(&mut frame);
    place_child_ports(&mut frame, true);

    let mut items = Vec::new();
    let mut rects = Vec::new();
    draw_boxes(&frame, true, &mut items, &mut rects);
    let mut ports = Vec::new();
    draw_ports(&frame, &mut ports);
    // Connector labels keep clear of box titles, port labels and each other.
    let mut taken: Vec<Rect> = items
        .iter()
        .chain(&ports)
        .filter(|i| matches!(i.shape, Shape::Text { .. }))
        .map(|i| i.shape.extent())
        .collect();
    draw_connectors(&frame, &mut items, &mut taken);
    items.extend(ports);
    Ok(page(items, &rects))
}

fn definition_of<'a>(model: &'a Model, usage: &Element) -> Option<&'a Element> {
    usage
        .typed_by
        .as_ref()
        .and_then(|d| model.element(d))
        .filter(|d| d.kind == ElementKind::PartDefinition)
}

/// A box showing its definition's parts (each opened if `layout.opened`
/// names its path, else closed) and the wires between them. `path` is this
/// box's feature path from the root (empty for the Frame).
fn open_box<'a>(model: &'a Model, element: &'a Element, definition: &'a Element, path: &str, layout: &Layout) -> BoxNode<'a> {
    let mut node = closed_box(model, element, Some(definition));
    node.children = model
        .owned_members(&definition.qualified_name)
        .filter(|e| e.kind == ElementKind::PartUsage)
        .map(|usage| {
            let child_path = if path.is_empty() { usage.name.clone() } else { format!("{path}.{}", usage.name) };
            match definition_of(model, usage) {
                Some(d) if layout.opened.contains(&child_path) => open_box(model, usage, d, &child_path, layout),
                d => closed_box(model, usage, d),
            }
        })
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

/// The definition's connections and interfaces whose two ends resolve
/// within this box: to its own ports, or down through its parts as far as
/// they are drawn.
fn wires<'a>(model: &'a Model, definition: &'a Element, node: &BoxNode<'a>) -> Vec<Wire<'a>> {
    model
        .owned_members(&definition.qualified_name)
        .filter(|e| e.kind.is_connection())
        .filter_map(|connection| match &connection.ends[..] {
            [a, b] => Some(Wire { connection, a: resolve(node, &a.path)?, b: resolve(node, &b.path)? }),
            _ => None,
        })
        .collect()
}

/// Follows a chain of features (`rearAxleAssembly.leftWheel.hub`) into the
/// box tree: to the port if every part on the way is open, otherwise to the
/// innermost closed part on the chain.
fn resolve(node: &BoxNode, chain: &[QualifiedName]) -> Option<End> {
    let port_of = |b: &BoxNode, name: &QualifiedName| b.ports.iter().position(|s| &s.port.qualified_name == name);
    let child_of = |b: &BoxNode, name: &QualifiedName| b.children.iter().position(|c| &c.element.qualified_name == name);
    let (first, rest) = chain.split_first()?;
    if rest.is_empty()
        && let Some(port) = port_of(node, first)
    {
        return Some(End::Port { path: Vec::new(), port });
    }
    let mut path = vec![child_of(node, first)?];
    let mut current = &node.children[path[0]];
    for (i, name) in rest.iter().enumerate() {
        let last = i == rest.len() - 1;
        if let Some(c) = child_of(current, name) {
            path.push(c);
            current = &current.children[c];
        } else if last && let Some(port) = port_of(current, name) {
            return Some(End::Port { path, port });
        } else if current.children.is_empty() {
            // A closed part: the rest of the chain is inside it.
            return Some(End::Part { path });
        } else {
            return None;
        }
    }
    Some(End::Part { path })
}

fn port_direction(node: &BoxNode, end: &End) -> Option<Direction> {
    match end {
        End::Port { path, port } => node.at(path).ports[*port].port.direction,
        End::Part { .. } => None,
    }
}

/// Whether the wire's flow runs from its second end to its first. An end
/// at the box's own `in` port is where flow enters, so it counts as an
/// `out` end inside, and the other way round.
fn flows_from_b(node: &BoxNode, wire: &Wire) -> bool {
    let inward = |end: &End| match (end.is_own_port(), port_direction(node, end)) {
        (true, Some(Direction::In)) => Some(Direction::Out),
        (true, Some(Direction::Out)) => Some(Direction::In),
        (_, d) => d,
    };
    inward(&wire.a) == Some(Direction::In) || inward(&wire.b) == Some(Direction::Out)
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

/// Extra room below child `c` for the labels of wires leaving it from the
/// same end, which stack.
fn label_stack_below(node: &BoxNode, c: usize) -> f32 {
    let mut counts: BTreeMap<&End, usize> = BTreeMap::new();
    for wire in &node.wires {
        let from = if flows_from_b(node, wire) { &wire.b } else { &wire.a };
        if from.child() == Some(c) {
            *counts.entry(from).or_default() += 1;
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
    for (child, placed) in node.children.iter_mut().zip(layout::layered(&sizes, &edges)) {
        child.rect.x = placed.x + content.x;
        child.rect.y = placed.y + content.y + LABEL_ROOM;
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

/// Where the other end of a wire is, roughly, for choosing sides.
fn rough_point(node: &BoxNode, end: &End) -> Point {
    match end {
        End::Port { path, port } if path.is_empty() => port_point(node.rect, &node.ports[*port]),
        _ => center(node.at(end.path()).rect),
    }
}

/// The Frame's own ports: by direction, or facing their inner peer.
fn place_own_ports(node: &mut BoxNode) {
    let rect = node.rect;
    let mut peers: BTreeMap<usize, Point> = BTreeMap::new();
    for wire in &node.wires {
        for (this, other) in [(&wire.a, &wire.b), (&wire.b, &wire.a)] {
            if let (End::Port { path, port }, false) = (this, other.is_own_port())
                && path.is_empty()
            {
                peers.insert(*port, center(node.at(other.path()).rect));
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
/// they connect to; then does the same inside each Opened Part. With
/// `align_own`, the box's own ports line up with the child ports they're
/// wired to (only the Frame: an Opened Part's ports are placed from outside).
fn place_child_ports(node: &mut BoxNode, align_own: bool) {
    let mut peers: BTreeMap<(usize, usize), Point> = BTreeMap::new();
    for wire in &node.wires {
        for (this, other) in [(&wire.a, &wire.b), (&wire.b, &wire.a)] {
            if let End::Port { path, port } = this
                && path.len() == 1
            {
                peers.insert((path[0], *port), rough_point(node, other));
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
    for child in &mut node.children {
        if !child.children.is_empty() {
            clear_of_title(child);
            place_child_ports(child, false);
        }
    }
    if align_own {
        align_own_ports(node);
        clear_of_title(node);
    }
}

/// Moves a box's own top ports off its title, to whichever side of it is
/// nearer, so connectors from them don't run through the title text.
fn clear_of_title(node: &mut BoxNode) {
    let rect = node.rect;
    let half = text_width(&title(node.element), USAGE_NAME_FONT).max(text_width("«part def»", KEYWORD_FONT)) / 2.0
        + PORT_SPACING;
    let (left, right) = (rect.width / 2.0 - half, rect.width / 2.0 + half);
    let mut on_top: Vec<usize> = (0..node.ports.len()).filter(|&q| node.ports[q].side == Side::Top).collect();
    on_top.sort_by(|a, b| node.ports[*a].offset.total_cmp(&node.ports[*b].offset));
    // Ports left of the title's middle go left of it, the rest right of it.
    let (to_left, to_right): (Vec<usize>, Vec<usize>) = on_top.into_iter().partition(|&q| node.ports[q].offset < rect.width / 2.0);
    let mut edge = left;
    for &q in to_left.iter().rev() {
        let offset = node.ports[q].offset.min(edge).max(PORT_SPACING / 2.0);
        node.ports[q].offset = offset.round();
        edge = offset - PORT_SPACING;
    }
    let mut edge = right;
    for &q in &to_right {
        let offset = node.ports[q].offset.max(edge).min(rect.width - PORT_SPACING / 2.0);
        node.ports[q].offset = offset.round();
        edge = offset + PORT_SPACING;
    }
}

/// Moves each of the box's own ports in line with the port it is wired to
/// inside, keeping ports on one side apart.
fn align_own_ports(node: &mut BoxNode) {
    let rect = node.rect;
    let mut targets: BTreeMap<usize, f32> = BTreeMap::new();
    for wire in &node.wires {
        for (this, other) in [(&wire.a, &wire.b), (&wire.b, &wire.a)] {
            if let (End::Port { path, port }, End::Port { path: inner, port: q }) = (this, other)
                && path.is_empty()
                && !inner.is_empty()
            {
                let target = node.at(inner);
                let point = port_point(target.rect, &target.ports[*q]);
                let along = if node.ports[*port].side.vertical() { point.x - rect.x } else { point.y - rect.y };
                targets.insert(*port, along);
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

/// Where a wire end meets its box, the side it leaves from, and whether it
/// leaves inward (from the owning box's own port).
fn end_point(node: &BoxNode, end: &End, toward: Point) -> (Point, Side, bool) {
    match end {
        End::Port { path, port } => {
            let target = node.at(path);
            let slot = &target.ports[*port];
            (port_point(target.rect, slot), slot.side, path.is_empty())
        }
        End::Part { path } => {
            let rect = node.at(path).rect;
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

/// Right-angled connectors, box by box: straight out of each end, then
/// across the gap between them. Each label goes beside the middle of its
/// connector's longest run, stepping up or down a line until it clears the
/// labels in `taken`.
fn draw_connectors(node: &BoxNode, items: &mut Vec<Item>, taken: &mut Vec<Rect>) {
    for wire in &node.wires {
        let leave = |end: &End, other: &End| {
            let (point, side, inward) = end_point(node, end, rough_point(node, other));
            let (dx, dy) = side.outward();
            let sign = if inward { -1.0 } else { 1.0 };
            // Into a box from its top, a connector first clears the title.
            let length = if inward && side == Side::Top { TITLE_BAND + CONNECTOR_STUB } else { CONNECTOR_STUB };
            let stub = Point::new(point.x + sign * dx * length, point.y + sign * dy * length);
            (point, stub, side)
        };
        let (a, a1, side_a) = leave(&wire.a, &wire.b);
        let (b, b1, side_b) = leave(&wire.b, &wire.a);
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

        // Beside a vertical run, above a horizontal one.
        let (from, to) = points
            .windows(2)
            .map(|s| (s[0], s[1]))
            .max_by(|x, y| run_length(*x).total_cmp(&run_length(*y)))
            .expect("a connector has segments");
        let middle = Point::new((from.x + to.x) / 2.0, (from.y + to.y) / 2.0);
        let (preferred, anchor) = if from.x == to.x {
            (Point::new(middle.x + LABEL_OFFSET, middle.y + LABEL_FONT.size / 3.0), TextAnchor::Start)
        } else {
            (Point::new(middle.x, middle.y - LABEL_OFFSET), TextAnchor::Middle)
        };
        let text = notation::usage_label(wire.connection);
        let label_at = |step: f32| Shape::Text {
            position: Point::new(preferred.x, preferred.y + step * LABEL_STACK),
            text: text.clone(),
            anchor,
            font: LABEL_FONT,
        };
        let label = (0..=2 * LABEL_TRIES)
            .map(|k| if k % 2 == 1 { (k / 2 + 1) as f32 } else { -((k / 2) as f32) })
            .map(label_at)
            .find(|shape| !taken.iter().any(|r| r.intersects(&shape.extent())))
            .unwrap_or_else(|| label_at(0.0));
        taken.push(label.extent());
        let depicts = Depicts::Connection(wire.connection.qualified_name.clone());
        items.push(Item { depicts: depicts.clone(), shape: Shape::Polyline { points, dashed: false } });
        items.push(Item { depicts, shape: label });
    }
    for child in &node.children {
        draw_connectors(child, items, taken);
    }
}

fn run_length((a, b): (Point, Point)) -> f32 {
    (b.x - a.x).abs() + (b.y - a.y).abs()
}
