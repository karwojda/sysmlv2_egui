//! Checks a loaded Model with the service: its Satisfaction Claims, every
//! top-level part tree (Constraint Results and Computed Values) and its
//! analysis usages, and records what the service found on the elements.
//!
//! Everything asked of the service goes through the `Checker` seam, which
//! answers in our own terms; the service's adapter is `Service`.

mod analyses;
mod claims;
mod parts;
mod service;
#[cfg(test)]
pub(crate) mod tests;

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use viewer_core::backend::Analyses;
use viewer_core::model::{
    AnalysisResult, ClaimResult, Diagnostic, Element, QualifiedName, Severity,
};

pub(crate) use service::Service;

/// What the checks ask of the service.
pub(crate) trait Checker {
    /// What checking each Satisfaction Claim found, for those the service
    /// could check.
    fn verify_claims(&self) -> Result<Vec<ClaimOutcome>, String>;
    /// Checks part usage `part` and everything beneath it.
    fn check_part(&self, part: &QualifiedName) -> Result<PartCheck, String>;
    /// Runs analysis usage `analysis`, and says how long that took.
    fn run_analysis(&self, analysis: &QualifiedName) -> (Result<AnalysisResult, String>, Duration);
}

/// What verifying one claim found.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ClaimOutcome {
    /// The requirement usage the claim names.
    pub requirement: QualifiedName,
    /// The satisfier, when the service names one.
    pub satisfier: Option<QualifiedName>,
    pub result: ClaimResult,
}

/// What checking one part tree found.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PartCheck {
    pub constraints: Vec<ConstraintOutcome>,
    /// Every evaluated feature value, in the order the instances were
    /// reported.
    pub values: Vec<ComputedValue>,
}

/// What checking one asserted constraint on one part found.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConstraintOutcome {
    pub constraint: QualifiedName,
    /// Where the checked part sits under the checked tree's root (`tank`,
    /// `axle.wheel`), empty for the root itself.
    pub path: String,
    pub result: ClaimResult,
}

/// One Computed Value: what the service evaluated feature `feature` to,
/// written as the Model would write it (`1363.0`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ComputedValue {
    pub feature: QualifiedName,
    pub text: String,
}

/// Checks the Model in `elements` with `checker` and records the results:
/// claims first, then the analyses `analyses` asks for (the Model's setting
/// in `folder` when it says "as set"), then every top-level part. Returns a
/// warning for each thing that couldn't be checked.
pub(crate) fn run(
    checker: &dyn Checker,
    elements: &mut BTreeMap<QualifiedName, Element>,
    folder: &Path,
    analyses: &Analyses,
) -> Vec<Diagnostic> {
    let mut warnings = Vec::new();
    warnings.extend(claims::check(checker, elements));
    let (which, setting) = analyses::selection(folder, analyses);
    warnings.extend(setting);
    warnings.extend(analyses::run(checker, elements, &which));
    warnings.extend(parts::check(checker, elements));
    warnings.into_iter().map(warning).collect()
}

fn warning(message: String) -> Diagnostic {
    Diagnostic {
        severity: Severity::Warning,
        message,
        location: None,
    }
}
