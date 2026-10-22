# 02: Board envelope from KiCad

**What to build:** A Model links a part to a KiCad project with `@ToolLink { tool = "KiCad"; file = "….kicad_pro"; object = "board"; }`. Its features take `Width`, `Height`, `Thickness` and `Area` as Design Values from the real board. `sysml-viewer check` judges an envelope requirement (width ≤ 60 mm, thickness ≤ 1.6 mm) against them.

The KiCad adapter is registered as `"KiCad"` beside FreeCAD. It knows nothing of SysML.

- **Finding `kicad-cli`:** `SYSML_VIEWER_KICAD` first and final, then `PATH`, then known install locations, newest version first. Those include `/mnt/c/Program Files/KiCad/<version>/bin/kicad-cli.exe` under WSL, with paths converted for Windows. It follows FreeCAD's `Machine` pattern.
- **Version:** `kicad-cli version` must be 10 or newer, or every property gets "KiCad 10 or newer needed; found …".
- **Timeout:** each run is killed after `SYSML_VIEWER_KICAD_TIMEOUT` seconds, 30 by default.
- **Language:** every run gets its own temporary `KICAD_CONFIG_HOME` (passed through `WSLENV` to a Windows executable). It holds `10.0/kicad_common.json` with the language set to English, so a localized install gives the same numbers. The user's own KiCad config is never read or written.
- **Files:** the `.kicad_sch` and `.kicad_pcb` named like the `.kicad_pro`. A missing one is a reason naming it.
- **Objects:** only `board`. Any other object gets a reason saying so.
- **Values:** `pcb export stats --format json --units mm`, keys `board.width`, `board.height`, `board.board_thickness` and `board.area`. Their unit strings become SysML units: `58.42 [mm]`, `605.284 [mm^2]`.
- **Dependent files:** the trait gains a way for an adapter to name the files an answer depends on; by default that is the file itself. KiCad names every `.kicad_sch` and `.kicad_pcb` in the project folder, recursively. Nothing consumes this yet.

The example starts here: `examples/feather-budget/` holds the RoyalBlue54L Feather from KiCad 10's demos (`share/kicad/demos/royalblue54L_feather`, CERN-OHL-P-2.0), copied without `img/`, with its LICENSE and a NOTICE. Beside it sit a Model with the `EngineeringTools` library, `part feather` linked to `board`, and envelope requirements.

See `.scratch/kicad-tracer/spec.md` (stories 1–4, 20–28, 30, 37, 38) and its probe notes, ADR 0005 and its amendment, and `.scratch/engineering-tools/issues/14-kicad-on-paper.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `"KiCad"` is in `built_in()`; a Model without KiCad links never starts it
- [ ] Stub `kicad-cli` (a script named by `SYSML_VIEWER_KICAD` that answers `version` and writes recorded output to `-o`): each dimension with its unit
- [ ] The stub asserts it was given `KICAD_CONFIG_HOME`; a stats recording in Polish (decimal commas, translated keys) is kept to show what English forcing prevents
- [ ] Reasons for: missing executable, version below 10, timeout (a stub that sleeps), missing `.kicad_pro`/`.kicad_sch`/`.kicad_pcb`, an object other than `board`
- [ ] The files an answer depends on: every schematic and board file in the project folder; FreeCAD's is its own file
- [ ] Real `kicad-cli` on `examples/feather-budget/`: 58.42 × 22.86 × 1.6 mm; skipped with a printed note when KiCad can't be found
- [ ] `sysml-viewer check examples/feather-budget` shows the Design Values section with the board dimensions and judges the envelope requirements
- [ ] The example folder carries the Feather's LICENSE and a NOTICE naming its source
