//! Design Values (ADR-0003 to ADR-0005): from the Tool Links a Model
//! writes, asks each Engineering Tool once per file for what is linked to
//! it, and says which feature takes which value. The edit that puts those
//! values into the Model in memory is the service's, in `lib.rs`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use viewer_core::engineering_tools::{
    DesignValue, DesignValueOutcome, DesignValueSource, EngineeringTool, ToolLink, Wanted,
};
use viewer_core::model::{Element, QualifiedName};

/// A Tool Link and the document holding it, whose folder its `file` is
/// relative to.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HeldLink {
    pub link: ToolLink,
    /// The document as the parse named it.
    pub document: String,
}

/// A feature with a `@ToolProperty { name = "Volume"; }`.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct LinkedProperty {
    /// The feature, `Drive::drive::engine::volume`.
    pub feature: QualifiedName,
    /// The element whose Tool Link it reads, `Drive::drive::engine`.
    pub owner: QualifiedName,
    /// The tool's property, `Volume`.
    pub property: String,
    /// Whether the feature writes a value of its own.
    pub has_value: bool,
    /// The document declaring the feature, as the parse named it.
    pub document: String,
}

/// What the Model's Tool Links say, as the backend read them.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ToolLinks {
    pub links: Vec<HeldLink>,
    pub properties: Vec<LinkedProperty>,
}

impl ToolLinks {
    pub fn is_empty(&self) -> bool {
        self.links.is_empty() && self.properties.is_empty()
    }
}

/// One Design Value to write into the Model: `feature`, declared in
/// `document`, takes `value` (`1000000.0 [mm^3]`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Edit {
    pub document: String,
    pub feature: QualifiedName,
    pub value: String,
}

/// What the tools answered: the edits to make, and what the Model records.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    pub edits: Vec<Edit>,
    pub tool_links: Vec<ToolLink>,
    pub outcomes: Vec<DesignValueOutcome>,
}

/// Asks `tools` for every linked property, one call per tool and file, and
/// plans the edits. Model Values come from `elements`. Features without a
/// value of their own aren't linked yet.
pub(crate) fn plan(
    read: &ToolLinks,
    tools: &mut BTreeMap<String, Box<dyn EngineeringTool>>,
    elements: &BTreeMap<QualifiedName, Element>,
) -> Plan {
    let linked: Vec<(&LinkedProperty, &HeldLink, PathBuf)> = read
        .properties
        .iter()
        .filter(|p| p.has_value)
        .filter_map(|p| {
            let held = read.links.iter().find(|h| h.link.element == p.owner)?;
            Some((p, held, resolve(&held.document, &held.link.file)))
        })
        .collect();

    let mut wanted: BTreeMap<(&str, &Path), Wanted> = BTreeMap::new();
    for (property, held, file) in &linked {
        wanted
            .entry((held.link.tool.as_str(), file.as_path()))
            .or_default()
            .entry(held.link.object.clone())
            .or_default()
            .insert(property.property.clone());
    }
    let answers: BTreeMap<_, _> = wanted
        .iter()
        .map(|(&(tool, file), wanted)| {
            let answer = match tools.get_mut(tool) {
                Some(adapter) => adapter.read(file, wanted),
                None => Err(format!("no Engineering Tool named {tool}")),
            };
            ((tool, file), answer)
        })
        .collect();

    let mut plan = Plan {
        tool_links: read.links.iter().map(|h| h.link.clone()).collect(),
        ..Plan::default()
    };
    for (property, held, file) in &linked {
        let link = &held.link;
        let answer = &answers[&(link.tool.as_str(), file.as_path())];
        let value = match answer {
            Err(reason) => Err(reason.clone()),
            Ok(objects) => objects
                .get(&link.object)
                .and_then(|properties| properties.get(&property.property))
                .cloned()
                .unwrap_or_else(|| Err(format!("{} gave no answer for it", link.tool))),
        };
        let design_value = match value {
            Ok(value) => {
                plan.edits.push(Edit {
                    document: property.document.clone(),
                    feature: property.feature.clone(),
                    value: value.clone(),
                });
                DesignValue::Read { value }
            }
            Err(reason) => DesignValue::Missing { reason },
        };
        plan.outcomes.push(DesignValueOutcome {
            feature: property.feature.clone(),
            source: DesignValueSource {
                tool: link.tool.clone(),
                file: link.file.clone(),
                object: link.object.clone(),
                property: property.property.clone(),
            },
            design_value,
            model_value: elements
                .get(&property.feature)
                .and_then(|e| e.value.clone()),
        });
    }
    plan
}

/// `file` as a Tool Link writes it, relative to the folder of `document`,
/// made absolute.
fn resolve(document: &str, file: &str) -> PathBuf {
    let joined = Path::new(document)
        .parent()
        .unwrap_or(Path::new(""))
        .join(file);
    std::path::absolute(&joined).unwrap_or(joined)
}
