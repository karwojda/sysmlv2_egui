//! The Model backend interface (ADR-0001) and the fake backend used in tests.

use std::path::{Path, PathBuf};

use crate::model::{Model, QualifiedName};

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("model folder not found: {0}")]
    FolderNotFound(PathBuf),
    #[error("model backend unavailable: {0}")]
    Unavailable(String),
}

/// Which analysis usages a load runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Analyses {
    /// As the Model's settings say (`run_analyses`).
    #[default]
    AsSet,
    All,
    None,
    /// Just this one, as "Run now" asks.
    Only(QualifiedName),
}

/// Loads a Model from a folder of SysML v2 files. Parse and resolution
/// problems are reported as Model diagnostics; `Err` means the backend
/// itself could not do its job.
pub trait ModelBackend {
    /// Loads with analyses as the Model's settings say.
    fn load(&mut self, folder: &Path) -> Result<Model, BackendError> {
        self.load_with(folder, &Analyses::AsSet)
    }

    /// Loads, running the analyses `analyses` asks for.
    fn load_with(&mut self, folder: &Path, analyses: &Analyses) -> Result<Model, BackendError>;
}

/// Serves a fixed Model regardless of the folder asked for.
pub struct FakeBackend {
    model: Model,
}

impl FakeBackend {
    pub fn new(model: Model) -> Self {
        Self { model }
    }
}

impl ModelBackend for FakeBackend {
    fn load_with(&mut self, _folder: &Path, _analyses: &Analyses) -> Result<Model, BackendError> {
        Ok(self.model.clone())
    }
}
