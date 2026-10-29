//! A parent's composition edges to children below it are drawn as one
//! tree: one diamond at the parent's bottom centre, a trunk, a bar, and a
//! branch down to the top middle of each child.

use viewer_core::diagram::{build_scene, build_scene_with_layout};
use viewer_core::diagram_file::Layout;
use viewer_core::scene::{Depicts, Fill, Point, Rect, Scene, Shape};
use viewer_core::{DiagramKind, Model, QualifiedName};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

/// A composite requirement with three nested requirements.
fn requirements() -> Model {
    Model::builder()
        .package("P")
        .requirement("P::spec")
        .requirement("P::spec::a")
        .requirement("P::spec::b")
        .requirement("P::spec::c")
        .build()
}

/// A part def composing three part defs through named, counted usages.
fn parts() -> Model {
    Model::builder()
        .package("P")
        .part_def("P::Car")
        .part("P::Car::engine")
        .typed_by("P::Engine")
        .part("P::Car::wheels")
        .typed_by("P::Wheel")
        .multiplicity("4", "4")
        .part("P::Car::body")
        .typed_by("P::Body")
        .part_def("P::Engine")
        .part_def("P::Wheel")
        .part_def("P::Body")
        .build()
}

fn is_composition(depicts: &Depicts) -> bool {
    matches!(
        depicts,
        Depicts::Composition { .. } | Depicts::CompositionTree { .. }
    )
}

/// The filled four-point polygons drawn for compositions.
fn diamonds(scene: &Scene) -> Vec<Vec<Point>> {
    scene
        .items
        .iter()
        .filter(|i| is_composition(&i.depicts))
        .filter_map(|i| match &i.shape {
            Shape::Polygon {
                points,
                fill: Fill::Stroke,
            } if points.len() == 4 => Some(points.clone()),
            _ => None,
        })
        .collect()
}

fn lines(scene: &Scene) -> Vec<Vec<Point>> {
    scene
        .items
        .iter()
        .filter(|i| is_composition(&i.depicts))
        .filter_map(|i| match &i.shape {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .collect()
}

fn bounds(scene: &Scene, name: &str) -> Rect {
    scene.element_bounds(&qn(name)).unwrap()
}

fn close(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.5
}

#[test]
fn a_parent_with_three_children_has_one_diamond_at_its_bottom_centre() {
    let scene = build_scene(&requirements(), DiagramKind::General, &qn("P::spec")).unwrap();

    let diamonds = diamonds(&scene);
    assert_eq!(diamonds.len(), 1, "{diamonds:?}");
    let parent = bounds(&scene, "P::spec");
    let tip = diamonds[0][0];
    assert!(
        close(tip.x, parent.x + parent.width / 2.0) && close(tip.y, parent.bottom()),
        "{tip:?} on {parent:?}"
    );
}

#[test]
fn the_tree_is_orthogonal_and_each_branch_ends_at_its_child_s_top_middle() {
    let scene = build_scene(&requirements(), DiagramKind::General, &qn("P::spec")).unwrap();

    for line in lines(&scene) {
        for segment in line.windows(2) {
            assert!(
                close(segment[0].x, segment[1].x) || close(segment[0].y, segment[1].y),
                "{segment:?}"
            );
        }
    }
    for child in ["P::spec::a", "P::spec::b", "P::spec::c"] {
        let rect = bounds(&scene, child);
        let top_middle = Point::new(rect.x + rect.width / 2.0, rect.y);
        let branch = scene
            .items
            .iter()
            .filter(|i| {
                i.depicts
                    == Depicts::Composition {
                        owner: qn("P::spec"),
                        usage: qn(child),
                    }
            })
            .find_map(|i| match &i.shape {
                Shape::Polyline { points, .. } => Some(points.clone()),
                _ => None,
            })
            .expect("a branch of its own, selectable as its edge");
        let end = *branch.last().unwrap();
        assert!(
            close(end.x, top_middle.x) && close(end.y, top_middle.y),
            "{child}: {end:?} vs {top_middle:?}"
        );
    }
}

#[test]
fn a_definition_level_composition_keeps_its_label_beside_its_branch() {
    let scene = build_scene(&parts(), DiagramKind::General, &qn("P")).unwrap();

    assert_eq!(diamonds(&scene).len(), 1);
    let labels: Vec<String> = scene
        .items
        .iter()
        .filter(|i| matches!(i.depicts, Depicts::Composition { .. }))
        .filter_map(|i| match &i.shape {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(labels.contains(&"wheels [4]".to_owned()), "{labels:?}");
    assert!(labels.contains(&"engine".to_owned()), "{labels:?}");
}

#[test]
fn a_single_child_hangs_straight_below_its_parent() {
    let model = Model::builder()
        .package("P")
        .requirement("P::spec")
        .requirement("P::spec::only")
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P::spec")).unwrap();

    assert_eq!(diamonds(&scene).len(), 1);
    for line in lines(&scene) {
        for segment in line.windows(2) {
            assert!(close(segment[0].x, segment[1].x), "vertical: {segment:?}");
        }
    }
}

#[test]
fn an_edge_with_bends_set_by_the_user_is_drawn_on_its_own() {
    let plain = build_scene(&requirements(), DiagramKind::General, &qn("P::spec")).unwrap();
    let rect = bounds(&plain, "P::spec::a");
    let mut layout = Layout::default();
    let key = Depicts::Composition {
        owner: qn("P::spec"),
        usage: qn("P::spec::a"),
    }
    .edge_key()
    .unwrap();
    layout
        .edges
        .insert(key, vec![[rect.x - 30.0, rect.y - 10.0]]);

    let scene = build_scene_with_layout(
        &requirements(),
        DiagramKind::General,
        &qn("P::spec"),
        &layout,
    )
    .unwrap();

    assert_eq!(diamonds(&scene).len(), 2, "the tree's, and its own");
    let own = scene
        .items
        .iter()
        .filter(|i| {
            i.depicts
                == Depicts::Composition {
                    owner: qn("P::spec"),
                    usage: qn("P::spec::a"),
                }
        })
        .find_map(|i| match &i.shape {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .unwrap();
    assert!(
        own.iter()
            .any(|p| close(p.x, rect.x - 30.0) && close(p.y, rect.y - 10.0))
    );
}
