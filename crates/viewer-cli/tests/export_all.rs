//! The `export --all` command, driven with the fake backend.

use std::ffi::OsString;
use std::path::Path;

use tempfile::TempDir;
use viewer_cli::{EXIT_FAILURE, EXIT_OK, run};
use viewer_core::model::Severity;
use viewer_core::{DiagramFile, DiagramKind, FakeBackend, Model, ModelBackend, QualifiedName, diagram_file};

fn vehicles() -> Model {
    Model::builder()
        .package("Vehicles")
        .part_def("Vehicles::Vehicle")
        .part_def("Vehicles::Car")
        .specializes("Vehicles::Vehicle")
        .package("Vehicles::Parts")
        .part_def("Vehicles::Parts::Wheel")
        .build()
}

fn model_folder(diagrams: &[(&str, &str)]) -> TempDir {
    let folder = TempDir::new().unwrap();
    for (name, root) in diagrams {
        let path = diagram_file::path_for(folder.path(), name);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let file = DiagramFile::new(*name, DiagramKind::General, QualifiedName::from(*root));
        std::fs::write(path, file.to_text()).unwrap();
    }
    folder
}

struct Outcome {
    code: u8,
    stderr: String,
}

fn export_all(backend: &mut dyn ModelBackend, folder: &Path, out_dir: &Path) -> Outcome {
    let args: Vec<OsString> = vec![
        "sysml-viewer".into(),
        "export".into(),
        folder.into(),
        "--all".into(),
        "--out-dir".into(),
        out_dir.into(),
    ];
    let mut stderr = Vec::new();
    let code = run(args, backend, &mut stderr);
    Outcome { code, stderr: String::from_utf8(stderr).unwrap() }
}

fn svg_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    names.sort();
    names
}

#[test]
fn exports_every_diagram_file_named_after_its_diagram() {
    let folder = model_folder(&[("overview", "Vehicles"), ("parts", "Vehicles::Parts")]);
    let out_dir = folder.path().join("docs/img");

    let outcome = export_all(&mut FakeBackend::new(vehicles()), folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    assert_eq!(svg_files(&out_dir), ["overview.svg", "parts.svg"]);
    assert!(std::fs::read_to_string(out_dir.join("parts.svg")).unwrap().contains(">Wheel</text>"));
}

#[test]
fn rerunning_on_unchanged_inputs_gives_identical_files() {
    let folder = model_folder(&[("overview", "Vehicles"), ("parts", "Vehicles::Parts")]);
    let (first, second) = (folder.path().join("first"), folder.path().join("second"));

    export_all(&mut FakeBackend::new(vehicles()), folder.path(), &first);
    export_all(&mut FakeBackend::new(vehicles()), folder.path(), &second);

    for name in svg_files(&first) {
        assert_eq!(std::fs::read(first.join(&name)).unwrap(), std::fs::read(second.join(&name)).unwrap(), "{name}");
    }
}

#[test]
fn reports_every_failed_diagram_and_still_exports_the_rest() {
    let folder = model_folder(&[("a-gone", "Vehicles::Gone"), ("overview", "Vehicles"), ("z-gone", "Old")]);
    let out_dir = folder.path().join("out");

    let outcome = export_all(&mut FakeBackend::new(vehicles()), folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("Diagram `a-gone`"), "{}", outcome.stderr);
    assert!(outcome.stderr.contains("Diagram `z-gone`"), "{}", outcome.stderr);
    assert!(outcome.stderr.contains("2 of 3 Diagrams failed"), "{}", outcome.stderr);
    assert_eq!(svg_files(&out_dir), ["overview.svg"]);
}

#[test]
fn fails_when_the_model_has_errors() {
    let model = Model::builder()
        .package("Vehicles")
        .diagnostic(Severity::Error, "unexpected ';'")
        .build();
    let folder = model_folder(&[("overview", "Vehicles")]);
    let out_dir = folder.path().join("out");

    let outcome = export_all(&mut FakeBackend::new(model), folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("error: unexpected ';'"), "{}", outcome.stderr);
    assert!(!out_dir.exists());
}

/// Fails the test if the Model is ever loaded.
struct UnusedBackend;

impl ModelBackend for UnusedBackend {
    fn load(&mut self, _folder: &Path) -> Result<Model, viewer_core::BackendError> {
        panic!("the Model should not be loaded");
    }
}

#[test]
fn a_model_without_diagram_files_succeeds_and_says_so() {
    let folder = model_folder(&[]);

    let outcome = export_all(&mut UnusedBackend, folder.path(), &folder.path().join("out"));

    assert_eq!(outcome.code, EXIT_OK);
    assert!(outcome.stderr.contains("no Diagram Files"), "{}", outcome.stderr);
}

#[test]
fn ignores_other_files_in_the_diagram_folder() {
    let folder = model_folder(&[("overview", "Vehicles")]);
    std::fs::write(folder.path().join("diagrams/README.md"), "notes").unwrap();
    let out_dir = folder.path().join("out");

    let outcome = export_all(&mut FakeBackend::new(vehicles()), folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    assert_eq!(svg_files(&out_dir), ["overview.svg"]);
}

#[test]
fn diagram_and_all_cannot_be_combined() {
    let folder = model_folder(&[("overview", "Vehicles")]);
    let args: Vec<OsString> = ["sysml-viewer", "export", ".", "--all", "--out-dir", "x", "--diagram", "overview"]
        .into_iter()
        .map(Into::into)
        .collect();

    let code = run(args, &mut UnusedBackend, &mut Vec::new());

    assert_ne!(code, EXIT_OK);
    drop(folder);
}
