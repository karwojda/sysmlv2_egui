//! Design Values in `check` and `requirements`: a section listing each
//! linked feature first, and failing on any Tool Link that couldn't be
//! read, driven with the fake backend.

#[path = "../../viewer-core/tests/common/mod.rs"]
mod common;

use std::ffi::OsString;

use tempfile::TempDir;
use viewer_cli::{EXIT_FAILURE, EXIT_OK, run_with_output};
use viewer_core::engineering_tools::DesignValue;
use viewer_core::model::ClaimResult;
use viewer_core::{FakeBackend, Model};

struct Outcome {
    code: u8,
    stdout: String,
    stderr: String,
}

fn run(command: &str, model: Model, flags: &[&str]) -> Outcome {
    let folder = TempDir::new().unwrap();
    let mut args: Vec<OsString> = vec!["sysml-viewer".into(), command.into(), folder.path().into()];
    args.extend(flags.iter().map(OsString::from));
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = run_with_output(args, &mut FakeBackend::new(model), &mut stdout, &mut stderr);
    Outcome {
        code,
        stdout: String::from_utf8(stdout).unwrap(),
        stderr: String::from_utf8(stderr).unwrap(),
    }
}

fn read(value: &str) -> DesignValue {
    DesignValue::Read {
        value: value.to_owned(),
    }
}

fn missing(reason: &str) -> DesignValue {
    DesignValue::Missing {
        reason: reason.to_owned(),
    }
}

/// The drive whose engine FreeCAD reads at 1.0e6 mm³ where the Model says
/// 0.9e6, with the volume requirement it then violates.
fn read_drive() -> Model {
    Model::builder()
        .package("Drive")
        .part("Drive::drive")
        .part("Drive::drive::engine")
        .tool_link("FreeCAD", "cad/engine.FCStd", "Engine")
        .attribute("Drive::drive::engine::volume")
        .value("900000.0 [millimetre ** 3]")
        .design_value("Volume", read("1000000.0 [mm^3]"))
        .requirement("Drive::volumeReq")
        .satisfy("Drive::@1", "Drive::volumeReq", "Drive::drive")
        .result(ClaimResult::Violated {
            condition: "engine.volume <= 0.95e6 [mm^3]".to_owned(),
        })
        .build()
}

#[test]
fn check_lists_each_design_value_with_its_source_and_model_value_first() {
    let outcome = run("check", read_drive(), &[]);

    let lines: Vec<&str> = outcome.stdout.lines().collect();
    assert_eq!(
        &lines[..2],
        [
            "⚙ Read        Drive::drive::engine::volume  1000000.0 [mm³] from FreeCAD: Volume of Engine in cad/engine.FCStd (Model says 900000.0 [mm³])",
            "1 Design Value: 1 read, 0 not read",
        ],
        "{}",
        outcome.stdout
    );
    assert!(
        lines[2].starts_with("✗ Violated    Drive::volumeReq "),
        "{}",
        outcome.stdout
    );
}

/// The drive with FreeCAD missing: the engine's volume and the gearbox's
/// volume have no Design Value, and the volume requirement is Undecided.
fn unread_drive() -> Model {
    Model::builder()
        .package("Drive")
        .part("Drive::drive")
        .part("Drive::drive::engine")
        .tool_link("FreeCAD", "cad/engine.FCStd", "Engine")
        .attribute("Drive::drive::engine::volume")
        .value("900000.0 [millimetre ** 3]")
        .design_value("Volume", missing("FreeCAD not available"))
        .part("Drive::drive::gearbox")
        .tool_link("FreeCAD", "cad/gearbox.FCStd", "Gearbox")
        .attribute("Drive::drive::gearbox::volume")
        .design_value("Volume", missing("no object Gearbox"))
        .requirement("Drive::volumeReq")
        .satisfy("Drive::@1", "Drive::volumeReq", "Drive::drive")
        .result(ClaimResult::Undecided {
            reason: "no Design Value for engine.volume".to_owned(),
        })
        .build()
}

#[test]
fn a_feature_without_a_design_value_is_listed_with_the_reason() {
    let outcome = run("check", unread_drive(), &[]);

    let lines: Vec<&str> = outcome.stdout.lines().collect();
    assert_eq!(
        &lines[..3],
        [
            "⚠ Not read    Drive::drive::engine::volume   FreeCAD not available, for FreeCAD: Volume of Engine in cad/engine.FCStd (Model says 900000.0 [mm³])",
            "⚠ Not read    Drive::drive::gearbox::volume  no object Gearbox, for FreeCAD: Volume of Gearbox in cad/gearbox.FCStd",
            "2 Design Values: 0 read, 2 not read",
        ],
        "{}",
        outcome.stdout
    );
}

