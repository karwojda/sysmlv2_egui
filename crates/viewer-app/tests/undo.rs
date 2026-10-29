//! Undo and redo: every Layout edit is a step, a whole drag is one, and
//! the unsaved mark follows whether the Diagram is back where it was saved.

use tempfile::TempDir;
use viewer_app::canvas::OpenDiagram;
use viewer_core::editing::Edit;
use viewer_core::{DiagramKind, Layout, Model, QualifiedName};

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

fn move_car(dx: f32) -> Edit {
    Edit::Move {
        elements: vec![qn("P::Car")],
        dx,
        dy: 0.0,
    }
}

fn car_x(diagram: &OpenDiagram) -> f32 {
    diagram.layout.nodes[&qn("P::Car")].x
}

#[test]
fn undo_goes_back_to_the_layout_before_the_edit_and_redo_forward_again() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    diagram.edit(&model, &move_car(40.0));
    let moved = diagram.layout.clone();

    diagram.undo(&model);

    assert_eq!(diagram.layout, Layout::default());
    assert!(!diagram.dirty, "back at the Diagram as opened");
    assert!(!diagram.can_undo());
    assert!(diagram.can_redo());

    diagram.redo(&model);

    assert_eq!(diagram.layout, moved);
    assert!(diagram.dirty);
    assert!(!diagram.can_redo());
}

#[test]
fn undo_redraws_the_scene() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    let bounds = |d: &OpenDiagram| {
        d.scene
            .as_ref()
            .unwrap()
            .node_bounds(&qn("P::Car"))
            .unwrap()
    };
    let before = bounds(&diagram);
    diagram.edit(&model, &move_car(40.0));

    diagram.undo(&model);

    assert_eq!(bounds(&diagram), before);
}

#[test]
fn each_edit_outside_a_drag_is_its_own_step() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    diagram.edit(&model, &move_car(40.0));
    let first = car_x(&diagram);
    diagram.edit(&model, &move_car(10.0));

    diagram.undo(&model);

    assert_eq!(car_x(&diagram), first);
}

#[test]
fn a_whole_drag_is_one_step() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));

    diagram.begin_drag();
    for _ in 0..5 {
        diagram.edit(&model, &move_car(8.0));
    }
    diagram.end_drag();
    diagram.undo(&model);

    assert_eq!(diagram.layout, Layout::default());
    assert!(!diagram.can_undo());
}

#[test]
fn a_drag_that_changes_nothing_records_no_step() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    diagram.edit(&model, &move_car(40.0));

    diagram.begin_drag();
    diagram.end_drag();
    diagram.edit(&model, &Edit::ShowAll);
    diagram.undo(&model);

    assert_eq!(diagram.layout, Layout::default());
}

#[test]
fn a_new_edit_drops_what_could_be_redone() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    diagram.edit(&model, &move_car(40.0));
    diagram.undo(&model);

    diagram.edit(&model, &Edit::Hide(qn("P::Vehicle")));

    assert!(!diagram.can_redo());
}

#[test]
fn reset_can_be_undone() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    diagram.edit(&model, &move_car(40.0));
    diagram.edit(&model, &Edit::Hide(qn("P::Vehicle")));
    let arranged = diagram.layout.clone();
    diagram.edit(&model, &Edit::Reset);

    diagram.undo(&model);

    assert_eq!(diagram.layout, arranged);
}

#[test]
fn the_history_survives_saving_and_the_saved_layout_is_not_unsaved() {
    let folder = TempDir::new().unwrap();
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    diagram.edit(&model, &move_car(40.0));
    diagram.save_as(folder.path(), "overview").unwrap();

    diagram.undo(&model);
    assert!(diagram.dirty, "the saved Diagram had Car moved");

    diagram.redo(&model);
    assert!(!diagram.dirty, "back at the saved Layout");
}

#[test]
fn undo_with_nothing_to_undo_changes_nothing() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));

    diagram.undo(&model);
    diagram.redo(&model);

    assert_eq!(diagram.layout, Layout::default());
    assert!(!diagram.dirty);
}
