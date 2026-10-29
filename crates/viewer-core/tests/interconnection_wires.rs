//! Each wire of an Interconnection Diagram can be followed on its own: no
//! segment shared with another wire, labels apart, a flow's arrowhead at
//! its own target, and no route through a box that isn't one of its ends.

use viewer_core::diagram::{build_scene, build_scene_with_layout};
use viewer_core::diagram_file::{Layout, NodePlacement};
use viewer_core::model::ModelBuilder;
use viewer_core::scene::{Depicts, Point, Rect, Scene, Shape};
use viewer_core::{DiagramKind, Model, QualifiedName};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

/// Parts `a : A` and `b : B` inside `V`, with ports `A::p1..p3` and
/// `B::q1..q3`.
fn two_parts() -> ModelBuilder {
    let mut builder = Model::builder()
        .package("P")
        .port_def("P::D")
        .part_def("P::A")
        .part_def("P::B");
    for i in 1..=3 {
        builder = builder
            .port(&format!("P::A::p{i}"))
            .typed_by("P::D")
            .port(&format!("P::B::q{i}"))
            .typed_by("P::D");
    }
    builder
        .part_def("P::V")
        .part("P::V::a")
        .typed_by("P::A")
        .part("P::V::b")
        .typed_by("P::B")
}

fn line(scene: &Scene, wire: &str) -> Vec<Point> {
    scene
        .items_depicting(&Depicts::Connection(qn(wire)))
        .find_map(|s| match s {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{wire} is drawn"))
}

fn label(scene: &Scene, wire: &str) -> Rect {
    scene
        .items_depicting(&Depicts::Connection(qn(wire)))
        .find(|s| matches!(s, Shape::Text { .. }))
        .map(Shape::extent)
        .unwrap_or_else(|| panic!("{wire} is labelled"))
}

fn arrow_tip(scene: &Scene, wire: &str) -> Point {
    scene
        .items_depicting(&Depicts::Connection(qn(wire)))
        .find_map(|s| match s {
            Shape::Polygon { points, .. } => Some(points[0]),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{wire} has an arrowhead"))
}

/// Whether two axis-aligned segments run along each other for more than a
/// point.
fn share(a: (Point, Point), b: (Point, Point)) -> bool {
    let overlap = |(a0, a1): (f32, f32), (b0, b1): (f32, f32)| {
        a0.max(a1).min(b0.max(b1)) - a0.min(a1).max(b0.min(b1)) > 1.0
    };
    let vertical = |s: (Point, Point)| (s.0.x - s.1.x).abs() < 0.5;
    let horizontal = |s: (Point, Point)| (s.0.y - s.1.y).abs() < 0.5;
    (vertical(a)
        && vertical(b)
        && (a.0.x - b.0.x).abs() < 1.0
        && overlap((a.0.y, a.1.y), (b.0.y, b.1.y)))
        || (horizontal(a)
            && horizontal(b)
            && (a.0.y - b.0.y).abs() < 1.0
            && overlap((a.0.x, a.1.x), (b.0.x, b.1.x)))
}

/// The segments of `points` away from its ends: the stubs leaving a port
/// are the port's own.
fn runs(points: &[Point]) -> Vec<(Point, Point)> {
    let segments: Vec<(Point, Point)> = points.windows(2).map(|s| (s[0], s[1])).collect();
    if segments.len() > 2 {
        segments[1..segments.len() - 1].to_vec()
    } else {
        segments
    }
}

fn assert_no_shared_runs(scene: &Scene, wires: &[&str]) {
    for (i, a) in wires.iter().enumerate() {
        for b in &wires[i + 1..] {
            for sa in runs(&line(scene, a)) {
                for sb in runs(&line(scene, b)) {
                    assert!(!share(sa, sb), "{a} {sa:?} runs along {b} {sb:?}");
                }
            }
        }
    }
}

#[test]
fn two_opposing_flows_between_two_ports_are_two_routes_with_their_own_arrows() {
    let model = two_parts()
        .flow("P::V::there")
        .end(&["P::V::a", "P::A::p1"])
        .end(&["P::V::b", "P::B::q1"])
        .flow("P::V::back")
        .end(&["P::V::b", "P::B::q1"])
        .end(&["P::V::a", "P::A::p1"])
        .build();
    let scene = build_scene(&model, DiagramKind::Interconnection, &qn("P::V")).unwrap();

    assert_no_shared_runs(&scene, &["P::V::there", "P::V::back"]);
    assert!(!label(&scene, "P::V::there").intersects(&label(&scene, "P::V::back")));
    for wire in ["P::V::there", "P::V::back"] {
        let points = line(&scene, wire);
        let target = *points.last().unwrap();
        let tip = arrow_tip(&scene, wire);
        assert!(
            (tip.x - target.x).abs() < 8.0 && (tip.y - target.y).abs() < 8.0,
            "{wire}'s arrow {tip:?} is at its target {target:?}"
        );
    }
}

#[test]
fn three_wires_between_neighbouring_ports_share_no_segment() {
    let mut builder = two_parts();
    for i in 1..=3 {
        builder = builder
            .connection(&format!("P::V::c{i}"))
            .end(&["P::V::a", &format!("P::A::p{i}")])
            .end(&["P::V::b", &format!("P::B::q{}", 4 - i)]);
    }
    let scene = build_scene(&builder.build(), DiagramKind::Interconnection, &qn("P::V")).unwrap();

    assert_no_shared_runs(&scene, &["P::V::c1", "P::V::c2", "P::V::c3"]);
}

#[test]
fn a_wire_whose_straight_route_crosses_a_third_box_goes_around_it() {
    let model = two_parts()
        .part_def("P::C")
        .part("P::V::c")
        .typed_by("P::C")
        .connection("P::V::link")
        .end(&["P::V::a", "P::A::p1"])
        .end(&["P::V::b", "P::B::q1"])
        .build();
    // `c` stands between `a` and `b`.
    let mut layout = Layout::default();
    let at = |x: f32, y: f32| NodePlacement { x, y, size: None };
    layout.nodes.insert(qn("a"), at(40.0, 80.0));
    layout.nodes.insert(qn("c"), at(40.0, 240.0));
    layout.nodes.insert(qn("b"), at(40.0, 400.0));
    let scene = build_scene_with_layout(&model, DiagramKind::Interconnection, &qn("P::V"), &layout)
        .unwrap();

    let c = scene
        .items
        .iter()
        .find_map(|i| match (&i.depicts, &i.shape) {
            (Depicts::Part { key, .. }, Shape::Rect { rect, .. }) if *key == qn("c") => Some(*rect),
            _ => None,
        })
        .expect("c is drawn");
    let points = line(&scene, "P::V::link");
    for s in points.windows(2) {
        let crosses = (0..=50).any(|k| {
            let t = k as f32 / 50.0;
            let p = Point::new(
                s[0].x + (s[1].x - s[0].x) * t,
                s[0].y + (s[1].y - s[0].y) * t,
            );
            p.x > c.x && p.x < c.right() && p.y > c.y && p.y < c.bottom()
        });
        assert!(!crosses, "{s:?} crosses c {c:?}");
    }
}
