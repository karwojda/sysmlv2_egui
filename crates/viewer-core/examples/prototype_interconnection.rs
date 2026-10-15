//! PROTOTYPE, throwaway: Interconnection Diagram layout spike (not product code).
//!
//! Question: does a layered layout plus packing, with ports on borders, give
//! acceptable Interconnection Diagrams? Three structurally different
//! variants, rendered as SVG for two Models, to compare side by side:
//!   A: layered left-to-right, ports face their peer, straight connectors
//!   B: compact grid in connection order, ports face their peer, straight
//!   C: layered left-to-right, ports by direction (in left, out right),
//!      right-angled connectors
//!   D: C, with gaps widened to fit port labels and stacked labels for
//!      connections that share their ports
//!
//! `cargo run -p viewer-core --example prototype_interconnection -- <out-dir>`

use std::collections::BTreeMap;
use std::path::PathBuf;

use rust_sugiyama::configure::Config;
use viewer_core::model::{Direction, ElementKind, Model, QualifiedName};
use viewer_core::scene::{Depicts, Fill, Font, Item, Point, Rect, Scene, Shape, TextAnchor};
use viewer_core::svg;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Variant {
    A,
    B,
    C,
    D,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Side {
    Left,
    Right,
    Top,
    Bottom,
}

struct Port {
    name: String,
    label: String,
    direction: Option<Direction>,
    side: Side,
    offset: f32,
}

#[derive(Clone)]
enum End {
    /// A port of child `child`, or of the container itself.
    Port { child: Option<usize>, port: String },
    Part { child: usize },
}

struct Conn {
    label: String,
    a: End,
    b: End,
}

struct Node {
    label: String,
    ports: Vec<Port>,
    children: Vec<Node>,
    conns: Vec<Conn>,
    w: f32,
    h: f32,
    x: f32,
    y: f32,
}

const HEADER: f32 = 40.0;
const PAD: f32 = 30.0;
const PORT: f32 = 10.0;
const PORT_GAP: f32 = 26.0;
const GAP: f32 = 70.0;

fn text_w(s: &str, size: f32) -> f32 {
    s.chars().count() as f32 * size * 0.58
}

fn qn(s: &str) -> QualifiedName {
    QualifiedName::from(s)
}

/// The box for definition `def`, labelled `label`; parts are drawn inside
/// it if `open(path)`.
fn build(model: &Model, def: Option<&QualifiedName>, label: String, path: &str, open: &dyn Fn(&str) -> bool) -> Node {
    let mut node = Node { label, ports: vec![], children: vec![], conns: vec![], w: 0.0, h: 0.0, x: 0.0, y: 0.0 };
    let Some(def) = def else { return node };
    for m in model.owned_members(def) {
        if m.kind == ElementKind::PortUsage {
            let ty = m.typed_by.as_ref().map(|t| format!(" : {}{}", if m.conjugated { "~" } else { "" }, t.simple_name())).unwrap_or_default();
            node.ports.push(Port { name: m.name.clone(), label: format!("{}{ty}", m.name), direction: m.direction, side: Side::Bottom, offset: 0.0 });
        }
    }
    if !open(path) {
        return node;
    }
    let parts: Vec<_> = model.owned_members(def).filter(|m| m.kind == ElementKind::PartUsage).collect();
    for p in &parts {
        let ty = p.typed_by.as_ref().map(|t| format!(":{}", t.simple_name())).unwrap_or_default();
        let child_path = format!("{path}.{}", p.name);
        node.children.push(build(model, p.typed_by.as_ref(), format!("{}{ty}", p.name), &child_path, open));
    }
    let child_index = |name: &str| parts.iter().position(|p| p.name == name);
    let end = |chain: &[QualifiedName]| -> Option<End> {
        let first = chain.first()?.simple_name();
        match (child_index(first), chain.len()) {
            (Some(child), 1) => Some(End::Part { child }),
            (Some(child), _) => Some(End::Port { child: Some(child), port: chain[1].simple_name().to_owned() }),
            (None, 1) if node.ports.iter().any(|p| p.name == first) => Some(End::Port { child: None, port: first.to_owned() }),
            _ => None,
        }
    };
    for c in model.owned_members(def).filter(|m| m.kind.is_connection()) {
        if let [a, b] = &c.ends[..]
            && let (Some(a), Some(b)) = (end(&a.path), end(&b.path))
        {
            let label = match &c.typed_by {
                Some(t) => format!("{}:{}", c.name, t.simple_name()),
                None => c.name.clone(),
            };
            node.conns.push(Conn { label, a, b });
        }
    }
    node
}

fn child_of(end: &End) -> Option<usize> {
    match end {
        End::Port { child, .. } => *child,
        End::Part { child } => Some(*child),
    }
}

fn port_direction(node: &Node, end: &End) -> Option<Direction> {
    match end {
        End::Port { child: Some(c), port } => node.children[*c].ports.iter().find(|p| &p.name == port)?.direction,
        End::Port { child: None, port } => node.ports.iter().find(|p| &p.name == port)?.direction,
        End::Part { .. } => None,
    }
}

/// Sizes and arranges `node` and everything inside it, bottom up.
fn layout(node: &mut Node, variant: Variant) {
    for child in &mut node.children {
        layout(child, variant);
    }
    if node.children.is_empty() {
        let per_side = node.ports.len().div_ceil(2).max(1) as f32;
        node.w = (text_w(&node.label, 13.0) + 2.0 * PAD).max(120.0);
        node.h = (per_side * PORT_GAP + 20.0).max(54.0);
    } else {
        match variant {
            Variant::A | Variant::C => layered(node, false),
            Variant::D => layered(node, true),
            Variant::B => grid(node),
        }
        let right = node.children.iter().map(|c| c.x + c.w).fold(0.0, f32::max);
        let bottom = node.children.iter().map(|c| c.y + c.h).fold(0.0, f32::max);
        node.w = (right + 2.0 * PAD).max(text_w(&node.label, 13.0) + 2.0 * PAD);
        node.h = bottom + HEADER + 2.0 * PAD;
        let ports = node.ports.len().div_ceil(2) as f32 * PORT_GAP + HEADER;
        node.h = node.h.max(ports);
    }
    assign_ports(node, variant);
}

/// Left-to-right layers: the layout library is top-down, so sizes and
/// coordinates are transposed.
fn layered(node: &mut Node, label_room: bool) {
    // With `label_room`, each part reserves room beside it for the labels
    // of its in (left) and out (right) ports.
    let room = |c: &Node, d: Direction| {
        c.ports.iter().filter(|p| p.direction == Some(d)).map(|p| text_w(&p.label, 10.0) + 12.0).fold(0.0, f32::max)
    };
    let extra: Vec<(f32, f32)> = node
        .children
        .iter()
        .map(|c| if label_room { (room(c, Direction::In), room(c, Direction::Out)) } else { (0.0, 0.0) })
        .collect();
    let vertices: Vec<(u32, (f64, f64))> = node
        .children
        .iter()
        .enumerate()
        .map(|(i, c)| (i as u32, (c.h as f64, (c.w + extra[i].0 + extra[i].1) as f64)))
        .collect();
    let mut edges = Vec::new();
    for conn in &node.conns {
        if let (Some(a), Some(b)) = (child_of(&conn.a), child_of(&conn.b))
            && a != b
        {
            // Flow runs from an `out` end to an `in` end.
            let reversed = port_direction(node, &conn.a) == Some(Direction::In)
                || port_direction(node, &conn.b) == Some(Direction::Out);
            edges.push(if reversed { (b as u32, a as u32) } else { (a as u32, b as u32) });
        }
    }
    let config = Config { vertex_spacing: GAP as f64, ..Config::default() };
    let mut components = rust_sugiyama::from_vertices_and_edges(&vertices, &edges, &config);
    components.sort_by_key(|(p, _, _)| p.iter().map(|(id, _)| *id).min());
    let mut offset_y = 0.0;
    for (positions, _, _) in components {
        let placed: Vec<(usize, f32, f32)> = positions
            .iter()
            .map(|&(id, (cx, cy))| {
                let c = &node.children[id];
                let (left, right) = extra[id];
                (id, cy as f32 - (c.w + left + right) / 2.0 + left, cx as f32 - c.h / 2.0)
            })
            .collect();
        let min_x = placed.iter().map(|p| p.1 - extra[p.0].0).fold(f32::INFINITY, f32::min);
        let min_y = placed.iter().map(|p| p.2).fold(f32::INFINITY, f32::min);
        let mut bottom: f32 = 0.0;
        for (id, x, y) in placed {
            let c = &mut node.children[id];
            c.x = (x - min_x).round();
            c.y = (y - min_y + offset_y).round();
            bottom = bottom.max(c.y + c.h);
        }
        offset_y = bottom + GAP;
    }
}

/// A compact grid, filled in the order connections reach the parts.
fn grid(node: &mut Node) {
    let n = node.children.len();
    let mut order = Vec::new();
    for start in 0..n {
        if order.contains(&start) {
            continue;
        }
        let mut queue = vec![start];
        while let Some(i) = queue.first().copied() {
            queue.remove(0);
            if order.contains(&i) {
                continue;
            }
            order.push(i);
            for conn in &node.conns {
                match (child_of(&conn.a), child_of(&conn.b)) {
                    (Some(a), Some(b)) if a == i => queue.push(b),
                    (Some(a), Some(b)) if b == i => queue.push(a),
                    _ => {}
                }
            }
        }
    }
    let cols = (n as f32).sqrt().ceil() as usize;
    let col_w: Vec<f32> = (0..cols)
        .map(|c| order.iter().skip(c).step_by(cols).map(|&i| node.children[i].w).fold(0.0, f32::max))
        .collect();
    let (mut y, mut row_h) = (0.0, 0.0_f32);
    for (k, &i) in order.iter().enumerate() {
        let col = k % cols;
        if col == 0 && k > 0 {
            y += row_h + GAP;
            row_h = 0.0;
        }
        let x: f32 = col_w[..col].iter().map(|w| w + GAP).sum();
        node.children[i].x = x;
        node.children[i].y = y;
        row_h = row_h.max(node.children[i].h);
    }
}

fn center(c: &Node) -> Point {
    Point::new(c.x + c.w / 2.0, c.y + c.h / 2.0)
}

fn side_facing(from: Point, to: Point) -> Side {
    let (dx, dy) = (to.x - from.x, to.y - from.y);
    if dx.abs() >= dy.abs() {
        if dx >= 0.0 { Side::Right } else { Side::Left }
    } else if dy >= 0.0 {
        Side::Bottom
    } else {
        Side::Top
    }
}

/// Puts each child's ports, and the container's own, on a side and spreads
/// them along it.
fn assign_ports(node: &mut Node, variant: Variant) {
    let by_direction = |d: Option<Direction>| match d {
        Some(Direction::In) => Some(Side::Left),
        Some(Direction::Out) => Some(Side::Right),
        _ => None,
    };
    // Where each port's peer is, in this container's coordinates.
    let inner = Point::new(node.w / 2.0 - PAD, (node.h - HEADER) / 2.0 - PAD);
    let mut peers: BTreeMap<(Option<usize>, String), Point> = BTreeMap::new();
    for conn in &node.conns {
        for (this, other) in [(&conn.a, &conn.b), (&conn.b, &conn.a)] {
            if let End::Port { child, port } = this {
                let peer = match child_of(other) {
                    Some(c) => center(&node.children[c]),
                    None => match child {
                        // A child's port meeting the frame faces the nearest frame side.
                        Some(c) => {
                            let cc = center(&node.children[*c]);
                            if cc.x < inner.x { Point::new(-1e4, cc.y) } else { Point::new(1e4, cc.y) }
                        }
                        None => inner,
                    },
                };
                peers.insert((*child, port.clone()), peer);
            }
        }
    }
    // Frame ports face their inner peer from the content's middle.
    for port in &mut node.ports {
        let peer = peers.get(&(None, port.name.clone()));
        port.side = match (variant, by_direction(port.direction), peer) {
            (Variant::C | Variant::D, Some(side), _) => side,
            (_, _, Some(p)) => side_facing(inner, *p),
            (_, Some(side), None) => side,
            _ => Side::Bottom,
        };
    }
    spread(&mut node.ports, node.w, node.h, &|_| 0.0);
    for (i, child) in node.children.iter_mut().enumerate() {
        let c = Point::new(child.x + child.w / 2.0, child.y + child.h / 2.0);
        for port in &mut child.ports {
            let peer = peers.get(&(Some(i), port.name.clone()));
            port.side = match (variant, by_direction(port.direction), peer) {
                (Variant::C | Variant::D, Some(side), _) => side,
                (_, _, Some(p)) => side_facing(c, *p),
                (_, Some(side), None) => side,
                _ => Side::Bottom,
            };
        }
        let along = |p: &Port| match peers.get(&(Some(i), p.name.clone())) {
            Some(peer) if matches!(p.side, Side::Left | Side::Right) => peer.y,
            Some(peer) => peer.x,
            None => f32::MAX,
        };
        spread(&mut child.ports, child.w, child.h, &along);
    }
}

/// Evenly spaces the ports on each side, ordered by where their peers are
/// so connections don't cross needlessly.
fn spread(ports: &mut [Port], w: f32, h: f32, along: &dyn Fn(&Port) -> f32) {
    for side in [Side::Left, Side::Right, Side::Top, Side::Bottom] {
        let mut on: Vec<&mut Port> = ports.iter_mut().filter(|p| p.side == side).collect();
        on.sort_by(|a, b| along(a).total_cmp(&along(b)));
        let length = if matches!(side, Side::Left | Side::Right) { h } else { w };
        let n = on.len() as f32;
        for (k, port) in on.iter_mut().enumerate() {
            port.offset = length * (k as f32 + 1.0) / (n + 1.0);
        }
    }
}

fn port_point(rect: Rect, port: &Port) -> Point {
    match port.side {
        Side::Left => Point::new(rect.x, rect.y + port.offset),
        Side::Right => Point::new(rect.right(), rect.y + port.offset),
        Side::Top => Point::new(rect.x + port.offset, rect.y),
        Side::Bottom => Point::new(rect.x + port.offset, rect.bottom()),
    }
}

fn outward(side: Side) -> (f32, f32) {
    match side {
        Side::Left => (-1.0, 0.0),
        Side::Right => (1.0, 0.0),
        Side::Top => (0.0, -1.0),
        Side::Bottom => (0.0, 1.0),
    }
}

struct Painter {
    items: Vec<Item>,
    ports: Vec<Item>,
}

impl Painter {
    fn add(&mut self, shape: Shape) {
        self.items.push(Item { depicts: Depicts::Element(qn("prototype")), shape });
    }
    fn port(&mut self, shape: Shape) {
        self.ports.push(Item { depicts: Depicts::Element(qn("prototype")), shape });
    }
    fn text(&mut self, at: Point, text: &str, anchor: TextAnchor, size: f32) {
        self.add(Shape::Text { position: at, text: text.to_owned(), anchor, font: Font { size, bold: false, italic: false } });
    }
}

/// Draws `node` at absolute (`ox`, `oy`) and everything inside it.
fn paint(node: &Node, ox: f32, oy: f32, variant: Variant, out: &mut Painter) {
    let rect = Rect { x: ox, y: oy, width: node.w, height: node.h };
    out.add(Shape::Rect { rect, fill: Fill::Background, corner_radius: 12.0 });
    let leaf = node.children.is_empty();
    let title_y = if leaf { oy + node.h / 2.0 - 4.0 } else { oy + 18.0 };
    out.text(Point::new(rect.center_x(), title_y), "«part»", TextAnchor::Middle, 11.0);
    out.text(Point::new(rect.center_x(), title_y + 15.0), &node.label, TextAnchor::Middle, 13.0);

    let content = (ox + PAD, oy + HEADER + PAD);
    let child_rect = |c: &Node| Rect { x: content.0 + c.x, y: content.1 + c.y, width: c.w, height: c.h };
    for child in &node.children {
        paint(child, content.0 + child.x, content.1 + child.y, variant, out);
    }

    // Connections between this box's children (and its own ports).
    let end_point = |end: &End, toward: Point| -> (Point, Option<Side>) {
        match end {
            End::Port { child: Some(c), port } => {
                let child = &node.children[*c];
                let p = child.ports.iter().find(|p| &p.name == port).unwrap();
                (port_point(child_rect(child), p), Some(p.side))
            }
            End::Port { child: None, port } => {
                let p = node.ports.iter().find(|p| &p.name == port).unwrap();
                (port_point(rect, p), Some(p.side))
            }
            End::Part { child } => {
                let r = child_rect(&node.children[*child]);
                let c = Point::new(r.center_x(), r.y + r.height / 2.0);
                let side = side_facing(c, toward);
                let p = match side {
                    Side::Left => Point::new(r.x, c.y),
                    Side::Right => Point::new(r.right(), c.y),
                    Side::Top => Point::new(c.x, r.y),
                    Side::Bottom => Point::new(c.x, r.bottom()),
                };
                (p, Some(side))
            }
        }
    };
    let rough = |end: &End| match child_of(end) {
        Some(c) => {
            let r = child_rect(&node.children[c]);
            Point::new(r.center_x(), r.y + r.height / 2.0)
        }
        None => Point::new(rect.center_x(), rect.y + rect.height / 2.0),
    };
    let mut drawn: Vec<(Point, Point)> = Vec::new();
    for conn in &node.conns {
        let (a, side_a) = end_point(&conn.a, rough(&conn.b));
        let (b, side_b) = end_point(&conn.b, rough(&conn.a));
        // A frame port's line runs inward, so flip its outward direction.
        let inward = |end: &End, side: Option<Side>| {
            let (dx, dy) = outward(side.unwrap_or(Side::Right));
            if child_of(end).is_none() { (-dx, -dy) } else { (dx, dy) }
        };
        let points = if variant == Variant::C {
            let (ax, ay) = inward(&conn.a, side_a);
            let (bx, by) = inward(&conn.b, side_b);
            let a1 = Point::new(a.x + ax * 16.0, a.y + ay * 16.0);
            let b1 = Point::new(b.x + bx * 16.0, b.y + by * 16.0);
            if ax != 0.0 || bx != 0.0 {
                let mx = (a1.x + b1.x) / 2.0;
                vec![a, a1, Point::new(mx, a1.y), Point::new(mx, b1.y), b1, b]
            } else {
                let my = (a1.y + b1.y) / 2.0;
                vec![a, a1, Point::new(a1.x, my), Point::new(b1.x, my), b1, b]
            }
        } else {
            vec![a, b]
        };
        let mid = points[points.len() / 2 - 1];
        let mid2 = points[points.len() / 2];
        // Connections over the same pair of points stack their labels.
        let stacked = if variant == Variant::D {
            drawn.iter().filter(|(p, q)| (*p == a && *q == b) || (*p == b && *q == a)).count() as f32
        } else {
            0.0
        };
        drawn.push((a, b));
        out.add(Shape::Polyline { points: points.clone(), dashed: false });
        let label_at = Point::new((mid.x + mid2.x) / 2.0 + 4.0, (mid.y + mid2.y) / 2.0 - 5.0 + stacked * 12.0 + if stacked > 0.0 { 6.0 } else { 0.0 });
        out.text(label_at, &conn.label, TextAnchor::Start, 10.0);
    }

    // Ports last, on top of the lines.
    for port in &node.ports {
        let p = port_point(rect, port);
        out.port(Shape::Rect { rect: Rect { x: p.x - PORT / 2.0, y: p.y - PORT / 2.0, width: PORT, height: PORT }, fill: Fill::Background, corner_radius: 0.0 });
        let (dx, dy) = outward(port.side);
        let (at, anchor) = match port.side {
            Side::Left => (Point::new(p.x - 8.0, p.y - 8.0), TextAnchor::End),
            Side::Right => (Point::new(p.x + 8.0, p.y - 8.0), TextAnchor::Start),
            _ => (Point::new(p.x + 8.0, p.y + dy * 14.0 + if dy > 0.0 { 8.0 } else { 0.0 }), TextAnchor::Start),
        };
        let _ = dx;
        out.ports.push(Item {
            depicts: Depicts::Element(qn("prototype")),
            shape: Shape::Text { position: at, text: port.label.clone(), anchor, font: Font { size: 10.0, bold: false, italic: false } },
        });
    }
}

fn render(model: &Model, root: &str, label: &str, open: &dyn Fn(&str) -> bool, variant: Variant) -> String {
    let mut node = build(model, Some(&qn(root)), label.to_owned(), "root", open);
    layout(&mut node, variant);
    let mut painter = Painter { items: vec![], ports: vec![] };
    let margin = 120.0;
    paint(&node, margin, 40.0, variant, &mut painter);
    let mut items = painter.items;
    items.extend(painter.ports);
    let scene = Scene { x: 0.0, y: 0.0, width: node.w + 2.0 * margin, height: node.h + 80.0, items };
    svg::render(&scene)
}

fn structure() -> Model {
    Model::builder()
        .package("Structure")
        .port_def("Structure::FuelPort")
        .part_def("Structure::Engine")
        .port("Structure::Engine::fuelIn")
        .typed_by("Structure::FuelPort")
        .conjugated()
        .direction(Direction::In)
        .part_def("Structure::Tank")
        .port("Structure::Tank::fuelOut")
        .typed_by("Structure::FuelPort")
        .direction(Direction::Out)
        .interface_def("Structure::FuelInterface")
        .connection_def("Structure::FuelLine")
        .part_def("Structure::Vehicle")
        .part("Structure::Vehicle::engine")
        .typed_by("Structure::Engine")
        .multiplicity("1", "1")
        .part("Structure::Vehicle::tanks")
        .typed_by("Structure::Tank")
        .multiplicity("1", "2")
        .connection("Structure::Vehicle::line")
        .typed_by("Structure::FuelLine")
        .end(&["Structure::Vehicle::tanks", "Structure::Tank::fuelOut"])
        .end(&["Structure::Vehicle::engine", "Structure::Engine::fuelIn"])
        .interface("Structure::Vehicle::iface")
        .typed_by("Structure::FuelInterface")
        .end(&["Structure::Vehicle::tanks", "Structure::Tank::fuelOut"])
        .end(&["Structure::Vehicle::engine", "Structure::Engine::fuelIn"])
        .build()
}

/// After slide 40 of the notation reference: a drivetrain with a nested
/// rear axle assembly, and a controller.
fn drivetrain() -> Model {
    let m = "VehicleModel";
    let q = |s: &str| format!("{m}::{s}");
    let mut b = Model::builder().package(m);
    for d in ["FuelPort", "TorquePort", "ControlPort", "PowerPort"] {
        b = b.port_def(&q(d));
    }
    let port = |b: viewer_core::model::ModelBuilder, owner: &str, name: &str, ty: &str, dir: Option<Direction>, conj: bool| {
        let mut b = b.port(&q(&format!("{owner}::{name}"))).typed_by(&q(ty));
        if conj {
            b = b.conjugated();
        }
        if let Some(d) = dir {
            b = b.direction(d);
        }
        b
    };
    b = b.part_def(&q("FuelTank"));
    b = port(b, "FuelTank", "fuelOut", "FuelPort", Some(Direction::Out), false);
    b = b.part_def(&q("Engine"));
    b = port(b, "Engine", "fuelIn", "FuelPort", Some(Direction::In), true);
    b = port(b, "Engine", "torqueOut", "TorquePort", Some(Direction::Out), false);
    b = port(b, "Engine", "status", "ControlPort", None, false);
    b = port(b, "Engine", "power", "PowerPort", Some(Direction::In), true);
    b = b.part_def(&q("Transmission"));
    b = port(b, "Transmission", "torqueIn", "TorquePort", Some(Direction::In), true);
    b = port(b, "Transmission", "torqueOut", "TorquePort", Some(Direction::Out), false);
    b = b.part_def(&q("Driveshaft"));
    b = port(b, "Driveshaft", "torqueIn", "TorquePort", Some(Direction::In), true);
    b = port(b, "Driveshaft", "torqueOut", "TorquePort", Some(Direction::Out), false);
    b = b.part_def(&q("Differential"));
    b = port(b, "Differential", "torqueIn", "TorquePort", Some(Direction::In), true);
    b = port(b, "Differential", "left", "TorquePort", Some(Direction::Out), false);
    b = port(b, "Differential", "right", "TorquePort", Some(Direction::Out), false);
    b = b.part_def(&q("Wheel"));
    b = port(b, "Wheel", "hub", "TorquePort", Some(Direction::In), true);
    b = b.part_def(&q("AxleAssembly"));
    b = port(b, "AxleAssembly", "torqueIn", "TorquePort", Some(Direction::In), true);
    for (name, ty) in [("differential", "Differential"), ("leftWheel", "Wheel"), ("rightWheel", "Wheel")] {
        b = b.part(&q(&format!("AxleAssembly::{name}"))).typed_by(&q(ty));
    }
    let a = |s: &str| q(&format!("AxleAssembly::{s}"));
    b = b.connection(&a("drive")).end(&[&a("torqueIn")]).end(&[&a("differential"), &q("Differential::torqueIn")]);
    b = b.connection(&a("toLeft")).end(&[&a("differential"), &q("Differential::left")]).end(&[&a("leftWheel"), &q("Wheel::hub")]);
    b = b.connection(&a("toRight")).end(&[&a("differential"), &q("Differential::right")]).end(&[&a("rightWheel"), &q("Wheel::hub")]);
    b = b.part_def(&q("Controller"));
    b = port(b, "Controller", "status", "ControlPort", None, true);
    b = b.part_def(&q("Vehicle"));
    b = port(b, "Vehicle", "power", "PowerPort", Some(Direction::In), true);
    for (name, ty) in [
        ("fuelTank", "FuelTank"),
        ("engine", "Engine"),
        ("transmission", "Transmission"),
        ("driveshaft", "Driveshaft"),
        ("rearAxleAssembly", "AxleAssembly"),
        ("controller", "Controller"),
    ] {
        b = b.part(&q(&format!("Vehicle::{name}"))).typed_by(&q(ty));
    }
    let v = |s: &str| q(&format!("Vehicle::{s}"));
    b = b.interface(&v("fuelLine")).typed_by(&q("FuelPort")).end(&[&v("fuelTank"), &q("FuelTank::fuelOut")]).end(&[&v("engine"), &q("Engine::fuelIn")]);
    b = b.connection(&v("torque")).end(&[&v("engine"), &q("Engine::torqueOut")]).end(&[&v("transmission"), &q("Transmission::torqueIn")]);
    b = b.connection(&v("shaft")).end(&[&v("transmission"), &q("Transmission::torqueOut")]).end(&[&v("driveshaft"), &q("Driveshaft::torqueIn")]);
    b = b.connection(&v("axle")).end(&[&v("driveshaft"), &q("Driveshaft::torqueOut")]).end(&[&v("rearAxleAssembly"), &q("AxleAssembly::torqueIn")]);
    b = b.connection(&v("engineStatus")).end(&[&v("engine"), &q("Engine::status")]).end(&[&v("controller"), &q("Controller::status")]);
    b = b.connection(&v("supply")).end(&[&v("power")]).end(&[&v("engine"), &q("Engine::power")]);
    b.build()
}

fn main() {
    let out = PathBuf::from(std::env::args().nth(1).expect("output folder"));
    std::fs::create_dir_all(&out).unwrap();
    let structure = structure();
    let drivetrain = drivetrain();
    let closed = |p: &str| p == "root";
    let opened = |p: &str| p == "root" || p == "root.rearAxleAssembly";
    for variant in [Variant::A, Variant::B, Variant::C, Variant::D] {
        let v = format!("{variant:?}");
        let files = [
            ("vehicle", render(&structure, "Structure::Vehicle", "Vehicle", &closed, variant)),
            ("drivetrain", render(&drivetrain, "VehicleModel::Vehicle", "vehicle_b:Vehicle", &closed, variant)),
            ("drivetrain-opened", render(&drivetrain, "VehicleModel::Vehicle", "vehicle_b:Vehicle", &opened, variant)),
        ];
        for (name, svg) in files {
            std::fs::write(out.join(format!("{name}-{v}.svg")), svg).unwrap();
        }
    }
    println!("wrote 12 SVGs to {}", out.display());
}
