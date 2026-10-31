//! Diagram Files are readable, stable text that round-trips.

use viewer_core::{
    DiagramFile, DiagramFileError, DiagramKind, DiagramSource, DiagramStyle, QualifiedName,
};

#[test]
fn is_readable_json_with_a_fixed_key_order() {
    let file = DiagramFile::new(
        "overview",
        DiagramKind::General,
        QualifiedName::from("Vehicles"),
    );

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
    let file = DiagramFile::new(
        "overview",
        DiagramKind::General,
        QualifiedName::from("Vehicles::Parts"),
    );

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

#[test]
fn a_model_defined_diagram_refers_to_its_view_instead_of_a_kind_and_root() {
    let file = DiagramFile::of(
        "vehicle-tree",
        DiagramSource::View {
            view: QualifiedName::from("V::vehicle tree"),
        },
    );

    assert_eq!(
        file.to_text(),
        r#"{
  "format_version": 1,
  "name": "vehicle-tree",
  "view": "V::vehicle tree",
  "layout": {}
}
"#
    );
    assert_eq!(DiagramFile::parse(&file.to_text()).unwrap(), file);
}

#[test]
fn a_tool_defined_file_still_reads_as_a_kind_and_root() {
    let text = "{\"format_version\": 1, \"name\": \"x\", \"kind\": \"interconnection\", \"root\": \"P::V\", \"layout\": {}}";

    let file = DiagramFile::parse(text).unwrap();

    assert_eq!(
        file.source,
        DiagramSource::Tool {
            kind: DiagramKind::Interconnection,
            root: QualifiedName::from("P::V")
        }
    );
}

#[test]
fn a_diagram_without_its_own_style_writes_no_style_block() {
    let file = DiagramFile::new("x", DiagramKind::General, QualifiedName::from("P"));

    assert!(!file.to_text().contains("style"));
    assert_eq!(file.style, DiagramStyle::default());
    assert!(file.style.model_style && !file.style.legend);
}

#[test]
fn a_diagram_style_round_trips_after_the_layout() {
    use viewer_core::scene::Rgb;
    use viewer_core::style::{Condition, StyleRule};
    let mut file = DiagramFile::new("x", DiagramKind::General, QualifiedName::from("P"));
    file.style = DiagramStyle {
        model_style: false,
        rules: vec![StyleRule {
            condition: Condition::Package("P::Sub".into()),
            fill: Rgb(1, 2, 3),
            label: None,
        }],
        legend: true,
    };

    let text = file.to_text();

    assert!(text.find("\"layout\"").unwrap() < text.find("\"style\"").unwrap());
    assert!(text.contains("\"package\": \"P::Sub\""), "{text}");
    assert_eq!(DiagramFile::parse(&text).unwrap(), file);
}

#[test]
fn missing_style_fields_take_their_defaults() {
    let text = r#"{"format_version": 1, "name": "x", "kind": "general", "root": "P", "layout": {}, "style": {"legend": true}}"#;

    let style = DiagramFile::parse(text).unwrap().style;

    assert_eq!(
        style,
        DiagramStyle {
            model_style: true,
            rules: vec![],
            legend: true
        }
    );
}

#[test]
fn inherited_members_turned_off_are_saved_and_on_writes_nothing() {
    let mut file = DiagramFile::new("x", DiagramKind::General, "P".into());
    assert!(!file.to_text().contains("inherited_hidden"));

    file.layout.inherited_hidden = true;
    let text = file.to_text();

    assert!(text.contains(r#""inherited_hidden": true"#), "{text}");
    assert!(DiagramFile::parse(&text).unwrap().layout.inherited_hidden);
}
