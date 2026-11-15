//! Seam 1: editing an Interconnection Diagram's Layout, checked through
//! the scene it produces and the Diagram File it saves.

mod common;

use std::sync::Arc;

use viewer_core::editing::{Alignment, Dimension, Edit};
use viewer_core::scene::{Depicts, Point, Rect, Shape};
use viewer_core::{
    DiagramFile, DiagramKind, Layout, NodePlacement, OpenDiagram, PortSide, QualifiedName, Scene,
    layout_key_exists,
};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

/// The drivetrain's Interconnection Diagram as saved with `layout`.
fn file_with(layout: Layout) -> DiagramFile {
    let mut file = DiagramFile::new("wiring", DiagramKind::Interconnection, qn("D::Vehicle"));
    file.layout = layout;
    file
}

/// The drivetrain's Interconnection Diagram, with the parts `opened`
/// opened in place.
fn open(opened: &[&str]) -> OpenDiagram {
    let mut layout = Layout::default();
    layout.opened.extend(opened.iter().map(|o| o.to_string()));
    OpenDiagram::open_saved(Arc::new(common::drivetrain()), None, file_with(layout))
}

fn scene(diagram: &OpenDiagram) -> &Scene {
    diagram.scene().unwrap()
}

fn part(diagram: &OpenDiagram, path: &str) -> Rect {
    scene(diagram)
        .node_bounds(&qn(path))
        .unwrap_or_else(|| panic!("{path} is drawn"))
}

fn port(diagram: &OpenDiagram, key: &str) -> Point {
    scene(diagram)
        .items
        .iter()
        .find_map(|i| match (&i.depicts, &i.shape) {
            (Depicts::Port { key: k, .. }, Shape::Rect { rect, .. }) if k == key => Some(
                Point::new(rect.x + rect.width / 2.0, rect.y + rect.height / 2.0),
            ),
            _ => None,
        })
        .unwrap_or_else(|| panic!("port {key} is drawn"))
}

/// The Diagram File it would save, read back from its text.
fn saved(diagram: &OpenDiagram) -> DiagramFile {
    DiagramFile::parse(&file_with(diagram.layout().clone()).to_text()).unwrap()
}

#[test]
fn parts_are_keyed_by_feature_path() {
    let diagram = open(&["rearAxleAssembly"]);

    assert!(scene(&diagram).node_bounds(&qn("engine")).is_some());
    assert!(
        scene(&diagram)
            .node_bounds(&qn("rearAxleAssembly.differential"))
            .is_some()
    );
    assert_eq!(
        scene(&diagram).element_of(&qn("rearAxleAssembly.differential")),
        Some(&qn("D::AxleAssembly::differential"))
    );
}

#[test]
fn moving_a_part_saves_its_position_relative_to_its_box() {
    let mut diagram = open(&[]);
    let frame = scene(&diagram).node_bounds(&qn("D::Vehicle")).unwrap();
    let engine = part(&diagram, "engine");

    diagram.edit(&Edit::Move {
        elements: vec![qn("engine")],
        dx: 40.0,
        dy: 10.0,
    });

    assert_eq!(
        (part(&diagram, "engine").x, part(&diagram, "engine").y),
        (engine.x + 40.0, engine.y + 10.0)
    );
    let saved = saved(&diagram).layout.nodes[&qn("engine")];
    assert_eq!(
        (saved.x, saved.y),
        (engine.x + 40.0 - frame.x, engine.y + 10.0 - frame.y)
    );
}

#[test]
fn moving_an_opened_part_moves_what_is_inside_it() {
    let mut diagram = open(&["rearAxleAssembly"]);
    let differential = part(&diagram, "rearAxleAssembly.differential");

    diagram.edit(&Edit::Move {
        elements: vec![qn("rearAxleAssembly")],
        dx: 60.0,
        dy: 30.0,
    });

    let moved = part(&diagram, "rearAxleAssembly.differential");
    assert_eq!(
        (moved.x, moved.y),
        (differential.x + 60.0, differential.y + 30.0)
    );
}

