//! Seam 1: a saved Layout merged with the Model when a Diagram is built.

use viewer_core::scene::{Depicts, Point, Rect, Shape};
use viewer_core::{
    DiagramFile, DiagramKind, Layout, Model, NodePlacement, QualifiedName, Scene, build_scene,
    build_scene_with_layout,
};

fn model() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .part("P::Vehicle::engine")
        .typed_by("P::Engine")
        .attribute("P::Vehicle::mass")
        .part_def("P::Engine")
        .part_def("P::Car")
        .specializes("P::Vehicle")
        .part_def("P::Truck")
        .specializes("P::Vehicle")
        .build()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn build(layout: &Layout) -> Scene {
    build_scene_with_layout(&model(), DiagramKind::General, &qn("P"), layout).unwrap()
}

fn placed(x: f32, y: f32, width: f32, height: f32) -> NodePlacement {
    NodePlacement {
        x,
        y,
        size: Some([width, height]),
    }
}

fn bounds(scene: &Scene, name: &str) -> Rect {
    scene.node_bounds(&qn(name)).unwrap()
}

#[test]
fn an_empty_layout_changes_nothing() {
    let automatic = build_scene(&model(), DiagramKind::General, &qn("P")).unwrap();

    assert_eq!(build(&Layout::default()), automatic);
}

#[test]
fn saved_nodes_are_drawn_exactly_where_and_as_big_as_saved() {
    let mut layout = Layout::default();
    layout
        .nodes
        .insert(qn("P::Vehicle"), placed(300.0, 40.0, 260.0, 140.0));

    let scene = build(&layout);

    assert_eq!(
        bounds(&scene, "P::Vehicle"),
        Rect {
            x: 300.0,
            y: 40.0,
            width: 260.0,
            height: 140.0
        }
    );
}

#[test]
fn new_nodes_are_placed_below_saved_ones_without_moving_or_overlapping_them() {
    let mut layout = Layout::default();
    layout
        .nodes
        .insert(qn("P::Vehicle"), placed(300.0, 40.0, 260.0, 140.0));
    layout
        .nodes
        .insert(qn("P::Engine"), placed(40.0, 60.0, 150.0, 60.0));

    let scene = build(&layout);

    let saved = [bounds(&scene, "P::Vehicle"), bounds(&scene, "P::Engine")];
    assert_eq!(saved[0].x, 300.0);
    assert_eq!(saved[1].x, 40.0);
    for new in ["P::Car", "P::Truck"] {
        let rect = bounds(&scene, new);
        assert!(rect.y > saved[0].bottom(), "{new} is below the saved nodes");
        assert!(
            saved.iter().all(|s| !s.intersects(&rect)),
            "{new} overlaps a saved node"
        );
    }
    assert!(!bounds(&scene, "P::Car").intersects(&bounds(&scene, "P::Truck")));
}

#[test]
fn a_node_saved_without_a_size_keeps_its_natural_size() {
    let automatic = build_scene(&model(), DiagramKind::General, &qn("P")).unwrap();
    let natural = bounds(&automatic, "P::Vehicle");
    let mut layout = Layout::default();
    layout.nodes.insert(
        qn("P::Vehicle"),
        NodePlacement {
            x: 500.0,
            y: 30.0,
            size: None,
        },
    );

    let moved = bounds(&build(&layout), "P::Vehicle");

    assert_eq!((moved.x, moved.y), (500.0, 30.0));
    assert_eq!((moved.width, moved.height), (natural.width, natural.height));
}

#[test]
fn the_scene_grows_to_hold_saved_nodes() {
    let mut layout = Layout::default();
    layout
        .nodes
        .insert(qn("P::Engine"), placed(1500.0, 900.0, 150.0, 60.0));

    let scene = build(&layout);

    assert!(scene.width >= 1650.0 && scene.height >= 960.0);
}

