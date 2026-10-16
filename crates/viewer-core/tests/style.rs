//! Seam 1: Style Rules, how they match, and styled scenes and Exports.

mod common;

use std::path::Path;

use tempfile::TempDir;
use viewer_core::scene::{Depicts, Fill, Rgb, Shape};
use viewer_core::style::{self, Condition, Style, StyleError, StyleRule, apply, rule_for};
use viewer_core::{DiagramKind, Model, QualifiedName, Scene, build_scene, svg};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

const RED: Rgb = Rgb(0xf4, 0xcc, 0xcc);
const BLUE: Rgb = Rgb(0xdb, 0xe8, 0xf6);
const GREEN: Rgb = Rgb(0xd9, 0xea, 0xd3);
const NAVY: Rgb = Rgb(0x1f, 0x2a, 0x44);

fn rule(condition: Condition, fill: Rgb) -> StyleRule {
    StyleRule { condition, fill, label: None }
}

fn kind(keyword: &str, fill: Rgb) -> StyleRule {
    rule(Condition::Kind(keyword.to_owned()), fill)
}

fn package(name: &str, fill: Rgb) -> StyleRule {
    rule(Condition::Package(qn(name)), fill)
}

fn metadata(name: &str, fill: Rgb) -> StyleRule {
    rule(Condition::Metadata(qn(name)), fill)
}

/// Nested packages, and parts with and without metadata.
fn model() -> Model {
    Model::builder()
        .package("V")
        .part_def("V::Radio")
        .package("V::Powertrain")
        .part_def("V::Powertrain::Engine")
        .part_def("V::Powertrain::Pump")
        .annotated("V::Safety")
        .annotated("V::Supplier")
        .package("V::Powertrain::Fuel")
        .part_def("V::Powertrain::Fuel::Tank")
        .unsupported("V::Safety", "metadata def")
        .unsupported("V::Supplier", "metadata def")
        .build()
}

fn fill_for(element: &str, levels: &[&[StyleRule]]) -> Option<Rgb> {
    let model = model();
    rule_for(model.element(&qn(element)).unwrap(), levels).map(|r| r.fill)
}

#[test]
fn each_condition_matches_what_it_names() {
    let rules = [kind("part def", BLUE)];
    assert_eq!(fill_for("V::Radio", &[&rules]), Some(BLUE));
    assert_eq!(fill_for("V::Powertrain", &[&rules]), None, "a package isn't a part def");

    let rules = [package("V::Powertrain", GREEN)];
    assert_eq!(fill_for("V::Powertrain::Fuel::Tank", &[&rules]), Some(GREEN), "at any depth");
    assert_eq!(fill_for("V::Radio", &[&rules]), None);
    assert_eq!(fill_for("V::Powertrain", &[&rules]), None, "the package isn't inside itself");

    let rules = [metadata("V::Safety", RED)];
    assert_eq!(fill_for("V::Powertrain::Pump", &[&rules]), Some(RED));
    assert_eq!(fill_for("V::Powertrain::Engine", &[&rules]), None);
}

#[test]
fn a_package_rule_only_matches_whole_names() {
    let rules = [package("V::Power", GREEN)];

    assert_eq!(fill_for("V::Powertrain::Engine", &[&rules]), None);
}

#[test]
fn metadata_beats_package_beats_kind_whatever_the_order() {
    let rules = [kind("part def", BLUE), package("V::Powertrain", GREEN), metadata("V::Safety", RED)];

    assert_eq!(fill_for("V::Powertrain::Pump", &[&rules]), Some(RED));
    assert_eq!(fill_for("V::Powertrain::Engine", &[&rules]), Some(GREEN));
    assert_eq!(fill_for("V::Radio", &[&rules]), Some(BLUE));
}

#[test]
fn the_deeper_package_wins() {
    let rules = [package("V::Powertrain::Fuel", RED), package("V::Powertrain", GREEN)];
    assert_eq!(fill_for("V::Powertrain::Fuel::Tank", &[&rules]), Some(RED));

    let reversed = [package("V::Powertrain", GREEN), package("V::Powertrain::Fuel", RED)];
    assert_eq!(fill_for("V::Powertrain::Fuel::Tank", &[&reversed]), Some(RED));
}

