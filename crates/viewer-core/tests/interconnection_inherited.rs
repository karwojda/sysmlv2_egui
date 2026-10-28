//! Inherited Members in Interconnection Diagrams: a definition's box draws
//! the parts, ports and wires it has through the definitions it
//! specialises, as it draws its own.

use viewer_core::diagram::build_scene;
use viewer_core::model::ModelBuilder;
use viewer_core::scene::{Depicts, Scene};
use viewer_core::{DiagramKind, Model, QualifiedName};

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

/// The issue's snippet: `Design :> Box` binds `Box`'s port to a part of its
/// own, and `Box` connects two of its own members.
fn snippet() -> ModelBuilder {
    Model::builder()
        .package("P")
        .port_def("P::PD")
        .part_def("P::Inner")
        .port("P::Inner::p")
        .typed_by("P::PD")
        .part_def("P::Box")
        .port("P::Box::outer")
        .typed_by("P::PD")
        .port("P::Box::side")
        .typed_by("P::PD")
        .connection("P::Box::link")
        .end(&["P::Box::outer"])
        .end(&["P::Box::side"])
        .part_def("P::Design")
        .specializes("P::Box")
        .part("P::Design::inner")
        .typed_by("P::Inner")
        .binding("P::Design::tie")
        .end(&["P::Box::outer"])
        .end(&["P::Design::inner", "P::Inner::p"])
        .part("P::specialised")
        .typed_by("P::Design")
}

fn interconnection(model: &Model, root: &str) -> Scene {
    build_scene(model, DiagramKind::Interconnection, &qn(root)).unwrap()
}

/// The ports drawn on the box of `part`, by name.
fn ports_on(scene: &Scene, part: &str) -> Vec<String> {
    let mut ports: Vec<String> = scene
        .items
        .iter()
        .filter_map(|i| match &i.depicts {
            Depicts::Port { part: p, port, .. } if *p == qn(part) => {
                Some(port.simple_name().to_owned())
            }
            _ => None,
        })
        .collect();
    ports.sort();
    ports.dedup();
    ports
}

fn draws(scene: &Scene, depicts: Depicts) -> bool {
    scene.items_depicting(&depicts).next().is_some()
}

#[test]
fn a_usage_s_frame_draws_what_its_definition_inherits() {
    let model = snippet().build();
    let scene = interconnection(&model, "P::specialised");

    assert_eq!(ports_on(&scene, "P::specialised"), ["outer", "side"]);
    assert!(scene.element_bounds(&qn("P::Design::inner")).is_some());
    assert!(draws(&scene, Depicts::Connection(qn("P::Design::tie"))));
}

#[test]
fn a_connection_the_specialised_definition_declares_is_drawn() {
    let model = snippet().build();
    let scene = interconnection(&model, "P::Design");

    assert!(draws(&scene, Depicts::Connection(qn("P::Box::link"))));
}

#[test]
fn inheritance_is_followed_through_every_definition() {
    let model = snippet()
        .part_def("P::Design2")
        .specializes("P::Design")
        .build();
    let scene = interconnection(&model, "P::Design2");

    assert_eq!(ports_on(&scene, "P::Design2"), ["outer", "side"]);
    assert!(scene.element_bounds(&qn("P::Design::inner")).is_some());
    assert!(draws(&scene, Depicts::Connection(qn("P::Design::tie"))));
    assert!(draws(&scene, Depicts::Connection(qn("P::Box::link"))));
}

#[test]
fn a_redefined_port_is_drawn_once() {
    let model = snippet().port("P::Design::outer").typed_by("P::PD").build();
    let scene = interconnection(&model, "P::Design");

    assert_eq!(ports_on(&scene, "P::Design"), ["outer", "side"]);
    // Wires to the port it redefines reach it.
    assert!(draws(&scene, Depicts::Connection(qn("P::Design::tie"))));
    assert!(draws(&scene, Depicts::Connection(qn("P::Box::link"))));
}

#[test]
fn a_closed_part_shows_the_ports_its_definition_inherits() {
    let model = snippet()
        .part_def("P::Rack")
        .part("P::Rack::unit")
        .typed_by("P::Design")
        .build();
    let scene = interconnection(&model, "P::Rack");

    assert_eq!(ports_on(&scene, "P::Rack::unit"), ["outer", "side"]);
}

#[test]
fn a_part_s_own_port_hides_the_one_of_its_definition_it_redefines() {
    let model = snippet()
        .part_def("P::Rack")
        .part("P::Rack::unit")
        .typed_by("P::Design")
        .port("P::Rack::unit::outer")
        .typed_by("P::PD")
        .build();
    let scene = interconnection(&model, "P::Rack");

    let outers = scene
        .items
        .iter()
        .filter(|i| matches!(&i.depicts, Depicts::Port { part, port, .. } if *part == qn("P::Rack::unit") && port.simple_name() == "outer"))
        .filter(|i| matches!(i.shape, viewer_core::scene::Shape::Rect { .. }))
        .count();
    assert_eq!(outers, 1);
}

#[test]
fn an_end_reaches_only_a_feature_that_redefines_it() {
    // `Other::outer` shares the name but is no supertype's member.
    let model = snippet()
        .part_def("P::Other")
        .port("P::Other::outer")
        .typed_by("P::PD")
        .connection("P::Design::stray")
        .end(&["P::Other::outer"])
        .end(&["P::Box::side"])
        .build();
    let scene = interconnection(&model, "P::Design");

    assert!(!draws(&scene, Depicts::Connection(qn("P::Design::stray"))));
}

#[test]
fn a_nested_port_is_reached_through_the_one_redefining_it() {
    let model = snippet()
        .port_def("P::Bus")
        .port("P::Bus::line")
        .typed_by("P::PD")
        .port_def("P::FastBus")
        .specializes("P::Bus")
        .port("P::FastBus::line")
        .typed_by("P::PD")
        .port("P::Box::bus")
        .typed_by("P::FastBus")
        .binding("P::Design::wire")
        .end(&["P::Box::bus", "P::Bus::line"])
        .end(&["P::Design::inner", "P::Inner::p"])
        .build();
    let scene = interconnection(&model, "P::Design");

    assert!(draws(&scene, Depicts::Connection(qn("P::Design::wire"))));
}
