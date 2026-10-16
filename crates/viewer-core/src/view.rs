//! Reading a SysML `view` as a Model-defined Diagram: which Diagram Kind it
//! is drawn as, and what it shows.

use std::collections::{BTreeSet, HashSet, VecDeque};

use crate::diagram::{DiagramKind, build_general_scene, build_scene_with_layout, is_node_in_general};
use crate::diagram_file::Layout;
use crate::model::{Element, ElementKind, ExposeScope, Model, QualifiedName};
use crate::scene::Scene;

const TREE: &str = "Views::asTreeDiagram";
const INTERCONNECTION: &str = "Views::asInterconnectionDiagram";
const GENERAL_VIEW: &str = "StandardViewDefinitions::GeneralView";
const INTERCONNECTION_VIEW: &str = "StandardViewDefinitions::InterconnectionView";

/// What a supported view draws.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ViewContents {
    /// A General Diagram of exactly these elements, in qualified-name order.
    General(Vec<QualifiedName>),
    /// An Interconnection Diagram rooted at this part or part def.
    Interconnection(QualifiedName),
}

impl ViewContents {
    pub fn kind(&self) -> DiagramKind {
        match self {
            ViewContents::General(_) => DiagramKind::General,
            ViewContents::Interconnection(_) => DiagramKind::Interconnection,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ViewError {
    #[error("`{0}` is not a view in the Model")]
    NotAView(QualifiedName),
    /// An Unsupported View: listed, but no Diagram Kind draws it.
    #[error("`{view}` can't be drawn: {reason}")]
    Unsupported { view: QualifiedName, reason: String },
    /// A supported view whose exposes don't fit its Diagram Kind.
    #[error("`{view}` can't be drawn: {reason}")]
    Invalid { view: QualifiedName, reason: String },
}

/// Every view usage in the Model, in qualified-name order.
pub fn views(model: &Model) -> impl Iterator<Item = &Element> {
    model.elements().filter(|e| e.kind == ElementKind::ViewUsage)
}

/// The Diagram Kind `view` is drawn as, from its own rendering, else the
/// first rendering on its definitions (breadth first), else the standard
/// view definition it specializes. An Unsupported View gives the reason.
pub fn kind(model: &Model, view: &QualifiedName) -> Result<DiagramKind, ViewError> {
    let element = model
        .element(view)
        .filter(|e| e.kind == ElementKind::ViewUsage)
        .ok_or_else(|| ViewError::NotAView(view.clone()))?;
    let unsupported = |reason: String| ViewError::Unsupported { view: view.clone(), reason };

    let mut standard = None;
    let mut seen = HashSet::new();
    let mut queue = VecDeque::from([element]);
    while let Some(current) = queue.pop_front() {
        if let Some(rendering) = &current.rendering {
            return match rendering.as_str() {
                TREE => Ok(DiagramKind::General),
                INTERCONNECTION => Ok(DiagramKind::Interconnection),
                _ => Err(unsupported(format!("it renders as `{}`", rendering.simple_name()))),
            };
        }
        for general in current.typed_by.iter().chain(&current.specializes) {
            if !seen.insert(general.clone()) {
                continue;
            }
            match model.element(general) {
                Some(definition) => queue.push_back(definition),
                None if standard.is_none() && general.as_str().starts_with("StandardViewDefinitions::") => {
                    standard = Some(general.clone());
                }
                None => {}
            }
        }
    }
    match standard.as_ref().map(QualifiedName::as_str) {
        Some(GENERAL_VIEW) => Ok(DiagramKind::General),
        Some(INTERCONNECTION_VIEW) => Ok(DiagramKind::Interconnection),
        Some(_) => Err(unsupported(format!("it is a `{}`", standard.as_ref().unwrap().simple_name()))),
        None => Err(unsupported("it has no rendering".to_owned())),
    }
}

/// What `view` draws: the Exposed Elements a General Diagram shows, or the
/// root of an Interconnection Diagram.
pub fn resolve(model: &Model, view: &QualifiedName) -> Result<ViewContents, ViewError> {
    let kind = kind(model, view)?;
    let exposes = &model.element(view).expect("kind() found the view").exposes;
    match kind {
        DiagramKind::General => {
            let mut shown = BTreeSet::new();
            for expose in exposes {
                let target = model.element(&expose.target);
                let is_package = target.is_some_and(|t| t.kind == ElementKind::Package);
                let itself = matches!(expose.scope, ExposeScope::Element | ExposeScope::ElementAndDescendants);
                if itself && !is_package {
                    shown.extend(target.map(|t| &t.qualified_name));
                }
                match expose.scope {
                    // A package isn't drawn, so exposing one shows its members.
                    ExposeScope::Element if is_package => shown.extend(members(model, &expose.target)),
                    ExposeScope::Element => {}
                    ExposeScope::Members => shown.extend(members(model, &expose.target)),
                    ExposeScope::ElementAndDescendants | ExposeScope::Descendants => {
                        descendants(model, &expose.target, &mut shown)
                    }
                }
            }
            let nodes = shown
                .into_iter()
                .filter(|name| model.element(name).is_some_and(is_node_in_general))
                .cloned()
                .collect();
            Ok(ViewContents::General(nodes))
        }
        DiagramKind::Interconnection => {
            let invalid = |reason: String| ViewError::Invalid { view: view.clone(), reason };
            let targets: BTreeSet<&QualifiedName> = exposes.iter().map(|e| &e.target).collect();
            let root = match targets.into_iter().collect::<Vec<_>>().as_slice() {
                [] => return Err(invalid("it exposes nothing to draw as an Interconnection Diagram".to_owned())),
                [root] => (*root).clone(),
                many => {
                    let names: Vec<String> = many.iter().map(|n| format!("`{n}`")).collect();
                    return Err(invalid(format!(
                        "an Interconnection Diagram needs exactly one exposed part, but it exposes {}",
                        names.join(", ")
                    )));
                }
            };
            match model.element(&root) {
                Some(element) if DiagramKind::Interconnection.accepts_root(element) => {
                    Ok(ViewContents::Interconnection(root))
                }
                Some(_) => Err(invalid(format!("`{root}` is not a part or part def"))),
                None => Err(invalid(format!("`{root}` is not in the Model"))),
            }
        }
    }
}

/// Draws `view` as its Model-defined Diagram, with the user's `layout`.
pub fn build_scene(model: &Model, view: &QualifiedName, layout: &Layout) -> Result<Scene, ViewError> {
    match resolve(model, view)? {
        ViewContents::General(nodes) => Ok(build_general_scene(model, &nodes, layout)),
        ViewContents::Interconnection(root) => build_scene_with_layout(model, DiagramKind::Interconnection, &root, layout)
            .map_err(|error| ViewError::Invalid { view: view.clone(), reason: error.to_string() }),
    }
}

/// Whether a Layout key of `view`'s Diagram still names something in the
/// Model, as [`crate::diagram::layout_key_exists`] does for its kind. Keys
/// of a view that no longer resolves are all kept as orphaned.
pub fn layout_key_exists(model: &Model, view: &QualifiedName, key: &QualifiedName) -> bool {
    match resolve(model, view) {
        Ok(ViewContents::General(_)) => model.element(key).is_some(),
        Ok(ViewContents::Interconnection(root)) => {
            crate::diagram::layout_key_exists(model, DiagramKind::Interconnection, &root, key)
        }
        Err(_) => true,
    }
}

fn members<'a>(model: &'a Model, owner: &'a QualifiedName) -> impl Iterator<Item = &'a QualifiedName> {
    model.owned_members(owner).map(|e| &e.qualified_name)
}

fn descendants<'a>(model: &'a Model, owner: &'a QualifiedName, out: &mut BTreeSet<&'a QualifiedName>) {
    for member in model.owned_members(owner) {
        if out.insert(&member.qualified_name) {
            descendants(model, &member.qualified_name, out);
        }
    }
}
