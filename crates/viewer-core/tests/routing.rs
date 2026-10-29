//! Seam 1: edges route around nodes they would cross, and ends crowding the
//! same spot on a border are spread apart. Saved routes are left alone.

use viewer_core::scene::{Depicts, Point, Rect, Shape};
use viewer_core::{
    DiagramKind, Layout, Model, NodePlacement, QualifiedName, Scene, build_scene_with_layout,
};

fn model() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Vehicle")
        .part("P::Vehicle::tanks")
        .typed_by("P::Tank")
        .part_def("P::Tank")
        .part("P::vehicle")
        .typed_by("P::Vehicle")
        .build()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn at(x: f32, y: f32) -> NodePlacement {
    NodePlacement { x, y, size: None }
}

/// `Tank` sits right between `vehicle` and its definition `Vehicle`, so the
/// straight defined-by edge would pass through it.
fn blocked() -> Layout {
    let mut layout = Layout::default();
    layout.nodes.insert(qn("P::Vehicle"), at(100.0, 20.0));
    layout.nodes.insert(qn("P::Tank"), at(100.0, 200.0));
    layout.nodes.insert(qn("P::vehicle"), at(100.0, 400.0));
    layout
}

fn build(layout: &Layout) -> Scene {
    build_scene_with_layout(&model(), DiagramKind::General, &qn("P"), layout).unwrap()
}

fn line(scene: &Scene, depicts: &Depicts) -> Vec<Point> {
    scene
        .items_depicting(depicts)
        .find_map(|s| match s {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .unwrap()
}

/// Whether any point sampled along the polyline lies strictly inside `rect`.
fn passes_through(points: &[Point], rect: Rect) -> bool {
    points.windows(2).any(|s| {
        (1..100).any(|i| {
            let t = i as f32 / 100.0;
            let p = Point::new(
                s[0].x + (s[1].x - s[0].x) * t,
                s[0].y + (s[1].y - s[0].y) * t,
            );
            p.x > rect.x && p.x < rect.right() && p.y > rect.y && p.y < rect.bottom()
        })
    })
}

fn defined_by() -> Depicts {
    Depicts::Typing {
        usage: qn("P::vehicle"),
        definition: qn("P::Vehicle"),
    }
}

#[test]
fn an_edge_that_would_cross_a_node_bends_around_it() {
    let scene = build(&blocked());
    let tank = scene.node_bounds(&qn("P::Tank")).unwrap();

    let path = line(&scene, &defined_by());

    assert!(path.len() >= 3, "it bends: {path:?}");
    assert!(!passes_through(&path, tank), "it clears Tank: {path:?}");
}

#[test]
fn a_saved_route_is_kept_even_if_it_crosses_a_node() {
    let mut layout = blocked();
    layout
        .edges
        .insert(defined_by().edge_key().unwrap(), vec![[150.0, 300.0]]);

    let path = line(&build(&layout), &defined_by());

    assert_eq!(path[1], Point::new(150.0, 300.0));
    assert_eq!(path.len(), 3);
}

#[test]
fn edge_ends_on_the_same_side_of_a_node_are_spread_apart() {
    // Tank and vehicle both sit straight below Vehicle, so the composition
    // to Tank and the defined-by edge (after its detour) would leave
    // Vehicle's bottom close together.
    let scene = build(&blocked());
    let vehicle = scene.node_bounds(&qn("P::Vehicle")).unwrap();
    let composition = Depicts::Composition {
        owner: qn("P::Vehicle"),
        usage: qn("P::Vehicle::tanks"),
    };

    let ends: Vec<Point> = [
        scene
            .items_depicting(&composition)
            .chain(scene.items_depicting(&Depicts::CompositionTree {
                owner: qn("P::Vehicle"),
            }))
            .find_map(|s| match s {
                Shape::Polygon { points, .. } => Some(points[0]),
                _ => None,
            })
            .unwrap(),
        scene
            .items_depicting(&defined_by())
            .find_map(|s| match s {
                Shape::Polygon { points, .. } => Some(points[0]),
                _ => None,
            })
            .unwrap(),
    ]
    .into();

    for end in &ends {
        assert_eq!(end.y, vehicle.bottom(), "{end:?} is on Vehicle's bottom");
    }
    assert!(
        (ends[0].x - ends[1].x).abs() >= 14.0,
        "{ends:?} are spread apart"
    );
}
