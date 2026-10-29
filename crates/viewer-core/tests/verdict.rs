//! Seam 1: Verdicts, one scale for Satisfaction Status, Constraint Results
//! and Analysis Results, each read in its own words.

mod common;

use viewer_core::model::{ClaimResult, Model};
use viewer_core::verdict::{self, Finding, Kind, Status, Verdict};
use viewer_core::{QualifiedName, style};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

fn verdict_of(model: &Model, name: &str) -> Option<Verdict> {
    verdict::of(model, model.element(&qn(name)).unwrap())
}

fn violated(condition: &str) -> ClaimResult {
    ClaimResult::Violated {
        condition: condition.to_owned(),
    }
}

#[test]
fn each_kind_reads_the_scale_in_its_own_words() {
    let words = |kind: Kind| Status::ALL.map(|s| kind.label(s));

    assert_eq!(
        words(Kind::Requirement),
        ["Satisfied", "Violated", "Undecided", "Not claimed"]
    );
    assert_eq!(
        words(Kind::Constraint),
        ["Holds", "Violated", "Undecided", "Not checked"]
    );
    assert_eq!(
        words(Kind::Analysis),
        ["Holds", "Violated", "Undecided", "Not run"]
    );
    assert_eq!(Status::ALL.map(Status::mark), ["✓", "✗", "?", "○"]);
}

#[test]
fn the_worst_decides_violated_then_undecided_then_absent_then_holds() {
    use Status::*;
    assert_eq!(verdict::worst([Holds, Absent]), Some(Absent));
    assert_eq!(verdict::worst([Absent, Undecided, Holds]), Some(Undecided));
    assert_eq!(verdict::worst([Undecided, Violated]), Some(Violated));
    assert_eq!(verdict::worst([]), None);
}

#[test]
fn a_requirement_s_findings_are_its_claims() {
    let model = common::requirements();

    let range = verdict_of(&model, "Req::vehicleSpec::rangeReq").unwrap();

    assert_eq!(range.kind, Kind::Requirement);
    assert_eq!(range.label(), "Violated");
    assert_eq!(
        range.findings,
        [Finding {
            about: Some(qn("Req::vehicle")),
            status: Status::Violated,
            detail: Some("v.range >= 500.0".to_owned()),
        }]
    );
    let spare = verdict_of(&model, "Req::vehicleSpec::spareReq").unwrap();
    assert_eq!(
        (spare.status, spare.label()),
        (Status::Absent, "Not claimed")
    );
}

#[test]
fn an_asserted_constraint_s_findings_are_its_parts() {
    let model = common::assertions();

    let fits = verdict_of(&model, "C::Tank::fits").unwrap();

    assert_eq!(fits.kind, Kind::Constraint);
    assert_eq!(fits.status, Status::Violated);
    assert_eq!(
        fits.findings
            .iter()
            .map(|f| f.about.clone())
            .collect::<Vec<_>>(),
        [Some(qn("C::vehicle::tank"))]
    );
    assert_eq!(
        verdict_of(&model, "C::vehicle::heavy"),
        None,
        "an unasserted constraint has nothing to judge"
    );
}

#[test]
fn an_asserted_constraint_nobody_checked_is_not_checked_and_a_failed_check_undecided() {
    let model = Model::builder()
        .package("P")
        .part_def("P::D")
        .constraint("P::D::c")
        .asserted()
        .part("P::p")
        .constraint("P::p::c")
        .asserted()
        .checked_on(
            "P::p",
            ClaimResult::Unchecked {
                reason: "service unavailable".to_owned(),
            },
        )
        .build();

    let never = verdict_of(&model, "P::D::c").unwrap();
    assert_eq!(
        (never.status, never.label()),
        (Status::Absent, "Not checked")
    );
    assert!(never.findings.is_empty());
    let failed = verdict_of(&model, "P::p::c").unwrap();
    assert_eq!(failed.status, Status::Undecided);
    assert_eq!(
        failed.findings[0].detail.as_deref(),
        Some("service unavailable")
    );
}

#[test]
fn an_analysis_s_finding_is_its_objective_and_not_run_is_absent() {
    let model = Model::builder()
        .package("P")
        .analysis_def("P::D")
        .objective(None, &["x <= 1"])
        .analysis_def("P::Free")
        .analysis("P::bad")
        .typed_by("P::D")
        .analysed(&[("x", "2.0")], Some(violated("x <= 1")))
        .analysis("P::idle")
        .typed_by("P::D")
        .analysis("P::free")
        .typed_by("P::Free")
        .analysed(&[("x", "2.0")], None)
        .build();

    let bad = verdict_of(&model, "P::bad").unwrap();
    assert_eq!(
        bad.findings,
        [Finding {
            about: None,
            status: Status::Violated,
            detail: Some("x <= 1".to_owned()),
        }]
    );
    let idle = verdict_of(&model, "P::idle").unwrap();
    assert_eq!((idle.status, idle.label()), (Status::Absent, "Not run"));
    assert_eq!(
        verdict_of(&model, "P::free"),
        None,
        "no objective, nothing to judge"
    );
}

