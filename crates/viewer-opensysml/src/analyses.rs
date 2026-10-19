//! Runs the Model's analysis usages with the service and records each
//! Analysis Result.

use std::collections::BTreeMap;

use opensysml::AnalysisOptions;
use viewer_core::model::{AnalysisResult, ClaimResult, Element, ElementKind, QualifiedName};

/// An objective verdict, reduced to what recording needs.
#[derive(Clone, Debug, PartialEq)]
struct Objective {
    holds: bool,
    condition: String,
    error: String,
}

/// What one run gave, reduced to what recording needs.
#[derive(Clone, Debug, PartialEq)]
struct Run {
    outputs: Vec<(String, String)>,
    objective: Option<Objective>,
}

/// Runs every analysis usage in `elements` and records its result. Returns
/// a warning for each that couldn't be run.
pub fn run(
    parsed: &opensysml::Model,
    elements: &mut BTreeMap<QualifiedName, Element>,
) -> Vec<String> {
    let analyses: Vec<QualifiedName> = elements
        .values()
        .filter(|e| e.kind == ElementKind::AnalysisUsage)
        .map(|e| e.qualified_name.clone())
        .collect();
    let mut warnings = Vec::new();
    for analysis in analyses {
        let run = parsed
            .run_analysis(analysis.as_str(), &AnalysisOptions::default())
            .map(|result| Run {
                outputs: result
                    .outputs
                    .iter()
                    .filter_map(|o| {
                        Some((o.name.clone(), crate::constraints::value_text(&o.value)?))
                    })
                    .collect(),
                objective: result
                    .verdicts
                    .iter()
                    .find(|v| v.kind == opensysml::KIND_OBJECTIVE)
                    .map(|v| Objective {
                        holds: v.holds,
                        condition: v.condition.clone(),
                        error: v.error.clone(),
                    }),
            })
            .map_err(|error| error.to_string());
        if let Some(warning) = record(elements, &analysis, run) {
            warnings.push(format!(
                "analysis `{analysis}` could not be run, so it is Undecided: {warning}"
            ));
        }
    }
    warnings
}

/// Records a run of `analysis`. A failed run leaves the objective
/// Unchecked and returns the reason.
fn record(
    elements: &mut BTreeMap<QualifiedName, Element>,
    analysis: &QualifiedName,
    run: Result<Run, String>,
) -> Option<String> {
    let element = elements.get_mut(analysis)?;
    match run {
        Ok(run) => {
            let objective = run.objective.map(|o| {
                if !o.error.is_empty() {
                    ClaimResult::Undecided { reason: o.error }
                } else if o.holds {
                    ClaimResult::Holds
                } else {
                    ClaimResult::Violated {
                        condition: o.condition,
                    }
                }
            });
            element.analysis_result = Some(AnalysisResult {
                outputs: run.outputs,
                objective,
            });
            None
        }
        Err(reason) => {
            element.analysis_result = Some(AnalysisResult {
                outputs: Vec::new(),
                objective: Some(ClaimResult::Unchecked {
                    reason: reason.clone(),
                }),
            });
            Some(reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qn(s: &str) -> QualifiedName {
        QualifiedName::new(s)
    }

    fn one() -> BTreeMap<QualifiedName, Element> {
        [(
            qn("A::check"),
            Element::new(qn("A::check"), ElementKind::AnalysisUsage),
        )]
        .into_iter()
        .collect()
    }

    fn result(elements: &BTreeMap<QualifiedName, Element>) -> AnalysisResult {
        elements[&qn("A::check")].analysis_result.clone().unwrap()
    }

    fn run(holds: bool, condition: &str, error: &str) -> Run {
        Run {
            outputs: vec![("result".to_owned(), "230.0".to_owned())],
            objective: Some(Objective {
                holds,
                condition: condition.to_owned(),
                error: error.to_owned(),
            }),
        }
    }

    #[test]
    fn outputs_and_a_holding_objective_are_recorded() {
        let mut elements = one();

        assert_eq!(
            record(&mut elements, &qn("A::check"), Ok(run(true, "", ""))),
            None
        );

        let recorded = result(&elements);
        assert_eq!(
            recorded.outputs,
            [("result".to_owned(), "230.0".to_owned())]
        );
        assert_eq!(recorded.objective, Some(ClaimResult::Holds));
    }

    #[test]
    fn a_failing_objective_keeps_its_condition() {
        let mut elements = one();

        record(
            &mut elements,
            &qn("A::check"),
            Ok(run(false, "result <= limit", "")),
        );

        assert_eq!(
            result(&elements).objective,
            Some(ClaimResult::Violated {
                condition: "result <= limit".to_owned()
            })
        );
    }

    #[test]
    fn an_evaluation_error_is_undecided() {
        let mut elements = one();

        record(
            &mut elements,
            &qn("A::check"),
            Ok(run(false, "", "limit has no value")),
        );

        assert_eq!(
            result(&elements).objective,
            Some(ClaimResult::Undecided {
                reason: "limit has no value".to_owned()
            })
        );
    }

    #[test]
    fn an_analysis_without_objective_records_only_outputs() {
        let mut elements = one();

        record(
            &mut elements,
            &qn("A::check"),
            Ok(Run {
                objective: None,
                ..run(true, "", "")
            }),
        );

        assert_eq!(result(&elements).objective, None);
    }

    #[test]
    fn a_failed_run_is_unchecked_and_says_why() {
        let mut elements = one();

        let warning = record(
            &mut elements,
            &qn("A::check"),
            Err("analysis is not supported".to_owned()),
        );

        assert_eq!(warning.as_deref(), Some("analysis is not supported"));
        assert_eq!(
            result(&elements).objective,
            Some(ClaimResult::Unchecked {
                reason: "analysis is not supported".to_owned()
            })
        );
    }
}
