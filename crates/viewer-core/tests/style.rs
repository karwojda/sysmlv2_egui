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
    StyleRule {
        condition,
        fill,
        label: None,
    }
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
    rule_for(&model, model.element(&qn(element)).unwrap(), levels).map(|r| r.fill)
}

#[test]
fn each_condition_matches_what_it_names() {
    let rules = [kind("part def", BLUE)];
    assert_eq!(fill_for("V::Radio", &[&rules]), Some(BLUE));
    assert_eq!(
        fill_for("V::Powertrain", &[&rules]),
        None,
        "a package isn't a part def"
    );

    let rules = [package("V::Powertrain", GREEN)];
    assert_eq!(
        fill_for("V::Powertrain::Fuel::Tank", &[&rules]),
        Some(GREEN),
        "at any depth"
    );
    assert_eq!(fill_for("V::Radio", &[&rules]), None);
    assert_eq!(
        fill_for("V::Powertrain", &[&rules]),
        None,
        "the package isn't inside itself"
    );

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
    let rules = [
        kind("part def", BLUE),
        package("V::Powertrain", GREEN),
        metadata("V::Safety", RED),
    ];

    assert_eq!(fill_for("V::Powertrain::Pump", &[&rules]), Some(RED));
    assert_eq!(fill_for("V::Powertrain::Engine", &[&rules]), Some(GREEN));
    assert_eq!(fill_for("V::Radio", &[&rules]), Some(BLUE));
}

#[test]
fn the_deeper_package_wins() {
    let rules = [
        package("V::Powertrain::Fuel", RED),
        package("V::Powertrain", GREEN),
    ];
    assert_eq!(fill_for("V::Powertrain::Fuel::Tank", &[&rules]), Some(RED));

    let reversed = [
        package("V::Powertrain", GREEN),
        package("V::Powertrain::Fuel", RED),
    ];
    assert_eq!(
        fill_for("V::Powertrain::Fuel::Tank", &[&reversed]),
        Some(RED)
    );
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

    assert_eq!(
        fill_for("V::Powertrain::Pump", &[&diagram, &model_style]),
        Some(BLUE)
    );
    assert_eq!(
        fill_for("V::Radio", &[&diagram, &model_style]),
        Some(GREEN),
        "falls through to the Model's"
    );
}

#[test]
fn rules_naming_missing_elements_never_match() {
    let rules = [package("V::Gone", RED), metadata("V::Missing", RED)];

    assert_eq!(fill_for("V::Powertrain::Pump", &[&rules]), None);
}

