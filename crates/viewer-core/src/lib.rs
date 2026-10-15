//! Core of the SysML v2 viewer: the Model, Diagram building, layout and the
//! renderer-neutral scene, plus the SVG renderer used for Export.
//!
//! Nothing in this crate knows about egui, the command line, or any SysML
//! parsing library (ADR-0001).

pub mod backend;
pub mod diagram;
pub mod diagram_file;
mod layout;
pub mod model;
mod notation;
pub mod scene;
pub mod svg;

pub use backend::{BackendError, FakeBackend, ModelBackend};
pub use diagram::{DiagramError, DiagramKind, build_scene};
pub use diagram_file::{DiagramFile, DiagramFileError};
pub use model::{Diagnostic, Element, ElementKind, Model, QualifiedName, Severity, SourceLocation};
pub use scene::Scene;
