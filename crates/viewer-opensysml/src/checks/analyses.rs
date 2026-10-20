//! Analysis usages: which run, and each one's Analysis Result.

use std::collections::BTreeMap;
use std::path::Path;
use std::time::Duration;

use viewer_core::backend::Analyses;
use viewer_core::model::{AnalysisResult, ClaimResult, Element, ElementKind, QualifiedName};

use super::Checker;

/// Above this, a load's analyses are slow enough to point at the switch.
const SLOW: Duration = Duration::from_secs(3);

/// Which analyses run: what the caller asks for, the Model's setting in
/// `folder` when it says "as set". A broken settings file gives a warning
/// and the default, every analysis.
pub(super) fn selection(folder: &Path, asked: &Analyses) -> (Analyses, Option<String>) {
    match asked {
        Analyses::AsSet => match viewer_core::settings::load(folder) {
            Ok(settings) if settings.run_analyses => (Analyses::All, None),
            Ok(_) => (Analyses::None, None),
            Err(error) => (
                Analyses::All,
                Some(format!("{error}; using the default settings")),
            ),
        },
        other => (other.clone(), None),
    }
}

/// Runs the analysis usages in `elements` that `which` names and records
/// their results; the rest have none ("not run"). Returns a warning for
/// each that couldn't be run, and one if they were slow.
pub(super) fn run(
    checker: &dyn Checker,
    elements: &mut BTreeMap<QualifiedName, Element>,
    which: &Analyses,
) -> Vec<String> {
    let analyses: Vec<QualifiedName> = elements
        .values()
        .filter(|e| e.kind == ElementKind::AnalysisUsage)
        .filter(|e| match which {
            Analyses::All | Analyses::AsSet => true,
            Analyses::None => false,
            Analyses::Only(name) => &e.qualified_name == name,
        })
        .map(|e| e.qualified_name.clone())
        .collect();
    let mut took = Duration::ZERO;
    let mut warnings = Vec::new();
    for analysis in analyses {
        let (run, time) = checker.run_analysis(&analysis);
        took += time;
        let result = match run {
            Ok(result) => result,
            Err(reason) => {
                warnings.push(format!(
                    "analysis `{analysis}` could not be run, so it is Undecided: {reason}"
                ));
                AnalysisResult {
                    outputs: Vec::new(),
                    objective: Some(ClaimResult::Unchecked { reason }),
                }
            }
        };
        if let Some(element) = elements.get_mut(&analysis) {
            element.analysis_result = Some(result);
        }
    }
    if took > SLOW {
        warnings.push(format!(
            "analyses took {:.1} s; to load faster, untick \"Run analyses on load\" (run_analyses in diagrams/settings.json)",
            took.as_secs_f32()
        ));
    }
    warnings
}
