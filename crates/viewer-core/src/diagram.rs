//! Building a Diagram's scene from the Model.

use std::collections::BTreeMap;

use crate::diagram_file::Layout;
use crate::layout;
use crate::model::{Element, ElementKind, Model, QualifiedName};
use crate::notation;
use crate::scene::{Depicts, Item, Point, Rect, Scene};

const MARGIN: f32 = 20.0;
/// Space kept around anything drawn past the nodes' margin.
const EDGE_PADDING: f32 = 4.0;
/// Space between saved nodes and newly placed ones below them.
const NEW_NODES_GAP: f32 = 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagramKind {
    General,
}

impl DiagramKind {
    /// Whether a Diagram of this kind can be rooted at `element`: a General
    /// Diagram shows the members of a package or definition.
    pub fn accepts_root(self, element: &Element) -> bool {
        match self {
            DiagramKind::General => element.kind == ElementKind::Package || element.kind.is_definition(),
        }
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DiagramError {
    #[error("root element `{0}` does not exist in the Model")]
    RootNotFound(QualifiedName),
}

/// Builds the scene for a Diagram of `kind` rooted at `root`, arranged
/// automatically.
pub fn build_scene(model: &Model, kind: DiagramKind, root: &QualifiedName) -> Result<Scene, DiagramError> {
    build_scene_with_layout(model, kind, root, &Layout::default())
}

/// Builds the scene for a Diagram with the user's `layout` applied: saved
/// nodes keep their place and size, the rest are arranged automatically
/// below them, hidden elements are left out and collapsed ones drawn
/// without compartments. Entries for elements not in the Diagram are
/// ignored.
pub fn build_scene_with_layout(
    model: &Model,
    kind: DiagramKind,
    root: &QualifiedName,
    layout: &Layout,
) -> Result<Scene, DiagramError> {
    if model.element(root).is_none() {
        return Err(DiagramError::RootNotFound(root.clone()));
    }
    match kind {
        DiagramKind::General => Ok(general(model, root, layout)),
    }
}

/// Definitions, and part, attribute and port usages, are nodes. Connections
/// are edges; Unsupported Elements are never drawn.
fn is_node_in_general(element: &Element) -> bool {
    element.kind.is_definition()
        || matches!(
            element.kind,
            ElementKind::PartUsage | ElementKind::AttributeUsage | ElementKind::PortUsage
        )
}

enum Edge<'a> {
    Specialization { specific: usize, general: usize },
    Typing { usage: usize, definition: usize },
    Composition { owner: usize, part_type: usize, usage: &'a Element },
    Connection { a: usize, b: usize, connection: &'a Element },
}

/// A General Diagram: the definitions and usages owned by `root`, each
/// listing its features in compartments, with specialization, typing,
/// composition and connection edges between them.
fn general(model: &Model, root: &QualifiedName, saved: &Layout) -> Scene {
    let nodes: Vec<&Element> = model
        .owned_members(root)
        .filter(|e| is_node_in_general(e) && !saved.hidden.contains(&e.qualified_name))
        .collect();
    let index_of = |name: &QualifiedName| nodes.iter().position(|e| &e.qualified_name == name);

    let mut edges = Vec::new();
    for (i, element) in nodes.iter().enumerate() {
        for general in &element.specializes {
            if let Some(general) = index_of(general) {
                edges.push(Edge::Specialization { specific: i, general });
            }
        }
        if let Some(definition) = element.typed_by.as_ref().and_then(index_of) {
            edges.push(Edge::Typing { usage: i, definition });
        }
        for usage in model
            .owned_members(&element.qualified_name)
            .filter(|m| m.kind == ElementKind::PartUsage)
        {
            if let Some(part_type) = usage.typed_by.as_ref().and_then(index_of).filter(|&t| t != i) {
                edges.push(Edge::Composition { owner: i, part_type, usage });
            }
        }
    }
    for connection in model.owned_members(root).filter(|e| e.kind.is_connection()) {
        let joined: Vec<usize> = connection
            .ends
            .iter()
            .filter_map(|end| end.path.first().and_then(index_of))
            .collect();
        if let [a, b] = joined[..] {
            edges.push(Edge::Connection { a, b, connection });
        }
    }

    // Generals, definitions and owners go in layers above what refers to
    // them; connected usages go in neighbouring layers.
    let layering: Vec<(usize, usize)> = edges
        .iter()
        .map(|edge| match *edge {
            Edge::Specialization { specific, general } => (general, specific),
            Edge::Typing { usage, definition } => (definition, usage),
            Edge::Composition { owner, part_type, .. } => (owner, part_type),
            Edge::Connection { a, b, .. } => (a, b),
        })
        .collect();

    let contents: Vec<notation::NodeContent> = nodes
        .iter()
        .map(|e| notation::node_content(e, model, !saved.collapsed.contains(&e.qualified_name)))
        .collect();
    let rects = place(&nodes, &contents, &layering, saved);

    // Edges without a saved route that join the same two nodes are fanned
    // out, numbered from the lower node index so direction doesn't matter.
    let mut parallel: BTreeMap<(usize, usize), usize> = BTreeMap::new();
    let mut fan_slot = Vec::with_capacity(edges.len());
    for edge in &edges {
        let (depicts, from, to) = describe(edge, &nodes);
        let routed = depicts.edge_key().is_some_and(|key| saved.edges.contains_key(&key));
        let pair = (from.min(to), from.max(to));
        fan_slot.push((!routed && from != to).then(|| {
            let slot = parallel.entry(pair).or_default();
            *slot += 1;
            (pair, *slot - 1)
        }));
    }

    let mut items: Vec<Item> = Vec::new();
    for (edge, slot) in edges.iter().zip(&fan_slot) {
        let (depicts, from, to) = describe(edge, &nodes);
        let bends: Vec<Point> = match (depicts.edge_key().and_then(|key| saved.edges.get(&key)), slot) {
            (Some(points), _) => points.iter().map(|&[x, y]| Point::new(x, y)).collect(),
            (None, Some((pair, index))) => {
                notation::fan_bends(rects[pair.0], rects[pair.1], *index, parallel[pair])
            }
            (None, None) => Vec::new(),
        };
        let path = notation::route(rects[from], rects[to], &bends);
        items.extend(match *edge {
            Edge::Specialization { .. } => notation::specialization(depicts, path),
            Edge::Typing { .. } => notation::defined_by(depicts, path),
            Edge::Composition { usage, .. } => notation::composition(depicts, usage, path),
            Edge::Connection { connection, .. } => notation::connection(depicts, connection, path),
        });
    }
    for ((element, content), rect) in nodes.iter().zip(&contents).zip(&rects) {
        items.extend(notation::node(element, content, *rect));
    }

    // The page holds every node with a margin, grown to anything drawn
    // beyond that (such as labels beside edges).
    let mut left: f32 = 0.0;
    let mut top: f32 = 0.0;
    let mut right = rects.iter().map(Rect::right).fold(0.0, f32::max) + MARGIN;
    let mut bottom = rects.iter().map(Rect::bottom).fold(0.0, f32::max) + MARGIN;
    for item in &items {
        let e = notation::extent(&item.shape);
        left = left.min((e.x - EDGE_PADDING).floor());
        top = top.min((e.y - EDGE_PADDING).floor());
        right = right.max((e.right() + EDGE_PADDING).ceil());
        bottom = bottom.max((e.bottom() + EDGE_PADDING).ceil());
    }
    Scene { x: left, y: top, width: right - left, height: bottom - top, items }
}

/// What `edge` depicts, and the indices of the nodes it runs from and to.
fn describe(edge: &Edge, nodes: &[&Element]) -> (Depicts, usize, usize) {
    let name = |i: usize| nodes[i].qualified_name.clone();
    match *edge {
        Edge::Specialization { specific, general } => {
            (Depicts::Specialization { specific: name(specific), general: name(general) }, specific, general)
        }
        Edge::Typing { usage, definition } => {
            (Depicts::Typing { usage: name(usage), definition: name(definition) }, usage, definition)
        }
        Edge::Composition { owner, part_type, usage } => {
            (Depicts::Composition { owner: name(owner), usage: usage.qualified_name.clone() }, owner, part_type)
        }
        Edge::Connection { a, b, connection } => (Depicts::Connection(connection.qualified_name.clone()), a, b),
    }
}

/// Node rects: saved placements as they are; every other node arranged
/// automatically (with the edges among them) and set below the saved ones,
/// so nothing saved moves and nothing new overlaps it.
fn place(
    nodes: &[&Element],
    contents: &[notation::NodeContent],
    layering: &[(usize, usize)],
    saved: &Layout,
) -> Vec<Rect> {
    let saved_rect = |i: usize| {
        saved.nodes.get(&nodes[i].qualified_name).map(|p| Rect { x: p.x, y: p.y, width: p.width, height: p.height })
    };
    let unplaced: Vec<usize> = (0..nodes.len()).filter(|&i| saved_rect(i).is_none()).collect();
    let local = |i: usize| unplaced.iter().position(|&u| u == i);
    let sizes: Vec<(f32, f32)> = unplaced.iter().map(|&i| notation::node_size(&contents[i])).collect();
    let edges: Vec<(usize, usize)> = layering
        .iter()
        .filter_map(|&(a, b)| Some((local(a)?, local(b)?)))
        .collect();
    let auto = layout::layered(&sizes, &edges);

    let saved_rects: Vec<Rect> = (0..nodes.len()).filter_map(saved_rect).collect();
    let (left, top) = if saved_rects.is_empty() {
        (MARGIN, MARGIN)
    } else {
        let left = saved_rects.iter().map(|r| r.x).fold(f32::INFINITY, f32::min).max(0.0);
        let bottom = saved_rects.iter().map(Rect::bottom).fold(0.0, f32::max);
        (left, bottom + NEW_NODES_GAP)
    };
    (0..nodes.len())
        .map(|i| match saved_rect(i) {
            Some(rect) => rect,
            None => {
                let r = auto[local(i).expect("an unplaced node has an automatic place")];
                Rect { x: r.x + left, y: r.y + top, ..r }
            }
        })
        .collect()
}
