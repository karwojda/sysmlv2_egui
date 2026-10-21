//! PROTOTYPE, throwaway: "How do Design Values appear in the viewer?"
//! Three variants, switched at run time (PROTOTYPE_VARIANT=A|B|C, or the
//! floating bar in the app). Design Values are stubbed for
//! `prototypes/design-values-ui/model`; no tool runs.

use std::sync::atomic::{AtomicU8, Ordering};

use crate::model::QualifiedName;

pub const VARIANTS: [(&str, &str); 3] = [
    ("A", "Inline: values in place, Model Value beside"),
    ("B", "Diagram compartment + Properties section"),
    ("C", "Design Values table"),
];

static VARIANT: AtomicU8 = AtomicU8::new(u8::MAX);

pub fn variant() -> usize {
    let v = VARIANT.load(Ordering::Relaxed);
    if v != u8::MAX {
        return v as usize;
    }
    let from_env = match std::env::var("PROTOTYPE_VARIANT").as_deref() {
        Ok("B") => 1,
        Ok("C") => 2,
        _ => 0,
    };
    VARIANT.store(from_env, Ordering::Relaxed);
    from_env as usize
}

pub fn set_variant(v: usize) {
    VARIANT.store((v % VARIANTS.len()) as u8, Ordering::Relaxed);
}

pub struct ToolLink {
    pub element: &'static str,
    pub tool: &'static str,
    pub file: &'static str,
    pub object: &'static str,
}

pub struct DesignValue {
    pub feature: &'static str,
    pub property: &'static str,
    /// `Err(reason)` when there is no Design Value.
    pub value: Result<&'static str, &'static str>,
    pub model_value: &'static str,
}

pub const LINKS: [ToolLink; 2] = [
    ToolLink {
        element: "Drive::drive::engine",
        tool: "FreeCAD",
        file: "cad/engine.FCStd",
        object: "Engine",
    },
    ToolLink {
        element: "Drive::drive::gearbox",
        tool: "FreeCAD",
        file: "cad/engine.FCStd",
        object: "Assembly.Link.",
    },
];

pub const VALUES: [DesignValue; 2] = [
    DesignValue {
        feature: "Drive::drive::engine::volume",
        property: "Volume",
        value: Ok("1000000.0 [mm³]"),
        model_value: "900000.0 [mm³]",
    },
    DesignValue {
        feature: "Drive::drive::gearbox::volume",
        property: "Volume",
        value: Err("no object named 'Assembly.Link.' in cad/engine.FCStd"),
        model_value: "default 200000.0 [mm³] (from Gearbox)",
    },
];

/// Verdicts that rest on Design Values, and the stubbed outcome with them.
pub const RESTING: [(&str, &str); 2] = [
    (
        "Drive::driveMass",
        "? Undecided: no Design Value for drive::gearbox::volume",
    ),
    (
        "Drive::Engine::engineMass",
        "✗ Violated on engine: mass = 7.85 [kg] (from Design Values)",
    ),
];

pub fn link(element: &QualifiedName) -> Option<&'static ToolLink> {
    LINKS.iter().find(|l| l.element == element.to_string())
}

pub fn value(feature: &QualifiedName) -> Option<&'static DesignValue> {
    VALUES.iter().find(|v| v.feature == feature.to_string())
}

/// Design Values of features owned by `element`.
pub fn values_of(element: &QualifiedName) -> impl Iterator<Item = &'static DesignValue> {
    let prefix = format!("{element}::");
    VALUES.iter().filter(move |v| {
        v.feature
            .strip_prefix(&prefix)
            .is_some_and(|rest| !rest.contains("::"))
    })
}

pub fn short(feature: &str) -> &str {
    feature.strip_prefix("Drive::").unwrap_or(feature)
}
