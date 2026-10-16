//! Reads the standard API's JSON element form of a parsed Model for what the
//! symbol tree lacks. The service marks this form experimental, so every
//! read is defensive: a missing property leaves the element as it was.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use opensysml::{Connection, ConvertOptions, ConvertSource};
use serde_json::Value;
use viewer_core::model::{ConnectionEnd, Element, ElementKind, Expose, ExposeScope, QualifiedName, SourceLocation};

/// Adds source locations, documentation, conjugation and connection ends
/// to `elements`, converting one document at a time so each element's file
/// is known. A document that cannot be converted is skipped, and its
/// failure returned, so one broken file does not cost the others.
pub fn enrich(
    connection: &Connection,
    parsed: &opensysml::Model,
    elements: &mut BTreeMap<QualifiedName, Element>,
) -> Vec<(String, opensysml::Error)> {
    // Element JSON by id, and the file each came from, across all documents,
    // since references cross documents.
    let mut items: HashMap<String, (Value, PathBuf)> = HashMap::new();
    let mut failures = Vec::new();
    for document in parsed.documents() {
        let options = ConvertOptions { documents: vec![document.clone()], ..ConvertOptions::default() };
        // The service refuses to convert any document of a Model with a
        // syntax error anywhere; the file on its own may still convert,
        // losing only ends that reach into other files.
        let conversion = match connection
            .convert("api-json", &ConvertSource::Model(parsed.hash().to_owned()), &options)
            .or_else(|_| {
                let alone = ConvertOptions { tolerate_syntax_errors: true, ..ConvertOptions::default() };
                connection.convert("api-json", &ConvertSource::File(PathBuf::from(document)), &alone)
            })
        {
            Ok(conversion) => conversion,
            Err(error) => {
                failures.push((document.clone(), error));
                continue;
            }
        };
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
    let mut children: HashMap<&str, Vec<&Value>> = HashMap::new();
    for (item, _) in items.values() {
        if let Some(owner) = reference(item, "owner") {
            children.entry(owner).or_default().push(item);
        }
    }

    // Unnamed connections, interfaces, flows and bindings aren't in the
    // symbol tree; add them under the service's generated names.
    for (item, _) in items.values() {
        let kind = match item.get("@type").and_then(Value::as_str) {
            Some("ConnectionUsage") => ElementKind::ConnectionUsage,
            Some("InterfaceUsage") => ElementKind::InterfaceUsage,
            Some("FlowUsage") => ElementKind::FlowUsage,
            Some("BindingConnectorAsUsage") => ElementKind::BindingConnection,
            _ => continue,
        };
        let Some(name) = item.get("qualifiedName").and_then(Value::as_str).map(QualifiedName::new) else { continue };
        if elements.contains_key(&name) {
            continue;
        }
        let mut element = Element::new(name.clone(), kind);
        element.name = String::new();
        element.typed_by = first_reference(item, "type").and_then(&qualified_name);
        elements.insert(name, element);
    }

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
        if item.get("isReference").and_then(Value::as_bool) == Some(true) {
            element.is_reference = true;
        }
        if let Some(ends) = item.get("connectorEnd").and_then(Value::as_array) {
            element.ends = ends
                .iter()
                .filter_map(|end| end.get("@id").and_then(Value::as_str))
                .filter_map(|end| end_path(&items, &children, end))
                .filter_map(|path| path.iter().map(|id| qualified_name(id)).collect::<Option<Vec<_>>>())
                .map(|path| ConnectionEnd { path })
                .collect();
        }
        {
            let id = item.get("@id").and_then(Value::as_str).unwrap_or_default();
            let mut annotations: Vec<&Value> = children
                .get(id)
                .into_iter()
                .flatten()
                .copied()
                .filter(|c| c.get("@type").and_then(Value::as_str) == Some("MetadataUsage"))
                .collect();
            annotations.sort_by_key(|c| member_index(c));
            element.metadata =
                annotations.iter().filter_map(|c| first_reference(c, "type")).filter_map(&qualified_name).collect();
        }
        if matches!(element.kind, ElementKind::ViewUsage | ElementKind::ViewDefinition) {
            let id = item.get("@id").and_then(Value::as_str).unwrap_or_default();
            let mut owned: Vec<&Value> = children.get(id).cloned().unwrap_or_default();
            owned.sort_by_key(|c| member_index(c));
            for child in owned {
                match child.get("@type").and_then(Value::as_str) {
                    Some("MembershipExpose" | "NamespaceExpose") => {
                        element.exposes.extend(expose(&items, child).and_then(|(target, scope)| {
                            Some(Expose { target: qualified_name(target)?, scope })
                        }));
                    }
                    Some("RenderingUsage") => {
                        element.rendering = reference(child, "references").and_then(&qualified_name);
                    }
                    _ => {}
                }
            }
        }
        // A flow carries its payload (`flow of Fuel`), else its own type.
        if element.kind == ElementKind::FlowUsage {
            let id = item.get("@id").and_then(Value::as_str).unwrap_or_default();
            let payload = children
                .get(id)
                .into_iter()
                .flatten()
                .find(|c| c.get("@type").and_then(Value::as_str) == Some("PayloadFeature"))
                .and_then(|c| first_reference(c, "type"))
                .and_then(&qualified_name);
            element.payload = payload.or_else(|| element.typed_by.clone());
        }
    }
    failures
}

