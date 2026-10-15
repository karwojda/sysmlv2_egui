//! The Model backend interface (ADR-0001) and the fake backend used in tests.

use std::path::{Path, PathBuf};

use crate::model::Model;

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("model folder not found: {0}")]
    FolderNotFound(PathBuf),
    #[error("model backend unavailable: {0}")]
    Unavailable(String),
}

/// Loads a Model from a folder of SysML v2 files. Parse and resolution
/// problems are reported as Model diagnostics; `Err` means the backend
/// itself could not do its job.
pub trait ModelBackend {
    fn load(&mut self, folder: &Path) -> Result<Model, BackendError>;
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
    fn load(&mut self, _folder: &Path) -> Result<Model, BackendError> {
        Ok(self.model.clone())
    }
}
