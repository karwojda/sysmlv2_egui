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
    /// Back to the automatic arrangement: positions, sizes, routes, moved
    /// ports and hidden and collapsed elements are dropped, apart from
    /// Orphaned Layout Entries, which are never dropped silently. Opened
    /// Parts stay open: which parts to show is not arrangement.
    Reset,
}

/// Applies `edit` to a General Diagram's `layout`; see [`apply_with`].
pub fn apply(layout: &mut Layout, model: &Model, scene: &Scene, edit: &Edit) {
    apply_with(layout, &|key| model.element(key).is_some(), scene, edit);
}

/// Applies `edit` to `layout`. `scene` is the Diagram as currently drawn:
/// every edit except `Reset` first pins each drawn node where it is, so
/// changing one node never rearranges the others. `known` tells whether a
/// node key still names something in the Model, so `Reset` keeps Orphaned
/// Layout Entries. An edit that changes nothing leaves the Layout exactly
/// as it was.
pub fn apply_with(
    layout: &mut Layout,
    known: &dyn Fn(&QualifiedName) -> bool,
    scene: &Scene,
    edit: &Edit,
) {
    let mut pinned = layout.clone();
    if *edit != Edit::Reset {
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
