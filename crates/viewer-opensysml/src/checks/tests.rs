//! The checks, across `run`, with a test `Checker` answering from tables.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::time::Duration;

use tempfile::TempDir;
use viewer_core::backend::Analyses;
use viewer_core::model::{ClaimResult, Element, ElementKind, QualifiedName};
use viewer_core::settings::{self, Settings};

use super::*;

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

fn violated(condition: &str) -> ClaimResult {
    ClaimResult::Violated {
        condition: condition.to_owned(),
    }
}

fn unchecked(reason: &str) -> ClaimResult {
    ClaimResult::Unchecked {
        reason: reason.to_owned(),
    }
}

/// Answers from tables, and remembers what it was asked.
#[derive(Default)]
struct Answers {
    claims: Option<Result<Vec<ClaimOutcome>, String>>,
    parts: BTreeMap<QualifiedName, Result<PartCheck, String>>,
    analyses: BTreeMap<QualifiedName, Result<Run, String>>,
    /// How long each analysis run claims to take.
    run_time: Duration,
    asked: RefCell<Vec<String>>,
}

impl Checker for Answers {
    fn verify_claims(&self) -> Result<Vec<ClaimOutcome>, String> {
        self.asked.borrow_mut().push("claims".to_owned());
        self.claims.clone().unwrap_or(Ok(Vec::new()))
    }

    fn check_part(&self, part: &QualifiedName) -> Result<PartCheck, String> {
        self.asked.borrow_mut().push(format!("part {part}"));
        self.parts
            .get(part)
            .cloned()
            .unwrap_or(Ok(PartCheck::default()))
    }

    fn run_analysis(&self, analysis: &QualifiedName) -> (Result<Run, String>, Duration) {
        self.asked.borrow_mut().push(format!("analysis {analysis}"));
        let run = self.analyses.get(analysis).cloned().unwrap_or(Ok(Run {
            outputs: Vec::new(),
            objective: None,
        }));
        (run, self.run_time)
    }
}

/// A Model's elements, built up one at a time.
#[derive(Default)]
struct Elements(BTreeMap<QualifiedName, Element>);

impl Elements {
    fn add(mut self, name: &str, kind: ElementKind, f: impl FnOnce(&mut Element)) -> Self {
        let mut element = Element::new(qn(name), kind);
        element.owner = qn(name).parent();
        f(&mut element);
        self.0.insert(qn(name), element);
        self
    }

    fn package(self, name: &str) -> Self {
        self.add(name, ElementKind::Package, |_| {})
    }

    fn part(self, name: &str) -> Self {
        self.add(name, ElementKind::PartUsage, |_| {})
    }

    fn claim(self, name: &str, requirement: &str, satisfier: &str) -> Self {
        self.add(name, ElementKind::SatisfyRequirement, |c| {
            c.satisfies = Some(qn(requirement));
            c.satisfier = Some(qn(satisfier));
        })
    }

    fn asserted(self, name: &str) -> Self {
        self.add(name, ElementKind::ConstraintUsage, |c| c.is_asserted = true)
    }

    fn analysis(self, name: &str) -> Self {
        self.add(name, ElementKind::AnalysisUsage, |_| {})
    }
}

/// What checking found: the elements afterwards, the warnings, and what
/// the service was asked.
struct Checked {
    elements: BTreeMap<QualifiedName, Element>,
    warnings: Vec<String>,
    asked: Vec<String>,
}

impl Checked {
    fn claim(&self, name: &str) -> ClaimResult {
        self.elements[&qn(name)].claim_result.clone().unwrap()
    }

    fn checks(&self, name: &str) -> Vec<(String, ClaimResult)> {
        self.elements[&qn(name)]
            .checks
            .iter()
            .map(|c| (c.part.to_string(), c.result.clone()))
            .collect()
    }

    fn analysis(&self, name: &str) -> Option<viewer_core::model::AnalysisResult> {
        self.elements[&qn(name)].analysis_result.clone()
    }
}

