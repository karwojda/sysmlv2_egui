//! Core of the SysML v2 viewer: the Model, Diagram building, layout and the
//! renderer-neutral scene, plus the SVG renderer used for Export.
//!
//! Nothing in this crate knows about egui, the command line, or any SysML
//! parsing library (ADR-0001).

pub mod analysis;
pub mod backend;
pub mod browser;
pub mod constraint;
pub mod diagram;
pub mod diagram_file;
pub mod editing;
mod interconnection;
mod layout;
pub mod model;
mod notation;
pub mod properties;
pub mod satisfaction;
pub mod scene;
pub mod settings;
pub mod style;
pub mod svg;
pub mod view;
pub mod watch;

pub use backend::{BackendError, FakeBackend, ModelBackend};
pub use diagram::{
    DiagramError, DiagramKind, DiagramSource, build_general_scene, build_scene,
    build_scene_with_layout, layout_key_exists,
};
pub use diagram_file::{
    DiagramFile, DiagramFileError, DiagramStyle, Layout, NodePlacement, PortPlacement, PortSide,
};
pub use model::{Diagnostic, Element, ElementKind, Model, QualifiedName, Severity, SourceLocation};
pub use scene::Scene;
