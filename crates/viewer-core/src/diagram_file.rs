//! Diagram Files: saved Diagrams stored beside the Model (ADR-0002).
//!
//! The format is pretty-printed JSON with a fixed key order, so diffs stay
//! reviewable.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::diagram::DiagramKind;
use crate::model::{Model, QualifiedName};

/// The folder inside a Model folder that holds its Diagram Files.
pub const FOLDER: &str = "diagrams";
const EXTENSION: &str = ".diagram.json";
const FORMAT_VERSION: u32 = 1;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiagramFile {
    pub format_version: u32,
    pub name: String,
    pub kind: DiagramKind,
    pub root: QualifiedName,
    pub layout: Layout,
}

/// The user's adjustments to the automatic arrangement, keyed by element
/// qualified name (edges by their edge key). Entries that match nothing in
/// the Model are Orphaned Layout Entries: ignored when drawing, but kept.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Layout {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub nodes: BTreeMap<QualifiedName, NodePlacement>,
    /// Bend points of edges, in scene coordinates, from source to target.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub edges: BTreeMap<String, Vec<[f32; 2]>>,
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub hidden: BTreeSet<QualifiedName>,
    /// Elements drawn without their compartments.
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub collapsed: BTreeSet<QualifiedName>,
    /// In an Interconnection Diagram, the Opened Parts, by feature path from
    /// the root (`rearAxleAssembly`, `rearAxleAssembly.differential`).
    #[serde(default, skip_serializing_if = "BTreeSet::is_empty")]
    pub opened: BTreeSet<String>,
}

/// Where a node sits, in scene coordinates, and its size if the user set
/// one; without a size the node keeps its natural size.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodePlacement {
    pub x: f32,
    pub y: f32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub size: Option<[f32; 2]>,
}

impl Layout {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
            && self.edges.is_empty()
            && self.hidden.is_empty()
            && self.collapsed.is_empty()
            && self.opened.is_empty()
    }

    /// The Orphaned Layout Entries: elements the Layout mentions that the
    /// Model no longer has, in order.
    pub fn orphaned(&self, model: &Model) -> Vec<&QualifiedName> {
        let mentioned: BTreeSet<&QualifiedName> =
            self.nodes.keys().chain(&self.hidden).chain(&self.collapsed).collect();
        mentioned.into_iter().filter(|e| model.element(e).is_none()).collect()
    }
}

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
