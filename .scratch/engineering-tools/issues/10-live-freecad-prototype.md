# Does a live FreeCAD query work from this tool?

**Status:** resolved
**Type:** prototype
**Blocked by:** 01, 07, 09
**Map:** [Engineering Tools](../map.md)

## Question

Throwaway end to end: a Tool Link in a small Model → this tool calls FreeCAD headless → the Design Value reaches a check and a Verdict changes. Is it fast and reliable enough to run on load? What breaks? Include a linked usage whose Model Value is only inherited from its def, so the in-memory edit has to add a redefinition (see the answer to "Does an Engineering Tool's value replace or challenge the Model's?"). Write the Tool Link as decided in "What does a Tool Link look like?". FreeCAD is the Windows install from "Make room for FreeCAD". Two viewer gaps the units probe found: `value_text` in `crates/viewer-opensysml/src/checks/service.rs` shows nothing for `Rational` magnitudes (a derived 7.85 kg mass comes back as `157/20`), and prints units raw (`mm**3`, `SI::'kg⋅m⁻³'`).

## Answer

**Yes: a Tool Link in the Model reaches FreeCAD and changes a Verdict, in about 1 s on load.** Prototype on local branch `prototype/live-freecad` (66cc813, folder `prototypes/live-freecad/`; not pushed: no GitHub credentials). The verdicts below were agreed with the user.

- **End to end:** two Tool Links (a plain body, and a body reached through an assembly `App::Link`) are read from the api-json form, one `freecadcmd.exe` run answers both, the Design Values go in by an in-memory edit, the Model is re-parsed, and `driveMass` and `engineMass` flip from holds (7.605 kg) to violated (8.529 kg).
- **Timing:** FreeCAD 0.59–0.75 s per file (startup included); reading links from api-json ~110 ms; edit ~55 ms; re-parse ~45 ms; checks ~25 ms. Fast enough for on load, provided all files go through **one FreeCAD process** and the query runs **off the UI thread**, with checks showing nothing until Design Values arrive (never Model-Value Verdicts first). *When* queries run is still open.
- **Reading links:** each `MetadataUsage` has `owner` (the linked element) and per field a `<metadata id>__<field>_pvalue` literal. A `:>> volume` redefinition is unnamed (`engine::@1`); its name comes from following `redefines` until a named feature.
- **Writing Design Values:** `set_value` works on a feature that has a value. It **refuses a feature with a body and no value**, and a `@ToolProperty` feature always has a body: the workaround is `delete` + `add_member(redefines = [name], value)`. A request for `set_value` on such declarations is in the OpenSysML issue draft. One editor targets one document (`in_document`), so links in several files need one edit each, and the untouched files are sent again from disk on re-parse.
- **No fallback, enforced:** removing a value is not enough, because a def's `default` shows through a valueless redefinition (the prototype first judged the gearbox on its Model Value this way). Decided: write a string marker, `"no Design Value: <reason>"`. OpenSysML then makes exactly the dependent checks Undecided, with the reason in its error chain (`drive.mass → gearbox.mass → gearbox.volume: type mismatch: cannot write "no Design Value: …"`); independent checks still run.
- **Failures:** FreeCAD missing, `.FCStd` missing and object missing each give a clear reason. An assembly path whose first object doesn't exist gives a raw Python error; the adapter should check it.
- **Inherited Model Value:** a valueless `attribute :>> volume { @ToolProperty … }` shows the def's `default` as its Model Value, and takes a Design Value through the delete + add_member route.
- **Viewer gaps** (from the units probe, confirmed here): fraction-valued results (`Rational`, e.g. 8528584013175395261/10^18 kg) show nothing in `value_text`, and units print raw (`mm**3`, `SI::kg`).
