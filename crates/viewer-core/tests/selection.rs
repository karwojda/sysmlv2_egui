//! The Selection: which nodes clicks and selection boxes pick, keeping it
//! within one box, and what drops out of it.

mod common;

use viewer_core::editing::Edit;
use viewer_core::open_diagram::{OpenDiagram, Selection};
use viewer_core::scene::Rect;
use viewer_core::{DiagramKind, Model, QualifiedName, Scene};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn general() -> OpenDiagram {
    let model = Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .part_def("P::Car")
        .specializes("P::Vehicle")
        .part_def("P::Truck")
        .specializes("P::Vehicle")
        .build();
    common::open_tool(&model, DiagramKind::General, qn("P"))
}

/// A Vehicle with an engine and an axle, the axle opened in place to show
/// its two wheels.
fn interconnection() -> OpenDiagram {
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
    let mut diagram = common::open_tool(&model, DiagramKind::Interconnection, qn("P::Vehicle"));
    diagram.edit(&Edit::Open("axle".into()));
    diagram
}

fn scene(diagram: &OpenDiagram) -> &Scene {
    diagram.scene().unwrap()
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
    let mut diagram = general();

    diagram.select_only(&qn("P::Car"));
    diagram.toggle_selected(&qn("P::Truck"));
    assert_eq!(keys(diagram.selection()), ["P::Car", "P::Truck"]);

    diagram.toggle_selected(&qn("P::Car"));
    assert_eq!(keys(diagram.selection()), ["P::Truck"]);

    diagram.select_only(&qn("P::Vehicle"));
    assert_eq!(keys(diagram.selection()), ["P::Vehicle"]);
}

#[test]
fn the_interconnection_frame_cannot_be_selected() {
    let mut diagram = interconnection();

    diagram.select_only(&qn("P::Vehicle"));
    assert!(diagram.selection().is_empty());

    diagram.select_only(&qn("engine"));
    diagram.toggle_selected(&qn("P::Vehicle"));
    assert_eq!(keys(diagram.selection()), ["engine"]);
}

#[test]
fn adding_a_node_from_another_box_starts_a_new_selection() {
    let mut diagram = interconnection();

    diagram.select_only(&qn("axle.left"));
    diagram.toggle_selected(&qn("axle.right"));
    assert_eq!(keys(diagram.selection()), ["axle.left", "axle.right"]);

    diagram.toggle_selected(&qn("engine"));
    assert_eq!(keys(diagram.selection()), ["engine"]);
}

#[test]
fn a_selection_box_picks_the_nodes_wholly_inside_it() {
    let mut diagram = general();

    diagram.select_in(around(scene(&diagram), &["P::Car", "P::Truck"]));
    let selection = diagram.selection();
    assert_eq!(selection.len(), 2);
    assert!(selection.contains(&qn("P::Car")) && selection.contains(&qn("P::Truck")));

    let car = scene(&diagram).node_bounds(&qn("P::Car")).unwrap();
    diagram.select_in(Rect {
        width: car.width / 2.0,
        ..car
    });
    assert!(diagram.selection().is_empty(), "Car is only half inside");
}

#[test]
fn a_selection_box_around_an_opened_part_picks_it_without_what_is_inside() {
    let mut diagram = interconnection();

    diagram.select_in(around(scene(&diagram), &["engine", "axle"]));
    let selection = diagram.selection();
    assert_eq!(selection.len(), 2);
    assert!(selection.contains(&qn("engine")) && selection.contains(&qn("axle")));

    diagram.select_in(around(scene(&diagram), &["axle.left", "axle.right"]));
    let selection = diagram.selection();
    assert_eq!(selection.len(), 2);
    assert!(selection.contains(&qn("axle.left")) && selection.contains(&qn("axle.right")));
}

#[test]
fn hidden_nodes_drop_out_and_reset_empties_it() {
    let mut diagram = general();
    diagram.select_only(&qn("P::Car"));
    diagram.toggle_selected(&qn("P::Truck"));

    diagram.edit(&Edit::Hide(qn("P::Truck")));
    assert_eq!(keys(diagram.selection()), ["P::Car"]);

    diagram.edit(&Edit::Reset);
    assert!(diagram.selection().is_empty());
}