#[test]
fn the_style_file_is_readable_json_that_round_trips() {
    let style = Style::new(vec![
        StyleRule {
            label: Some("Safety-critical".to_owned()),
            ..metadata("V::Safety", RED)
        },
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
    std::fs::write(
        style::path_for(folder.path()),
        r##"{"format_version": 1, "rules": [{"kind": "part def", "fill": "red"}]}"##,
    )
    .unwrap();

    let error = style::load(folder.path()).unwrap_err();

    assert!(matches!(error, StyleError::Invalid { .. }));
    assert!(error.to_string().contains("style.json"), "{error}");
    assert!(
        error.to_string().contains("`red` is not a #rrggbb color"),
        "{error}"
    );
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
    assert_eq!(
        rect_fill(&styled, &Depicts::Element(qn("V::Powertrain::Engine"))),
        Fill::Background
    );
    assert!(
        text_fills(&styled, &pump)
            .iter()
            .all(|f| *f == Fill::Stroke),
        "black text on a light fill"
    );
    let geometry = |s: &Scene| {
        s.items
            .iter()
            .map(|i| (i.depicts.clone(), i.shape.extent()))
            .collect::<Vec<_>>()
    };
    assert_eq!(geometry(&styled), geometry(&plain), "nothing moves");
    assert_eq!((styled.width, styled.height), (plain.width, plain.height));
}

#[test]
fn text_inside_a_dark_fill_turns_white() {
    let model = model();
    let mut scene = general(&model, "V::Powertrain");

    apply(&mut scene, &model, &[&[kind("part def", NAVY)]]);

    let engine = Depicts::Element(qn("V::Powertrain::Engine"));
    assert!(
        text_fills(&scene, &engine)
            .iter()
            .all(|f| *f == Fill::Background)
    );
    assert!(Rgb(0x1f, 0x2a, 0x44).is_dark() && !RED.is_dark() && !Rgb(0xff, 0xff, 0x00).is_dark());
}

#[test]
fn interconnection_parts_and_ports_are_filled_but_port_labels_stay_black() {
    let model = common::drivetrain();
    let mut scene = build_scene(&model, DiagramKind::Interconnection, &qn("D::Vehicle")).unwrap();

    apply(
        &mut scene,
        &model,
        &[&[kind("part", BLUE), kind("port", NAVY)]],
    );

    let part = scene
        .items
        .iter()
        .find_map(|i| match (&i.depicts, &i.shape) {
            (Depicts::Part { .. }, Shape::Rect { fill, .. }) => Some(*fill),
            _ => None,
        });
    assert_eq!(part, Some(Fill::Color(BLUE)));
    let port_items: Vec<&Shape> = scene
        .items
        .iter()
        .filter(|i| matches!(i.depicts, Depicts::Port { .. }))
        .map(|i| &i.shape)
        .collect();
    assert!(port_items.iter().any(|s| matches!(
        s,
        Shape::Rect {
            fill: Fill::Color(NAVY),
            ..
        }
    )));
    assert!(
        port_items.iter().all(|s| !matches!(
            s,
            Shape::Text {
                fill: Fill::Background,
                ..
            }
        )),
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
    assert!(
        svg.contains(r##"fill="#ffffff">Engine</text>"##),
        "white text on navy"
    );
}

fn assert_golden(name: &str, scene: &Scene) {
    let svg = svg::render(scene);
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name);
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &svg).unwrap();
    }
    let expected = std::fs::read_to_string(&golden).unwrap_or_else(|_| {
        panic!(
            "missing {}; run with UPDATE_GOLDEN=1 to create it",
            golden.display()
        )
    });
    assert_eq!(
        svg, expected,
        "Export differs from the golden file; if intended, rerun with UPDATE_GOLDEN=1"
    );
}

#[test]
fn a_styled_general_diagram_matches_the_golden_file() {
    let model = common::metadata();
    let mut scene = general(&model, "M");
    apply(
        &mut scene,
        &model,
        &[&[
            metadata("M::Safety", RED),
            metadata("M::Supplier", NAVY),
            kind("part def", BLUE),
        ]],
    );

    assert_golden("styled-general.svg", &scene);
}

#[test]
fn a_styled_interconnection_diagram_matches_the_golden_file() {
    let model = common::drivetrain();
    let mut scene = build_scene(&model, DiagramKind::Interconnection, &qn("D::Vehicle")).unwrap();
    apply(
        &mut scene,
        &model,
        &[&[
            package("D::Vehicle", GREEN),
            kind("part", BLUE),
            kind("port", NAVY),
        ]],
    );

    assert_golden("styled-interconnection.svg", &scene);
}

#[test]
fn a_diagram_style_layers_over_the_model_style_or_switches_it_off() {
    use viewer_core::DiagramStyle;
    let model = model();
    let model_style = Style::new(vec![kind("part def", BLUE)]);
    let mut diagram = DiagramStyle {
        rules: vec![package("V::Powertrain::Fuel", RED)],
        ..DiagramStyle::default()
    };
    let fill_of = |scene: &Scene, e: &str| rect_fill(scene, &Depicts::Element(qn(e)));

    let mut scene = general(&model, "V::Powertrain");
    viewer_core::style::apply_styles(&mut scene, &model, Some(&model_style), &diagram);
    assert_eq!(
        fill_of(&scene, "V::Powertrain::Engine"),
        Fill::Color(BLUE),
        "the Model's rule"
    );

    let mut fuel = general(&model, "V::Powertrain::Fuel");
    viewer_core::style::apply_styles(&mut fuel, &model, Some(&model_style), &diagram);
    assert_eq!(
        fill_of(&fuel, "V::Powertrain::Fuel::Tank"),
        Fill::Color(RED),
        "the Diagram's rule wins"
    );

    diagram.model_style = false;
    let mut scene = general(&model, "V::Powertrain");
    viewer_core::style::apply_styles(&mut scene, &model, Some(&model_style), &diagram);
    assert_eq!(
        fill_of(&scene, "V::Powertrain::Engine"),
        Fill::Background,
        "Model Style off"
    );
}

fn legend_texts(scene: &Scene) -> Vec<String> {
    scene
        .items_depicting(&Depicts::Legend)
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

fn with_legend(rules: Vec<StyleRule>, model_rules: Vec<StyleRule>) -> (Model, Scene, Scene) {
    use viewer_core::DiagramStyle;
    let model = model();
    let plain = general(&model, "V::Powertrain");
    let mut scene = plain.clone();
    let diagram = DiagramStyle {
        rules,
        legend: true,
        ..DiagramStyle::default()
    };
    viewer_core::style::apply_styles(&mut scene, &model, Some(&Style::new(model_rules)), &diagram);
    (model, plain, scene)
}

#[test]
fn the_legend_lists_only_rules_that_colored_something_in_rule_order_diagram_first() {
    let (_, _, scene) = with_legend(
        vec![StyleRule {
            label: Some("Safety-critical".to_owned()),
            ..metadata("V::Safety", RED)
        }],
        vec![
            kind("attribute def", GREEN),
            kind("part def", BLUE),
            package("V::Elsewhere", NAVY),
        ],
    );

    assert_eq!(legend_texts(&scene), ["Safety-critical", "«part def»"]);
}

#[test]
fn legend_labels_default_to_the_condition() {
    let (_, _, scene) = with_legend(
        vec![package("V::Powertrain", GREEN), metadata("V::Safety", NAVY)],
        vec![],
    );

    assert_eq!(
        legend_texts(&scene),
        ["Powertrain", "@Safety"],
        "kind rules read «part def»"
    );
}

#[test]
fn the_legend_sits_bottom_right_outside_the_nodes_and_grows_the_page() {
    let (_, plain, scene) = with_legend(vec![], vec![kind("part def", BLUE)]);

    let legend = scene
        .items_depicting(&Depicts::Legend)
        .find_map(|s| match s {
            Shape::Rect { rect, .. } => Some(*rect),
            _ => None,
        })
        .unwrap();
    let nodes_bottom = scene
        .items
        .iter()
        .filter(|i| i.depicts != Depicts::Legend)
        .map(|i| i.shape.extent().bottom())
        .fold(0.0, f32::max);
    assert!(legend.y > nodes_bottom, "below everything else");
    assert!(legend.bottom() < scene.y + scene.height, "on the page");
    assert!(
        legend.right() <= scene.x + scene.width - 19.0,
        "right-aligned within the margin"
    );
    assert!(scene.height > plain.height);
    let others = |s: &Scene| {
        s.items
            .iter()
            .filter(|i| i.depicts != Depicts::Legend)
            .map(|i| i.shape.extent())
            .collect::<Vec<_>>()
    };
    assert_eq!(others(&scene), others(&plain), "nothing else moves");
}

#[test]
fn no_legend_without_the_switch_or_without_colors() {
    use viewer_core::DiagramStyle;
    let model = model();
    let mut off = general(&model, "V::Powertrain");
    viewer_core::style::apply_styles(
        &mut off,
        &model,
        Some(&Style::new(vec![kind("part def", BLUE)])),
        &DiagramStyle::default(),
    );
    assert!(off.items_depicting(&Depicts::Legend).next().is_none());

    let (_, plain, nothing) = with_legend(vec![], vec![kind("attribute def", BLUE)]);
    assert!(nothing.items_depicting(&Depicts::Legend).next().is_none());
    assert_eq!(nothing.height, plain.height);
}

#[test]
fn a_styled_diagram_with_a_legend_matches_the_golden_file() {
    use viewer_core::DiagramStyle;
    let model = common::metadata();
    let mut scene = general(&model, "M");
    let model_style = Style::new(vec![
        StyleRule {
            label: Some("Safety-critical".to_owned()),
            ..metadata("M::Safety", RED)
        },
        metadata("M::Supplier", NAVY),
        kind("part def", BLUE),
    ]);
    viewer_core::style::apply_styles(
        &mut scene,
        &model,
        Some(&model_style),
        &DiagramStyle {
            legend: true,
            ..DiagramStyle::default()
        },
    );

    assert_golden("styled-legend.svg", &scene);
}

#[test]
fn rules_move_up_and_down_and_stop_at_the_ends() {
    let mut rules = vec![
        kind("part def", BLUE),
        kind("port", RED),
        kind("part", GREEN),
    ];
    let order = |r: &[StyleRule]| r.iter().map(|r| r.fill).collect::<Vec<_>>();

    viewer_core::style::move_up(&mut rules, 2);
    assert_eq!(order(&rules), [BLUE, GREEN, RED]);
    viewer_core::style::move_down(&mut rules, 0);
    assert_eq!(order(&rules), [GREEN, BLUE, RED]);
    viewer_core::style::move_up(&mut rules, 0);
    viewer_core::style::move_down(&mut rules, 2);
    viewer_core::style::move_down(&mut rules, 9);
    assert_eq!(
        order(&rules),
        [GREEN, BLUE, RED],
        "no change at the ends or out of range"
    );
}

#[test]
fn a_condition_knows_whether_what_it_names_is_in_the_model() {
    let model = model();

    assert!(Condition::Package(qn("V::Powertrain")).exists_in(&model));
    assert!(!Condition::Package(qn("V::Gone")).exists_in(&model));
    assert!(Condition::Metadata(qn("V::Safety")).exists_in(&model));
    assert!(Condition::Kind("part def".to_owned()).exists_in(&model));
    assert!(!Condition::Kind("widget".to_owned()).exists_in(&model));
}
