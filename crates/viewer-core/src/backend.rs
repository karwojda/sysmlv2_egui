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

    /// What loaded the last Model, as the user is told it: `OpenSysML
    /// sysml-grpc v0.11.0`. `None` before a load, or from a backend with
    /// nothing to name.
    fn version(&self) -> Option<String> {
        None
    }
}

/// Serves a fixed Model regardless of the folder asked for. Its Analysis
/// Results count as the Model's setting; analyses a load doesn't run lose
/// theirs, as on a real load.
pub struct FakeBackend {
    model: Model,
    version: Option<String>,
}

impl FakeBackend {
    pub fn new(model: Model) -> Self {
        Self {
            model,
            version: None,
        }
    }

    /// Names itself `version` after every load.
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = Some(version.into());
        self
    }
}

impl ModelBackend for FakeBackend {
    fn load_with(&mut self, _folder: &Path, analyses: &Analyses) -> Result<Model, BackendError> {
        let runs = |name: &QualifiedName| match analyses {
            Analyses::AsSet | Analyses::All => true,
            Analyses::None => false,
            Analyses::Only(only) => only == name,
        };
        let elements = self.model.elements().cloned().map(|mut e| {
            if !runs(&e.qualified_name) {
                e.analysis_result = None;
            }
            e
        });
        Ok(
            Model::new(elements, self.model.diagnostics().to_vec()).with_design_values(
                self.model.tool_links().to_vec(),
                self.model.design_values().to_vec(),
            ),
        )
    }

    fn version(&self) -> Option<String> {
        self.version.clone()
    }
}
