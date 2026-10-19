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
    if !elements
        .values()
        .any(|e| e.kind == ElementKind::ConstraintUsage && e.is_asserted)
    {
        return Vec::new();
    }
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
        let verdicts = parsed
            .validate_instance(root.as_str(), None)
            .map(|validation| {
                validation
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
                    .collect()
            })
            .map_err(|error| error.to_string());
        if let Some(warning) = record(elements, &root, verdicts) {
            warnings.push(format!(
                "constraints of `{root}` could not be checked, so they are Undecided: {warning}"
            ));
        }
    }
    warnings
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
