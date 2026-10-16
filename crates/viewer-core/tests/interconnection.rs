//! Seam 1: Interconnection Diagrams from a fake-backend Model.

use std::path::Path;

use viewer_core::model::Direction;
use viewer_core::scene::{Depicts, Point, Rect, Shape, TextAnchor};
use viewer_core::{DiagramError, DiagramFile, DiagramKind, Model, QualifiedName, Scene, build_scene, svg};

/// The `structure` fixture's wiring: a vehicle whose tanks feed its engine
/// over two connections, plus a controller on an undirected status port.
fn vehicle() -> Model {
    Model::builder()
        .package("S")
        .port_def("S::FuelPort")
        .port_def("S::StatusPort")
        .interface_def("S::FuelInterface")
        .connection_def("S::FuelLine")
        .part_def("S::Engine")
        .port("S::Engine::fuelIn")
        .typed_by("S::FuelPort")
        .conjugated()
        .direction(Direction::In)
        .port("S::Engine::status")
        .typed_by("S::StatusPort")
        .part_def("S::Tank")
        .port("S::Tank::fuelOut")
        .typed_by("S::FuelPort")
        .direction(Direction::Out)
        .part_def("S::Controller")
        .port("S::Controller::status")
        .typed_by("S::StatusPort")
        .conjugated()
        .part_def("S::Vehicle")
        .part("S::Vehicle::engine")
        .typed_by("S::Engine")
        .multiplicity("1", "1")
        .part("S::Vehicle::tanks")
        .typed_by("S::Tank")
        .multiplicity("1", "2")
        .part("S::Vehicle::controller")
        .typed_by("S::Controller")
        .attribute("S::Vehicle::mass")
        .connection("S::Vehicle::line")
        .typed_by("S::FuelLine")
        .end(&["S::Vehicle::tanks", "S::Tank::fuelOut"])
        .end(&["S::Vehicle::engine", "S::Engine::fuelIn"])
        .interface("S::Vehicle::iface")
        .typed_by("S::FuelInterface")
        .end(&["S::Vehicle::tanks", "S::Tank::fuelOut"])
        .end(&["S::Vehicle::engine", "S::Engine::fuelIn"])
        .connection("S::Vehicle::statusLink")
        .end(&["S::Vehicle::engine", "S::Engine::status"])
        .end(&["S::Vehicle::controller", "S::Controller::status"])
        .part("S::car")
        .typed_by("S::Vehicle")
        .part("S::loose")
        .build()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn interconnection(root: &str) -> Scene {
    build_scene(&vehicle(), DiagramKind::Interconnection, &qn(root)).unwrap()
}

fn texts(scene: &Scene, depicts: &Depicts) -> Vec<String> {
    scene
        .items_depicting(depicts)
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

fn port_square(scene: &Scene, part: &str, port: &str) -> Rect {
    scene
        .items_depicting(&Depicts::Port { part: qn(part), port: qn(port) })
        .find_map(|s| match s {
            Shape::Rect { rect, .. } => Some(*rect),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no port {port} on {part}"))
}

fn centre(r: Rect) -> Point {
    Point::new(r.x + r.width / 2.0, r.y + r.height / 2.0)
}

#[test]
fn the_frame_holds_the_definitions_parts() {
    let scene = interconnection("S::Vehicle");

    assert_eq!(texts(&scene, &Depicts::Element(qn("S::Vehicle"))), ["«part def»", "Vehicle"]);
    let frame = scene.node_bounds(&qn("S::Vehicle")).unwrap();
    for part in ["S::Vehicle::engine", "S::Vehicle::tanks", "S::Vehicle::controller"] {
        let rect = scene.node_bounds(&qn(part)).unwrap();
        assert!(rect.x > frame.x && rect.right() < frame.right() && rect.y > frame.y && rect.bottom() < frame.bottom(), "{part} inside");
    }
    assert!(scene.node_bounds(&qn("S::Vehicle::mass")).is_none(), "attributes are not parts");
}

#[test]
fn parts_are_labelled_with_their_type_and_multiplicity() {
    let scene = interconnection("S::Vehicle");

    assert_eq!(texts(&scene, &Depicts::Element(qn("S::Vehicle::tanks"))), ["«part»", "tanks:Tank [1..2]"]);
}

#[test]
fn a_part_usage_root_shows_the_parts_of_its_definition() {
    let scene = interconnection("S::car");

    assert_eq!(texts(&scene, &Depicts::Element(qn("S::car"))), ["«part»", "car:Vehicle"]);
    assert!(scene.node_bounds(&qn("S::Vehicle::engine")).is_some());
}

#[test]
fn only_parts_can_be_roots() {
    for root in ["S", "S::loose", "S::FuelPort"] {
        let result = build_scene(&vehicle(), DiagramKind::Interconnection, &qn(root));
        assert!(matches!(result, Err(DiagramError::UnsuitableRoot { .. })), "{root}: {result:?}");
    }
}

#[test]
fn in_ports_sit_on_top_out_ports_at_the_bottom() {
    let scene = interconnection("S::Vehicle");
    let engine = scene.node_bounds(&qn("S::Vehicle::engine")).unwrap();
    let tanks = scene.node_bounds(&qn("S::Vehicle::tanks")).unwrap();

    assert_eq!(centre(port_square(&scene, "S::Vehicle::engine", "S::Engine::fuelIn")).y, engine.y);
    assert_eq!(centre(port_square(&scene, "S::Vehicle::tanks", "S::Tank::fuelOut")).y, tanks.bottom());
}

#[test]
fn ports_are_labelled_with_name_and_type() {
    let scene = interconnection("S::Vehicle");

    let label = texts(&scene, &Depicts::Port { part: qn("S::Vehicle::engine"), port: qn("S::Engine::fuelIn") });

    assert_eq!(label, ["fuelIn : ~FuelPort"]);
}

#[test]
fn flow_runs_top_to_bottom_from_out_to_in() {
    let scene = interconnection("S::Vehicle");

    let tanks = scene.node_bounds(&qn("S::Vehicle::tanks")).unwrap();
    let engine = scene.node_bounds(&qn("S::Vehicle::engine")).unwrap();
    assert!(tanks.bottom() < engine.y, "tanks feed the engine from above");
}

#[test]
fn undirected_ports_face_what_they_connect_to() {
    let scene = interconnection("S::Vehicle");
    let engine = centre(scene.node_bounds(&qn("S::Vehicle::engine")).unwrap());
    let controller = centre(scene.node_bounds(&qn("S::Vehicle::controller")).unwrap());

    let status = centre(port_square(&scene, "S::Vehicle::engine", "S::Engine::status"));

    let toward = (controller.x - engine.x, controller.y - engine.y);
    let placed = (status.x - engine.x, status.y - engine.y);
    let dominant = |(x, y): (f32, f32)| if y.abs() >= x.abs() { (0.0, y.signum()) } else { (x.signum(), 0.0) };
    assert_eq!(dominant(placed), dominant(toward));
}

#[test]
fn connectors_join_their_ports_with_right_angles() {
    let scene = interconnection("S::Vehicle");
    let line = scene
        .items_depicting(&Depicts::Connection(qn("S::Vehicle::line")))
        .find_map(|s| match s {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .unwrap();

    assert_eq!(line[0], centre(port_square(&scene, "S::Vehicle::tanks", "S::Tank::fuelOut")));
    assert_eq!(*line.last().unwrap(), centre(port_square(&scene, "S::Vehicle::engine", "S::Engine::fuelIn")));
    for segment in line.windows(2) {
        assert!(segment[0].x == segment[1].x || segment[0].y == segment[1].y, "{segment:?} is not straight");
    }
}

#[test]
fn connectors_over_the_same_ports_keep_separate_labels() {
    let scene = interconnection("S::Vehicle");
    let label = |name: &str| {
        scene
            .items_depicting(&Depicts::Connection(qn(name)))
            .find_map(|s| match s {
                Shape::Text { position, text, anchor: TextAnchor::Start, .. } => Some((*position, text.clone())),
                _ => None,
            })
            .unwrap()
    };

    let (line_at, line_text) = label("S::Vehicle::line");
    let (iface_at, iface_text) = label("S::Vehicle::iface");

    assert_eq!(line_text, "line:FuelLine");
    assert_eq!(iface_text, "iface:FuelInterface");
    assert!((line_at.y - iface_at.y).abs() >= 11.0, "stacked, not overprinted");
}

#[test]
fn parts_do_not_overlap() {
    let scene = interconnection("S::Vehicle");
    let parts: Vec<Rect> = ["S::Vehicle::engine", "S::Vehicle::tanks", "S::Vehicle::controller"]
        .iter()
        .map(|p| scene.node_bounds(&qn(p)).unwrap())
        .collect();

    for (i, a) in parts.iter().enumerate() {
        for b in &parts[i + 1..] {
            assert!(!a.intersects(b));
        }
    }
}

#[test]
fn export_is_deterministic_and_matches_the_golden_file() {
    let first = svg::render(&interconnection("S::Vehicle"));
    for _ in 0..20 {
        assert_eq!(svg::render(&interconnection("S::Vehicle")), first);
    }
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/vehicle-interconnection.svg");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &first).unwrap();
    }
    let expected = std::fs::read_to_string(&golden)
        .unwrap_or_else(|_| panic!("missing {}; run with UPDATE_GOLDEN=1 to create it", golden.display()));
    assert_eq!(first, expected, "Export differs from the golden file; if intended, rerun with UPDATE_GOLDEN=1");
}

#[test]
fn diagram_files_name_the_kind_interconnection() {
    let file = DiagramFile::new("wiring", DiagramKind::Interconnection, qn("S::Vehicle"));

    assert!(file.to_text().contains(r#""kind": "interconnection""#));
    assert_eq!(DiagramFile::parse(&file.to_text()).unwrap(), file);
}

#[test]
fn finds_the_port_under_a_point() {
    let scene = interconnection("S::Vehicle");
    let square = port_square(&scene, "S::Vehicle::engine", "S::Engine::fuelIn");

    let found = scene.port_at(centre(square));

    assert_eq!(found, Some((&qn("S::Vehicle::engine"), &qn("S::Engine::fuelIn"))));
    assert_eq!(scene.port_at(Point::new(square.x - 20.0, square.y - 20.0)), None);
}
