# 02: EngineeringTools library and `sysml-viewer library`

**What to build:** A systems engineer can get the `EngineeringTools.sysml` library from the tool itself (`sysml-viewer library` prints it) and copy it into their Model, so that `@ToolLink { tool; file; object; }` and `@ToolProperty { name; }` resolve here and in any other SysML v2 tool. The library defines `metadata def ToolLink { attribute tool : String; attribute file : String; attribute object : String; }` and `metadata def ToolProperty { attribute name : String; attribute tool : String[0..1]; }` with the doc comments from the prototype (branch `prototype/live-freecad`). The tool never writes the file.

See `.scratch/freecad-tracer/spec.md` (stories 1, 2) and ADR 0004.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] The library text is embedded in the tool, with the prototype's doc comments
- [x] `sysml-viewer library` prints it to stdout and exits 0
- [x] A fixture Model that includes the library and uses `@ToolLink` / `@ToolProperty` parses through OpenSysML without errors
- [x] CLI test covers `library` output (prior art: `viewer-cli/tests/check.rs`)

## Comments

- Done.
  - **Library:** `crates/viewer-core/library/EngineeringTools.sysml`, embedded as `viewer_core::engineering_tools::LIBRARY` (`include_str!`). Same text as the prototype, with the doc comments, minus its `// PROTOTYPE` header line. The `engineering_tools` module in `viewer-core` is the place for the `EngineeringTool` trait later.
  - **Command:** `sysml-viewer library` prints it to stdout and exits 0. It doesn't load a Model, so the service never starts.
  - **Fixture:** `crates/viewer-opensysml/tests/fixtures/tool_links/`, a copy of the library plus the prototype's `drive.sysml` (engine with a value, gearbox with a body and only the def's `default`, the requirement and the asserted constraint). It loads through OpenSysML without diagnostics. Without the library it reports unresolved `EngineeringTools`, `ToolLink` and `ToolProperty`.
  - **Tests:** `viewer-cli/tests/library.rs` (the output's defs and doc comments, exit code, empty stderr). In `viewer-opensysml/tests/contract.rs`: the fixture parses without diagnostics (needs the service), and a test that doesn't need the service checks the fixture's copy is byte-identical to `LIBRARY`, so the two can't drift.
  - **Deviation:** the fixture has no fake-backend twin yet. Ticket 03 adds the Design Value expectations.
