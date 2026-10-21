//! Design Values (ADR-0003 to ADR-0005): from the Tool Links a Model
//! writes, asks each Engineering Tool once per file for what is linked to
//! it, and says which feature takes which value. The edit that puts those
//! values into the Model in memory is the service's, in `lib.rs`.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use viewer_core::engineering_tools::{
    DesignValue, DesignValueOutcome, DesignValueSource, EngineeringTool, ToolLink, Wanted,
};
use viewer_core::model::{Diagnostic, Element, QualifiedName, Severity, SourceLocation};

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
    /// What the feature redefines, nearest first (`Drive::Gearbox::volume`,
    /// `Drive::Component::volume`): where its Model Value comes from when
    /// it writes none.
    pub redefines: Vec<QualifiedName>,
    /// The document declaring the feature, as the parse named it.
    pub document: String,
    /// Where the feature is declared: an unnamed redefinition is no
    /// element of ours, so has no location of its own.
    pub source: Option<SourceLocation>,
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
    pub how: How,
}

/// How a Design Value gets into the Model.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum How {
    /// The feature writes a value of its own, which `value` replaces.
    SetValue,
    /// The feature has a body and no value, which the service won't give
    /// one: it is replaced by `attribute :>> <redefines> = <value>;` in
    /// `owner`.
    Redefine {
        owner: QualifiedName,
        redefines: String,
    },
}

/// What the tools answered: the edits to make, and what the Model records.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    pub edits: Vec<Edit>,
    pub tool_links: Vec<ToolLink>,
    pub outcomes: Vec<DesignValueOutcome>,
    /// What is wrong with the Tool Links as written.
    pub diagnostics: Vec<Diagnostic>,
    /// Each marker written in place of a missing Design Value
    /// (`no Design Value #0#`), and what it means in words.
    markers: Vec<(String, String)>,
    /// Each Design Value written: how the service names its feature from
    /// a part (`pump.mass`), and how a reason about it begins in words
    /// (`Design Value for Drive::drive::pump::mass (FreeCAD: Pump.Volume)`).
    written: Vec<(String, String)>,
}

/// How a marker begins; its number and a closing `#` follow.
const MARKER: &str = "no Design Value #";

/// How the service says a value doesn't fit its feature's type:
/// `feature value pump.mass: type mismatch: cannot write 1500000.0
/// [mm**3] (dimension L^3) to a feature typed by MassValue (dimension M)`.
const FEATURE_VALUE: &str = "feature value ";
const TYPE_MISMATCH: &str = ": type mismatch: ";

impl Plan {
    /// `reason`, a check's, in words when it is about a Design Value:
    /// `no Design Value for Drive::drive::gearbox::volume (FreeCAD: no
    /// object Assembly.Link.)` when there is none, `Design Value for
    /// Drive::drive::pump::mass (FreeCAD: Pump.Volume) does not fit it:
    /// cannot write …` when its dimension is wrong. `None` otherwise.
    pub fn translate(&self, reason: &str) -> Option<String> {
        let found: Vec<&str> = self
            .markers
            .iter()
            .filter(|(marker, _)| reason.contains(marker.as_str()))
            .map(|(_, words)| words.as_str())
            .collect();
        if !found.is_empty() {
            return Some(found.join("; "));
        }
        let (_, rest) = reason.split_once(FEATURE_VALUE)?;
        let (path, mismatch) = rest.split_once(TYPE_MISMATCH)?;
        let (_, words) = self.written.iter().find(|(feature, _)| {
            path == feature
                || path
                    .strip_suffix(feature.as_str())
                    .is_some_and(|p| p.ends_with('.'))
        })?;
        Some(format!("{words} does not fit it: {mismatch}"))
    }
}

