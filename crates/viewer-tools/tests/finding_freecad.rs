//! Finding `freecadcmd` and giving up on it, with stub executables in
//! place of FreeCAD: `SYSML_VIEWER_FREECAD` first, then `PATH`, then the
//! install folders; and the timeout.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use viewer_core::engineering_tools::{EngineeringTool, Wanted};
use viewer_tools::freecad::{FreeCad, Machine};

/// A shell script at `path` running `body`, made executable.
fn stub(path: &Path, body: &str) -> PathBuf {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.to_path_buf()
}

fn found(machine: Machine) -> PathBuf {
    FreeCad::from_machine(machine)
        .executable()
        .unwrap()
        .to_path_buf()
}

fn volume_of_body() -> Wanted {
    Wanted::from([("Body".into(), ["Volume".into()].into())])
}

fn parts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parts.FCStd")
}

#[test]
fn without_freecad_every_read_says_freecad_is_not_available() {
    let mut freecad = FreeCad::from_machine(Machine::default());

    let reason = freecad.read(&parts(), &volume_of_body()).unwrap_err();

    assert!(reason.starts_with("FreeCAD not available"), "{reason}");
    assert!(reason.contains("SYSML_VIEWER_FREECAD"), "{reason}");
}

#[test]
fn the_variable_comes_before_path_and_install_folders() {
    let dir = tempfile::tempdir().unwrap();
    let named = stub(&dir.path().join("mine/freecadcmd"), "true");
    stub(&dir.path().join("bin/freecadcmd"), "true");
    stub(&dir.path().join("opt/FreeCAD 1.1/bin/freecadcmd"), "true");

    let executable = found(Machine {
        executable: Some(named.clone().into()),
        path: Some(dir.path().join("bin").into()),
        install_dirs: vec![dir.path().join("opt")],
        ..Machine::default()
    });

    assert_eq!(executable, named);
}

#[test]
fn a_variable_naming_nothing_is_not_passed_over_for_path() {
    let dir = tempfile::tempdir().unwrap();
    stub(&dir.path().join("bin/freecadcmd"), "true");
    let missing = dir.path().join("nowhere/freecadcmd");

    let freecad = FreeCad::from_machine(Machine {
        executable: Some(missing.clone().into()),
        path: Some(dir.path().join("bin").into()),
        ..Machine::default()
    });

    let reason = freecad.executable().unwrap_err();
    assert_eq!(
        reason,
        format!(
            "FreeCAD not available: SYSML_VIEWER_FREECAD names {}, which does not exist",
            missing.display()
        )
    );
}

#[test]
fn path_is_searched_for_either_name() {
    for name in ["freecadcmd", "FreeCADCmd"] {
        let dir = tempfile::tempdir().unwrap();
        let on_path = stub(&dir.path().join("second").join(name), "true");
        std::fs::create_dir(dir.path().join("first")).unwrap();
        let path =
            std::env::join_paths([dir.path().join("first"), dir.path().join("second")]).unwrap();

        let executable = found(Machine {
            path: Some(path),
            ..Machine::default()
        });

        assert_eq!(executable, on_path, "{name}");
    }
}

#[test]
fn path_comes_before_install_folders() {
    let dir = tempfile::tempdir().unwrap();
    let on_path = stub(&dir.path().join("bin/freecadcmd"), "true");
    stub(&dir.path().join("opt/FreeCAD 1.1/bin/freecadcmd"), "true");

    let executable = found(Machine {
        path: Some(dir.path().join("bin").into()),
        install_dirs: vec![dir.path().join("opt")],
        ..Machine::default()
    });

    assert_eq!(executable, on_path);
}

#[test]
fn the_newest_install_is_found_including_a_windows_one_seen_from_wsl() {
    // As `/mnt/c/Program Files` looks from WSL.
    let dir = tempfile::tempdir().unwrap();
    let program_files = dir.path().join("Program Files");
    stub(
        &program_files.join("FreeCAD 1.0/bin/freecadcmd.exe"),
        "true",
    );
    let newest = stub(
        &program_files.join("FreeCAD 1.1/bin/freecadcmd.exe"),
        "true",
    );
    std::fs::create_dir_all(program_files.join("Inkscape/bin")).unwrap();

    let executable = found(Machine {
        install_dirs: vec![dir.path().join("opt"), program_files],
        ..Machine::default()
    });

    assert_eq!(executable, newest);
}