#[test]
fn every_element_with_a_verdict_and_nothing_else() {
    let model = common::analysis();

    let names: Vec<String> = verdict::all(&model)
        .map(|(e, _)| e.qualified_name.to_string())
        .collect();

    assert_eq!(names, ["A::carMass"]);
}

#[test]
fn style_files_name_each_kind_s_absent_value_in_its_own_words() {
    let text = r##"{
  "format_version": 1,
  "rules": [
    { "satisfaction": "not-claimed", "fill": "#eeeeee" },
    { "constraint": "not-checked", "fill": "#eeeeee" },
    { "analysis": "not-run", "fill": "#eeeeee" }
  ]
}"##;
    let folder = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(folder.path().join("diagrams")).unwrap();
    std::fs::write(style::path_for(folder.path()), text).unwrap();

    let loaded = style::load(folder.path()).unwrap().unwrap();

    let conditions: Vec<String> = loaded
        .rules
        .iter()
        .map(|r| r.condition.describe())
        .collect();
    assert_eq!(
        conditions,
        ["Not claimed", "Constraint not checked", "Analysis not run"]
    );
    let written = loaded.to_text();
    assert!(
        written.contains(r#""constraint": "not-checked""#),
        "{written}"
    );
    std::fs::write(
        style::path_for(folder.path()),
        r##"{"format_version": 1, "rules": [{"constraint": "not-run", "fill": "#eeeeee"}]}"##,
    )
    .unwrap();
    assert!(
        style::load(folder.path()).is_err(),
        "a kind's words only: a constraint is never 'not run'"
    );
}

/// Satisfaction Status: derived from claim results, and rolled up.
mod requirements {
    use super::*;
    use viewer_core::model::ModelBuilder;

    /// The Satisfaction Status of `name`: its Verdict, if it has one.
    fn status(model: &Model, name: &QualifiedName) -> Option<Status> {
        verdict::of(model, model.element(name)?).map(|v| v.status)
    }

    fn undecided(reason: &str) -> ClaimResult {
        ClaimResult::Undecided {
            reason: reason.to_owned(),
        }
    }

    /// A requirement `P::r`, two parts, and the claims `add` puts in.
    fn model(add: impl FnOnce(ModelBuilder) -> ModelBuilder) -> Model {
        let builder = Model::builder()
            .package("P")
            .requirement_def("P::R")
            .requirement("P::r")
            .typed_by("P::R")
            .part("P::a")
            .part("P::b");
        add(builder).build()
    }

    fn status_of(model: &Model) -> Option<Status> {
        status(model, &qn("P::r"))
    }

    #[test]
    fn a_requirement_nothing_claims_is_not_claimed() {
        assert_eq!(status_of(&model(|b| b)), Some(Status::Absent));
    }

    #[test]
    fn a_requirement_whose_claims_all_hold_is_satisfied() {
        let m = model(|b| {
            b.satisfy("P::@1", "P::r", "P::a")
                .result(ClaimResult::Holds)
                .satisfy("P::@2", "P::r", "P::b")
                .result(ClaimResult::Holds)
        });

        assert_eq!(status_of(&m), Some(Status::Holds));
    }

    #[test]
    fn one_violated_claim_among_holding_ones_makes_it_violated() {
        let m = model(|b| {
            b.satisfy("P::@1", "P::r", "P::a")
                .result(ClaimResult::Holds)
                .satisfy("P::@2", "P::r", "P::b")
                .result(violated("b.x <= 1"))
        });

        assert_eq!(status_of(&m), Some(Status::Violated));
    }

    #[test]
    fn violated_wins_over_undecided_and_undecided_over_satisfied() {
        let violated_and_undecided = model(|b| {
            b.satisfy("P::@1", "P::r", "P::a")
                .result(undecided("no value"))
                .satisfy("P::@2", "P::r", "P::b")
                .result(violated("b.x <= 1"))
        });
        assert_eq!(status_of(&violated_and_undecided), Some(Status::Violated));

        let undecided_and_holding = model(|b| {
            b.satisfy("P::@1", "P::r", "P::a")
                .result(ClaimResult::Holds)
                .satisfy("P::@2", "P::r", "P::b")
                .result(undecided("no value"))
        });
        assert_eq!(status_of(&undecided_and_holding), Some(Status::Undecided));
    }

