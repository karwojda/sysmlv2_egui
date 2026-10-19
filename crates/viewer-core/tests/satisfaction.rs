//! Seam 1: Satisfaction Status, derived from claim results, and how it is
//! drawn and explained.

mod common;

use viewer_core::model::{ClaimResult, Model, ModelBuilder};
use viewer_core::properties::properties;
use viewer_core::satisfaction::{SatisfactionStatus, status};
use viewer_core::scene::{Depicts, Shape};
use viewer_core::{DiagramKind, QualifiedName, Scene, build_scene};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

fn violated(condition: &str) -> ClaimResult {
    ClaimResult::Violated {
        condition: condition.to_owned(),
    }
}

fn undecided(reason: &str) -> ClaimResult {
    ClaimResult::Undecided {
        reason: reason.to_owned(),
    }
}

/// A requirement `P::r`, two parts, and the claims `add` puts in.
fn model(add: impl FnOnce(ModelBuilder) -> ModelBuilder) -> Model {
    let builder = Model::builder()
        .package("P")
        .requirement_def("P::R")
        .requirement("P::r")
        .typed_by("P::R")
        .part("P::a")
        .part("P::b");
    add(builder).build()
}

fn status_of(model: &Model) -> Option<SatisfactionStatus> {
    status(model, &qn("P::r"))
}

#[test]
fn a_requirement_nothing_claims_is_not_claimed() {
    assert_eq!(
        status_of(&model(|b| b)),
        Some(SatisfactionStatus::NotClaimed)
    );
}

#[test]
fn a_requirement_whose_claims_all_hold_is_satisfied() {
    let m = model(|b| {
        b.satisfy("P::@1", "P::r", "P::a")
            .result(ClaimResult::Holds)
            .satisfy("P::@2", "P::r", "P::b")
            .result(ClaimResult::Holds)
    });

    assert_eq!(status_of(&m), Some(SatisfactionStatus::Satisfied));
}

#[test]
fn one_violated_claim_among_holding_ones_makes_it_violated() {
    let m = model(|b| {
        b.satisfy("P::@1", "P::r", "P::a")
            .result(ClaimResult::Holds)
            .satisfy("P::@2", "P::r", "P::b")
            .result(violated("b.x <= 1"))
    });

    assert_eq!(status_of(&m), Some(SatisfactionStatus::Violated));
}

#[test]
fn violated_wins_over_undecided_and_undecided_over_satisfied() {
    let violated_and_undecided = model(|b| {
        b.satisfy("P::@1", "P::r", "P::a")
            .result(undecided("no value"))
            .satisfy("P::@2", "P::r", "P::b")
            .result(violated("b.x <= 1"))
    });
    assert_eq!(
        status_of(&violated_and_undecided),
        Some(SatisfactionStatus::Violated)
    );

    let undecided_and_holding = model(|b| {
        b.satisfy("P::@1", "P::r", "P::a")
            .result(ClaimResult::Holds)
            .satisfy("P::@2", "P::r", "P::b")
            .result(undecided("no value"))
    });
    assert_eq!(
        status_of(&undecided_and_holding),
        Some(SatisfactionStatus::Undecided)
    );
}

#[test]
fn an_unchecked_or_not_yet_checked_claim_counts_as_undecided() {
    let unchecked = model(|b| {
        b.satisfy("P::@1", "P::r", "P::a")
            .result(ClaimResult::Unchecked {
                reason: "service down".to_owned(),
            })
    });
    assert_eq!(status_of(&unchecked), Some(SatisfactionStatus::Undecided));

    let never = model(|b| b.satisfy("P::@1", "P::r", "P::a"));
    assert_eq!(status_of(&never), Some(SatisfactionStatus::Undecided));
}

#[test]
fn only_requirement_usages_have_a_status() {
    let m = model(|b| b);

    assert_eq!(
        status(&m, &qn("P::R")),
        None,
        "a definition can't be satisfied"
    );
    assert_eq!(status(&m, &qn("P::a")), None);
}

#[test]
fn the_fixture_covers_every_status() {
    let m = common::requirements();
    let of = |name: &str| status(&m, &qn(&format!("Req::vehicleSpec::{name}"))).unwrap();

    assert_eq!(of("massReq"), SatisfactionStatus::Satisfied);
    assert_eq!(of("rangeReq"), SatisfactionStatus::Violated);
    assert_eq!(of("costReq"), SatisfactionStatus::Undecided);
    assert_eq!(of("carMassReq"), SatisfactionStatus::Undecided);
    assert_eq!(of("spareReq"), SatisfactionStatus::NotClaimed);
}