#[test]
fn the_newest_install_is_found_by_its_version_numbers_not_its_spelling() {
    let dir = tempfile::tempdir().unwrap();
    let opt = dir.path().join("opt");
    stub(&opt.join("FreeCAD 1.9/bin/freecadcmd"), "true");
    let newest = stub(&opt.join("FreeCAD 1.10/bin/freecadcmd"), "true");
    stub(&opt.join("FreeCAD 1.2/bin/freecadcmd"), "true");
    stub(&opt.join("FreeCAD/bin/freecadcmd"), "true");

    let executable = found(Machine {
        install_dirs: vec![opt],
        ..Machine::default()
    });

    assert_eq!(executable, newest);
}

#[test]
fn a_missing_file_is_named() {
    let dir = tempfile::tempdir().unwrap();
    let freecad = stub(&dir.path().join("freecadcmd"), "true");
    let file = dir.path().join("cad/engine.FCStd");

    let reason = FreeCad::from_machine(Machine {
        executable: Some(freecad.into()),
        ..Machine::default()
    })
    .read(&file, &volume_of_body())
    .unwrap_err();

    assert_eq!(reason, format!("file {} not found", file.display()));
}

#[test]
fn an_answer_depends_on_its_own_file_only() {
    let freecad = FreeCad::from_machine(Machine::default());

    assert_eq!(freecad.dependent_files(&parts()), vec![parts()]);
}

#[test]
fn a_run_may_take_30_seconds_unless_the_machine_says_otherwise() {
    assert_eq!(
        FreeCad::from_machine(Machine::default()).timeout(),
        Ok(Duration::from_secs(30))
    );
    let freecad = FreeCad::from_machine(Machine {
        timeout: Some("120".into()),
        ..Machine::default()
    });
    assert_eq!(freecad.timeout(), Ok(Duration::from_secs(120)));
}

#[test]
fn a_run_that_does_not_answer_in_time_is_killed() {
    let dir = tempfile::tempdir().unwrap();
    let hanging = stub(&dir.path().join("freecadcmd"), "exec sleep 30");
    let mut freecad = FreeCad::from_machine(Machine {
        executable: Some(hanging.into()),
        timeout: Some("1".into()),
        ..Machine::default()
    });
    let started = Instant::now();

    let reason = freecad.read(&parts(), &volume_of_body()).unwrap_err();

    assert_eq!(reason, "FreeCAD did not answer within 1 s");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "gave up promptly"
    );
}

#[test]
fn a_timeout_that_is_not_a_number_of_seconds_is_a_reason() {
    let dir = tempfile::tempdir().unwrap();
    let freecad = stub(&dir.path().join("freecadcmd"), "true");
    let mut freecad = FreeCad::from_machine(Machine {
        executable: Some(freecad.into()),
        timeout: Some("soon".into()),
        ..Machine::default()
    });

    let reason = freecad.read(&parts(), &volume_of_body()).unwrap_err();

    assert_eq!(
        reason,
        "SYSML_VIEWER_FREECAD_TIMEOUT is soon, not a number of seconds"
    );
}

#[test]
fn a_run_that_prints_no_answer_says_what_freecad_said() {
    let dir = tempfile::tempdir().unwrap();
    let broken = stub(
        &dir.path().join("freecadcmd"),
        "echo 'ModuleNotFoundError: No module named Part' >&2; exit 1",
    );
    let mut freecad = FreeCad::from_machine(Machine {
        executable: Some(broken.into()),
        ..Machine::default()
    });

    let reason = freecad.read(&parts(), &volume_of_body()).unwrap_err();

    assert_eq!(
        reason,
        "FreeCAD gave no answer: ModuleNotFoundError: No module named Part"
    );
}
