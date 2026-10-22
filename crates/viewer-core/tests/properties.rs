//! The properties panel's content, from a Model served by the fake backend.

mod common;

use std::path::Path;

use viewer_core::engineering_tools::{DesignValue, Parts};
use viewer_core::model::Direction;
use viewer_core::properties::{Property, properties};
use viewer_core::{FakeBackend, Model, ModelBackend, QualifiedName};

fn model() -> Model {
    let model = Model::builder()
        .package("P")
        .part_def("P::Engine")
        .doc("Turns fuel into motion.")
        .at("/models/p.sysml", 3)
        .port("P::Engine::fuelIn")
        .typed_by("P::FuelPort")
        .conjugated()
        .direction(Direction::In)
        .port_def("P::FuelPort")
        .part_def("P::Vehicle")
        .specializes("P::Base")
        .part("P::Vehicle::engine")
        .typed_by("P::Engine")
        .multiplicity("1", "1")
        .at("/models/p.sysml", 9)
        .connection("P::link")
        .end(&["P::Vehicle::engine", "P::Engine::fuelIn"])
        .end(&["P::other"])
        .build();
    FakeBackend::new(model).load(Path::new("models")).unwrap()
}

fn qn(name: &str) -> QualifiedName {
    QualifiedName::from(name)
}

fn rows(element: &str) -> Vec<(&'static str, String)> {
    properties(&model(), &qn(element))
        .unwrap()
        .rows
        .into_iter()
        .map(|p| (p.label, p.value))
        .collect()
}

#[test]
fn a_usage_shows_kind_names_type_multiplicity_and_source() {
    assert_eq!(
        rows("P::Vehicle::engine"),
        [
            ("Kind", "«part»".to_owned()),
            ("Name", "engine".to_owned()),
            ("Qualified name", "P::Vehicle::engine".to_owned()),
            ("Owner", "P::Vehicle".to_owned()),
            ("Type", "P::Engine".to_owned()),
            ("Multiplicity", "[1]".to_owned()),
            ("Source", "/models/p.sysml:9".to_owned()),
        ]
    );
}

#[test]
fn a_definition_shows_its_documentation() {
    assert!(rows("P::Engine").contains(&("Documentation", "Turns fuel into motion.".to_owned())));
}

#[test]
fn a_port_shows_its_direction_and_conjugated_type() {
    let rows = rows("P::Engine::fuelIn");
    assert!(rows.contains(&("Type", "~P::FuelPort".to_owned())));
    assert!(rows.contains(&("Direction", "in".to_owned())));
}

#[test]
fn a_connection_shows_its_ends() {
    let rows = rows("P::link");
    assert!(rows.contains(&("Connects", "engine.fuelIn".to_owned())));
    assert!(rows.contains(&("To", "other".to_owned())));
}

#[test]
fn types_owners_and_generals_refer_to_their_elements_when_the_model_has_them() {
    let engine = properties(&model(), &qn("P::Vehicle::engine")).unwrap();
    let refers = |label: &str| {
        engine
            .rows
            .iter()
            .find(|p| p.label == label)
            .and_then(|p| p.refers_to.clone())
    };
    assert_eq!(refers("Type"), Some(qn("P::Engine")));
    assert_eq!(refers("Owner"), Some(qn("P::Vehicle")));
    assert_eq!(refers("Name"), None);

    let vehicle = properties(&model(), &qn("P::Vehicle")).unwrap();
    let general: &Property = vehicle
        .rows
        .iter()
        .find(|p| p.label == "Specializes")
        .unwrap();
    assert_eq!(
        general.refers_to, None,
        "P::Base isn't in the Model, so there's nothing to follow"
    );
}

#[test]
fn the_source_location_is_given_for_opening_in_an_editor() {
    let source = properties(&model(), &qn("P::Engine"))
        .unwrap()
        .source
        .unwrap();

    assert_eq!(
        (source.file.as_path(), source.line),
        (Path::new("/models/p.sysml"), 3)
    );
}