#[test]
fn among_matching_metadata_rules_the_first_listed_wins() {
    let rules = [metadata("V::Supplier", BLUE), metadata("V::Safety", RED)];

    assert_eq!(fill_for("V::Powertrain::Pump", &[&rules]), Some(BLUE));
}

#[test]
fn an_earlier_level_wins_over_a_more_specific_rule_in_a_later_one() {
    let diagram = [package("V::Powertrain", BLUE)];
    let model_style = [metadata("V::Safety", RED), kind("part def", GREEN)];

    assert_eq!(fill_for("V::Powertrain::Pump", &[&diagram, &model_style]), Some(BLUE));
    assert_eq!(fill_for("V::Radio", &[&diagram, &model_style]), Some(GREEN), "falls through to the Model's");
}

#[test]
fn rules_naming_missing_elements_never_match() {
    let rules = [package("V::Gone", RED), metadata("V::Missing", RED)];

    assert_eq!(fill_for("V::Powertrain::Pump", &[&rules]), None);
}

#[test]
fn the_style_file_is_readable_json_that_round_trips() {
    let style = Style::new(vec![
        StyleRule { label: Some("Safety-critical".to_owned()), ..metadata("V::Safety", RED) },
        kind("part def", BLUE),
    ]);

    assert_eq!(
        style.to_text(),
        r##"{
  "format_version": 1,
  "rules": [
    {
      "metadata": "V::Safety",
      "fill": "#f4cccc",
      "label": "Safety-critical"
    },
    {
      "kind": "part def",
      "fill": "#dbe8f6"
    }
  ]
}
"##
    );
    let folder = TempDir::new().unwrap();
    std::fs::create_dir_all(folder.path().join("diagrams")).unwrap();
    std::fs::write(style::path_for(folder.path()), style.to_text()).unwrap();
    assert_eq!(style::load(folder.path()).unwrap(), Some(style));
}

#[test]
fn a_model_without_a_style_file_has_no_model_style() {
    assert_eq!(style::load(TempDir::new().unwrap().path()).unwrap(), None);
}

