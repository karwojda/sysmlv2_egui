//! Seam 1: Computed Values shown beside the expressions they come from.

mod common;

use viewer_core::properties::properties;
use viewer_core::scene::{Depicts, Shape};
use viewer_core::{DiagramKind, Model, QualifiedName, Scene, build_scene};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

fn texts_of(scene: &Scene, element: &str) -> Vec<String> {
    scene
        .items_depicting(&Depicts::Element(qn(element)))
        .filter_map(|shape| match shape {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn an_expression_row_shows_what_it_comes_to() {
    let scene = build_scene(&common::analysis(), DiagramKind::General, &qn("A")).unwrap();

    assert!(
        texts_of(&scene, "A::car")
            .contains(&"mass = engine.mass + sum(wheels.mass) = 230.0".to_owned()),
        "{:?}",
        texts_of(&scene, "A::car")
    );
}

/// A part with attribute `a` written as `value`, computed to `computed`.
fn one(value: &str, computed: &[&str]) -> Vec<String> {
    let mut b = Model::builder()
        .package("P")
        .part("P::p")
        .attribute("P::p::a")
        .value(value);
    for c in computed {
        b = b.computed(c);
    }
    let scene = build_scene(&b.build(), DiagramKind::General, &qn("P")).unwrap();
    texts_of(&scene, "P::p")
}

#[test]
fn a_literal_row_is_left_as_written() {
    for literal in ["180.0", "-1.5", "4", "true", "\"x\"", "1650 [kg]"] {
        let rows = one(literal, &["999.0"]);
        assert!(
            rows.contains(&format!("a = {literal}")),
            "{literal}: {rows:?}"
        );
    }
}

#[test]
fn several_instances_show_their_distinct_values() {
    assert!(one("x * 2", &["44.0"]).contains(&"a = x * 2 = 44.0".to_owned()));
    assert!(one("x * 2", &["44.0", "50.0"]).contains(&"a = x * 2 = 44.0, 50.0".to_owned()));
}

#[test]
fn a_computed_quantity_follows_its_expression_with_its_unit() {
    assert!(
        one("volume * density", &["7.85 [kg]"])
            .contains(&"a = volume * density = 7.85 [kg]".to_owned())
    );
    assert!(
        one("1.0e6 [mm^3]", &["0.001 [m³]"]).contains(&"a = 1.0e6 [mm^3]".to_owned()),
        "a quantity literal says itself"
    );
}

#[test]
fn an_expression_without_a_computed_value_keeps_just_its_expression() {
    assert!(one("x * 2", &[]).contains(&"a = x * 2".to_owned()));
}

#[test]
fn properties_show_the_computed_value() {
    let props = properties(&common::analysis(), &qn("A::car::mass")).unwrap();

    assert!(
        props
            .rows
            .iter()
            .any(|p| p.label == "Computed" && p.value == "230.0")
    );
}
