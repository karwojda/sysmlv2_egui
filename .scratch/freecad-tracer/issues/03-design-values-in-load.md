# 03: Design Values replace Model Values in load (tracer, fake tool)

**What to build:** The thinnest end-to-end path. A Model annotates a part usage with `@ToolLink` and a valued feature (`attribute volume = 0.9e6 [mm^3] { @ToolProperty { name = "Volume"; } }`) with `@ToolProperty`. On load, the real OpenSysML backend reads the Tool Links, asks the named Engineering Tool once per file for the wanted properties, uses the answer as the feature's Design Value in place of its Model Value, and runs every check on the result. With a fake tool answering `1.0e6 [mm^3]`, a requirement and an asserted constraint that held on the Model Values are now violated, and roll-ups (`mass = volume * density`, `drive.mass = engine.mass + gearbox.mass`) recompute. The `.sysml` files are never touched.

This ticket introduces the `EngineeringTool` trait in the domain (file in, per object the wanted property names; out, per object and property a value text with unit or a reason; a whole-file error applies to every property asked of it) and the Model's record of Tool Links and Design Value outcomes (Design Value text and source, or a reason, plus the Model Value as written). These carry no OpenSysML types (ADR 0001). The backend is given its tools by name (`"FreeCAD"`).

Scope here: features that already have a value. Body-without-value and inherited `default` shapes are ticket 04; failures and diagnostics are ticket 05.

See `.scratch/freecad-tracer/spec.md` (stories 3, 4, 7, 10–13, 15, 27, 28), `docs/engineering-tools.md`, ADRs 0003–0005, and the prototype pipeline on branch `prototype/live-freecad`.

**Blocked by:** 02 (EngineeringTools library and `sysml-viewer library`)

**Status:** done

- [x] `EngineeringTool` trait and Model outcome types in `viewer-core`, free of OpenSysML types
- [x] Tool Links and `@ToolProperty` read from the api-json form (fields by `<metadata id>__<field>_pvalue`)
- [x] `file` is resolved relative to the `.sysml` file holding the link before the tool is called
- [x] One tool call per tool and file, however many links point at it
- [x] Fixture Model folder (library + drive Model from the prototype) loaded through the real backend with a fake tool: requirement and asserted constraint flip to violated; roll-ups recompute
- [x] Only the linked usage changes; its def and sibling usages keep their values
- [x] Design Value is written with its unit and OpenSysML converts it to the feature's unit
- [x] The Model records each Tool Link and each feature's Design Value outcome with its Model Value
- [x] `.sysml` files are byte-for-byte unchanged after load
- [x] A Model without Tool Links never calls the tool and takes the existing load path

## Comments

- Done.
  - **Domain (`viewer_core::engineering_tools`):** trait `EngineeringTool::read(&mut self, file: &Path, wanted: &Wanted) -> Result<Answer, String>` (`Wanted` = object to property names, `Answer` = object to property to `Result<value text, reason>`, `Err` = whole-file reason). The Model records `ToolLink { element, tool, file, object }` and `DesignValueOutcome { feature, source: DesignValueSource { tool, file, object, property }, design_value: DesignValue::Read { value } | DesignValue::Missing { reason }, model_value }`. Read them with `Model::tool_links`, `tool_link`, `design_values`, `design_value`; set them with `Model::with_design_values`. `ModelBuilder::tool_link` and `design_value` build them for tests, and `FakeBackend` serves them. No OpenSysML types.
  - **Backend:** `OpenSysmlBackend::with_tool("FreeCAD", tool)`. `api_json::enrich` also returns the Tool Links. It reads them from the conversion it already makes, so a Model without links costs nothing extra and takes the old path. `design_values::plan` (no service types) groups by tool and absolute file and calls each tool once. It builds the outcomes and the `set_value` edits. `lib.rs` makes one editor per document, re-parses the edited documents inline with the rest from disk, and runs the checks on that Model. Elements, diagnostics and source lines come from the unedited parse, so `value` stays the Model Value and Computed Values show the Design Value (`1000000.0 [mm³]`).
  - **Fixture:** a new `design_values/` folder rather than `tool_links/`, because `tool_links/` has the gearbox with a body and no value (ticket 04). The drive Model sits in `drive/` to test relative `file`. The gearbox is written in m³ to test unit conversion. A sibling `spare : Engine` has no link. Tests are in `viewer-opensysml/tests/design_values.rs` (7, service-backed) and `viewer-core/tests/design_values.rs` (fake backend).
  - **Deviation, no fake-backend twin:** the outcomes are asserted directly instead. A full twin would mostly restate the Computed Values.
  - **Left for later:** features without a value of their own are skipped entirely: no tool call, no outcome (ticket 04). A missing answer, an unknown tool or a whole-file error is recorded as `DesignValue::Missing`, but nothing is written yet, so those checks still run on the Model Value (ticket 05 writes the marker). An edit or re-parse failure is one Error diagnostic, and the checks run on the unedited Model. The outcome's `value` is the tool's text (`1000000.0 [mm^3]`) and `model_value` is the element's value text (`900000.0 [millimetre ** 3]`). The CLI and GUI format these for display.
