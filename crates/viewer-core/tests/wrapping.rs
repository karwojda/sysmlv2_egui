//! Long compartment rows wrap at the width `doc` text uses, so a node
//! keeps its normal width.

mod common;

use common::CONTINUED;
use viewer_core::diagram::build_scene;
use viewer_core::scene::{Depicts, Scene, Shape};
use viewer_core::{DiagramKind, Model, QualifiedName};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

/// The width `doc` text wraps at, in characters.
const WIDTH: usize = 40;

const CONSTRAINT: &str =
    "unit.minOperatingTemperature <= -10 ['°C'] and unit.maxOperatingTemperature >= 40 ['°C']";

fn texts_of(scene: &Scene, element: &str) -> Vec<String> {
    scene
        .items_depicting(&Depicts::Element(qn(element)))
        .filter_map(|shape| match shape {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

/// The lines after the compartment titled `title`, up to the next title.
fn compartment(texts: &[String], title: &str) -> Vec<String> {
    let at = texts.iter().position(|t| t == title).expect(title);
    texts[at + 1..]
        .iter()
        .take_while(|t| !["attributes", "constraints", "doc", "parts"].contains(&t.as_str()))
        .cloned()
        .collect()
}

/// A wrapped row's lines joined back into the row.
fn rejoined(lines: &[String]) -> String {
    common::rejoined(lines.to_vec()).join("\n")
}

fn requirements() -> Model {
    Model::builder()
        .package("P")
        .requirement_def("P::R")
        .requirement("P::P1")
        .typed_by("P::R")
        .doc("Detect a vehicle approaching from behind the rider early enough to react.")
        .requirement("P::S11")
        .typed_by("P::R")
        .doc("Work from -10 °C to 40 °C.")
        .requires(CONSTRAINT)
        .build()
}

#[test]
fn a_long_constraint_wraps_and_the_node_keeps_its_neighbours_width() {
    let scene = build_scene(&requirements(), DiagramKind::General, &qn("P")).unwrap();

    let lines = compartment(&texts_of(&scene, "P::S11"), "constraints");
    assert!(lines.len() > 1, "{lines:?}");
    for line in &lines {
        assert!(line.chars().count() <= WIDTH, "{line:?}");
    }
    assert_eq!(rejoined(&lines), format!("require {{{CONSTRAINT}}}"));
    assert!(
        lines[1..].iter().all(|l| l.starts_with(CONTINUED)),
        "continuation lines are indented: {lines:?}"
    );
    let width = |name: &str| scene.element_bounds(&qn(name)).unwrap().width;
    assert!(
        // Lines of as many characters; glyphs differ in width.
        width("P::S11") <= width("P::P1") * 1.15,
        "{} vs {}: {lines:?} / {:?}",
        width("P::S11"),
        width("P::P1"),
        texts_of(&scene, "P::P1")
    );
}

#[test]
fn lines_break_after_operators_and_never_inside_a_unit_or_name() {
    let scene = build_scene(&requirements(), DiagramKind::General, &qn("P")).unwrap();

    let lines = compartment(&texts_of(&scene, "P::S11"), "constraints");
    for line in &lines {
        let line = line.trim_start_matches(CONTINUED);
        assert!(!line.starts_with("and"), "{lines:?}");
        assert_eq!(
            line.matches('[').count(),
            line.matches(']').count(),
            "{line:?}"
        );
    }
    let words: Vec<&str> = lines
        .iter()
        .flat_map(|l| l.trim_start_matches(CONTINUED).split(' '))
        .collect();
    for name in [
        "unit.minOperatingTemperature",
        "unit.maxOperatingTemperature",
    ] {
        assert!(
            words.iter().any(|w| w.contains(name)),
            "{name} whole: {words:?}"
        );
    }
}

#[test]
fn a_long_attribute_row_with_a_computed_value_wraps_the_same_way() {
    let value = "engine.mass + body.mass + fuelTank.mass + sum(wheels.mass)";
    let model = Model::builder()
        .package("P")
        .part_def("P::V")
        .attribute("P::V::mass")
        .value(value)
        .computed("1363.0")
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    let lines = compartment(&texts_of(&scene, "P::V"), "attributes");
    assert!(lines.len() > 1, "{lines:?}");
    for line in &lines {
        assert!(line.chars().count() <= WIDTH, "{line:?}");
    }
    assert_eq!(rejoined(&lines), format!("mass = {value} = 1363.0"));
}

#[test]
fn a_token_longer_than_the_width_is_not_broken() {
    let long = "maxOperatingTemperature:ThermodynamicTemperatureValue";
    let model = Model::builder()
        .package("P")
        .part_def("P::V")
        .attribute("P::V::maxOperatingTemperature")
        .typed_by("ISQ::ThermodynamicTemperatureValue")
        .attribute("P::V::short")
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    let lines = compartment(&texts_of(&scene, "P::V"), "attributes");
    assert_eq!(lines, [long, "short"]);
}

/// The `attributes` lines of a part def with one attribute `x` whose
/// value is written `value`.
fn value_lines(value: &str) -> Vec<String> {
    let model = Model::builder()
        .package("P")
        .part_def("P::V")
        .attribute("P::V::x")
        .value(value)
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();
    compartment(&texts_of(&scene, "P::V"), "attributes")
}

#[test]
fn words_carried_to_a_line_are_measured_again() {
    let lines = value_lines("aaaaaaaaaaaaaaaa bbbbbbbbbbbbbbbbbb cccc d");

    for line in &lines {
        assert!(line.chars().count() <= WIDTH, "{lines:?}");
    }
    assert!(
        lines[0].chars().count() > WIDTH / 2,
        "no near-empty first line: {lines:?}"
    );
}

#[test]
fn a_value_keeps_its_unit_and_a_row_its_mark() {
    let lines =
        value_lines("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa + -10 ['°C'] and bbbbbbbbbbbbbbbbbbb ✓");

    let lines: Vec<&str> = lines
        .iter()
        .map(|l| l.trim_start_matches(CONTINUED))
        .collect();
    assert!(lines.iter().any(|l| l.contains("-10 ['°C']")), "{lines:?}");
    assert!(lines.iter().all(|l| *l != "✓"), "{lines:?}");
}

#[test]
fn an_unclosed_quote_does_not_stop_a_row_wrapping() {
    let lines = value_lines("'unclosed aaaaaaaaaa bbbbbbbbbb cccccccccc dddddddddd eeeeeeeeee");

    assert!(lines.len() > 1, "{lines:?}");
}
