//! Seam 1: constraint definitions, constraint usages and asserted
//! constraints in General Diagrams.

mod common;

use std::path::Path;

use viewer_core::scene::{Depicts, Shape};
use viewer_core::{
    DiagramKind, Layout, QualifiedName, Scene, build_general_scene, build_scene, svg,
};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

fn package() -> Scene {
    build_scene(&common::assertions(), DiagramKind::General, &qn("C")).unwrap()
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

/// The rows of the compartment titled `title` in `texts`.
fn compartment(texts: &[String], title: &str, titles: &[&str]) -> Vec<String> {
    let Some(at) = texts.iter().position(|t| t == title) else {
        return Vec::new();
    };
    texts[at + 1..]
        .iter()
        .take_while(|t| !titles.contains(&t.as_str()))
        .cloned()
        .collect()
}

const TITLES: [&str; 7] = [
    "attributes",
    "ports",
    "parts",
    "assert constraints",
    "constraints",
    "requirements",
    "connections",
];

#[test]
fn a_part_def_lists_its_asserted_constraints_after_its_attributes() {
    let tank = texts_of(&package(), "C::Tank");

    let titles: Vec<&String> = tank
        .iter()
        .filter(|t| TITLES.contains(&t.as_str()))
        .collect();
    assert_eq!(titles, ["attributes", "assert constraints"]);
    assert_eq!(
        compartment(&tank, "assert constraints", &TITLES),
        ["{fuelMass <= fuelMassMax}"]
    );
}

#[test]
fn a_part_lists_asserted_and_plain_constraints_separately() {
    let vehicle = texts_of(&package(), "C::vehicle");

    assert_eq!(
        compartment(&vehicle, "assert constraints", &TITLES),
        ["{mass <= 1500.0} ✓", "powerToWeight:MaxRatio ✗"],
        "an expression is shown in braces; a usage of a definition by name and type"
    );
    assert_eq!(
        compartment(&vehicle, "constraints", &TITLES),
        ["{mass > 2000.0}"]
    );
    let order: Vec<&String> = vehicle
        .iter()
        .filter(|t| TITLES.contains(&t.as_str()))
        .collect();
    assert_eq!(
        order,
        ["attributes", "assert constraints", "constraints", "parts"]
    );
}

#[test]
fn a_constraint_def_shows_its_parameters_and_expression() {
    let texts = texts_of(&package(), "C::MaxRatio");

    assert_eq!(&texts[..2], ["«constraint def»", "MaxRatio"]);
    assert_eq!(
        compartment(&texts, "parameters", &["expression"]),
        ["in denominator:Real", "in limit:Real", "in numerator:Real"],
        "without source lines, name order"
    );
    assert_eq!(
        compartment(&texts, "expression", &[]),
        ["numerator / denominator <= limit"]
    );
}

#[test]
fn parameters_keep_the_order_they_are_written_in() {
    let mut model = viewer_core::Model::builder()
        .package("P")
        .constraint_def("P::Ratio");
    for (name, line) in [("numerator", 3), ("denominator", 4), ("limit", 5)] {
        model = model
            .unsupported(&format!("P::Ratio::{name}"), "reference")
            .direction(viewer_core::model::Direction::In)
            .at("/m.sysml", line);
    }
    let scene = build_scene(&model.build(), DiagramKind::General, &qn("P")).unwrap();

    assert_eq!(
        compartment(&texts_of(&scene, "P::Ratio"), "parameters", &[]),
        ["in numerator", "in denominator", "in limit"]
    );
}

/// The vehicle with its asserted `powerToWeight` drawn as a node of its own,
/// as a view exposing both would.
fn usage_node() -> Scene {
    let elements = [
        qn("C::vehicle"),
        qn("C::vehicle::powerToWeight"),
        qn("C::MaxRatio"),
    ];
    build_general_scene(&common::assertions(), &elements, &Default::default())
}

#[test]
fn an_asserted_constraint_usage_node_shows_its_bound_parameters() {
    let texts = texts_of(&usage_node(), "C::vehicle::powerToWeight");

    assert_eq!(
        &texts[..2],
        ["«assert constraint»", "powerToWeight:MaxRatio"]
    );
    assert_eq!(
        compartment(&texts, "parameters", &["expression"]),
        [
            "in denominator = power = 90.0",
            "in limit = 12.0",
            "in numerator = mass = 1400.0"
        ]
    );
    assert!(
        !texts.contains(&"expression".to_owned()),
        "it uses its definition's expression"
    );
}

#[test]
fn a_constraint_drawn_as_a_node_leaves_its_owner_s_compartment() {
    let vehicle = texts_of(&usage_node(), "C::vehicle");

    assert_eq!(
        compartment(&vehicle, "assert constraints", &TITLES),
        ["{mass <= 1500.0} ✓"]
    );
}

fn assert_golden(name: &str, scene: &Scene) {
    let svg = svg::render(scene);
    let golden = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden")
        .join(name);
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
fn a_package_with_constraints_matches_the_golden_file() {
    assert_golden("constraints-package.svg", &package());
}

#[test]
fn a_constraint_usage_node_matches_the_golden_file() {
    assert_golden("constraints-usage.svg", &usage_node());
}

/// The vehicle and its tank as nodes, as a view exposing both would draw them.
fn vehicle_and_tank() -> Scene {
    vehicle_and_tank_with(&Layout::default())
}

fn vehicle_and_tank_with(layout: &Layout) -> Scene {
    let elements = [
        qn("C::vehicle"),
        qn("C::vehicle::tank"),
        qn("C::vehicle::heavy"),
        qn("C::vehicle::powerToWeight"),
    ];
    build_general_scene(&common::assertions(), &elements, layout)
}

#[test]
fn a_part_shows_constraints_it_inherits_with_its_own_result() {
    let tank = texts_of(&vehicle_and_tank(), "C::vehicle::tank");

    assert_eq!(
        compartment(&tank, "assert constraints", &TITLES),
        ["^{fuelMass <= fuelMassMax} ✗"]
    );
}

#[test]
fn a_diagram_with_inherited_members_turned_off_drops_inherited_constraints() {
    let layout = Layout {
        inherited_hidden: true,
        ..Layout::default()
    };

    let tank = texts_of(&vehicle_and_tank_with(&layout), "C::vehicle::tank");

    assert!(compartment(&tank, "assert constraints", &TITLES).is_empty());
}

#[test]
fn a_part_def_s_constraints_carry_no_mark() {
    let tank = texts_of(&package(), "C::Tank");

    assert_eq!(
        compartment(&tank, "assert constraints", &TITLES),
        ["{fuelMass <= fuelMassMax}"]
    );
}

#[test]
fn an_asserted_constraint_never_checked_on_a_part_reads_not_checked() {
    let model = viewer_core::Model::builder()
        .package("P")
        .part("P::p")
        .constraint("P::p::c")
        .asserted()
        .expression("x <= 1")
        .build();
    let scene = build_scene(&model, DiagramKind::General, &qn("P")).unwrap();

    assert_eq!(
        compartment(&texts_of(&scene, "P::p"), "assert constraints", &TITLES),
        ["{x <= 1} ○"]
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
                Shape::Text { text, .. } if ["✓", "✗", "?", "○"].contains(&text.as_str()) => {
                    Some(text.clone())
                }
                _ => None,
            })
            .unwrap()
    })
}