fn check_in(folder: &Path, elements: Elements, answers: Answers, analyses: &Analyses) -> Checked {
    let mut elements = elements.0;
    let warnings = run(&answers, &mut elements, folder, analyses)
        .into_iter()
        .map(|d| d.message)
        .collect();
    Checked {
        elements,
        warnings,
        asked: answers.asked.into_inner(),
    }
}

fn check(elements: Elements, answers: Answers) -> Checked {
    let folder = TempDir::new().unwrap();
    check_in(folder.path(), elements, answers, &Analyses::AsSet)
}

fn outcome(requirement: &str, satisfier: &str, result: ClaimResult) -> ClaimOutcome {
    ClaimOutcome {
        requirement: qn(requirement),
        satisfier: (!satisfier.is_empty()).then(|| qn(satisfier)),
        result,
    }
}

mod claims {
    use super::*;

    fn two_claims() -> Elements {
        Elements::default()
            .package("P")
            .claim("P::@1", "P::r", "P::a")
            .claim("P::@2", "P::r", "P::b")
    }

    #[test]
    fn outcomes_go_to_the_claim_on_their_requirement_by_their_satisfier() {
        let checked = check(
            two_claims(),
            Answers {
                claims: Some(Ok(vec![
                    outcome("P::r", "P::b", violated("b.x <= 1")),
                    outcome("P::r", "P::a", ClaimResult::Holds),
                ])),
                ..Answers::default()
            },
        );

        assert!(checked.warnings.is_empty(), "{:?}", checked.warnings);
        assert_eq!(checked.claim("P::@1"), ClaimResult::Holds);
        assert_eq!(checked.claim("P::@2"), violated("b.x <= 1"));
    }

    #[test]
    fn an_outcome_naming_no_satisfier_takes_the_next_claim_in_declaration_order() {
        let elements = Elements::default()
            .package("P")
            .claim("P::@10", "P::r", "P::b")
            .claim("P::@9", "P::r", "P::a");

        let checked = check(
            elements,
            Answers {
                claims: Some(Ok(vec![
                    outcome("P::r", "", ClaimResult::Holds),
                    outcome("P::r", "", violated("x")),
                ])),
                ..Answers::default()
            },
        );

        assert_eq!(
            checked.claim("P::@9"),
            ClaimResult::Holds,
            "@9 is declared before @10"
        );
        assert_eq!(checked.claim("P::@10"), violated("x"));
    }

    #[test]
    fn a_claim_the_service_says_nothing_about_is_unchecked() {
        let checked = check(
            two_claims(),
            Answers {
                claims: Some(Ok(vec![outcome("P::r", "P::a", ClaimResult::Holds)])),
                ..Answers::default()
            },
        );

        assert_eq!(
            checked.claim("P::@2"),
            unchecked("the service gave no result for it")
        );
    }

    #[test]
    fn a_failed_verify_leaves_every_claim_unchecked_and_warns() {
        let checked = check(
            two_claims(),
            Answers {
                claims: Some(Err("verification is not supported".to_owned())),
                ..Answers::default()
            },
        );

        for name in ["P::@1", "P::@2"] {
            assert_eq!(
                checked.claim(name),
                unchecked("verification is not supported")
            );
        }
        assert_eq!(
            checked.warnings,
            [
                "requirements could not be checked, so their status is Undecided: verification is not supported"
            ]
        );
    }

    #[test]
    fn a_model_without_claims_is_not_verified() {
        let checked = check(Elements::default().package("P"), Answers::default());

        assert!(!checked.asked.contains(&"claims".to_owned()));
    }
}

mod parts {
    use super::*;

    fn vehicle() -> Elements {
        Elements::default()
            .package("C")
            .part("C::vehicle")
            .part("C::vehicle::tank")
            .asserted("C::vehicle::light")
            .asserted("C::vehicle::tank::full")
            .add("C::Tank", ElementKind::PartDefinition, |_| {})
            .asserted("C::Tank::fits")
            .part("C::other")
            .asserted("C::other::x")
    }

