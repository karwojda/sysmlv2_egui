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
use viewer_core::{Model, ModelBackend, QualifiedName, Severity};
use viewer_opensysml::OpenSysmlBackend;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Every call a fake tool was given: the file and what was wanted of it.
type Calls = Arc<Mutex<Vec<(PathBuf, Wanted)>>>;

/// Answers every property it is asked for from a fixed table of objects,
/// answers nothing for the files it can't read, and records each call.
struct FakeTool {
    volumes: Vec<(&'static str, &'static str)>,
    /// File names, and why each can't be read.
    unreadable: Vec<(&'static str, &'static str)>,
    calls: Calls,
}

impl EngineeringTool for FakeTool {
    fn read(&mut self, file: &Path, wanted: &Wanted) -> Result<Answer, String> {
        self.calls
            .lock()
            .unwrap()
            .push((file.to_path_buf(), wanted.clone()));
        if let Some((_, reason)) = self
            .unreadable
            .iter()
            .find(|(name, _)| file.file_name().is_some_and(|f| f == *name))
        {
            return Err(reason.to_string());
        }
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
/// 1.0e6 mm³ (the Model says 0.9e6), the gearbox 0.3e6 mm³ (0.2e6). The
/// shaft's bore is 8 mm, and the pump is 1.5e6 mm³.
fn designed() -> (FakeTool, Calls) {
    let calls = Calls::default();
    let tool = FakeTool {
        volumes: vec![
            ("Engine", "1000000.0 [mm^3]"),
            ("Assembly.Link.", "300000.0 [mm^3]"),
            ("Shaft", "8.0 [mm]"),
            ("Pump", "1500000.0 [mm^3]"),
        ],
        unreadable: Vec::new(),
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

/// `fixtures/design_value_shapes`: the engine's volume has a body and no
/// value, and the gearbox's only redefines Gearbox's `default`.
fn load_shapes() -> (Model, Calls) {
    let (tool, calls) = designed();
    let model = OpenSysmlBackend::new()
        .with_tool("FreeCAD", tool)
        .load(&fixture("design_value_shapes"))
        .unwrap();
    assert!(model.diagnostics().is_empty(), "{:?}", model.diagnostics());
    (model, calls)
}

/// What checking `constraint` (`Engine::engineMass`) on `part` found.
fn check_on(model: &Model, constraint: &str, part: &str) -> Status {
    let part = QualifiedName::new(format!("Drive::drive::{part}"));
    let check = element(model, constraint)
        .checks
        .iter()
        .find(|c| c.part == part)
        .unwrap_or_else(|| panic!("{constraint} not checked on {part}"));
    Status::of(&check.result)
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_feature_with_a_body_and_no_value_takes_the_design_value() {
    let (model, _) = load_shapes();

    // The Model gives the engine no volume; the design's 1.0e6 mm³ makes
    // it 7.85 kg, over its 7.5 kg.
    assert_eq!(
        element(&model, "drive::engine::volume").computed,
        ["1000000.0 [mm³]"]
    );
    assert_eq!(
        check_on(&model, "Engine::engineMass", "engine"),
        Status::Violated
    );
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_redefinition_of_an_inherited_default_takes_the_design_value_and_the_def_keeps_it() {
    let (model, _) = load_shapes();

    // Gearbox's default is 0.2e6 mm³, 0.54 kg; the design's 0.3e6 mm³
    // makes the linked gearbox 0.81 kg, over its 0.6 kg. The spare keeps
    // the default. The drive is 7.85 kg + 0.81 kg.
    assert_eq!(
        element(&model, "drive::gearbox::volume").computed,
        ["300000.0 [mm³]"]
    );
    assert_eq!(
        check_on(&model, "Gearbox::gearboxMass", "gearbox"),
        Status::Violated
    );
    assert_eq!(
        check_on(&model, "Gearbox::gearboxMass", "spare"),
        Status::Holds
    );
    assert_eq!(
        element(&model, "Gearbox::volume").value.as_deref(),
        Some("default 200000.0 [millimetre ** 3]")
    );
    assert_eq!(element(&model, "drive::mass").computed, ["8.66 [kg]"]);
    assert_eq!(requirement_status(&model), Status::Violated);
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn valueless_features_are_recorded_by_their_resolved_names_with_the_model_value_they_inherit() {
    let (model, _) = load_shapes();
    let outcome =
        |feature: &str, object: &str, value: &str, model_value: Option<&str>| DesignValueOutcome {
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
            model_value: model_value.map(str::to_owned),
        };

    // The engine's volume has no Model Value anywhere; the gearbox's is
    // Gearbox's default.
    assert_eq!(
        model.design_values(),
        [
            outcome(
                "Drive::drive::engine::volume",
                "Engine",
                "1000000.0 [mm^3]",
                None
            ),
            outcome(
                "Drive::drive::gearbox::volume",
                "Assembly.Link.",
                "300000.0 [mm^3]",
                Some("default 200000.0 [millimetre ** 3]")
            ),
        ]
    );
}

/// `fixtures/no_design_values`, loaded with a tool that knows only the
/// objects in `known` and can't read the files in `unreadable`.
fn load_without(known: &[&str], unreadable: Vec<(&'static str, &'static str)>) -> Model {
    let (mut tool, _) = designed();
    tool.volumes.retain(|(object, _)| known.contains(object));
    tool.unreadable = unreadable;
    let model = OpenSysmlBackend::new()
        .with_tool("FreeCAD", tool)
        .load(&fixture("no_design_values"))
        .unwrap();
    assert!(model.diagnostics().is_empty(), "{:?}", model.diagnostics());
    model
}

/// What checking `constraint` on `part` found, and why when undecided.
fn check_result(model: &Model, constraint: &str, part: &str) -> (Status, Option<String>) {
    let part = QualifiedName::new(format!("Drive::drive::{part}"));
    let check = element(model, constraint)
        .checks
        .iter()
        .find(|c| c.part == part)
        .unwrap_or_else(|| panic!("{constraint} not checked on {part}"));
    let status = Status::of(&check.result);
    let reason = viewer_core::verdict::detail(&check.result)
        .filter(|_| status == Status::Undecided)
        .map(str::to_owned);
    (status, reason)
}

fn undecided(reason: &str) -> (Status, Option<String>) {
    (Status::Undecided, Some(reason.to_owned()))
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_failed_link_makes_exactly_its_dependent_checks_undecided_and_says_why() {
    let model = load_without(&["Engine", "Pump"], Vec::new());
    let gearbox =
        "no Design Value for Drive::drive::gearbox::volume (FreeCAD: no object Assembly.Link.)";

    // The gearbox has no Design Value; Gearbox's default (0.54 kg, within
    // 0.6 kg) must not show through.
    assert_eq!(
        check_result(&model, "Gearbox::gearboxMass", "gearbox"),
        undecided(gearbox)
    );
    let requirement = viewer_core::verdict::of(&model, element(&model, "driveMass")).unwrap();
    assert_eq!(requirement.status, Status::Undecided);
    assert_eq!(requirement.findings[0].detail.as_deref(), Some(gearbox));
    // The engine's Design Value still decides its check, and the unlinked
    // spares are judged on their Model Values.
    assert_eq!(
        check_result(&model, "Engine::engineMass", "engine").0,
        Status::Violated
    );
    assert_eq!(
        check_result(&model, "Engine::engineMass", "spareEngine").0,
        Status::Holds
    );
    assert_eq!(
        check_result(&model, "Gearbox::gearboxMass", "spareGearbox").0,
        Status::Holds
    );
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_file_the_tool_cannot_read_leaves_every_feature_linked_into_it_without_a_design_value() {
    let model = load_without(
        &["Engine", "Assembly.Link.", "Pump"],
        vec![("drive.FCStd", "cannot open drive.FCStd")],
    );

    // The engine's own Model Value (7.065 kg, within 7.5 kg) is not used.
    assert_eq!(
        check_result(&model, "Engine::engineMass", "engine"),
        undecided(
            "no Design Value for Drive::drive::engine::volume (FreeCAD: cannot open drive.FCStd)"
        )
    );
    assert_eq!(
        check_result(&model, "Gearbox::gearboxMass", "gearbox"),
        undecided(
            "no Design Value for Drive::drive::gearbox::volume (FreeCAD: cannot open drive.FCStd)"
        )
    );
    let reason = DesignValue::Missing {
        reason: "cannot open drive.FCStd".to_owned(),
    };
    let missing: Vec<&str> = model
        .design_values()
        .iter()
        .filter(|o| o.design_value == reason)
        .map(|o| o.feature.as_str())
        .collect();
    assert_eq!(
        missing,
        [
            "Drive::drive::engine::volume",
            "Drive::drive::gearbox::volume"
        ]
    );
    assert_eq!(
        check_result(&model, "Engine::engineMass", "spareEngine").0,
        Status::Holds
    );
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_design_value_of_the_wrong_dimension_is_undecided() {
    let model = load_without(&["Engine", "Assembly.Link.", "Pump"], Vec::new());

    // The pump's mass is linked to its volume, 1.5e6 mm³.
    assert_eq!(
        check_result(&model, "Pump::pumpMass", "pump"),
        undecided(
            "Design Value for Drive::drive::pump::mass (FreeCAD: Pump.Volume) does not fit it: \
             cannot write 1500000.0 [mm**3] (dimension L^3) to a feature typed by MassValue \
             (dimension M)"
        )
    );
}

/// `fixtures/tool_link_mistakes`: the engine has two links to FreeCAD,
/// and the spare's volume a Tool Property without one.
fn load_mistakes() -> (Model, Calls) {
    let (tool, calls) = designed();
    let model = OpenSysmlBackend::new()
        .with_tool("FreeCAD", tool)
        .load(&fixture("tool_link_mistakes"))
        .unwrap();
    (model, calls)
}

/// The diagnostic whose message mentions `text`, as severity, message and
/// line.
fn diagnostic_about(model: &Model, text: &str) -> (Severity, String, Option<u32>) {
    let found = model
        .diagnostics()
        .iter()
        .find(|d| d.message.contains(text))
        .unwrap_or_else(|| panic!("no diagnostic about {text}: {:?}", model.diagnostics()));
    let location = found.location.as_ref();
    assert!(
        location.is_none_or(|l| l.file.ends_with("tool_link_mistakes/drive.sysml")),
        "{found:?}"
    );
    (
        found.severity,
        found.message.clone(),
        location.map(|l| l.line),
    )
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_tool_property_whose_owner_has_no_tool_link_is_a_diagnostic() {
    let (model, _) = load_mistakes();

    assert_eq!(
        diagnostic_about(&model, "spare::volume"),
        (
            Severity::Warning,
            "@ToolProperty on Drive::drive::spare::volume, but Drive::drive::spare has no \
             Tool Link: it takes no Design Value"
                .to_owned(),
            Some(25)
        )
    );
    assert_eq!(
        check_on(&model, "Engine::engineMass", "spare"),
        Status::Holds
    );
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn two_tool_links_to_one_tool_on_one_element_are_a_diagnostic_and_neither_is_read() {
    let (model, calls) = load_mistakes();

    assert_eq!(
        diagnostic_about(&model, "Tool Links to FreeCAD"),
        (
            Severity::Warning,
            "Drive::drive::engine has 2 Tool Links to FreeCAD: its features take no Design Value"
                .to_owned(),
            Some(17)
        )
    );
    assert!(calls.lock().unwrap().is_empty());
    let (status, reason) = check_result(&model, "Engine::engineMass", "engine");
    assert_eq!(status, Status::Undecided);
    assert_eq!(
        reason.as_deref(),
        Some(
            "no Design Value for Drive::drive::engine::volume \
             (FreeCAD: Drive::drive::engine has 2 Tool Links to FreeCAD)"
        )
    );
}
