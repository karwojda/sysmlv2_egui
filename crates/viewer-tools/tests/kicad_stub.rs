//! The KiCad adapter through the `EngineeringTool` trait, with a stub
//! `kicad-cli` in place of KiCad. The stub answers `version`, and writes
//! to the `-o` path a netlist (recorded from the Feather demo, or written
//! by the test) and the board statistics recorded from the Feather demo:
//! in English when its config folder says so, as a forced-English KiCad
//! does, and otherwise as the same KiCad recorded them in Polish, with
//! decimal commas. It refuses to run without `KICAD_CONFIG_HOME`.
#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use viewer_core::engineering_tools::{Answer, EngineeringTool, Parts, ReadValue, Wanted};
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

/// A stub `kicad-cli` in `dir` that says it is KiCad `version` and
/// answers with the recordings of the Feather demo.
fn stub(dir: &Path, version: &str) -> PathBuf {
    answering(dir, version, Some(&fixture("feather-netlist.xml")), true)
}

/// A stub `kicad-cli` in `dir` that says it is KiCad `version`, writes
/// `netlist` for `sch export netlist`, and, when `stats`, the recorded
/// board statistics for `pcb export stats`. An export it isn't given
/// fails.
fn answering(dir: &Path, version: &str, netlist: Option<&Path>, stats: bool) -> PathBuf {
    let english = fixture("feather-stats.json");
    let polish = fixture("feather-stats-polish.json");
    let netlist = match netlist {
        Some(netlist) => format!(r#"cp "{}" "$out""#, netlist.display()),
        None => "echo 'no netlist here' >&2; exit 4".into(),
    };
    let stats = if stats {
        format!(
            r#"if grep -q English "$KICAD_CONFIG_HOME/10.0/kicad_common.json" 2>/dev/null; then
    cp "{english}" "$out"
  else
    cp "{polish}" "$out"
  fi"#,
            english = english.display(),
            polish = polish.display(),
        )
    } else {
        "echo 'no statistics here' >&2; exit 4".into()
    };
    script(
        &dir.join("kicad-cli"),
        &format!(
            r#"[ -n "$KICAD_CONFIG_HOME" ] || {{ echo "no KICAD_CONFIG_HOME" >&2; exit 9; }}
if [ "$1" = version ]; then echo "{version}"; exit 0; fi
command="$1 $2 $3"
out=""
while [ $# -gt 0 ]; do
  if [ "$1" = -o ]; then out="$2"; fi
  shift
done
case "$command" in
  "sch export netlist")
  {netlist}
  ;;
  "pcb export stats")
  {stats}
  ;;
  *) echo "unknown command $command" >&2; exit 5 ;;
esac"#
        ),
    )
}

/// A part in a netlist: its reference designator, its fields
/// (`("Power", "45mW")`; an empty value is written as an empty field) and
/// its properties (`dnp`, `exclude_from_bom`).
type Part<'a> = (&'a str, &'a [(&'a str, &'a str)], &'a [&'a str]);

/// A netlist of `parts` at `path`, as `sch export netlist --format
/// kicadxml` writes it.
fn netlist(path: &Path, parts: &[Part]) -> PathBuf {
    let mut xml = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<export version=\"E\">\n  <components>\n",
    );
    for (reference, fields, properties) in parts {
        xml += &format!("    <comp ref=\"{reference}\">\n      <value>x</value>\n      <fields>\n");
        for (name, value) in *fields {
            if value.is_empty() {
                xml += &format!("        <field name=\"{name}\"/>\n");
            } else {
                xml += &format!("        <field name=\"{name}\">{value}</field>\n");
            }
        }
        xml += "      </fields>\n";
        for property in *properties {
            xml += &format!("      <property name=\"{property}\"/>\n");
        }
        xml += "    </comp>\n";
    }
    xml += "  </components>\n</export>\n";
    std::fs::write(path, xml).unwrap();
    path.to_path_buf()
}

/// What KiCad answers for `properties` of the board, given a netlist of
/// `parts` and the recorded board statistics.
fn board_of(parts: &[Part], properties: &[&str]) -> Answer {
    let dir = tempfile::tempdir().unwrap();
    let netlist = netlist(&dir.path().join("netlist.xml"), parts);
    let mut kicad = kicad(&answering(dir.path(), "10.0.7", Some(&netlist), true));
    kicad
        .read(&project(dir.path()), &wanted("board", properties))
        .unwrap()
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
    answer[object][property].clone().map(|read| {
        assert_eq!(read.parts, None, "a board dimension sums no parts");
        read.value
    })
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

    let read = kicad.read(&project, &wanted("board", &["Width"])).unwrap();

    assert_eq!(
        answer(&read, "board", "Width"),
        Err(format!(
            "KiCad gave no board statistics for {}: Failed to load board: \
             Expecting '(' in 'board.kicad_pcb', line 1, offset 1.",
            project.with_extension("kicad_pcb").display()
        ))
    );
}

#[test]
fn component_count_is_the_number_of_placed_parts() {
    let read = board_of(
        &[
            ("U1", &[], &[]),
            ("R1", &[], &[]),
            ("TP1", &[], &["dnp"]),
            ("JP1", &[], &["exclude_from_bom"]),
            ("H1", &[], &["exclude_from_bom", "dnp"]),
        ],
        &["ComponentCount"],
    );

    assert_eq!(read["board"]["ComponentCount"], Ok(ReadValue::new("2")));
}

