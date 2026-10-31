//! Layout editing: the changes a user makes to a Diagram's arrangement,
//! applied to its Layout. The Model is never touched (ADR-0002).

use crate::diagram_file::{Layout, NodePlacement, PortPlacement, PortSide};
use crate::model::{Model, QualifiedName};
use crate::scene::{Point, Rect, Scene};

/// The smallest size a node can be resized to.
pub const MIN_NODE_SIZE: [f32; 2] = [40.0, 24.0];

/// Edits name nodes by their key (see [`crate::scene::Depicts::node_key`]):
/// their element in a General Diagram, their feature path in an
/// Interconnection Diagram.
#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    /// Moves nodes by a scene-coordinate offset, together: they all stop
    /// when one reaches the top or left of the page (or of the box it's
    /// drawn in). The bends of an edge whose ends are both on moved nodes
    /// move with them.
    Move {
        elements: Vec<QualifiedName>,
        dx: f32,
        dy: f32,
    },
    /// Sets a node's size, no smaller than [`MIN_NODE_SIZE`].
    Resize {
        element: QualifiedName,
        width: f32,
        height: f32,
    },
    Hide(QualifiedName),
    Show(QualifiedName),
    ShowAll,
    /// Draws a node without its compartments, at its natural size.
    Collapse(QualifiedName),
    Expand(QualifiedName),
    /// Leaves Inherited Members out of a General Diagram's compartments, or
    /// lists them again. No node moves: boxes change height where they are,
    /// and those never placed rearrange at their new size.
    HideInherited,
    ShowInherited,
    /// Sets the bend points of the edge with key `edge` (see
    /// [`crate::scene::Depicts::edge_key`]), in scene coordinates from
    /// source to target. No bends returns it to automatic routing.
    Route {
        edge: String,
        bends: Vec<[f32; 2]>,
    },
    /// Puts the port with key `port` (its feature path) on `side` of its
    /// box, `offset` along it.
    MovePort {
        port: String,
        side: PortSide,
        offset: f32,
    },
    /// Opens or closes the part at a feature path in an Interconnection
    /// Diagram.
    Open(String),
    Close(String),
    /// Lines `elements` up with `reference` along one of its edges or
    /// centre lines. Only the others move, and only across that line.
    Align {
        elements: Vec<QualifiedName>,
        reference: QualifiedName,
        to: Alignment,
    },
    /// Gives `elements` the drawn width, height or both of `reference`.
    /// Their top-left corners stay; collapsed nodes keep their natural size.
    MatchSize {
        elements: Vec<QualifiedName>,
        reference: QualifiedName,
        dimension: Dimension,
    },
    /// Spaces three or more `elements` along `axis` so that the gaps
    /// between neighbouring boxes are equal. The two outermost stay put and
    /// the others keep their order.
    Distribute {
        elements: Vec<QualifiedName>,
        axis: Axis,
    },
    /// Back to the automatic arrangement: positions, sizes, routes, moved
    /// ports and hidden and collapsed elements are dropped, apart from
    /// Orphaned Layout Entries, which are never dropped silently. Opened
    /// Parts stay open: which parts to show is not arrangement.
    Reset,
}

/// The edge or centre line an [`Edit::Align`] lines nodes up on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Alignment {
    Left,
    /// Centres on one vertical line.
    HorizontalCenter,
    Right,
    Top,
    /// Centres on one horizontal line.
    VerticalCenter,
    Bottom,
}

/// What an [`Edit::MatchSize`] copies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dimension {
    Width,
    Height,
    Both,
}

/// The direction an [`Edit::Distribute`] spaces nodes along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

/// Applies `edit` to a General Diagram's `layout`; see [`apply_with`].
pub fn apply(layout: &mut Layout, model: &Model, scene: &Scene, edit: &Edit) {
    apply_with(layout, &|key| model.element(key).is_some(), scene, edit);
}

/// Applies `edit` to `layout`. `scene` is the Diagram as currently drawn:
/// every edit except `Reset` and showing or hiding Inherited Members first
/// pins each drawn node where it is, so changing one node never rearranges
/// the others. `known` tells whether a node key still names something in
/// the Model, so `Reset` keeps Orphaned Layout Entries. An edit that
/// changes nothing leaves the Layout exactly as it was.
pub fn apply_with(
    layout: &mut Layout,
    known: &dyn Fn(&QualifiedName) -> bool,
    scene: &Scene,
    edit: &Edit,
) {
    let mut pinned = layout.clone();
    if !matches!(
        edit,
        Edit::Reset | Edit::HideInherited | Edit::ShowInherited
    ) {
        pin(&mut pinned, scene);
    }
    let mut edited = pinned.clone();
    change(&mut edited, known, scene, edit);
    if edited != pinned {
        *layout = edited;
    }
}