fn badge_of(scene: &Scene, element: &str) -> Option<String> {
    let node = Depicts::Element(qn(element));
    let shapes: Vec<&Shape> = scene.items_depicting(&node).collect();
    let circle = shapes.iter().any(|s| matches!(s, Shape::Rect { corner_radius, rect, .. } if *corner_radius * 2.0 == rect.width));
    circle.then(|| {
        shapes
            .iter()
            .find_map(|s| match s {
                Shape::Text { text, .. } if ["✓", "✗", "?", "○"].contains(&text.as_str()) => {
                    Some(text.clone())
                }
                _ => None,
            })
            .unwrap()
    })
}

#[test]
fn requirement_usages_wear_their_status_badge_and_definitions_none() {
    let tree = build_scene(
        &common::requirements(),
        DiagramKind::General,
        &qn("Req::vehicleSpec"),
    )
    .unwrap();

    for (name, mark) in [
        ("massReq", "✓"),
        ("rangeReq", "✗"),
        ("costReq", "?"),
        ("carMassReq", "?"),
        ("spareReq", "○"),
    ] {
        assert_eq!(
            badge_of(&tree, &format!("Req::vehicleSpec::{name}")).as_deref(),
            Some(mark),
            "{name}"
        );
    }
    assert_eq!(
        badge_of(&tree, "Req::vehicleSpec").as_deref(),
        Some("○"),
        "nothing claims the specification itself"
    );
    let package = build_scene(&common::requirements(), DiagramKind::General, &qn("Req")).unwrap();
    assert_eq!(badge_of(&package, "Req::MassLimit"), None);
}

#[test]
fn the_badge_sits_in_the_top_right_corner_clear_of_the_title() {
    let tree = build_scene(
        &common::requirements(),
        DiagramKind::General,
        &qn("Req::vehicleSpec"),
    )
    .unwrap();
    let node = Depicts::Element(qn("Req::vehicleSpec::rangeReq"));
    let rects: Vec<viewer_core::scene::Rect> = tree
        .items_depicting(&node)
        .filter_map(|s| match s {
            Shape::Rect { rect, .. } => Some(*rect),
            _ => None,
        })
        .collect();
    let (body, badge) = (rects[0], rects[1]);
    let title = tree
        .items_depicting(&node)
        .find(|s| matches!(s, Shape::Text { text, .. } if text.contains("rangeReq")))
        .unwrap()
        .extent();

    assert!(
        badge.right() < body.right() && badge.y > body.y,
        "inside the box"
    );
    assert!(badge.x > body.center_x(), "on the right");
    assert!(title.right() < badge.x, "the title stays clear of it");
}

#[test]
fn properties_explain_the_status_and_each_claim() {
    let m = common::requirements();
    let rows = |element: &str| -> Vec<(&'static str, String)> {
        properties(&m, &qn(element))
            .unwrap()
            .rows
            .into_iter()
            .map(|p| (p.label, p.value))
            .collect()
    };

    let range = rows("Req::vehicleSpec::rangeReq");
    assert!(range.contains(&("Status", "✗ Violated".to_owned())));
    assert!(range.contains(&("Satisfied by", "Req::vehicle ✗".to_owned())));
    assert!(range.contains(&("Failing condition", "v.range >= 500.0".to_owned())));

    let cost = rows("Req::vehicleSpec::costReq");
    assert!(cost.contains(&("Status", "? Undecided".to_owned())));
    assert!(
        cost.iter()
            .any(|(l, v)| *l == "Undecided because" && v.ends_with("no value for feature v.cost"))
    );

    assert!(rows("Req::vehicleSpec::spareReq").contains(&("Status", "○ Not claimed".to_owned())));
    assert!(rows("Req::@9").contains(&("Result", "✗ violated: v.range >= 500.0".to_owned())));
    assert!(!rows("Req::MassLimit").iter().any(|(l, _)| *l == "Status"));
}

mod styles {
    use super::*;
    use viewer_core::DiagramStyle;
    use viewer_core::scene::{Fill, Rgb};
    use viewer_core::style::{Condition, Satisfaction, Style, StyleRule, apply_styles, rule_for};

    const RED: Rgb = Rgb(0xf4, 0xcc, 0xcc);
    const GREEN: Rgb = Rgb(0xd9, 0xea, 0xd3);
    const AMBER: Rgb = Rgb(0xff, 0xe5, 0x99);
    const GREY: Rgb = Rgb(0xee, 0xee, 0xee);
    const NAVY: Rgb = Rgb(0x1f, 0x2a, 0x44);

