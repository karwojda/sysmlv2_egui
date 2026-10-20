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
mod tests;

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use viewer_core::backend::Analyses;
use viewer_core::model::{ClaimResult, Diagnostic, Element, QualifiedName, Severity};

pub(crate) use service::Service;

/// What the checks ask of the service.
pub(crate) trait Checker {
    /// A verdict for each Satisfaction Claim the service could check.
    fn verify_claims(&self) -> Result<Vec<ClaimVerdict>, String>;
    /// Checks part usage `part` and everything beneath it.
    fn check_part(&self, part: &QualifiedName) -> Result<PartCheck, String>;
    /// Runs analysis usage `analysis`, and says how long that took.
    fn run_analysis(&self, analysis: &QualifiedName) -> (Result<Run, String>, Duration);
}

/// What verifying one claim found.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ClaimVerdict {
    /// The requirement usage the claim names.
    pub requirement: QualifiedName,
    /// The satisfier, when the service names one.
    pub satisfier: Option<QualifiedName>,
    pub result: ClaimResult,
}

/// What checking one part tree found.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct PartCheck {
    pub constraints: Vec<ConstraintVerdict>,
    /// Every evaluated feature value, as `(feature, text)`, in the order
    /// the instances were reported.
    pub values: Vec<(QualifiedName, String)>,
}

/// What checking one asserted constraint on one part found.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConstraintVerdict {
    pub constraint: QualifiedName,
    /// Where the checked part sits under the checked tree's root (`tank`,
    /// `axle.wheel`), empty for the root itself.
    pub path: String,
    pub result: ClaimResult,
}

/// What one analysis run gave.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Run {
    pub outputs: Vec<(String, String)>,
    pub objective: Option<ClaimResult>,
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
