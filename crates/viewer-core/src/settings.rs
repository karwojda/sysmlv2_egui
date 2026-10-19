//! Per-Model viewer settings, kept beside the Diagram Files in
//! `diagrams/settings.json` so they travel with the Model.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::diagram_file::FOLDER;

const FILE_NAME: &str = "settings.json";
const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Settings {
    pub format_version: u32,
    /// Whether analysis usages run automatically on every load.
    #[serde(default = "yes")]
    pub run_analyses: bool,
}

fn yes() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            format_version: FORMAT_VERSION,
            run_analyses: true,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SettingsError {
    #[error("cannot read {path}: {source}")]
    Unreadable {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid settings file {path}: {source}")]
    Invalid {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("settings file {path} has format version {found}, expected {FORMAT_VERSION}")]
    UnsupportedVersion { path: PathBuf, found: u32 },
}

impl Settings {
    pub fn to_text(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).expect("settings always serialize");
        text.push('\n');
        text
    }

    /// Writes the settings into `model_folder`.
    pub fn save(&self, model_folder: &Path) -> std::io::Result<()> {
        let path = path_for(model_folder);
        std::fs::create_dir_all(path.parent().expect("the settings file is in a folder"))?;
        std::fs::write(path, self.to_text())
    }
}

/// Where the settings of `model_folder` live.
pub fn path_for(model_folder: &Path) -> PathBuf {
    model_folder.join(FOLDER).join(FILE_NAME)
}

/// The settings of `model_folder`: the defaults when it has none.
pub fn load(model_folder: &Path) -> Result<Settings, SettingsError> {
    let path = path_for(model_folder);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Settings::default());
        }
        Err(source) => return Err(SettingsError::Unreadable { path, source }),
    };
    let settings: Settings =
        serde_json::from_str(&text).map_err(|source| SettingsError::Invalid {
            path: path.clone(),
            source,
        })?;
    if settings.format_version != FORMAT_VERSION {
        return Err(SettingsError::UnsupportedVersion {
            path,
            found: settings.format_version,
        });
    }
    Ok(settings)
}
