//! Diagram Files are readable, stable text that round-trips.

use viewer_core::{DiagramFile, DiagramFileError, DiagramKind, QualifiedName};

#[test]
fn is_readable_json_with_a_fixed_key_order() {
    let file = DiagramFile::new("overview", DiagramKind::General, QualifiedName::from("Vehicles"));

    assert_eq!(
        file.to_text(),
        r#"{
  "format_version": 1,
  "name": "overview",
  "kind": "general",
  "root": "Vehicles",
  "layout": {}
}
"#
    );
}

#[test]
fn round_trips() {
    let file = DiagramFile::new("overview", DiagramKind::General, QualifiedName::from("Vehicles::Parts"));

    assert_eq!(DiagramFile::parse(&file.to_text()).unwrap(), file);
}

#[test]
fn rejects_an_unknown_format_version() {
    let text = DiagramFile::new("x", DiagramKind::General, "P".into())
        .to_text()
        .replace("\"format_version\": 1", "\"format_version\": 99");

    assert!(matches!(
        DiagramFile::parse(&text),
        Err(DiagramFileError::UnsupportedVersion { found: 99 })
    ));
}
