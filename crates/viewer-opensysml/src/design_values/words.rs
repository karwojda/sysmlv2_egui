//! The words: a `Checker` whose reasons about the Design Values (and
//! markers) written into the Model say what they are, not what the
//! service saw (`"no Design Value #0#"`).

use std::time::Duration;

use viewer_core::model::{AnalysisResult, ClaimResult, QualifiedName};
use viewer_core::units::readable_value;

use super::plan::{Written, WrittenValue};
use crate::checks::{Checker, ClaimOutcome, PartCheck};

/// Checks with `checker`, a Model with the Design Values of `written`
/// written into it, putting what its reasons say about them in words.
pub(super) struct Worded<'a> {
    pub checker: &'a dyn Checker,
    pub written: &'a Written,
}

impl Worded<'_> {
    /// `result`, its reason in words when it is Undecided; `within` is the
    /// part the check was about, if known.
    fn result(&self, result: ClaimResult, within: Option<&QualifiedName>) -> ClaimResult {
        match result {
            ClaimResult::Undecided { reason } => ClaimResult::Undecided {
                reason: self.words(reason, within),
            },
            result => result,
        }
    }

    /// `reason` in words when it is about a Design Value (see
    /// [`design_value_reason`]); as it is otherwise.
    fn words(&self, reason: String, within: Option<&QualifiedName>) -> String {
        design_value_reason(self.written, &reason, within).unwrap_or(reason)
    }
}

impl Checker for Worded<'_> {
    fn verify_claims(&self) -> Result<Vec<ClaimOutcome>, String> {
        let outcomes = self
            .checker
            .verify_claims()
            .map_err(|e| self.words(e, None))?;
        Ok(outcomes
            .into_iter()
            .map(|outcome| ClaimOutcome {
                result: self.result(outcome.result, outcome.satisfier.as_ref()),
                ..outcome
            })
            .collect())
    }

    fn check_part(&self, part: &QualifiedName) -> Result<PartCheck, String> {
        let mut check = self
            .checker
            .check_part(part)
            .map_err(|e| self.words(e, Some(part)))?;
        for constraint in &mut check.constraints {
            // The checked part, as `parts` names it from the path.
            let checked = match constraint.path.as_str() {
                "" => part.clone(),
                path => QualifiedName::new(format!("{part}::{}", path.replace('.', "::"))),
            };
            constraint.result = self.result(constraint.result.clone(), Some(&checked));
        }
        Ok(check)
    }

    fn run_analysis(&self, analysis: &QualifiedName) -> (Result<AnalysisResult, String>, Duration) {
        let (run, took) = self.checker.run_analysis(analysis);
        let run = run
            .map(|result| AnalysisResult {
                objective: result.objective.map(|o| self.result(o, None)),
                ..result
            })
            .map_err(|e| self.words(e, None));
        (run, took)
    }
}

/// How the service says a value doesn't fit its feature's type, naming
/// the feature from its part: `feature value pump.mass: type mismatch:
/// cannot write 1500000.0 [mm**3] (dimension L^3) to a feature typed by
/// MassValue (dimension M)`.
const FEATURE_VALUE: &str = "feature value ";
const TYPE_MISMATCH: &str = ": type mismatch: ";
const CANNOT_WRITE: &str = "cannot write ";
const DIMENSION: &str = " (dimension";

/// `reason`, a check's, in words when it is about a Design Value of
/// `written`: `no Design Value for Drive::drive::gearbox::volume (FreeCAD:
/// no object Assembly.Link.)` when there is none, `Design Value for
/// Drive::drive::pump::mass (FreeCAD: Pump.Volume) does not fit it: cannot
/// write …` when its dimension is wrong. `None` otherwise.
///
/// The service names a feature only from its part (`engine.mass`), so
/// several linked features may answer to it. Those beneath `within`, the
/// part the check was about, come first, then those whose value is the
/// one the service could not write; if several remain, all are named.
fn design_value_reason(
    written: &Written,
    reason: &str,
    within: Option<&QualifiedName>,
) -> Option<String> {
    let missing: Vec<&str> = written
        .markers
        .iter()
        .filter(|(marker, _)| reason.contains(marker.as_str()))
        .map(|(_, words)| words.as_str())
        .collect();
    if !missing.is_empty() {
        return Some(missing.join("; "));
    }
    let (_, rest) = reason.split_once(FEATURE_VALUE)?;
    let (path, mismatch) = rest.split_once(TYPE_MISMATCH)?;
    let suffix = format!("::{}", path.replace('.', "::"));
    let named: Vec<&WrittenValue> = written
        .values
        .iter()
        .filter(|w| w.feature.as_str().ends_with(&suffix))
        .collect();
    let beneath = within.map(|part| format!("{part}::"));
    let named = narrowed(named, |w| {
        beneath
            .as_ref()
            .is_some_and(|b| w.feature.as_str().starts_with(b.as_str()))
    });
    let refused = mismatch
        .strip_prefix(CANNOT_WRITE)
        .and_then(|r| r.split_once(DIMENSION))
        .map(|(value, _)| readable_value(value));
    let named = narrowed(named, |w| {
        refused
            .as_ref()
            .is_some_and(|r| *r == readable_value(&w.value))
    });
    if named.is_empty() {
        return None;
    }
    let words: Vec<&str> = named.iter().map(|w| w.words.as_str()).collect();
    Some(format!(
        "{} does not fit it: {mismatch}",
        words.join(" or ")
    ))
}

/// Those of `candidates` that are `wanted`, unless none is.
fn narrowed(
    candidates: Vec<&WrittenValue>,
    wanted: impl Fn(&WrittenValue) -> bool,
) -> Vec<&WrittenValue> {
    let kept: Vec<&WrittenValue> = candidates.iter().copied().filter(|w| wanted(w)).collect();
    if kept.is_empty() { candidates } else { kept }
}
