//! Engineering Tools: where a Model's Design Values come from (ADR-0003 to
//! ADR-0005). The `EngineeringTool` trait is what a backend asks a tool;
//! `ToolLink` and `DesignValueOutcome` are what the Model records of it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::model::QualifiedName;

/// The `EngineeringTools` library, as `EngineeringTools.sysml`: the
/// `ToolLink` and `ToolProperty` metadata defs users copy into their Model
/// so `@ToolLink { tool; file; object; }` and `@ToolProperty { name; }`
/// resolve here and in any other SysML v2 tool (ADR-0004). This tool never
/// writes the file.
pub const LIBRARY: &str = include_str!("../library/EngineeringTools.sysml");

/// What a load asks of one file: per object, the names of the properties
/// wanted (`{"Engine": {"Volume"}}`).
pub type Wanted = BTreeMap<String, BTreeSet<String>>;

/// What a tool said about one file: per object and property, a value with
/// its unit as SysML writes it (`1000000.0 [mm^3]`) or the reason there is
/// none (`no property Area`).
pub type Answer = BTreeMap<String, BTreeMap<String, Result<String, String>>>;

/// An Engineering Tool as a backend reaches it: an adapter that knows
/// nothing of SysML (ADR-0005).
pub trait EngineeringTool: Send {
    /// Reads the properties `wanted` of the objects in `file`, an absolute
    /// path. `Err` is a reason for the whole file (the tool isn't
    /// installed, the file is missing, the tool timed out) and applies to
    /// every property asked of it.
    fn read(&mut self, file: &Path, wanted: &Wanted) -> Result<Answer, String>;
}

/// A Tool Link: the Model says `element` is designed as `object` in
/// `file`, owned by Engineering Tool `tool`
/// (`@ToolLink { tool = "FreeCAD"; file = "cad/engine.FCStd"; object = "Engine"; }`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolLink {
    /// The linked usage, `Drive::drive::engine`.
    pub element: QualifiedName,
    /// The tool's name, `FreeCAD`.
    pub tool: String,
    /// The file as the Model writes it, relative to the `.sysml` file
    /// holding the link: `cad/engine.FCStd`.
    pub file: String,
    /// The object inside the file, `Engine`.
    pub object: String,
}

/// Where a feature's Design Value comes from: `property` of `object` in
/// `file`, read by `tool` (`FreeCAD`, `cad/engine.FCStd`, `Engine`,
/// `Volume`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesignValueSource {
    pub tool: String,
    /// As the Tool Link writes it.
    pub file: String,
    pub object: String,
    pub property: String,
}

/// What a linked feature's Design Value came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesignValue {
    /// The tool's value with its unit, as SysML writes it:
    /// `1000000.0 [mm^3]`. The checks used it in place of the Model Value.
    Read { value: String },
    /// There is no Design Value, and why: `FreeCAD not available`.
    Missing { reason: String },
}

/// One feature with a `@ToolProperty`: its Design Value, where that comes
/// from, and the Model Value it replaced in the checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesignValueOutcome {
    /// The feature, `Drive::drive::engine::volume`.
    pub feature: QualifiedName,
    pub source: DesignValueSource,
    pub design_value: DesignValue,
    /// The value the Model writes for the feature, as written
    /// (`0.9e6 [mm^3]`), if it writes one. A feature that writes none has
    /// the one it inherits through what it redefines, if any
    /// (`default 0.2e6 [mm^3]` from its def).
    pub model_value: Option<String>,
}
