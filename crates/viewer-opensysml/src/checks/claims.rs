//! Satisfaction Claims: each outcome goes to the claim it is about.

use std::collections::BTreeMap;

use viewer_core::model::{ClaimResult, Element, ElementKind, QualifiedName};

use super::{Checker, ClaimOutcome};

/// Verifies the claims in `elements`, if there are any, and records each
/// result. An outcome goes to the first claim, in declaration order, on its
/// requirement and by its satisfier (any satisfier when the service names
/// none). Claims left without an outcome are Unchecked; when verifying
/// failed, all of them are, and the warning says why.
pub(super) fn check(
    checker: &dyn Checker,
    elements: &mut BTreeMap<QualifiedName, Element>,
) -> Option<String> {
    let mut unmatched: Vec<QualifiedName> = elements
        .values()
        .filter(|e| e.kind == ElementKind::SatisfyRequirement)
        .map(|e| e.qualified_name.clone())
        .collect();
    if unmatched.is_empty() {
        return None;
    }
    unmatched.sort_by_key(declaration_order);
    let outcomes = match checker.verify_claims() {
        Ok(outcomes) => outcomes,
        Err(reason) => {
            for name in &unmatched {
                set(
                    elements,
                    name,
                    ClaimResult::Unchecked {
                        reason: reason.clone(),
                    },
                );
            }
            return Some(format!(
                "requirements could not be checked, so their status is Undecided: {reason}"
            ));
        }
    };
    for ClaimOutcome {
        requirement,
        satisfier,
        result,
    } in outcomes
    {
        let matches = |name: &QualifiedName| {
            let claim = &elements[name];
            claim.satisfies.as_ref() == Some(&requirement)
                && satisfier
                    .as_ref()
                    .is_none_or(|s| claim.satisfier.as_ref() == Some(s))
        };
        if let Some(index) = unmatched.iter().position(matches) {
            let name = unmatched.remove(index);
            set(elements, &name, result);
        }
    }
    for name in unmatched {
        let reason = "the service gave no result for it".to_owned();
        set(elements, &name, ClaimResult::Unchecked { reason });
    }
    None
}

fn set(elements: &mut BTreeMap<QualifiedName, Element>, name: &QualifiedName, result: ClaimResult) {
    if let Some(claim) = elements.get_mut(name) {
        claim.claim_result = Some(result);
    }
}

/// Generated names number claims within their owner (`Req::@8`,
/// `Req::@10`); order by that number, not as text.
fn declaration_order(name: &QualifiedName) -> (String, u64) {
    let text = name.as_str();
    match text
        .rsplit_once("::@")
        .and_then(|(owner, n)| Some((owner, n.parse::<u64>().ok()?)))
    {
        Some((owner, n)) => (owner.to_owned(), n),
        None => (text.to_owned(), 0),
    }
}
