//! Per-Model viewer settings in `diagrams/settings.json`.

use tempfile::TempDir;
use viewer_core::settings::{self, Settings, SettingsError};

fn write(folder: &TempDir, text: &str) {
    std::fs::create_dir_all(folder.path().join("diagrams")).unwrap();
    std::fs::write(settings::path_for(folder.path()), text).unwrap();
}

#[test]
fn a_model_without_settings_runs_analyses() {
    let folder = TempDir::new().unwrap();

    assert_eq!(settings::load(folder.path()).unwrap(), Settings::default());
    assert!(Settings::default().run_analyses);
}

#[test]
fn settings_round_trip_as_readable_json() {
    let folder = TempDir::new().unwrap();
    let off = Settings {
        run_analyses: false,
        ..Settings::default()
    };

    off.save(folder.path()).unwrap();

    assert_eq!(
        std::fs::read_to_string(settings::path_for(folder.path())).unwrap(),
        "{\n  \"format_version\": 1,\n  \"run_analyses\": false\n}\n"
    );
    assert_eq!(settings::load(folder.path()).unwrap(), off);
}

#[test]
fn a_missing_field_takes_its_default() {
    let folder = TempDir::new().unwrap();
    write(&folder, r#"{"format_version": 1}"#);

    assert!(settings::load(folder.path()).unwrap().run_analyses);
}

#[test]
fn a_broken_settings_file_is_an_error_naming_it() {
    let folder = TempDir::new().unwrap();
    write(&folder, "{ not json");

    let error = settings::load(folder.path()).unwrap_err();

    assert!(matches!(error, SettingsError::Invalid { .. }));
    assert!(error.to_string().contains("settings.json"), "{error}");
}
