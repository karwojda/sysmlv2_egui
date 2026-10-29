//! A parent whose children would make a row wider than a page stacks its
//! leaf children in a column beside the composition trunk.

use viewer_core::diagram::build_general_scene;
use viewer_core::diagram_file::{Layout, NodePlacement};
use viewer_core::model::{ExposeScope, ModelBuilder};
use viewer_core::scene::{Depicts, Fill, Point, Rect, Scene, Shape};
use viewer_core::view::{ViewContents, resolve};
use viewer_core::{Model, QualifiedName};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

/// The page-width limit rows wrap at.
const LIMIT: f32 = 900.0;

const FILE: &str = "model.sysml";

/// `P::spec` with the nested requirements `children`, in that order, each
/// with `grandchildren` of its own; and a tree view of it all.
fn tree(children: &[(&str, usize)]) -> ModelBuilder {
    let mut builder = Model::builder()
        .package("P")
        .at(FILE, 1)
        .requirement("P::spec")
        .at(FILE, 2);
    let mut line = 3;
    for (child, grandchildren) in children {
        builder = builder
            .requirement(&format!("P::spec::{child}"))
            .at(FILE, line);
        line += 1;
        for g in 0..*grandchildren {
            builder = builder
                .requirement(&format!("P::spec::{child}::g{g}"))
                .at(FILE, line);
            line += 1;
        }
    }
    builder
        .view("P::tree")
        .exposes("P::spec", ExposeScope::ElementAndDescendants)
        .renders("Views::asTreeDiagram")
}

fn draw(model: &Model, layout: &Layout) -> Scene {
    let ViewContents::General(shown) = resolve(model, &qn("P::tree")).unwrap() else {
        panic!("a General Diagram");
    };
    build_general_scene(model, &shown, layout)
}

fn rect(scene: &Scene, name: &str) -> Rect {
    scene.element_bounds(&qn(name)).unwrap()
}

