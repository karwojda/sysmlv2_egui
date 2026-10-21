//! The service's adapter at the `Checker` seam: asks the parsed Model and
//! translates the service's vocabulary into ours.

use std::time::{Duration, Instant};

use opensysml::{AnalysisOptions, VerifyOptions};
use viewer_core::model::{AnalysisResult, ClaimResult, QualifiedName};
use viewer_core::units::readable_unit;

use super::{Checker, ClaimOutcome, ComputedValue, ConstraintOutcome, PartCheck};

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

    fn run_analysis(&self, analysis: &QualifiedName) -> (Result<AnalysisResult, String>, Duration) {
        let started = Instant::now();
        let run = self
            .0
            .run_analysis(analysis.as_str(), &AnalysisOptions::default())
            .map(|result| AnalysisResult {
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

/// Every evaluated feature value of `instances`: the value of feature `f`
/// on an instance of type symbol `T` belongs to `T::f`.
fn computed_values(instances: &[opensysml::Instance]) -> Vec<ComputedValue> {
    let mut values = Vec::new();
    for instance in instances {
        let mut features: Vec<(&String, &opensysml::FeatureValue)> =
            instance.feature_values().iter().collect();
        features.sort_by(|a, b| a.0.cmp(b.0));
        for (name, feature) in features {
            let feature_name = QualifiedName::new(format!("{}::{name}", instance.type_symbol_id()));
            for value in feature.value().into_iter().chain(feature.values()) {
                if let Some(text) = value_text(value) {
                    values.push(ComputedValue {
                        feature: feature_name.clone(),
                        text,
                    });
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
        Value::BigInteger(i) => Some(i.as_str().to_owned()),
        // An exact fraction (`157/20`) as the decimal number it comes to.
        Value::Rational(q) => Some(real(q.to_f64())),
        Value::Real(r) => Some(real(*r)),
        Value::Boolean(b) => Some(b.to_string()),
        Value::Text(t) => Some(format!("\"{t}\"")),
        Value::Quantity(q) => {
            let magnitude = match &q.magnitude {
                Magnitude::Integer(i) => i.to_string(),
                Magnitude::BigInteger(i) => i.as_str().to_owned(),
                Magnitude::Rational(q) => real(q.to_f64()),
                Magnitude::Real(r) => real(*r),
            };
            Some(format!("{magnitude} [{}]", readable_unit(&q.unit)))
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

    fn quantity(magnitude: opensysml::Magnitude, unit: &str) -> opensysml::Value {
        opensysml::Value::Quantity(opensysml::Quantity {
            magnitude,
            unit: unit.to_owned(),
            unit_term: None,
        })
    }

    fn rational(numerator: &str, denominator: &str) -> opensysml::Rational {
        opensysml::Rational::parse(numerator, denominator).unwrap()
    }

    #[test]
    fn exact_fractions_are_written_as_decimal_numbers() {
        use opensysml::{Magnitude, Value};
        assert_eq!(
            value_text(&quantity(Magnitude::Rational(rational("157", "20")), "kg")).as_deref(),
            Some("7.85 [kg]")
        );
        assert_eq!(
            value_text(&Value::Rational(rational("-1", "3"))).as_deref(),
            Some("-0.3333333333333333")
        );
    }

    #[test]
    fn integers_beyond_i64_are_written_as_their_digits() {
        use opensysml::{BigInteger, Magnitude, Value};
        let big = || BigInteger::parse("30000000000000000000000").unwrap();
        assert_eq!(
            value_text(&Value::BigInteger(big())).as_deref(),
            Some("30000000000000000000000")
        );
        assert_eq!(
            value_text(&quantity(Magnitude::BigInteger(big()), "kg")).as_deref(),
            Some("30000000000000000000000 [kg]")
        );
    }

    #[test]
    fn units_read_like_the_model_writes_them() {
        let unit = |service: &str| {
            value_text(&quantity(opensysml::Magnitude::Integer(1), service)).unwrap()
        };
        for (service, shown) in [
            ("kg", "1 [kg]"),
            ("SI::kg", "1 [kg]"),
            ("SI::'m³'", "1 [m³]"),
            ("mm**3", "1 [mm³]"),
            ("m**-1", "1 [m⁻¹]"),
            ("mm^2", "1 [mm²]"),
            ("SI::'kg⋅m⁻³'", "1 [kg/m³]"),
            ("SI::'kg⋅m⋅s⁻²'", "1 [kg⋅m/s²]"),
            ("SI::'kg⋅m⁻¹⋅s⁻²'", "1 [kg/(m⋅s²)]"),
            ("SI::'s⁻¹'", "1 [s⁻¹]"),
            ("SI::'m⋅s⁻¹¹'", "1 [m/s¹¹]"),
            ("kg/m^3", "1 [kg/m³]"),
        ] {
            assert_eq!(unit(service), shown, "{service}");
        }
    }
}
