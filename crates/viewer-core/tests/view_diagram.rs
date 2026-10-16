//! Seam 1: Model-defined Diagrams drawn from views, with their Layout.

mod common;

use std::path::Path;

use viewer_core::editing::{Edit, apply_with};
use viewer_core::model::{ExposeScope, Model};
use viewer_core::scene::{Depicts, Shape};
use viewer_core::view::{ViewError, build_scene, layout_key_exists};
use viewer_core::{Layout, QualifiedName, Scene, svg};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

/// A vehicle's parts tree with a connection inside, and a tree view of it.
fn parts_tree() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Engine")
        .part_def("P::Body")
        .part("P::vehicle")
        .attribute("P::vehicle::mass")
        .part("P::vehicle::body")
        .typed_by("P::Body")
        .part("P::vehicle::body::door")
        .multiplicity("2", "2")
        .part("P::vehicle::engine")
        .typed_by("P::Engine")
        .connection("P::vehicle::mount")
        .end(&["P::vehicle::body"])
        .end(&["P::vehicle::engine"])
        .view("P::tree")
        .exposes("P::vehicle", ExposeScope::ElementAndDescendants)
        .renders("Views::asTreeDiagram")
        .build()
}

fn tree(layout: &Layout) -> Scene {
    build_scene(&parts_tree(), &qn("P::tree"), layout).unwrap()
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

fn composition(owner: &str, usage: &str) -> Depicts {
    Depicts::Composition { owner: qn(owner), usage: qn(usage) }
}

#[test]
fn a_tree_view_draws_each_exposed_part_as_a_node() {
    let scene = tree(&Layout::default());

    let mut nodes: Vec<&str> = scene.elements().iter().map(|e| e.as_str()).collect();
    nodes.sort();
    assert_eq!(
        nodes,
        ["P::vehicle", "P::vehicle::body", "P::vehicle::body::door", "P::vehicle::engine", "P::vehicle::mass"]
    );
}

#[test]
fn owned_usages_hang_below_their_owner_by_composition() {
    let scene = tree(&Layout::default());

    let edges: Vec<&Depicts> = scene.edges().into_iter().map(|(d, _)| d).collect();
    for (owner, usage) in [
        ("P::vehicle", "P::vehicle::body"),
        ("P::vehicle", "P::vehicle::engine"),
        ("P::vehicle", "P::vehicle::mass"),
        ("P::vehicle::body", "P::vehicle::body::door"),
    ] {
        assert!(edges.contains(&&composition(owner, usage)), "{owner} ◆→ {usage}");
        let (o, u) = (scene.element_bounds(&qn(owner)).unwrap(), scene.element_bounds(&qn(usage)).unwrap());
        assert!(u.y > o.bottom(), "{usage} is drawn below {owner}");
    }
}

#[test]
fn a_tree_edge_has_a_filled_diamond_and_no_label() {
    let scene = tree(&Layout::default());

    let door = composition("P::vehicle::body", "P::vehicle::body::door");
    let shapes: Vec<&Shape> = scene.items_depicting(&door).collect();
    assert!(shapes.iter().any(|s| matches!(s, Shape::Polygon { .. })), "the diamond");
    assert!(!shapes.iter().any(|s| matches!(s, Shape::Text { .. })), "the door's node already names it");
}

#[test]
fn a_part_drawn_as_a_node_is_not_repeated_in_its_owner_s_compartments() {
    let scene = tree(&Layout::default());

    let vehicle = texts_of(&scene, "P::vehicle");
    assert!(!vehicle.iter().any(|t| t.starts_with("body") || t.starts_with("engine") || t.starts_with("mass")));
    assert!(vehicle.iter().any(|t| t.starts_with("mount")), "the connection isn't a node, so it's still listed");
}

#[test]
fn connections_between_exposed_parts_are_drawn() {
    let scene = tree(&Layout::default());

    assert!(scene.edges().iter().any(|(d, _)| **d == Depicts::Connection(qn("P::vehicle::mount"))));
}

#[test]
fn moving_and_hiding_work_on_a_view_s_diagram() {
    let model = parts_tree();
    let view = qn("P::tree");
    let known = |key: &QualifiedName| layout_key_exists(&model, &view, key);
    let mut layout = Layout::default();
    let before = tree(&layout);

    apply_with(&mut layout, &known, &before, &Edit::Move { element: qn("P::vehicle::engine"), dx: 40.0, dy: 10.0 });
    let moved = tree(&layout);
    let (a, b) = (before.element_bounds(&qn("P::vehicle::engine")).unwrap(), moved.element_bounds(&qn("P::vehicle::engine")).unwrap());
    assert_eq!((b.x - a.x, b.y - a.y), (40.0, 10.0));

    apply_with(&mut layout, &known, &moved, &Edit::Hide(qn("P::vehicle::body::door")));
    let hidden = tree(&layout);
    assert!(hidden.element_bounds(&qn("P::vehicle::body::door")).is_none());
    assert!(!hidden.edges().iter().any(|(d, _)| **d == composition("P::vehicle::body", "P::vehicle::body::door")));
}

#[test]
fn a_saved_route_is_kept_on_a_tree_edge() {
    let mut layout = Layout::default();
    let key = composition("P::vehicle", "P::vehicle::engine").edge_key().unwrap();
    let start = tree(&layout).element_bounds(&qn("P::vehicle")).unwrap();
    layout.edges.insert(key, vec![[start.right() + 30.0, start.y + 10.0]]);

    let scene = tree(&layout);

    let (_, path) = scene.edges().into_iter().find(|(d, _)| **d == composition("P::vehicle", "P::vehicle::engine")).unwrap();
    assert!(path.iter().any(|p| (p.x, p.y) == (start.right() + 30.0, start.y + 10.0)));
}

#[test]
fn layout_keys_of_elements_not_exposed_are_orphaned() {
    let model = parts_tree();

    assert!(layout_key_exists(&model, &qn("P::tree"), &qn("P::vehicle::body")));
    assert!(!layout_key_exists(&model, &qn("P::tree"), &qn("P::vehicle::gone")));
    assert!(!layout_key_exists(&model, &qn("P::tree"), &qn("P::Engine")), "in the Model, but not exposed");
}

#[test]
fn an_unsupported_or_invalid_view_is_not_drawn() {
    let model = common::views();

    assert!(matches!(
        build_scene(&model, &qn("V::table"), &Layout::default()),
        Err(ViewError::Unsupported { .. })
    ));
}

#[test]
fn an_interconnection_view_draws_its_root_part() {
    let scene = build_scene(&common::views(), &qn("V::wiring"), &Layout::default()).unwrap();

    assert!(scene.items.iter().any(|i| matches!(&i.depicts, Depicts::Part { element, .. } if *element == qn("V::Vehicle::engine"))));
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
fn a_tree_view_matches_the_golden_file() {
    assert_golden("views-tree.svg", &tree(&Layout::default()));
}

#[test]
fn an_interconnection_view_matches_the_golden_file() {
    let scene = build_scene(&common::views(), &qn("V::wiring"), &Layout::default()).unwrap();

    assert_golden("views-wiring.svg", &scene);
}
