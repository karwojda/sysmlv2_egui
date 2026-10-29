//! Seam 1: requirement definitions and usages in General Diagrams.

mod common;

use std::path::Path;

use viewer_core::properties::properties;
use viewer_core::scene::{Depicts, Shape};
use viewer_core::{DiagramKind, QualifiedName, Scene, build_scene, svg};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

fn general(root: &str) -> Scene {
    build_scene(&common::requirements(), DiagramKind::General, &qn(root)).unwrap()
}

fn texts_of(scene: &Scene, element: &str) -> Vec<String> {
    scene
        .items_depicting(&Depicts::Element(qn(element)))
        .filter_map(|shape| match shape {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn a_requirement_def_shows_its_short_id_subject_doc_and_attributes() {
    let scene = general("Req");

    let texts = texts_of(&scene, "Req::MassLimit");

    assert_eq!(
        texts,
        [
            "«requirement def»",
            "<R1> MassLimit",
            "subject v:Vehicle",
            "doc",
            "The vehicle shall weigh no more than the",
            "limit.",
            "attributes",
            "limit:Real",
            "constraints",
            "require {v.mass <= limit}",
        ]
    );
}

#[test]
fn constraints_follow_the_header_when_there_is_no_doc_or_attribute() {
    let scene = general("Req");

    assert_eq!(
        texts_of(&scene, "Req::RangeTarget"),
        [
            "«requirement def»",
            "RangeTarget",
            "subject v:Vehicle",
            "constraints",
            "require {v.range >= 500.0}"
        ]
    );
}

#[test]
fn a_collapsed_requirement_hides_its_constraints() {
    let mut layout = viewer_core::Layout::default();
    layout.collapsed.insert(qn("Req::RangeTarget"));
    let scene = viewer_core::build_scene_with_layout(
        &common::requirements(),
        DiagramKind::General,
        &qn("Req"),
        &layout,
    )
    .unwrap();

    assert_eq!(
        texts_of(&scene, "Req::RangeTarget"),
        ["«requirement def»", "RangeTarget", "subject v:Vehicle"]
    );
}

#[test]
fn properties_list_each_require_constraint() {
    let props = properties(&common::requirements(), &qn("Req::MassLimit")).unwrap();
    let requires: Vec<&str> = props
        .rows
        .iter()
        .filter(|p| p.label == "Requires")
        .map(|p| p.value.as_str())
        .collect();

    assert_eq!(requires, ["{v.mass <= limit}"]);
}

#[test]
fn the_subject_is_not_repeated_as_a_part() {
    let scene = general("Req");

    for requirement in ["Req::MassLimit", "Req::RangeTarget", "Req::vehicleSpec"] {
        let texts = texts_of(&scene, requirement);
        assert!(
            !texts.iter().any(|t| t == "parts" || t == "v:Vehicle"),
            "{requirement}: {texts:?}"
        );
    }
}

#[test]
fn nested_requirements_are_listed_with_their_ids_in_a_package_diagram() {
    let scene = general("Req");

    let texts = texts_of(&scene, "Req::vehicleSpec");

    assert_eq!(
        &texts[..3],
        ["«requirement»", "vehicleSpec", "subject v:Vehicle"]
    );
    let listed = texts
        .iter()
        .position(|t| t == "requirements")
        .expect("a requirements compartment");
    assert_eq!(
        &texts[listed + 1..],
        [
            "<5> carMassReq:MassLimit ✓",
            "<3> costReq:CostCap ?",
            "<1> massReq:MassLimit ✓",
            "<2> rangeReq:RangeTarget ✗",
            "<4> spareReq:RangeTarget ○"
        ]
    );
}

#[test]
fn a_requirement_usage_opens_as_the_top_of_its_tree() {
    let scene = general("Req::vehicleSpec");

    let mut nodes: Vec<&str> = scene.elements().iter().map(|e| e.as_str()).collect();
    nodes.sort();
    assert_eq!(
        nodes,
        [
            "Req::vehicleSpec",
            "Req::vehicleSpec::carMassReq",
            "Req::vehicleSpec::costReq",
            "Req::vehicleSpec::massReq",
            "Req::vehicleSpec::rangeReq",
            "Req::vehicleSpec::spareReq",
        ],
        "the subject is on the header, not a node"
    );
    let top = scene.element_bounds(&qn("Req::vehicleSpec")).unwrap();
    for child in ["carMassReq", "costReq", "massReq", "rangeReq", "spareReq"] {
        let name = format!("Req::vehicleSpec::{child}");
        let composition = Depicts::Composition {
            owner: qn("Req::vehicleSpec"),
            usage: qn(&name),
        };
        assert!(
            scene.edges().iter().any(|(d, _)| **d == composition),
            "◆→ {child}"
        );
        assert!(
            scene.element_bounds(&qn(&name)).unwrap().y > top.bottom(),
            "{child} hangs below"
        );
    }
    assert!(
        !texts_of(&scene, "Req::vehicleSpec")
            .iter()
            .any(|t| t == "requirements"),
        "children drawn as nodes aren't listed again"
    );
}

#[test]
fn a_nested_requirement_node_reads_id_name_and_type_not_bold() {
    let scene = general("Req::vehicleSpec");

    let title = scene
        .items_depicting(&Depicts::Element(qn("Req::vehicleSpec::massReq")))
        .find_map(|s| match s {
            Shape::Text { text, font, .. } if text.contains("massReq") => {
                Some((text.clone(), font.bold))
            }
            _ => None,
        });

    assert_eq!(title, Some(("<1> massReq:MassLimit".to_owned(), false)));
}

#[test]
fn a_part_opens_as_an_interconnection_diagram_and_requirements_as_general_ones() {
    let model = common::requirements();
    let default = |name: &str| DiagramKind::default_for(model.element(&qn(name)).unwrap());

    assert_eq!(default("Req::vehicleSpec"), Some(DiagramKind::General));
    assert_eq!(default("Req::MassLimit"), Some(DiagramKind::General));
    assert_eq!(default("Req::vehicle"), Some(DiagramKind::Interconnection));
    assert!(
        DiagramKind::General.accepts_root(model.element(&qn("Req::vehicle")).unwrap()),
        "a part can still be drawn on top of its own members"
    );
}

#[test]
fn properties_show_the_short_id_and_a_subject_link() {
    let props = properties(&common::requirements(), &qn("Req::MassLimit")).unwrap();
    let row = |label: &str| {
        props
            .rows
            .iter()
            .find(|p| p.label == label)
            .map(|p| (p.value.clone(), p.refers_to.clone()))
    };

    assert_eq!(row("Short id"), Some(("R1".to_owned(), None)));
    assert_eq!(
        row("Subject"),
        Some(("v:Vehicle".to_owned(), Some(qn("Req::MassLimit::v"))))
    );
}

#[test]
fn a_requirement_tree_matches_the_golden_file() {
    let svg = svg::render(&general("Req::vehicleSpec"));
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/requirements-tree.svg");
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
fn a_subject_draws_no_composition_edge_to_its_type() {
    let scene = general("Req");

    let to_vehicle: Vec<String> = scene
        .edges()
        .into_iter()
        .filter_map(|(d, _)| match d {
            Depicts::Composition { owner, usage } => Some(format!("{owner} ◆ {usage}")),
            _ => None,
        })
        .collect();

    assert!(to_vehicle.is_empty(), "{to_vehicle:?}");
}

#[test]
fn a_package_of_requirements_matches_the_golden_file() {
    let svg = svg::render(&general("Req"));
    let golden =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/requirements-package.svg");
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

/// The vehicles and the specification's requirements on one Diagram, as a
/// view exposing both would draw them.
fn parts_and_requirements(layout: &viewer_core::Layout) -> Scene {
    let elements: Vec<QualifiedName> = [
        "Req::vehicle",
        "Req::car",
        "Req::vehicleSpec::massReq",
        "Req::vehicleSpec::rangeReq",
        "Req::vehicleSpec::costReq",
        "Req::vehicleSpec::spareReq",
        "Req::vehicleSpec::carMassReq",
    ]
    .iter()
    .map(|n| qn(n))
    .collect();
    viewer_core::build_general_scene(&common::requirements(), &elements, layout)
}

fn satisfy_edges(scene: &Scene) -> Vec<String> {
    let mut edges: Vec<String> = scene
        .edges()
        .into_iter()
        .filter_map(|(d, _)| match d {
            Depicts::Satisfy(claim) => Some(claim.to_string()),
            _ => None,
        })
        .collect();
    edges.sort();
    edges
}

#[test]
fn each_claim_is_a_satisfy_edge_from_the_satisfier_to_the_requirement() {
    let scene = parts_and_requirements(&viewer_core::Layout::default());

    assert_eq!(
        satisfy_edges(&scene),
        ["Req::@10", "Req::@8", "Req::@9", "Req::car::@1"]
    );
    let (_, path) = scene
        .edges()
        .into_iter()
        .find(|(d, _)| **d == Depicts::Satisfy(qn("Req::@9")))
        .unwrap();
    let vehicle = scene.element_bounds(&qn("Req::vehicle")).unwrap();
    let range = scene
        .element_bounds(&qn("Req::vehicleSpec::rangeReq"))
        .unwrap();
    let near = |r: viewer_core::scene::Rect, p: viewer_core::scene::Point| {
        p.x >= r.x - 1.0 && p.x <= r.right() + 1.0 && p.y >= r.y - 1.0 && p.y <= r.bottom() + 1.0
    };
    assert!(near(vehicle, path[0]), "starts at the satisfier");
    assert!(
        near(range, *path.last().unwrap()),
        "ends at the requirement"
    );
}

#[test]
fn a_satisfy_edge_is_labelled_and_ends_in_an_open_arrowhead() {
    let scene = parts_and_requirements(&viewer_core::Layout::default());
    let claim = Depicts::Satisfy(qn("Req::@8"));

    let shapes: Vec<&Shape> = scene.items_depicting(&claim).collect();

    assert!(
        shapes
            .iter()
            .any(|s| matches!(s, Shape::Text { text, .. } if text == "«satisfy» ✓"))
    );
    assert_eq!(
        shapes
            .iter()
            .filter(|s| matches!(s, Shape::Polyline { .. }))
            .count(),
        2,
        "the line and an open (unfilled) arrowhead"
    );
    assert!(
        !shapes.iter().any(|s| matches!(s, Shape::Polygon { .. })),
        "no filled head"
    );
}

#[test]
fn no_satisfy_edge_without_both_ends_on_the_diagram() {
    let scene = general("Req");

    assert!(
        satisfy_edges(&scene).is_empty(),
        "the nested requirements aren't nodes here"
    );
}

#[test]
fn a_satisfier_lists_its_claims_in_its_requirements_compartment() {
    let scene = general("Req");

    let vehicle = texts_of(&scene, "Req::vehicle");
    let at = vehicle
        .iter()
        .position(|t| t == "requirements")
        .expect("a requirements compartment");
    assert_eq!(
        &vehicle[at + 1..],
        [
            "satisfy massReq ✓",
            "satisfy rangeReq ✗",
            "satisfy costReq ?"
        ]
    );
    let car = texts_of(&scene, "Req::car");
    assert!(
        car.ends_with(&["requirements".to_owned(), "satisfy carMassReq ✓".to_owned()]),
        "{car:?}"
    );
}

#[test]
fn a_saved_route_bends_a_satisfy_edge() {
    let mut layout = viewer_core::Layout::default();
    let key = Depicts::Satisfy(qn("Req::@8")).edge_key().unwrap();
    assert_eq!(key, "satisfy Req::@8");
    layout.edges.insert(key, vec![[700.0, 40.0]]);

    let scene = parts_and_requirements(&layout);

    let (_, path) = scene
        .edges()
        .into_iter()
        .find(|(d, _)| **d == Depicts::Satisfy(qn("Req::@8")))
        .unwrap();
    assert!(path.iter().any(|p| (p.x, p.y) == (700.0, 40.0)));
}

#[test]
fn properties_link_requirements_and_satisfiers_both_ways() {
    let model = common::requirements();
    let rows = |element: &str, label: &str| -> Vec<String> {
        properties(&model, &qn(element))
            .unwrap()
            .rows
            .into_iter()
            .filter(|p| p.label == label)
            .map(|p| p.value)
            .collect()
    };

    assert_eq!(
        rows("Req::vehicleSpec::massReq", "Satisfied by"),
        ["Req::vehicle ✓"]
    );
    assert_eq!(
        rows("Req::vehicle", "Satisfies"),
        [
            "Req::vehicleSpec::massReq ✓",
            "Req::vehicleSpec::rangeReq ✗",
            "Req::vehicleSpec::costReq ?"
        ]
    );
    assert_eq!(
        rows("Req::@9", "Requirement"),
        ["Req::vehicleSpec::rangeReq"]
    );
    assert_eq!(rows("Req::@9", "Satisfier"), ["Req::vehicle"]);
    assert!(rows("Req::vehicleSpec::spareReq", "Satisfied by").is_empty());
}

#[test]
fn a_claim_is_named_by_what_it_claims() {
    let model = common::requirements();

    assert_eq!(
        model.element(&qn("Req::car::@1")).unwrap().display_name(),
        "carMassReq by car"
    );
}

#[test]
fn parts_with_satisfy_edges_match_the_golden_file() {
    let svg = svg::render(&parts_and_requirements(&viewer_core::Layout::default()));
    let golden =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/requirements-satisfy.svg");
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
fn a_feature_s_value_follows_it_in_its_compartment_row() {
    let scene = general("Req");

    let vehicle = texts_of(&scene, "Req::vehicle");
    assert!(vehicle.contains(&"mass = 1800.0".to_owned()), "{vehicle:?}");
    assert!(vehicle.contains(&"range = 420.0".to_owned()), "{vehicle:?}");
    let tree = general("Req::vehicleSpec");
    assert!(texts_of(&tree, "Req::vehicleSpec::massReq").contains(&"limit = 2000.0".to_owned()));
}

#[test]
fn a_roll_up_reads_as_written_and_a_default_says_so() {
    let model = viewer_core::Model::builder()
        .package("P")
        .part_def("P::Car")
        .attribute("P::Car::mass")
        .value("engine.mass + body.mass")
        .attribute("P::Car::doors")
        .value("default 4")
        .build();

    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    let car = texts_of(&scene, "P::Car");
    assert!(
        car.contains(&"mass = engine.mass + body.mass".to_owned()),
        "{car:?}"
    );
    assert!(car.contains(&"doors default 4".to_owned()), "{car:?}");
}

#[test]
fn properties_show_a_feature_s_value() {
    let props = properties(&common::requirements(), &qn("Req::vehicle::mass")).unwrap();

    assert!(
        props
            .rows
            .iter()
            .any(|p| p.label == "Value" && p.value == "1800.0")
    );
}
