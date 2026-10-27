# Engineering Tools: design

How this tool reads values from an Engineering Tool (FreeCAD and KiCad) into its checks. The decisions are in ADRs 0003 to 0006; the reasoning, measurements and prototypes are in the wayfinder map `.scratch/engineering-tools/map.md` and its tickets. This note is the input for specifying a FreeCAD tracer build.

## In one paragraph

A **Tool Link** in the Model says where an element is designed: `@ToolLink { tool; file; object; }` on a usage, and `@ToolProperty { name; }` on each of its features that should take a value from there. On every load, the OpenSysML backend reads the links, asks each tool once for everything linked to it, writes each answer into the Model **in memory** as the feature's **Design Value**, re-parses, and runs the checks. So Satisfaction Status, Constraint Results, Computed Values and Analysis Results all reflect the design, while the `.sysml` files stay untouched. The **Model Value** is kept and shown beside the Design Value; their difference is shown, never judged. When a Design Value can't be had, the checks that depend on it are Undecided with the reason, never quietly run on the Model Value.

## The pipeline

1. **Parse** the Model as today (`viewer-opensysml`, `load_with`).
2. **Read Tool Links** from the api-json form: each `MetadataUsage` typed `EngineeringTools::ToolLink` or `::ToolProperty` has an `owner` (the linked element or feature), and each field's literal is the item `<metadata id>__<field>_pvalue`. A `:>> volume` redefinition is unnamed there; name it by following `redefines` (the backend's `valued_element` already does this recursion). Check: a `@ToolProperty` whose owner's owner has no Tool Link is a diagnostic; two links to one tool on one element is a diagnostic.
3. **Query the tools**: group the wanted `(object, property)` pairs per tool and file, and call each tool's adapter once per load. `file` is relative to the `.sysml` file holding the link.
4. **Write Design Values** with one OpenSysML editor per document (`in_document`):
   - feature with a value: `set_value(feature, "<value> [<unit>]")`;
   - feature with a body and no value (every `@ToolProperty` feature written without one): `delete` + `add_member(owner, "attribute", "", redefines = [name], value)`, since `set_value` refuses it (request in `docs/upstream/opensysml-issues.md`, Issue 3);
   - **no Design Value**: write the string `"no Design Value: <reason>"` the same way. Removing the value isn't enough, because a def's `default` shows through a valueless redefinition; the string can't be a quantity, so OpenSysML makes exactly the dependent checks Undecided with our text in its error chain, and independent checks still run.
5. **Re-parse** the edited documents inline, plus the untouched ones from disk, and **run the checks** on that Model. Translate the marker back into words in every report.

The list of `(feature qualified name, value text)` from step 4 is the store-neutral change set a future write to the Model would apply and save (needs a new ADR amending ADR 0002).

## Code layout

- `viewer-core`: the `EngineeringTool` trait (a file, and per object the properties wanted, in; per property a value with unit or a reason, out), and on the `Model` the Tool Links and Design Value outcomes (Design Value, Model Value, source, or reason). No OpenSysML.
- `viewer-tools` (new): the adapters. Knows nothing of SysML.
  - **FreeCAD**: runs `freecadcmd` with an embedded script (`include_str!`, written to a temp dir) that opens the `.FCStd`, resolves each object by `Name` or assembly path (`getSubObject`), and prints JSON. Properties: `Volume` [mm³], `Area` [mm²], `CenterOfMass.x|y|z` [mm], `Mass` [kg] only when the object has a real material (not "Default", found in the local library), otherwise a reason. Check that the first object of an assembly path exists, for a clean reason.
  - **KiCad**: runs `kicad-cli` (10 or newer) on the `.kicad_sch` and `.kicad_pcb` named like the linked `.kicad_pro`; see "KiCad adapter" below.
- `viewer-opensysml`: steps 2, 4 and 5, given the adapters.
- GUI and CLI build the backend with the adapters; tests use a fake tool.

