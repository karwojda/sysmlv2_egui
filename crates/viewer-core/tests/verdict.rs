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
