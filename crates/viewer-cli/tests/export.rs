//! The `export --diagram` command, driven with the fake backend.

use std::ffi::OsString;
use std::path::Path;

use tempfile::TempDir;
use viewer_cli::{EXIT_FAILURE, EXIT_OK, run};
use viewer_core::model::Severity;
use viewer_core::{DiagramFile, DiagramKind, FakeBackend, Model, QualifiedName, build_scene, diagram_file, svg};

fn vehicles() -> Model {
    Model::builder()
        .package("Vehicles")
        .part_def("Vehicles::Vehicle")
        .part_def("Vehicles::Car")
        .specializes("Vehicles::Vehicle")
        .build()
}

/// A Model folder holding the given Diagram Files.
fn model_folder(diagrams: &[DiagramFile]) -> TempDir {
    let folder = TempDir::new().unwrap();
    for diagram in diagrams {
        save(folder.path(), &diagram.name, &diagram.to_text());
    }
    folder
}

fn save(folder: &Path, name: &str, text: &str) {
    let path = diagram_file::path_for(folder, name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

fn general(name: &str, root: &str) -> DiagramFile {
    DiagramFile::new(name, DiagramKind::General, QualifiedName::from(root))
}

struct Outcome {
    code: u8,
    stderr: String,
}

fn export(model: Model, folder: &Path, diagram: &str, out: &Path) -> Outcome {
    let args: Vec<OsString> = vec![
        "sysml-viewer".into(),
        "export".into(),
        folder.into(),
        "--diagram".into(),
        diagram.into(),
        "--out".into(),
        out.into(),
    ];
    let mut stderr = Vec::new();
    let code = run(args, &mut FakeBackend::new(model), &mut stderr);
    Outcome { code, stderr: String::from_utf8(stderr).unwrap() }
}

#[test]
fn writes_the_diagram_as_svg() {
    let folder = model_folder(&[general("overview", "Vehicles")]);
    let out = folder.path().join("overview.svg");

    let outcome = export(vehicles(), folder.path(), "overview", &out);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    let expected = svg::render(&build_scene(&vehicles(), DiagramKind::General, &"Vehicles".into()).unwrap());
    assert_eq!(std::fs::read_to_string(&out).unwrap(), expected);
}

#[test]
fn creates_missing_output_folders() {
    let folder = model_folder(&[general("overview", "Vehicles")]);
    let out = folder.path().join("docs/img/overview.svg");

    let outcome = export(vehicles(), folder.path(), "overview", &out);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    assert!(out.is_file());
}

#[test]
fn fails_and_lists_diagnostics_when_the_model_has_errors() {
    let model = Model::builder()
        .package("Vehicles")
        .diagnostic(Severity::Error, "unexpected ';'")
        .build();
    let folder = model_folder(&[general("overview", "Vehicles")]);
    let out = folder.path().join("overview.svg");

    let outcome = export(model, folder.path(), "overview", &out);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("error: unexpected ';'"), "{}", outcome.stderr);
    assert!(!out.exists(), "nothing is written for a broken Model");
}

#[test]
fn reports_warnings_but_still_exports() {
    let model = Model::builder()
        .package("Vehicles")
        .diagnostic(Severity::Warning, "unused import")
        .build();
    let folder = model_folder(&[general("overview", "Vehicles")]);
    let out = folder.path().join("overview.svg");

    let outcome = export(model, folder.path(), "overview", &out);

    assert_eq!(outcome.code, EXIT_OK);
    assert!(outcome.stderr.contains("warning: unused import"), "{}", outcome.stderr);
    assert!(out.is_file());
}

#[test]
fn fails_and_names_the_diagram_when_its_root_element_is_gone() {
    let folder = model_folder(&[general("engines", "Vehicles::Engines")]);
    let out = folder.path().join("engines.svg");

    let outcome = export(vehicles(), folder.path(), "engines", &out);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("Diagram `engines`"), "{}", outcome.stderr);
    assert!(outcome.stderr.contains("Vehicles::Engines"), "{}", outcome.stderr);
    assert!(!out.exists());
}

#[test]
fn fails_clearly_when_the_diagram_file_does_not_exist() {
    let folder = model_folder(&[]);

    let outcome = export(vehicles(), folder.path(), "missing", &folder.path().join("x.svg"));

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("cannot read Diagram File"), "{}", outcome.stderr);
    assert!(outcome.stderr.contains("missing.diagram.json"), "{}", outcome.stderr);
}

#[test]
fn fails_clearly_when_the_diagram_file_is_invalid() {
    let folder = model_folder(&[]);
    save(folder.path(), "broken", "{ not json");

    let outcome = export(vehicles(), folder.path(), "broken", &folder.path().join("x.svg"));

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("invalid Diagram File"), "{}", outcome.stderr);
}

