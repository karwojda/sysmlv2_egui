//! Checks the Model's Satisfaction Claims with the service and records
//! each claim's result.

use std::collections::BTreeMap;

use opensysml::VerifyOptions;
use viewer_core::model::{ClaimResult, Element, ElementKind, QualifiedName};

/// One verdict as the service gives it, reduced to what matching needs.
#[derive(Clone, Debug, PartialEq)]
struct Verdict {
    /// The requirement usage the claim names.
    requirement: QualifiedName,
    /// The satisfier, when the service names one.
    satisfier: Option<QualifiedName>,
    status: String,
    condition: String,
    error: String,
}

/// Asks the service to check every claim in `parsed` and records the
/// results on the claims in `elements`. A failed check marks every claim
/// Unchecked and returns the reason, for a Model warning.
pub fn check(
    parsed: &opensysml::Model,
    elements: &mut BTreeMap<QualifiedName, Element>,
) -> Option<String> {
    if !elements
        .values()
        .any(|e| e.kind == ElementKind::SatisfyRequirement)
    {
        return None;
    }
    let verdicts = parsed
        .verify_satisfaction(None, &VerifyOptions::default())
        .map(|satisfaction| {
            satisfaction
                .verdicts
                .iter()
                .map(|v| Verdict {
                    requirement: QualifiedName::new(v.requirement_id.clone()),
                    satisfier: (!v.instance_type_id.is_empty())
                        .then(|| QualifiedName::new(v.instance_type_id.clone())),
                    status: v.status.clone(),
                    condition: v.condition.clone(),
                    error: v.error.clone(),
                })
                .collect()
        })
        .map_err(|error| error.to_string());
    record(elements, verdicts)
}

/// Gives each claim its result. A verdict goes to the first claim, in
/// declaration order, on its requirement and by its satisfier (any
/// satisfier when the service names none). Claims left without a verdict,
/// or all of them when checking failed, are Unchecked.
fn record(
    elements: &mut BTreeMap<QualifiedName, Element>,
    verdicts: Result<Vec<Verdict>, String>,
) -> Option<String> {
    let mut unmatched: Vec<QualifiedName> = elements
        .values()
        .filter(|e| e.kind == ElementKind::SatisfyRequirement)
        .map(|e| e.qualified_name.clone())
        .collect();
    unmatched.sort_by_key(declaration_order);
    let verdicts = match verdicts {
        Ok(verdicts) => verdicts,
        Err(reason) => {
            for name in &unmatched {
                set(
                    elements,
                    name,
                    ClaimResult::Unchecked {
                        reason: reason.clone(),
                    },
                );
            }
            return Some(reason);
        }
    };
    for verdict in verdicts {
        let matches = |name: &QualifiedName| {
            let claim = &elements[name];
            claim.satisfies.as_ref() == Some(&verdict.requirement)
                && verdict
                    .satisfier
                    .as_ref()
                    .is_none_or(|s| claim.satisfier.as_ref() == Some(s))
        };
        let Some(index) = unmatched.iter().position(matches) else {
            continue;
        };
        let name = unmatched.remove(index);
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
        set(elements, &name, result);
    }
    for name in unmatched {
        let reason = "the service gave no verdict for it".to_owned();
        set(elements, &name, ClaimResult::Unchecked { reason });
    }
    None
}

fn set(elements: &mut BTreeMap<QualifiedName, Element>, name: &QualifiedName, result: ClaimResult) {
    if let Some(claim) = elements.get_mut(name) {
        claim.claim_result = Some(result);
    }
}

