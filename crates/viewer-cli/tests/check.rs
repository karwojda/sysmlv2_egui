//! The `check` command: requirements, asserted constraints and analyses in one report
//! that fails CI on any violation, driven with the fake backend.

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

fn check(model: Model, strict: bool) -> Outcome {
    check_with(model, if strict { &["--strict"] } else { &[] })
}

fn check_with(model: Model, flags: &[&str]) -> Outcome {
    let folder = TempDir::new().unwrap();
    let mut args: Vec<OsString> = vec!["sysml-viewer".into(), "check".into(), folder.path().into()];
    args.extend(flags.iter().map(OsString::from));
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = run_with_output(args, &mut FakeBackend::new(model), &mut stdout, &mut stderr);
    Outcome {
        code,
        stdout: String::from_utf8(stdout).unwrap(),
        stderr: String::from_utf8(stderr).unwrap(),
    }
}

#[test]
fn lists_every_constraint_result_by_constraint_and_part() {
    let outcome = check(common::assertions(), false);

    let lines: Vec<&str> = outcome.stdout.lines().collect();
    assert!(
        lines[0].starts_with("no requirement usages in "),
        "{}",
        lines[0]
    );
    assert_eq!(
        &lines[1..5],
        [
            "✗ Violated    C::Tank::fits              on C::vehicle::tank: fuelMass <= fuelMassMax",
            "✓ Holds       C::vehicle::light          on C::vehicle",
            "✗ Violated    C::vehicle::powerToWeight  on C::vehicle: numerator / denominator <= limit",
            "3 constraint results: 1 hold, 2 violated, 0 undecided",
        ]
    );
}

