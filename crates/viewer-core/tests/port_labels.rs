//! Port labels on one side of a box in an Interconnection Diagram never
//! overlap each other; a box too small for them grows.

use viewer_core::diagram::build_scene;
use viewer_core::model::ModelBuilder;
use viewer_core::scene::{Depicts, Rect, Scene, Shape};
use viewer_core::{DiagramKind, Model, QualifiedName};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

/// The bike radar's rear unit: five undirected, unconnected ports.
fn rear_unit(builder: ModelBuilder, owner: &str) -> ModelBuilder {
    let mut builder = builder;
    for (port, def) in [
        ("mount", "SeatpostMount"),
        ("rearLight", "LightPort"),
        ("ble", "BlePort"),
        ("radar", "RadarPort"),
        ("usbC", "UsbCPort"),
    ] {
        builder = builder
            .port(&format!("{owner}::{port}"))
            .typed_by(&format!("P::{def}"));
    }
    builder
}

fn defs() -> ModelBuilder {
    let mut builder = Model::builder().package("P");
    for def in [
        "SeatpostMount",
        "LightPort",
        "BlePort",
        "RadarPort",
        "UsbCPort",
    ] {
        builder = builder.port_def(&format!("P::{def}"));
    }
    builder
}

/// Each port label drawn on `part`'s box, with its extent.
fn labels(scene: &Scene, part: &str) -> Vec<(String, Rect)> {
    scene
        .items
        .iter()
        .filter(|i| matches!(&i.depicts, Depicts::Port { part: p, .. } if *p == qn(part)))
        .filter_map(|i| match &i.shape {
            Shape::Text { text, .. } => Some((text.clone(), i.shape.extent())),
            _ => None,
        })
        .collect()
}

fn assert_apart(labels: &[(String, Rect)]) {
    for (i, (a, ra)) in labels.iter().enumerate() {
        for (b, rb) in &labels[i + 1..] {
            assert!(!ra.intersects(rb), "{a} {ra:?} overlaps {b} {rb:?}");
        }
    }
}

#[test]
fn five_ports_on_a_closed_part_keep_their_labels_apart() {
    let model = rear_unit(
        defs()
            .part_def("P::RearUnit")
            .part_def("P::Bike")
            .part("P::Bike::rearUnit")
            .typed_by("P::RearUnit"),
        "P::RearUnit",
    )
    .build();
    let scene = build_scene(&model, DiagramKind::Interconnection, &qn("P::Bike")).unwrap();

    let labels = labels(&scene, "P::Bike::rearUnit");
    assert_eq!(labels.len(), 5);
    assert_apart(&labels);
}

#[test]
fn a_box_too_small_for_its_port_labels_grows_until_they_fit() {
    let model = rear_unit(defs().part_def("P::RearUnit"), "P::RearUnit").build();
    let scene = build_scene(&model, DiagramKind::Interconnection, &qn("P::RearUnit")).unwrap();

    let frame = scene.element_bounds(&qn("P::RearUnit")).unwrap();
    let labels = labels(&scene, "P::RearUnit");
    assert_eq!(labels.len(), 5);
    assert_apart(&labels);
    for (label, rect) in &labels {
        // A label starts beside its port, which lies on the frame.
        assert!(
            rect.x >= frame.x && rect.x <= frame.right(),
            "{label} {rect:?} along {frame:?}"
        );
    }
}

#[test]
fn ports_with_short_labels_keep_their_even_spacing() {
    let model = Model::builder()
        .package("P")
        .port_def("P::D")
        // A long title makes the box roomy for two short labels.
        .part_def("P::ControllerHousingAssembly")
        .port("P::ControllerHousingAssembly::a")
        .typed_by("P::D")
        .port("P::ControllerHousingAssembly::b")
        .typed_by("P::D")
        .part_def("P::Rack")
        .part("P::Rack::box")
        .typed_by("P::ControllerHousingAssembly")
        .build();
    let scene = build_scene(&model, DiagramKind::Interconnection, &qn("P::Rack")).unwrap();

    let rect = scene.element_bounds(&qn("P::Rack::box")).unwrap();
    let mut xs: Vec<f32> = scene
        .items
        .iter()
        .filter(|i| matches!(&i.depicts, Depicts::Port { part, .. } if *part == qn("P::Rack::box")))
        .filter_map(|i| match &i.shape {
            Shape::Rect { rect, .. } => Some(rect.x + rect.width / 2.0),
            _ => None,
        })
        .collect();
    xs.sort_by(f32::total_cmp);
    assert_eq!(xs.len(), 2);
    let third = rect.width / 3.0;
    assert!((xs[0] - (rect.x + third)).abs() < 1.0, "{xs:?} on {rect:?}");
    assert!(
        (xs[1] - (rect.x + 2.0 * third)).abs() < 1.0,
        "{xs:?} on {rect:?}"
    );
}
