//! The `requirements` command: a Satisfaction Status report that fails CI
//! on violated requirements, driven with the fake backend.

#[path = "../../viewer-core/tests/common/mod.rs"]
mod common;

use std::ffi::OsString;

use tempfile::TempDir;
use viewer_cli::{EXIT_FAILURE, EXIT_OK, run_with_output};
use viewer_core::model::{ClaimResult, Severity};
use viewer_core::{FakeBackend, Model};

struct Outcome {
    code: u8,
    stdout: String,
    stderr: String,
}

fn report(model: Model, strict: bool) -> Outcome {
    let folder = TempDir::new().unwrap();
    let mut args: Vec<OsString> = vec![
        "sysml-viewer".into(),
        "requirements".into(),
        folder.path().into(),
    ];
    if strict {
        args.push("--strict".into());
    }
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = run_with_output(args, &mut FakeBackend::new(model), &mut stdout, &mut stderr);
    Outcome {
        code,
        stdout: String::from_utf8(stdout).unwrap(),
        stderr: String::from_utf8(stderr).unwrap(),
    }
}

#[test]
fn lists_every_requirement_usage_with_its_status_and_claims_in_name_order() {
    let outcome = report(common::requirements(), false);

    let lines: Vec<&str> = outcome.stdout.lines().collect();
    assert_eq!(lines.len(), 7, "{}", outcome.stdout);
    assert_eq!(
        lines[0],
        "○ Not claimed Req::vehicleSpec              nothing claims to satisfy it"
    );
    assert!(
        lines[1].starts_with("? Undecided   Req::vehicleSpec::carMassReq  by Req::car ? "),
        "{}",
        lines[1]
    );
    assert_eq!(
        lines[3],
        "✓ Satisfied   Req::vehicleSpec::massReq     by Req::vehicle ✓"
    );
    assert_eq!(
        lines[4],
        "✗ Violated    Req::vehicleSpec::rangeReq    by Req::vehicle ✗ v.range >= 500.0"
    );
    assert_eq!(
        lines[6],
        "6 requirements: 1 satisfied, 1 violated, 2 undecided, 2 not claimed"
    );
}

#[test]
fn a_violated_requirement_fails_the_run() {
    let outcome = report(common::requirements(), false);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(
        outcome.stderr.contains("error: 1 requirement is violated"),
        "{}",
        outcome.stderr
    );
}

/// A requirement satisfied by one part and not claimed by anything else.
fn satisfied_and_unclaimed() -> Model {
    Model::builder()
        .package("P")
        .requirement("P::met")
        .requirement("P::open")
        .part("P::a")
        .satisfy("P::@1", "P::met", "P::a")
        .result(ClaimResult::Holds)
        .build()
}

#[test]
fn undecided_and_unclaimed_requirements_pass_unless_strict() {
    let lenient = report(satisfied_and_unclaimed(), false);
    assert_eq!(lenient.code, EXIT_OK, "{}", lenient.stderr);

    let strict = report(satisfied_and_unclaimed(), true);
    assert_eq!(strict.code, EXIT_FAILURE);
    assert!(
        strict
            .stderr
            .contains("error: 1 requirement is not satisfied"),
        "{}",
        strict.stderr
    );
}

#[test]
fn every_claim_on_a_requirement_is_listed() {
    let model = Model::builder()
        .package("P")
        .requirement("P::r")
        .part("P::a")
        .part("P::b")
        .satisfy("P::@1", "P::r", "P::a")
        .result(ClaimResult::Holds)
        .satisfy("P::@2", "P::r", "P::b")
        .result(ClaimResult::Violated {
            condition: "b.x <= 1".to_owned(),
        })
        .build();

    let outcome = report(model, false);

    assert!(
        outcome
            .stdout
            .contains("✗ Violated    P::r  by P::a ✓; by P::b ✗ b.x <= 1"),
        "{}",
        outcome.stdout
    );
}

#[test]
fn a_model_without_requirements_says_so_and_passes() {
    let outcome = report(Model::builder().package("P").part("P::a").build(), true);

    assert_eq!(outcome.code, EXIT_OK);
    assert!(
        outcome.stdout.starts_with("no requirement usages in "),
        "{}",
        outcome.stdout
    );
}

#[test]
fn a_model_with_errors_fails_with_its_diagnostics() {
    let model = Model::builder()
        .package("P")
        .diagnostic(Severity::Error, "unexpected ';'")
        .build();

    let outcome = report(model, false);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(
        outcome.stderr.contains("error: unexpected ';'"),
        "{}",
        outcome.stderr
    );
    assert!(
        outcome
            .stderr
            .contains("the Model has errors; no requirement was checked"),
        "{}",
        outcome.stderr
    );
    assert!(outcome.stdout.is_empty());
}

#[test]
fn the_report_is_the_same_on_every_run() {
    assert_eq!(
        report(common::requirements(), false).stdout,
        report(common::requirements(), false).stdout
    );
}
