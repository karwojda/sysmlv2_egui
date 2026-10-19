//! Reads the standard API's JSON element form of a parsed Model for what the
//! symbol tree lacks. The service marks this form experimental, so every
//! read is defensive: a missing property leaves the element as it was.

use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;

use opensysml::{Connection, ConvertOptions, ConvertSource};
use serde_json::Value;
use viewer_core::model::{
    ConnectionEnd, Element, ElementKind, Expose, ExposeScope, Objective, QualifiedName,
    SourceLocation,
};

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
        let options = ConvertOptions {
            documents: vec![document.clone()],
            ..ConvertOptions::default()
        };
        // The service refuses to convert any document of a Model with a
        // syntax error anywhere; the file on its own may still convert,
        // losing only ends that reach into other files.
        let conversion = match connection
            .convert(
                "api-json",
                &ConvertSource::Model(parsed.hash().to_owned()),
                &options,
            )
            .or_else(|_| {
                let alone = ConvertOptions {
                    tolerate_syntax_errors: true,
                    ..ConvertOptions::default()
                };
                connection.convert(
                    "api-json",
                    &ConvertSource::File(PathBuf::from(document)),
                    &alone,
                )
            }) {
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

    // Unnamed connections, interfaces, flows and bindings, and Satisfaction
    // Claims, aren't in the symbol tree; add them under the service's
    // generated names.
    for (item, _) in items.values() {
        let kind = match item.get("@type").and_then(Value::as_str) {
            Some("ConnectionUsage") => ElementKind::ConnectionUsage,
            Some("InterfaceUsage") => ElementKind::InterfaceUsage,
            Some("FlowUsage") => ElementKind::FlowUsage,
            Some("BindingConnectorAsUsage") => ElementKind::BindingConnection,
            Some("SatisfyRequirementUsage") => ElementKind::SatisfyRequirement,
            _ => continue,
        };
        let Some(name) = item
            .get("qualifiedName")
            .and_then(Value::as_str)
            .map(QualifiedName::new)
        else {
            continue;
        };
        if elements.contains_key(&name) {
            continue;
        }
        let mut element = Element::new(name.clone(), kind);
        element.name = String::new();
        if element.kind == ElementKind::SatisfyRequirement {
            // `satisfy r by x` names its satisfier; `satisfy r;` inside a
            // part is satisfied by that part. Either may be a feature chain.
            element.satisfies = reference(item, "subsets")
                .and_then(|id| chain_end(&items, id))
                .and_then(qualified_name);
            let id = item.get("@id").and_then(Value::as_str).unwrap_or_default();
            element.satisfier = reference(item, "subject")
                .and_then(|id| chain_end(&items, id))
                .or_else(|| subject_value(&items, &children, id))
                .or_else(|| reference(item, "owner"))
                .and_then(qualified_name);
        } else {
            element.typed_by = first_reference(item, "type").and_then(qualified_name);
        }
        elements.insert(name, element);
    }

    // `return` parameters, marked once the loop no longer holds their owner.
    let mut returns: Vec<QualifiedName> = Vec::new();
    for (item, file) in items.values() {
        if item.get("@type").and_then(Value::as_str) == Some("Documentation") {
            let owner = reference(item, "owner").and_then(qualified_name);
            let body = item.get("body").and_then(Value::as_str).map(str::trim);
            if let (Some(element), Some(body)) = (owner.and_then(|o| elements.get_mut(&o)), body) {
                element.documentation = Some(body.to_owned());
            }
            continue;
        }
        if item.get("@type").and_then(Value::as_str) == Some("FeatureValue") {
            let feature = reference(item, "featureWithValue");
            let target = feature.and_then(|id| valued_element(&items, &children, id));
            let text = reference(item, "memberElement").map(|e| crate::expression::text(&items, e));
            if let (Some(element), Some(text)) = (target.and_then(|t| elements.get_mut(&t)), text) {
                let default = item.get("isDefault").and_then(Value::as_bool) == Some(true);
                element.value = Some(if default {
                    format!("default {text}")
                } else {
                    text
                });
            }
            continue;
        }
        if item.get("@type").and_then(Value::as_str) == Some("SubjectMembership") {
            let owner = reference(item, "owner").and_then(qualified_name);
            let member = reference(item, "memberElement");
            let Some(subject) = member.and_then(qualified_name) else {
                continue;
            };
            // A requirement's or analysis's subject; a claim's subject is its
            // satisfier.
            let requirement = owner.and_then(|o| elements.get_mut(&o)).filter(|e| {
                matches!(
                    e.kind,
                    ElementKind::RequirementDefinition
                        | ElementKind::RequirementUsage
                        | ElementKind::AnalysisDefinition
                        | ElementKind::AnalysisUsage
                )
            });
            let Some(element) = requirement else { continue };
            element.subject = Some(subject.clone());
            // The symbol tree leaves a subject untyped; the element form has it.
            let typed = member
                .and_then(|id| items.get(id))
                .and_then(|(subject_item, _)| first_reference(subject_item, "type"))
                .and_then(qualified_name);
            if let Some(feature) = elements.get_mut(&subject).filter(|f| f.typed_by.is_none()) {
                feature.typed_by = typed;
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
        if let Some(id) = item.get("declaredShortName").and_then(Value::as_str) {
            element.short_name = Some(id.to_owned());
        }
        if let Some(line) = item.get("sysx:sourceLine").and_then(Value::as_u64) {
            element.source = Some(SourceLocation {
                file: file.clone(),
                line: line as u32,
            });
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
                .filter_map(|path| {
                    path.iter()
                        .map(|id| qualified_name(id))
                        .collect::<Option<Vec<_>>>()
                })
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
            element.metadata = annotations
                .iter()
                .filter_map(|c| first_reference(c, "type"))
                .filter_map(&qualified_name)
                .collect();
        }
        if matches!(
            element.kind,
            ElementKind::ConstraintDefinition | ElementKind::ConstraintUsage
        ) {
            let id = item.get("@id").and_then(Value::as_str).unwrap_or_default();
            element.is_asserted =
                item.get("@type").and_then(Value::as_str) == Some("AssertConstraintUsage");
            element.expression = children
                .get(id)
                .into_iter()
                .flatten()
                .find(|c| {
                    c.get("@type").and_then(Value::as_str) == Some("ResultExpressionMembership")
                })
                .and_then(|membership| reference(membership, "memberElement"))
                .map(|expression| crate::expression::text(&items, expression));
        }
        if matches!(
            element.kind,
            ElementKind::RequirementDefinition | ElementKind::RequirementUsage
        ) {
            let id = item.get("@id").and_then(Value::as_str).unwrap_or_default();
            element.require = require_constraints(&items, &children, id);
        }
        if matches!(
            element.kind,
            ElementKind::AnalysisDefinition | ElementKind::AnalysisUsage
        ) {
            let id = item.get("@id").and_then(Value::as_str).unwrap_or_default();
            // `subject v = vehicle` binds the subject by value, as a chained
            // `satisfy … by` does.
            element.subject_target = subject_value(&items, &children, id).and_then(qualified_name);
            let owned = |kind: &str| {
                children
                    .get(id)
                    .into_iter()
                    .flatten()
                    .copied()
                    .filter(move |c| c.get("@type").and_then(Value::as_str) == Some(kind))
                    .filter_map(|m| reference(m, "memberElement"))
                    .collect::<Vec<_>>()
            };
            // The objective is an unnamed requirement with a doc and
            // `require` constraints of its own.
            element.objective = owned("ObjectiveMembership").first().map(|objective| {
                let doc = children
                    .get(*objective)
                    .into_iter()
                    .flatten()
                    .find(|c| c.get("@type").and_then(Value::as_str) == Some("Documentation"))
                    .and_then(|d| d.get("body").and_then(Value::as_str))
                    .map(|body| body.trim().to_owned());
                Objective {
                    doc,
                    require: require_constraints(&items, &children, objective),
                }
            });
            for parameter in owned("ReturnParameterMembership")
                .into_iter()
                .filter_map(qualified_name)
            {
                returns.push(parameter);
            }
        }
        if matches!(
            element.kind,
            ElementKind::ViewUsage | ElementKind::ViewDefinition
        ) {
            let id = item.get("@id").and_then(Value::as_str).unwrap_or_default();
            let mut owned: Vec<&Value> = children.get(id).cloned().unwrap_or_default();
            owned.sort_by_key(|c| member_index(c));
            for child in owned {
                match child.get("@type").and_then(Value::as_str) {
                    Some("MembershipExpose" | "NamespaceExpose") => {
                        element.exposes.extend(expose(&items, child).and_then(
                            |(target, scope)| {
                                Some(Expose {
                                    target: qualified_name(target)?,
                                    scope,
                                })
                            },
                        ));
                    }
                    Some("RenderingUsage") => {
                        element.rendering = reference(child, "references").and_then(qualified_name);
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
                .and_then(qualified_name);
            element.payload = payload.or_else(|| element.typed_by.clone());
        }
    }
    for parameter in returns {
        if let Some(parameter) = elements.get_mut(&parameter) {
            parameter.is_return = true;
        }
    }
    failures
}

/// The id of what an `expose` shows, and how much of it. A membership
/// expose names a membership (`expose X`, or recursively `X::**`, which
/// includes X); a namespace expose names the namespace (`X::*`, or
/// recursively `X::*::**`).
fn expose<'a>(
    items: &'a HashMap<String, (Value, PathBuf)>,
    item: &'a Value,
) -> Option<(&'a str, ExposeScope)> {
    let recursive = item.get("isRecursive").and_then(Value::as_bool) == Some(true);
    if item.get("@type").and_then(Value::as_str) == Some("NamespaceExpose") {
        let scope = if recursive {
            ExposeScope::Descendants
        } else {
            ExposeScope::Members
        };
        return Some((reference(item, "importedNamespace")?, scope));
    }
    let membership = reference(item, "importedMembership")?;
    let target = items
        .get(membership)
        .and_then(|(m, _)| reference(m, "memberElement"))
        .or_else(|| {
            membership
                .strip_suffix("_om")
                .filter(|id| items.contains_key(*id))
        })?;
    let scope = if recursive {
        ExposeScope::ElementAndDescendants
    } else {
        ExposeScope::Element
    };
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

/// The text of each `require` constraint a requirement owns, in order: the
/// expression its constraint usage results in, or `…` if it has none (a
/// constraint given by reference, say).
fn require_constraints(
    items: &HashMap<String, (Value, PathBuf)>,
    children: &HashMap<&str, Vec<&Value>>,
    requirement: &str,
) -> Vec<String> {
    let mut constraints: Vec<&Value> = children
        .get(requirement)
        .into_iter()
        .flatten()
        .filter(|c| {
            c.get("@type").and_then(Value::as_str) == Some("RequirementConstraintMembership")
        })
        .filter(|c| c.get("kind").and_then(Value::as_str) == Some("requirement"))
        .filter_map(|membership| reference(membership, "memberElement"))
        .filter_map(|id| items.get(id).map(|(item, _)| item))
        .collect();
    constraints.sort_by_key(|c| member_index(c));
    constraints
        .iter()
        .map(|constraint| {
            let id = constraint
                .get("@id")
                .and_then(Value::as_str)
                .unwrap_or_default();
            children
                .get(id)
                .into_iter()
                .flatten()
                .find(|c| {
                    c.get("@type").and_then(Value::as_str) == Some("ResultExpressionMembership")
                })
                .and_then(|membership| reference(membership, "memberElement"))
                .map(|expression| crate::expression::text(items, expression))
                .unwrap_or_else(|| crate::expression::UNKNOWN.to_owned())
        })
        .collect()
}

/// Our element for the feature `id` the element form gives a value to: the
/// one of the same qualified name, or for a redefinition without a name of
/// its own (`attribute :>> mass = 180.0`, which the element form calls
/// `engine::@0`), the owner's feature named after the one it redefines.
fn valued_element(
    items: &HashMap<String, (Value, PathBuf)>,
    children: &HashMap<&str, Vec<&Value>>,
    id: &str,
) -> Option<QualifiedName> {
    let (feature, _) = items.get(id)?;
    let qualified = feature.get("qualifiedName").and_then(Value::as_str)?;
    // Names generated for unnamed elements (`@2`) don't match the symbol
    // tree's; anything without one is already ours.
    if !qualified
        .split("::")
        .any(|segment| segment.starts_with('@'))
    {
        return Some(QualifiedName::new(qualified));
    }
    let name = match feature.get("declaredName").and_then(Value::as_str) {
        Some(name) => name,
        None => children
            .get(id)?
            .iter()
            .find(|c| c.get("@type").and_then(Value::as_str) == Some("Redefinition"))
            .and_then(|r| reference(r, "redefinedFeature"))
            .and_then(|r| items.get(r))
            .and_then(|(r, _)| r.get("declaredName")?.as_str())?,
    };
    // The owner may be an unnamed redefinition too (`part :>> tank { … }`).
    let owner = valued_element(items, children, reference(feature, "owner")?)?;
    Some(QualifiedName::new(format!("{owner}::{name}")))
}

/// The feature a claim's subject is bound to by value: `by vehicle.powertrain`
/// gives no `subject` reference but a subject feature whose value is the
/// chain, so the satisfier is the chain's target (`powertrain`).
fn subject_value<'a>(
    items: &'a HashMap<String, (Value, PathBuf)>,
    children: &HashMap<&str, Vec<&'a Value>>,
    claim: &str,
) -> Option<&'a str> {
    let owned_of_type = |owner: &str, kind: &str| -> Option<&'a Value> {
        children
            .get(owner)?
            .iter()
            .copied()
            .find(|c| c.get("@type").and_then(Value::as_str) == Some(kind))
    };
    let subject = reference(owned_of_type(claim, "SubjectMembership")?, "memberElement")?;
    let value = reference(owned_of_type(subject, "FeatureValue")?, "memberElement")?;
    let (expression, _) = items.get(value)?;
    match expression.get("@type").and_then(Value::as_str)? {
        // The referent may itself be a chain (`vehicle.powertrain`).
        "FeatureReferenceExpression" => chain_end(items, reference(expression, "referent")?),
        "FeatureChainExpression" => reference(expression, "targetFeature"),
        _ => None,
    }
}

/// The feature `id` stands for: itself, or the last feature of a chain
/// (`vehicleSpec.massReq`).
fn chain_end<'a>(items: &'a HashMap<String, (Value, PathBuf)>, id: &'a str) -> Option<&'a str> {
    let (item, _) = items.get(id)?;
    match item.get("chainingFeature").and_then(Value::as_array) {
        Some(chain) => chain.last()?.get("@id")?.as_str(),
        None => Some(id),
    }
}

/// The id the first entry of a many-valued reference property points at.
fn first_reference<'a>(item: &'a Value, property: &str) -> Option<&'a str> {
    item.get(property)?
        .as_array()?
        .first()?
        .get("@id")?
        .as_str()
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
fn referenced_path<'a>(
    items: &'a HashMap<String, (Value, PathBuf)>,
    end: &Value,
) -> Option<Vec<&'a str>> {
    let referenced = end
        .get("ownedRelationship")?
        .as_array()?
        .iter()
        .filter_map(|r| r.get("@id").and_then(Value::as_str))
        .filter_map(|id| items.get(id).map(|(item, _)| item))
        .find(|r| r.get("@type").and_then(Value::as_str) == Some("ReferenceSubsetting"))
        .and_then(|r| reference(r, "referencedFeature"))?;
    match items
        .get(referenced)
        .and_then(|(item, _)| item.get("chainingFeature"))
        .and_then(Value::as_array)
    {
        Some(chain) => chain
            .iter()
            .map(|f| f.get("@id").and_then(Value::as_str))
            .collect(),
        None => Some(vec![items.get_key_value(referenced)?.0.as_str()]),
    }
}

/// The id a single-valued reference property points at.
fn reference<'a>(item: &'a Value, property: &str) -> Option<&'a str> {
    item.get(property)?.get("@id")?.as_str()
}