#[test]
fn only_asserted_constraint_nodes_wear_a_badge() {
    let scene = vehicle_and_tank();

    assert_eq!(
        badge(&scene, "C::vehicle::powerToWeight").as_deref(),
        Some("✗")
    );
    assert_eq!(
        badge(&scene, "C::vehicle::heavy"),
        None,
        "a plain constraint is no claim"
    );
    assert_eq!(badge(&scene, "C::vehicle"), None, "parts carry no badge");
}

#[test]
fn properties_explain_each_result_on_a_part_and_on_a_constraint() {
    use viewer_core::properties::properties;
    let model = common::assertions();
    let rows = |element: &str| -> Vec<(&'static str, String)> {
        properties(&model, &qn(element))
            .unwrap()
            .rows
            .into_iter()
            .map(|p| (p.label, p.value))
            .collect()
    };

    let tank = rows("C::vehicle::tank");
    assert!(tank.contains(&("Asserts", "fits ✗".to_owned())), "{tank:?}");
    assert!(
        tank.contains(&("Failing condition", "fuelMass <= fuelMassMax".to_owned())),
        "{tank:?}"
    );
    let vehicle = rows("C::vehicle");
    assert!(
        vehicle.contains(&("Asserts", "light ✓".to_owned()))
            && vehicle.contains(&("Asserts", "powerToWeight ✗".to_owned()))
    );
    assert!(rows("C::Tank::fits").contains(&(
        "Result",
        "on C::vehicle::tank ✗ violated: fuelMass <= fuelMassMax".to_owned()
    )));
}

#[test]
fn parts_with_checked_constraints_match_the_golden_file() {
    assert_golden("constraints-checked.svg", &vehicle_and_tank());
}

mod styles {
    use super::*;
    use viewer_core::DiagramStyle;
    use viewer_core::model::Model;
    use viewer_core::scene::Rgb;
    use viewer_core::style::{
        Condition, PartConstraints, Style, StyleRule, apply_styles, rule_for,
    };
    use viewer_core::verdict::{Kind, Status};

