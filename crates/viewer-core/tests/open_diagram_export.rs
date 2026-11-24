//! Exporting an Open Diagram, and redrawing it when its Model reloads.

mod common;

use std::sync::Arc;
use viewer_core::model::Direction;
use viewer_core::{DiagramKind, Model, QualifiedName};

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
fn exporting_a_diagram_whose_root_is_gone_fails_with_the_reason() {
    let diagram = common::open_tool(&model(), DiagramKind::General, QualifiedName::from("Gone"));
    let folder = tempfile::TempDir::new().unwrap();

    let error = diagram
        .export_svg(&folder.path().join("x.svg"))
        .unwrap_err();

    assert!(error.to_string().contains("Gone"), "{error}");
}

#[test]
fn reopening_a_model_rebuilds_the_open_diagram() {
    let mut diagram = common::open_tool(&model(), DiagramKind::General, QualifiedName::from("P"));
    let smaller = Model::builder().package("P").part_def("P::Engine").build();

    diagram.reload(Arc::new(smaller), None);

    assert_eq!(
        diagram.scene().unwrap().elements(),
        [&QualifiedName::from("P::Engine")]
    );
}
