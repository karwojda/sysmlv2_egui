//! General Diagrams draw the children of one parent left to right in the
//! order the Model declares them, not by name.

use viewer_core::diagram::build_scene;
use viewer_core::model::ModelBuilder;
use viewer_core::scene::Scene;
use viewer_core::{DiagramKind, Model, QualifiedName};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

const FILE: &str = "model.sysml";

/// Left to right, the nodes of `names`.
fn left_to_right(scene: &Scene, names: &[String]) -> Vec<String> {
    let mut placed: Vec<(f32, String)> = names
        .iter()
        .map(|n| (scene.element_bounds(&qn(n)).unwrap().x, n.clone()))
        .collect();
    placed.sort_by(|a, b| a.0.total_cmp(&b.0));
    placed.into_iter().map(|(_, n)| n).collect()
}

/// A composite requirement `spec` with children declared in `order`, one
/// per line.
fn composite(order: &[&str]) -> (Model, Vec<String>) {
    let mut builder: ModelBuilder = Model::builder()
        .package("P")
        .at(FILE, 1)
        .requirement("P::spec")
        .at(FILE, 2);
    let mut names = Vec::new();
    for (i, child) in order.iter().enumerate() {
        let name = format!("P::spec::{child}");
        builder = builder.requirement(&name).at(FILE, 3 + i as u32);
        names.push(name);
    }
    (builder.build(), names)
}

#[test]
fn a_composite_requirement_s_children_are_drawn_in_declaration_order() {
    let (model, declared) = composite(&["c", "a", "b"]);

    let scene = build_scene(&model, DiagramKind::General, &qn("P::spec")).unwrap();

    assert_eq!(left_to_right(&scene, &declared), declared);
}

#[test]
fn eleven_children_keep_their_order() {
    let order = [
        "s1", "s10", "s11", "s2", "s3", "s4", "s5", "s6", "s7", "s8", "s9",
    ];
    let mut sorted = order.to_vec();
    sorted.sort_by_key(|s| s[1..].parse::<u32>().unwrap());
    let (model, declared) = composite(&sorted);

    let scene = build_scene(&model, DiagramKind::General, &qn("P::spec")).unwrap();

    assert_eq!(left_to_right(&scene, &declared), declared);
}

#[test]
fn a_definition_s_parts_are_drawn_in_declaration_order() {
    let model = Model::builder()
        .package("P")
        .at(FILE, 1)
        .part_def("P::Vehicle")
        .at(FILE, 2)
        .part("P::Vehicle::wheels")
        .at(FILE, 3)
        .part("P::Vehicle::body")
        .at(FILE, 4)
        .part("P::Vehicle::engine")
        .at(FILE, 5)
        .build();
    let declared: Vec<String> = ["wheels", "body", "engine"]
        .iter()
        .map(|p| format!("P::Vehicle::{p}"))
        .collect();

    let scene = build_scene(&model, DiagramKind::General, &qn("P::Vehicle")).unwrap();

    assert_eq!(left_to_right(&scene, &declared), declared);
}

#[test]
fn members_declared_on_one_line_keep_their_order() {
    let model = Model::builder()
        .package("P")
        .at(FILE, 1)
        .requirement("P::spec")
        .at(FILE, 2)
        .requirement("P::spec::z")
        .at_column(FILE, 3, 5)
        .requirement("P::spec::y")
        .at_column(FILE, 3, 30)
        .requirement("P::spec::x")
        .at_column(FILE, 3, 60)
        .build();
    let declared: Vec<String> = ["z", "y", "x"]
        .iter()
        .map(|c| format!("P::spec::{c}"))
        .collect();

    let scene = build_scene(&model, DiagramKind::General, &qn("P::spec")).unwrap();

    assert_eq!(left_to_right(&scene, &declared), declared);
}

#[test]
fn members_without_a_location_follow_by_name() {
    let model = Model::builder()
        .package("P")
        .part_def("P::D")
        .part("P::D::b")
        .part("P::D::a")
        .part("P::D::z")
        .at(FILE, 9)
        .build();
    let names: Vec<String> = ["z", "a", "b"]
        .iter()
        .map(|c| format!("P::D::{c}"))
        .collect();

    let scene = build_scene(&model, DiagramKind::General, &qn("P::D")).unwrap();

    assert_eq!(left_to_right(&scene, &names), names);
}
