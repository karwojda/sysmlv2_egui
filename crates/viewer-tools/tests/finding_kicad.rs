//! Finding `kicad-cli` and giving up on it, with stub executables in
//! place of KiCad: `SYSML_VIEWER_KICAD` first, then `PATH`, then the
//! install folders; and the timeout.
#![cfg(unix)]

mod common;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use common::script;
use viewer_core::engineering_tools::{EngineeringTool, Wanted};
use viewer_tools::kicad::{KiCad, Machine};

fn found(machine: Machine) -> PathBuf {
    KiCad::from_machine(machine)
        .executable()
        .unwrap()
        .to_path_buf()
}

fn width_of_board() -> Wanted {
    Wanted::from([("board".into(), ["Width".into()].into())])
}

/// An empty KiCad project in `dir`, with its schematic and board file.
fn project(dir: &Path) -> PathBuf {
    for extension in ["kicad_pro", "kicad_sch", "kicad_pcb"] {
        std::fs::write(dir.join(format!("board.{extension}")), "").unwrap();
    }
    dir.join("board.kicad_pro")
}

#[test]
fn without_kicad_every_read_says_kicad_is_not_available() {
    let dir = tempfile::tempdir().unwrap();
    let mut kicad = KiCad::from_machine(Machine::default());

    let reason = kicad
        .read(&project(dir.path()), &width_of_board())
        .unwrap_err();

    assert!(reason.starts_with("KiCad not available"), "{reason}");
    assert!(reason.contains("SYSML_VIEWER_KICAD"), "{reason}");
}

#[test]
fn the_variable_comes_before_path_and_install_folders() {
    let dir = tempfile::tempdir().unwrap();
    let named = script(&dir.path().join("mine/kicad-cli"), "true");
    script(&dir.path().join("bin/kicad-cli"), "true");
    script(&dir.path().join("opt/KiCad/10.0/bin/kicad-cli"), "true");

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
    script(&dir.path().join("bin/kicad-cli"), "true");
    let missing = dir.path().join("nowhere/kicad-cli");

    let kicad = KiCad::from_machine(Machine {
        executable: Some(missing.clone().into()),
        path: Some(dir.path().join("bin").into()),
        ..Machine::default()
    });

    assert_eq!(
        kicad.executable().unwrap_err(),
        format!(
            "KiCad not available: SYSML_VIEWER_KICAD names {}, which does not exist",
            missing.display()
        )
    );
}

#[test]
fn path_comes_before_install_folders() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("first")).unwrap();
    let on_path = script(&dir.path().join("second/kicad-cli"), "true");
    script(&dir.path().join("opt/KiCad/10.0/bin/kicad-cli"), "true");
    let path = std::env::join_paths([dir.path().join("first"), dir.path().join("second")]).unwrap();

    let executable = found(Machine {
        path: Some(path),
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
    script(&program_files.join("KiCad/9.0/bin/kicad-cli.exe"), "true");
    let newest = script(&program_files.join("KiCad/10.0/bin/kicad-cli.exe"), "true");
    script(&program_files.join("KiCad/8.0/bin/kicad-cli.exe"), "true");

    let executable = found(Machine {
        install_dirs: vec![dir.path().join("Applications"), program_files],
        ..Machine::default()
    });

    assert_eq!(executable, newest);
}

#[test]
fn a_macos_install_is_found_in_its_app_bundle() {
    let dir = tempfile::tempdir().unwrap();
    let applications = dir.path().join("Applications");
    let bundled = script(
        &applications.join("KiCad/KiCad.app/Contents/MacOS/kicad-cli"),
        "true",
    );

    let executable = found(Machine {
        install_dirs: vec![applications],
        ..Machine::default()
    });

    assert_eq!(executable, bundled);
}

#[test]
fn a_run_may_take_30_seconds_unless_the_machine_says_otherwise() {
    assert_eq!(
        KiCad::from_machine(Machine::default()).timeout(),
        Ok(Duration::from_secs(30))
    );
    let kicad = KiCad::from_machine(Machine {
        timeout: Some("60".into()),
        ..Machine::default()
    });
    assert_eq!(kicad.timeout(), Ok(Duration::from_secs(60)));
}

#[test]
fn a_run_that_does_not_answer_in_time_is_killed() {
    let dir = tempfile::tempdir().unwrap();
    let hanging = script(&dir.path().join("kicad-cli"), "exec sleep 30");
    let mut kicad = KiCad::from_machine(Machine {
        executable: Some(hanging.into()),
        timeout: Some("1".into()),
        ..Machine::default()
    });
    let started = Instant::now();

    let reason = kicad
        .read(&project(dir.path()), &width_of_board())
        .unwrap_err();

    assert_eq!(reason, "KiCad did not answer within 1 s");
    assert!(
        started.elapsed() < Duration::from_secs(10),
        "gave up promptly"
    );
}

#[test]
fn a_timeout_that_is_not_a_number_of_seconds_is_a_reason() {
    let dir = tempfile::tempdir().unwrap();
    let kicad_cli = script(&dir.path().join("kicad-cli"), "echo 10.0.7");
    let mut kicad = KiCad::from_machine(Machine {
        executable: Some(kicad_cli.into()),
        timeout: Some("soon".into()),
        ..Machine::default()
    });

    let reason = kicad
        .read(&project(dir.path()), &width_of_board())
        .unwrap_err();

    assert_eq!(
        reason,
        "SYSML_VIEWER_KICAD_TIMEOUT is soon, not a number of seconds"
    );
}