    fn on(status: Satisfaction, fill: Rgb) -> StyleRule {
        StyleRule {
            condition: Condition::Satisfaction(status),
            fill,
            label: None,
        }
    }

    fn traffic_lights() -> Vec<StyleRule> {
        vec![
            on(Satisfaction::Satisfied, GREEN),
            on(Satisfaction::Violated, RED),
            on(Satisfaction::Undecided, AMBER),
            on(Satisfaction::NotClaimed, GREY),
        ]
    }

    fn fill_of(model: &Model, element: &str, rules: &[StyleRule]) -> Option<Rgb> {
        rule_for(model, model.element(&qn(element)).unwrap(), &[rules]).map(|r| r.fill)
    }

    #[test]
    fn a_satisfaction_rule_matches_requirement_usages_by_status() {
        let m = common::requirements();
        let rules = traffic_lights();
        let of = |name: &str| fill_of(&m, &format!("Req::vehicleSpec::{name}"), &rules);

        assert_eq!(of("massReq"), Some(GREEN));
        assert_eq!(of("rangeReq"), Some(RED));
        assert_eq!(of("costReq"), Some(AMBER));
        assert_eq!(of("spareReq"), Some(GREY));
        assert_eq!(
            fill_of(&m, "Req::MassLimit", &rules),
            None,
            "a definition has no status"
        );
        assert_eq!(fill_of(&m, "Req::vehicle", &rules), None);
    }

    #[test]
    fn satisfaction_beats_metadata_whatever_the_order() {
        let m = Model::builder()
            .package("P")
            .requirement("P::r")
            .annotated("P::Safety")
            .part("P::a")
            .satisfy("P::@1", "P::r", "P::a")
            .result(violated("a.x <= 1"))
            .build();
        let rules = [
            StyleRule {
                condition: Condition::Metadata(qn("P::Safety")),
                fill: NAVY,
                label: None,
            },
            on(Satisfaction::Violated, RED),
        ];

        assert_eq!(fill_of(&m, "P::r", &rules), Some(RED));
    }

    #[test]
    fn a_satisfaction_rule_is_written_by_status_name_in_the_style_file() {
        let style = Style::new(vec![on(Satisfaction::NotClaimed, GREY)]);

        let text = style.to_text();

        assert!(text.contains(r#""satisfaction": "not-claimed""#), "{text}");
        let folder = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(folder.path().join("diagrams")).unwrap();
        std::fs::write(viewer_core::style::path_for(folder.path()), text).unwrap();
        assert_eq!(
            viewer_core::style::load(folder.path()).unwrap(),
            Some(style)
        );
    }

    #[test]
    fn satisfaction_rules_are_never_marked_missing() {
        assert!(Condition::Satisfaction(Satisfaction::Violated).exists_in(&Model::default()));
    }

    fn styled_tree(rules: Vec<StyleRule>) -> Scene {
        let model = common::requirements();
        let mut scene = build_scene(&model, DiagramKind::General, &qn("Req::vehicleSpec")).unwrap();
        let diagram = DiagramStyle {
            legend: true,
            ..DiagramStyle::default()
        };
        apply_styles(&mut scene, &model, Some(&Style::new(rules)), &diagram);
        scene
    }

    #[test]
    fn the_legend_names_satisfaction_rules_by_status() {
        let scene = styled_tree(traffic_lights());

        let legend: Vec<String> = scene
            .items_depicting(&Depicts::Legend)
            .filter_map(|s| match s {
                Shape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(
            legend,
            ["Satisfied", "Violated", "Undecided", "Not claimed"]
        );
    }

    #[test]
    fn the_badge_keeps_its_mark_readable_on_a_dark_fill() {
        let scene = styled_tree(vec![on(Satisfaction::Violated, NAVY)]);
        let node = Depicts::Element(qn("Req::vehicleSpec::rangeReq"));

        let mark = scene.items_depicting(&node).find_map(|s| match s {
            Shape::Text { text, fill, .. } if text == "✗" => Some(*fill),
            _ => None,
        });
        let body = scene.items_depicting(&node).find_map(|s| match s {
            Shape::Rect { fill, .. } => Some(*fill),
            _ => None,
        });

        assert_eq!(body, Some(Fill::Color(NAVY)));
        assert_eq!(mark, Some(Fill::Background), "white on navy");
    }

    #[test]
    fn a_traffic_light_tree_matches_the_golden_file() {
        let svg = viewer_core::svg::render(&styled_tree(traffic_lights()));
        let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden/requirements-status-styled.svg");
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
}
