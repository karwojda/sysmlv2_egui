//! Reads the standard API's JSON element form of a parsed Model for what the
//! symbol tree lacks. The service marks this form experimental, so every
//! read is defensive: a missing property leaves the element as it was.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use opensysml::{Connection, ConvertOptions, ConvertSource};
use serde_json::Value;
use viewer_core::model::{ConnectionEnd, Element, QualifiedName, SourceLocation};

/// Adds source locations, documentation, conjugation and connection ends
/// to `elements`, converting one document at a time so each element's file
/// is known.
pub fn enrich(
    connection: &Connection,
    parsed: &opensysml::Model,
    elements: &mut BTreeMap<QualifiedName, Element>,
) -> Result<(), opensysml::Error> {
    // Element JSON by id, and the file each came from, across all documents,
    // since references cross documents.
    let mut items: HashMap<String, (Value, PathBuf)> = HashMap::new();
    for document in parsed.documents() {
        let options = ConvertOptions { documents: vec![document.clone()], ..ConvertOptions::default() };
        let conversion = connection.convert("api-json", &ConvertSource::Model(parsed.hash().to_owned()), &options)?;
        let Ok(Value::Array(array)) = serde_json::from_str::<Value>(&conversion.content) else {
            continue;
        };
        for item in array {
            if let Some(id) = item.get("@id").and_then(Value::as_str) {
                items.insert(id.to_owned(), (item.clone(), PathBuf::from(document)));
            }
        }
    }
    let qualified_name = |id: &str| -> Option<QualifiedName> {
        items
            .get(id)
            .and_then(|(item, _)| item.get("qualifiedName"))
            .and_then(Value::as_str)
            .map(QualifiedName::new)
    };

    for (item, file) in items.values() {
        if item.get("@type").and_then(Value::as_str) == Some("Documentation") {
            let owner = reference(item, "owner").and_then(&qualified_name);
            let body = item.get("body").and_then(Value::as_str).map(str::trim);
            if let (Some(element), Some(body)) = (owner.and_then(|o| elements.get_mut(&o)), body) {
                element.documentation = Some(body.to_owned());
            }
            continue;
        }
        let Some(element) = item
            .get("qualifiedName")
            .and_then(Value::as_str)
            .and_then(|name| elements.get_mut(&QualifiedName::new(name)))
        else {
            continue;
        };
        if let Some(line) = item.get("sysx:sourceLine").and_then(Value::as_u64) {
            element.source = Some(SourceLocation { file: file.clone(), line: line as u32 });
        }
        if item.get("sysx:conjugatedTyping").and_then(Value::as_bool) == Some(true) {
            element.conjugated = true;
        }
        if let Some(ends) = item.get("connectorEnd").and_then(Value::as_array) {
            element.ends = ends
                .iter()
                .filter_map(|end| end.get("@id").and_then(Value::as_str))
                .filter_map(|end| end_path(&items, end))
                .filter_map(|path| path.iter().map(|id| qualified_name(id)).collect::<Option<Vec<_>>>())
                .map(|path| ConnectionEnd { path })
                .collect();
        }
    }
    Ok(())
}

/// The ids of the features a connector end reaches: the chain of a
/// `tanks.fuelOut` end, or the single feature of a plain one.
fn end_path<'a>(items: &'a HashMap<String, (Value, PathBuf)>, end: &str) -> Option<Vec<&'a str>> {
    let (end, _) = items.get(end)?;
    let referenced = end
        .get("ownedRelationship")?
        .as_array()?
        .iter()
        .filter_map(|r| r.get("@id").and_then(Value::as_str))
        .filter_map(|id| items.get(id).map(|(item, _)| item))
        .find(|r| r.get("@type").and_then(Value::as_str) == Some("ReferenceSubsetting"))
        .and_then(|r| reference(r, "referencedFeature"))?;
    match items.get(referenced).and_then(|(item, _)| item.get("chainingFeature")).and_then(Value::as_array) {
        Some(chain) => chain.iter().map(|f| f.get("@id").and_then(Value::as_str)).collect(),
        None => Some(vec![items.get_key_value(referenced)?.0.as_str()]),
    }
}

/// The id a single-valued reference property points at.
fn reference<'a>(item: &'a Value, property: &str) -> Option<&'a str> {
    item.get(property)?.get("@id")?.as_str()
}
