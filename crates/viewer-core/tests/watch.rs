//! Live reload: which files are the Model, when to reload, and what to show
//! when the reloaded Model has errors.

use std::path::Path;

use tempfile::TempDir;
use viewer_core::model::Severity;
use viewer_core::watch::{ChangeDetector, Shown, accept, fingerprint, model_files};
use viewer_core::{FakeBackend, Model, ModelBackend, QualifiedName};

fn write(folder: &Path, file: &str, text: &str) {
    let path = folder.join(file);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, text).unwrap();
}

#[test]
fn the_model_is_every_sysml_and_kerml_file_but_not_diagram_files() {
    let folder = TempDir::new().unwrap();
    write(folder.path(), "a.sysml", "package A;");
    write(folder.path(), "sub/b.kerml", "package B;");
    write(folder.path(), "diagrams/overview.diagram.json", "{}");
    write(folder.path(), "notes.md", "");

    let files: Vec<_> = model_files(folder.path())
        .unwrap()
        .into_iter()
        .map(|f| f.strip_prefix(folder.path()).unwrap().to_path_buf())
        .collect();

    assert_eq!(files, [Path::new("a.sysml"), Path::new("sub/b.kerml")]);
}

#[test]
fn the_fingerprint_changes_when_a_model_file_is_edited_added_or_removed() {
    let folder = TempDir::new().unwrap();
    write(folder.path(), "a.sysml", "package A;");
    let original = fingerprint(folder.path()).unwrap();

    write(folder.path(), "a.sysml", "package A { part def B; }");
    let edited = fingerprint(folder.path()).unwrap();
    write(folder.path(), "c.sysml", "package C;");
    let added = fingerprint(folder.path()).unwrap();
    std::fs::remove_file(folder.path().join("c.sysml")).unwrap();
    let removed = fingerprint(folder.path()).unwrap();

    assert_ne!(original, edited);
    assert_ne!(edited, added);
    assert_eq!(removed, edited);
}

#[test]
fn saving_a_diagram_file_does_not_change_the_fingerprint() {
    let folder = TempDir::new().unwrap();
    write(folder.path(), "a.sysml", "package A;");
    let before = fingerprint(folder.path()).unwrap();

    write(folder.path(), "diagrams/overview.diagram.json", "{}");

    assert_eq!(fingerprint(folder.path()).unwrap(), before);
}

fn fp(entries: &[(&str, u64)]) -> viewer_core::watch::Fingerprint {
    entries.iter().map(|(f, size)| (f.into(), (None, *size))).collect()
}

#[test]
fn a_change_reloads_once_it_has_held_for_one_more_look() {
    let mut detector = ChangeDetector::new(fp(&[("a", 1)]));

    assert!(!detector.observe(fp(&[("a", 1)])), "nothing changed");
    assert!(!detector.observe(fp(&[("a", 2)])), "changed, not yet settled");
    assert!(detector.observe(fp(&[("a", 2)])), "settled: reload");
    assert!(!detector.observe(fp(&[("a", 2)])), "already reloaded");
}

#[test]
fn a_burst_of_saves_gives_one_reload() {
    let mut detector = ChangeDetector::new(fp(&[("a", 1), ("b", 1)]));

    let reloads = [
        fp(&[("a", 2), ("b", 1)]),
        fp(&[("a", 2), ("b", 2)]),
        fp(&[("a", 3), ("b", 2)]),
        fp(&[("a", 3), ("b", 2)]),
        fp(&[("a", 3), ("b", 2)]),
    ]
    .into_iter()
    .filter(|f| detector.observe(f.clone()))
    .count();

    assert_eq!(reloads, 1);
}

#[test]
fn a_change_reverted_before_it_settles_causes_no_reload() {
    let mut detector = ChangeDetector::new(fp(&[("a", 1)]));

    assert!(!detector.observe(fp(&[("a", 2)])));
    assert!(!detector.observe(fp(&[("a", 1)])));
    assert!(!detector.observe(fp(&[("a", 1)])));
}

fn good(part: &str) -> Model {
    let model = Model::builder().package("P").part_def(part).build();
    FakeBackend::new(model).load(Path::new("m")).unwrap()
}

fn broken() -> Model {
    Model::builder().package("P").diagnostic(Severity::Error, "expected a name").build()
}

#[test]
fn a_good_reload_replaces_what_is_shown() {
    let shown = accept(Some(accept(None, good("P::A"))), good("P::B"));

    assert!(!shown.stale);
    assert!(shown.model.element(&QualifiedName::from("P::B")).is_some());
}

#[test]
fn a_reload_with_errors_keeps_the_last_good_model_and_lists_the_new_errors() {
    let shown = accept(Some(accept(None, good("P::A"))), broken());

    assert!(shown.stale);
    assert!(shown.model.element(&QualifiedName::from("P::A")).is_some());
    assert_eq!(shown.diagnostics[0].message, "expected a name");
}

#[test]
fn the_last_good_model_stays_through_several_broken_reloads_until_a_good_one() {
    let first = accept(None, good("P::A"));
    let still_broken = accept(Some(accept(Some(first), broken())), broken());
    assert!(still_broken.stale);
    assert!(still_broken.model.element(&QualifiedName::from("P::A")).is_some());

    let fixed = accept(Some(still_broken), good("P::C"));
    assert!(!fixed.stale);
    assert!(fixed.model.element(&QualifiedName::from("P::C")).is_some());
    assert!(fixed.diagnostics.is_empty());
}

#[test]
fn a_broken_first_load_is_shown_as_it_is() {
    let shown: Shown = accept(None, broken());

    assert!(!shown.stale);
    assert!(shown.model.has_errors());
}

#[test]
fn editing_the_model_style_changes_the_fingerprint() {
    let folder = TempDir::new().unwrap();
    write(folder.path(), "a.sysml", "package A;");
    let before = fingerprint(folder.path()).unwrap();

    write(folder.path(), "diagrams/style.json", r#"{"format_version": 1, "rules": []}"#);

    assert_ne!(fingerprint(folder.path()).unwrap(), before);
    assert_eq!(model_files(folder.path()).unwrap().len(), 1, "but it isn't a Model file");
}
