//! Exporting the Model's views: `export --all` includes them, and
//! `export --view` exports one, driven with the fake backend.

use std::ffi::OsString;
use std::path::Path;

use tempfile::TempDir;
use viewer_cli::{EXIT_FAILURE, EXIT_OK, run};
use viewer_core::model::{ExposeScope, ModelBuilder};
use viewer_core::{
    DiagramFile, DiagramKind, DiagramSource, FakeBackend, Layout, Model, NodePlacement, QualifiedName, diagram_file,
    svg, view,
};

/// A vehicle with a tree view, an interconnection view and a table view.
fn base() -> ModelBuilder {
    Model::builder()
        .package("V")
        .part_def("V::Engine")
        .part_def("V::Vehicle")
        .part("V::Vehicle::engine")
        .typed_by("V::Engine")
        .part("V::vehicle")
        .typed_by("V::Vehicle")
        .part("V::vehicle::body")
        .view("V::vehicle tree")
        .exposes("V::vehicle", ExposeScope::ElementAndDescendants)
        .renders("Views::asTreeDiagram")
        .view("V::wiring")
        .exposes("V::Vehicle", ExposeScope::Element)
        .renders("Views::asInterconnectionDiagram")
        .view("V::table")
        .exposes("V::Vehicle", ExposeScope::Members)
        .renders("Views::asElementTable")
}

fn qn(s: &str) -> QualifiedName {
    QualifiedName::from(s)
}

struct Outcome {
    code: u8,
    stderr: String,
}

fn run_with(model: Model, args: &[&OsString]) -> Outcome {
    let mut all: Vec<OsString> = vec!["sysml-viewer".into(), "export".into()];
    all.extend(args.iter().map(|a| (*a).clone()));
    let mut stderr = Vec::new();
    let code = run(all, &mut FakeBackend::new(model), &mut stderr);
    Outcome { code, stderr: String::from_utf8(stderr).unwrap() }
}

fn export_all(model: Model, folder: &Path, out_dir: &Path) -> Outcome {
    run_with(model, &[&folder.into(), &"--all".into(), &"--out-dir".into(), &out_dir.into()])
}

fn export_view(model: Model, folder: &Path, name: &str, out: &Path) -> Outcome {
    run_with(model, &[&folder.into(), &"--view".into(), &name.into(), &"--out".into(), &out.into()])
}

fn save(folder: &Path, file: &DiagramFile) {
    let path = diagram_file::path_for(folder, &file.name);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, file.to_text()).unwrap();
}

fn svg_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> =
        std::fs::read_dir(dir).unwrap().map(|e| e.unwrap().file_name().into_string().unwrap()).collect();
    names.sort();
    names
}

