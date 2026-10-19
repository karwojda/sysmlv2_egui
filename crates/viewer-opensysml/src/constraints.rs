//! Checks the Model's asserted constraints with the service, one top-level
//! part tree at a time, and records each result on its constraint.

use std::collections::BTreeMap;

use viewer_core::model::{ClaimResult, ConstraintCheck, Element, ElementKind, QualifiedName};

/// One `constraint` verdict as the service gives it, reduced to what
/// recording needs.
#[derive(Clone, Debug, PartialEq)]
struct Verdict {
    constraint: QualifiedName,
    /// Where the checked part sits under the validated one (`tank`), empty
    /// for the validated part itself.
    path: String,
    status: String,
    condition: String,
    error: String,
}

/// Checks every top-level part usage (one owned by a package) of `parsed`
/// with the service and records the results on the asserted constraints in
/// `elements`. Returns a warning for each part that couldn't be checked.
pub fn check(
    parsed: &opensysml::Model,
    elements: &mut BTreeMap<QualifiedName, Element>,
) -> Vec<String> {
    let roots: Vec<QualifiedName> = elements
        .values()
        .filter(|e| e.kind == ElementKind::PartUsage)
        .filter(|e| {
            e.owner
                .as_ref()
                .and_then(|o| elements.get(o))
                .is_none_or(|o| o.kind == ElementKind::Package)
        })
        .map(|e| e.qualified_name.clone())
        .collect();
    let mut warnings = Vec::new();
    for root in roots {
        let verdicts = match parsed.validate_instance(root.as_str(), None) {
            Ok(validation) => {
                record_values(elements, computed_values(validation.instances()));
                Ok(validation
                    .verdicts
                    .iter()
                    .filter(|v| v.kind == opensysml::KIND_CONSTRAINT)
                    .map(|v| Verdict {
                        constraint: QualifiedName::new(v.element_id.clone()),
                        path: v.instance_path.clone(),
                        status: v.status.clone(),
                        condition: v.condition.clone(),
                        error: v.error.clone(),
                    })
                    .collect())
            }
            Err(error) => Err(error.to_string()),
        };
        if let Some(warning) = record(elements, &root, verdicts) {
            warnings.push(format!(
                "constraints of `{root}` could not be checked, so they are Undecided: {warning}"
            ));
        }
    }
    warnings
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

/// Gives each feature in `elements` its Computed Values, distinct and in
/// first-seen order. Values for features the Model doesn't have, and values
/// equal to the feature's own written value, are dropped.
fn record_values(
    elements: &mut BTreeMap<QualifiedName, Element>,
    values: Vec<(QualifiedName, String)>,
) {
    for (feature, text) in values {
        // A value the Model writes out already says itself.
        if let Some(element) = elements.get_mut(&feature)
            && element.value.as_deref() != Some(text.as_str())
            && !element.computed.contains(&text)
        {
            element.computed.push(text);
        }
    }
}

/// Records the verdicts of validating `root` on their constraints. A
/// verdict at instance path `a.b` is about the part `root::a::b`. When
/// validation failed, the asserted constraints owned in `root`'s tree are
/// Unchecked there, and the reason is returned.
fn record(
    elements: &mut BTreeMap<QualifiedName, Element>,
    root: &QualifiedName,
    verdicts: Result<Vec<Verdict>, String>,
) -> Option<String> {
    let verdicts = match verdicts {
        Ok(verdicts) => verdicts,
        Err(reason) => {
            let prefix = format!("{root}::");
            let owned: Vec<QualifiedName> = elements
                .values()
                .filter(|e| e.kind == ElementKind::ConstraintUsage && e.is_asserted)
                .filter(|e| {
                    e.owner
                        .as_ref()
                        .is_some_and(|o| o == root || o.as_str().starts_with(&prefix))
                })
                .map(|e| e.qualified_name.clone())
                .collect();
            for name in owned {
                let constraint = elements.get_mut(&name).expect("listed above");
                let part = constraint.owner.clone().expect("filtered on its owner");
                constraint.checks.push(ConstraintCheck {
                    part,
                    result: ClaimResult::Unchecked {
                        reason: reason.clone(),
                    },
                });
            }
            return Some(reason);
        }
    };
    for verdict in verdicts {
        let part = if verdict.path.is_empty() {
            root.clone()
        } else {
            QualifiedName::new(format!("{root}::{}", verdict.path.replace('.', "::")))
        };
        let result = if !verdict.error.is_empty() {
            ClaimResult::Undecided {
                reason: verdict.error,
            }
        } else {
            match verdict.status.as_str() {
                opensysml::STATUS_HOLDS => ClaimResult::Holds,
                opensysml::STATUS_VIOLATED => ClaimResult::Violated {
                    condition: verdict.condition,
                },
                other => ClaimResult::Undecided {
                    reason: format!("the service answered `{other}`"),
                },
            }
        };
        if let Some(constraint) = elements.get_mut(&verdict.constraint) {
            constraint.checks.push(ConstraintCheck { part, result });
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qn(s: &str) -> QualifiedName {
        QualifiedName::new(s)
    }

    /// Asserted constraints, as `(name, owner)`.
    fn constraints(list: &[(&str, &str)]) -> BTreeMap<QualifiedName, Element> {
        list.iter()
            .map(|(name, owner)| {
                let mut c = Element::new(qn(name), ElementKind::ConstraintUsage);
                c.is_asserted = true;
                c.owner = Some(qn(owner));
                (qn(name), c)
            })
            .collect()
    }

    fn verdict(constraint: &str, path: &str, status: &str) -> Verdict {
        Verdict {
            constraint: qn(constraint),
            path: path.to_owned(),
            status: status.to_owned(),
            condition: String::new(),
            error: String::new(),
        }
    }

    fn checks(
        elements: &BTreeMap<QualifiedName, Element>,
        name: &str,
    ) -> Vec<(String, ClaimResult)> {
        elements[&qn(name)]
            .checks
            .iter()
            .map(|c| (c.part.to_string(), c.result.clone()))
            .collect()
    }

    #[test]
    fn computed_values_go_to_their_features_distinct_and_in_order() {
        let mut elements: BTreeMap<QualifiedName, Element> =
            ["C::car::mass", "C::car::wheels::mass"]
                .into_iter()
                .map(|n| (qn(n), Element::new(qn(n), ElementKind::AttributeUsage)))
                .collect();

        record_values(
            &mut elements,
            vec![
                (qn("C::car::mass"), "1363.0".to_owned()),
                (qn("C::car::wheels::mass"), "22.0".to_owned()),
                (qn("C::car::wheels::mass"), "22.0".to_owned()),
                (qn("C::car::wheels::mass"), "25.0".to_owned()),
                (qn("C::car::gone"), "1.0".to_owned()),
            ],
        );

        assert_eq!(elements[&qn("C::car::mass")].computed, ["1363.0"]);
        assert_eq!(
            elements[&qn("C::car::wheels::mass")].computed,
            ["22.0", "25.0"]
        );
    }

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

    #[test]
    fn a_verdict_is_about_the_validated_part_or_the_one_at_its_path() {
        let mut elements = constraints(&[
            ("C::vehicle::light", "C::vehicle"),
            ("C::Tank::fits", "C::Tank"),
        ]);
        let fits = Verdict {
            condition: "fuelMass <= fuelMassMax".to_owned(),
            ..verdict("C::Tank::fits", "tank", "violated")
        };

        let warning = record(
            &mut elements,
            &qn("C::vehicle"),
            Ok(vec![verdict("C::vehicle::light", "", "holds"), fits]),
        );

        assert_eq!(warning, None);
        assert_eq!(
            checks(&elements, "C::vehicle::light"),
            [("C::vehicle".to_owned(), ClaimResult::Holds)]
        );
        assert_eq!(
            checks(&elements, "C::Tank::fits"),
            [(
                "C::vehicle::tank".to_owned(),
                ClaimResult::Violated {
                    condition: "fuelMass <= fuelMassMax".to_owned()
                }
            )]
        );
    }

    #[test]
    fn a_dotted_path_reaches_a_nested_part() {
        let mut elements = constraints(&[("C::Wheel::round", "C::Wheel")]);

        record(
            &mut elements,
            &qn("C::car"),
            Ok(vec![verdict("C::Wheel::round", "axle.wheel", "holds")]),
        );

        assert_eq!(
            checks(&elements, "C::Wheel::round")[0].0,
            "C::car::axle::wheel"
        );
    }

    #[test]
    fn an_evaluation_error_is_undecided_with_the_service_s_reason() {
        let mut elements = constraints(&[("C::vehicle::light", "C::vehicle")]);
        let failed = Verdict {
            error: "mass has no value".to_owned(),
            ..verdict("C::vehicle::light", "", "undecided")
        };

        record(&mut elements, &qn("C::vehicle"), Ok(vec![failed]));

        assert_eq!(
            checks(&elements, "C::vehicle::light"),
            [(
                "C::vehicle".to_owned(),
                ClaimResult::Undecided {
                    reason: "mass has no value".to_owned()
                }
            )]
        );
    }

    #[test]
    fn a_failed_validation_leaves_the_tree_s_own_constraints_unchecked_and_says_why() {
        let mut elements = constraints(&[
            ("C::vehicle::light", "C::vehicle"),
            ("C::vehicle::tank::full", "C::vehicle::tank"),
            ("C::other::x", "C::other"),
        ]);

        let warning = record(
            &mut elements,
            &qn("C::vehicle"),
            Err("validation is not supported".to_owned()),
        );

        assert_eq!(warning.as_deref(), Some("validation is not supported"));
        let unchecked = ClaimResult::Unchecked {
            reason: "validation is not supported".to_owned(),
        };
        assert_eq!(
            checks(&elements, "C::vehicle::light"),
            [("C::vehicle".to_owned(), unchecked.clone())]
        );
        assert_eq!(
            checks(&elements, "C::vehicle::tank::full"),
            [("C::vehicle::tank".to_owned(), unchecked)]
        );
        assert!(checks(&elements, "C::other::x").is_empty(), "another tree");
    }
}
