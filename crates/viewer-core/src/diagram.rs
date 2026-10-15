//! Building a Diagram's scene from the Model.

use crate::layout;
use crate::model::{Element, ElementKind, Model, QualifiedName};
use crate::notation;
use crate::scene::{Depicts, Rect, Scene};

const MARGIN: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

fn is_drawn_in_general(element: &Element) -> bool {
    matches!(element.kind, ElementKind::PartDefinition)
}

/// A General Diagram: the definitions owned by `root`, with the
/// specializations between them.
fn general(model: &Model, root: &QualifiedName) -> Scene {
    let nodes: Vec<&Element> = model
        .owned_members(root)
        .filter(|e| is_drawn_in_general(e))
        .collect();
    let index_of = |name: &QualifiedName| nodes.iter().position(|e| &e.qualified_name == name);

    // Each edge is (general, specific), so generals land in upper layers.
    let mut specializations = Vec::new();
    for (specific, element) in nodes.iter().enumerate() {
        for general in &element.specializes {
            if let Some(general) = index_of(general) {
                specializations.push((general, specific));
            }
        }
    }

    let sizes: Vec<(f32, f32)> = nodes.iter().map(|e| notation::node_size(e)).collect();
    let rects: Vec<Rect> = layout::layered(&sizes, &specializations)
        .into_iter()
        .map(|r| Rect { x: r.x + MARGIN, y: r.y + MARGIN, ..r })
        .collect();

    let mut scene = Scene {
        width: rects.iter().map(Rect::right).fold(0.0, f32::max) + MARGIN,
        height: rects.iter().map(Rect::bottom).fold(0.0, f32::max) + MARGIN,
        items: Vec::new(),
    };
    for &(general, specific) in &specializations {
        let depicts = Depicts::Specialization {
            specific: nodes[specific].qualified_name.clone(),
            general: nodes[general].qualified_name.clone(),
        };
        scene
            .items
            .extend(notation::specialization(depicts, rects[specific], rects[general]));
    }
    for (element, rect) in nodes.iter().zip(&rects) {
        scene.items.extend(notation::node(element, *rect));
    }
    scene
}
