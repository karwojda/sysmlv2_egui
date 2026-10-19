//! Satisfaction Status: whether a requirement usage is met, derived from
//! the results of checking its Satisfaction Claims.

use crate::model::{ClaimResult, Element, ElementKind, Model, QualifiedName};

/// Whether a requirement usage is met. With several claims, the worst one
/// decides: Violated, then Undecided, then Satisfied.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SatisfactionStatus {
    Satisfied,
    Undecided,
    Violated,
    /// Nothing claims to satisfy it.
    NotClaimed,
}

impl SatisfactionStatus {
    /// The mark drawn on Diagrams: ✓ ✗ ? ○.
    pub fn mark(self) -> &'static str {
        match self {
            SatisfactionStatus::Satisfied => "✓",
            SatisfactionStatus::Violated => "✗",
            SatisfactionStatus::Undecided => "?",
            SatisfactionStatus::NotClaimed => "○",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            SatisfactionStatus::Satisfied => "Satisfied",
            SatisfactionStatus::Violated => "Violated",
            SatisfactionStatus::Undecided => "Undecided",
            SatisfactionStatus::NotClaimed => "Not claimed",
        }
    }
}

/// What a claim's result counts as; an unchecked claim (or one with no
/// result yet) is Undecided.
pub fn claim_status(claim: &Element) -> SatisfactionStatus {
    match &claim.claim_result {
        Some(ClaimResult::Holds) => SatisfactionStatus::Satisfied,
        Some(ClaimResult::Violated { .. }) => SatisfactionStatus::Violated,
        Some(ClaimResult::Undecided { .. } | ClaimResult::Unchecked { .. }) | None => {
            SatisfactionStatus::Undecided
        }
    }
}

/// How a claim's result reads in the Properties panel: `✓ holds`,
/// `✗ violated: v.range >= 500.0`, `? undecided: no value for v.cost`.
pub fn describe_claim(claim: &Element) -> String {
    let mark = claim_status(claim).mark();
    match &claim.claim_result {
        Some(ClaimResult::Holds) => format!("{mark} holds"),
        Some(ClaimResult::Violated { condition }) => format!("{mark} violated: {condition}"),
        Some(ClaimResult::Undecided { reason }) => format!("{mark} undecided: {reason}"),
        Some(ClaimResult::Unchecked { reason }) => format!("{mark} not checked: {reason}"),
        None => format!("{mark} not checked"),
    }
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

/// The Satisfaction Status of a requirement usage; `None` for anything
/// else (a requirement definition can't be satisfied).
pub fn status(model: &Model, requirement: &QualifiedName) -> Option<SatisfactionStatus> {
    let element = model
        .element(requirement)
        .filter(|e| e.kind == ElementKind::RequirementUsage)?;
    let worst = claims_on(model, &element.qualified_name)
        .map(claim_status)
        .max_by_key(|s| match s {
            SatisfactionStatus::Satisfied => 0,
            SatisfactionStatus::Undecided => 1,
            SatisfactionStatus::Violated => 2,
            SatisfactionStatus::NotClaimed => 0,
        });
    Some(worst.unwrap_or(SatisfactionStatus::NotClaimed))
}
