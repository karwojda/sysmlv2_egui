//! Seam 1: unnamed connections, flows, bindings and reference parts from
//! the `flows` fixture's fake twin.

mod common;

use viewer_core::browser::tree;
use viewer_core::properties::properties;
use viewer_core::scene::{Depicts, Shape};
use viewer_core::{DiagramKind, QualifiedName, Scene, build_scene};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn scene(kind: DiagramKind, root: &str) -> Scene {
    build_scene(&common::flows(), kind, &qn(root)).unwrap()
}

fn connector(scene: &Scene, connection: &str) -> (usize, Vec<String>) {
    let depicts = Depicts::Connection(qn(connection));
    let shapes: Vec<&Shape> = scene.items_depicting(&depicts).collect();
    let lines = shapes
        .iter()
        .filter(|s| matches!(s, Shape::Polyline { .. }))
        .count();
    let texts = shapes
        .iter()
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    (lines, texts)
}

#[test]
fn an_unnamed_connection_is_drawn_without_a_label_in_an_interconnection_diagram() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");

    let (lines, texts) = connector(&scene, "F::Vehicle::@7");

    assert_eq!(lines, 1, "it is drawn");
    assert!(texts.is_empty(), "no name label: {texts:?}");
}

#[test]
fn an_unnamed_connection_is_an_unlabelled_edge_in_a_general_diagram() {
    let scene = scene(DiagramKind::General, "F::Vehicle");

    let (lines, texts) = connector(&scene, "F::Vehicle::@7");

    assert_eq!(lines, 1);
    assert!(texts.is_empty(), "{texts:?}");
}

#[test]
fn an_unnamed_connection_is_listed_by_its_ends_in_the_connections_compartment() {
    let scene = scene(DiagramKind::General, "F");
    let rows: Vec<String> = scene
        .items_depicting(&Depicts::Element(qn("F::Vehicle")))
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    assert!(
        rows.contains(&"(tank.fuelOut.inner → engine.fuelIn.inner)".to_owned()),
        "{rows:?}"
    );
}

#[test]
fn unnamed_elements_are_named_by_their_ends_in_the_browser() {
    let entries = tree(&common::flows(), "");
    let vehicle = entries[0]
        .children
        .iter()
        .find(|e| e.name == "Vehicle")
        .unwrap();
    let names: Vec<(&str, &str)> = vehicle
        .children
        .iter()
        .map(|e| (e.keyword.as_str(), e.name.as_str()))
        .collect();

    assert!(
        names.contains(&("connection", "tank.fuelOut.inner → engine.fuelIn.inner")),
        "{names:?}"
    );
    assert!(
        names.contains(&("bind", "tank.level = engine.fuelLevel")),
        "{names:?}"
    );
    assert!(names.contains(&("flow", "fuelFlow")), "{names:?}");
}

#[test]
fn the_properties_panel_calls_an_unnamed_element_unnamed() {
    let props = properties(&common::flows(), &qn("F::Vehicle::@6")).unwrap();

    let name = props.rows.iter().find(|p| p.label == "Name").unwrap();
    assert_eq!(name.value, "(unnamed)");
    assert!(
        props
            .rows
            .iter()
            .any(|p| p.label == "Kind" && p.value == "«bind»")
    );
}

fn arrow(scene: &Scene, connection: &str) -> Option<Vec<viewer_core::scene::Point>> {
    scene
        .items_depicting(&Depicts::Connection(qn(connection)))
        .find_map(|s| match s {
            Shape::Polygon {
                points,
                fill: viewer_core::scene::Fill::Stroke,
            } => Some(points.clone()),
            _ => None,
        })
}

fn line(scene: &Scene, connection: &str) -> Vec<viewer_core::scene::Point> {
    scene
        .items_depicting(&Depicts::Connection(qn(connection)))
        .find_map(|s| match s {
            Shape::Polyline { points, .. } => Some(points.clone()),
            _ => None,
        })
        .unwrap()
}

fn port_centre(scene: &Scene, key: &str) -> viewer_core::scene::Point {
    scene
        .items
        .iter()
        .find_map(|i| match (&i.depicts, &i.shape) {
            (Depicts::Port { key: k, .. }, Shape::Rect { rect, .. }) if k == key => {
                Some(viewer_core::scene::Point::new(
                    rect.x + rect.width / 2.0,
                    rect.y + rect.height / 2.0,
                ))
            }
            _ => None,
        })
        .unwrap_or_else(|| panic!("port {key}"))
}

