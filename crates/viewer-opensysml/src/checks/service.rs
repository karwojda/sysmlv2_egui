//! The service's adapter at the `Checker` seam: asks the parsed Model and
//! translates the service's vocabulary into ours.

use std::time::{Duration, Instant};

use opensysml::{AnalysisOptions, VerifyOptions};
use viewer_core::model::{AnalysisResult, ClaimResult, QualifiedName};
use viewer_core::units::{readable_unit, readable_value};

use super::{Checker, ClaimOutcome, ComputedValue, ConstraintOutcome, PartCheck};
use crate::design_values::{Written, WrittenValue};

/// Checks a Model the service has parsed, putting what its reasons say
/// about the Design Values written into it in words.
pub(crate) struct Service<'a> {
    model: &'a opensysml::Model,
    written: Option<&'a Written>,
}

impl<'a> Service<'a> {
    /// Checks `model`, which has no Design Values written into it.
    pub fn new(model: &'a opensysml::Model) -> Self {
        Self {
            model,
            written: None,
        }
    }

    /// Checks `model`, which has the Design Values (and markers) of
    /// `written` written into it.
    pub fn with_design_values(model: &'a opensysml::Model, written: &'a Written) -> Self {
        Self {
            model,
            written: Some(written),
        }
    }

    /// What a service verdict comes to, its reason in words; `within` is
    /// the part the check was about, if known.
    fn result_of(
        &self,
        verdict: &opensysml::Verdict,
        within: Option<&QualifiedName>,
    ) -> ClaimResult {
        match result_of(verdict) {
            ClaimResult::Undecided { reason } => ClaimResult::Undecided {
                reason: self.in_words(reason, within),
            },
            result => result,
        }
    }

    /// `reason`, the service's, in words when it is about a Design Value
    /// (see [`design_value_reason`]); as it is otherwise.
    fn in_words(&self, reason: String, within: Option<&QualifiedName>) -> String {
        self.written
            .and_then(|written| design_value_reason(written, &reason, within))
            .unwrap_or(reason)
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

impl Checker for Service<'_> {
    fn verify_claims(&self) -> Result<Vec<ClaimOutcome>, String> {
        let satisfaction = self
            .model
            .verify_satisfaction(None, &VerifyOptions::default())
            .map_err(|e| self.in_words(e.to_string(), None))?;
        Ok(satisfaction
            .verdicts
            .iter()
            .map(|v| {
                let satisfier = (!v.instance_type_id.is_empty())
                    .then(|| QualifiedName::new(v.instance_type_id.clone()));
                ClaimOutcome {
                    requirement: QualifiedName::new(v.requirement_id.clone()),
                    result: self.result_of(v, satisfier.as_ref()),
                    satisfier,
                }
            })
            .collect())
    }

    fn check_part(&self, part: &QualifiedName) -> Result<PartCheck, String> {
        let validation = self
            .model
            .validate_instance(part.as_str(), None)
            .map_err(|e| self.in_words(e.to_string(), Some(part)))?;
        Ok(PartCheck {
            constraints: validation
                .verdicts
                .iter()
                .filter(|v| v.kind == opensysml::KIND_CONSTRAINT)
                .map(|v| {
                    // The checked part, as `parts` names it from the path.
                    let checked = match v.instance_path.as_str() {
                        "" => part.clone(),
                        path => QualifiedName::new(format!("{part}::{}", path.replace('.', "::"))),
                    };
                    ConstraintOutcome {
                        constraint: QualifiedName::new(v.element_id.clone()),
                        path: v.instance_path.clone(),
                        result: self.result_of(v, Some(&checked)),
                    }
                })
                .collect(),
            values: computed_values(validation.instances()),
        })
    }

    fn run_analysis(&self, analysis: &QualifiedName) -> (Result<AnalysisResult, String>, Duration) {
        let started = Instant::now();
        let run = self
            .model
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
                    .map(|v| self.result_of(v, None)),
            })
            .map_err(|e| self.in_words(e.to_string(), None));
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