#[test]
fn a_field_is_summed_over_the_placed_parts_in_its_base_unit() {
    let read = board_of(
        &[
            ("U1", &[("Power", "45mW")], &[]),
            ("U2", &[("Power", "1.2 W")], &[]),
            ("D1", &[("Power", "350uW")], &[]),
            ("D2", &[("Power", "350µW")], &[]),
            ("R1", &[], &[]),
            ("TP1", &[("Power", "5W")], &["dnp"]),
            ("JP1", &[("Power", "5W")], &["exclude_from_bom"]),
        ],
        &["Power"],
    );

    assert_eq!(
        read["board"]["Power"],
        Ok(ReadValue::summed(
            "1.2457 [W]",
            Parts::new(["U1", "U2", "D1", "D2"], ["R1"])
        ))
    );
}

#[test]
fn bare_numbers_give_a_sum_with_no_unit() {
    let read = board_of(
        &[
            ("U1", &[("Price", "2.95")], &[]),
            ("U2", &[("Price", "0.1")], &[]),
            ("Y1", &[("Price", "0.2")], &[]),
        ],
        &["Price"],
    );

    assert_eq!(
        read["board"]["Price"],
        Ok(ReadValue::summed(
            "3.25",
            Parts::new(["U1", "U2", "Y1"], [] as [&str; 0])
        ))
    );
}

#[test]
fn an_empty_field_leaves_its_part_uncounted() {
    let read = board_of(
        &[
            ("U1", &[("Power", "45mW")], &[]),
            ("C1", &[("Power", "")], &[]),
        ],
        &["Power"],
    );

    assert_eq!(
        read["board"]["Power"],
        Ok(ReadValue::summed("0.045 [W]", Parts::new(["U1"], ["C1"])))
    );
}

#[test]
fn a_field_no_placed_part_has_sums_to_zero_with_every_part_uncounted() {
    let read = board_of(
        &[
            ("U1", &[("Power", "45mW")], &[]),
            ("C1", &[], &[]),
            ("TP1", &[("Price", "1")], &["dnp"]),
        ],
        &["Price"],
    );

    assert_eq!(
        read["board"]["Price"],
        Ok(ReadValue::summed(
            "0.0",
            Parts::new([] as [&str; 0], ["U1", "C1"])
        ))
    );
}

#[test]
fn field_names_match_case_included() {
    let read = board_of(
        &[
            ("U1", &[("Power", "45mW")], &[]),
            ("U2", &[("power", "1W")], &[]),
        ],
        &["Power"],
    );

    assert_eq!(
        read["board"]["Power"],
        Ok(ReadValue::summed("0.045 [W]", Parts::new(["U1"], ["U2"])))
    );
}

#[test]
fn unreadable_fields_leave_no_sum_and_are_named() {
    let read = board_of(
        &[
            ("U1", &[("Power", "45mW")], &[]),
            ("U10", &[("Power", "TBD")], &[]),
            ("U2", &[("Power", "lots")], &[]),
            ("U3", &[("Power", "1,5 W")], &[]),
        ],
        &["Power", "ComponentCount"],
    );

    assert_eq!(
        read["board"]["Power"],
        Err("the Power field can't be read as a number with a unit on \
             U2 ('lots'), U3 ('1,5 W'), U10 ('TBD')"
            .into())
    );
    assert_eq!(read["board"]["ComponentCount"], Ok(ReadValue::new("4")));
}

#[test]
fn fields_in_units_of_different_kinds_leave_no_sum() {
    let read = board_of(
        &[
            ("U1", &[("Power", "45mW")], &[]),
            ("U2", &[("Power", "12mA")], &[]),
            ("D1", &[("Power", "2 mW")], &[]),
        ],
        &["Power"],
    );

    assert_eq!(
        read["board"]["Power"],
        Err("the Power fields mix units: A on U2; W on D1, U1".into())
    );
}

#[test]
fn bare_numbers_mixed_with_units_leave_no_sum() {
    let read = board_of(
        &[
            ("U1", &[("Power", "45mW")], &[]),
            ("U2", &[("Power", "0.2")], &[]),
            ("D1", &[("Power", "0.1")], &[]),
        ],
        &["Power"],
    );

    assert_eq!(
        read["board"]["Power"],
        Err("the Power fields mix bare numbers (D1, U2) with units (U1)".into())
    );
}

#[test]
fn sums_and_counts_need_no_board_statistics() {
    let dir = tempfile::tempdir().unwrap();
    let netlist = fixture("feather-netlist.xml");
    let mut kicad = kicad(&answering(dir.path(), "10.0.7", Some(&netlist), false));

    let read = kicad
        .read(
            &project(dir.path()),
            &wanted("board", &["ComponentCount", "Power"]),
        )
        .unwrap();

    assert!(read["board"]["ComponentCount"].is_ok());
    assert!(read["board"]["Power"].is_ok());
}

#[test]
fn dimensions_need_no_netlist() {
    let dir = tempfile::tempdir().unwrap();
    let mut kicad = kicad(&answering(dir.path(), "10.0.7", None, true));

    let read = kicad
        .read(&project(dir.path()), &wanted("board", &["Width"]))
        .unwrap();

    assert_eq!(answer(&read, "board", "Width"), Ok("58.42 [mm]".into()));
}

#[test]
fn a_failed_netlist_leaves_only_the_sums_and_counts_without_a_value() {
    let dir = tempfile::tempdir().unwrap();
    let mut kicad = kicad(&answering(dir.path(), "10.0.7", None, true));
    let project = project(dir.path());

    let read = kicad
        .read(&project, &wanted("board", &["Width", "Power"]))
        .unwrap();

    assert_eq!(answer(&read, "board", "Width"), Ok("58.42 [mm]".into()));
    assert_eq!(
        read["board"]["Power"],
        Err(format!(
            "KiCad gave no netlist for {}: no netlist here",
            project.with_extension("kicad_sch").display()
        ))
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
