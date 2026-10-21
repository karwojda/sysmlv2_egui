//! Design Values through the real backend: `fixtures/design_values` links
//! the drive's engine and gearbox to FreeCAD, and a fake tool answers for
//! it. These need the `sysml-grpc` service, so they are opt-in:
//! `cargo test -p viewer-opensysml -- --ignored`.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use viewer_core::engineering_tools::{
    Answer, DesignValue, DesignValueOutcome, DesignValueSource, EngineeringTool, ToolLink, Wanted,
};
use viewer_core::verdict::Status;
use viewer_core::{Model, ModelBackend, QualifiedName};
use viewer_opensysml::OpenSysmlBackend;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Every call a fake tool was given: the file and what was wanted of it.
type Calls = Arc<Mutex<Vec<(PathBuf, Wanted)>>>;

/// Answers every property it is asked for from a fixed table of objects,
/// and records each call.
struct FakeTool {
    volumes: Vec<(&'static str, &'static str)>,
    calls: Calls,
}

impl EngineeringTool for FakeTool {
    fn read(&mut self, file: &Path, wanted: &Wanted) -> Result<Answer, String> {
        self.calls
            .lock()
            .unwrap()
            .push((file.to_path_buf(), wanted.clone()));
        Ok(wanted
            .iter()
            .map(|(object, properties)| {
                let volume = self
                    .volumes
                    .iter()
                    .find(|(o, _)| o == object)
                    .map(|(_, v)| v.to_string())
                    .ok_or_else(|| format!("no object {object}"));
                let answers = properties
                    .iter()
                    .map(|p| (p.clone(), volume.clone()))
                    .collect();
                (object.clone(), answers)
            })
            .collect())
    }
}

/// The design is bigger than the Model's estimates: the engine is
/// 1.0e6 mm³ (the Model says 0.9e6), the gearbox 0.3e6 mm³ (0.2e6).
fn designed() -> (FakeTool, Calls) {
    let calls = Calls::default();
    let tool = FakeTool {
        volumes: vec![
            ("Engine", "1000000.0 [mm^3]"),
            ("Assembly.Link.", "300000.0 [mm^3]"),
        ],
        calls: calls.clone(),
    };
    (tool, calls)
}

fn load_designed() -> (Model, Calls) {
    let (tool, calls) = designed();
    let model = OpenSysmlBackend::new()
        .with_tool("FreeCAD", tool)
        .load(&fixture("design_values"))
        .unwrap();
    assert!(model.diagnostics().is_empty(), "{:?}", model.diagnostics());
    (model, calls)
}

fn element<'a>(model: &'a Model, name: &str) -> &'a viewer_core::Element {
    model
        .element(&QualifiedName::new(format!("Drive::{name}")))
        .unwrap_or_else(|| panic!("no Drive::{name}"))
}

fn requirement_status(model: &Model) -> Status {
    viewer_core::verdict::of(model, element(model, "driveMass"))
        .unwrap()
        .status
}

