//! Seam 1: analysis cases in General Diagrams (slide 99).

mod common;

use std::path::Path;

use viewer_core::scene::{Depicts, Shape};
use viewer_core::{
    DiagramKind, Layout, QualifiedName, Scene, build_general_scene, build_scene, svg,
};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
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

fn package() -> Scene {
    build_scene(&common::analysis(), DiagramKind::General, &qn("A")).unwrap()
}

#[test]
fn an_analysis_def_shows_subject_objective_and_parameters() {
    let texts = texts_of(&package(), "A::MassCheck");

    assert_eq!(
        texts,
        [
            "«analysis def»",
            "MassCheck",
            "subject",
            "c:Component",
            "objective",
            "The mass stays within the limit.",
            "require {result <= limit}",
            "parameters",
            "in limit:Real = 300.0",
            "return result:Real = c.mass",
        ]
    );
}

#[test]
fn an_analysis_usage_shows_its_subject_binding() {
    let texts = texts_of(&package(), "A::carMass");

    assert_eq!(
        texts,
        [
            "«analysis»",
            "carMass:MassCheck",
            "✓",
            "subject",
            "c = car",
            "results",
            "return result:Real = 230.0"
        ]
    );
}

#[test]
fn the_subject_is_no_part_of_the_analysis() {
    let scene = package();

    assert!(!texts_of(&scene, "A::MassCheck").contains(&"parts".to_owned()));
    let from_analysis = scene.edges().into_iter().any(
        |(d, _)| matches!(d, Depicts::Composition { owner, .. } if *owner == qn("A::MassCheck")),
    );
    assert!(!from_analysis, "no composition edge to the subject's type");
}

fn with_car() -> Scene {
    build_general_scene(
        &common::analysis(),
        &[qn("A::car"), qn("A::carMass")],
        &Layout::default(),
    )
}

#[test]
fn a_subject_edge_runs_from_the_analysis_to_its_part() {
    let scene = with_car();

    let (_, path) = scene
        .edges()
        .into_iter()
        .find(|(d, _)| **d == Depicts::Subject(qn("A::carMass")))
        .expect("a «subject» edge");
    let car = scene.element_bounds(&qn("A::car")).unwrap();
    let last = *path.last().unwrap();
    assert!(
        last.x >= car.x - 1.0
            && last.x <= car.right() + 1.0
            && last.y >= car.y - 1.0
            && last.y <= car.bottom() + 1.0
    );
    let edge = Depicts::Subject(qn("A::carMass"));
    let shapes: Vec<&Shape> = scene.items_depicting(&edge).collect();
    assert!(
        shapes
            .iter()
            .any(|s| matches!(s, Shape::Text { text, .. } if text == "«subject»"))
    );
    assert!(
        shapes
            .iter()
            .any(|s| matches!(s, Shape::Polyline { dashed: true, .. })),
        "dashed"
    );
}

#[test]
fn a_saved_route_bends_a_subject_edge() {
    let key = Depicts::Subject(qn("A::carMass")).edge_key().unwrap();
    assert_eq!(key, "subject A::carMass");
    let mut layout = Layout::default();
    layout.edges.insert(key, vec![[600.0, 30.0]]);

    let scene = build_general_scene(
        &common::analysis(),
        &[qn("A::car"), qn("A::carMass")],
        &layout,
    );

    let (_, path) = scene
        .edges()
        .into_iter()
        .find(|(d, _)| **d == Depicts::Subject(qn("A::carMass")))
        .unwrap();
    assert!(path.iter().any(|p| (p.x, p.y) == (600.0, 30.0)));
}

#[test]
fn an_analysis_package_matches_the_golden_file() {
    let svg = svg::render(&package());
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/analysis-package.svg");
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

fn badge(scene: &Scene, element: &str) -> Option<String> {
    let node = Depicts::Element(qn(element));
    let circle = scene.items_depicting(&node).any(
        |s| matches!(s, Shape::Rect { corner_radius, rect, .. } if *corner_radius * 2.0 == rect.width),
    );
    circle.then(|| {
        scene
            .items_depicting(&node)
            .find_map(|s| match s {
                Shape::Text { text, .. } if ["✓", "✗", "?"].contains(&text.as_str()) => {
                    Some(text.clone())
                }
                _ => None,
            })
            .unwrap()
    })
}

/// An analysis `P::a` of a definition with (or without) an objective,
/// with the given result.
fn one(objective: bool, result: Option<viewer_core::model::AnalysisResult>) -> Scene {
    let mut b = viewer_core::Model::builder()
        .package("P")
        .analysis_def("P::D");
    if objective {
        b = b.objective(None, &["x <= 1"]);
    }
    b = b.analysis("P::a").typed_by("P::D");
    if let Some(r) = result {
        let outputs: Vec<(&str, &str)> = r
            .outputs
            .iter()
            .map(|(n, v)| (n.as_str(), v.as_str()))
            .collect();
        b = b.analysed(&outputs, r.objective);
    }
    build_scene(&b.build(), DiagramKind::General, &qn("P")).unwrap()
}

#[test]
fn an_analysis_wears_its_objective_s_outcome() {
    use viewer_core::model::{AnalysisResult, ClaimResult};
    let violated = AnalysisResult {
        outputs: vec![],
        objective: Some(ClaimResult::Violated {
            condition: "x <= 1".to_owned(),
        }),
    };

    assert_eq!(
        badge(&one(true, Some(violated)), "P::a").as_deref(),
        Some("✗")
    );
    assert_eq!(
        badge(&one(true, None), "P::a").as_deref(),
        Some("?"),
        "not run yet"
    );
    assert_eq!(
        badge(&one(false, None), "P::a"),
        None,
        "no objective, no badge"
    );
    assert_eq!(
        badge(&package(), "A::MassCheck"),
        None,
        "a definition isn't run"
    );
}

#[test]
fn properties_list_outputs_and_the_objective() {
    let props =
        viewer_core::properties::properties(&common::analysis(), &qn("A::carMass")).unwrap();
    let rows: Vec<(&str, String)> = props.rows.into_iter().map(|p| (p.label, p.value)).collect();

    assert!(
        rows.contains(&("Output", "result = 230.0".to_owned())),
        "{rows:?}"
    );
    assert!(
        rows.contains(&("Objective", "✓ holds".to_owned())),
        "{rows:?}"
    );
}

#[test]
fn an_analysis_with_its_result_matches_the_golden_file() {
    let svg = svg::render(&with_car());
    let golden = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/analysis-result.svg");
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
fn an_analysis_not_run_says_so_in_properties() {
    let model = viewer_core::Model::builder()
        .package("P")
        .analysis_def("P::D")
        .objective(None, &["x <= 1"])
        .analysis("P::a")
        .typed_by("P::D")
        .build();

    let props = viewer_core::properties::properties(&model, &qn("P::a")).unwrap();

    assert!(
        props
            .rows
            .iter()
            .any(|p| p.label == "Objective" && p.value == "? not run")
    );
}