    #[test]
    fn an_unchecked_or_not_yet_checked_claim_counts_as_undecided() {
        let unchecked = model(|b| {
            b.satisfy("P::@1", "P::r", "P::a")
                .result(ClaimResult::Unchecked {
                    reason: "service down".to_owned(),
                })
        });
        assert_eq!(status_of(&unchecked), Some(Status::Undecided));

        let never = model(|b| b.satisfy("P::@1", "P::r", "P::a"));
        assert_eq!(status_of(&never), Some(Status::Undecided));
    }

    #[test]
    fn only_requirement_usages_have_a_status() {
        let m = model(|b| b);

        assert_eq!(
            status(&m, &qn("P::R")),
            None,
            "a definition can't be satisfied"
        );
        assert_eq!(status(&m, &qn("P::a")), None);
    }

    #[test]
    fn the_fixture_covers_every_status() {
        let m = common::requirements();
        let of = |name: &str| status(&m, &qn(&format!("Req::vehicleSpec::{name}"))).unwrap();

        assert_eq!(of("massReq"), Status::Holds);
        assert_eq!(of("rangeReq"), Status::Violated);
        assert_eq!(of("costReq"), Status::Undecided);
        assert_eq!(of("carMassReq"), Status::Holds);
        assert_eq!(of("spareReq"), Status::Absent);
    }

    mod roll_up {
        use super::*;

        /// `P::spec` owning `P::spec::a` and `P::spec::b`, plus whatever `add`
        /// puts in (claims, or a constraint on the spec).
        fn spec(add: impl FnOnce(ModelBuilder) -> ModelBuilder) -> Model {
            let builder = Model::builder()
                .package("P")
                .part("P::x")
                .requirement("P::spec")
                .requirement("P::spec::a")
                .requires("x.m <= 1")
                .requirement("P::spec::b")
                .requires("x.m <= 2");
            add(builder).build()
        }

        fn of(model: &Model) -> Status {
            status(model, &qn("P::spec")).unwrap()
        }

        fn holds(builder: ModelBuilder, name: &str, requirement: &str) -> ModelBuilder {
            builder
                .satisfy(name, requirement, "P::x")
                .result(ClaimResult::Holds)
        }

        #[test]
        fn a_composite_is_satisfied_when_every_child_is() {
            let m = spec(|b| holds(holds(b, "P::@1", "P::spec::a"), "P::@2", "P::spec::b"));

            assert_eq!(of(&m), Status::Holds);
        }

        #[test]
        fn an_unclaimed_child_leaves_the_composite_not_claimed() {
            let m = spec(|b| holds(b, "P::@1", "P::spec::a"));

            assert_eq!(of(&m), Status::Absent);
        }

        #[test]
        fn violated_beats_undecided_beats_not_claimed_among_children() {
            let one_undecided = spec(|b| {
                b.satisfy("P::@1", "P::spec::a", "P::x")
                    .result(undecided("no value"))
            });
            assert_eq!(
                of(&one_undecided),
                Status::Undecided,
                "b is unclaimed, a undecided"
            );

            let one_violated = spec(|b| {
                b.satisfy("P::@1", "P::spec::a", "P::x")
                    .result(undecided("no value"))
                    .satisfy("P::@2", "P::spec::b", "P::x")
                    .result(violated("x.m <= 2"))
            });
            assert_eq!(of(&one_violated), Status::Violated);
        }

        #[test]
        fn a_claim_on_a_composite_with_nothing_to_check_is_ignored() {
            let m = spec(|b| {
                let b = holds(holds(b, "P::@1", "P::spec::a"), "P::@2", "P::spec::b");
                b.satisfy("P::@3", "P::spec", "P::x")
                    .result(undecided("no condition to evaluate"))
            });

            assert_eq!(of(&m), Status::Holds);
        }

        #[test]
        fn a_claim_on_a_composite_with_its_own_constraint_counts_too() {
            let m = Model::builder()
                .package("P")
                .part("P::x")
                .requirement("P::spec")
                .requires("x.total <= 10")
                .requirement("P::spec::a")
                .requires("x.m <= 1")
                .satisfy("P::@1", "P::spec::a", "P::x")
                .result(ClaimResult::Holds)
                .satisfy("P::@2", "P::spec", "P::x")
                .result(violated("x.total <= 10"))
                .build();

            assert_eq!(of(&m), Status::Violated);
        }

        #[test]
        fn composites_roll_up_through_every_level() {
            let m = Model::builder()
                .package("P")
                .part("P::x")
                .requirement("P::top")
                .requirement("P::top::mid")
                .requirement("P::top::mid::leaf")
                .requires("x.m <= 1")
                .satisfy("P::@1", "P::top::mid::leaf", "P::x")
                .result(violated("x.m <= 1"))
                .build();

            assert_eq!(status(&m, &qn("P::top")), Some(Status::Violated));
        }
    }
}
