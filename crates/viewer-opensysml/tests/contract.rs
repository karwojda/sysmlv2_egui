//! Seam 2: the OpenSysML backend returns the same Model the fake backend
//! serves for equivalent fixtures. These need the `sysml-grpc` service (the
//! client downloads it on first use), so they are opt-in:
//! `cargo test -p viewer-opensysml -- --ignored`.

use std::path::{Path, PathBuf};

use viewer_core::model::Severity;
use viewer_core::{BackendError, FakeBackend, Model, ModelBackend, QualifiedName};
use viewer_opensysml::OpenSysmlBackend;

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// The fake-backend twin of `fixtures/vehicles`.
fn vehicles() -> Model {
    Model::builder()
        .package("Fleet")
        .part_def("Fleet::Taxi")
        .specializes("Vehicles::Car")
        .package("Vehicles")
        .part_def("Vehicles::Vehicle")
        .part_def("Vehicles::Car")
        .specializes("Vehicles::Vehicle")
        .part_def("Vehicles::Truck")
        .specializes("Vehicles::Vehicle")
        .part_def("Vehicles::ElectricCar")
        .specializes("Vehicles::Car")
        .part_def("Vehicles::Engine")
        .unsupported("Vehicles::TopSpeed", "requirement def")
        .package("Vehicles::Parts")
        .part_def("Vehicles::Parts::Wheel")
        .build()
}

#[test]
#[ignore = "needs the sysml-grpc service"]
fn loads_the_same_model_as_the_fake_backend() {
    let real = OpenSysmlBackend::new().load(&fixture("vehicles")).unwrap();
    let fake = FakeBackend::new(vehicles()).load(&fixture("vehicles")).unwrap();

    assert_eq!(real.diagnostics(), &[]);
    assert_eq!(
        real.elements().collect::<Vec<_>>(),
        fake.elements().collect::<Vec<_>>()
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