/// Generated names number claims within their owner (`Req::@8`,
/// `Req::@10`); order by that number, not as text.
fn declaration_order(name: &QualifiedName) -> (String, u64) {
    let text = name.as_str();
    match text
        .rsplit_once("::@")
        .and_then(|(owner, n)| Some((owner, n.parse::<u64>().ok()?)))
    {
        Some((owner, n)) => (owner.to_owned(), n),
        None => (text.to_owned(), 0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn qn(s: &str) -> QualifiedName {
        QualifiedName::new(s)
    }

    /// Claims as `(name, requirement, satisfier)`.
    fn claims(list: &[(&str, &str, &str)]) -> BTreeMap<QualifiedName, Element> {
        list.iter()
            .map(|(name, requirement, satisfier)| {
                let mut claim = Element::new(qn(name), ElementKind::SatisfyRequirement);
                claim.satisfies = Some(qn(requirement));
                claim.satisfier = Some(qn(satisfier));
                (qn(name), claim)
            })
            .collect()
    }

    fn verdict(requirement: &str, satisfier: &str, status: &str) -> Verdict {
        Verdict {
            requirement: qn(requirement),
            satisfier: (!satisfier.is_empty()).then(|| qn(satisfier)),
            status: status.to_owned(),
            condition: String::new(),
            error: String::new(),
        }
    }

    fn result(elements: &BTreeMap<QualifiedName, Element>, name: &str) -> ClaimResult {
        elements[&qn(name)].claim_result.clone().unwrap()
    }

    #[test]
    fn verdicts_go_to_the_claim_on_their_requirement_by_their_satisfier() {
        let mut elements = claims(&[("P::@1", "P::r", "P::a"), ("P::@2", "P::r", "P::b")]);
        let violated = Verdict {
            condition: "b.x <= 1".to_owned(),
            ..verdict("P::r", "P::b", "violated")
        };

        let warning = record(
            &mut elements,
            Ok(vec![violated, verdict("P::r", "P::a", "holds")]),
        );

        assert_eq!(warning, None);
        assert_eq!(result(&elements, "P::@1"), ClaimResult::Holds);
        assert_eq!(
            result(&elements, "P::@2"),
            ClaimResult::Violated {
                condition: "b.x <= 1".to_owned()
            }
        );
    }

    #[test]
    fn a_verdict_naming_no_satisfier_takes_the_next_claim_on_its_requirement_in_declaration_order()
    {
        let mut elements = claims(&[("P::@10", "P::r", "P::b"), ("P::@9", "P::r", "P::a")]);

        record(
            &mut elements,
            Ok(vec![
                verdict("P::r", "", "holds"),
                verdict("P::r", "", "violated"),
            ]),
        );

        assert_eq!(
            result(&elements, "P::@9"),
            ClaimResult::Holds,
            "@9 is declared before @10"
        );
        assert!(matches!(
            result(&elements, "P::@10"),
            ClaimResult::Violated { .. }
        ));
    }

    #[test]
    fn an_evaluation_error_is_undecided_with_the_service_s_reason() {
        let mut elements = claims(&[("P::@1", "P::r", "P::a")]);
        let failed = Verdict {
            error: "no value for feature v.cost".to_owned(),
            ..verdict("P::r", "P::a", "undecided")
        };

        record(&mut elements, Ok(vec![failed]));

        assert_eq!(
            result(&elements, "P::@1"),
            ClaimResult::Undecided {
                reason: "no value for feature v.cost".to_owned()
            }
        );
    }

    #[test]
    fn a_claim_the_service_says_nothing_about_is_unchecked() {
        let mut elements = claims(&[("P::@1", "P::r", "P::a"), ("P::@2", "P::s", "P::a")]);

        record(&mut elements, Ok(vec![verdict("P::r", "P::a", "holds")]));

        assert!(matches!(
            result(&elements, "P::@2"),
            ClaimResult::Unchecked { .. }
        ));
    }

    #[test]
    fn a_failed_check_marks_every_claim_unchecked_and_reports_why() {
        let mut elements = claims(&[("P::@1", "P::r", "P::a"), ("P::@2", "P::s", "P::b")]);

        let warning = record(
            &mut elements,
            Err("verification is not supported".to_owned()),
        );

        assert_eq!(warning.as_deref(), Some("verification is not supported"));
        for name in ["P::@1", "P::@2"] {
            assert_eq!(
                result(&elements, name),
                ClaimResult::Unchecked {
                    reason: "verification is not supported".to_owned()
                }
            );
        }
    }
}
