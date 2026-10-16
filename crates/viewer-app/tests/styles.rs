//! Styles on open Diagrams: the Model Style colors them, and a Diagram's
//! own Style is an unsaved change until saved with it.

use tempfile::TempDir;
use viewer_app::canvas::OpenDiagram;
use viewer_core::scene::{Depicts, Fill, Rgb, Shape};
use viewer_core::style::{Condition, Style, StyleRule};
use viewer_core::{
    DiagramFile, DiagramKind, DiagramStyle, Model, QualifiedName, Scene, diagram_file,
};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn model() -> Model {
    Model::builder().package("P").part_def("P::Engine").build()
}

fn rule(keyword: &str, fill: Rgb) -> StyleRule {
    StyleRule {
        condition: Condition::Kind(keyword.to_owned()),
        fill,
        label: None,
    }
}

fn engine_fill(scene: &Scene) -> Fill {
    scene
        .items_depicting(&Depicts::Element(qn("P::Engine")))
        .find_map(|s| match s {
            Shape::Rect { fill, .. } => Some(*fill),
            _ => None,
        })
        .unwrap()
}

#[test]
fn the_model_style_colors_an_open_diagram_without_making_it_unsaved() {
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));

    diagram.set_model_style(
        &model,
        Some(Style::new(vec![rule("part def", Rgb(9, 9, 9))])),
    );

    assert_eq!(
        engine_fill(diagram.scene.as_ref().unwrap()),
        Fill::Color(Rgb(9, 9, 9))
    );
    assert!(!diagram.dirty);

    diagram.set_model_style(&model, None);
    assert_eq!(
        engine_fill(diagram.scene.as_ref().unwrap()),
        Fill::Background,
        "a removed Style file un-colors"
    );
}

#[test]
fn a_diagram_style_is_an_unsaved_change_saved_and_reopened_with_the_diagram() {
    let folder = TempDir::new().unwrap();
    let model = model();
    let mut diagram = OpenDiagram::open(&model, DiagramKind::General, qn("P"));
    diagram.set_model_style(
        &model,
        Some(Style::new(vec![rule("part def", Rgb(9, 9, 9))])),
    );

    let own = DiagramStyle {
        model_style: true,
        rules: vec![rule("part def", Rgb(200, 0, 0))],
        legend: false,
    };
    diagram.set_style(&model, own.clone());

    assert!(diagram.dirty);
    assert_eq!(
        engine_fill(diagram.scene.as_ref().unwrap()),
        Fill::Color(Rgb(200, 0, 0)),
        "its own rule wins"
    );
    diagram.save_as(folder.path(), "overview").unwrap();
    let text = std::fs::read_to_string(diagram_file::path_for(folder.path(), "overview")).unwrap();
    let reopened = OpenDiagram::open_saved(&model, DiagramFile::parse(&text).unwrap());
    assert_eq!(reopened.style, own);
}