/// Asks `tools` for every linked property, one call per tool and file, and
/// plans the edits. Model Values come from `elements`.
pub(crate) fn plan(
    read: &ToolLinks,
    tools: &mut BTreeMap<String, Box<dyn EngineeringTool>>,
    elements: &BTreeMap<QualifiedName, Element>,
) -> Plan {
    let mut plan = Plan {
        tool_links: read.links.iter().map(|h| h.link.clone()).collect(),
        ..Plan::default()
    };
    let at = |element: &QualifiedName| elements.get(element).and_then(|e| e.source.clone());

    // An element with several links to one tool has no one object to
    // read; its features take no Design Value.
    let mut per_tool: BTreeMap<(&QualifiedName, &str), usize> = BTreeMap::new();
    for held in &read.links {
        *per_tool
            .entry((&held.link.element, held.link.tool.as_str()))
            .or_default() += 1;
    }
    let ambiguous = |link: &ToolLink| {
        let count = per_tool[&(&link.element, link.tool.as_str())];
        (count > 1).then(|| format!("{} has {count} Tool Links to {}", link.element, link.tool))
    };
    for (&(element, tool), &count) in &per_tool {
        if count > 1 {
            plan.diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!(
                    "{element} has {count} Tool Links to {tool}: its features take no Design Value"
                ),
                location: at(element),
            });
        }
    }

    let mut linked: Vec<(&LinkedProperty, &HeldLink, PathBuf)> = Vec::new();
    for property in &read.properties {
        match read.links.iter().find(|h| h.link.element == property.owner) {
            Some(held) => linked.push((property, held, resolve(&held.document, &held.link.file))),
            None => plan.diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!(
                    "@ToolProperty on {}, but {} has no Tool Link: it takes no Design Value",
                    property.feature, property.owner
                ),
                location: property.source.clone(),
            }),
        }
    }

    let mut wanted: BTreeMap<(&str, &Path), Wanted> = BTreeMap::new();
    for (property, held, file) in linked
        .iter()
        .filter(|(_, h, _)| ambiguous(&h.link).is_none())
    {
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

    for (property, held, file) in &linked {
        let link = &held.link;
        let answer = ambiguous(link)
            .map(Err)
            .unwrap_or_else(|| answers[&(link.tool.as_str(), file.as_path())].clone());
        let value = match answer {
            Err(reason) => Err(reason),
            Ok(objects) => objects
                .get(&link.object)
                .and_then(|properties| properties.get(&property.property))
                .cloned()
                .unwrap_or_else(|| {
                    Err(format!(
                        "gave no answer for {} of {}",
                        property.property, link.object
                    ))
                }),
        };
        // Without a Design Value the feature takes a marker in its place:
        // removing its value would let a def's `default` show through, and
        // a string can't be a quantity, so exactly the checks that depend
        // on it are Undecided, with the marker in the service's reason.
        let (written, design_value) = match value {
            Ok(value) => {
                plan.written.push((
                    format!(
                        "{}.{}",
                        last_segment(&property.owner),
                        last_segment(&property.feature)
                    ),
                    format!(
                        "Design Value for {} ({}: {}.{})",
                        property.feature, link.tool, link.object, property.property
                    ),
                ));
                (value.clone(), DesignValue::Read { value })
            }
            Err(reason) => {
                let marker = format!("{MARKER}{}#", plan.markers.len());
                plan.markers.push((
                    marker.clone(),
                    format!(
                        "no Design Value for {} ({}: {reason})",
                        property.feature, link.tool
                    ),
                ));
                (format!("\"{marker}\""), DesignValue::Missing { reason })
            }
        };
        plan.edits.push(Edit {
            document: property.document.clone(),
            feature: property.feature.clone(),
            value: written,
            how: how(property),
        });
        plan.outcomes.push(DesignValueOutcome {
            feature: property.feature.clone(),
            source: DesignValueSource {
                tool: link.tool.clone(),
                file: link.file.clone(),
                object: link.object.clone(),
                property: property.property.clone(),
            },
            design_value,
            model_value: std::iter::once(&property.feature)
                .chain(&property.redefines)
                .find_map(|feature| elements.get(feature)?.value.clone()),
        });
    }
    plan
}

/// How `property`'s feature takes its Design Value.
fn how(property: &LinkedProperty) -> How {
    if property.has_value {
        return How::SetValue;
    }
    How::Redefine {
        owner: property.owner.clone(),
        redefines: last_segment(&property.feature).to_owned(),
    }
}

/// `volume` of `Drive::drive::engine::volume`.
fn last_segment(name: &QualifiedName) -> &str {
    let name = name.as_str();
    name.rsplit("::").next().unwrap_or(name)
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
