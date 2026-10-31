//! Inherited Members in General Diagrams: rows marked `^`, in italics, in
//! the compartments of the element that has them, never nodes of their own.

use viewer_core::diagram::{build_general_scene, build_scene};
use viewer_core::diagram_file::Layout;
use viewer_core::model::{ElementKind, ExposeScope, ModelBuilder};
use viewer_core::scene::{Depicts, Scene, Shape};
use viewer_core::view::{ViewContents, resolve};
use viewer_core::{DiagramKind, Model, QualifiedName};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

/// The issue's snippet: `Design :> Box`, and `specialised : Design` with a
/// part of its own.
fn snippet() -> ModelBuilder {
    Model::builder()
        .package("P")
        .port_def("P::PD")
        .part_def("P::Inner")
        .port("P::Inner::p")
        .typed_by("P::PD")
        .part_def("P::Box")
        .port("P::Box::outer")
        .typed_by("P::PD")
        .attribute("P::Box::mass")
        .part_def("P::Design")
        .specializes("P::Box")
        .part("P::Design::inner")
        .typed_by("P::Inner")
        .part("P::specialised")
        .typed_by("P::Design")
        .part("P::specialised::own")
}

/// Each text `element`'s node shows, and whether it is in italics.
fn texts_of(scene: &Scene, element: &str) -> Vec<(String, bool)> {
    scene
        .items_depicting(&Depicts::Element(qn(element)))
        .filter_map(|shape| match shape {
            Shape::Text { text, font, .. } => Some((text.clone(), font.italic)),
            _ => None,
        })
        .collect()
}

/// The rows of the compartment titled `title`.
fn compartment(texts: &[(String, bool)], title: &str) -> Vec<(String, bool)> {
    const TITLES: [&str; 6] = [
        "attributes",
        "ports",
        "parts",
        "connections",
        "requirements",
        "constraints",
    ];
    let Some(at) = texts.iter().position(|(t, _)| t == title) else {
        return Vec::new();
    };
    texts[at + 1..]
        .iter()
        .take_while(|(t, _)| !TITLES.contains(&t.as_str()))
        .cloned()
        .collect()
}

fn row(text: &str, italic: bool) -> (String, bool) {
    (text.to_owned(), italic)
}

