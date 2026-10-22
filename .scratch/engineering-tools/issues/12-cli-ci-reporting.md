# How do the CLI and CI report Design Values and missing ones?

**Status:** resolved
**Type:** grilling
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

A missing Design Value makes the dependent checks Undecided with a reason (ADR 0003). How does the CLI show Design Values, Model Values and those reasons? Does a run with any missing Design Value fail CI, or only a Violated Verdict? Is there an option to run on Model Values only (ignore Tool Links), as ADR 0003 left open? How does CI get FreeCAD (AppImage, Flatpak, container) and set `SYSML_VIEWER_FREECAD`, and what about the material library that `Mass` depends on?

## Answer

**An unreadable Tool Link fails the gate; everything else follows today's CLI.** Resolved by grilling; no ADR. Today `check` and `requirements` fail on violated, and `--strict` also on undecided, not claimed and not run (`crates/viewer-cli/src/lib.rs`, `tests/check.rs`).

1. **A Tool Link that can't be read fails `check` and `requirements` by default**, without `--strict`, with its own error line: `error: 2 Tool Links could not be read`. Reason: an unanswered link is the environment failing to read the design, not a gap in the Model; otherwise a CI machine without FreeCAD turns "violated by the design" into a passing "undecided".
2. **The marker is translated.** Undecided lines read `no Design Value for drive::gearbox::volume (FreeCAD: no object named 'Nope.' in cad/engine.FCStd)`, never the service's `type mismatch: cannot write "no Design Value: …"`.
3. **`--model-values`** on every command that loads a Model: ignore Tool Links, use Model Values, run no tools. It is the estimate-stage run ADR 0003 left open, and the explicit way to run CI without FreeCAD.
4. **A "Design Values" section** printed first by `check` and `requirements` when the Model has Tool Links, one line per linked feature: `drive::engine::volume   1.0e6 mm³   FreeCAD cad/engine.FCStd Engine.Volume   (Model says 0.9e6 mm³)`; failed links listed there with their reason. No change for Models without links.
5. **CI gets FreeCAD by a documented recipe**, not a published container: the FreeCAD AppImage with `--appimage-extract` (no FUSE), `SYSML_VIEWER_FREECAD` pointing at its `freecadcmd`. `Mass` depends on the materials that FreeCAD version ships; custom materials give Undecided, one more reason for the volume × density route.
6. **`export` warns on stderr and still exports** when a Tool Link can't be read. `check` stays the gate.
