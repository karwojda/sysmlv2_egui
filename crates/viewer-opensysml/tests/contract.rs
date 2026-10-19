//! Seam 2: the OpenSysML backend returns the same Model the fake backend
//! serves for equivalent fixtures. These need the `sysml-grpc` service (the
//! client downloads it on first use), so they are opt-in:
//! `cargo test -p viewer-opensysml -- --ignored`.

use std::path::{Path, PathBuf};

use viewer_core::model::{Direction, Severity};
use viewer_core::{BackendError, FakeBackend, Model, ModelBackend, QualifiedName};
use viewer_opensysml::OpenSysmlBackend;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

/// Asserts element by element, so a mismatch names the element.
fn assert_same_model(real: &Model, fake: &Model) {
    assert_eq!(real.diagnostics(), fake.diagnostics());
    let names = |m: &Model| {
        m.elements()
            .map(|e| e.qualified_name.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(real), names(fake));
    for (real, fake) in real.elements().zip(fake.elements()) {
        assert_eq!(real, fake);
    }
}

/// The fake-backend twin of `fixtures/vehicles`.
fn vehicles() -> Model {
    let vehicles = fixture("vehicles").join("vehicles.sysml");
    let fleet = fixture("vehicles").join("fleet.sysml");
    Model::builder()
        .package("Fleet")
        .at(&fleet, 1)
        .part_def("Fleet::Taxi")
        .specializes("Vehicles::Car")
        .at(&fleet, 3)
        .package("Vehicles")
        .at(&vehicles, 1)
        .part_def("Vehicles::Vehicle")
        .at(&vehicles, 2)
        .part_def("Vehicles::Car")
        .specializes("Vehicles::Vehicle")
        .at(&vehicles, 3)
        .part_def("Vehicles::Truck")
        .specializes("Vehicles::Vehicle")
        .at(&vehicles, 4)
        .part_def("Vehicles::ElectricCar")
        .specializes("Vehicles::Car")
        .at(&vehicles, 5)
        .part_def("Vehicles::Engine")
        .at(&vehicles, 6)
        .requirement_def("Vehicles::TopSpeed")
        .at(&vehicles, 7)
        .package("Vehicles::Parts")
        .at(&vehicles, 8)
        .part_def("Vehicles::Parts::Wheel")
        .at(&vehicles, 9)
        .build()
}

/// The fake-backend twin of `fixtures/structure`, also used (without
/// source lines) by the core crate's General Diagram tests.
fn structure() -> Model {
    let file = fixture("structure").join("structure.sysml");
    Model::builder()
        .package("Structure")
        .at(&file, 1)
        .attribute_def("Structure::Fuel")
        .at(&file, 2)
        .attribute_def("Structure::Mass")
        .at(&file, 3)
        .port_def("Structure::FuelPort")
        .at(&file, 4)
        .attribute("Structure::FuelPort::fuel")
        .typed_by("Structure::Fuel")
        .direction(Direction::Out)
        .at(&file, 5)
        .interface_def("Structure::FuelInterface")
        .at(&file, 7)
        .port("Structure::FuelInterface::supply")
        .typed_by("Structure::FuelPort")
        .at(&file, 8)
        .port("Structure::FuelInterface::demand")
        .typed_by("Structure::FuelPort")
        .conjugated()
        .at(&file, 9)
        .connection_def("Structure::FuelLine")
        .at(&file, 11)
        .unsupported("Structure::FuelLine::a", "reference")
        .typed_by("Structure::FuelPort")
        .at(&file, 12)
        .unsupported("Structure::FuelLine::b", "reference")
        .typed_by("Structure::FuelPort")
        .at(&file, 13)
        .part_def("Structure::Engine")
        .doc("Turns fuel into motion.")
        .at(&file, 15)
        .attribute("Structure::Engine::mass")
        .typed_by("Structure::Mass")
        .at(&file, 17)
        .port("Structure::Engine::fuelIn")
        .typed_by("Structure::FuelPort")
        .conjugated()
        .direction(Direction::In)
        .at(&file, 18)
        .part_def("Structure::Tank")
        .at(&file, 20)
        .port("Structure::Tank::fuelOut")
        .typed_by("Structure::FuelPort")
        .direction(Direction::Out)
        .at(&file, 21)
        .part_def("Structure::Vehicle")
        .at(&file, 23)
        .part("Structure::Vehicle::engine")
        .typed_by("Structure::Engine")
        .multiplicity("1", "1")
        .at(&file, 24)
        .part("Structure::Vehicle::tanks")
        .typed_by("Structure::Tank")
        .multiplicity("1", "2")
        .at(&file, 25)
        .attribute("Structure::Vehicle::mass")
        .typed_by("Structure::Mass")
        .at(&file, 26)
        .connection("Structure::Vehicle::line")
        .typed_by("Structure::FuelLine")
        .end(&["Structure::Vehicle::tanks", "Structure::Tank::fuelOut"])
        .end(&["Structure::Vehicle::engine", "Structure::Engine::fuelIn"])
        .at(&file, 27)
        .interface("Structure::Vehicle::iface")
        .typed_by("Structure::FuelInterface")
        .end(&["Structure::Vehicle::tanks", "Structure::Tank::fuelOut"])
        .end(&["Structure::Vehicle::engine", "Structure::Engine::fuelIn"])
        .at(&file, 28)
        .part("Structure::vehicle")
        .typed_by("Structure::Vehicle")
        .at(&file, 30)
        .part("Structure::spare")
        .typed_by("Structure::Engine")
        .multiplicity("0", "*")
        .at(&file, 31)
        .connection("Structure::link")
        .end(&["Structure::vehicle"])
        .end(&["Structure::spare"])
        .at(&file, 32)
        .build()
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_the_same_model_as_the_fake_backend() {
    let real = OpenSysmlBackend::new().load(&fixture("vehicles")).unwrap();
    let fake = FakeBackend::new(vehicles())
        .load(&fixture("vehicles"))
        .unwrap();

    assert_same_model(&real, &fake);
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_the_full_language_subset_like_the_fake_backend() {
    let real = OpenSysmlBackend::new().load(&fixture("structure")).unwrap();

    assert_same_model(&real, &structure());
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn resolves_types_from_imports_and_the_standard_library() {
    let model = OpenSysmlBackend::new().load(&fixture("imports")).unwrap();
    let typed_by = |name: &str| {
        model
            .element(&QualifiedName::from(name))
            .unwrap()
            .typed_by
            .clone()
    };

    assert_eq!(model.diagnostics(), &[]);
    assert_eq!(
        typed_by("Uses::limit"),
        Some(QualifiedName::from("ScalarValues::Real"))
    );
    assert_eq!(
        typed_by("Defs::Car::speed"),
        Some(QualifiedName::from("ScalarValues::Real"))
    );
    assert_eq!(
        typed_by("Uses::car"),
        Some(QualifiedName::from("Defs::Car"))
    );
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_sysml_and_kerml_files_in_subfolders_and_ignores_other_files() {
    let model = OpenSysmlBackend::new().load(&fixture("nested")).unwrap();

    assert!(model.element(&QualifiedName::from("Top::A")).is_some());
    assert!(model.element(&QualifiedName::from("Deep")).is_some());
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn reports_syntax_errors_as_diagnostics_with_file_and_line() {
    let model = OpenSysmlBackend::new().load(&fixture("broken")).unwrap();

    assert!(model.has_errors());
    let error = model
        .diagnostics()
        .iter()
        .find(|d| d.severity == Severity::Error)
        .unwrap();
    let location = error.location.as_ref().expect("the error has a location");
    assert!(location.file.ends_with("broken.sysml"), "{location:?}");
    assert_eq!(location.line, 3);
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn one_backend_loads_several_models() {
    let mut backend = OpenSysmlBackend::new();

    let first = backend.load(&fixture("vehicles")).unwrap();
    let second = backend.load(&fixture("nested")).unwrap();

    assert!(first.element(&QualifiedName::from("Vehicles")).is_some());
    assert!(second.element(&QualifiedName::from("Vehicles")).is_none());
}

#[test]
fn a_missing_folder_is_an_error_without_starting_the_service() {
    let result = OpenSysmlBackend::new().load(&fixture("does-not-exist"));

    assert!(matches!(result, Err(BackendError::FolderNotFound(_))));
}

#[test]
fn an_empty_folder_is_an_empty_model_without_starting_the_service() {
    let empty = std::env::temp_dir().join(format!("viewer-opensysml-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();

    let model = OpenSysmlBackend::new().load(&empty).unwrap();

    std::fs::remove_dir(&empty).unwrap();
    assert_eq!(model, Model::default());
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn a_broken_file_does_not_cost_other_files_their_source_lines() {
    let folder =
        std::env::temp_dir().join(format!("viewer-opensysml-mixed-{}", std::process::id()));
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::copy(
        fixture("broken").join("broken.sysml"),
        folder.join("broken.sysml"),
    )
    .unwrap();
    std::fs::copy(
        fixture("vehicles").join("vehicles.sysml"),
        folder.join("vehicles.sysml"),
    )
    .unwrap();

    let model = OpenSysmlBackend::new().load(&folder).unwrap();
    std::fs::remove_dir_all(&folder).unwrap();

    let car = model
        .element(&QualifiedName::from("Vehicles::Car"))
        .unwrap();
    assert_eq!(car.source.as_ref().map(|s| s.line), Some(3));
    assert!(model.has_errors());
    assert!(
        model
            .diagnostics()
            .iter()
            .all(|d| d.severity == Severity::Error),
        "no extra warning repeats the broken file's error: {:?}",
        model.diagnostics()
    );
}

#[path = "../../viewer-core/tests/common/mod.rs"]
mod common;

/// The Model without source locations, which the fake twin doesn't carry.
fn without_sources(model: &Model) -> Model {
    let elements = model.elements().cloned().map(|mut e| {
        e.source = None;
        e
    });
    Model::new(elements, model.diagnostics().to_vec())
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_the_drivetrain_like_the_interconnection_tests_fake_twin() {
    let real = OpenSysmlBackend::new()
        .load(&fixture("drivetrain"))
        .unwrap();

    assert_same_model(&without_sources(&real), &common::drivetrain());
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_unnamed_connections_flows_bindings_and_reference_parts_like_the_fake_twin() {
    let real = OpenSysmlBackend::new().load(&fixture("flows")).unwrap();

    assert_same_model(&without_sources(&real), &common::flows());
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_views_with_their_exposes_and_renderings_like_the_fake_twin() {
    let real = OpenSysmlBackend::new().load(&fixture("views")).unwrap();

    assert_same_model(&without_sources(&real), &common::views());
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_metadata_annotations_like_the_fake_twin() {
    let real = OpenSysmlBackend::new().load(&fixture("metadata")).unwrap();

    assert_same_model(&without_sources(&real), &common::metadata());
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_requirements_like_the_fake_twin() {
    let real = OpenSysmlBackend::new()
        .load(&fixture("requirements"))
        .unwrap();

    assert_same_model(&without_sources(&real), &common::requirements());
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn rebuilds_require_constraint_text_from_the_service_s_expression_trees() {
    let real = OpenSysmlBackend::new()
        .load(&fixture("constraints"))
        .unwrap();
    assert!(real.diagnostics().is_empty(), "{:?}", real.diagnostics());

    let requirement = real.element(&QualifiedName::new("E::R")).unwrap();

    assert_eq!(
        requirement.require,
        [
            "v.a <= limit",
            "(v.a + v.b) * 2 > 10 and not v.ok",
            "v.a == 3 or v.b != -1.5 implies v.ok",
            // Library elements arrive without their short names, so a
            // unit reads as its full name.
            "v.m <= 1650 [kilogram]",
            "\"x\" == \"y\" xor true",
            "Cap(v.a)",
            "v.a - (v.b - 1) >= v.b ** 2",
        ]
    );
}

/// The mass roll-up demo in `examples/mass-rollup`: its requirements are
/// checked against masses the service rolls up from the parts, in a variant
/// that fails and one that passes.
fn mass_rollup(variant: &str) -> Model {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/mass-rollup")
        .join(variant);
    let model = OpenSysmlBackend::new().load(&folder).unwrap();
    assert!(model.diagnostics().is_empty(), "{:?}", model.diagnostics());
    model
}

fn demo_status(model: &Model, requirement: &str) -> viewer_core::satisfaction::SatisfactionStatus {
    let name = if requirement.is_empty() {
        "MassRollup::massSpec".to_owned()
    } else {
        format!("MassRollup::massSpec::{requirement}")
    };
    viewer_core::satisfaction::status(model, &QualifiedName::new(name)).unwrap()
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn the_failing_mass_rollup_demo_fails_where_its_readme_says() {
    use viewer_core::satisfaction::SatisfactionStatus::*;
    let model = mass_rollup("failing");

    // 180 + 85 + 950 + 60 + 4 × 22 = 1363 kg ≤ 1400; the powertrain's
    // 180 + 85 = 265 kg > 250; nothing claims the tank.
    assert_eq!(demo_status(&model, "vehicleMass"), Satisfied);
    assert_eq!(demo_status(&model, "powertrainMass"), Violated);
    assert_eq!(demo_status(&model, "tankMass"), NotClaimed);
    assert_eq!(
        demo_status(&model, ""),
        Violated,
        "the specification rolls up its children"
    );
    let satisfiers: Vec<String> = model
        .claims()
        .filter_map(|c| c.satisfier.as_ref())
        .map(|s| s.to_string())
        .collect();
    assert_eq!(
        satisfiers,
        ["MassRollup::vehicle", "MassRollup::vehicle::powertrain"],
        "a chained `by` is followed"
    );
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn the_passing_mass_rollup_demo_meets_its_whole_budget() {
    use viewer_core::satisfaction::SatisfactionStatus::*;
    let model = mass_rollup("passing");

    for requirement in ["vehicleMass", "powertrainMass", "tankMass", ""] {
        assert_eq!(
            demo_status(&model, requirement),
            Satisfied,
            "{requirement:?}"
        );
    }
}

/// The SVGs the demo's README shows, `<variant>/mass-overview.svg`, drawn as
/// the CLI's Export draws them. `UPDATE_GOLDEN=1` rewrites them.
#[test]
#[ignore = "needs the sysml-grpc service"]
fn the_mass_rollup_readme_pictures_match_the_models() {
    use viewer_core::{DiagramFile, diagram_file, style, svg};
    for variant in ["failing", "passing"] {
        let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/mass-rollup")
            .join(variant);
        let model = mass_rollup(variant);
        let file = DiagramFile::parse(
            &std::fs::read_to_string(diagram_file::path_for(&folder, "mass-overview")).unwrap(),
        )
        .unwrap();
        let mut scene = file.source.build_scene(&model, &file.layout).unwrap();
        let model_style = style::load(&folder).unwrap();
        style::apply_styles(&mut scene, &model, model_style.as_ref(), &file.style);
        let drawn = svg::render(&scene);

        let picture = folder.join("mass-overview.svg");
        if std::env::var_os("UPDATE_GOLDEN").is_some() {
            std::fs::write(&picture, &drawn).unwrap();
        }
        let committed = std::fs::read_to_string(&picture).unwrap();
        assert_eq!(
            drawn, committed,
            "{variant}: the README picture is stale; rerun with UPDATE_GOLDEN=1"
        );
    }
}
