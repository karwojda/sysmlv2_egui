//! Engineering Tools: where a Model's Design Values come from (ADR-0003 to
//! ADR-0005). The `EngineeringTool` trait is what a backend asks a tool;
//! `ToolLink` and `DesignValueOutcome` are what the Model records of it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

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

/// What a tool said about one file: per object and property, the value it
/// read or the reason there is none (`no property Area`).
pub type Answer = BTreeMap<String, BTreeMap<String, Result<ReadValue, String>>>;

/// A value a tool read: its text with its unit as SysML writes it
/// (`1000000.0 [mm^3]`), and, for a Summed Design Value, the parts it was
/// added up over.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReadValue {
    /// The value, `0.412 [W]`.
    pub value: String,
    /// The parts counted and left uncounted, when the value is a sum
    /// (ADR-0006); `None` for a single value such as FreeCAD's volume.
    pub parts: Option<Parts>,
}

impl ReadValue {
    /// A single value, not added up over parts: `1000000.0 [mm^3]`.
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            parts: None,
        }
    }

    /// A Summed Design Value: `value` added up over `parts`.
    pub fn summed(value: impl Into<String>, parts: Parts) -> Self {
        Self {
            value: value.into(),
            parts: Some(parts),
        }
    }
}

/// The parts a Summed Design Value was added up over, by reference
/// designator: those counted, and those left out because they state no
/// value (ADR-0006). Each list is kept in natural order, `C2` before `C10`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parts {
    counted: Vec<String>,
    uncounted: Vec<String>,
}

impl Parts {
    /// The parts `counted` and `uncounted`, in any order.
    pub fn new(
        counted: impl IntoIterator<Item = impl Into<String>>,
        uncounted: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let sorted = |mut refs: Vec<String>| {
            refs.sort_by(|a, b| natural_order(a, b));
            refs
        };
        Self {
            counted: sorted(counted.into_iter().map(Into::into).collect()),
            uncounted: sorted(uncounted.into_iter().map(Into::into).collect()),
        }
    }

    /// The parts counted, in natural order: `D1`, `U1`, `U2`.
    pub fn counted(&self) -> &[String] {
        &self.counted
    }

    /// The parts left uncounted, in natural order: `C2`, `C10`, `R1`.
    pub fn uncounted(&self) -> &[String] {
        &self.uncounted
    }

    /// How many were counted of all the parts: `summed 9 of 71 parts`.
    pub fn summary(&self) -> String {
        format!(
            "summed {} of {} parts",
            self.counted.len(),
            self.counted.len() + self.uncounted.len()
        )
    }
}

/// `a` before `b` with runs of digits compared as numbers: `C2` before
/// `C10`, `R1` after both.
pub fn natural_order(a: &str, b: &str) -> std::cmp::Ordering {
    /// Splits off the leading run of digits or of other characters.
    fn chunk(s: &str) -> (&str, &str) {
        let digits = s.starts_with(|c: char| c.is_ascii_digit());
        let end = s
            .find(|c: char| c.is_ascii_digit() != digits)
            .unwrap_or(s.len());
        s.split_at(end)
    }
    let (mut a, mut b) = (a, b);
    while !a.is_empty() && !b.is_empty() {
        let ((x, rest_a), (y, rest_b)) = (chunk(a), chunk(b));
        let numbers = x.starts_with(|c: char| c.is_ascii_digit())
            && y.starts_with(|c: char| c.is_ascii_digit());
        let order = if numbers {
            let (x, y) = (x.trim_start_matches('0'), y.trim_start_matches('0'));
            x.len().cmp(&y.len()).then_with(|| x.cmp(y))
        } else {
            x.cmp(y)
        };
        if order.is_ne() {
            return order;
        }
        (a, b) = (rest_a, rest_b);
    }
    a.len().cmp(&b.len())
}

/// An Engineering Tool as a backend reaches it: an adapter that knows
/// nothing of SysML (ADR-0005).
pub trait EngineeringTool: Send {
    /// Reads the properties `wanted` of the objects in `file`, an absolute
    /// path. `Err` is a reason for the whole file (the tool isn't
    /// installed, the file is missing, the tool timed out) and applies to
    /// every property asked of it.
    fn read(&mut self, file: &Path, wanted: &Wanted) -> Result<Answer, String>;

    /// The files an answer for `file` depends on: a change to any of them
    /// may change it. By default `file` itself; a KiCad project
    /// (`board.kicad_pro`) depends on every schematic and board file in
    /// its folder instead. Nothing asks this until linked files are
    /// watched.
    fn dependent_files(&self, file: &Path) -> Vec<PathBuf> {
        vec![file.to_path_buf()]
    }
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

/// Where a feature's Design Value comes from: `property` of the object
/// its owner's Tool Link names (`Volume` of `Engine` in
/// `cad/engine.FCStd`, read by `FreeCAD`).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesignValueSource {
    /// The Tool Link on the feature's owner: tool, file and object.
    pub link: ToolLink,
    /// The tool's property the value is, `Volume`.
    pub property: String,
}

/// The mark of a Tool Link and of a Design Value read through one: `⚙`.
pub const TOOL_MARK: &str = "⚙";

impl ToolLink {
    /// The link as one line: `⚙ FreeCAD · cad/engine.FCStd · Engine`.
    pub fn summary(&self) -> String {
        format!(
            "{TOOL_MARK} {} · {} · {}",
            self.tool, self.file, self.object
        )
    }
}

impl DesignValue {
    /// The mark it is shown with: `⚙` read, `⚠` missing.
    pub fn mark(&self) -> &'static str {
        match self {
            DesignValue::Read { .. } => TOOL_MARK,
            DesignValue::Missing { .. } => "⚠",
        }
    }

    /// What became of it, in a word or two: `Read`, `Not read`.
    pub fn label(&self) -> &'static str {
        match self {
            DesignValue::Read { .. } => "Read",
            DesignValue::Missing { .. } => "Not read",
        }
    }
}

/// What a linked feature's Design Value came to.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesignValue {
    /// The tool's value with its unit, as SysML writes it:
    /// `1000000.0 [mm^3]`. The checks used it in place of the Model Value.
    Read {
        /// The value, `1000000.0 [mm^3]`.
        value: String,
        /// For a Summed Design Value, the parts counted and uncounted;
        /// `None` for a single value.
        parts: Option<Parts>,
    },
    /// There is no Design Value, and why: `FreeCAD not available`.
    Missing {
        /// Why, `FreeCAD not available`.
        reason: String,
    },
}

/// One feature with a `@ToolProperty`: its Design Value, where that comes
/// from, and the Model Value it replaced in the checks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DesignValueOutcome {
    /// The feature, `Drive::drive::engine::volume`.
    pub feature: QualifiedName,
    /// Where its Design Value comes from.
    pub source: DesignValueSource,
    /// The Design Value, or why there is none.
    pub design_value: DesignValue,
    /// The value the Model writes for the feature, as written
    /// (`0.9e6 [mm^3]`), if it writes one. A feature that writes none has
    /// the one it inherits through what it redefines, if any
    /// (`default 0.2e6 [mm^3]` from its def).
    pub model_value: Option<String>,
}
