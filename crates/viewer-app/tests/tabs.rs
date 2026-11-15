//! A tab's title: the saved name, or the root and Diagram Kind for an ad
//! hoc Diagram, marked while there are unsaved changes.

use tempfile::TempDir;
use viewer_app::canvas::Tab;
use viewer_core::editing::Edit;
use viewer_core::{DiagramKind, DiagramSource, Model, OpenDiagram, QualifiedName};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn model() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .part("P::Vehicle::engine")
        .view("P::vehicle tree")
        .renders("Views::asTreeDiagram")
        .build()
}

#[test]
fn an_ad_hoc_diagram_is_titled_by_its_root_and_kind() {
    let model = model();
    let general = Tab::new(OpenDiagram::open(&model, DiagramKind::General, qn("P")));
    let interconnection = Tab::new(OpenDiagram::open(
        &model,
        DiagramKind::Interconnection,
        qn("P::Vehicle"),
    ));
    let view = Tab::new(OpenDiagram::open_source(
        &model,
        DiagramSource::View {
            view: qn("P::vehicle tree"),
        },
    ));

    assert_eq!(general.title(), "P (General)");
    assert_eq!(interconnection.title(), "Vehicle (Interconnection)");
    assert_eq!(view.title(), "vehicle tree (view)");
}

#[test]
fn an_unsaved_change_marks_the_title_until_saved_under_its_name() {
    let model = model();
    let folder = TempDir::new().unwrap();
    let mut tab = Tab::new(OpenDiagram::open(&model, DiagramKind::General, qn("P")));

    tab.diagram.edit(&model, &Edit::Hide(qn("P::Vehicle")));
    assert_eq!(tab.title(), "P (General) •");

    tab.diagram.save_as(folder.path(), "overview").unwrap();
    assert_eq!(tab.title(), "overview");
}
