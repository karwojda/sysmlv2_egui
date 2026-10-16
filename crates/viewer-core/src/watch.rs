//! Which files make up a Model, and noticing when they change, by polling
//! (file-change notifications don't reach every filesystem, such as Windows
//! drives under WSL).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use crate::model::{Diagnostic, Model};

const MODEL_EXTENSIONS: [&str; 2] = ["sysml", "kerml"];

/// Every SysML and KerML file under `folder`, recursively, in a stable
/// order. Diagram Files and anything else are not part of the Model.
pub fn model_files(folder: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![folder.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| MODEL_EXTENSIONS.contains(&e))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// The Model files with their modification time and size: when it differs,
/// the Model has changed on disk.
pub type Fingerprint = BTreeMap<PathBuf, (Option<SystemTime>, u64)>;

pub fn fingerprint(folder: &Path) -> std::io::Result<Fingerprint> {
    let mut fingerprint = Fingerprint::new();
    // The Model Style restyles every Diagram, so editing it counts too.
    for file in model_files(folder)?
        .into_iter()
        .chain([crate::style::path_for(folder)])
    {
        // A file deleted between listing and reading simply drops out.
        if let Ok(metadata) = std::fs::metadata(&file) {
            fingerprint.insert(file, (metadata.modified().ok(), metadata.len()));
        }
    }
    Ok(fingerprint)
}

/// Decides when to reload from successive fingerprints: once a change has
/// held for one more observation, so a burst of saves gives one reload.
#[derive(Default)]
pub struct ChangeDetector {
    loaded: Option<Fingerprint>,
    seen: Option<Fingerprint>,
}

impl ChangeDetector {
    /// Starts from the fingerprint of the Model as loaded.
    pub fn new(loaded: Fingerprint) -> Self {
        Self {
            loaded: Some(loaded.clone()),
            seen: Some(loaded),
        }
    }

    /// Records the latest fingerprint; true means "reload now".
    pub fn observe(&mut self, current: Fingerprint) -> bool {
        let settled = self.seen.as_ref() == Some(&current);
        let changed = self.loaded.as_ref() != Some(&current);
        self.seen = Some(current.clone());
        if settled && changed {
            self.loaded = Some(current);
            return true;
        }
        false
    }
}

/// What the viewer shows after a load: the newly loaded Model, unless it
/// has errors and an earlier good one exists, in which case the earlier one
/// keeps being drawn (`stale`) while the new diagnostics are listed.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub model: Model,
    pub diagnostics: Vec<Diagnostic>,
    pub stale: bool,
}

pub fn accept(previous: Option<Shown>, loaded: Model) -> Shown {
    let diagnostics = loaded.diagnostics().to_vec();
    match previous {
        // The Model drawn is good, or is itself the last good one kept.
        Some(previous)
            if loaded.has_errors() && (previous.stale || !previous.model.has_errors()) =>
        {
            Shown {
                model: previous.model,
                diagnostics,
                stale: true,
            }
        }
        _ => Shown {
            model: loaded,
            diagnostics,
            stale: false,
        },
    }
}