#[test]
fn a_flow_has_a_filled_arrowhead_pointing_toward_its_target() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");
    let points = line(&scene, "F::Vehicle::fuelFlow");
    let triangle = arrow(&scene, "F::Vehicle::fuelFlow").expect("a flow has an arrowhead");

    // The tip is the triangle point furthest along the flow's direction.
    let (from, to) = (points[0], *points.last().unwrap());
    let along = |p: viewer_core::scene::Point| {
        (p.x - from.x) * (to.x - from.x) + (p.y - from.y) * (to.y - from.y)
    };
    let tip = triangle
        .iter()
        .copied()
        .max_by(|a, b| along(*a).total_cmp(&along(*b)))
        .unwrap();
    let base_mid = {
        let others: Vec<_> = triangle.iter().filter(|p| **p != tip).collect();
        viewer_core::scene::Point::new(
            (others[0].x + others[1].x) / 2.0,
            (others[0].y + others[1].y) / 2.0,
        )
    };
    assert!(along(tip) > along(base_mid), "points from source to target");
    assert_eq!(triangle.len(), 3);
}

#[test]
fn flows_are_labelled_with_their_item() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");

    assert_eq!(
        connector(&scene, "F::Vehicle::fuelFlow").1,
        ["fuelFlow:Fuel"]
    );
    assert_eq!(connector(&scene, "F::Vehicle::@3").1, [":Fuel"]);
}

#[test]
fn a_flow_from_an_item_inside_a_port_attaches_to_that_port() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");
    let points = line(&scene, "F::Vehicle::@3");

    let near = |a: viewer_core::scene::Point, b: viewer_core::scene::Point| {
        (a.x - b.x).abs() <= 2.5 && (a.y - b.y).abs() <= 2.5
    };
    assert!(
        near(points[0], port_centre(&scene, "tank.fuelOut")),
        "{points:?}"
    );
    assert!(
        near(
            *points.last().unwrap(),
            port_centre(&scene, "engine.fuelIn")
        ),
        "{points:?}"
    );
}

#[test]
fn the_general_diagram_lists_flows_and_draws_them_with_arrowheads() {
    let package = scene(DiagramKind::General, "F");
    let rows: Vec<String> = package
        .items_depicting(&Depicts::Element(qn("F::Vehicle")))
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert!(
        rows.contains(&"flow fuelFlow:Fuel (tank.fuelOut → engine.fuelIn)".to_owned()),
        "{rows:?}"
    );
    assert!(
        rows.contains(&"flow :Fuel (tank.fuelOut.fuel → engine.fuelIn.fuel)".to_owned()),
        "{rows:?}"
    );

    let vehicle = scene(DiagramKind::General, "F::Vehicle");
    assert!(arrow(&vehicle, "F::Vehicle::fuelFlow").is_some());
}

#[test]
fn the_flows_interconnection_diagram_is_readable_and_matches_the_golden_file() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");
    let labels: Vec<(String, viewer_core::scene::Rect)> = scene
        .items
        .iter()
        .filter_map(|i| match &i.shape {
            Shape::Text { text, .. } => Some((text.clone(), i.shape.extent())),
            _ => None,
        })
        .collect();
    for (i, (a, ra)) in labels.iter().enumerate() {
        for (b, rb) in &labels[i + 1..] {
            assert!(!ra.intersects(rb), "{a} overlaps {b}");
        }
    }
    let svg = viewer_core::svg::render(&scene);
    let golden = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden/flows-interconnection.svg");
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
fn a_binding_between_ports_is_a_connector_marked_equals() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");

    let (lines, texts) = connector(&scene, "F::Vehicle::@6");

    assert_eq!(lines, 1);
    assert_eq!(texts, ["="]);
}

#[test]
fn a_binding_between_attribute_values_is_not_drawn_but_still_listed() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");

    assert_eq!(connector(&scene, "F::Vehicle::@5").0, 0, "not drawn");
    let props = properties(&common::flows(), &qn("F::Vehicle::@5"));
    assert!(props.is_some(), "still in the Properties panel");
}