fn names(prefix: &str, count: usize) -> Vec<String> {
    (1..=count).map(|i| format!("{prefix}{i}")).collect()
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

/// The lines and diamonds drawn for `spec`'s compositions.
fn composition_shapes(scene: &Scene) -> (Vec<Vec<Point>>, usize) {
    let spec = qn("P::spec");
    let mine = |d: &Depicts| match d {
        Depicts::Composition { owner, .. } | Depicts::CompositionTree { owner } => *owner == spec,
        _ => false,
    };
    let mut lines = Vec::new();
    let mut diamonds = 0;
    for item in scene.items.iter().filter(|i| mine(&i.depicts)) {
        match &item.shape {
            Shape::Polyline { points, .. } => lines.push(points.clone()),
            Shape::Polygon {
                fill: Fill::Stroke,
                points,
            } if points.len() == 4 => diamonds += 1,
            _ => {}
        }
    }
    (lines, diamonds)
}

#[test]
fn eleven_leaves_too_wide_for_a_row_stack_in_one_column_in_model_order() {
    let leaves = names("s", 11);
    let children: Vec<(&str, usize)> = leaves.iter().map(|s| (s.as_str(), 0)).collect();
    let scene = draw(&tree(&children).build(), &Layout::default());

    let rects: Vec<Rect> = leaves
        .iter()
        .map(|s| rect(&scene, &format!("P::spec::{s}")))
        .collect();
    for pair in rects.windows(2) {
        assert!(close(pair[0].x, pair[1].x), "one column: {pair:?}");
        assert!(pair[0].bottom() < pair[1].y, "in Model order: {pair:?}");
    }
    assert!(scene.width <= LIMIT, "{} wide", scene.width);
}

#[test]
fn a_stacked_column_hangs_from_the_trunk_by_branches_into_left_edges() {
    let leaves = names("s", 11);
    let children: Vec<(&str, usize)> = leaves.iter().map(|s| (s.as_str(), 0)).collect();
    let scene = draw(&tree(&children).build(), &Layout::default());

    let (lines, diamonds) = composition_shapes(&scene);
    assert_eq!(diamonds, 1);
    let parent = rect(&scene, "P::spec");
    let tip = scene
        .items_depicting(&Depicts::CompositionTree {
            owner: qn("P::spec"),
        })
        .find_map(|s| match s {
            Shape::Polygon { points, .. } => Some(points[0]),
            _ => None,
        })
        .unwrap();
    assert!(
        close(tip.x, parent.x + parent.width / 2.0) && close(tip.y, parent.bottom()),
        "the diamond {tip:?} at the bottom middle of {parent:?}"
    );
    for line in &lines {
        for s in line.windows(2) {
            assert!(close(s[0].x, s[1].x) || close(s[0].y, s[1].y), "{s:?}");
        }
    }
    for s in &leaves {
        let r = rect(&scene, &format!("P::spec::{s}"));
        let enters = lines.iter().any(|l| {
            let end = *l.last().unwrap();
            close(end.x, r.x) && end.y > r.y && end.y < r.bottom()
        });
        assert!(enters, "a branch into {s}'s left edge");
    }
}

#[test]
fn three_narrow_leaves_stay_in_a_row() {
    let scene = draw(
        &tree(&[("a", 0), ("b", 0), ("c", 0)]).build(),
        &Layout::default(),
    );

    let ys: Vec<f32> = ["a", "b", "c"]
        .iter()
        .map(|c| rect(&scene, &format!("P::spec::{c}")).y)
        .collect();
    assert!(ys.iter().all(|y| close(*y, ys[0])), "{ys:?}");
}

#[test]
fn leaves_stack_left_of_children_with_subtrees_which_stay_in_a_row() {
    let children = [
        ("l1", 0),
        ("t1", 2),
        ("l2", 0),
        ("l3", 0),
        ("t2", 2),
        ("l4", 0),
        ("l5", 0),
        ("l6", 0),
    ];
    let scene = draw(&tree(&children).build(), &Layout::default());

    let r = |n: &str| rect(&scene, &format!("P::spec::{n}"));
    let column_right = ["l1", "l2", "l3", "l4", "l5", "l6"]
        .iter()
        .map(|l| r(l).right())
        .fold(f32::MIN, f32::max);
    for leaf in ["l2", "l3", "l4", "l5", "l6"] {
        assert!(close(r(leaf).x, r("l1").x), "{leaf} in the column");
    }
    assert!(close(r("t1").y, r("t2").y), "the subtrees side by side");
    assert!(r("t1").x > column_right && r("t2").x > r("t1").right());
}

#[test]
fn a_child_placed_by_hand_keeps_its_place() {
    let leaves = names("s", 11);
    let children: Vec<(&str, usize)> = leaves.iter().map(|s| (s.as_str(), 0)).collect();
    let mut layout = Layout::default();
    layout.nodes.insert(
        qn("P::spec::s3"),
        NodePlacement {
            x: 700.0,
            y: 40.0,
            size: None,
        },
    );
    let scene = draw(&tree(&children).build(), &layout);

    let s3 = rect(&scene, "P::spec::s3");
    assert!(close(s3.x, 700.0) && close(s3.y, 40.0), "{s3:?}");
}

#[test]
fn a_subtree_s_own_wide_row_of_leaves_stacks_too() {
    let scene = draw(&tree(&[("t1", 11), ("l1", 0)]).build(), &Layout::default());

    let t1 = rect(&scene, "P::spec::t1");
    let column: Vec<Rect> = (0..11)
        .map(|g| rect(&scene, &format!("P::spec::t1::g{g}")))
        .collect();
    for pair in column.windows(2) {
        assert!(close(pair[0].x, pair[1].x), "one column: {pair:?}");
        assert!(pair[0].bottom() < pair[1].y, "in Model order: {pair:?}");
    }
    assert!(column[0].x > t1.x + t1.width / 2.0, "right of t1's trunk");
}
