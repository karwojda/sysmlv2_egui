//! Model-defined Diagrams in the viewer: opening a view, saving it as a
//! Diagram File that refers to the view, and following the view as the
//! Model changes.

use tempfile::TempDir;
use viewer_app::canvas::OpenDiagram;
use viewer_core::editing::Edit;
use viewer_core::model::ExposeScope;
use viewer_core::{
    DiagramError, DiagramFile, DiagramKind, DiagramSource, Model, QualifiedName, diagram_file,
};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

/// A parts tree and a view of it exposing `exposed`.
fn model_exposing(exposed: &[&str]) -> Model {
    let mut builder = Model::builder()
        .package("P")
        .part_def("P::Engine")
        .part("P::vehicle")
        .part("P::vehicle::body")
        .part("P::vehicle::engine")
        .typed_by("P::Engine")
        .view("P::vehicle tree")
        .renders("Views::asTreeDiagram");
    for element in exposed {
        builder = builder.exposes(element, ExposeScope::Element);
    }
    builder.build()
}

fn tree() -> DiagramSource {
    DiagramSource::View {
        view: qn("P::vehicle tree"),
    }
}

#[test]
fn a_view_opens_as_its_model_defined_diagram() {
    let model = model_exposing(&["P::vehicle", "P::vehicle::body"]);

    let diagram = OpenDiagram::open_source(&model, tree());

    assert_eq!(diagram.title(), "vehicle tree (view)");
    assert_eq!(diagram.kind, Some(DiagramKind::General));
    let scene = diagram.scene.unwrap();
    assert!(scene.element_bounds(&qn("P::vehicle::body")).is_some());
    assert!(
        scene.element_bounds(&qn("P::vehicle::engine")).is_none(),
        "not exposed"
    );
}

#[test]
fn a_view_is_saved_under_a_file_safe_version_of_its_name_referring_to_the_view() {
    let folder = TempDir::new().unwrap();
    let model = model_exposing(&["P::vehicle"]);
    let mut diagram = OpenDiagram::open_source(&model, tree());
    assert_eq!(diagram.default_name(), "vehicle-tree");

    diagram.edit(
        &model,
        &Edit::Move {
            element: qn("P::vehicle"),
            dx: 30.0,
            dy: 0.0,
        },
    );
    diagram.save_as(folder.path(), "vehicle-tree").unwrap();

    let text =
        std::fs::read_to_string(diagram_file::path_for(folder.path(), "vehicle-tree")).unwrap();
    assert!(text.contains(r#""view": "P::vehicle tree""#));
    assert!(!text.contains(r#""kind""#) && !text.contains(r#""root""#));
    let reopened = OpenDiagram::open_saved(&model, DiagramFile::parse(&text).unwrap());
    assert_eq!(reopened.title(), "vehicle-tree");
    assert_eq!(reopened.layout, diagram.layout);
}

#[test]
fn a_reloaded_model_redraws_the_view_with_its_new_exposes_keeping_the_layout() {
    let before = model_exposing(&["P::vehicle"]);
    let mut diagram = OpenDiagram::open_source(&before, tree());
    diagram.edit(
        &before,
        &Edit::Move {
            element: qn("P::vehicle"),
            dx: 30.0,
            dy: 0.0,
        },
    );
    let placed = diagram
        .scene
        .as_ref()
        .unwrap()
        .element_bounds(&qn("P::vehicle"))
        .unwrap();

    let after = model_exposing(&["P::vehicle", "P::vehicle::engine"]);
    diagram.refresh(&after);

    let scene = diagram.scene.as_ref().unwrap();
    assert!(
        scene.element_bounds(&qn("P::vehicle::engine")).is_some(),
        "newly exposed"
    );
    let now = scene.element_bounds(&qn("P::vehicle")).unwrap();
    assert_eq!(
        (now.x, now.y),
        (placed.x, placed.y),
        "the saved position holds"
    );
}

#[test]
fn layout_entries_of_elements_no_longer_exposed_are_kept() {
    let before = model_exposing(&["P::vehicle", "P::vehicle::body"]);
    let mut diagram = OpenDiagram::open_source(&before, tree());
    diagram.edit(
        &before,
        &Edit::Move {
            element: qn("P::vehicle::body"),
            dx: 10.0,
            dy: 10.0,
        },
    );

    let after = model_exposing(&["P::vehicle"]);
    diagram.refresh(&after);
    diagram.edit(&after, &Edit::Reset);

    assert!(
        diagram.layout.nodes.contains_key(&qn("P::vehicle::body")),
        "Reset keeps entries it can't place"
    );
}

#[test]
fn a_diagram_whose_view_is_gone_says_so() {
    let model = model_exposing(&["P::vehicle"]);
    let file = DiagramFile::of(
        "old",
        DiagramSource::View {
            view: qn("P::renamed"),
        },
    );

    let diagram = OpenDiagram::open_saved(&model, file);

    let error = diagram.scene.unwrap_err();
    assert!(matches!(error, DiagramError::View(_)));
    assert_eq!(error.to_string(), "`P::renamed` is not a view in the Model");
}