#[test]
fn a_violated_constraint_fails_the_run() {
    let outcome = check(common::assertions(), false);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(
        outcome.stderr.contains("error: 2 constraints are violated"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn requirements_come_first_and_both_kinds_of_failure_are_named() {
    let outcome = check(common::requirements(), false);

    assert!(
        outcome
            .stdout
            .starts_with("✗ Violated    Req::vehicleSpec "),
        "{}",
        outcome.stdout
    );
    assert!(
        outcome.stdout.contains("no checked constraints in "),
        "{}",
        outcome.stdout
    );
    assert!(
        outcome
            .stderr
            .contains("error: 2 requirements are violated"),
        "{}",
        outcome.stderr
    );

    let both = Model::builder()
        .package("P")
        .part("P::p")
        .requirement("P::r")
        .satisfy("P::@1", "P::r", "P::p")
        .result(ClaimResult::Violated {
            condition: "p.x <= 1".to_owned(),
        })
        .constraint("P::p::c")
        .asserted()
        .checked_on(
            "P::p",
            ClaimResult::Violated {
                condition: "x <= 2".to_owned(),
            },
        )
        .build();
    let outcome = check(both, false);
    assert!(
        outcome
            .stderr
            .contains("error: 1 requirement is violated; 1 constraint is violated"),
        "{}",
        outcome.stderr
    );
}

/// A part whose one asserted constraint couldn't be decided.
fn undecided() -> Model {
    Model::builder()
        .package("P")
        .part("P::p")
        .constraint("P::p::c")
        .asserted()
        .checked_on(
            "P::p",
            ClaimResult::Undecided {
                reason: "x has no value".to_owned(),
            },
        )
        .build()
}

#[test]
fn an_undecided_constraint_passes_unless_strict() {
    let lenient = check(undecided(), false);
    assert_eq!(lenient.code, EXIT_OK, "{}", lenient.stderr);
    assert!(
        lenient
            .stdout
            .contains("? Undecided   P::p::c  on P::p: x has no value"),
        "{}",
        lenient.stdout
    );

    let strict = check(undecided(), true);
    assert_eq!(strict.code, EXIT_FAILURE);
    assert!(
        strict.stderr.contains("error: 1 constraint is not holding"),
        "{}",
        strict.stderr
    );
}

#[test]
fn a_model_with_nothing_to_check_says_so_and_passes() {
    let outcome = check(Model::builder().package("P").part("P::p").build(), true);

    assert_eq!(outcome.code, EXIT_OK);
    assert!(outcome.stdout.contains("no requirement usages in "));
    assert!(outcome.stdout.contains("no checked constraints in "));
    assert!(outcome.stdout.contains("no analysis usages in "));
}

#[test]
fn a_model_with_errors_fails_saying_nothing_was_checked() {
    let model = Model::builder()
        .package("P")
        .diagnostic(Severity::Error, "unexpected ';'")
        .build();

    let outcome = check(model, false);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(
        outcome.stderr.contains("error: unexpected ';'"),
        "{}",
        outcome.stderr
    );
    assert!(
        outcome
            .stderr
            .contains("the Model has errors; nothing was checked"),
        "{}",
        outcome.stderr
    );
    assert!(outcome.stdout.is_empty());
}

/// Analyses of `P::D` (objective `x <= 1`): one holding, one violated, one
/// undecided and one not run.
fn analyses() -> Model {
    Model::builder()
        .package("P")
        .analysis_def("P::D")
        .objective(None, &["x <= 1"])
        .analysis("P::good")
        .typed_by("P::D")
        .analysed(&[("x", "1.0")], Some(ClaimResult::Holds))
        .analysis("P::bad")
        .typed_by("P::D")
        .analysed(
            &[("x", "2.0")],
            Some(ClaimResult::Violated {
                condition: "x <= 1".to_owned(),
            }),
        )
        .analysis("P::odd")
        .typed_by("P::D")
        .analysed(
            &[],
            Some(ClaimResult::Undecided {
                reason: "x has no value".to_owned(),
            }),
        )
        .analysis("P::idle")
        .typed_by("P::D")
        .build()
}

#[test]
fn lists_every_analysis_usage_with_its_outputs_and_objective() {
    let outcome = check(analyses(), false);

    let lines: Vec<&str> = outcome.stdout.lines().collect();
    assert_eq!(
        &lines[2..],
        [
            "✗ Violated    P::bad   x = 2.0: x <= 1",
            "✓ Holds       P::good  x = 1.0",
            "○ Not run     P::idle",
            "? Undecided   P::odd   x has no value",
            "4 analyses: 1 hold, 1 violated, 1 undecided, 1 not run",
        ],
        "{}",
        outcome.stdout
    );
}

#[test]
fn the_analysis_twin_holds() {
    let outcome = check(common::analysis(), true);

    assert!(
        outcome
            .stdout
            .contains("✓ Holds       A::carMass  result = 230.0"),
        "{}",
        outcome.stdout
    );
    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
}

#[test]
fn a_violated_analysis_fails_and_strict_also_fails_on_undecided_and_not_run() {
    let lenient = check(analyses(), false);
    assert_eq!(lenient.code, EXIT_FAILURE);
    assert!(
        lenient.stderr.contains("error: 1 analysis is violated"),
        "{}",
        lenient.stderr
    );

    let strict = check(analyses(), true);
    assert!(
        strict.stderr.contains("error: 3 analyses are not holding"),
        "{}",
        strict.stderr
    );
}

#[test]
fn no_analyses_leaves_every_analysis_not_run() {
    let outcome = check_with(common::analysis(), &["--no-analyses"]);

    assert!(
        outcome.stdout.contains("○ Not run     A::carMass"),
        "{}",
        outcome.stdout
    );
    assert_eq!(outcome.code, EXIT_OK, "not run passes unless strict");

    let strict = check_with(common::analysis(), &["--no-analyses", "--strict"]);
    assert_eq!(strict.code, EXIT_FAILURE);
}

#[test]
fn analyses_overrides_the_setting_and_the_two_flags_exclude_each_other() {
    let outcome = check_with(common::analysis(), &["--analyses"]);
    assert!(outcome.stdout.contains("✓ Holds       A::carMass"));

    let both = check_with(common::analysis(), &["--analyses", "--no-analyses"]);
    assert_ne!(both.code, EXIT_OK);
    assert!(
        both.stderr.contains("cannot be used with"),
        "{}",
        both.stderr
    );
}
