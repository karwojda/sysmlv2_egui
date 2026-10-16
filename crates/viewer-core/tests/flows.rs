//! Seam 1: unnamed connections, flows, bindings and reference parts from
//! the `flows` fixture's fake twin.

mod common;

use viewer_core::browser::tree;
use viewer_core::properties::properties;
use viewer_core::scene::{Depicts, Shape};
use viewer_core::{DiagramKind, QualifiedName, Scene, build_scene};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn scene(kind: DiagramKind, root: &str) -> Scene {
    build_scene(&common::flows(), kind, &qn(root)).unwrap()
}

fn connector(scene: &Scene, connection: &str) -> (usize, Vec<String>) {
    let depicts = Depicts::Connection(qn(connection));
    let shapes: Vec<&Shape> = scene.items_depicting(&depicts).collect();
    let lines = shapes.iter().filter(|s| matches!(s, Shape::Polyline { .. })).count();
    let texts = shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    (lines, texts)
}

#[test]
fn an_unnamed_connection_is_drawn_without_a_label_in_an_interconnection_diagram() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");

    let (lines, texts) = connector(&scene, "F::Vehicle::@7");

    assert_eq!(lines, 1, "it is drawn");
    assert!(texts.is_empty(), "no name label: {texts:?}");
}

#[test]
fn an_unnamed_connection_is_an_unlabelled_edge_in_a_general_diagram() {
    let scene = scene(DiagramKind::General, "F::Vehicle");

    let (lines, texts) = connector(&scene, "F::Vehicle::@7");

    assert_eq!(lines, 1);
    assert!(texts.is_empty(), "{texts:?}");
}

#[test]
fn an_unnamed_connection_is_listed_by_its_ends_in_the_connections_compartment() {
    let scene = scene(DiagramKind::General, "F");
    let rows: Vec<String> = scene
        .items_depicting(&Depicts::Element(qn("F::Vehicle")))
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    assert!(rows.contains(&"(tank.fuelOut.inner → engine.fuelIn.inner)".to_owned()), "{rows:?}");
}

#[test]
fn unnamed_elements_are_named_by_their_ends_in_the_browser() {
    let entries = tree(&common::flows(), "");
    let vehicle = entries[0].children.iter().find(|e| e.name == "Vehicle").unwrap();
    let names: Vec<(&str, &str)> = vehicle.children.iter().map(|e| (e.keyword.as_str(), e.name.as_str())).collect();

    assert!(names.contains(&("connection", "tank.fuelOut.inner → engine.fuelIn.inner")), "{names:?}");
    assert!(names.contains(&("bind", "tank.level = engine.fuelLevel")), "{names:?}");
    assert!(names.contains(&("flow", "fuelFlow")), "{names:?}");
}

#[test]
fn the_properties_panel_calls_an_unnamed_element_unnamed() {
    let props = properties(&common::flows(), &qn("F::Vehicle::@6")).unwrap();

    let name = props.rows.iter().find(|p| p.label == "Name").unwrap();
    assert_eq!(name.value, "(unnamed)");
    assert!(props.rows.iter().any(|p| p.label == "Kind" && p.value == "«bind»"));
}
