//! The Selection: which nodes clicks and selection boxes pick, keeping it
//! within one box, and what drops out of it.

use viewer_app::canvas::{OpenDiagram, Selection};
use viewer_core::editing::Edit;
use viewer_core::scene::Rect;
use viewer_core::{DiagramKind, Model, QualifiedName, Scene};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn general() -> (Model, OpenDiagram) {
    let model = Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .part_def("P::Car")
        .specializes("P::Vehicle")
        .part_def("P::Truck")
        .specializes("P::Vehicle")
        .build();
    let diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    (model, diagram)
}

/// A Vehicle with an engine and an axle, the axle opened in place to show
/// its two wheels.
fn interconnection() -> (Model, OpenDiagram) {
    let model = Model::builder()
        .package("P")
        .part_def("P::Engine")
        .part_def("P::Axle")
        .part("P::Axle::left")
        .part("P::Axle::right")
        .part_def("P::Vehicle")
        .part("P::Vehicle::engine")
        .typed_by("P::Engine")
        .part("P::Vehicle::axle")
        .typed_by("P::Axle")
        .build();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::Interconnection, qn("P::Vehicle"));
    diagram.edit(&model, &Edit::Open("axle".into()));
    (model, diagram)
}

fn scene(diagram: &OpenDiagram) -> &Scene {
    diagram.scene.as_ref().unwrap()
}

fn keys(selection: &Selection) -> Vec<&str> {
    selection.keys().iter().map(|k| k.as_str()).collect()
}

/// The smallest rect around the nodes `keys`, grown by a margin.
fn around(scene: &Scene, keys: &[&str]) -> Rect {
    let bounds: Vec<Rect> = keys
        .iter()
        .map(|k| scene.node_bounds(&qn(k)).unwrap())
        .collect();
    let left = bounds.iter().map(|b| b.x).fold(f32::MAX, f32::min) - 2.0;
    let top = bounds.iter().map(|b| b.y).fold(f32::MAX, f32::min) - 2.0;
    let right = bounds.iter().map(|b| b.right()).fold(f32::MIN, f32::max) + 2.0;
    let bottom = bounds.iter().map(|b| b.bottom()).fold(f32::MIN, f32::max) + 2.0;
    Rect {
        x: left,
        y: top,
        width: right - left,
        height: bottom - top,
    }
}

#[test]
fn a_plain_click_picks_one_node_and_adding_toggles_others() {
    let (_, diagram) = general();
    let scene = scene(&diagram);
    let mut selection = Selection::default();

    selection.only(scene, &qn("P::Car"));
    selection.toggle(scene, &qn("P::Truck"));
    assert_eq!(keys(&selection), ["P::Car", "P::Truck"]);

    selection.toggle(scene, &qn("P::Car"));
    assert_eq!(keys(&selection), ["P::Truck"]);

    selection.only(scene, &qn("P::Vehicle"));
    assert_eq!(keys(&selection), ["P::Vehicle"]);
}

#[test]
fn the_interconnection_frame_cannot_be_selected() {
    let (_, diagram) = interconnection();
    let scene = scene(&diagram);
    let mut selection = Selection::default();

    selection.only(scene, &qn("P::Vehicle"));
    assert!(selection.is_empty());

    selection.only(scene, &qn("engine"));
    selection.toggle(scene, &qn("P::Vehicle"));
    assert_eq!(keys(&selection), ["engine"]);
}

#[test]
fn adding_a_node_from_another_box_starts_a_new_selection() {
    let (_, diagram) = interconnection();
    let scene = scene(&diagram);
    let mut selection = Selection::default();

    selection.only(scene, &qn("axle.left"));
    selection.toggle(scene, &qn("axle.right"));
    assert_eq!(keys(&selection), ["axle.left", "axle.right"]);

    selection.toggle(scene, &qn("engine"));
    assert_eq!(keys(&selection), ["engine"]);
}

#[test]
fn a_selection_box_picks_the_nodes_wholly_inside_it() {
    let (_, diagram) = general();
    let scene = scene(&diagram);
    let mut selection = Selection::default();

    selection.boxed(scene, around(scene, &["P::Car", "P::Truck"]));
    assert_eq!(selection.len(), 2);
    assert!(selection.contains(&qn("P::Car")) && selection.contains(&qn("P::Truck")));

    let car = scene.node_bounds(&qn("P::Car")).unwrap();
    selection.boxed(
        scene,
        Rect {
            width: car.width / 2.0,
            ..car
        },
    );
    assert!(selection.is_empty(), "Car is only half inside");
}

#[test]
fn a_selection_box_around_an_opened_part_picks_it_without_what_is_inside() {
    let (_, diagram) = interconnection();
    let scene = scene(&diagram);
    let mut selection = Selection::default();

    selection.boxed(scene, around(scene, &["engine", "axle"]));
    assert_eq!(selection.len(), 2);
    assert!(selection.contains(&qn("engine")) && selection.contains(&qn("axle")));

    selection.boxed(scene, around(scene, &["axle.left", "axle.right"]));
    assert_eq!(selection.len(), 2);
    assert!(selection.contains(&qn("axle.left")) && selection.contains(&qn("axle.right")));
}

#[test]
fn hidden_nodes_drop_out_and_reset_empties_it() {
    let (model, mut diagram) = general();
    let scene = diagram.scene.as_ref().unwrap().clone();
    diagram.selection.only(&scene, &qn("P::Car"));
    diagram.selection.toggle(&scene, &qn("P::Truck"));

    diagram.edit(&model, &Edit::Hide(qn("P::Truck")));
    assert_eq!(keys(&diagram.selection), ["P::Car"]);

    diagram.edit(&model, &Edit::Reset);
    assert!(diagram.selection.is_empty());
}
