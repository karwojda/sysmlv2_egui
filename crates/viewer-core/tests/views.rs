//! Reading `view` usages as Model-defined Diagrams, from Models served by
//! the fake backend.

mod common;

use viewer_core::model::{ExposeScope, ModelBuilder};
use viewer_core::view::{ViewContents, ViewError, kind, resolve, views};
use viewer_core::{DiagramKind, Model, QualifiedName};

fn qn(s: &str) -> QualifiedName {
    QualifiedName::new(s)
}

fn names(list: &[&str]) -> Vec<QualifiedName> {
    list.iter().map(|s| qn(s)).collect()
}

fn unsupported_reason(model: &Model, view: &str) -> String {
    match kind(model, &qn(view)) {
        Err(ViewError::Unsupported { reason, .. }) => reason,
        other => panic!("expected `{view}` to be unsupported, got {other:?}"),
    }
}

fn invalid_reason(model: &Model, view: &str) -> String {
    match resolve(model, &qn(view)) {
        Err(ViewError::Invalid { reason, .. }) => reason,
        other => panic!("expected `{view}` to be invalid, got {other:?}"),
    }
}

/// A package with a small parts tree, definitions in a nested package, and
/// room for views added by each test.
fn base() -> ModelBuilder {
    Model::builder()
        .package("P")
        .package("P::Defs")
        .part_def("P::Defs::Frame")
        .attribute_def("P::Defs::Mass")
        .unsupported("P::Defs::Fuel", "item def")
        .part_def("P::Engine")
        .part_def("P::Vehicle")
        .part("P::Vehicle::engine")
        .typed_by("P::Engine")
        .connection("P::Vehicle::link")
        .part("P::vehicle")
        .typed_by("P::Vehicle")
        .part("P::vehicle::body")
        .part("P::vehicle::body::door")
        .attribute("P::vehicle::mass")
}

#[test]
fn every_view_usage_is_listed_but_not_view_definitions() {
    let model = common::views();

    let listed: Vec<&str> = views(&model).map(|v| v.qualified_name.as_str()).collect();

    assert_eq!(
        listed,
        [
            "V::inherited",
            "V::overview",
            "V::table",
            "V::vehicle tree",
            "V::wiring"
        ]
    );
}

#[test]
fn a_view_is_drawn_as_its_own_rendering_says() {
    let model = common::views();

    assert_eq!(
        kind(&model, &qn("V::vehicle tree")),
        Ok(DiagramKind::General)
    );
    assert_eq!(
        kind(&model, &qn("V::wiring")),
        Ok(DiagramKind::Interconnection)
    );
}

#[test]
fn a_view_without_a_rendering_uses_its_view_definitions() {
    let model = common::views();

    assert_eq!(
        kind(&model, &qn("V::inherited")),
        Ok(DiagramKind::General),
        "TreeView renders asTreeDiagram"
    );
}

#[test]
fn a_rendering_is_found_through_specialized_view_definitions() {
    let model = base()
        .view_def("P::Wiring")
        .renders("Views::asInterconnectionDiagram")
        .view_def("P::EngineWiring")
        .specializes("P::Wiring")
        .view("P::v")
        .typed_by("P::EngineWiring")
        .build();

    assert_eq!(kind(&model, &qn("P::v")), Ok(DiagramKind::Interconnection));
}

#[test]
fn the_view_s_own_rendering_wins_over_its_definition_s() {
    let model = base()
        .view_def("P::Tree")
        .renders("Views::asTreeDiagram")
        .view("P::v")
        .typed_by("P::Tree")
        .renders("Views::asInterconnectionDiagram")
        .build();

    assert_eq!(kind(&model, &qn("P::v")), Ok(DiagramKind::Interconnection));
}

#[test]
fn the_standard_view_definitions_decide_when_nothing_renders() {
    let model = base()
        .view("P::general")
        .typed_by("StandardViewDefinitions::GeneralView")
        .view_def("P::MyWiring")
        .specializes("StandardViewDefinitions::InterconnectionView")
        .view("P::wiring")
        .typed_by("P::MyWiring")
        .build();

    assert_eq!(kind(&model, &qn("P::general")), Ok(DiagramKind::General));
    assert_eq!(
        kind(&model, &qn("P::wiring")),
        Ok(DiagramKind::Interconnection)
    );
}

#[test]
fn other_standard_views_and_renderings_are_unsupported_with_a_reason() {
    let model = base()
        .view("P::flow")
        .typed_by("StandardViewDefinitions::ActionFlowView")
        .view("P::states")
        .typed_by("StandardViewDefinitions::StateTransitionView")
        .view("P::plain")
        .build();

    assert_eq!(
        unsupported_reason(&model, "P::flow"),
        "it is a `ActionFlowView`"
    );
    assert_eq!(
        unsupported_reason(&model, "P::states"),
        "it is a `StateTransitionView`"
    );
    assert_eq!(
        unsupported_reason(&model, "P::plain"),
        "it has no rendering"
    );
    assert_eq!(
        unsupported_reason(&common::views(), "V::table"),
        "it renders as `asElementTable`"
    );
}