/// The id of what an `expose` shows, and how much of it. A membership
/// expose names a membership (`expose X`, or recursively `X::**`, which
/// includes X); a namespace expose names the namespace (`X::*`, or
/// recursively `X::*::**`).
fn expose<'a>(items: &'a HashMap<String, (Value, PathBuf)>, item: &'a Value) -> Option<(&'a str, ExposeScope)> {
    let recursive = item.get("isRecursive").and_then(Value::as_bool) == Some(true);
    if item.get("@type").and_then(Value::as_str) == Some("NamespaceExpose") {
        let scope = if recursive { ExposeScope::Descendants } else { ExposeScope::Members };
        return Some((reference(item, "importedNamespace")?, scope));
    }
    let membership = reference(item, "importedMembership")?;
    let target = items
        .get(membership)
        .and_then(|(m, _)| reference(m, "memberElement"))
        .or_else(|| membership.strip_suffix("_om").filter(|id| items.contains_key(*id)))?;
    let scope = if recursive { ExposeScope::ElementAndDescendants } else { ExposeScope::Element };
    Some((target, scope))
}

/// The position of an owned element among its owner's unnamed members,
/// from its generated name (`V::view::@2` is 2), to keep declaration order.
fn member_index(item: &Value) -> u32 {
    item.get("qualifiedName")
        .and_then(Value::as_str)
        .and_then(|name| name.rsplit_once("::@"))
        .and_then(|(_, index)| index.parse().ok())
        .unwrap_or(u32::MAX)
}

/// The id the first entry of a many-valued reference property points at.
fn first_reference<'a>(item: &'a Value, property: &str) -> Option<&'a str> {
    item.get(property)?.as_array()?.first()?.get("@id")?.as_str()
}

/// The ids of the features a connector end reaches: the chain of a
/// `tanks.fuelOut` end, or the single feature of a plain one. A flow end
/// references the part (or chain) and, in a nested reference, redefines
/// the port or item it reaches, which completes the path.
fn end_path<'a>(
    items: &'a HashMap<String, (Value, PathBuf)>,
    children: &HashMap<&str, Vec<&'a Value>>,
    end_id: &str,
) -> Option<Vec<&'a str>> {
    let (end, _) = items.get(end_id)?;
    let mut path = referenced_path(items, end)?;
    let redefined = children
        .get(end_id)
        .into_iter()
        .flatten()
        .filter(|c| c.get("@type").and_then(Value::as_str) == Some("ReferenceUsage"))
        .find_map(|c| reference(c, "redefines"))
        .and_then(|id| items.get_key_value(id))
        .map(|(id, _)| id.as_str());
    path.extend(redefined);
    Some(path)
}

/// The chain an end's reference subsetting points at.
fn referenced_path<'a>(items: &'a HashMap<String, (Value, PathBuf)>, end: &Value) -> Option<Vec<&'a str>> {
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