    fn found(constraints: &[(&str, &str, ClaimResult)]) -> Result<PartCheck, String> {
        Ok(PartCheck {
            constraints: constraints
                .iter()
                .map(|(constraint, path, result)| ConstraintOutcome {
                    constraint: qn(constraint),
                    path: (*path).to_owned(),
                    result: result.clone(),
                })
                .collect(),
            values: Vec::new(),
        })
    }

    #[test]
    fn every_top_level_part_is_checked_and_nothing_nested() {
        let checked = check(vehicle(), Answers::default());

        assert_eq!(checked.asked, ["part C::other", "part C::vehicle"]);
    }

    #[test]
    fn a_result_is_about_the_checked_part_or_the_one_at_its_path() {
        let answers = Answers {
            parts: [(
                qn("C::vehicle"),
                found(&[
                    ("C::vehicle::light", "", ClaimResult::Holds),
                    ("C::Tank::fits", "tank", violated("fuelMass <= fuelMassMax")),
                    ("C::Tank::fits", "spare.tank", ClaimResult::Holds),
                ]),
            )]
            .into(),
            ..Answers::default()
        };

        let checked = check(vehicle(), answers);

        assert_eq!(
            checked.checks("C::vehicle::light"),
            [("C::vehicle".to_owned(), ClaimResult::Holds)]
        );
        assert_eq!(
            checked.checks("C::Tank::fits"),
            [
                (
                    "C::vehicle::tank".to_owned(),
                    violated("fuelMass <= fuelMassMax")
                ),
                ("C::vehicle::spare::tank".to_owned(), ClaimResult::Holds),
            ],
            "an inherited constraint has a result on each part, a dotted path reaches a nested one"
        );
    }

    #[test]
    fn a_part_that_cannot_be_checked_leaves_its_tree_s_constraints_unchecked_and_warns() {
        let answers = Answers {
            parts: [(qn("C::vehicle"), Err("validation failed".to_owned()))].into(),
            ..Answers::default()
        };

        let checked = check(vehicle(), answers);

        assert_eq!(
            checked.checks("C::vehicle::light"),
            [("C::vehicle".to_owned(), unchecked("validation failed"))]
        );
        assert_eq!(
            checked.checks("C::vehicle::tank::full"),
            [(
                "C::vehicle::tank".to_owned(),
                unchecked("validation failed")
            )]
        );
        assert!(checked.checks("C::other::x").is_empty(), "another tree");
        assert_eq!(
            checked.warnings,
            [
                "constraints of `C::vehicle` could not be checked, so they are Undecided: validation failed"
            ]
        );
    }

    #[test]
    fn computed_values_go_to_their_features_distinct_and_unless_written_out() {
        let elements = Elements::default()
            .package("C")
            .part("C::car")
            .add("C::car::mass", ElementKind::AttributeUsage, |_| {})
            .add("C::car::wheels", ElementKind::PartUsage, |_| {})
            .add("C::car::wheels::mass", ElementKind::AttributeUsage, |a| {
                a.value = Some("22.0".to_owned())
            })
            .add("C::car::wheels::grip", ElementKind::AttributeUsage, |_| {});
        let value = |feature: &str, text: &str| (qn(feature), text.to_owned());
        let answers = Answers {
            parts: [(
                qn("C::car"),
                Ok(PartCheck {
                    constraints: Vec::new(),
                    values: vec![
                        value("C::car::mass", "1363.0"),
                        value("C::car::wheels::mass", "22.0"),
                        value("C::car::wheels::grip", "0.8"),
                        value("C::car::wheels::grip", "0.8"),
                        value("C::car::wheels::grip", "0.7"),
                        value("C::car::gone", "1.0"),
                    ],
                }),
            )]
            .into(),
            ..Answers::default()
        };

        let checked = check(elements, answers);

        let computed = |name: &str| checked.elements[&qn(name)].computed.clone();
        assert_eq!(computed("C::car::mass"), ["1363.0"]);
        assert!(
            computed("C::car::wheels::mass").is_empty(),
            "already written out"
        );
        assert_eq!(computed("C::car::wheels::grip"), ["0.8", "0.7"]);
    }
}

mod analyses {
    use super::*;