#[test]
fn all_exports_every_supported_view_named_after_it() {
    let folder = TempDir::new().unwrap();
    let out_dir = folder.path().join("out");

    let outcome = export_all(base().build(), folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    assert_eq!(svg_files(&out_dir), ["vehicle-tree.svg", "wiring.svg"]);
    let model = base().build();
    let expected = svg::render(&view::build_scene(&model, &qn("V::vehicle tree"), &Layout::default()).unwrap());
    assert_eq!(std::fs::read_to_string(out_dir.join("vehicle-tree.svg")).unwrap(), expected, "same as the viewer");
}

#[test]
fn all_warns_about_unsupported_views_and_still_succeeds() {
    let folder = TempDir::new().unwrap();

    let outcome = export_all(base().build(), folder.path(), &folder.path().join("out"));

    assert_eq!(outcome.code, EXIT_OK);
    assert!(
        outcome.stderr.contains("warning: view `V::table` is not Exported: it renders as `asElementTable`"),
        "{}",
        outcome.stderr
    );
}

#[test]
fn all_exports_diagram_files_alongside_views() {
    let folder = TempDir::new().unwrap();
    save(folder.path(), &DiagramFile::new("overview", DiagramKind::General, qn("V")));
    let out_dir = folder.path().join("out");

    let outcome = export_all(base().build(), folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    assert_eq!(svg_files(&out_dir), ["overview.svg", "vehicle-tree.svg", "wiring.svg"]);
}

#[test]
fn a_view_saved_as_a_diagram_file_is_exported_once_under_that_name_with_its_layout() {
    let folder = TempDir::new().unwrap();
    let mut file = DiagramFile::of("parts", DiagramSource::View { view: qn("V::vehicle tree") });
    file.layout.nodes.insert(qn("V::vehicle"), NodePlacement { x: 300.0, y: 40.0, size: None });
    save(folder.path(), &file);
    let out_dir = folder.path().join("out");

    let outcome = export_all(base().build(), folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_OK, "{}", outcome.stderr);
    assert_eq!(svg_files(&out_dir), ["parts.svg", "wiring.svg"]);
    let model = base().build();
    let expected = svg::render(&view::build_scene(&model, &qn("V::vehicle tree"), &file.layout).unwrap());
    assert_eq!(std::fs::read_to_string(out_dir.join("parts.svg")).unwrap(), expected);
}

#[test]
fn two_views_with_the_same_name_fail_naming_both_before_writing_anything() {
    let model = base()
        .package("W")
        .part("W::thing")
        .view("W::wiring")
        .exposes("W::thing", ExposeScope::Element)
        .renders("Views::asTreeDiagram")
        .build();
    let folder = TempDir::new().unwrap();
    let out_dir = folder.path().join("out");

    let outcome = export_all(model, folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("view `V::wiring` and view `W::wiring` would both be Exported as wiring.svg"), "{}", outcome.stderr);
    assert!(!out_dir.exists(), "nothing written");
}

#[test]
fn a_view_clashing_with_a_diagram_file_name_fails_naming_both() {
    let folder = TempDir::new().unwrap();
    save(folder.path(), &DiagramFile::new("wiring", DiagramKind::General, qn("V")));

    let outcome = export_all(base().build(), folder.path(), &folder.path().join("out"));

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("Diagram File `wiring` and view `V::wiring`"), "{}", outcome.stderr);
}

#[test]
fn an_invalid_view_is_reported_and_the_rest_still_exported() {
    let model = base()
        .view("V::broken")
        .renders("Views::asInterconnectionDiagram")
        .build();
    let folder = TempDir::new().unwrap();
    let out_dir = folder.path().join("out");

    let outcome = export_all(model, folder.path(), &out_dir);

    assert_eq!(outcome.code, EXIT_FAILURE);
    assert!(outcome.stderr.contains("Diagram `broken`: `V::broken` can't be drawn: it exposes nothing"), "{}", outcome.stderr);
    assert_eq!(svg_files(&out_dir), ["vehicle-tree.svg", "wiring.svg"]);
}

#[test]
fn view_exports_one_view_by_simple_or_qualified_name() {
    let folder = TempDir::new().unwrap();
    let (a, b) = (folder.path().join("a.svg"), folder.path().join("b.svg"));

    let by_simple = export_view(base().build(), folder.path(), "wiring", &a);
    let by_qualified = export_view(base().build(), folder.path(), "V::vehicle tree", &b);

    assert_eq!(by_simple.code, EXIT_OK, "{}", by_simple.stderr);
    assert_eq!(by_qualified.code, EXIT_OK, "{}", by_qualified.stderr);
    assert!(std::fs::read_to_string(&a).unwrap().contains("engine"));
    assert!(std::fs::read_to_string(&b).unwrap().contains("body"));
}

#[test]
fn view_fails_clearly_for_unsupported_unknown_and_ambiguous_names() {
    let folder = TempDir::new().unwrap();
    let out = folder.path().join("x.svg");

    let unsupported = export_view(base().build(), folder.path(), "table", &out);
    assert_eq!(unsupported.code, EXIT_FAILURE);
    assert!(unsupported.stderr.contains("view `V::table` can't be drawn: it renders as `asElementTable`"), "{}", unsupported.stderr);

    let unknown = export_view(base().build(), folder.path(), "nope", &out);
    assert!(unknown.stderr.contains("the Model has no view named `nope`"), "{}", unknown.stderr);

    let model = base().package("W").view("W::wiring").renders("Views::asTreeDiagram").build();
    let ambiguous = export_view(model, folder.path(), "wiring", &out);
    assert!(ambiguous.stderr.contains("`wiring` names several views (`V::wiring`, `W::wiring`)"), "{}", ambiguous.stderr);
    assert!(!out.exists());
}
