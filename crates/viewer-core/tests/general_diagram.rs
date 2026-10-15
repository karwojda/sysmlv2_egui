//! Seam 1: a Model from the fake backend becomes a General Diagram scene and
//! an SVG Export.

use std::path::Path;

use viewer_core::scene::{Depicts, Fill, Shape};
use viewer_core::{DiagramError, DiagramKind, FakeBackend, Model, ModelBackend, QualifiedName, Scene, build_scene, svg};

fn vehicles() -> Model {
    let model = Model::builder()
        .package("Vehicles")
        .part_def("Vehicles::Vehicle")
        .part_def("Vehicles::Car")
        .specializes("Vehicles::Vehicle")
        .part_def("Vehicles::Truck")
        .specializes("Vehicles::Vehicle")
        .part_def("Vehicles::ElectricCar")
        .specializes("Vehicles::Car")
        .part_def("Vehicles::Engine")
        .unsupported("Vehicles::TopSpeed", "requirement def")
        .package("Vehicles::Parts")
        .part_def("Vehicles::Parts::Wheel")
        .build();
    FakeBackend::new(model).load(Path::new("fixtures/vehicles")).unwrap()
}

fn general(model: &Model, root: &str) -> Scene {
    build_scene(model, DiagramKind::General, &QualifiedName::from(root)).unwrap()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
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
fn shows_the_part_definitions_owned_by_the_root_package() {
    let scene = general(&vehicles(), "Vehicles");

    let mut elements: Vec<&str> = scene.elements().iter().map(|e| e.as_str()).collect();
    elements.sort();
    assert_eq!(
        elements,
        ["Vehicles::Car", "Vehicles::ElectricCar", "Vehicles::Engine", "Vehicles::Truck", "Vehicles::Vehicle"]
    );
}

#[test]
fn leaves_out_unsupported_elements_and_nested_package_contents() {
    let scene = general(&vehicles(), "Vehicles");

    assert!(scene.node_bounds(&qn("Vehicles::TopSpeed")).is_none());
    assert!(scene.node_bounds(&qn("Vehicles::Parts")).is_none());
    assert!(scene.node_bounds(&qn("Vehicles::Parts::Wheel")).is_none());
}

#[test]
fn draws_part_definitions_with_keyword_above_name() {
    let scene = general(&vehicles(), "Vehicles");

    assert_eq!(texts_of(&scene, "Vehicles::Car"), ["«part def»", "Car"]);
}

#[test]
fn draws_a_specialization_edge_ending_in_a_hollow_triangle_at_the_general() {
    let scene = general(&vehicles(), "Vehicles");
    let general_bounds = scene.node_bounds(&qn("Vehicles::Vehicle")).unwrap();
    let edge = Depicts::Specialization { specific: qn("Vehicles::Car"), general: qn("Vehicles::Vehicle") };

    let triangle = scene
        .items_depicting(&edge)
        .find_map(|shape| match shape {
            Shape::Polygon { points, fill } => Some((points.clone(), *fill)),
            _ => None,
        })
        .expect("specialization edge has an end marker");

    let (points, fill) = triangle;
    assert_eq!(points.len(), 3);
    assert_eq!(fill, Fill::Background, "the triangle is hollow");
    let tip = points[0];
    assert_eq!(tip.y, general_bounds.bottom(), "the tip touches the general node");
}

#[test]
fn draws_only_specializations_between_nodes_in_the_diagram() {
    let model = Model::builder()
        .package("P")
        .part_def("P::A")
        .specializes("Other::Base")
        .build();
    let scene = general(&model, "P");

    assert!(
        scene
            .items
            .iter()
            .all(|item| matches!(item.depicts, Depicts::Element(_)))
    );
}

#[test]
fn places_general_definitions_above_their_specializations() {
    let scene = general(&vehicles(), "Vehicles");
    let bounds = |name: &str| scene.node_bounds(&qn(name)).unwrap();

    assert!(bounds("Vehicles::Vehicle").bottom() < bounds("Vehicles::Car").y);
    assert!(bounds("Vehicles::Vehicle").bottom() < bounds("Vehicles::Truck").y);
    assert!(bounds("Vehicles::Car").bottom() < bounds("Vehicles::ElectricCar").y);
}

#[test]
fn nodes_do_not_overlap_and_fit_inside_the_scene() {
    let scene = general(&vehicles(), "Vehicles");
    let rects: Vec<_> = scene
        .elements()
        .iter()
        .map(|e| scene.node_bounds(e).unwrap())
        .collect();

    for (i, a) in rects.iter().enumerate() {
        assert!(a.x >= 0.0 && a.y >= 0.0 && a.right() <= scene.width && a.bottom() <= scene.height);
        for b in &rects[i + 1..] {
            assert!(!a.intersects(b), "{a:?} overlaps {b:?}");
        }
    }
}

#[test]
fn finds_the_element_whose_node_is_under_a_point() {
    let scene = general(&vehicles(), "Vehicles");
    let car = scene.node_bounds(&qn("Vehicles::Car")).unwrap();
    let inside = viewer_core::scene::Point::new(car.x + 5.0, car.bottom() - 5.0);
    let outside = viewer_core::scene::Point::new(scene.width + 10.0, 0.0);

    assert_eq!(scene.element_at(inside), Some(&qn("Vehicles::Car")));
    assert_eq!(scene.element_at(outside), None);
}

#[test]
fn general_diagrams_are_rooted_at_packages_and_definitions() {
    let model = vehicles();
    let accepts = |name: &str| DiagramKind::General.accepts_root(model.element(&qn(name)).unwrap());

    assert!(accepts("Vehicles"));
    assert!(accepts("Vehicles::Car"));
    assert!(!accepts("Vehicles::TopSpeed"));
}

#[test]
fn an_empty_package_gives_an_empty_diagram() {
    let model = Model::builder().package("Empty").build();

    assert!(general(&model, "Empty").items.is_empty());
}

#[test]
fn a_missing_root_element_is_an_error() {
    let result = build_scene(&vehicles(), DiagramKind::General, &qn("Vehicles::Gone"));

    assert_eq!(result.unwrap_err(), DiagramError::RootNotFound(qn("Vehicles::Gone")));
}

#[test]
fn export_is_deterministic() {
    let model = vehicles();
    let first = svg::render(&general(&model, "Vehicles"));

    for _ in 0..50 {
        assert_eq!(svg::render(&general(&model, "Vehicles")), first);
    }
}

#[test]
fn export_escapes_names() {
    let model = Model::builder().package("P").part_def("P::'A<B>&C'").build();

    let svg = svg::render(&general(&model, "P"));

    assert!(svg.contains(">'A&lt;B&gt;&amp;C'</text>"));
}

#[test]
fn export_matches_the_golden_file() {
    let svg = svg::render(&general(&vehicles(), "Vehicles"));
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/vehicles-general.svg");

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::create_dir_all(golden.parent().unwrap()).unwrap();
        std::fs::write(&golden, &svg).unwrap();
    }
    let expected = std::fs::read_to_string(&golden)
        .unwrap_or_else(|_| panic!("missing {}; run with UPDATE_GOLDEN=1 to create it", golden.display()));
    assert_eq!(svg, expected, "Export differs from the golden file; if intended, rerun with UPDATE_GOLDEN=1");
}