#[test]
fn the_general_diagram_lists_bindings_as_bind_a_equals_b() {
    let scene = scene(DiagramKind::General, "F");
    let rows: Vec<String> = scene
        .items_depicting(&Depicts::Element(qn("F::Vehicle")))
        .filter_map(|s| match s {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();

    assert!(
        rows.contains(&"bind tank.level = engine.fuelLevel".to_owned()),
        "{rows:?}"
    );
    assert!(
        rows.contains(&"bind tank.fuelOut = engine.fuelIn".to_owned()),
        "{rows:?}"
    );
}

fn box_dashed(scene: &Scene, element: &str) -> bool {
    scene
        .items
        .iter()
        .find_map(|i| match (&i.depicts, &i.shape) {
            (
                Depicts::Element(e) | Depicts::Part { element: e, .. },
                Shape::Rect { dashed, .. },
            ) if *e == qn(element) => Some(*dashed),
            _ => None,
        })
        .unwrap_or_else(|| panic!("{element} is drawn"))
}

#[test]
fn reference_parts_are_dashed_in_both_diagram_kinds() {
    let interconnection = scene(DiagramKind::Interconnection, "F::Vehicle");
    assert!(box_dashed(&interconnection, "F::Vehicle::driver"));
    assert!(!box_dashed(&interconnection, "F::Vehicle::tank"));

    let general = scene(DiagramKind::General, "F::Vehicle");
    assert!(box_dashed(&general, "F::Vehicle::driver"));
    assert!(!box_dashed(&general, "F::Vehicle::engine"));
}

#[test]
fn a_dashed_box_is_exported_with_a_dash_pattern() {
    let svg = viewer_core::svg::render(&scene(DiagramKind::Interconnection, "F::Vehicle"));

    assert_eq!(
        svg.matches(r#"stroke-dasharray="6 4"/>"#).count(),
        1,
        "only the reference part is dashed"
    );
}

#[test]
fn the_properties_panel_marks_reference_parts() {
    let driver = properties(&common::flows(), &qn("F::Vehicle::driver")).unwrap();
    let tank = properties(&common::flows(), &qn("F::Vehicle::tank")).unwrap();

    assert!(
        driver
            .rows
            .iter()
            .any(|p| p.label == "Reference" && p.value == "yes")
    );
    assert!(!tank.rows.iter().any(|p| p.label == "Reference"));
}

fn port_rect(scene: &Scene, key: &str) -> viewer_core::scene::Rect {
    scene
        .items
        .iter()
        .find_map(|i| match (&i.depicts, &i.shape) {
            (Depicts::Port { key: k, .. }, Shape::Rect { rect, .. }) if k == key => Some(*rect),
            _ => None,
        })
        .unwrap_or_else(|| panic!("port {key}"))
}

#[test]
fn a_port_with_ports_is_a_bar_with_its_nested_ports_on_the_outer_edge() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");
    let tank = scene.node_bounds(&qn("tank")).unwrap();
    let bar = port_rect(&scene, "tank.fuelOut");
    let inner = port_rect(&scene, "tank.fuelOut.inner");

    assert!(
        bar.width > 3.0 * bar.height,
        "a bar along the bottom edge: {bar:?}"
    );
    assert!(
        (bar.y + bar.height / 2.0 - tank.bottom()).abs() < 0.5,
        "straddling the border"
    );
    let inner_centre =
        viewer_core::scene::Point::new(inner.x + inner.width / 2.0, inner.y + inner.height / 2.0);
    assert!(
        inner_centre.x > bar.x && inner_centre.x < bar.right(),
        "on the bar"
    );
    assert!(
        (inner_centre.y - bar.bottom()).abs() < 0.5,
        "on its outer (lower) edge"
    );
}

#[test]
fn a_connection_to_a_nested_port_attaches_to_its_square_and_one_to_the_composite_to_the_bar() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");
    let near = |a: viewer_core::scene::Point, b: viewer_core::scene::Point| {
        (a.x - b.x).abs() <= 2.5 && (a.y - b.y).abs() <= 2.5
    };

    let to_inner = line(&scene, "F::Vehicle::@7");
    assert!(
        near(to_inner[0], port_centre(&scene, "tank.fuelOut.inner")),
        "{to_inner:?}"
    );
    assert!(
        near(
            *to_inner.last().unwrap(),
            port_centre(&scene, "engine.fuelIn.inner")
        ),
        "{to_inner:?}"
    );

    let to_bar = line(&scene, "F::Vehicle::fuelFlow");
    assert!(
        near(to_bar[0], port_centre(&scene, "tank.fuelOut")),
        "{to_bar:?}"
    );
}

#[test]
fn nested_ports_are_labelled_and_selectable() {
    let scene = scene(DiagramKind::Interconnection, "F::Vehicle");
    let inner = port_rect(&scene, "tank.fuelOut.inner");
    let centre =
        viewer_core::scene::Point::new(inner.x + inner.width / 2.0, inner.y + inner.height / 2.0);

    let found = scene.port_at(centre);

    assert_eq!(
        found,
        Some((
            &qn("F::Vehicle::tank"),
            &qn("F::FuelPort::inner"),
            "tank.fuelOut.inner"
        ))
    );
    let labels: Vec<String> = scene
        .items
        .iter()
        .filter(|i| matches!(&i.depicts, Depicts::Port { key, .. } if key == "tank.fuelOut.inner"))
        .filter_map(|i| match &i.shape {
            Shape::Text { text, .. } => Some(text.clone()),
            _ => None,
        })
        .collect();
    assert_eq!(labels, ["inner : InnerPort"]);
}
