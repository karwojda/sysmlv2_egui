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
            "<3> costReq:CostCap",
            "<1> massReq:MassLimit",
            "<2> rangeReq:RangeTarget",
            "<4> spareReq:RangeTarget"
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
            "Req::vehicleSpec::costReq",
            "Req::vehicleSpec::massReq",
            "Req::vehicleSpec::rangeReq",
            "Req::vehicleSpec::spareReq",
        ],
        "the subject is on the header, not a node"
    );
    let top = scene.element_bounds(&qn("Req::vehicleSpec")).unwrap();
    for child in ["costReq", "massReq", "rangeReq", "spareReq"] {
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
fn only_packages_definitions_and_requirement_usages_can_be_general_roots() {
    let model = common::requirements();
    let accepts = |name: &str| DiagramKind::General.accepts_root(model.element(&qn(name)).unwrap());

    assert!(accepts("Req::vehicleSpec"));
    assert!(accepts("Req::MassLimit"));
    assert!(
        !accepts("Req::vehicle"),
        "a part opens as an Interconnection Diagram"
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
