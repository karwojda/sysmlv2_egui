//! The KiCad adapter through the `EngineeringTool` trait, with a stub
//! `kicad-cli` in place of KiCad. The stub answers `version`, and writes
//! the board statistics recorded from the Feather demo to the `-o` path:
//! in English when its config folder says so, as a forced-English KiCad
//! does, and otherwise as the same KiCad recorded them in Polish, with
//! decimal commas. It refuses to run without `KICAD_CONFIG_HOME`.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use viewer_core::engineering_tools::{Answer, EngineeringTool, Wanted};
use viewer_tools::kicad::{KiCad, Machine};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/kicad")
        .join(name)
}

/// A shell script at `path` running `body`, made executable.
fn script(path: &Path, body: &str) -> PathBuf {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path.to_path_buf()
}

/// A stub `kicad-cli` in `dir` that says it is KiCad `version`.
fn stub(dir: &Path, version: &str) -> PathBuf {
    let english = fixture("feather-stats.json");
    let polish = fixture("feather-stats-polish.json");
    script(
        &dir.join("kicad-cli"),
        &format!(
            r#"[ -n "$KICAD_CONFIG_HOME" ] || {{ echo "no KICAD_CONFIG_HOME" >&2; exit 9; }}
if [ "$1" = version ]; then echo "{version}"; exit 0; fi
out=""
while [ $# -gt 0 ]; do
  if [ "$1" = -o ]; then out="$2"; fi
  shift
done
if grep -q English "$KICAD_CONFIG_HOME/10.0/kicad_common.json" 2>/dev/null; then
  cp "{english}" "$out"
else
  cp "{polish}" "$out"
fi"#,
            english = english.display(),
            polish = polish.display(),
        ),
    )
}

/// A KiCad project in `dir`: `board.kicad_pro` with its schematic and
/// board file, all empty, since the stub never reads them.
fn project(dir: &Path) -> PathBuf {
    for extension in ["kicad_pro", "kicad_sch", "kicad_pcb"] {
        std::fs::write(dir.join(format!("board.{extension}")), "").unwrap();
    }
    dir.join("board.kicad_pro")
}

fn kicad(executable: &Path) -> KiCad {
    KiCad::from_machine(Machine {
        executable: Some(executable.into()),
        ..Machine::default()
    })
}

fn wanted(object: &str, properties: &[&str]) -> Wanted {
    Wanted::from([(
        object.to_string(),
        properties.iter().map(|p| p.to_string()).collect(),
    )])
}

fn answer(answer: &Answer, object: &str, property: &str) -> Result<String, String> {
    answer[object][property].clone()
}

#[test]
fn the_board_gives_its_dimensions_with_their_units() {
    let dir = tempfile::tempdir().unwrap();
    let mut kicad = kicad(&stub(dir.path(), "10.0.7"));

    let read = kicad
        .read(
            &project(dir.path()),
            &wanted("board", &["Width", "Height", "Thickness", "Area"]),
        )
        .unwrap();

    assert_eq!(answer(&read, "board", "Width"), Ok("58.42 [mm]".into()));
    assert_eq!(answer(&read, "board", "Height"), Ok("22.86 [mm]".into()));
    assert_eq!(answer(&read, "board", "Thickness"), Ok("1.6 [mm]".into()));
    assert_eq!(answer(&read, "board", "Area"), Ok("1329.932 [mm^2]".into()));
}

#[test]
fn kicad_is_not_started_until_a_link_is_read() {
    let dir = tempfile::tempdir().unwrap();
    let started = dir.path().join("started");
    let recording = script(
        &dir.path().join("kicad-cli"),
        &format!("touch '{}'", started.display()),
    );

    let kicad = kicad(&recording);

    assert!(kicad.executable().is_ok());
    assert!(!started.exists(), "kicad-cli was run");
}

#[test]
fn a_kicad_older_than_10_is_a_reason_naming_the_version_found() {
    let dir = tempfile::tempdir().unwrap();
    let mut kicad = kicad(&stub(dir.path(), "9.0.3"));

    let reason = kicad
        .read(&project(dir.path()), &wanted("board", &["Width"]))
        .unwrap_err();

    assert_eq!(reason, "KiCad 10 or newer needed; found 9.0.3");
}

#[test]
fn each_missing_project_file_is_named() {
    for missing in ["kicad_pro", "kicad_sch", "kicad_pcb"] {
        let dir = tempfile::tempdir().unwrap();
        let mut kicad = kicad(&stub(dir.path(), "10.0.7"));
        let project = project(dir.path());
        let gone = project.with_extension(missing);
        std::fs::remove_file(&gone).unwrap();

        let reason = kicad
            .read(&project, &wanted("board", &["Width"]))
            .unwrap_err();

        assert_eq!(reason, format!("file {} not found", gone.display()));
    }
}

#[test]
fn only_the_board_can_be_linked() {
    let dir = tempfile::tempdir().unwrap();
    let mut kicad = kicad(&stub(dir.path(), "10.0.7"));
    let mut wanted = wanted("R12", &["Power"]);
    wanted.insert("board".into(), ["Width".into()].into());

    let read = kicad.read(&project(dir.path()), &wanted).unwrap();

    assert_eq!(
        answer(&read, "R12", "Power"),
        Err("only board can be linked in KiCad for now, not 'R12'".into())
    );
    assert_eq!(answer(&read, "board", "Width"), Ok("58.42 [mm]".into()));
}

#[test]
fn a_board_property_kicad_does_not_give_is_a_reason_naming_it() {
    let dir = tempfile::tempdir().unwrap();
    let mut kicad = kicad(&stub(dir.path(), "10.0.7"));

    let read = kicad
        .read(&project(dir.path()), &wanted("board", &["Colour", "Width"]))
        .unwrap();

    assert_eq!(
        answer(&read, "board", "Colour"),
        Err(
            "KiCad has no board property Colour (it reads Width, Height, Thickness and Area)"
                .into()
        )
    );
    assert_eq!(answer(&read, "board", "Width"), Ok("58.42 [mm]".into()));
}

#[test]
fn a_board_kicad_cannot_load_is_a_reason_saying_what_kicad_said() {
    let dir = tempfile::tempdir().unwrap();
    let failing = script(
        &dir.path().join("kicad-cli"),
        r#"if [ "$1" = version ]; then echo 10.0.7; exit 0; fi
echo "Failed to load board: Expecting '(' in 'board.kicad_pcb', line 1, offset 1." >&2
exit 3"#,
    );
    let mut kicad = kicad(&failing);
    let project = project(dir.path());

    let reason = kicad
        .read(&project, &wanted("board", &["Width"]))
        .unwrap_err();

    assert_eq!(
        reason,
        format!(
            "KiCad gave no board statistics for {}: Failed to load board: \
             Expecting '(' in 'board.kicad_pcb', line 1, offset 1.",
            project.with_extension("kicad_pcb").display()
        )
    );
}

#[test]
fn a_kicad_project_depends_on_every_schematic_and_board_file_in_its_folder() {
    let dir = tempfile::tempdir().unwrap();
    let project = project(dir.path());
    let sheets = dir.path().join("sch");
    std::fs::create_dir(&sheets).unwrap();
    std::fs::write(sheets.join("power.kicad_sch"), "").unwrap();
    std::fs::write(sheets.join("notes.txt"), "").unwrap();
    std::fs::write(dir.path().join("board.kicad_prl"), "").unwrap();
    let kicad = KiCad::from_machine(Machine::default());

    let files = kicad.dependent_files(&project);

    assert_eq!(
        files,
        vec![
            dir.path().join("board.kicad_pcb"),
            dir.path().join("board.kicad_sch"),
            sheets.join("power.kicad_sch"),
        ]
    );
}