#[test]
fn an_unknown_element_has_no_properties() {
    assert!(properties(&model(), &qn("P::Gone")).is_none());
}

#[test]
fn a_view_shows_its_rendering_and_exposes_in_order() {
    let view = properties(&common::views(), &qn("V::overview")).unwrap();
    let rows: Vec<(&str, String, Option<QualifiedName>)> = view
        .rows
        .into_iter()
        .map(|p| (p.label, p.value, p.refers_to))
        .collect();

    assert!(rows.contains(&("Kind", "«view»".to_owned(), None)));
    let exposes: Vec<_> = rows.iter().filter(|r| r.0 == "Exposes").collect();
    assert_eq!(
        exposes,
        [
            &("Exposes", "V::Defs::*".to_owned(), Some(qn("V::Defs"))),
            &("Exposes", "V::Engine".to_owned(), Some(qn("V::Engine"))),
        ]
    );

    let tree = properties(&common::views(), &qn("V::vehicle tree")).unwrap();
    let values: Vec<(&str, String)> = tree.rows.into_iter().map(|p| (p.label, p.value)).collect();
    assert!(values.contains(&("Renders", "asTreeDiagram".to_owned())));
    assert!(values.contains(&("Exposes", "V::vehicle::**".to_owned())));
}

#[test]
fn a_view_says_how_it_is_drawn_or_why_it_cant_be() {
    let drawn_as = |view: &str| {
        let props = properties(&common::views(), &qn(view)).unwrap();
        props
            .rows
            .into_iter()
            .find(|p| p.label == "Drawn as")
            .map(|p| p.value)
    };

    assert_eq!(
        drawn_as("V::vehicle tree").as_deref(),
        Some("General Diagram")
    );
    assert_eq!(
        drawn_as("V::wiring").as_deref(),
        Some("Interconnection Diagram")
    );
    assert_eq!(
        drawn_as("V::table").as_deref(),
        Some("unsupported: it renders as `asElementTable`")
    );
    assert_eq!(
        drawn_as("V::TreeView"),
        None,
        "a view definition isn't drawn"
    );
}

#[test]
fn metadata_annotations_are_listed_and_lead_to_their_defs() {
    let airbag = properties(&common::metadata(), &qn("M::Vehicle::airbag")).unwrap();
    let rows: Vec<(&str, String, Option<QualifiedName>)> = airbag
        .rows
        .into_iter()
        .filter(|p| p.label == "Metadata")
        .map(|p| (p.label, p.value, p.refers_to))
        .collect();

    assert_eq!(
        rows,
        [
            ("Metadata", "@Safety".to_owned(), Some(qn("M::Safety"))),
            ("Metadata", "@Supplier".to_owned(), Some(qn("M::Supplier"))),
        ]
    );
}

/// A drive whose engine is designed in FreeCAD, as a load with the tool
/// would give it.
fn linked_drive() -> Model {
    let model = Model::builder()
        .package("Drive")
        .part("Drive::drive")
        .part("Drive::drive::engine")
        .tool_link("FreeCAD", "cad/engine.FCStd", "Engine")
        .attribute("Drive::drive::engine::volume")
        .value("900000.0 [millimetre ** 3]")
        .design_value(
            "Volume",
            DesignValue::Read {
                value: "1000000.0 [mm^3]".to_owned(),
                parts: None,
            },
        )
        .part("Drive::drive::gearbox")
        .tool_link("FreeCAD", "cad/engine.FCStd", "Assembly.Link.")
        .attribute("Drive::drive::gearbox::volume")
        .value("default 200000.0 [millimetre ** 3]")
        .design_value(
            "Volume",
            DesignValue::Missing {
                reason: "FreeCAD not available".to_owned(),
            },
        )
        .attribute("Drive::drive::gearbox::area")
        .design_value(
            "Area",
            DesignValue::Missing {
                reason: "FreeCAD has no property Area (it reads Volume)".to_owned(),
            },
        )
        // A sibling of the same kind, with no link.
        .part("Drive::drive::spare")
        .attribute("Drive::drive::spare::volume")
        .value("900000.0 [millimetre ** 3]")
        .build();
    FakeBackend::new(model).load(Path::new("drive")).unwrap()
}

