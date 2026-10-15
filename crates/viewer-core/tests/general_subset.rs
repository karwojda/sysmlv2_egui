//! Seam 1: the full first-slice language subset in General Diagrams.

use std::path::Path;

use viewer_core::model::Direction;
use viewer_core::scene::{Depicts, Fill, Shape};
use viewer_core::{DiagramKind, Model, QualifiedName, Scene, build_scene, svg};

/// The fake twin of the backend crate's `fixtures/structure`.
fn structure() -> Model {
    Model::builder()
        .package("Structure")
        .attribute_def("Structure::Fuel")
        .attribute_def("Structure::Mass")
        .port_def("Structure::FuelPort")
        .attribute("Structure::FuelPort::fuel")
        .typed_by("Structure::Fuel")
        .direction(Direction::Out)
        .interface_def("Structure::FuelInterface")
        .port("Structure::FuelInterface::supply")
        .typed_by("Structure::FuelPort")
        .port("Structure::FuelInterface::demand")
        .typed_by("Structure::FuelPort")
        .conjugated()
        .connection_def("Structure::FuelLine")
        .unsupported("Structure::FuelLine::a", "reference")
        .typed_by("Structure::FuelPort")
        .unsupported("Structure::FuelLine::b", "reference")
        .typed_by("Structure::FuelPort")
        .part_def("Structure::Engine")
        .doc("Turns fuel into motion.")
        .attribute("Structure::Engine::mass")
        .typed_by("Structure::Mass")
        .port("Structure::Engine::fuelIn")
        .typed_by("Structure::FuelPort")
        .conjugated()
        .direction(Direction::In)
        .part_def("Structure::Tank")
        .port("Structure::Tank::fuelOut")
        .typed_by("Structure::FuelPort")
        .direction(Direction::Out)
        .part_def("Structure::Vehicle")
        .part("Structure::Vehicle::engine")
        .typed_by("Structure::Engine")
        .multiplicity("1", "1")
        .part("Structure::Vehicle::tanks")
        .typed_by("Structure::Tank")
        .multiplicity("1", "2")
        .attribute("Structure::Vehicle::mass")
        .typed_by("Structure::Mass")
        .connection("Structure::Vehicle::line")
        .typed_by("Structure::FuelLine")
        .end(&["Structure::Vehicle::tanks", "Structure::Tank::fuelOut"])
        .end(&["Structure::Vehicle::engine", "Structure::Engine::fuelIn"])
        .interface("Structure::Vehicle::iface")
        .typed_by("Structure::FuelInterface")
        .end(&["Structure::Vehicle::tanks", "Structure::Tank::fuelOut"])
        .end(&["Structure::Vehicle::engine", "Structure::Engine::fuelIn"])
        .part("Structure::vehicle")
        .typed_by("Structure::Vehicle")
        .part("Structure::spare")
        .typed_by("Structure::Engine")
        .multiplicity("0", "*")
        .connection("Structure::link")
        .end(&["Structure::vehicle"])
        .end(&["Structure::spare"])
        .unsupported("Structure::TopSpeed", "requirement def")
        .build()
}

