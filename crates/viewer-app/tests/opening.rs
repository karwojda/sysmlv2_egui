//! Which Diagram opens for a root, and saving Interconnection Diagrams.

use tempfile::TempDir;
use viewer_app::canvas::OpenDiagram;
use viewer_app::startup_kind;
use viewer_core::{DiagramFile, DiagramKind, Model, QualifiedName, diagram_file};

fn model() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .part("P::Vehicle::engine")
        .typed_by("P::Engine")
        .part_def("P::Engine")
        .part("P::car")
        .typed_by("P::Vehicle")
        .attribute_def("P::Mass")
        .build()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

#[test]
fn a_command_line_root_opens_a_general_diagram_where_one_fits() {
    assert_eq!(
        startup_kind(&model(), &qn("P"), false),
        Some(DiagramKind::General)
    );
    assert_eq!(
        startup_kind(&model(), &qn("P::Vehicle"), false),
        Some(DiagramKind::General)
    );
}

#[test]
fn a_part_usage_root_opens_its_interconnection_diagram() {
    assert_eq!(
        startup_kind(&model(), &qn("P::car"), false),
        Some(DiagramKind::Interconnection)
    );
}

#[test]
fn asking_for_interconnection_opens_one_for_a_part_def() {
    assert_eq!(
        startup_kind(&model(), &qn("P::Vehicle"), true),
        Some(DiagramKind::Interconnection)
    );
}

#[test]
fn asking_for_interconnection_falls_back_where_it_doesnt_fit() {
    assert_eq!(
        startup_kind(&model(), &qn("P"), true),
        Some(DiagramKind::General)
    );
}

#[test]
fn a_root_no_diagram_fits_opens_nothing() {
    assert_eq!(
        startup_kind(&model(), &qn("P::Vehicle::nothing"), false),
        None
    );
}

#[test]
fn an_interconnection_diagram_saves_and_reopens_as_one() {
    let folder = TempDir::new().unwrap();
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::Interconnection, qn("P::Vehicle"));
    assert_eq!(diagram.title(), "Vehicle (Interconnection)");

    diagram.save_as(folder.path(), "wiring").unwrap();

    let text = std::fs::read_to_string(diagram_file::path_for(folder.path(), "wiring")).unwrap();
    assert!(text.contains(r#""kind": "interconnection""#));
    let reopened = OpenDiagram::open_saved(&model, DiagramFile::parse(&text).unwrap());
    assert_eq!(reopened.kind, Some(DiagramKind::Interconnection));
    assert!(
        reopened
            .scene
            .unwrap()
            .element_bounds(&qn("P::Vehicle::engine"))
            .is_some()
    );
}
