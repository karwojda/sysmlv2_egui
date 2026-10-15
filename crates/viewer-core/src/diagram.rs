//! Building a Diagram's scene from the Model.

use crate::layout;
use crate::model::{Element, ElementKind, Model, QualifiedName};
use crate::notation;
use crate::scene::{Depicts, Item, Rect, Scene};

const MARGIN: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagramKind {
    General,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DiagramError {
    #[error("root element `{0}` does not exist in the Model")]
    RootNotFound(QualifiedName),
}

/// Builds the scene for a Diagram of `kind` rooted at `root`, arranged
/// automatically.
pub fn build_scene(model: &Model, kind: DiagramKind, root: &QualifiedName) -> Result<Scene, DiagramError> {
    if model.element(root).is_none() {
        return Err(DiagramError::RootNotFound(root.clone()));
    }
    match kind {
        DiagramKind::General => Ok(general(model, root)),
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
fn general(model: &Model, root: &QualifiedName) -> Scene {
    let nodes: Vec<&Element> = model
        .owned_members(root)
        .filter(|e| is_node_in_general(e))
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

    let contents: Vec<notation::NodeContent> = nodes.iter().map(|e| notation::node_content(e, model)).collect();
    let sizes: Vec<(f32, f32)> = contents.iter().map(notation::node_size).collect();
    let rects: Vec<Rect> = layout::layered(&sizes, &layering)
        .into_iter()
        .map(|r| Rect { x: r.x + MARGIN, y: r.y + MARGIN, ..r })
        .collect();

    let mut items: Vec<Item> = Vec::new();
    for edge in &edges {
        let name = |i: usize| nodes[i].qualified_name.clone();
        items.extend(match *edge {
            Edge::Specialization { specific, general } => notation::specialization(
                Depicts::Specialization { specific: name(specific), general: name(general) },
                rects[specific],
                rects[general],
            ),
            Edge::Typing { usage, definition } => notation::defined_by(
                Depicts::Typing { usage: name(usage), definition: name(definition) },
                rects[usage],
                rects[definition],
            ),
            Edge::Composition { owner, part_type, usage } => notation::composition(
                Depicts::Composition { owner: name(owner), usage: usage.qualified_name.clone() },
                usage,
                rects[owner],
                rects[part_type],
            ),
            Edge::Connection { a, b, connection } => notation::connection(
                Depicts::Connection(connection.qualified_name.clone()),
                connection,
                rects[a],
                rects[b],
            ),
        });
    }
    for ((element, content), rect) in nodes.iter().zip(&contents).zip(&rects) {
        items.extend(notation::node(element, content, *rect));
    }

    Scene {
        width: rects.iter().map(Rect::right).fold(0.0, f32::max) + MARGIN,
        height: rects.iter().map(Rect::bottom).fold(0.0, f32::max) + MARGIN,
        items,
    }
}