    const RED: Rgb = Rgb(0xf4, 0xcc, 0xcc);
    const GREEN: Rgb = Rgb(0xd9, 0xea, 0xd3);
    const AMBER: Rgb = Rgb(0xff, 0xe5, 0x99);
    const NAVY: Rgb = Rgb(0x1f, 0x2a, 0x44);

    fn rule(condition: Condition, fill: Rgb) -> StyleRule {
        StyleRule {
            condition,
            fill,
            label: None,
        }
    }

    fn rules() -> Vec<StyleRule> {
        vec![
            rule(Condition::Verdict(Kind::Constraint, Status::Holds), GREEN),
            rule(Condition::Verdict(Kind::Constraint, Status::Violated), RED),
            rule(
                Condition::Verdict(Kind::Constraint, Status::Undecided),
                AMBER,
            ),
            rule(Condition::Constraints(PartConstraints::Violated), RED),
        ]
    }

    fn fill_of(model: &Model, element: &str, rules: &[StyleRule]) -> Option<Rgb> {
        rule_for(model, model.element(&qn(element)).unwrap(), &[rules]).map(|r| r.fill)
    }

    #[test]
    fn a_constraint_rule_matches_asserted_constraints_by_result() {
        let m = common::assertions();

        assert_eq!(fill_of(&m, "C::vehicle::light", &rules()), Some(GREEN));
        assert_eq!(
            fill_of(&m, "C::vehicle::powerToWeight", &rules()),
            Some(RED)
        );
        assert_eq!(
            fill_of(&m, "C::vehicle::heavy", &rules()),
            None,
            "not asserted"
        );
        assert_eq!(fill_of(&m, "C::MaxRatio", &rules()), None);
    }

    #[test]
    fn a_parts_rule_matches_parts_with_a_violated_constraint_of_their_own() {
        let m = Model::builder()
            .package("P")
            .part("P::car")
            .part("P::car::tank")
            .constraint("P::car::tank::full")
            .asserted()
            .checked_on(
                "P::car::tank",
                viewer_core::model::ClaimResult::Violated {
                    condition: "x".to_owned(),
                },
            )
            .build();

        assert_eq!(fill_of(&m, "P::car::tank", &rules()), Some(RED));
        assert_eq!(
            fill_of(&m, "P::car", &rules()),
            None,
            "a nested part's violation doesn't colour its owner"
        );
        assert_eq!(
            fill_of(&common::assertions(), "C::vehicle::tank", &rules()),
            Some(RED),
            "inherited counts"
        );
    }

    #[test]
    fn constraint_rules_beat_metadata() {
        let m = Model::builder()
            .package("P")
            .part("P::p")
            .annotated("P::Safety")
            .constraint("P::p::c")
            .asserted()
            .checked_on(
                "P::p",
                viewer_core::model::ClaimResult::Violated {
                    condition: "x".to_owned(),
                },
            )
            .build();
        let rules = [
            rule(Condition::Metadata(qn("P::Safety")), NAVY),
            rule(Condition::Constraints(PartConstraints::Violated), RED),
        ];

        assert_eq!(fill_of(&m, "P::p", &rules), Some(RED));
    }

    #[test]
    fn both_rules_are_written_by_name_in_the_style_file() {
        let style = Style::new(rules());

        let text = style.to_text();

        assert!(text.contains(r#""constraint": "violated""#), "{text}");
        assert!(text.contains(r#""constraints": "violated""#), "{text}");
        let folder = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(folder.path().join("diagrams")).unwrap();
        std::fs::write(viewer_core::style::path_for(folder.path()), text).unwrap();
        assert_eq!(
            viewer_core::style::load(folder.path()).unwrap(),
            Some(style)
        );
    }

    fn styled() -> Scene {
        let model = common::assertions();
        let mut scene = vehicle_and_tank();
        let diagram = DiagramStyle {
            legend: true,
            ..DiagramStyle::default()
        };
        apply_styles(&mut scene, &model, Some(&Style::new(rules())), &diagram);
        scene
    }

    #[test]
    fn the_legend_names_constraint_rules() {
        let legend: Vec<String> = styled()
            .items_depicting(&Depicts::Legend)
            .filter_map(|s| match s {
                Shape::Text { text, .. } => Some(text.clone()),
                _ => None,
            })
            .collect();

        assert_eq!(
            legend,
            ["Constraint violated", "Part with a violated constraint"]
        );
    }

    #[test]
    fn styled_constraints_match_the_golden_file() {
        assert_golden("constraints-styled.svg", &styled());
    }
}
