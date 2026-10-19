//! Constraint Results: what checking asserted constraints on parts found,
//! and how a part or a constraint reads because of it.

use crate::model::{ClaimResult, ConstraintCheck, Element, ElementKind, Model, QualifiedName};

/// What checking an asserted constraint found, on one part or, for a
/// constraint usage, worst over every part.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ConstraintStatus {
    Holds,
    Undecided,
    Violated,
}

impl ConstraintStatus {
    /// The mark drawn on Diagrams: ✓ ✗ ?.
    pub fn mark(self) -> &'static str {
        match self {
            ConstraintStatus::Holds => "✓",
            ConstraintStatus::Violated => "✗",
            ConstraintStatus::Undecided => "?",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            ConstraintStatus::Holds => "Holds",
            ConstraintStatus::Violated => "Violated",
            ConstraintStatus::Undecided => "Undecided",
        }
    }
}

/// What one check counts as; a check that couldn't run is Undecided.
pub fn check_status(result: &ClaimResult) -> ConstraintStatus {
    match result {
        ClaimResult::Holds => ConstraintStatus::Holds,
        ClaimResult::Violated { .. } => ConstraintStatus::Violated,
        ClaimResult::Undecided { .. } | ClaimResult::Unchecked { .. } => {
            ConstraintStatus::Undecided
        }
    }
}

fn worst(statuses: impl Iterator<Item = ConstraintStatus>) -> Option<ConstraintStatus> {
    statuses.max_by_key(|s| match s {
        ConstraintStatus::Holds => 0,
        ConstraintStatus::Undecided => 1,
        ConstraintStatus::Violated => 2,
    })
}

/// An asserted constraint usage's status: the worst of its checks, or
/// Undecided when it was never checked. `None` for anything else.
pub fn status(element: &Element) -> Option<ConstraintStatus> {
    (element.kind == ElementKind::ConstraintUsage && element.is_asserted).then(|| {
        worst(element.checks.iter().map(|c| check_status(&c.result)))
            .unwrap_or(ConstraintStatus::Undecided)
    })
}

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
    checks_on(model, part)
        .any(|(_, check)| check_status(&check.result) == ConstraintStatus::Violated)
}

/// How a constraint reads in a compartment row and the Properties panel:
/// its expression in braces, or `name:Def` for a usage of a definition.
pub fn row_text(constraint: &Element) -> String {
    match &constraint.expression {
        Some(expression) => format!("{{{expression}}}"),
        None => crate::notation::usage_label(constraint),
    }
}

/// `✓ holds`, `✗ violated: fuelMass <= fuelMassMax`, `? undecided: …`.
pub fn describe(result: &ClaimResult) -> String {
    let mark = check_status(result).mark();
    match result {
        ClaimResult::Holds => format!("{mark} holds"),
        ClaimResult::Violated { condition } => format!("{mark} violated: {condition}"),
        ClaimResult::Undecided { reason } => format!("{mark} undecided: {reason}"),
        ClaimResult::Unchecked { reason } => format!("{mark} not checked: {reason}"),
    }
}
