//! Tool Links and Design Value outcomes on a Model built by hand, as the
//! CLI and GUI tests serve them through the fake backend.

use std::path::Path;

use viewer_core::engineering_tools::{DesignValue, DesignValueSource, ToolLink};
use viewer_core::{FakeBackend, Model, ModelBackend, QualifiedName};

fn linked_drive() -> Model {
    Model::builder()
        .package("Drive")
        .part("Drive::drive")
        .part("Drive::drive::engine")
        .tool_link("FreeCAD", "cad/engine.FCStd", "Engine")
        .attribute("Drive::drive::engine::volume")
        .value("0.9e6 [mm^3]")
        .design_value(
            "Volume",
            DesignValue::Read {
                value: "1000000.0 [mm^3]".to_owned(),
            },
        )
        .part("Drive::drive::gearbox")
        .tool_link("FreeCAD", "cad/engine.FCStd", "Assembly.Link.")
        .attribute("Drive::drive::gearbox::volume")
        .design_value(
            "Volume",
            DesignValue::Missing {
                reason: "FreeCAD not available".to_owned(),
            },
        )
        .build()
}

#[test]
fn the_fake_backend_serves_tool_links_and_design_values() {
    let model = FakeBackend::new(linked_drive())
        .load(Path::new("drive"))
        .unwrap();

    let engine = QualifiedName::new("Drive::drive::engine");
    let link = ToolLink {
        element: engine.clone(),
        tool: "FreeCAD".to_owned(),
        file: "cad/engine.FCStd".to_owned(),
        object: "Engine".to_owned(),
    };
    assert_eq!(model.tool_link(&engine), Some(&link));
    let volume = model
        .design_value(&QualifiedName::new("Drive::drive::engine::volume"))
        .unwrap();
    assert_eq!(
        volume.source,
        DesignValueSource {
            link,
            property: "Volume".to_owned(),
        }
    );
    assert_eq!(volume.model_value.as_deref(), Some("0.9e6 [mm^3]"));
    let gearbox = model
        .design_value(&QualifiedName::new("Drive::drive::gearbox::volume"))
        .unwrap();
    assert_eq!(
        gearbox.design_value,
        DesignValue::Missing {
            reason: "FreeCAD not available".to_owned()
        }
    );
    assert_eq!(gearbox.model_value, None);
    assert_eq!(model.tool_links().len(), 2);
}