#[test]
fn parts_moved_together_take_the_bends_of_wires_between_them() {
    let mut diagram = open(&["rearAxleAssembly"]);
    let between = Depicts::Connection(qn("D::AxleAssembly::toLeft"))
        .edge_key()
        .unwrap();
    let outward = Depicts::Connection(qn("D::AxleAssembly::toRight"))
        .edge_key()
        .unwrap();
    let wheel = part(&diagram, "rearAxleAssembly.leftWheel");
    let other = part(&diagram, "rearAxleAssembly.rightWheel");
    diagram.edit(&Edit::Route {
        edge: between.clone(),
        bends: vec![[wheel.x - 20.0, wheel.y - 20.0]],
    });
    diagram.edit(&Edit::Route {
        edge: outward.clone(),
        bends: vec![[other.x - 20.0, other.y - 20.0]],
    });
    let (bend, kept) = (
        diagram.layout().edges[&between][0],
        diagram.layout().edges[&outward][0],
    );
    let differential = part(&diagram, "rearAxleAssembly.differential");

    diagram.edit(&Edit::Move {
        elements: vec![
            qn("rearAxleAssembly.differential"),
            qn("rearAxleAssembly.leftWheel"),
        ],
        dx: 30.0,
        dy: 20.0,
    });

    let moved = part(&diagram, "rearAxleAssembly.differential");
    assert_eq!(
        (moved.x, moved.y),
        (differential.x + 30.0, differential.y + 20.0)
    );
    assert_eq!(
        diagram.layout().edges[&between],
        vec![[bend[0] + 30.0, bend[1] + 20.0]]
    );
    assert_eq!(
        diagram.layout().edges[&outward],
        vec![kept],
        "its wheel stayed"
    );
}

#[test]
fn parts_inside_a_box_align_and_take_the_same_size() {
    let mut diagram = open(&["rearAxleAssembly"]);
    let wheels = vec![
        qn("rearAxleAssembly.leftWheel"),
        qn("rearAxleAssembly.rightWheel"),
    ];
    diagram.edit(&Edit::Move {
        elements: vec![qn("rearAxleAssembly.rightWheel")],
        dx: 0.0,
        dy: 45.0,
    });
    diagram.edit(&Edit::Resize {
        element: qn("rearAxleAssembly.leftWheel"),
        width: 150.0,
        height: 90.0,
    });

    diagram.edit(&Edit::Align {
        elements: wheels.clone(),
        reference: qn("rearAxleAssembly.leftWheel"),
        to: Alignment::Top,
    });
    diagram.edit(&Edit::MatchSize {
        elements: wheels,
        reference: qn("rearAxleAssembly.leftWheel"),
        dimension: Dimension::Both,
    });

    let (left, right) = (
        part(&diagram, "rearAxleAssembly.leftWheel"),
        part(&diagram, "rearAxleAssembly.rightWheel"),
    );
    assert_eq!(right.y, left.y);
    assert_eq!((right.width, right.height), (left.width, left.height));
}

#[test]
fn a_part_cannot_leave_its_box_and_the_box_grows_to_hold_it() {
    let mut diagram = open(&["rearAxleAssembly"]);
    let axle = part(&diagram, "rearAxleAssembly");

    diagram.edit(&Edit::Move {
        elements: vec![qn("rearAxleAssembly.leftWheel")],
        dx: -5000.0,
        dy: -5000.0,
    });
    let wheel = part(&diagram, "rearAxleAssembly.leftWheel");
    assert!(
        wheel.x >= axle.x && wheel.y >= axle.y,
        "stays inside its box"
    );

    diagram.edit(&Edit::Move {
        elements: vec![qn("rearAxleAssembly.leftWheel")],
        dx: 400.0,
        dy: 600.0,
    });
    let wheel = part(&diagram, "rearAxleAssembly.leftWheel");
    let axle = part(&diagram, "rearAxleAssembly");
    assert!(
        wheel.right() < axle.right() && wheel.bottom() < axle.bottom(),
        "the box grew around it"
    );
}

