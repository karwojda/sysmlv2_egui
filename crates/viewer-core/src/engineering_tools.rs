//! Engineering Tools: where a Model's Design Values come from (ADR-0003 to
//! ADR-0005).

/// The `EngineeringTools` library, as `EngineeringTools.sysml`: the
/// `ToolLink` and `ToolProperty` metadata defs users copy into their Model
/// so `@ToolLink { tool; file; object; }` and `@ToolProperty { name; }`
/// resolve here and in any other SysML v2 tool (ADR-0004). This tool never
/// writes the file.
pub const LIBRARY: &str = include_str!("../library/EngineeringTools.sysml");
