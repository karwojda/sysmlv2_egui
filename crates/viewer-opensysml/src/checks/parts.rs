//! Part trees: Constraint Results on the parts they were checked on, and
//! Computed Values on their features.

use std::collections::BTreeMap;

use viewer_core::model::{ClaimResult, ConstraintCheck, Element, ElementKind, QualifiedName};

use super::{Checker, ComputedValue};

/// Checks every top-level part usage (one owned by a package) in
/// `elements` and records what was found. Returns a warning for each part
/// that couldn't be checked.
pub(super) fn check(
    checker: &dyn Checker,
    elements: &mut BTreeMap<QualifiedName, Element>,
) -> Vec<String> {
    let roots: Vec<QualifiedName> = elements
        .values()
        .filter(|e| e.kind == ElementKind::PartUsage)
        .filter(|e| {
            e.owner
                .as_ref()
                .and_then(|o| elements.get(o))
                .is_none_or(|o| o.kind == ElementKind::Package)
        })
        .map(|e| e.qualified_name.clone())
        .collect();
    let mut warnings = Vec::new();
    for root in roots {
        match checker.check_part(&root) {
            Ok(found) => {
                record_values(elements, found.values);
                for outcome in found.constraints {
                    let part = if outcome.path.is_empty() {
                        root.clone()
                    } else {
                        QualifiedName::new(format!("{root}::{}", outcome.path.replace('.', "::")))
                    };
                    if let Some(constraint) = elements.get_mut(&outcome.constraint) {
                        constraint.checks.push(ConstraintCheck {
                            part,
                            result: outcome.result,
                        });
                    }
                }
            }
            Err(reason) => {
                unchecked(elements, &root, &reason);
                warnings.push(format!(
                    "constraints of `{root}` could not be checked, so they are Undecided: {reason}"
                ));
            }
        }
    }
    warnings
}

/// Marks the asserted constraints owned in `root`'s tree Unchecked on
/// their owners.
fn unchecked(elements: &mut BTreeMap<QualifiedName, Element>, root: &QualifiedName, reason: &str) {
    let prefix = format!("{root}::");
    for constraint in elements.values_mut().filter(|e| {
        e.kind == ElementKind::ConstraintUsage
            && e.is_asserted
            && e.owner
                .as_ref()
                .is_some_and(|o| o == root || o.as_str().starts_with(&prefix))
    }) {
        let part = constraint.owner.clone().expect("filtered on its owner");
        constraint.checks.push(ConstraintCheck {
            part,
            result: ClaimResult::Unchecked {
                reason: reason.to_owned(),
            },
        });
    }
}

/// Gives each feature in `elements` its Computed Values, distinct and in
/// first-seen order. Values for features the Model doesn't have, and values
/// equal to the feature's own written value, are dropped.
fn record_values(elements: &mut BTreeMap<QualifiedName, Element>, values: Vec<ComputedValue>) {
    for ComputedValue { feature, text } in values {
        // A value the Model writes out already says itself.
        if let Some(element) = elements.get_mut(&feature)
            && element.value.as_deref() != Some(text.as_str())
            && !element.computed.contains(&text)
        {
            element.computed.push(text);
        }
    }
}
