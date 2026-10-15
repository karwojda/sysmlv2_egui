//! What the properties panel shows for an element: labelled values, some of
//! which refer to other elements and can be followed.

use crate::model::{Model, QualifiedName, SourceLocation};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub label: &'static str,
    pub value: String,
    /// The element the value refers to, for a type, owner or general.
    pub refers_to: Option<QualifiedName>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Properties {
    pub rows: Vec<Property>,
    /// Where the element is declared, for "open in editor".
    pub source: Option<SourceLocation>,
}

/// The properties of `element`, or `None` if the Model has no such
/// element. Only properties the element has are listed.
pub fn properties(model: &Model, element: &QualifiedName) -> Option<Properties> {
    let e = model.element(element)?;
    let text = |label, value: String| Property { label, value, refers_to: None };
    let link = |label, target: &QualifiedName, value: String| Property {
        label,
        value,
        refers_to: model.element(target).map(|_| target.clone()),
    };

    let mut rows = vec![
        text("Kind", format!("«{}»", e.kind.keyword())),
        text("Name", e.name.clone()),
        text("Qualified name", e.qualified_name.to_string()),
    ];
    if let Some(owner) = &e.owner {
        rows.push(link("Owner", owner, owner.to_string()));
    }
    if let Some(definition) = &e.typed_by {
        let conjugate = if e.conjugated { "~" } else { "" };
        rows.push(link("Type", definition, format!("{conjugate}{definition}")));
    }
    if let Some(multiplicity) = &e.multiplicity {
        rows.push(text("Multiplicity", multiplicity.to_string()));
    }
    if let Some(direction) = e.direction {
        rows.push(text("Direction", direction.keyword().to_owned()));
    }
    for general in &e.specializes {
        rows.push(link("Specializes", general, general.to_string()));
    }
    for (i, end) in e.ends.iter().enumerate() {
        let label = if i == 0 { "Connects" } else { "To" };
        rows.push(text(label, end.to_string()));
    }
    if let Some(documentation) = &e.documentation {
        rows.push(text("Documentation", documentation.clone()));
    }
    if let Some(source) = &e.source {
        rows.push(text("Source", format!("{}:{}", source.file.display(), source.line)));
    }
    Some(Properties { rows, source: e.source.clone() })
}
