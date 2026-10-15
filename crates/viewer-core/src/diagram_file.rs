//! Diagram Files: saved Diagrams stored beside the Model (ADR-0002).
//!
//! The format is pretty-printed JSON with a fixed key order, so diffs stay
//! reviewable.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::diagram::DiagramKind;
use crate::model::QualifiedName;

/// The folder inside a Model folder that holds its Diagram Files.
pub const FOLDER: &str = "diagrams";
const EXTENSION: &str = ".diagram.json";
const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagramFile {
    pub format_version: u32,
    pub name: String,
    pub kind: DiagramKind,
    pub root: QualifiedName,
    pub layout: Layout,
}

/// The user's adjustments to the automatic arrangement. Empty until Layout
/// editing exists.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Layout {}

#[derive(Debug, thiserror::Error)]
pub enum DiagramFileError {
    #[error("invalid Diagram File: {0}")]
    Invalid(#[from] serde_json::Error),
    #[error("Diagram File format version {found} is not supported (expected {FORMAT_VERSION})")]
    UnsupportedVersion { found: u32 },
}

impl DiagramFile {
    pub fn new(name: impl Into<String>, kind: DiagramKind, root: QualifiedName) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            name: name.into(),
            kind,
            root,
            layout: Layout::default(),
        }
    }

    pub fn parse(text: &str) -> Result<Self, DiagramFileError> {
        let file: DiagramFile = serde_json::from_str(text)?;
        if file.format_version != FORMAT_VERSION {
            return Err(DiagramFileError::UnsupportedVersion { found: file.format_version });
        }
        Ok(file)
    }

    pub fn to_text(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).expect("a Diagram File always serializes");
        text.push('\n');
        text
    }
}

/// The names of every Diagram File in `model_folder`, sorted. A Model
/// without a Diagram File folder has none.
pub fn names_in(model_folder: &Path) -> std::io::Result<Vec<String>> {
    let folder = model_folder.join(FOLDER);
    if !folder.is_dir() {
        return Ok(Vec::new());
    }
    let mut names = Vec::new();
    for entry in std::fs::read_dir(folder)? {
        let file_name = entry?.file_name();
        if let Some(name) = file_name.to_str().and_then(|n| n.strip_suffix(EXTENSION)) {
            names.push(name.to_owned());
        }
    }
    names.sort();
    Ok(names)
}

/// Where the Diagram File named `name` lives in `model_folder`.
pub fn path_for(model_folder: &Path, name: &str) -> PathBuf {
    model_folder.join(FOLDER).join(format!("{name}{EXTENSION}"))
}