#[test]
fn entries_for_elements_the_model_lacks_are_ignored_and_reported_as_orphaned() {
    let mut layout = Layout::default();
    layout
        .nodes
        .insert(qn("P::Renamed"), placed(10.0, 10.0, 100.0, 50.0));
    layout.hidden.insert(qn("P::Gone"));
    layout
        .nodes
        .insert(qn("P::Engine"), placed(40.0, 60.0, 150.0, 60.0));

    let scene = build(&layout);

    assert!(scene.node_bounds(&qn("P::Renamed")).is_none());
    assert_eq!(
        layout.orphaned(&model()),
        [&qn("P::Gone"), &qn("P::Renamed")]
    );
}

#[test]
fn orphaned_entries_survive_a_save_round_trip() {
    let mut file = DiagramFile::new("p", DiagramKind::General, qn("P"));
    file.layout
        .nodes
        .insert(qn("P::Renamed"), placed(10.0, 10.0, 100.0, 50.0));

    let reread = DiagramFile::parse(&file.to_text()).unwrap();

    assert_eq!(reread, file);
}

#[test]
fn hidden_elements_and_their_edges_are_left_out() {
    let mut layout = Layout::default();
    layout.hidden.insert(qn("P::Vehicle"));

    let scene = build(&layout);

    assert!(scene.node_bounds(&qn("P::Vehicle")).is_none());
    let edge = Depicts::Specialization {
        specific: qn("P::Car"),
        general: qn("P::Vehicle"),
    };
    assert_eq!(scene.items_depicting(&edge).count(), 0);
    let composition = Depicts::Composition {
        owner: qn("P::Vehicle"),
        usage: qn("P::Vehicle::engine"),
    };
    assert_eq!(scene.items_depicting(&composition).count(), 0);
}

#[test]
fn collapsed_elements_are_drawn_without_compartments() {
    let mut layout = Layout::default();
    layout.collapsed.insert(qn("P::Vehicle"));

    let scene = build(&layout);

    let vehicle = Depicts::Element(qn("P::Vehicle"));
    let texts: Vec<&str> = scene
        .items_depicting(&vehicle)
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(texts, ["«part def»", "Vehicle"]);
}

#[test]
fn saved_edge_routes_bend_through_their_points() {
    let edge = Depicts::Specialization {
        specific: qn("P::Car"),
        general: qn("P::Vehicle"),
    };
    let mut layout = Layout::default();
    layout.edges.insert(
        edge.edge_key().unwrap(),
        vec![[700.0, 500.0], [720.0, 30.0]],
    );

    let scene = build(&layout);

    let line = scene
        .items_depicting(&edge)
        .find_map(|s| match s {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        line[1..3],
        [Point::new(700.0, 500.0), Point::new(720.0, 30.0)]
    );
}

#[test]
fn edges_without_a_saved_route_stay_straight() {
    let edge = Depicts::Specialization {
        specific: qn("P::Truck"),
        general: qn("P::Vehicle"),
    };

    let scene = build(&Layout::default());

    let line = scene
        .items_depicting(&edge)
        .find_map(|s| match s {
            Shape::Polyline { points, .. } => Some(points.len()),
            _ => None,
        })
        .unwrap();
    assert_eq!(line, 2);
}

#[test]
fn a_layout_is_stored_one_sorted_entry_per_element() {
    let mut file = DiagramFile::new("p", DiagramKind::General, qn("P"));
    file.layout
        .nodes
        .insert(qn("P::Vehicle"), placed(300.0, 40.0, 260.0, 140.0));
    file.layout.nodes.insert(
        qn("P::Engine"),
        NodePlacement {
            x: 40.5,
            y: 60.0,
            size: None,
        },
    );
    file.layout.hidden.insert(qn("P::Truck"));

    assert_eq!(
        file.to_text(),
        r#"{
  "format_version": 1,
  "name": "p",
  "kind": "general",
  "root": "P",
  "layout": {
    "nodes": {
      "P::Engine": {
        "x": 40.5,
        "y": 60.0
      },
      "P::Vehicle": {
        "x": 300.0,
        "y": 40.0,
        "size": [
          260.0,
          140.0
        ]
      }
    },
    "hidden": [
      "P::Truck"
    ]
  }
}
"#
    );
}
