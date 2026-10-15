//! The Browser Diagram's content: the Model's ownership tree, optionally
//! narrowed by a name filter.

use crate::model::{Element, ElementKind, Model, QualifiedName};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BrowserEntry {
    pub element: QualifiedName,
    pub name: String,
    /// The SysML keyword, such as `part def`.
    pub keyword: String,
    pub unsupported: bool,
    pub children: Vec<BrowserEntry>,
}

/// The tree of every element, rooted at elements without an owner, in
/// qualified-name order. With a `filter`, only entries whose name contains
/// it (ignoring case) are kept, together with their ancestors.
pub fn tree(model: &Model, filter: &str) -> Vec<BrowserEntry> {
    let filter = filter.trim().to_lowercase();
    model
        .elements()
        .filter(|e| e.owner.is_none())
        .filter_map(|e| entry(model, e, &filter))
        .collect()
}

fn entry(model: &Model, element: &Element, filter: &str) -> Option<BrowserEntry> {
    let children: Vec<BrowserEntry> = model
        .owned_members(&element.qualified_name)
        .filter_map(|child| entry(model, child, filter))
        .collect();
    let matches = filter.is_empty() || element.name.to_lowercase().contains(filter);
    (matches || !children.is_empty()).then(|| BrowserEntry {
        element: element.qualified_name.clone(),
        name: element.name.clone(),
        keyword: element.kind.keyword().to_owned(),
        unsupported: matches!(element.kind, ElementKind::Unsupported(_)),
        children,
    })
}
