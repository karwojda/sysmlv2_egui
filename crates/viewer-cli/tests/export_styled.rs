//! Exports use the Model Style in `diagrams/style.json`.

use std::ffi::OsString;
use std::path::Path;

use tempfile::TempDir;
use viewer_cli::{EXIT_OK, run};
use viewer_core::scene::Rgb;
use viewer_core::style::{Condition, Style, StyleRule, apply};
use viewer_core::{DiagramFile, DiagramKind, FakeBackend, Model, QualifiedName, build_scene, diagram_file, style, svg};

fn model() -> Model {
    Model::builder().package("P").part_def("P::Engine").part_def("P::Tank").build()
}

fn rules() -> Vec<StyleRule> {
    vec![StyleRule { condition: Condition::Kind("part def".to_owned()), fill: Rgb(0x1f, 0x2a, 0x44), label: None }]
}

fn folder_with(style_text: Option<&str>) -> TempDir {
    let folder = TempDir::new().unwrap();
    let file = DiagramFile::new("overview", DiagramKind::General, QualifiedName::from("P"));
    let path = diagram_file::path_for(folder.path(), "overview");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, file.to_text()).unwrap();
    if let Some(text) = style_text {
        std::fs::write(style::path_for(folder.path()), text).unwrap();
    }
    folder
}

fn export(folder: &Path, out: &Path) -> (u8, String) {
    let args: Vec<OsString> = vec![
        "sysml-viewer".into(),
        "export".into(),
        folder.into(),
        "--diagram".into(),
        "overview".into(),
        "--out".into(),
        out.into(),
    ];
    let mut stderr = Vec::new();
    let code = run(args, &mut FakeBackend::new(model()), &mut stderr);
    (code, String::from_utf8(stderr).unwrap())
}

#[test]
fn the_model_style_colors_the_export_exactly_as_the_styled_scene() {
    let folder = folder_with(Some(&Style::new(rules()).to_text()));
    let out = folder.path().join("overview.svg");

    let (code, stderr) = export(folder.path(), &out);

    assert_eq!(code, EXIT_OK, "{stderr}");
    let model = model();
    let mut scene = build_scene(&model, DiagramKind::General, &QualifiedName::from("P")).unwrap();
    apply(&mut scene, &model, &[&rules()]);
    assert_eq!(std::fs::read_to_string(&out).unwrap(), svg::render(&scene));
}

#[test]
fn a_broken_style_file_warns_and_exports_unstyled() {
    let folder = folder_with(Some("{ not json"));
    let out = folder.path().join("overview.svg");

    let (code, stderr) = export(folder.path(), &out);

    assert_eq!(code, EXIT_OK);
    assert!(stderr.contains("warning: invalid Style file"), "{stderr}");
    assert!(stderr.contains("Exporting without the Model Style"), "{stderr}");
    let model = model();
    let plain = build_scene(&model, DiagramKind::General, &QualifiedName::from("P")).unwrap();
    assert_eq!(std::fs::read_to_string(&out).unwrap(), svg::render(&plain));
}

#[test]
fn styled_exports_are_deterministic() {
    let folder = folder_with(Some(&Style::new(rules()).to_text()));
    let (a, b) = (folder.path().join("a.svg"), folder.path().join("b.svg"));

    export(folder.path(), &a);
    export(folder.path(), &b);

    assert_eq!(std::fs::read(a).unwrap(), std::fs::read(b).unwrap());
}

#[test]
fn a_diagram_s_own_style_wins_and_can_switch_the_model_style_off() {
    let folder = folder_with(Some(&Style::new(rules()).to_text()));
    let path = diagram_file::path_for(folder.path(), "overview");
    let mut file = DiagramFile::parse(&std::fs::read_to_string(&path).unwrap()).unwrap();
    file.style.model_style = false;
    std::fs::write(&path, file.to_text()).unwrap();
    let out = folder.path().join("overview.svg");

    let (code, stderr) = export(folder.path(), &out);

    assert_eq!(code, EXIT_OK, "{stderr}");
    let model = model();
    let plain = build_scene(&model, DiagramKind::General, &QualifiedName::from("P")).unwrap();
    assert_eq!(std::fs::read_to_string(&out).unwrap(), svg::render(&plain), "Model Style off");
}
