# 08: GUI: Tool Link, Design Value and Model Value rows

**What to build:** The GUI loads with the FreeCAD adapter on the existing worker thread, so the Model arrives with its Design Values in one load: Verdicts from Model Values never show first and the window never freezes. In Properties, a linked element shows a `Tool Link` row (`⚙ FreeCAD · cad/engine.FCStd · Engine`). A linked feature's `Value` row is replaced by `Design Value` (the value plus `⚙ <tool> <object>.<property>`, or `⚠ none: <reason>`), followed by `Model Value`. The look comes from the UI prototype (branch `prototype/design-values-ui`, variant A).

See `.scratch/freecad-tracer/spec.md` (stories 36–38, 41).

**Blocked by:** 03 (Design Values replace Model Values in load), 06 (FreeCAD adapter)

**Status:** done

- [x] The app's loader passes the FreeCAD adapter to the backend; loading stays on the worker thread
- [x] `Tool Link` row on a linked element
- [x] `Design Value` then `Model Value` rows replace `Value` on a linked feature
- [x] `⚠ none: <reason>` when there is no Design Value
- [x] Unlinked elements' Properties unchanged
- [x] Tests build a `Model` in the test and assert the rows (prior art: `viewer-core/tests/properties.rs`)

## Comments

- Done.
  - **Loader:** `viewer_app::backend(config)` is `OpenSysmlBackend::with_config(config).with_tool("FreeCAD", FreeCad::new())`. `main.rs` and the `screenshot` example both use it. The existing `Loader` owns it on its worker thread, so FreeCAD runs there and a load arrives with its Design Values in one piece. `viewer-app` now depends on `viewer-tools`.
  - **Rows** (`viewer-core/src/properties.rs`): `Tool Link` = `⚙ <tool> · <file> · <object>`. On a feature with a Design Value outcome, `Value` is replaced by `Design Value` (`<value>  ⚙ <tool> <object>.<property>`, or `⚠ none: <reason>`) and then `Model Value` (`none` when the feature writes no value). Units are made readable for both rows (`[mm^3]` and `[millimetre ** 3]` become `[mm³]`) by a small `readable` helper. Unlinked elements keep their plain `Value`, unchanged.
  - **Tests:** `viewer-core/tests/properties.rs` has 4 new tests: the Tool Link row; Design then Model Value; `⚠ none` with an inherited `default` Model Value and with no Model Value; and unlinked elements left unchanged.
  - **Screenshot aid:** `ViewerApp::select(element)` selects an element once the Model loads. The `screenshot` example reads `SCREENSHOT_SELECT=<qualified name>`. Checked against real FreeCAD with the `design_values` fixture, the `EngineeringTools` library and `parts.FCStd` (object `Body`): `Design Value 1000000.0 [mm³] ⚙ FreeCAD Body.Volume`, `Model Value 900000.0 [mm³]`, and the part's `Tool Link` row.
  - **Note:** the existing `Computed` row still shows after `Model Value` on a computed feature. It was left as is.