#[test]
fn unreadable_tool_links_fail_check_on_their_own_line_without_strict() {
    let outcome = run("check", unread_drive(), &[]);

    assert_eq!(outcome.code, EXIT_FAILURE, "{}", outcome.stderr);
    assert_eq!(
        outcome.stderr, "error: 2 Tool Links could not be read\n",
        "an Undecided requirement fails nothing else without --strict"
    );
}

#[test]
fn one_unreadable_tool_link_is_named_in_the_singular_before_other_failures() {
    let model = Model::builder()
        .package("Drive")
        .part("Drive::drive")
        .part("Drive::drive::engine")
        .tool_link("FreeCAD", "cad/engine.FCStd", "Engine")
        .attribute("Drive::drive::engine::volume")
        .value("900000.0 [millimetre ** 3]")
        .design_value("Volume", missing("FreeCAD not available"))
        .attribute("Drive::drive::engine::area")
        .design_value("Area", missing("FreeCAD not available"))
        .requirement("Drive::volumeReq")
        .satisfy("Drive::@1", "Drive::volumeReq", "Drive::drive")
        .result(ClaimResult::Violated {
            condition: "engine.mass <= 10 [kg]".to_owned(),
        })
        .build();

    let outcome = run("check", model, &[]);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert_eq!(
        outcome.stderr,
        "error: 1 Tool Link could not be read\nerror: 1 requirement is violated\n"
    );
}

#[test]
fn read_design_values_fail_nothing_of_their_own() {
    let model = Model::builder()
        .package("Drive")
        .part("Drive::drive")
        .part("Drive::drive::engine")
        .tool_link("FreeCAD", "cad/engine.FCStd", "Engine")
        .attribute("Drive::drive::engine::volume")
        .design_value("Volume", read("1000000.0 [mm^3]"))
        .build();

    let outcome = run("check", model, &[]);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    assert_eq!(outcome.stderr, "");
    assert!(
        outcome.stdout.starts_with(
            "⚙ Read        Drive::drive::engine::volume  1000000.0 [mm³] from FreeCAD: Volume of Engine in cad/engine.FCStd\n"
        ),
        "a feature the Model gives no value says nothing of one: {}",
        outcome.stdout
    );
}

#[test]
fn requirements_lists_design_values_first_and_fails_on_unreadable_links() {
    let read = run("requirements", read_drive(), &[]);
    assert!(
        read.stdout.starts_with(
            "⚙ Read        Drive::drive::engine::volume  1000000.0 [mm³] from FreeCAD: Volume of Engine in cad/engine.FCStd (Model says 900000.0 [mm³])\n1 Design Value: 1 read, 0 not read\n✗ Violated    Drive::volumeReq "
        ),
        "{}",
        read.stdout
    );

    let unread = run("requirements", unread_drive(), &[]);
    assert_eq!(unread.code, EXIT_FAILURE);
    assert_eq!(unread.stderr, "error: 2 Tool Links could not be read\n");
}

#[test]
fn a_model_without_tool_links_prints_no_design_values() {
    for command in ["check", "requirements"] {
        let outcome = run(command, common::requirements(), &["--strict"]);

        assert!(
            outcome
                .stdout
                .starts_with("✗ Violated    Req::vehicleSpec "),
            "{command}: {}",
            outcome.stdout
        );
        assert!(!outcome.stdout.contains("Design Value"), "{command}");
        assert!(!outcome.stderr.contains("Tool Link"), "{command}");
    }
}

#[test]
fn two_elements_linked_to_one_unreadable_object_are_two_tool_links_not_read() {
    let model = Model::builder()
        .package("Drive")
        .part("Drive::left")
        .tool_link("FreeCAD", "cad/wheel.FCStd", "Wheel")
        .attribute("Drive::left::volume")
        .design_value("Volume", missing("FreeCAD not available"))
        .part("Drive::right")
        .tool_link("FreeCAD", "cad/wheel.FCStd", "Wheel")
        .attribute("Drive::right::volume")
        .design_value("Volume", missing("FreeCAD not available"))
        .build();

    let outcome = run("check", model, &[]);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert_eq!(outcome.stderr, "error: 2 Tool Links could not be read\n");
}
