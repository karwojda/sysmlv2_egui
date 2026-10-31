//! Seam 1: Interconnection Diagrams from a fake-backend Model.

use std::path::Path;

use viewer_core::model::Direction;
use viewer_core::scene::{Depicts, Point, Rect, Shape, TextAnchor};
use viewer_core::{
    DiagramError, DiagramFile, DiagramKind, Model, QualifiedName, Scene, build_scene, svg,
};

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

fn port_items<'a>(scene: &'a Scene, part: &str, port: &str) -> Vec<&'a Shape> {
    scene
        .items
        .iter()
        .filter(|i| matches!(&i.depicts, Depicts::Port { part: p, port: q, .. } if *p == qn(part) && *q == qn(port)))
        .map(|i| &i.shape)
        .collect()
}

fn port_square(scene: &Scene, part: &str, port: &str) -> Rect {
    port_items(scene, part, port)
        .into_iter()
        .find_map(|s| match s {
            Shape::Rect { rect, .. } => Some(*rect),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no port {port} on {part}"))
}

/// The texts drawn for the node of `element`, whether the Frame or a part.
fn node_texts(scene: &Scene, element: &str) -> Vec<String> {
    scene
        .items
        .iter()
        .filter(|i| matches!(&i.depicts, Depicts::Element(e) | Depicts::Part { element: e, .. } if *e == qn(element)))
        .filter_map(|i| match &i.shape {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

fn centre(r: Rect) -> Point {
    Point::new(r.x + r.width / 2.0, r.y + r.height / 2.0)
}

#[test]
fn the_frame_holds_the_definitions_parts() {
    let scene = interconnection("S::Vehicle");

    assert_eq!(node_texts(&scene, "S::Vehicle"), ["«part def»", "Vehicle"]);
    let frame = scene.element_bounds(&qn("S::Vehicle")).unwrap();
    for part in [
        "S::Vehicle::engine",
        "S::Vehicle::tanks",
        "S::Vehicle::controller",
    ] {
        let rect = scene.element_bounds(&qn(part)).unwrap();
        assert!(
            rect.x > frame.x
                && rect.right() < frame.right()
                && rect.y > frame.y
                && rect.bottom() < frame.bottom(),
            "{part} inside"
        );
    }
    assert!(
        scene.element_bounds(&qn("S::Vehicle::mass")).is_none(),
        "attributes are not parts"
    );
}

#[test]
fn parts_are_labelled_with_their_type_and_multiplicity() {
    let scene = interconnection("S::Vehicle");

    assert_eq!(
        node_texts(&scene, "S::Vehicle::tanks"),
        ["«part»", "tanks:Tank [1..2]"]
    );
}

#[test]
fn a_part_usage_root_shows_the_parts_of_its_definition() {
    let scene = interconnection("S::car");

    assert_eq!(node_texts(&scene, "S::car"), ["«part»", "car:Vehicle"]);
    assert!(scene.element_bounds(&qn("S::Vehicle::engine")).is_some());
}

#[test]
fn only_parts_can_be_roots() {
    for root in ["S", "S::loose", "S::FuelPort"] {
        let result = build_scene(&vehicle(), DiagramKind::Interconnection, &qn(root));
        assert!(
            matches!(result, Err(DiagramError::UnsuitableRoot { .. })),
            "{root}: {result:?}"
        );
    }
}

#[test]
fn ports_are_labelled_with_name_and_type() {
    let scene = interconnection("S::Vehicle");

    let label: Vec<String> = port_items(&scene, "S::Vehicle::engine", "S::Engine::fuelIn")
        .into_iter()
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    assert_eq!(label, ["fuelIn : ~FuelPort"]);
}

#[test]
fn flow_runs_top_to_bottom_from_out_to_in() {
    let scene = interconnection("S::Vehicle");

    let tanks = scene.element_bounds(&qn("S::Vehicle::tanks")).unwrap();
    let engine = scene.element_bounds(&qn("S::Vehicle::engine")).unwrap();
    assert!(
        tanks.bottom() < engine.y,
        "tanks feed the engine from above"
    );
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

    assert_eq!(
        line[0],
        centre(port_square(&scene, "S::Vehicle::tanks", "S::Tank::fuelOut"))
    );
    assert_eq!(
        *line.last().unwrap(),
        centre(port_square(
            &scene,
            "S::Vehicle::engine",
            "S::Engine::fuelIn"
        ))
    );
    for segment in line.windows(2) {
        assert!(
            segment[0].x == segment[1].x || segment[0].y == segment[1].y,
            "{segment:?} is not straight"
        );
    }
}

#[test]
fn connectors_over_the_same_ports_keep_separate_labels() {
    let scene = interconnection("S::Vehicle");
    let label = |name: &str| {
        scene
            .items_depicting(&Depicts::Connection(qn(name)))
            .find_map(|s| match s {
                Shape::Text {
                    position,
                    text,
                    anchor: TextAnchor::Start,
                    ..
                } => Some((*position, text.clone())),
                _ => None,
            })
            .unwrap()
    };

    let (line_at, line_text) = label("S::Vehicle::line");
    let (iface_at, iface_text) = label("S::Vehicle::iface");

    assert_eq!(line_text, "line:FuelLine");
    assert_eq!(iface_text, "iface:FuelInterface");
    assert!(
        (line_at.y - iface_at.y).abs() >= 11.0,
        "stacked, not overprinted"
    );
}

#[test]
fn parts_do_not_overlap() {
    let scene = interconnection("S::Vehicle");
    let parts: Vec<Rect> = [
        "S::Vehicle::engine",
        "S::Vehicle::tanks",
        "S::Vehicle::controller",
    ]
    .iter()
    .map(|p| scene.element_bounds(&qn(p)).unwrap())
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
    let golden =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/vehicle-interconnection.svg");
    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(&golden, &first).unwrap();
    }
    let expected = std::fs::read_to_string(&golden).unwrap_or_else(|_| {
        panic!(
            "missing {}; run with UPDATE_GOLDEN=1 to create it",
            golden.display()
        )
    });
    assert_eq!(
        first, expected,
        "Export differs from the golden file; if intended, rerun with UPDATE_GOLDEN=1"
    );
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

    assert_eq!(
        found,
        Some((
            &qn("S::Vehicle::engine"),
            &qn("S::Engine::fuelIn"),
            "engine.fuelIn"
        ))
    );
    assert_eq!(
        scene.port_at(Point::new(square.x - 20.0, square.y - 20.0)),
        None
    );
}

mod common;

/// Every pair of labels whose estimated boxes overlap.
fn overlapping_labels(scene: &Scene) -> Vec<(String, String)> {
    let labels: Vec<(String, Rect)> = scene
        .items
        .iter()
        .filter_map(|item| match &item.shape {
            Shape::Text { text, .. } => Some((text.clone(), item.shape.extent())),
            _ => None,
        })
        .collect();
    let mut overlaps = Vec::new();
    for (i, (a, ra)) in labels.iter().enumerate() {
        for (b, rb) in &labels[i + 1..] {
            if ra.intersects(rb) {
                overlaps.push((a.clone(), b.clone()));
            }
        }
    }
    overlaps
}

fn drivetrain(root: &str) -> Scene {
    build_scene(
        &common::drivetrain(),
        DiagramKind::Interconnection,
        &qn(root),
    )
    .unwrap()
}

#[test]
fn no_two_labels_overlap() {
    for (name, scene) in [
        ("vehicle", interconnection("S::Vehicle")),
        ("drivetrain", drivetrain("D::Vehicle")),
    ] {
        let overlaps = overlapping_labels(&scene);
        assert!(overlaps.is_empty(), "{name}: {overlaps:?}");
    }
}

#[test]
fn closed_parts_are_wide_enough_for_their_port_labels_side_by_side() {
    let scene = drivetrain("D::Vehicle");
    let engine = scene.element_bounds(&qn("D::Vehicle::engine")).unwrap();
    let top_labels: f32 = scene
        .items
        .iter()
        .filter(|item| matches!(&item.depicts, Depicts::Port { part, .. } if part == &qn("D::Vehicle::engine")))
        .filter_map(|item| match &item.shape {
            Shape::Text { position, .. } if position.y < engine.y => Some(item.shape.extent().width),
            _ => None,
        })
        .sum();

    assert!(
        engine.width >= top_labels,
        "{} < {top_labels}",
        engine.width
    );
}

#[test]
fn drivetrain_matches_the_golden_file() {
    let svg = svg::render(&drivetrain("D::Vehicle"));
    let golden =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/drivetrain-interconnection.svg");
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

fn polyline(scene: &Scene, connection: &str) -> Vec<Point> {
    scene
        .items_depicting(&Depicts::Connection(qn(connection)))
        .find_map(|s| match s {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("no connector for {connection}"))
}

#[test]
fn the_roots_own_ports_sit_on_the_frame_border() {
    let scene = drivetrain("D::Vehicle");
    let frame = scene.element_bounds(&qn("D::Vehicle")).unwrap();

    let power = centre(port_square(&scene, "D::Vehicle", "D::Vehicle::power"));

    assert_eq!(
        power.y, frame.y,
        "an in port of the root sits on the Frame's top edge"
    );
}

#[test]
fn a_connection_to_a_root_port_runs_from_the_frame_border_into_the_part() {
    let scene = drivetrain("D::Vehicle");
    let supply = polyline(&scene, "D::Vehicle::supply");

    let ends = [supply[0], *supply.last().unwrap()];
    let power = centre(port_square(&scene, "D::Vehicle", "D::Vehicle::power"));
    let engine_power = centre(port_square(
        &scene,
        "D::Vehicle::engine",
        "D::Engine::power",
    ));

    assert!(
        ends.contains(&power) && ends.contains(&engine_power),
        "{supply:?}"
    );
}

#[test]
fn a_connection_ending_at_a_part_attaches_to_its_border_facing_the_other_end() {
    let model = Model::builder()
        .package("P")
        .port_def("P::Flow")
        .part_def("P::Source")
        .port("P::Source::out")
        .typed_by("P::Flow")
        .direction(Direction::Out)
        .part_def("P::Sink")
        .part_def("P::System")
        .part("P::System::source")
        .typed_by("P::Source")
        .part("P::System::sink")
        .typed_by("P::Sink")
        .connection("P::System::feed")
        .end(&["P::System::source", "P::Source::out"])
        .end(&["P::System::sink"])
        .build();
    let scene = build_scene(&model, DiagramKind::Interconnection, &qn("P::System")).unwrap();
    let sink = scene.element_bounds(&qn("P::System::sink")).unwrap();
    let feed = polyline(&scene, "P::System::feed");

    let at_sink = *feed.last().unwrap();

    let on_border = at_sink.y == sink.y
        || at_sink.y == sink.bottom()
        || at_sink.x == sink.x
        || at_sink.x == sink.right();
    assert!(on_border, "{at_sink:?} on {sink:?}");
    assert!(
        feed[0].y < sink.y && at_sink.y == sink.y,
        "the source is above, so it enters from the top"
    );
}

/// Within the tolerance a straight connector allows for ports nearly in line
/// (still inside the port's square).
fn near(a: Point, b: Point) -> bool {
    (a.x - b.x).abs() <= 2.5 && (a.y - b.y).abs() <= 2.5
}

fn drivetrain_opened(paths: &[&str]) -> Scene {
    let mut layout = viewer_core::Layout::default();
    layout.opened.extend(paths.iter().map(|p| p.to_string()));
    viewer_core::build_scene_with_layout(
        &common::drivetrain(),
        DiagramKind::Interconnection,
        &qn("D::Vehicle"),
        &layout,
    )
    .unwrap()
}

#[test]
fn an_opened_part_shows_its_parts_inside_its_box() {
    let scene = drivetrain_opened(&["rearAxleAssembly"]);
    let axle = scene
        .element_bounds(&qn("D::Vehicle::rearAxleAssembly"))
        .unwrap();

    for inner in [
        "D::AxleAssembly::differential",
        "D::AxleAssembly::leftWheel",
        "D::AxleAssembly::rightWheel",
    ] {
        let rect = scene
            .element_bounds(&qn(inner))
            .unwrap_or_else(|| panic!("{inner} is drawn"));
        assert!(
            rect.x > axle.x
                && rect.right() < axle.right()
                && rect.y > axle.y
                && rect.bottom() < axle.bottom(),
            "{inner} inside"
        );
    }
    let closed = drivetrain("D::Vehicle");
    assert!(
        closed
            .element_bounds(&qn("D::AxleAssembly::differential"))
            .is_none(),
        "closed by default"
    );
}

#[test]
fn an_opened_part_draws_its_own_connections_including_from_its_own_ports() {
    let scene = drivetrain_opened(&["rearAxleAssembly"]);
    let drive = polyline(&scene, "D::AxleAssembly::drive");

    let own = centre(port_square(
        &scene,
        "D::Vehicle::rearAxleAssembly",
        "D::AxleAssembly::torqueIn",
    ));
    let differential = centre(port_square(
        &scene,
        "D::AxleAssembly::differential",
        "D::Differential::torqueIn",
    ));
    let ends = [drive[0], *drive.last().unwrap()];
    assert!(
        ends.iter().any(|e| near(*e, own)) && ends.iter().any(|e| near(*e, differential)),
        "{drive:?}"
    );
}

#[test]
fn an_end_inside_a_closed_part_attaches_to_its_border_and_to_the_port_once_opened() {
    let closed = drivetrain("D::Vehicle");
    let axle = closed
        .element_bounds(&qn("D::Vehicle::rearAxleAssembly"))
        .unwrap();
    let to_axle = *polyline(&closed, "D::Vehicle::leftDrive").last().unwrap();
    assert!(
        to_axle.y == axle.y
            || to_axle.y == axle.bottom()
            || to_axle.x == axle.x
            || to_axle.x == axle.right()
    );

    let opened = drivetrain_opened(&["rearAxleAssembly"]);
    let hub = centre(port_square(
        &opened,
        "D::AxleAssembly::leftWheel",
        "D::Wheel::hub",
    ));
    let left_drive = polyline(&opened, "D::Vehicle::leftDrive");
    let ends = [left_drive[0], *left_drive.last().unwrap()];
    assert!(
        ends.iter().any(|e| near(*e, hub)),
        "{left_drive:?} reaches {hub:?}"
    );
}

#[test]
fn opened_parts_are_saved_and_unknown_ones_kept() {
    let mut file = DiagramFile::new("wiring", DiagramKind::Interconnection, qn("D::Vehicle"));
    file.layout.opened.insert("rearAxleAssembly".into());
    file.layout.opened.insert("gone".into());

    let reread = DiagramFile::parse(&file.to_text()).unwrap();

    assert_eq!(reread, file);
    assert!(file.to_text().contains(r#""opened": ["#));
    let scene = viewer_core::build_scene_with_layout(
        &common::drivetrain(),
        DiagramKind::Interconnection,
        &qn("D::Vehicle"),
        &file.layout,
    )
    .unwrap();
    assert!(
        scene
            .element_bounds(&qn("D::AxleAssembly::differential"))
            .is_some()
    );
}

#[test]
fn an_opened_drivetrain_has_no_overlapping_labels_and_matches_the_golden_file() {
    let scene = drivetrain_opened(&["rearAxleAssembly"]);
    assert_eq!(overlapping_labels(&scene), Vec::<(String, String)>::new());
    let svg = svg::render(&scene);
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/drivetrain-opened-interconnection.svg");
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
fn two_usages_of_the_same_definition_open_independently() {
    let model = Model::builder()
        .package("P")
        .part_def("P::Cell")
        .part_def("P::Module")
        .part("P::Module::cell")
        .typed_by("P::Cell")
        .part_def("P::Pack")
        .part("P::Pack::front")
        .typed_by("P::Module")
        .part("P::Pack::rear")
        .typed_by("P::Module")
        .build();
    let mut layout = viewer_core::Layout::default();
    layout.opened.insert("front".into());

    let scene = viewer_core::build_scene_with_layout(
        &model,
        DiagramKind::Interconnection,
        &qn("P::Pack"),
        &layout,
    )
    .unwrap();

    let front = scene.element_bounds(&qn("P::Pack::front")).unwrap();
    let rear = scene.element_bounds(&qn("P::Pack::rear")).unwrap();
    let cells: Vec<Rect> = scene
        .items
        .iter()
        .filter(|i| matches!(&i.depicts, Depicts::Part { element, .. } if *element == qn("P::Module::cell")))
        .filter_map(|i| match i.shape {
            Shape::Rect { rect, .. } => Some(rect),
            _ => None,
        })
        .collect();
    assert_eq!(cells.len(), 1, "only the opened module shows its cell");
    assert!(
        cells[0].x > front.x && cells[0].right() < front.right(),
        "inside front"
    );
    assert!(rear.height < front.height, "rear stays closed");
}

#[test]
fn connectors_are_right_angled_and_never_pass_through_a_box_they_dont_connect() {
    let scenes = [
        ("vehicle", interconnection("S::Vehicle")),
        ("drivetrain", drivetrain("D::Vehicle")),
        (
            "drivetrain opened",
            drivetrain_opened(&["rearAxleAssembly"]),
        ),
    ];
    for (name, scene) in scenes {
        let boxes: Vec<(QualifiedName, Rect)> = scene
            .items
            .iter()
            .filter_map(|item| match (&item.depicts, &item.shape) {
                (
                    Depicts::Element(e) | Depicts::Part { element: e, .. },
                    Shape::Rect { rect, .. },
                ) => Some((e.clone(), *rect)),
                _ => None,
            })
            .collect();
        for item in &scene.items {
            let (Depicts::Connection(connection), Shape::Polyline { points, .. }) =
                (&item.depicts, &item.shape)
            else {
                continue;
            };
            let (first, last) = (points[0], *points.last().unwrap());
            for segment in points.windows(2) {
                let (a, b) = (segment[0], segment[1]);
                assert!(
                    a.x == b.x || a.y == b.y,
                    "{name}: {connection} has a slanted segment {a:?} → {b:?}"
                );
                for (element, rect) in &boxes {
                    // A connector may run inside the boxes its ends lie in.
                    if rect.contains(first) || rect.contains(last) {
                        continue;
                    }
                    let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
                    let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
                    let through = x0 < rect.right() - 1.0
                        && x1 > rect.x + 1.0
                        && y0 < rect.bottom() - 1.0
                        && y1 > rect.y + 1.0;
                    assert!(!through, "{name}: {connection} passes through {element}");
                }
            }
        }
    }
}