fn change(layout: &mut Layout, known: &dyn Fn(&QualifiedName) -> bool, scene: &Scene, edit: &Edit) {
    match edit {
        Edit::Move { elements, dx, dy } => move_nodes(layout, scene, elements, *dx, *dy),
        Edit::Resize {
            element,
            width,
            height,
        } => {
            if let Some(placement) = layout.nodes.get_mut(element) {
                placement.size = Some([width.max(MIN_NODE_SIZE[0]), height.max(MIN_NODE_SIZE[1])]);
            }
        }
        Edit::Hide(element) => {
            layout.hidden.insert(element.clone());
        }
        Edit::Show(element) => {
            layout.hidden.remove(element);
        }
        Edit::ShowAll => layout.hidden.clear(),
        Edit::HideInherited => layout.inherited_hidden = true,
        Edit::ShowInherited => layout.inherited_hidden = false,
        Edit::Collapse(element) => {
            layout.collapsed.insert(element.clone());
            natural_size(layout, element);
        }
        Edit::Expand(element) => {
            layout.collapsed.remove(element);
            natural_size(layout, element);
        }
        Edit::Route { edge, bends } => {
            if bends.is_empty() {
                layout.edges.remove(edge);
            } else {
                layout.edges.insert(edge.clone(), bends.clone());
            }
        }
        Edit::MovePort { port, side, offset } => {
            layout.ports.insert(
                port.clone(),
                PortPlacement {
                    side: *side,
                    offset: offset.max(0.0),
                },
            );
        }
        Edit::Open(path) => {
            layout.opened.insert(path.clone());
        }
        Edit::Close(path) => {
            layout.opened.remove(path);
        }
        Edit::Align {
            elements,
            reference,
            to,
        } => align(layout, scene, elements, reference, *to),
        Edit::MatchSize {
            elements,
            reference,
            dimension,
        } => match_size(layout, scene, elements, reference, *dimension),
        Edit::Distribute { elements, axis } => distribute(layout, scene, elements, *axis),
        Edit::Reset => {
            let orphaned = |e: &QualifiedName| !known(e);
            layout.nodes.retain(|e, _| orphaned(e));
            layout.hidden.retain(|e| orphaned(e));
            layout.collapsed.retain(|e| orphaned(e));
            layout.edges.clear();
            // A port is orphaned with the part it's on (a Frame port's part
            // is the root, which is always known).
            layout.ports.retain(|port, _| {
                port.rsplit_once('.')
                    .is_some_and(|(part, _)| orphaned(&QualifiedName::new(part)))
            });
        }
    }
}

/// Saves the position of every drawn node that has none yet: on the page,
/// or relative to the box it's drawn in. In an Interconnection Diagram
/// (whose scene lists box origins) the Frame isn't pinned.
fn pin(layout: &mut Layout, scene: &Scene) {
    let nested = !scene.origins.is_empty();
    for key in scene.elements() {
        let origin = scene.origins.get(key);
        if nested && origin.is_none() {
            continue;
        }
        if let Some(bounds) = scene.node_bounds(key) {
            let (ox, oy) = origin.map_or((0.0, 0.0), |o| (o.x, o.y));
            layout.nodes.entry(key.clone()).or_insert(NodePlacement {
                x: bounds.x - ox,
                y: bounds.y - oy,
                size: None,
            });
        }
    }
}

/// Moves `elements` by the largest part of (`dx`, `dy`) that keeps every
/// one of them at or past zero, and the bends of edges between them.
fn move_nodes(layout: &mut Layout, scene: &Scene, elements: &[QualifiedName], dx: f32, dy: f32) {
    let placed: Vec<&QualifiedName> = elements
        .iter()
        .filter(|e| layout.nodes.contains_key(*e))
        .collect();
    let (mut dx, mut dy) = (dx, dy);
    for element in &placed {
        let placement = &layout.nodes[*element];
        dx = dx.max(-placement.x.max(0.0));
        dy = dy.max(-placement.y.max(0.0));
    }
    if placed.is_empty() || (dx == 0.0 && dy == 0.0) {
        return;
    }
    for element in &placed {
        let placement = layout.nodes.get_mut(*element).expect("placed above");
        placement.x += dx;
        placement.y += dy;
    }
    // An edge attaches to the node nearest each of its ends (they can
    // stop short of it at an arrowhead), the innermost one where boxes
    // nest. A node drawn inside a moved one moves too.
    let nodes: Vec<Rect> = scene
        .elements()
        .into_iter()
        .filter_map(|e| scene.node_bounds(e))
        .collect();
    let moved: Vec<Rect> = placed.iter().filter_map(|e| scene.node_bounds(e)).collect();
    let on_moved = |p: &Point| {
        let nearest = nodes.iter().min_by(|a, b| {
            let key = |r: &Rect| (distance(r, *p), r.width * r.height);
            key(a).partial_cmp(&key(b)).expect("finite")
        });
        nearest.is_some_and(|n| moved.iter().any(|m| inside(n, m)))
    };
    for (depicts, path) in scene.edges() {
        let (Some(key), Some(first), Some(last)) = (depicts.edge_key(), path.first(), path.last())
        else {
            continue;
        };
        if on_moved(first)
            && on_moved(last)
            && let Some(bends) = layout.edges.get_mut(&key)
        {
            for bend in bends {
                bend[0] += dx;
                bend[1] += dy;
            }
        }
    }
}

