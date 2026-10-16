//! Open Diagrams: editing marks them unsaved, saving writes a Diagram File,
//! and reopening it restores the Layout. The Model files are never written.

use tempfile::TempDir;
use viewer_app::canvas::{OpenDiagram, check_name};
use viewer_core::editing::Edit;
use viewer_core::{DiagramFile, DiagramKind, Model, QualifiedName, diagram_file};

fn model() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .part_def("P::Car")
        .specializes("P::Vehicle")
        .build()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn moved() -> (Model, OpenDiagram) {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    diagram.edit(
        &model,
        &Edit::Move {
            element: qn("P::Car"),
            dx: 40.0,
            dy: 25.0,
        },
    );
    (model, diagram)
}

#[test]
fn an_ad_hoc_diagram_starts_saved_and_unnamed() {
    let diagram = OpenDiagram::open(&model(), DiagramKind::General, qn("P"));

    assert!(!diagram.dirty);
    assert_eq!(diagram.name, None);
    assert_eq!(diagram.title(), "P (General)");
}

#[test]
fn an_edit_marks_the_diagram_unsaved() {
    let (_, diagram) = moved();

    assert!(diagram.dirty);
    assert_eq!(diagram.title(), "P (General) •");
}

#[test]
fn an_edit_that_changes_nothing_leaves_it_saved() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));

    diagram.edit(&model, &Edit::ShowAll);

    assert!(!diagram.dirty);
}

#[test]
fn saving_writes_a_diagram_file_that_reopens_with_the_same_layout() {
    let folder = TempDir::new().unwrap();
    let (model, mut diagram) = moved();
    let car = diagram
        .scene
        .as_ref()
        .unwrap()
        .node_bounds(&qn("P::Car"))
        .unwrap();

    diagram.save_as(folder.path(), "overview").unwrap();

    assert!(!diagram.dirty);
    assert_eq!(diagram.title(), "overview");
    let text = std::fs::read_to_string(diagram_file::path_for(folder.path(), "overview")).unwrap();
    let reopened = OpenDiagram::open_saved(&model, DiagramFile::parse(&text).unwrap());
    assert_eq!(
        reopened.scene.unwrap().node_bounds(&qn("P::Car")).unwrap(),
        car
    );
    assert_eq!(diagram_file::names_in(folder.path()).unwrap(), ["overview"]);
}

#[test]
fn saving_writes_only_the_diagram_file() {
    let folder = TempDir::new().unwrap();
    std::fs::write(folder.path().join("model.sysml"), "package P;").unwrap();
    let (_, mut diagram) = moved();

    diagram.save_as(folder.path(), "overview").unwrap();

    assert_eq!(
        std::fs::read_to_string(folder.path().join("model.sysml")).unwrap(),
        "package P;"
    );
    let entries: Vec<_> = std::fs::read_dir(folder.path())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(
        entries.len(),
        2,
        "only the model file and the diagrams folder: {entries:?}"
    );
}

#[test]
fn saving_again_under_its_own_name_overwrites_but_another_diagrams_name_is_refused() {
    let folder = TempDir::new().unwrap();
    let (model, mut first) = moved();
    first.save_as(folder.path(), "overview").unwrap();
    first.edit(&model, &Edit::Hide(qn("P::Vehicle")));

    assert!(first.save_as(folder.path(), "overview").is_ok());

    let mut second = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    let error = second.save_as(folder.path(), "overview").unwrap_err();
    assert!(error.contains("already exists"), "{error}");
}

#[test]
fn diagram_names_must_be_safe_file_names() {
    for good in ["overview", "fuel-system_v2", "a.b"] {
        assert!(check_name(good).is_ok(), "{good}");
    }
    for bad in ["", ".hidden", "../escape", "with space", "a/b"] {
        assert!(check_name(bad).is_err(), "{bad}");
    }
}

#[test]
fn a_saved_diagram_is_rebuilt_with_its_layout_when_the_model_reloads() {
    let folder = TempDir::new().unwrap();
    let (_, mut diagram) = moved();
    diagram.save_as(folder.path(), "overview").unwrap();
    let car = diagram
        .scene
        .as_ref()
        .unwrap()
        .node_bounds(&qn("P::Car"))
        .unwrap();
    let grown = Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .part_def("P::Car")
        .specializes("P::Vehicle")
        .part_def("P::Bus")
        .build();

    diagram.refresh(&grown);

    let scene = diagram.scene.unwrap();
    assert_eq!(scene.node_bounds(&qn("P::Car")).unwrap(), car);
    assert!(scene.node_bounds(&qn("P::Bus")).is_some());
}
