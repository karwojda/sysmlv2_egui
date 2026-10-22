//! Design Values (ADR-0003 to ADR-0005): from the Tool Links a Model
//! writes, asks each Engineering Tool once per file for what is linked to
//! it, and says which feature takes which value. The edit that puts those
//! values into the Model in memory is the service's, in `lib.rs`; putting
//! the service's reasons about them into words is `checks/service.rs`'s.

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
    /// The link, `@ToolLink { tool = "FreeCAD"; … }` on `Drive::drive::engine`.
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
    /// Where the feature's declaration begins, line and column from 1,
    /// when it has a name of its own (`attribute bore : LengthValue`).
    pub named_at: Option<(u32, u32)>,
    /// The document declaring the feature, as the parse named it.
    pub document: String,
    /// Where the feature is declared: an unnamed redefinition is no
    /// element of ours, so has no location of its own.
    pub source: Option<SourceLocation>,
}

/// What the Model's Tool Links say, as the backend read them.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ToolLinks {
    /// Every `@ToolLink`, in the order read.
    pub links: Vec<HeldLink>,
    /// Every feature with a `@ToolProperty`, in the order read.
    pub properties: Vec<LinkedProperty>,
}

impl ToolLinks {
    /// Whether the Model writes no `@ToolLink` and no `@ToolProperty`, so
    /// a load has no Design Values to read.
    pub fn is_empty(&self) -> bool {
        self.links.is_empty() && self.properties.is_empty()
    }
}

/// One Design Value to write into the Model: `feature`, declared in
/// `document`, takes `value` (`1000000.0 [mm^3]`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ValueEdit {
    /// The document declaring the feature, as the parse named it.
    pub document: String,
    /// The linked feature, `Drive::drive::engine::volume`.
    pub feature: QualifiedName,
    /// What it takes, as SysML writes it: `1000000.0 [mm^3]`, or a
    /// marker string when there is no Design Value.
    pub value: String,
    /// Where the value is written.
    pub place: ValuePlace,
}

/// Where a Design Value is written: through the service's editor, or into
/// the text before the service parses it.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ValuePlace {
    /// Through the service's editor.
    Editor(EditorEdit),
    /// The feature has a name of its own and no value (`attribute bore :
    /// LengthValue { @ToolProperty … }`): the service won't give one to a
    /// declaration with a body, nor delete one anything refers to, so
    /// ` = <value>` is written into the text before its body or its `;`,
    /// the declaration beginning at `at` (line, column). See
    /// [`write_value`].
    InText { at: (u32, u32) },
}

/// What the service's editor is asked to do with a Design Value.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum EditorEdit {
    /// The feature writes a value of its own, which the Design Value
    /// replaces.
    SetValue,
    /// The feature has a body and no value, which the service won't give
    /// one: it is replaced by `attribute :>> <redefines> = <value>;` in
    /// `owner`.
    Redefine {
        /// The element holding the feature, `Drive::drive::engine`.
        owner: QualifiedName,
        /// The name of the feature it redefines, `volume`.
        redefines: String,
    },
}

/// What the tools answered: the edits to make, and what the Model records.
#[derive(Debug, Default)]
pub(crate) struct Plan {
    /// One edit per linked feature: its Design Value, or a marker.
    pub edits: Vec<ValueEdit>,
    /// Every Tool Link, for the Model to record.
    pub tool_links: Vec<ToolLink>,
    /// Every linked feature's Design Value outcome, for the Model to
    /// record.
    pub outcomes: Vec<DesignValueOutcome>,
    /// What is wrong with the Tool Links as written.
    pub diagnostics: Vec<Diagnostic>,
    /// What the values written mean, for the checks' reasons.
    pub written: Written,
}

/// What the values a plan writes mean in words, so the service's adapter
/// can put a check's reason about one into words.
#[derive(Debug, Default)]
pub(crate) struct Written {
    /// Each marker written in place of a missing Design Value
    /// (`no Design Value #0#`), and what it means: `no Design Value for
    /// Drive::drive::gearbox::volume (FreeCAD: no object Assembly.Link.)`.
    pub markers: Vec<(String, String)>,
    /// Each Design Value written.
    pub values: Vec<WrittenValue>,
}

/// A Design Value written into the Model.
#[derive(Debug)]
pub(crate) struct WrittenValue {
    /// The feature, `Drive::drive::pump::mass`.
    pub feature: QualifiedName,
    /// The value written, `1500000.0 [mm^3]`.
    pub value: String,
    /// What it is, in words: `Design Value for Drive::drive::pump::mass
    /// (FreeCAD: Pump.Volume)`.
    pub words: String,
}

/// How a marker begins; its number and a closing `#` follow.
const MARKER: &str = "no Design Value #";