fn scene() -> Scene {
    build_scene(&structure(), DiagramKind::General, &QualifiedName::from("Structure")).unwrap()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn texts(scene: &Scene, depicts: &Depicts) -> Vec<String> {
    scene
        .items_depicting(depicts)
        .filter_map(|shape| match shape {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

fn node_texts(scene: &Scene, element: &str) -> Vec<String> {
    texts(scene, &Depicts::Element(qn(element)))
}

fn corner_radius(scene: &Scene, element: &str) -> f32 {
    scene
        .items_depicting(&Depicts::Element(qn(element)))
        .find_map(|shape| match shape {
            Shape::Rect { corner_radius, .. } => Some(*corner_radius),
            _ => None,
        })
        .unwrap()
}

#[test]
fn every_definition_and_top_level_usage_is_a_node() {
    let scene = scene();

    let mut elements: Vec<&str> = scene.elements().iter().map(|e| e.as_str()).collect();
    elements.sort();
    assert_eq!(
        elements,
        [
            "Structure::Engine",
            "Structure::Fuel",
            "Structure::FuelInterface",
            "Structure::FuelLine",
            "Structure::FuelPort",
            "Structure::Mass",
            "Structure::Tank",
            "Structure::Vehicle",
            "Structure::spare",
            "Structure::vehicle",
        ]
    );
}

#[test]
fn definitions_have_square_corners_and_usages_rounded_ones() {
    let scene = scene();

    assert_eq!(corner_radius(&scene, "Structure::Engine"), 0.0);
    assert_eq!(corner_radius(&scene, "Structure::FuelPort"), 0.0);
    assert!(corner_radius(&scene, "Structure::vehicle") > 0.0);
}

#[test]
fn each_kind_shows_its_keyword() {
    let scene = scene();
    let keyword = |element: &str| node_texts(&scene, element)[0].clone();

    assert_eq!(keyword("Structure::Fuel"), "«attribute def»");
    assert_eq!(keyword("Structure::FuelPort"), "«port def»");
    assert_eq!(keyword("Structure::FuelLine"), "«connection def»");
    assert_eq!(keyword("Structure::FuelInterface"), "«interface def»");
    assert_eq!(keyword("Structure::Engine"), "«part def»");
    assert_eq!(keyword("Structure::vehicle"), "«part»");
}

#[test]
fn usages_show_their_type_and_multiplicity() {
    let scene = scene();

    assert_eq!(node_texts(&scene, "Structure::spare")[1], "spare : Engine [0..*]");
    assert_eq!(node_texts(&scene, "Structure::vehicle")[1], "vehicle : Vehicle");
}

#[test]
fn definitions_list_their_features_in_compartments() {
    let scene = scene();

    assert_eq!(
        node_texts(&scene, "Structure::Vehicle"),
        [
            "«part def»",
            "Vehicle",
            "attributes",
            "mass : Mass",
            "parts",
            "engine : Engine [1]",
            "tanks : Tank [1..2]",
            "connections",
            "iface : FuelInterface (tanks.fuelOut → engine.fuelIn)",
            "line : FuelLine (tanks.fuelOut → engine.fuelIn)",
        ]
    );
}

#[test]
fn ports_show_their_direction_and_conjugation() {
    let scene = scene();

    assert!(node_texts(&scene, "Structure::Engine").contains(&"in fuelIn : ~FuelPort".to_owned()));
    assert!(node_texts(&scene, "Structure::Tank").contains(&"out fuelOut : FuelPort".to_owned()));
    assert!(node_texts(&scene, "Structure::FuelPort").contains(&"out fuel : Fuel".to_owned()));
}

#[test]
fn composition_is_a_filled_diamond_at_the_owner_labelled_near_the_part_type() {
    let scene = scene();
    let edge = Depicts::Composition { owner: qn("Structure::Vehicle"), usage: qn("Structure::Vehicle::tanks") };
    let owner = scene.node_bounds(&qn("Structure::Vehicle")).unwrap();

    let diamond = scene
        .items_depicting(&edge)
        .find_map(|shape| match shape {
            Shape::Polygon { points, fill } => Some((points.clone(), *fill)),
            _ => None,
        })
        .expect("a composition has a diamond");

    assert_eq!(diamond.0.len(), 4);
    assert_eq!(diamond.1, Fill::Stroke, "the diamond is filled");
    let tip = diamond.0[0];
    assert!(tip.y == owner.bottom() || tip.y == owner.y || tip.x == owner.x || tip.x == owner.right());
    assert_eq!(texts(&scene, &edge), ["tanks [1..2]"]);
}

#[test]
fn owners_sit_above_the_definitions_of_their_parts() {
    let scene = scene();
    let bounds = |name: &str| scene.node_bounds(&qn(name)).unwrap();

    assert!(bounds("Structure::Vehicle").bottom() < bounds("Structure::Engine").y);
    assert!(bounds("Structure::Vehicle").bottom() < bounds("Structure::Tank").y);
}

#[test]
fn a_usage_node_is_joined_to_its_definition_by_a_dashed_typing_edge() {
    let scene = scene();
    let edge = Depicts::Typing { usage: qn("Structure::spare"), definition: qn("Structure::Engine") };

    let shapes: Vec<&Shape> = scene.items_depicting(&edge).collect();

    assert!(shapes.iter().any(|s| matches!(s, Shape::Polyline { dashed: true, .. })));
    assert!(shapes.iter().any(|s| matches!(s, Shape::Polygon { fill: Fill::Background, .. })));
}

#[test]
fn connections_between_usage_nodes_are_labelled_edges() {
    let scene = scene();
    let edge = Depicts::Connection(qn("Structure::link"));

    let shapes: Vec<&Shape> = scene.items_depicting(&edge).collect();

    assert!(shapes.iter().any(|s| matches!(s, Shape::Polyline { dashed: false, .. })));
    assert_eq!(texts(&scene, &edge), ["link"]);
}

#[test]
fn connections_whose_ends_are_not_nodes_are_not_edges() {
    let model = Model::builder()
        .package("P")
        .part("P::a")
        .connection("P::c")
        .end(&["P::a"])
        .end(&["Elsewhere::b"])
        .build();

    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    assert_eq!(scene.items_depicting(&Depicts::Connection(qn("P::c"))).count(), 0);
}

#[test]
fn unsupported_elements_and_their_features_are_never_drawn() {
    let model = Model::builder()
        .package("P")
        .part_def("P::A")
        .unsupported("P::A::doSomething", "action")
        .unsupported("P::Req", "requirement def")
        .build();

    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    assert!(scene.node_bounds(&qn("P::Req")).is_none());
    assert_eq!(node_texts(&scene, "P::A"), ["«part def»", "A"]);
}

#[test]
fn nodes_do_not_overlap() {
    let scene = scene();
    let rects: Vec<_> = scene
        .elements()
        .iter()
        .map(|e| scene.node_bounds(e).unwrap())
        .collect();

    for (i, a) in rects.iter().enumerate() {
        for b in &rects[i + 1..] {
            assert!(!a.intersects(b), "{a:?} overlaps {b:?}");
        }
    }
}

#[test]
fn export_matches_the_golden_file() {
    let svg = svg::render(&scene());
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/structure-general.svg");

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &svg).unwrap();
    }
    let expected = std::fs::read_to_string(&golden)
        .unwrap_or_else(|_| panic!("missing {}; run with UPDATE_GOLDEN=1 to create it", golden.display()));
    assert_eq!(svg, expected, "Export differs from the golden file; if intended, rerun with UPDATE_GOLDEN=1");
}
