//! Constraint Results: what checking asserted constraints on parts found,
//! seen from the part. The constraint's own Verdict is in `verdict`.

use crate::model::{ConstraintCheck, Element, ElementKind, Model, QualifiedName};
use crate::verdict::Status;

/// Every check on `part`, with its constraint: the part's own asserted
/// constraints and those it inherits, in qualified-name order.
pub fn checks_on<'a>(
    model: &'a Model,
    part: &'a QualifiedName,
) -> impl Iterator<Item = (&'a Element, &'a ConstraintCheck)> {
    model
        .elements()
        .filter(|e| e.kind == ElementKind::ConstraintUsage && e.is_asserted)
        .flat_map(move |c| {
            c.checks
                .iter()
                .filter(move |check| &check.part == part)
                .map(move |check| (c, check))
        })
}

/// Whether `part` has a violated constraint of its own (inherited ones
/// included, nested parts' not).
pub fn has_violation(model: &Model, part: &QualifiedName) -> bool {
    checks_on(model, part).any(|(_, check)| Status::of(&check.result) == Status::Violated)
}

/// How a constraint reads in a compartment row and the Properties panel:
/// its expression in braces, or `name:Def` for a usage of a definition.
pub fn row_text(constraint: &Element) -> String {
    match &constraint.expression {
        Some(expression) => format!("{{{expression}}}"),
        None => crate::notation::usage_label(constraint),
    }
}