/// Runs the real binary, headless, against a real Model.
#[test]
#[ignore = "needs the sysml-grpc service"]
fn the_real_binary_exports_a_real_model() {
    let folder = TempDir::new().unwrap();
    let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("../viewer-opensysml/tests/fixtures/vehicles");
    for file in ["vehicles.sysml", "fleet.sysml"] {
        std::fs::copy(fixture.join(file), folder.path().join(file)).unwrap();
    }
    save(folder.path(), "overview", &general("overview", "Vehicles").to_text());
    let out = folder.path().join("overview.svg");

    let status = std::process::Command::new(env!("CARGO_BIN_EXE_sysml-viewer"))
        .arg("export")
        .arg(folder.path())
        .args(["--diagram", "overview", "--out"])
        .arg(&out)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .status()
        .unwrap();

    assert!(status.success());
    let svg = std::fs::read_to_string(&out).unwrap();
    assert!(svg.contains(">ElectricCar</text>"));
    assert!(!svg.contains(">Wheel</text>"), "nested package contents are not drawn");
}

#[test]
fn honours_the_diagram_files_layout() {
    let mut diagram = general("overview", "Vehicles");
    diagram.layout.nodes.insert(
        QualifiedName::from("Vehicles::Car"),
        viewer_core::NodePlacement { x: 412.0, y: 300.0, size: Some([177.0, 66.0]) },
    );
    diagram.layout.hidden.insert(QualifiedName::from("Vehicles::Vehicle"));
    let folder = model_folder(&[diagram]);
    let out = folder.path().join("overview.svg");

    let outcome = export(vehicles(), folder.path(), "overview", &out);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    let svg = std::fs::read_to_string(&out).unwrap();
    assert!(svg.contains(r#"<rect x="412" y="300" width="177" height="66""#), "{svg}");
    assert!(!svg.contains(">Vehicle</text>"), "hidden elements are not drawn");
}

#[test]
fn service_flags_are_read_before_the_backend_is_built_and_win_over_the_environment() {
    let args: Vec<OsString> = ["sysml-viewer", "export", "m", "--all", "--out-dir", "o", "--sysml-binary", "/opt/sysml-grpc", "--sysml-service", "localhost:50051"]
        .into_iter()
        .map(Into::into)
        .collect();

    let config = viewer_cli::service_config(&args);

    assert_eq!(config.binary.as_deref(), Some(Path::new("/opt/sysml-grpc")));
    assert_eq!(config.service.as_deref(), Some("localhost:50051"));
}

#[test]
fn service_flags_are_accepted_by_the_command() {
    let folder = model_folder(&[general("overview", "Vehicles")]);
    let out = folder.path().join("overview.svg");
    let args: Vec<OsString> = vec![
        "sysml-viewer".into(),
        "--sysml-service".into(),
        "localhost:50051".into(),
        "export".into(),
        folder.path().into(),
        "--diagram".into(),
        "overview".into(),
        "--out".into(),
        out.clone().into(),
    ];

    let code = run(args, &mut FakeBackend::new(vehicles()), &mut Vec::new());

    assert_eq!(code, EXIT_OK);
    assert!(out.is_file());
}

#[test]
fn exports_an_interconnection_diagram() {
    let model = Model::builder()
        .package("P")
        .port_def("P::Flow")
        .part_def("P::Source")
        .port("P::Source::out")
        .typed_by("P::Flow")
        .direction(viewer_core::model::Direction::Out)
        .part_def("P::Sink")
        .port("P::Sink::in")
        .typed_by("P::Flow")
        .direction(viewer_core::model::Direction::In)
        .part_def("P::System")
        .part("P::System::source")
        .typed_by("P::Source")
        .part("P::System::sink")
        .typed_by("P::Sink")
        .connection("P::System::link")
        .end(&["P::System::source", "P::Source::out"])
        .end(&["P::System::sink", "P::Sink::in"])
        .build();
    let folder = model_folder(&[DiagramFile::new("wiring", DiagramKind::Interconnection, QualifiedName::from("P::System"))]);
    let out = folder.path().join("wiring.svg");

    let outcome = export(model, folder.path(), "wiring", &out);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    let svg = std::fs::read_to_string(&out).unwrap();
    assert!(svg.contains(">source:Source</text>") && svg.contains(">link</text>"), "{svg}");
}

#[test]
fn fails_and_names_the_diagram_when_an_interconnection_root_is_not_a_part() {
    let folder = model_folder(&[DiagramFile::new("wiring", DiagramKind::Interconnection, QualifiedName::from("Vehicles"))]);

    let outcome = export(vehicles(), folder.path(), "wiring", &folder.path().join("wiring.svg"));

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("Diagram `wiring`") && outcome.stderr.contains("part"), "{}", outcome.stderr);
}
