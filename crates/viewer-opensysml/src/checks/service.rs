//! The service's adapter at the `Checker` seam: asks the parsed Model and
//! translates the service's vocabulary into ours.

use std::time::{Duration, Instant};

use opensysml::{AnalysisOptions, VerifyOptions};
use viewer_core::model::{ClaimResult, QualifiedName};

use super::{Checker, ClaimOutcome, ConstraintOutcome, PartCheck, Run};

/// Checks a Model the service has parsed.
pub(crate) struct Service<'a>(pub &'a opensysml::Model);

impl Checker for Service<'_> {
    fn verify_claims(&self) -> Result<Vec<ClaimOutcome>, String> {
        let satisfaction = self
            .0
            .verify_satisfaction(None, &VerifyOptions::default())
            .map_err(|e| e.to_string())?;
        Ok(satisfaction
            .verdicts
            .iter()
            .map(|v| ClaimOutcome {
                requirement: QualifiedName::new(v.requirement_id.clone()),
                satisfier: (!v.instance_type_id.is_empty())
                    .then(|| QualifiedName::new(v.instance_type_id.clone())),
                result: result_of(v),
            })
            .collect())
    }

    fn check_part(&self, part: &QualifiedName) -> Result<PartCheck, String> {
        let validation = self
            .0
            .validate_instance(part.as_str(), None)
            .map_err(|e| e.to_string())?;
        Ok(PartCheck {
            constraints: validation
                .verdicts
                .iter()
                .filter(|v| v.kind == opensysml::KIND_CONSTRAINT)
                .map(|v| ConstraintOutcome {
                    constraint: QualifiedName::new(v.element_id.clone()),
                    path: v.instance_path.clone(),
                    result: result_of(v),
                })
                .collect(),
            values: computed_values(validation.instances()),
        })
    }

    fn run_analysis(&self, analysis: &QualifiedName) -> (Result<Run, String>, Duration) {
        let started = Instant::now();
        let run = self
            .0
            .run_analysis(analysis.as_str(), &AnalysisOptions::default())
            .map(|result| Run {
                outputs: result
                    .outputs
                    .iter()
                    .filter_map(|o| Some((o.name.clone(), value_text(&o.value)?)))
                    .collect(),
                objective: result
                    .verdicts
                    .iter()
                    .find(|v| v.kind == opensysml::KIND_OBJECTIVE)
                    .map(result_of),
            })
            .map_err(|e| e.to_string());
        (run, started.elapsed())
    }
}

/// What a service verdict comes to: undecided with the service's error if
/// it couldn't evaluate; otherwise as its status says, or, for a verdict
/// without status text (an analysis's objective), as its `holds` flag says.
fn result_of(verdict: &opensysml::Verdict) -> ClaimResult {
    if !verdict.error.is_empty() {
        return ClaimResult::Undecided {
            reason: verdict.error.clone(),
        };
    }
    let violated = || ClaimResult::Violated {
        condition: verdict.condition.clone(),
    };
    match verdict.status.as_str() {
        opensysml::STATUS_HOLDS => ClaimResult::Holds,
        opensysml::STATUS_VIOLATED => violated(),
        "" if verdict.holds => ClaimResult::Holds,
        "" => violated(),
        other => ClaimResult::Undecided {
            reason: format!("the service answered `{other}`"),
        },
    }
}

/// Every evaluated feature value of `instances`, as `(feature, text)`: the
/// value of feature `f` on an instance of type symbol `T` belongs to `T::f`.
fn computed_values(instances: &[opensysml::Instance]) -> Vec<(QualifiedName, String)> {
    let mut values = Vec::new();
    for instance in instances {
        let mut features: Vec<(&String, &opensysml::FeatureValue)> =
            instance.feature_values().iter().collect();
        features.sort_by(|a, b| a.0.cmp(b.0));
        for (name, feature) in features {
            let feature_name = QualifiedName::new(format!("{}::{name}", instance.type_symbol_id()));
            for value in feature.value().into_iter().chain(feature.values()) {
                if let Some(text) = value_text(value) {
                    values.push((feature_name.clone(), text));
                }
            }
        }
    }
    values
}

/// A value written as the Model would write it: `1363.0`, `4`, `true`,
/// `"x"`, `1650 [kg]`. `None` for anything else (instances, sequences, …).
fn value_text(value: &opensysml::Value) -> Option<String> {
    use opensysml::{Magnitude, Value};
    let real = |r: f64| {
        if r.fract() == 0.0 && r.abs() < 1e15 {
            format!("{r:.1}")
        } else {
            r.to_string()
        }
    };
    match value {
        Value::Integer(i) => Some(i.to_string()),
        Value::Real(r) => Some(real(*r)),
        Value::Boolean(b) => Some(b.to_string()),
        Value::Text(t) => Some(format!("\"{t}\"")),
        Value::Quantity(q) => {
            let magnitude = match &q.magnitude {
                Magnitude::Integer(i) => i.to_string(),
                Magnitude::Real(r) => real(*r),
                _ => return None,
            };
            Some(format!("{magnitude} [{}]", q.unit))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn values_are_written_like_literals() {
        use opensysml::Value;
        assert_eq!(value_text(&Value::Real(1363.0)).as_deref(), Some("1363.0"));
        assert_eq!(value_text(&Value::Real(15.144)).as_deref(), Some("15.144"));
        assert_eq!(value_text(&Value::Integer(4)).as_deref(), Some("4"));
        assert_eq!(value_text(&Value::Boolean(true)).as_deref(), Some("true"));
        assert_eq!(
            value_text(&Value::Text("x".to_owned())).as_deref(),
            Some("\"x\"")
        );
        assert_eq!(value_text(&Value::InstanceRef(3)), None);
    }
}
