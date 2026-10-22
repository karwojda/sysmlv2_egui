//! Engineering Tool adapters: each reaches one tool's files and answers
//! the `EngineeringTool` trait of `viewer-core` (ADR-0005). They know
//! nothing of SysML.

pub mod freecad;

pub use freecad::{FreeCad, Machine};

use viewer_core::engineering_tools::EngineeringTool;

/// The Engineering Tools built into this tool, each by the name Tool Links
/// give it, as this machine has them: FreeCAD as `"FreeCAD"`. The app and
/// the CLI give them to their backend
/// (`OpenSysmlBackend::with_config(config).with_tools(viewer_tools::built_in())`).
pub fn built_in() -> Vec<(&'static str, Box<dyn EngineeringTool>)> {
    vec![(freecad::NAME, Box::new(FreeCad::new()))]
}