fn align(
    layout: &mut Layout,
    scene: &Scene,
    elements: &[QualifiedName],
    reference: &QualifiedName,
    to: Alignment,
) {
    let Some(target) = scene.node_bounds(reference) else {
        return;
    };
    for element in elements.iter().filter(|e| *e != reference) {
        let Some(b) = scene.node_bounds(element) else {
            continue;
        };
        let (dx, dy) = match to {
            Alignment::Left => (target.x - b.x, 0.0),
            Alignment::HorizontalCenter => (target.center_x() - b.center_x(), 0.0),
            Alignment::Right => (target.right() - b.right(), 0.0),
            Alignment::Top => (0.0, target.y - b.y),
            Alignment::VerticalCenter => (
                0.0,
                (target.y + target.height / 2.0) - (b.y + b.height / 2.0),
            ),
            Alignment::Bottom => (0.0, target.bottom() - b.bottom()),
        };
        shift(layout, element, dx, dy);
    }
}

fn match_size(
    layout: &mut Layout,
    scene: &Scene,
    elements: &[QualifiedName],
    reference: &QualifiedName,
    dimension: Dimension,
) {
    let Some(target) = scene.node_bounds(reference) else {
        return;
    };
    if layout.collapsed.contains(reference) {
        return;
    }
    for element in elements.iter().filter(|e| *e != reference) {
        if layout.collapsed.contains(element) {
            continue;
        }
        let (Some(b), Some(placement)) =
            (scene.node_bounds(element), layout.nodes.get_mut(element))
        else {
            continue;
        };
        let (width, height) = match dimension {
            Dimension::Width => (target.width, b.height),
            Dimension::Height => (b.width, target.height),
            Dimension::Both => (target.width, target.height),
        };
        placement.size = Some([width.max(MIN_NODE_SIZE[0]), height.max(MIN_NODE_SIZE[1])]);
    }
}

fn distribute(layout: &mut Layout, scene: &Scene, elements: &[QualifiedName], axis: Axis) {
    // Each node's start and length along the axis.
    let span = |b: Rect| match axis {
        Axis::Horizontal => (b.x, b.width),
        Axis::Vertical => (b.y, b.height),
    };
    let mut nodes: Vec<(&QualifiedName, f32, f32)> = elements
        .iter()
        .filter_map(|e| scene.node_bounds(e).map(|b| (e, span(b).0, span(b).1)))
        .collect();
    if nodes.len() < 3 {
        return;
    }
    nodes.sort_by(|a, b| a.1.partial_cmp(&b.1).expect("finite"));
    let (first, last) = (nodes[0], nodes[nodes.len() - 1]);
    let lengths: f32 = nodes.iter().map(|n| n.2).sum();
    let gap = (last.1 + last.2 - first.1 - lengths) / (nodes.len() - 1) as f32;
    let mut at = first.1 + first.2 + gap;
    for &(element, start, length) in &nodes[1..nodes.len() - 1] {
        match axis {
            Axis::Horizontal => shift(layout, element, at - start, 0.0),
            Axis::Vertical => shift(layout, element, 0.0, at - start),
        }
        at += length + gap;
    }
}

/// Moves one node, stopping at zero as any move does.
fn shift(layout: &mut Layout, element: &QualifiedName, dx: f32, dy: f32) {
    if let Some(placement) = layout.nodes.get_mut(element) {
        placement.x = (placement.x + dx).max(0.0);
        placement.y = (placement.y + dy).max(0.0);
    }
}

/// How far `p` is from `r`: zero inside it.
fn distance(r: &Rect, p: Point) -> f32 {
    let dx = (r.x - p.x).max(p.x - r.right()).max(0.0);
    let dy = (r.y - p.y).max(p.y - r.bottom()).max(0.0);
    dx.hypot(dy)
}

/// Whether `inner` lies within `outer` (or is it).
fn inside(inner: &Rect, outer: &Rect) -> bool {
    inner.x >= outer.x
        && inner.y >= outer.y
        && inner.right() <= outer.right()
        && inner.bottom() <= outer.bottom()
}

fn natural_size(layout: &mut Layout, element: &QualifiedName) {
    if let Some(placement) = layout.nodes.get_mut(element) {
        placement.size = None;
    }
}