    fn two() -> Elements {
        Elements::default()
            .package("A")
            .analysis("A::first")
            .analysis("A::second")
    }

    fn ran(checked: &Checked) -> Vec<&str> {
        checked
            .asked
            .iter()
            .filter_map(|a| a.strip_prefix("analysis "))
            .collect()
    }

    fn with_setting(run_analyses: bool) -> TempDir {
        let folder = TempDir::new().unwrap();
        Settings {
            run_analyses,
            ..Settings::default()
        }
        .save(folder.path())
        .unwrap();
        folder
    }

    #[test]
    fn outputs_and_the_objective_are_recorded() {
        let answers = Answers {
            analyses: [(
                qn("A::first"),
                Ok(Run {
                    outputs: vec![("result".to_owned(), "230.0".to_owned())],
                    objective: Some(violated("result <= limit")),
                }),
            )]
            .into(),
            ..Answers::default()
        };

        let checked = check(two(), answers);

        let result = checked.analysis("A::first").unwrap();
        assert_eq!(result.outputs, [("result".to_owned(), "230.0".to_owned())]);
        assert_eq!(result.objective, Some(violated("result <= limit")));
    }

    #[test]
    fn a_failed_run_is_unchecked_and_warns() {
        let answers = Answers {
            analyses: [(qn("A::first"), Err("analysis failed".to_owned()))].into(),
            ..Answers::default()
        };

        let checked = check(two(), answers);

        assert_eq!(
            checked.analysis("A::first").unwrap().objective,
            Some(unchecked("analysis failed"))
        );
        assert_eq!(
            checked.warnings,
            ["analysis `A::first` could not be run, so it is Undecided: analysis failed"]
        );
    }

    #[test]
    fn the_setting_decides_unless_the_caller_does() {
        let on = with_setting(true);
        let off = with_setting(false);
        let run = |folder: &TempDir, asked: Analyses| {
            check_in(folder.path(), two(), Answers::default(), &asked)
        };

        assert_eq!(ran(&run(&on, Analyses::AsSet)), ["A::first", "A::second"]);
        let skipped = run(&off, Analyses::AsSet);
        assert!(ran(&skipped).is_empty());
        assert_eq!(skipped.analysis("A::first"), None, "not run");
        assert_eq!(ran(&run(&off, Analyses::All)), ["A::first", "A::second"]);
        assert!(ran(&run(&on, Analyses::None)).is_empty());
        assert_eq!(
            ran(&run(&off, Analyses::Only(qn("A::second")))),
            ["A::second"]
        );
    }

    #[test]
    fn a_broken_setting_warns_and_runs_every_analysis() {
        let folder = TempDir::new().unwrap();
        std::fs::create_dir_all(folder.path().join("diagrams")).unwrap();
        std::fs::write(settings::path_for(folder.path()), "{ not json").unwrap();

        let checked = check_in(folder.path(), two(), Answers::default(), &Analyses::AsSet);

        assert_eq!(ran(&checked), ["A::first", "A::second"]);
        assert_eq!(checked.warnings.len(), 1, "{:?}", checked.warnings);
        assert!(
            checked.warnings[0].ends_with("; using the default settings"),
            "{}",
            checked.warnings[0]
        );
    }

    #[test]
    fn slow_analyses_point_at_the_switch() {
        let answers = Answers {
            run_time: Duration::from_millis(1600),
            ..Answers::default()
        };

        let checked = check(two(), answers);

        assert_eq!(
            checked.warnings,
            [
                "analyses took 3.2 s; to load faster, untick \"Run analyses on load\" (run_analyses in diagrams/settings.json)"
            ]
        );
        let quick = check(two(), Answers::default());
        assert!(quick.warnings.is_empty());
    }
}

#[test]
fn claims_then_analyses_then_parts() {
    let elements = Elements::default()
        .package("P")
        .part("P::p")
        .claim("P::@1", "P::r", "P::p")
        .analysis("P::a");

    let checked = check(elements, Answers::default());

    assert_eq!(checked.asked, ["claims", "analysis P::a", "part P::p"]);
}