fn nodes(scene: &Scene) -> Vec<String> {
    scene
        .items
        .iter()
        .filter_map(|i| match (&i.depicts, &i.shape) {
            (Depicts::Element(e), Shape::Rect { .. }) => Some(e.to_string()),
            _ => None,
        })
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[test]
fn a_usage_lists_what_it_has_through_its_definition_after_its_own_members() {
    let model = snippet().build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    let texts = texts_of(&scene, "P::specialised");

    assert_eq!(
        compartment(&texts, "parts"),
        [row("own", false), row("^inner:Inner", true)]
    );
    assert_eq!(compartment(&texts, "ports"), [row("^outer:PD", true)]);
    assert_eq!(compartment(&texts, "attributes"), [row("^mass", true)]);
}

#[test]
fn members_of_farther_definitions_follow_nearer_ones() {
    let model = Model::builder()
        .package("P")
        .part_def("P::A")
        .attribute("P::A::a")
        .part_def("P::B")
        .specializes("P::A")
        .attribute("P::B::b")
        .part_def("P::C")
        .specializes("P::B")
        .attribute("P::C::c")
        .part("P::x")
        .typed_by("P::C")
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    let texts = texts_of(&scene, "P::x");

    assert_eq!(
        compartment(&texts, "attributes"),
        [row("^c", true), row("^b", true), row("^a", true)]
    );
}

#[test]
fn a_redefined_member_hides_the_one_it_redefines() {
    let model = snippet().attribute("P::Design::mass").build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    let design = texts_of(&scene, "P::Design");
    let specialised = texts_of(&scene, "P::specialised");

    assert_eq!(compartment(&design, "attributes"), [row("mass", false)]);
    assert_eq!(
        compartment(&specialised, "attributes"),
        [row("^mass", true)]
    );
}

#[test]
fn a_definition_reached_twice_contributes_once() {
    let model = Model::builder()
        .package("P")
        .part_def("P::A")
        .attribute("P::A::a")
        .part_def("P::B")
        .specializes("P::A")
        .part_def("P::C")
        .specializes("P::A")
        .part_def("P::D")
        .specializes("P::B")
        .specializes("P::C")
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    assert_eq!(
        compartment(&texts_of(&scene, "P::D"), "attributes"),
        [row("^a", true)]
    );
}

#[test]
fn a_definition_lists_what_it_has_through_the_definitions_it_specializes() {
    let model = snippet().build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    let texts = texts_of(&scene, "P::Design");

    assert_eq!(compartment(&texts, "ports"), [row("^outer:PD", true)]);
    assert_eq!(compartment(&texts, "attributes"), [row("^mass", true)]);
    assert_eq!(compartment(&texts, "parts"), [row("inner:Inner", false)]);
}

#[test]
fn a_general_diagram_of_a_usage_draws_the_usage_on_top_of_its_own_members() {
    let model = snippet().build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P::specialised")).unwrap();

    assert_eq!(nodes(&scene), ["P::specialised", "P::specialised::own"]);
    let texts = texts_of(&scene, "P::specialised");
    assert_eq!(compartment(&texts, "ports"), [row("^outer:PD", true)]);
    assert_eq!(compartment(&texts, "attributes"), [row("^mass", true)]);
    // `own` is a node here, so not a row.
    assert_eq!(compartment(&texts, "parts"), [row("^inner:Inner", true)]);
}

#[test]
fn a_cycle_of_specializations_ends() {
    let model = Model::builder()
        .package("P")
        .part_def("P::A")
        .specializes("P::B")
        .attribute("P::A::a")
        .part_def("P::B")
        .specializes("P::A")
        .attribute("P::B::b")
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    assert_eq!(
        compartment(&texts_of(&scene, "P::A"), "attributes"),
        [row("a", false), row("^b", true)]
    );
}

#[test]
fn exposing_a_usage_s_tree_draws_no_node_for_what_it_inherits() {
    let model = snippet()
        .view("P::tree")
        .exposes("P::specialised", ExposeScope::ElementAndDescendants)
        .renders("Views::asTreeDiagram")
        .build();
    let ViewContents::General(shown) = resolve(&model, &qn("P::tree")).unwrap() else {
        panic!("a General Diagram");
    };

    let scene = build_general_scene(&model, &shown, &Layout::default());

    assert_eq!(nodes(&scene), ["P::specialised", "P::specialised::own"]);
    let texts = texts_of(&scene, "P::specialised");
    assert_eq!(compartment(&texts, "parts"), [row("^inner:Inner", true)]);
    assert_eq!(compartment(&texts, "attributes"), [row("^mass", true)]);
}

#[test]
fn a_node_whose_definitions_add_nothing_is_unchanged() {
    let model = Model::builder()
        .package("P")
        .part_def("P::Plain")
        .attribute("P::Plain::m")
        .part("P::p")
        .typed_by("P::Plain")
        .part("P::p::m")
        .element("P::p::q", ElementKind::AttributeUsage)
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    // `p::m` redefines `Plain::m` by name; nothing is inherited.
    assert_eq!(
        compartment(&texts_of(&scene, "P::p"), "attributes"),
        [row("q", false)]
    );
}

#[test]
fn a_diagram_with_inherited_members_turned_off_lists_only_declared_ones() {
    let model = snippet().build();
    let elements = [qn("P::specialised")];
    let shown = build_general_scene(&model, &elements, &Layout::default());
    let layout = Layout {
        inherited_hidden: true,
        ..Layout::default()
    };

    let scene = build_general_scene(&model, &elements, &layout);

    let texts = texts_of(&scene, "P::specialised");
    assert_eq!(compartment(&texts, "parts"), [row("own", false)]);
    assert!(
        texts.iter().all(|(t, _)| !t.starts_with('^')),
        "no ^ rows: {texts:?}"
    );
    let height = |s: &Scene| s.node_bounds(&qn("P::specialised")).unwrap().height;
    assert!(height(&scene) < height(&shown), "the box shrinks to fit");
}
