# Where does the integration code run?

**Status:** resolved
**Type:** grilling
**Blocked by:** 01, 03, 05
**Map:** [Engineering Tools](../map.md)

## Question

Does this program call each Engineering Tool directly as a subprocess (e.g. `freecadcmd` with a bundled script), talk to a separate adapter process per tool over a small protocol, or does the integration sit behind a Model server? How do the viewer, the CLI and CI each reach it, and what does that mean for KiCad as the second tool and for writing to the Model later?

## Answer

**Built-in adapters behind one trait, run inside the load.** Resolved by grilling, recorded in ADR 0005 (`docs/adr/0005-engineering-tools-are-built-in-adapters-run-during-load.md`).

1. **Built-in adapters** behind an `EngineeringTool` trait: `(file, object → properties)` in, values with units or reasons out. Each adapter decides how to reach its tool (FreeCAD: `freecadcmd` with our script, as in "Does a live FreeCAD query work from this tool?"). No plug-in protocol (could be drawn out of the trait later); no Model server (out of scope: local-first, `.sysml` in git).
2. **Finding a tool:** a per-tool environment variable (for example `SYSML_VIEWER_FREECAD`), then `PATH`, then known install locations, including Windows installs seen from WSL (`/mnt/c/Program Files/FreeCAD */bin/freecadcmd.exe`). Never through the Model; no settings file yet. CI sets the variable.
3. **Once per load**, one process per tool for all its files. No long-lived tool processes; revisit if reloads feel slow.
4. **Code layout:**
   - `viewer-core`: the `EngineeringTool` trait, and Tool Links plus Design Value outcomes on the `Model` (value with unit and Model Value, or the reason there is none). No OpenSysML.
   - new `viewer-tools`: the adapters; FreeCAD's embeds `query.py` (`include_str!`), writes it to a temp dir and runs `freecadcmd`. Knows nothing of SysML.
   - `viewer-opensysml`: reads Tool Links from api-json, calls the adapters it is given, and does the OpenSysML-specific part: `set_value`, delete + add_member for features with a body, the "no Design Value" marker, the re-parse.
   - GUI and CLI build the backend with the adapters; tests use a fake tool.
5. **Inside `load_with`:** the GUI already loads on a worker thread (`crates/viewer-app/src/loader.rs`), the CLI calls the same `load_with`. A load returns one result with Design Values and their checks; no two-phase load, no "checks pending" state, never Model-Value Verdicts first. Diagrams of a Model with Tool Links appear ~0.7 s later.
6. **Future write:** the Design Value list is the store-neutral change set (feature qualified name, value text). Writing applies the same edits and saves the text instead of only re-parsing; it needs a new ADR amending ADR 0002.
7. **KiCad on paper:** the trait fits. A KiCad adapter could read `.kicad_sch`/`.kicad_pcb` S-expressions in Rust with no KiCad installed, and call `kicad-cli` only for netlists or board stats. How KiCad's text fields (`"10k"`, `"100nF"`) become quantities is left to "Does the design hold for KiCad?".
