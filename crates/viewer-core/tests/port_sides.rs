//! A port in an Interconnection Diagram sits on the side of its box that
//! faces the other end of its wire, whatever its direction: left or right
//! only when that end is level with the box, else top or bottom. Unwired
//! ports go to the side with the fewest wired ones.

use viewer_core::model::{Direction, ModelBuilder};
use viewer_core::scene::{Depicts, Point, Rect, Scene, Shape};
use viewer_core::{
    DiagramKind, Layout, Model, NodePlacement, QualifiedName, build_scene_with_layout,
};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

#[derive(Debug, PartialEq)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

/// A `System` holding parts `a` and `b` of definitions `A` and `B`, after
/// `wiring` has added the ports and connections.
fn system(wiring: impl FnOnce(ModelBuilder) -> ModelBuilder) -> Model {
    let builder = Model::builder()
        .package("P")
        .port_def("P::Link")
        .part_def("P::A")
        .part_def("P::B")
        .part_def("P::System")
        .part("P::System::a")
        .typed_by("P::A")
        .part("P::System::b")
        .typed_by("P::B");
    wiring(builder).build()
}

/// The diagram of `System` with each named part pinned as given, relative
/// to the Frame.
fn diagram(model: &Model, placements: &[(&str, NodePlacement)]) -> Scene {
    let mut layout = Layout::default();
    for (part, placement) in placements {
        layout.nodes.insert(qn(part), *placement);
    }
    build_scene_with_layout(
        model,
        DiagramKind::Interconnection,
        &qn("P::System"),
        &layout,
    )
    .unwrap()
}

/// A part 140 wide and `height` tall with its top-left corner at `x`, `y`.
fn at(x: f32, y: f32, height: f32) -> NodePlacement {
    NodePlacement {
        x,
        y,
        size: Some([140.0, height]),
    }
}

/// The side of `part`'s box that the port `key` (`a.p`) sits on.
fn side(scene: &Scene, part: &str, key: &str) -> Side {
    let rect = scene.node_bounds(&qn(part)).unwrap();
    let port = scene
        .items
        .iter()
        .find_map(|i| match (&i.depicts, &i.shape) {
            (Depicts::Port { key: k, .. }, Shape::Rect { rect, .. }) if k == key => {
                Some(centre(*rect))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("port {key} is drawn"));
    on_side(rect, port).unwrap_or_else(|| panic!("{key} at {port:?} is on no side of {rect:?}"))
}

fn centre(r: Rect) -> Point {
    Point::new(r.x + r.width / 2.0, r.y + r.height / 2.0)
}

fn on_side(rect: Rect, p: Point) -> Option<Side> {
    let near = |a: f32, b: f32| (a - b).abs() < 0.5;
    if near(p.y, rect.y) {
        Some(Side::Top)
    } else if near(p.y, rect.bottom()) {
        Some(Side::Bottom)
    } else if near(p.x, rect.x) {
        Some(Side::Left)
    } else if near(p.x, rect.right()) {
        Some(Side::Right)
    } else {
        None
    }
}

/// `a.p` wired to `b.q`.
fn linked() -> Model {
    system(|b| {
        b.port("P::A::p")
            .typed_by("P::Link")
            .port("P::B::q")
            .typed_by("P::Link")
            .connection("P::System::link")
            .end(&["P::System::a", "P::A::p"])
            .end(&["P::System::b", "P::B::q"])
    })
}

#[test]
fn ports_of_parts_in_one_row_face_each_other_sideways() {
    let scene = diagram(
        &linked(),
        &[("a", at(20.0, 40.0, 60.0)), ("b", at(420.0, 60.0, 60.0))],
    );

    assert_eq!(side(&scene, "a", "a.p"), Side::Right);
    assert_eq!(side(&scene, "b", "b.q"), Side::Left);
}

#[test]
fn a_port_whose_peer_is_below_and_aside_faces_down_however_far_aside() {
    // b is wholly below a but 400 to its right, far closer to level than
    // not; a side port would make the wire jog round a corner.
    let scene = diagram(
        &linked(),
        &[("a", at(20.0, 40.0, 60.0)), ("b", at(420.0, 130.0, 60.0))],
    );

    assert_eq!(side(&scene, "a", "a.p"), Side::Bottom);
    assert_eq!(side(&scene, "b", "b.q"), Side::Top);
}

#[test]
fn parts_of_different_heights_in_one_row_face_each_other_sideways() {
    // b is tall: its centre is below a, but the two boxes share a row.
    let scene = diagram(
        &linked(),
        &[("a", at(20.0, 40.0, 60.0)), ("b", at(420.0, 60.0, 240.0))],
    );

    assert_eq!(side(&scene, "a", "a.p"), Side::Right);
    assert_eq!(side(&scene, "b", "b.q"), Side::Left);
}

#[test]
fn a_directed_port_faces_its_peer_too() {
    let model = system(|b| {
        b.port("P::A::input")
            .typed_by("P::Link")
            .direction(Direction::In)
            .port("P::B::output")
            .typed_by("P::Link")
            .direction(Direction::Out)
            .connection("P::System::link")
            .end(&["P::System::b", "P::B::output"])
            .end(&["P::System::a", "P::A::input"])
    });

    // b, the source, sits below a.
    let scene = diagram(
        &model,
        &[("a", at(20.0, 40.0, 60.0)), ("b", at(20.0, 340.0, 60.0))],
    );

    assert_eq!(side(&scene, "a", "a.input"), Side::Bottom);
    assert_eq!(side(&scene, "b", "b.output"), Side::Top);
}

#[test]
fn unwired_ports_go_together_off_the_side_the_wired_ones_crowd() {
    let model = system(|mut b| {
        for n in 1..=3 {
            b = b
                .port(&format!("P::A::p{n}"))
                .typed_by("P::Link")
                .port(&format!("P::B::q{n}"))
                .typed_by("P::Link")
                .connection(&format!("P::System::link{n}"))
                .end(&["P::System::a", &format!("P::A::p{n}")])
                .end(&["P::System::b", &format!("P::B::q{n}")]);
        }
        b.port("P::A::u1")
            .typed_by("P::Link")
            .port("P::A::u2")
            .typed_by("P::Link")
    });

    let scene = diagram(
        &model,
        &[("a", at(20.0, 40.0, 60.0)), ("b", at(20.0, 340.0, 60.0))],
    );

    for n in 1..=3 {
        assert_eq!(side(&scene, "a", &format!("a.p{n}")), Side::Bottom);
    }
    let unwired = side(&scene, "a", "a.u1");
    assert_ne!(unwired, Side::Bottom);
    assert_eq!(side(&scene, "a", "a.u2"), unwired, "together on one side");
}

#[test]
fn a_box_with_only_unwired_ports_keeps_them_at_the_bottom() {
    let model = system(|b| {
        b.port("P::A::u1")
            .typed_by("P::Link")
            .port("P::A::u2")
            .typed_by("P::Link")
    });

    let scene = diagram(&model, &[]);

    assert_eq!(side(&scene, "a", "a.u1"), Side::Bottom);
    assert_eq!(side(&scene, "a", "a.u2"), Side::Bottom);
}
