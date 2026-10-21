//! The FreeCAD adapter through the `EngineeringTool` trait, against the
//! real `freecadcmd` and `fixtures/parts.FCStd` (built by
//! `fixtures/make_parts.py`): `Body` is a 100 mm cube, and `Assembly.Link.`
//! reaches a cylinder of radius 10 mm and height 100 mm through an
//! `App::Link` in an `App::Part`. When FreeCAD can't be found these tests
//! print a note and pass.

use std::path::{Path, PathBuf};

use viewer_core::engineering_tools::{Answer, EngineeringTool, Wanted};
use viewer_tools::{FreeCad, Machine};

fn parts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parts.FCStd")
}

/// FreeCAD as this machine has it, or `None` after a note saying the test
/// is skipped.
fn freecad(test: &str) -> Option<FreeCad> {
    let freecad = FreeCad::from_machine(Machine::current());
    match freecad.executable() {
        Ok(_) => Some(freecad),
        Err(reason) => {
            eprintln!("note: skipping {test}: {reason}");
            None
        }
    }
}

fn wanted(objects: &[(&str, &[&str])]) -> Wanted {
    objects
        .iter()
        .map(|(object, properties)| {
            let properties = properties.iter().map(|p| p.to_string()).collect();
            (object.to_string(), properties)
        })
        .collect()
}

fn answer(answer: &Answer, object: &str, property: &str) -> Result<String, String> {
    answer[object][property].clone()
}

#[test]
fn a_body_reports_its_volume_in_cubic_millimetres() {
    let Some(mut freecad) = freecad("a_body_reports_its_volume_in_cubic_millimetres") else {
        return;
    };

    let read = freecad
        .read(&parts(), &wanted(&[("Body", &["Volume"])]))
        .unwrap();

    assert_eq!(
        answer(&read, "Body", "Volume"),
        Ok("1000000.0 [mm^3]".into())
    );
}

#[test]
fn an_object_inside_an_assembly_is_reached_by_its_dotted_path() {
    let Some(mut freecad) = freecad("an_object_inside_an_assembly_is_reached_by_its_dotted_path")
    else {
        return;
    };

    let read = freecad
        .read(&parts(), &wanted(&[("Assembly.Link.", &["Volume"])]))
        .unwrap();

    // A cylinder of radius 10 mm and height 100 mm: 10000 pi mm^3.
    assert_eq!(
        answer(&read, "Assembly.Link.", "Volume"),
        Ok("31415.9265359 [mm^3]".into())
    );
}

#[test]
fn an_unknown_property_gives_a_reason_naming_it() {
    let Some(mut freecad) = freecad("an_unknown_property_gives_a_reason_naming_it") else {
        return;
    };

    let read = freecad
        .read(&parts(), &wanted(&[("Body", &["Volume", "Colour"])]))
        .unwrap();

    assert_eq!(
        answer(&read, "Body", "Colour"),
        Err("FreeCAD has no property Colour (it reads Volume)".into())
    );
    assert_eq!(
        answer(&read, "Body", "Volume"),
        Ok("1000000.0 [mm^3]".into())
    );
}

/// The reason `object` has no `Volume` in the fixture.
fn reason_for(test: &str, object: &str) -> Option<String> {
    let mut freecad = freecad(test)?;
    let read = freecad
        .read(
            &parts(),
            &wanted(&[(object, &["Volume"]), ("Body", &["Volume"])]),
        )
        .unwrap();
    assert_eq!(
        answer(&read, "Body", "Volume"),
        Ok("1000000.0 [mm^3]".into()),
        "other objects are still read"
    );
    Some(answer(&read, object, "Volume").unwrap_err())
}

#[test]
fn a_missing_object_names_the_object_and_the_file() {
    let Some(reason) = reason_for("a_missing_object_names_the_object_and_the_file", "Engine")
    else {
        return;
    };

    assert_eq!(
        reason,
        format!("no object named 'Engine' in {}", parts().display())
    );
}

#[test]
fn a_path_whose_first_object_is_missing_names_that_object_and_the_file() {
    let Some(reason) = reason_for(
        "a_path_whose_first_object_is_missing_names_that_object_and_the_file",
        "Gearbox.Link.",
    ) else {
        return;
    };

    assert_eq!(
        reason,
        format!("no object named 'Gearbox' in {}", parts().display())
    );
}

#[test]
fn a_path_that_leads_nowhere_names_the_path_and_the_file() {
    let Some(reason) = reason_for(
        "a_path_that_leads_nowhere_names_the_path_and_the_file",
        "Assembly.Shaft.",
    ) else {
        return;
    };

    assert_eq!(
        reason,
        format!("no object at 'Assembly.Shaft.' in {}", parts().display())
    );
}

#[test]
fn a_file_freecad_cannot_open_is_a_reason_for_the_whole_file() {
    let Some(mut freecad) = freecad("a_file_freecad_cannot_open_is_a_reason_for_the_whole_file")
    else {
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("notes.FCStd");
    std::fs::write(&file, "not a FreeCAD document").unwrap();

    let reason = freecad
        .read(&file, &wanted(&[("Body", &["Volume"])]))
        .unwrap_err();

    assert!(
        reason.starts_with(&format!("{}: FreeCAD could not open it", file.display())),
        "{reason}"
    );
}