#[test]
fn hiding_a_part_drops_it_and_its_connections() {
    let mut diagram = open(&[]);

    diagram.edit(&Edit::Hide(qn("controller")));

    assert!(scene(&diagram).node_bounds(&qn("controller")).is_none());
    assert_eq!(
        scene(&diagram)
            .items_depicting(&Depicts::Connection(qn("D::Vehicle::engineStatus")))
            .count(),
        0
    );
    assert!(saved(&diagram).layout.hidden.contains(&qn("controller")));
}

#[test]
fn a_port_moved_by_hand_stays_where_it_was_put() {
    let mut diagram = open(&[]);
    let engine = part(&diagram, "engine");

    diagram.edit(&Edit::MovePort {
        port: "engine.status".into(),
        side: PortSide::Right,
        offset: 20.0,
    });

    assert_eq!(
        port(&diagram, "engine.status"),
        Point::new(engine.right(), engine.y + 20.0)
    );
    assert_eq!(
        saved(&diagram).layout.ports["engine.status"].side,
        PortSide::Right
    );
}

#[test]
fn opening_and_closing_a_part_by_edit() {
    let mut diagram = open(&[]);

    diagram.edit(&Edit::Open("rearAxleAssembly".into()));
    assert!(
        scene(&diagram)
            .node_bounds(&qn("rearAxleAssembly.differential"))
            .is_some()
    );

    diagram.edit(&Edit::Close("rearAxleAssembly".into()));
    assert!(
        scene(&diagram)
            .node_bounds(&qn("rearAxleAssembly.differential"))
            .is_none()
    );
}

#[test]
fn saved_bends_take_precedence_and_stay_right_angled() {
    let mut diagram = open(&[]);
    let key = Depicts::Connection(qn("D::Vehicle::torque"))
        .edge_key()
        .unwrap();

    diagram.edit(&Edit::Route {
        edge: key,
        bends: vec![[100.0, 350.0]],
    });

    let line = scene(&diagram)
        .items_depicting(&Depicts::Connection(qn("D::Vehicle::torque")))
        .find_map(|s| match s {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .unwrap();
    assert!(line.contains(&Point::new(100.0, 350.0)), "{line:?}");
    for segment in line.windows(2) {
        assert!(
            segment[0].x == segment[1].x || segment[0].y == segment[1].y,
            "{segment:?}"
        );
    }
}

#[test]
fn reset_keeps_orphaned_entries_and_opened_parts() {
    let mut layout = Layout::default();
    layout.opened.insert("rearAxleAssembly".to_string());
    layout.nodes.insert(
        qn("renamedPart"),
        NodePlacement {
            x: 1.0,
            y: 2.0,
            size: None,
        },
    );
    let mut diagram =
        OpenDiagram::open_saved(Arc::new(common::drivetrain()), None, file_with(layout));

    diagram.edit(&Edit::Move {
        elements: vec![qn("engine")],
        dx: 50.0,
        dy: 0.0,
    });
    diagram.edit(&Edit::MovePort {
        port: "engine.status".into(),
        side: PortSide::Left,
        offset: 10.0,
    });
    diagram.edit(&Edit::Reset);

    assert!(
        diagram.layout().nodes.contains_key(&qn("renamedPart")),
        "orphaned entry kept"
    );
    assert!(!diagram.layout().nodes.contains_key(&qn("engine")));
    assert!(diagram.layout().ports.is_empty());
    assert!(
        diagram.layout().opened.contains("rearAxleAssembly"),
        "opened parts stay open"
    );
}

#[test]
fn feature_paths_that_no_longer_lead_anywhere_are_unknown() {
    let model = common::drivetrain();
    let root = qn("D::Vehicle");
    let exists =
        |path: &str| layout_key_exists(&model, DiagramKind::Interconnection, &root, &qn(path));

    assert!(exists("engine"));
    assert!(exists("rearAxleAssembly.leftWheel"));
    assert!(!exists("rearAxleAssembly.nothing"));
    assert!(!exists("gone"));
}