/// Asks `tools` for every linked property, one call per tool and file, and
/// plans the edits. Model Values come from `elements`.
pub(crate) fn plan(
    found: &ToolLinks,
    tools: &mut BTreeMap<String, Box<dyn EngineeringTool>>,
    elements: &BTreeMap<QualifiedName, Element>,
) -> Plan {
    let mut plan = Plan {
        tool_links: found.links.iter().map(|h| h.link.clone()).collect(),
        ..Plan::default()
    };
    let at = |element: &QualifiedName| elements.get(element).and_then(|e| e.source.clone());

    // An element with several links to one tool has no one object to
    // read; its features take no Design Value.
    let mut per_tool: BTreeMap<(&QualifiedName, &str), usize> = BTreeMap::new();
    for held in &found.links {
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
    for property in &found.properties {
        match found
            .links
            .iter()
            .find(|h| h.link.element == property.owner)
        {
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
                plan.written.values.push(WrittenValue {
                    feature: property.feature.clone(),
                    value: value.clone(),
                    words: format!(
                        "Design Value for {} ({}: {}.{})",
                        property.feature, link.tool, link.object, property.property
                    ),
                });
                (value.clone(), DesignValue::Read { value })
            }
            Err(reason) => {
                let marker = format!("{MARKER}{}#", plan.written.markers.len());
                plan.written.markers.push((
                    marker.clone(),
                    format!(
                        "no Design Value for {} ({}: {reason})",
                        property.feature, link.tool
                    ),
                ));
                (format!("\"{marker}\""), DesignValue::Missing { reason })
            }
        };
        plan.edits.push(ValueEdit {
            document: property.document.clone(),
            feature: property.feature.clone(),
            value: written,
            place: place(property),
        });
        plan.outcomes.push(DesignValueOutcome {
            feature: property.feature.clone(),
            source: DesignValueSource {
                link: link.clone(),
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

/// Where `property`'s feature takes its Design Value.
fn place(property: &LinkedProperty) -> ValuePlace {
    if property.has_value {
        return ValuePlace::Editor(EditorEdit::SetValue);
    }
    if let Some(at) = property.named_at {
        return ValuePlace::InText { at };
    }
    ValuePlace::Editor(EditorEdit::Redefine {
        owner: property.owner.clone(),
        redefines: last_segment(&property.feature).to_owned(),
    })
}

/// `text` with ` = <value>` written into the declaration beginning at
/// `at` (line from 1, column from 1 in UTF-8 bytes, as the service counts
/// it), before its body or its terminating `;`.
/// `None` when neither follows: `at` is no declaration of this text.
pub(crate) fn write_value(text: &str, at: (u32, u32), value: &str) -> Option<String> {
    let (line, column) = at;
    let line_start = match line {
        0 => return None,
        1 => 0,
        _ => text.match_indices('\n').nth(line as usize - 2)?.0 + 1,
    };
    // The service counts the column in UTF-8 bytes, not characters.
    let start = line_start + column.checked_sub(1)? as usize;
    if !text.is_char_boundary(start) || text[line_start..start].contains('\n') {
        return None;
    }
    // The header runs to the body's `{` or the `;`; neither counts inside
    // a string, a quoted name or a comment.
    let header = &text[start..];
    let mut chars = header.char_indices();
    let end = loop {
        let (i, c) = chars.next()?;
        match c {
            '{' | ';' => break start + i,
            '"' | '\'' => {
                chars.by_ref().find(|&(_, n)| n == c)?;
            }
            '/' if header[i..].starts_with("//") => {
                chars.by_ref().find(|&(_, n)| n == '\n')?;
            }
            '/' if header[i..].starts_with("/*") => {
                chars.next();
                chars
                    .by_ref()
                    .find(|&(j, _)| header[j..].starts_with("*/"))?;
                chars.next();
            }
            _ => {}
        }
    };
    // Not past a line break, which may end a `//` comment.
    let head = text[..end].trim_end_matches([' ', '\t']);
    Some(format!("{head} = {value} {}", &text[end..]))
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

#[cfg(test)]
mod tests {
    use super::write_value;

    #[test]
    fn a_value_is_written_before_the_body() {
        let text =
            "part p {\n    attribute bore : LengthValue { @ToolProperty { name = \"D\"; } }\n}\n";
        assert_eq!(
            write_value(text, (2, 5), "8.0 [mm]").as_deref(),
            Some(
                "part p {\n    attribute bore : LengthValue = 8.0 [mm] { @ToolProperty { name = \"D\"; } }\n}\n"
            )
        );
    }

    #[test]
    fn a_value_is_written_before_the_semicolon_of_a_declaration_without_a_body() {
        assert_eq!(
            write_value(
                "attribute bore : LengthValue;",
                (1, 1),
                "\"no Design Value #0#\""
            )
            .as_deref(),
            Some("attribute bore : LengthValue = \"no Design Value #0#\" ;")
        );
    }

    #[test]
    fn braces_in_quoted_names_and_comments_are_not_the_body() {
        let text = "attribute 'b{re' /* { */ : LengthValue // {\n    { }";
        assert_eq!(
            write_value(text, (1, 1), "1 [mm]").as_deref(),
            Some("attribute 'b{re' /* { */ : LengthValue // {\n = 1 [mm] { }")
        );
    }

    #[test]
    fn a_place_that_starts_no_declaration_writes_nothing() {
        assert_eq!(
            write_value("attribute bore : LengthValue", (1, 1), "1"),
            None
        );
        assert_eq!(write_value("attribute bore;", (3, 1), "1"), None);
        assert_eq!(write_value("attribute bore;", (1, 40), "1"), None);
    }
}