#[test]
fn a_broken_style_file_is_an_error_naming_it() {
    let folder = TempDir::new().unwrap();
    std::fs::create_dir_all(folder.path().join("diagrams")).unwrap();
    std::fs::write(style::path_for(folder.path()), r##"{"format_version": 1, "rules": [{"kind": "part def", "fill": "red"}]}"##).unwrap();

    let error = style::load(folder.path()).unwrap_err();

    assert!(matches!(error, StyleError::Invalid { .. }));
    assert!(error.to_string().contains("style.json"), "{error}");
    assert!(error.to_string().contains("`red` is not a #rrggbb color"), "{error}");
}

fn general(model: &Model, root: &str) -> Scene {
    build_scene(model, DiagramKind::General, &qn(root)).unwrap()
}

fn rect_fill(scene: &Scene, depicts: &Depicts) -> Fill {
    scene
        .items_depicting(depicts)
        .find_map(|s| match s {
            Shape::Rect { fill, .. } => Some(*fill),
            _ => None,
        })
        .unwrap()
}

fn text_fills(scene: &Scene, depicts: &Depicts) -> Vec<Fill> {
    scene
        .items_depicting(depicts)
        .filter_map(|s| match s {
            Shape::Text { fill, .. } => Some(*fill),
            _ => None,
        })
        .collect()
}

#[test]
fn applying_a_style_fills_matching_nodes_and_nothing_else_changes() {
    let model = model();
    let plain = general(&model, "V::Powertrain");
    let mut styled = plain.clone();
    let rules = [metadata("V::Safety", RED)];

    let used = apply(&mut styled, &model, &[&rules]);

    assert_eq!(used, [&rules[0]]);
    let pump = Depicts::Element(qn("V::Powertrain::Pump"));
    assert_eq!(rect_fill(&styled, &pump), Fill::Color(RED));
    assert_eq!(rect_fill(&styled, &Depicts::Element(qn("V::Powertrain::Engine"))), Fill::Background);
    assert!(text_fills(&styled, &pump).iter().all(|f| *f == Fill::Stroke), "black text on a light fill");
    let geometry = |s: &Scene| s.items.iter().map(|i| (i.depicts.clone(), i.shape.extent())).collect::<Vec<_>>();
    assert_eq!(geometry(&styled), geometry(&plain), "nothing moves");
    assert_eq!((styled.width, styled.height), (plain.width, plain.height));
}

#[test]
fn text_inside_a_dark_fill_turns_white() {
    let model = model();
    let mut scene = general(&model, "V::Powertrain");

    apply(&mut scene, &model, &[&[kind("part def", NAVY)]]);

    let engine = Depicts::Element(qn("V::Powertrain::Engine"));
    assert!(text_fills(&scene, &engine).iter().all(|f| *f == Fill::Background));
    assert!(Rgb(0x1f, 0x2a, 0x44).is_dark() && !RED.is_dark() && !Rgb(0xff, 0xff, 0x00).is_dark());
}

#[test]
fn interconnection_parts_and_ports_are_filled_but_port_labels_stay_black() {
    let model = common::drivetrain();
    let mut scene = build_scene(&model, DiagramKind::Interconnection, &qn("D::Vehicle")).unwrap();

    apply(&mut scene, &model, &[&[kind("part", BLUE), kind("port", NAVY)]]);

    let part = scene.items.iter().find_map(|i| match (&i.depicts, &i.shape) {
        (Depicts::Part { .. }, Shape::Rect { fill, .. }) => Some(*fill),
        _ => None,
    });
    assert_eq!(part, Some(Fill::Color(BLUE)));
    let port_items: Vec<&Shape> =
        scene.items.iter().filter(|i| matches!(i.depicts, Depicts::Port { .. })).map(|i| &i.shape).collect();
    assert!(port_items.iter().any(|s| matches!(s, Shape::Rect { fill: Fill::Color(NAVY), .. })));
    assert!(
        port_items.iter().all(|s| !matches!(s, Shape::Text { fill: Fill::Background, .. })),
        "labels sit outside the square, on the page"
    );
}

#[test]
fn svg_writes_style_colors_and_leaves_unstyled_text_alone() {
    let model = model();
    let mut scene = general(&model, "V::Powertrain");
    apply(&mut scene, &model, &[&[kind("part def", NAVY)]]);

    let svg = svg::render(&scene);

    assert!(svg.contains(r##"fill="#1f2a44""##));
    assert!(svg.contains(r##"fill="#ffffff">Engine</text>"##), "white text on navy");
}

fn assert_golden(name: &str, scene: &Scene) {
    let svg = svg::render(scene);
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden").join(name);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &svg).unwrap();
    }
    let expected = std::fs::read_to_string(&golden)
        .unwrap_or_else(|_| panic!("missing {}; run with UPDATE_GOLDEN=1 to create it", golden.display()));
    assert_eq!(svg, expected, "Export differs from the golden file; if intended, rerun with UPDATE_GOLDEN=1");
}

#[test]
fn a_styled_general_diagram_matches_the_golden_file() {
    let model = common::metadata();
    let mut scene = general(&model, "M");
    apply(&mut scene, &model, &[&[metadata("M::Safety", RED), metadata("M::Supplier", NAVY), kind("part def", BLUE)]]);

    assert_golden("styled-general.svg", &scene);
}

#[test]
fn a_styled_interconnection_diagram_matches_the_golden_file() {
    let model = common::drivetrain();
    let mut scene = build_scene(&model, DiagramKind::Interconnection, &qn("D::Vehicle")).unwrap();
    apply(&mut scene, &model, &[&[package("D::Vehicle", GREEN), kind("part", BLUE), kind("port", NAVY)]]);

    assert_golden("styled-interconnection.svg", &scene);
}