Finding a tool: an environment variable per tool (`SYSML_VIEWER_FREECAD`, `SYSML_VIEWER_KICAD`), then `PATH`, then known install locations, including Windows installs seen from WSL (`/mnt/c/Program Files/FreeCAD */bin/freecadcmd.exe`; a Windows tool needs paths converted with `wslpath -w`). Never through the Model.

## Decided behaviour

- **Values:** scalars only; vector properties per component. Units come from the adapter, never the link; OpenSysML converts (`1.0e6 [mm^3] * 7850 [kg/m^3]` is exactly 7.85 kg). Write units with `^` inside brackets. A wrong dimension is Undecided, never wrong.
- **Scope of a Design Value:** only the linked usage, not its def or sibling usages. Links go on usages of any kind, never on defs.
- **FreeCAD mass:** the recommended route is linking `Volume` and computing mass in the Model from a density Model Value.
- **When:** inside every `load_with` (GUI worker thread, CLI alike), one result per load, no "checks pending" state. Measured: FreeCAD ~0.6–0.75 s per run including startup; reading links ~110 ms, edit ~55 ms, re-parse ~45 ms. Saving a linked file reloads the Model (the watcher's fingerprint covers linked files). Design Values come from saved files only. The GUI caches answers in memory by file path, modification time and size plus what was asked; the CLI never caches. Each tool run times out after 30 s (changeable per machine), giving no Design Value with the reason. No fingerprint on the link.
- **CLI:** an unreadable Tool Link fails `check` and `requirements` without `--strict` (`error: N Tool Links could not be read`); both print a "Design Values" section first when the Model has links; `--model-values` on every command skips the tools; `export` only warns. CI gets FreeCAD from the AppImage (`--appimage-extract`) with `SYSML_VIEWER_FREECAD` set.
- **GUI:** a linked usage's compartment row carries `⚙ <tool> <object>` (`⚠` when any of its Design Values is missing); a linked feature's row shows `= <design> ⚙ (Model: <model value>)`. Properties shows a Tool Link row, and for a feature Design Value then Model Value. A Design Values tab beside Properties and Style lists every linked feature with difference, source or failure, then the Verdicts that rest on Design Values.
- **Library:** `EngineeringTools.sysml` ships with the tool (for example `sysml-viewer library`) and is copied into the Model by its users; the tool recognises the defs by qualified name and never writes the file.

## FreeCAD tracer build

The thinnest slice that exercises every seam, end to end:

1. `EngineeringTools` library file and a way to get it.
2. Reading Tool Links and Tool Properties, with the two diagnostics.
3. The `EngineeringTool` trait and the FreeCAD adapter (`Volume` is enough), found by variable, `PATH` and known locations, with the 30 s timeout.
4. Writing Design Values and the "no Design Value" marker, re-parse, checks.
5. CLI `check`: the Design Values section, the translated marker, and failing on an unreadable link.
6. GUI Properties: Tool Link, Design Value and Model Value rows.
7. Fix the viewer showing nothing for fraction-valued results (`value_text` in `crates/viewer-opensysml/src/checks/service.rs` drops `Rational`), or roll-ups like `drive.mass` stay blank; print units readably (`mm³`, not `mm**3`).

Then, as their own slices: the Design Values tab and the Diagram row marks, `--model-values`, watching linked files and the in-memory cache, `Mass` with the material guard, the CI recipe, and KiCad.

### As built

Where the tracer build differs from, or adds to, the design above:

- **One FreeCAD run per file, not per load.** The adapter is called once per tool and file, so a Model whose links reach three `.FCStd` files starts FreeCAD three times; story 27 of the tracer spec asks for one run per load. Each run costs about 0.6–0.75 s, mostly startup. Batching every file of a load into one run needs the script to take several documents; it is left for when Models link many files.
- **`export` and unreadable links.** `sysml-viewer export` loads Design Values like every other command, so a Diagram showing Verdicts shows the design's. When a Tool Link can't be read there, the affected Verdicts draw as Undecided, and `export` prints no warning and still succeeds: the planned `export` warning is out of the tracer's scope.
- **A timed-out Windows FreeCAD.** Under WSL the adapter runs `freecadcmd.exe` through the interop process and kills that process on timeout. Tried against FreeCAD 1.1 at `C:\Program Files\FreeCAD 1.1` with a script that sleeps: killing the interop process ends the Windows `freecadcmd.exe` too (`tasklist.exe` no longer lists it), so no `taskkill.exe` is needed. A process FreeCAD itself starts would not be covered; `freecadcmd` starts none for the `Volume` query.

## KiCad adapter

As built in the KiCad tracer (`.scratch/kicad-tracer/spec.md`, ADR 0006, the amendment to ADR 0005), in `crates/viewer-tools/src/kicad.rs`. Example: `examples/feather-budget/`.

- **Link:** `@ToolLink { tool = "KiCad"; file = "ecad/board.kicad_pro"; object = "board"; }`. The adapter uses the `.kicad_sch` and `.kicad_pcb` of the same name beside the `.kicad_pro`, and names whichever is missing. `board` is the only object; any other gets a reason saying so.
- **Properties of `board`:**
  - `Width`, `Height`, `Thickness`, `Area` from `pcb export stats --format json --units mm` (`board.width`, …, `"58.4200 mm"` becomes `58.42 [mm]`, `mm²` becomes `mm^2`).
  - `ComponentCount`, the number of placed parts, unitless, from `sch export netlist --format kicadxml`.
  - Any other name is a symbol field, matched exactly, case included, and added up over the placed parts into a **Summed Design Value** that carries its counted and uncounted parts (ADR 0006).
- **Placed parts** are the netlist's components on every sheet that are neither `dnp` nor `exclude_from_bom`. Parts that aren't placed are neither counted nor uncounted.
- **Summing a field:** a missing or empty field leaves its part uncounted. A present field reads as a number with an optional unit made of an SI prefix (`p n u µ m k M G`) and a base symbol (`45mW`, `1.2 W`, `350µW`). The sum is in the base unit (`0.060953 [W]`), or unitless when every field is a bare number (prices). An unreadable field (`lots`, `1,5 W`), two base symbols (`mW` and `mA`), or bare numbers mixed with units leave no Design Value, with a reason naming the parts. When no placed part has the field, the sum is 0 with every placed part uncounted.
- **Runs:** only the exports the wanted properties need are run (the netlist for sums and the count, the statistics for dimensions), side by side; one export failing leaves only its own properties without a value. Before them, `kicad-cli version` must say 10 or newer (`KiCad 10 or newer needed; found 9.0.3`), since `pcb export stats` is new in 10. Each run is killed after `SYSML_VIEWER_KICAD_TIMEOUT` seconds, 30 by default. About 1.5 to 2.5 s per export from WSL.
- **Finding it:** `SYSML_VIEWER_KICAD` (final when set), then `PATH`, then `KiCad/<version>/bin` under `/mnt/c/Program Files`, `/Applications` (`KiCad.app`) and `%ProgramFiles%`, newest version first.
- **Language:** every run gets a fresh `KICAD_CONFIG_HOME` (passed through `WSLENV` to a Windows `kicad-cli.exe`) whose `10.0/kicad_common.json` sets the language to English. A KiCad left on its default language writes the statistics with that locale's decimal commas and keys, and locale variables don't change it. The user's own KiCad config is never read or written.
- **Dependent files:** every `.kicad_sch` and `.kicad_pcb` in the project's folder and below it, for watching linked files when that is built.
- **Not yet:** single components as objects, `Value` fields as quantities, variants, nets, simulation (`kicad-cli` has none).

Reference Models and code: branches `prototype/live-freecad` (the pipeline as one program, `prototypes/live-freecad/`) and `prototype/design-values-ui` (the GUI variants).

## Not designed here

- Later, on top of this design: writing to the Model.
- Out of scope: Tool Links on defs, vector Design Values, pushing values into CAD or EDA, connectivity checks against KiCad nets, choosing where the Model is stored, the Sireum/AADL code-generation flow.
