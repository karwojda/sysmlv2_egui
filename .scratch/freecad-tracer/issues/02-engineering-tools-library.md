# 02: EngineeringTools library and `sysml-viewer library`

**What to build:** A systems engineer can get the `EngineeringTools.sysml` library from the tool itself (`sysml-viewer library` prints it) and copy it into their Model, so that `@ToolLink { tool; file; object; }` and `@ToolProperty { name; }` resolve here and in any other SysML v2 tool. The library defines `metadata def ToolLink { attribute tool : String; attribute file : String; attribute object : String; }` and `metadata def ToolProperty { attribute name : String; attribute tool : String[0..1]; }` with the doc comments from the prototype (branch `prototype/live-freecad`). The tool never writes the file.

See `.scratch/freecad-tracer/spec.md` (stories 1, 2) and ADR 0004.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The library text is embedded in the tool, with the prototype's doc comments
- [ ] `sysml-viewer library` prints it to stdout and exits 0
- [ ] A fixture Model that includes the library and uses `@ToolLink` / `@ToolProperty` parses through OpenSysML without errors
- [ ] CLI test covers `library` output (prior art: `viewer-cli/tests/check.rs`)
