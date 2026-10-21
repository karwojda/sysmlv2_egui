//! Engineering Tool adapters: each reaches one tool's files and answers
//! the `EngineeringTool` trait of `viewer-core` (ADR-0005). They know
//! nothing of SysML.

pub mod freecad;

pub use freecad::{FreeCad, Machine};
