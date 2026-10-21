# 03: Design Values replace Model Values in load (tracer, fake tool)

**What to build:** The thinnest end-to-end path. A Model annotates a part usage with `@ToolLink` and a valued feature (`attribute volume = 0.9e6 [mm^3] { @ToolProperty { name = "Volume"; } }`) with `@ToolProperty`. On load, the real OpenSysML backend reads the Tool Links, asks the named Engineering Tool once per file for the wanted properties, uses the answer as the feature's Design Value in place of its Model Value, and runs every check on the result. With a fake tool answering `1.0e6 [mm^3]`, a requirement and an asserted constraint that held on the Model Values are now violated, and roll-ups (`mass = volume * density`, `drive.mass = engine.mass + gearbox.mass`) recompute. The `.sysml` files are never touched.

This ticket introduces the `EngineeringTool` trait in the domain (file in, per object the wanted property names; out, per object and property a value text with unit or a reason; a whole-file error applies to every property asked of it) and the Model's record of Tool Links and Design Value outcomes (Design Value text and source, or a reason, plus the Model Value as written). These carry no OpenSysML types (ADR 0001). The backend is given its tools by name (`"FreeCAD"`).

Scope here: features that already have a value. Body-without-value and inherited `default` shapes are ticket 04; failures and diagnostics are ticket 05.

See `.scratch/freecad-tracer/spec.md` (stories 3, 4, 7, 10–13, 15, 27, 28), `docs/engineering-tools.md`, ADRs 0003–0005, and the prototype pipeline on branch `prototype/live-freecad`.

**Blocked by:** 02 (EngineeringTools library and `sysml-viewer library`)

**Status:** ready-for-agent

- [ ] `EngineeringTool` trait and Model outcome types in `viewer-core`, free of OpenSysML types
- [ ] Tool Links and `@ToolProperty` read from the api-json form (fields by `<metadata id>__<field>_pvalue`)
- [ ] `file` is resolved relative to the `.sysml` file holding the link before the tool is called
- [ ] One tool call per tool and file, however many links point at it
- [ ] Fixture Model folder (library + drive Model from the prototype) loaded through the real backend with a fake tool: requirement and asserted constraint flip to violated; roll-ups recompute
- [ ] Only the linked usage changes; its def and sibling usages keep their values
- [ ] Design Value is written with its unit and OpenSysML converts it to the feature's unit
- [ ] The Model records each Tool Link and each feature's Design Value outcome with its Model Value
- [ ] `.sysml` files are byte-for-byte unchanged after load
- [ ] A Model without Tool Links never calls the tool and takes the existing load path
