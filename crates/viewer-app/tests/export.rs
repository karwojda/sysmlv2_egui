//! Exporting from the viewer writes exactly what the command line writes.

use viewer_app::canvas::OpenDiagram;
use viewer_core::model::Direction;
use viewer_core::{DiagramFile, DiagramKind, FakeBackend, Model, QualifiedName, diagram_file};

fn model() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Engine")
        .port("P::Engine::fuelIn")
        .typed_by("P::FuelPort")
        .direction(Direction::In)
        .port_def("P::FuelPort")
        .part_def("P::Vehicle")
        .part("P::Vehicle::engine")
        .typed_by("P::Engine")
        .multiplicity("1", "1")
        .part("P::vehicle")
        .typed_by("P::Vehicle")
        .build()
}

#[test]
fn the_viewer_export_is_byte_identical_to_the_cli_export() {
    let folder = tempfile::TempDir::new().unwrap();
    let diagram = DiagramFile::new("p", DiagramKind::General, QualifiedName::from("P"));
    let path = diagram_file::path_for(folder.path(), "p");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, diagram.to_text()).unwrap();
    let (from_cli, from_viewer) = (folder.path().join("cli.svg"), folder.path().join("viewer.svg"));

    let code = viewer_cli::run(
        ["sysml-viewer".into(), "export".into(), folder.path().into(), "--diagram".into(), "p".into(), "--out".into(), from_cli.clone().into()],
        &mut FakeBackend::new(model()),
        &mut Vec::new(),
    );
    OpenDiagram::open(&model(), DiagramKind::General, QualifiedName::from("P"))
        .export_svg(&from_viewer)
        .unwrap();

    assert_eq!(code, viewer_cli::EXIT_OK);
    assert_eq!(std::fs::read(from_viewer).unwrap(), std::fs::read(from_cli).unwrap());
}

#[test]
fn exporting_a_diagram_whose_root_is_gone_fails_with_the_reason() {
    let diagram = OpenDiagram::open(&model(), DiagramKind::General, QualifiedName::from("Gone"));
    let folder = tempfile::TempDir::new().unwrap();

    let error = diagram.export_svg(&folder.path().join("x.svg")).unwrap_err();

    assert!(error.contains("Gone"), "{error}");
}

#[test]
fn reopening_a_model_rebuilds_the_open_diagram() {
    let mut diagram = OpenDiagram::open(&model(), DiagramKind::General, QualifiedName::from("P"));
    let smaller = Model::builder().package("P").part_def("P::Engine").build();

    diagram.refresh(&smaller);

    assert_eq!(diagram.scene.unwrap().elements(), [&QualifiedName::from("P::Engine")]);
}