/// What checking `engineMass` on `part` found.
fn engine_mass_on(model: &Model, part: &str) -> Status {
    let part = QualifiedName::new(format!("Drive::drive::{part}"));
    let check = element(model, "Engine::engineMass")
        .checks
        .iter()
        .find(|c| c.part == part)
        .unwrap_or_else(|| panic!("engineMass not checked on {part}"));
    Status::of(&check.result)
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn design_values_replace_model_values_in_requirements_constraints_and_roll_ups() {
    let (model, _) = load_designed();

    // Model Values: 0.9e6 mm³ × 7850 kg/m³ = 7.065 kg ≤ 7.5 kg, and with
    // the gearbox's 0.54 kg the drive is 7.605 kg ≤ 8 kg. Design Values:
    // the engine is 7.85 kg and the gearbox 0.81 kg, 8.66 kg in all.
    assert_eq!(requirement_status(&model), Status::Violated);
    assert_eq!(engine_mass_on(&model, "engine"), Status::Violated);
    assert_eq!(element(&model, "drive::mass").computed, ["8.66 [kg]"]);
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_design_value_in_another_unit_is_converted_to_the_feature_s() {
    let (model, _) = load_designed();

    // The Model writes the gearbox in m³ (0.0002 m³, 0.54 kg) and the tool
    // answers 300000.0 mm³: 0.0003 m³ × 2700 kg/m³ = 0.81 kg, so the drive
    // is 7.85 kg + 0.81 kg.
    assert_eq!(
        element(&model, "drive::gearbox::volume").computed,
        ["300000.0 [mm³]"]
    );
    assert_eq!(element(&model, "drive::mass").computed, ["8.66 [kg]"]);
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn only_the_linked_usage_takes_the_design_value() {
    let (model, _) = load_designed();

    // The spare engine is an Engine too, unlinked, at 7.065 kg.
    assert_eq!(engine_mass_on(&model, "spare"), Status::Holds);
    assert_eq!(
        element(&model, "drive::spare::volume").computed,
        ["900000.0 [mm³]"]
    );
    assert_eq!(element(&model, "Component::volume").value, None);
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn the_model_records_its_tool_links_and_each_feature_s_design_value_with_its_model_value() {
    let (model, _) = load_designed();
    let link = |element: &str, object: &str| ToolLink {
        element: QualifiedName::new(element),
        tool: "FreeCAD".to_owned(),
        file: "cad/drive.FCStd".to_owned(),
        object: object.to_owned(),
    };
    let outcome =
        |feature: &str, object: &str, value: &str, model_value: &str| DesignValueOutcome {
            feature: QualifiedName::new(feature),
            source: DesignValueSource {
                tool: "FreeCAD".to_owned(),
                file: "cad/drive.FCStd".to_owned(),
                object: object.to_owned(),
                property: "Volume".to_owned(),
            },
            design_value: DesignValue::Read {
                value: value.to_owned(),
            },
            model_value: Some(model_value.to_owned()),
        };

    assert_eq!(
        model.tool_links(),
        [
            link("Drive::drive::engine", "Engine"),
            link("Drive::drive::gearbox", "Assembly.Link."),
        ]
    );
    assert_eq!(
        model.design_values(),
        [
            outcome(
                "Drive::drive::engine::volume",
                "Engine",
                "1000000.0 [mm^3]",
                "900000.0 [millimetre ** 3]"
            ),
            outcome(
                "Drive::drive::gearbox::volume",
                "Assembly.Link.",
                "300000.0 [mm^3]",
                "0.0002 [metre ** 3]"
            ),
        ]
    );
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn the_tool_is_asked_once_per_file_relative_to_the_file_holding_the_links() {
    let (_, calls) = load_designed();

    let wanted: Wanted = [
        ("Assembly.Link.".to_owned(), ["Volume".to_owned()].into()),
        ("Engine".to_owned(), ["Volume".to_owned()].into()),
    ]
    .into();
    assert_eq!(
        *calls.lock().unwrap(),
        [(fixture("design_values/drive/cad/drive.FCStd"), wanted)]
    );
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loading_design_values_leaves_the_sysml_files_as_they_were() {
    let files = [
        fixture("design_values/drive/drive.sysml"),
        fixture("design_values/EngineeringTools.sysml"),
    ];
    let read = || -> Vec<Vec<u8>> { files.iter().map(|f| std::fs::read(f).unwrap()).collect() };
    let before = read();

    load_designed();

    assert!(read() == before, "a .sysml file changed");
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_model_without_tool_links_never_asks_the_tool() {
    let (tool, calls) = designed();

    let with_tool = OpenSysmlBackend::new()
        .with_tool("FreeCAD", tool)
        .load(&fixture("analysis"))
        .unwrap();
    let without = OpenSysmlBackend::new().load(&fixture("analysis")).unwrap();

    assert!(calls.lock().unwrap().is_empty());
    assert_eq!(with_tool, without);
    assert!(with_tool.tool_links().is_empty());
}