#[test]
fn only_view_usages_resolve() {
    let model = common::views();

    assert_eq!(
        kind(&model, &qn("V::TreeView")),
        Err(ViewError::NotAView(qn("V::TreeView")))
    );
    assert_eq!(
        resolve(&model, &qn("V::Gone")),
        Err(ViewError::NotAView(qn("V::Gone")))
    );
}

fn general(model: &Model, view: &str) -> Vec<QualifiedName> {
    match resolve(model, &qn(view)) {
        Ok(ViewContents::General(nodes)) => nodes,
        other => panic!("expected a General view, got {other:?}"),
    }
}

fn tree_view(builder: ModelBuilder) -> ModelBuilder {
    builder.view("P::v").renders("Views::asTreeDiagram")
}

#[test]
fn exposing_an_element_shows_just_that_element() {
    let model = tree_view(base())
        .exposes("P::Engine", ExposeScope::Element)
        .build();

    assert_eq!(general(&model, "P::v"), names(&["P::Engine"]));
}

#[test]
fn exposing_members_shows_the_drawable_members() {
    let model = tree_view(base())
        .exposes("P::Defs", ExposeScope::Members)
        .build();

    assert_eq!(
        general(&model, "P::v"),
        names(&["P::Defs::Frame", "P::Defs::Mass"]),
        "the item def is unsupported"
    );
}

#[test]
fn exposing_recursively_shows_the_element_and_everything_beneath_it() {
    let model = tree_view(base())
        .exposes("P::vehicle", ExposeScope::ElementAndDescendants)
        .build();

    assert_eq!(
        general(&model, "P::v"),
        names(&[
            "P::vehicle",
            "P::vehicle::body",
            "P::vehicle::body::door",
            "P::vehicle::mass"
        ])
    );
}

#[test]
fn exposing_descendants_leaves_out_the_element_itself() {
    let model = tree_view(base())
        .exposes("P::vehicle", ExposeScope::Descendants)
        .build();

    assert_eq!(
        general(&model, "P::v"),
        names(&[
            "P::vehicle::body",
            "P::vehicle::body::door",
            "P::vehicle::mass"
        ])
    );
}

#[test]
fn exposing_a_package_shows_its_members() {
    let model = tree_view(base())
        .exposes("P::Defs", ExposeScope::Element)
        .build();

    assert_eq!(
        general(&model, "P::v"),
        names(&["P::Defs::Frame", "P::Defs::Mass"])
    );
}

#[test]
fn connections_and_unsupported_elements_are_not_nodes() {
    let model = tree_view(base())
        .exposes("P::Vehicle", ExposeScope::ElementAndDescendants)
        .build();

    assert_eq!(
        general(&model, "P::v"),
        names(&["P::Vehicle", "P::Vehicle::engine"]),
        "not the connection"
    );
}

#[test]
fn several_exposes_combine_without_duplicates() {
    let model = tree_view(base())
        .exposes("P::Defs", ExposeScope::Members)
        .exposes("P::Engine", ExposeScope::Element)
        .exposes("P::Defs::Frame", ExposeScope::Element)
        .build();

    assert_eq!(
        general(&model, "P::v"),
        names(&["P::Defs::Frame", "P::Defs::Mass", "P::Engine"])
    );
}

#[test]
fn a_view_exposing_nothing_is_an_empty_general_diagram() {
    let model = tree_view(base())
        .exposes("P::Gone", ExposeScope::Members)
        .build();

    assert_eq!(general(&model, "P::v"), Vec::<QualifiedName>::new());
}

#[test]
fn the_fixture_views_resolve_to_their_contents() {
    let model = common::views();

    assert_eq!(
        general(&model, "V::vehicle tree"),
        names(&["V::vehicle", "V::vehicle::body"])
    );
    assert_eq!(
        general(&model, "V::overview"),
        names(&["V::Defs::Frame", "V::Defs::Mass", "V::Engine"])
    );
    assert_eq!(
        resolve(&model, &qn("V::wiring")),
        Ok(ViewContents::Interconnection(qn("V::Vehicle")))
    );
}

fn wiring(builder: ModelBuilder) -> ModelBuilder {
    builder
        .view("P::w")
        .renders("Views::asInterconnectionDiagram")
}

#[test]
fn an_interconnection_view_is_rooted_at_its_exposed_part_whatever_the_scope() {
    let model = wiring(base())
        .exposes("P::vehicle", ExposeScope::ElementAndDescendants)
        .build();

    assert_eq!(
        resolve(&model, &qn("P::w")),
        Ok(ViewContents::Interconnection(qn("P::vehicle")))
    );
}

#[test]
fn an_interconnection_view_needs_exactly_one_part() {
    let none = wiring(base()).build();
    assert_eq!(
        invalid_reason(&none, "P::w"),
        "it exposes nothing to draw as an Interconnection Diagram"
    );

    let two = wiring(base())
        .exposes("P::Vehicle", ExposeScope::Element)
        .exposes("P::Engine", ExposeScope::Members)
        .build();
    assert_eq!(
        invalid_reason(&two, "P::w"),
        "an Interconnection Diagram needs exactly one exposed part, but it exposes `P::Engine`, `P::Vehicle`"
    );

    let package = wiring(base())
        .exposes("P::Defs", ExposeScope::Members)
        .build();
    assert_eq!(
        invalid_reason(&package, "P::w"),
        "`P::Defs` is not a part or part def"
    );
}
