# 08: GUI: Tool Link, Design Value and Model Value rows

**What to build:** The GUI loads with the FreeCAD adapter on the existing worker thread, so the Model arrives with its Design Values in one load: Verdicts from Model Values never show first and the window never freezes. In Properties, a linked element shows a `Tool Link` row (`⚙ FreeCAD · cad/engine.FCStd · Engine`). A linked feature's `Value` row is replaced by `Design Value` (the value plus `⚙ <tool> <object>.<property>`, or `⚠ none: <reason>`), followed by `Model Value`. The look comes from the UI prototype (branch `prototype/design-values-ui`, variant A).

See `.scratch/freecad-tracer/spec.md` (stories 36–38, 41).

**Blocked by:** 03 (Design Values replace Model Values in load), 06 (FreeCAD adapter)

**Status:** ready-for-agent

- [ ] The app's loader passes the FreeCAD adapter to the backend; loading stays on the worker thread
- [ ] `Tool Link` row on a linked element
- [ ] `Design Value` then `Model Value` rows replace `Value` on a linked feature
- [ ] `⚠ none: <reason>` when there is no Design Value
- [ ] Unlinked elements' Properties unchanged
- [ ] Tests build a `Model` in the test and assert the rows (prior art: `viewer-core/tests/properties.rs`)