fn rows_of(model: &Model, element: &str) -> Vec<(&'static str, String)> {
    properties(model, &qn(element))
        .unwrap()
        .rows
        .into_iter()
        .map(|p| (p.label, p.value))
        .collect()
}

#[test]
fn a_linked_element_shows_its_tool_link() {
    assert!(rows_of(&linked_drive(), "Drive::drive::engine").contains(&(
        "Tool Link",
        "⚙ FreeCAD · cad/engine.FCStd · Engine".to_owned()
    )));
}

#[test]
fn a_linked_feature_shows_its_design_value_then_its_model_value_in_place_of_value() {
    assert_eq!(
        rows_of(&linked_drive(), "Drive::drive::engine::volume"),
        [
            ("Kind", "«attribute»".to_owned()),
            ("Name", "volume".to_owned()),
            ("Qualified name", "Drive::drive::engine::volume".to_owned()),
            ("Owner", "Drive::drive::engine".to_owned()),
            (
                "Design Value",
                "1000000.0 [mm³]  ⚙ FreeCAD Engine.Volume".to_owned()
            ),
            ("Model Value", "900000.0 [mm³]".to_owned()),
        ]
    );
}

#[test]
fn a_feature_without_a_design_value_says_why_beside_the_model_value_it_inherits() {
    let model = linked_drive();
    let rows = rows_of(&model, "Drive::drive::gearbox::volume");
    assert_eq!(
        rows[4..],
        [
            ("Design Value", "⚠ none: FreeCAD not available".to_owned()),
            ("Model Value", "default 200000.0 [mm³]".to_owned()),
        ]
    );
    assert_eq!(
        rows_of(&model, "Drive::drive::gearbox::area")[4..],
        [
            (
                "Design Value",
                "⚠ none: FreeCAD has no property Area (it reads Volume)".to_owned()
            ),
            ("Model Value", "none".to_owned()),
        ],
        "a feature that writes no value has no Model Value"
    );
}

#[test]
fn unlinked_elements_show_their_plain_value_and_no_tool_link() {
    let model = linked_drive();
    assert_eq!(
        rows_of(&model, "Drive::drive::spare::volume")[4..],
        [("Value", "900000.0 [millimetre ** 3]".to_owned())]
    );
    assert!(
        !rows_of(&model, "Drive::drive::spare")
            .iter()
            .any(|(label, _)| *label == "Tool Link")
    );
}

#[test]
fn a_summed_design_value_says_how_many_parts_it_counted_and_lists_every_uncounted_one() {
    let model = Model::builder()
        .package("Board")
        .part("Board::board")
        .tool_link("KiCad", "ecad/board.kicad_pro", "board")
        .attribute("Board::board::power")
        .value("0.4 [W]")
        .design_value(
            "Power",
            DesignValue::Read {
                value: "0.612 [W]".to_owned(),
                parts: Some(Parts::new(["U1", "D1"], ["R1", "C10", "C2"])),
            },
        )
        .build();
    let model = FakeBackend::new(model).load(Path::new("board")).unwrap();

    assert_eq!(
        rows_of(&model, "Board::board::power")[4..],
        [
            (
                "Design Value",
                "0.612 [W]  ⚙ KiCad board.Power  summed 2 of 5 parts".to_owned()
            ),
            ("Uncounted", "C2".to_owned()),
            ("Uncounted", "C10".to_owned()),
            ("Uncounted", "R1".to_owned()),
            ("Model Value", "0.4 [W]".to_owned()),
        ]
    );
}
