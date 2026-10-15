//! Layout editing: the changes a user makes to a Diagram's arrangement,
//! applied to its Layout. The Model is never touched (ADR-0002).

use crate::diagram_file::{Layout, NodePlacement};
use crate::model::{Model, QualifiedName};
use crate::scene::Scene;

/// The smallest size a node can be resized to.
pub const MIN_NODE_SIZE: [f32; 2] = [40.0, 24.0];

#[derive(Clone, Debug, PartialEq)]
pub enum Edit {
    /// Moves a node by a scene-coordinate offset; it stops at the page's
    /// top and left at zero.
    Move { element: QualifiedName, dx: f32, dy: f32 },
    /// Sets a node's size, no smaller than [`MIN_NODE_SIZE`].
    Resize { element: QualifiedName, width: f32, height: f32 },
    Hide(QualifiedName),
    Show(QualifiedName),
    ShowAll,
    /// Draws a node without its compartments, at its natural size.
    Collapse(QualifiedName),
    Expand(QualifiedName),
    /// Back to the automatic arrangement: the Layout is emptied, apart from
    /// Orphaned Layout Entries, which are never dropped silently.
    Reset,
}

/// Applies `edit` to `layout`. `scene` is the Diagram as currently drawn
/// from `model`: every edit except `Reset` first pins each drawn node where
/// it is, so changing one node never rearranges the others. An edit that
/// changes nothing leaves the Layout exactly as it was.
pub fn apply(layout: &mut Layout, model: &Model, scene: &Scene, edit: &Edit) {
    let mut pinned = layout.clone();
    if *edit != Edit::Reset {
        pin(&mut pinned, scene);
    }
    let mut edited = pinned.clone();
    change(&mut edited, model, edit);
    if edited != pinned {
        *layout = edited;
    }
}

fn change(layout: &mut Layout, model: &Model, edit: &Edit) {
    match edit {
        Edit::Move { element, dx, dy } => {
            if let Some(placement) = layout.nodes.get_mut(element) {
                placement.x = (placement.x + dx).max(0.0);
                placement.y = (placement.y + dy).max(0.0);
            }
        }
        Edit::Resize { element, width, height } => {
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
        Edit::Reset => {
            let orphaned = |e: &QualifiedName| model.element(e).is_none();
            layout.nodes.retain(|e, _| orphaned(e));
            layout.hidden.retain(|e| orphaned(e));
            layout.collapsed.retain(|e| orphaned(e));
            layout.edges.clear();
        }
    }
}

/// Saves the position of every drawn node that has none yet.
fn pin(layout: &mut Layout, scene: &Scene) {
    for element in scene.elements() {
        if let Some(bounds) = scene.node_bounds(element) {
            layout
                .nodes
                .entry(element.clone())
                .or_insert(NodePlacement { x: bounds.x, y: bounds.y, size: None });
        }
    }
}

fn natural_size(layout: &mut Layout, element: &QualifiedName) {
    if let Some(placement) = layout.nodes.get_mut(element) {
        placement.size = None;
    }
}
