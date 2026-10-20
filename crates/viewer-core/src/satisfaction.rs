//! Satisfaction Status, the Verdict of a requirement usage: derived from
//! the results of checking its Satisfaction Claims, and rolled up from its
//! nested requirements.

use crate::model::{Element, ElementKind, Model, QualifiedName};
use crate::verdict::{self, Status};

/// What a claim's result counts as; an unchecked claim (or one with no
/// result yet) is Undecided.
pub fn claim_status(claim: &Element) -> Status {
    claim
        .claim_result
        .as_ref()
        .map_or(Status::Undecided, Status::of)
}

/// The claims on `requirement`, in declaration order.
pub fn claims_on<'a>(
    model: &'a Model,
    requirement: &'a QualifiedName,
) -> impl Iterator<Item = &'a Element> {
    model
        .claims()
        .filter(move |c| c.satisfies.as_ref() == Some(requirement))
}

/// The Satisfaction Status of requirement usage `element`.
///
/// A requirement's own claims decide it, the worst one winning; with none
/// it is Not claimed (absent). A composite requirement (one with nested
/// requirement usages) also rolls up its children's statuses, worst
/// first. Claims on a composite with no constraint of its own have nothing
/// to check, so only its children count.
pub(crate) fn status(model: &Model, element: &Element) -> Status {
    let children: Vec<Status> = nested(model, element)
        .map(|child| status(model, child))
        .collect();
    let own_counts = children.is_empty() || has_constraints(model, element);
    let own: Vec<Status> = claims_on(model, &element.qualified_name)
        .filter(|_| own_counts)
        .map(claim_status)
        .collect();
    verdict::worst(own.into_iter().chain(children)).unwrap_or(Status::Absent)
}

/// The requirement usages `element` owns: the requirements a composite
/// requirement rolls up.
pub fn nested<'a>(model: &'a Model, element: &'a Element) -> impl Iterator<Item = &'a Element> {
    model
        .owned_members(&element.qualified_name)
        .filter(|m| m.kind == ElementKind::RequirementUsage)
}

/// Whether a requirement has a `require` constraint of its own or from its
/// definition, so that a claim on it has something to check.
fn has_constraints(model: &Model, element: &Element) -> bool {
    !element.require.is_empty()
        || element
            .typed_by
            .as_ref()
            .and_then(|d| model.element(d))
            .is_some_and(|d| !d.require.is_empty())
}
