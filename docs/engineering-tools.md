# Engineering Tools: design

How this tool reads values from an Engineering Tool (FreeCAD first, KiCad later) into its checks. The decisions are in ADRs 0003 to 0005; the reasoning, measurements and prototypes are in the wayfinder map `.scratch/engineering-tools/map.md` and its tickets. This note is the input for specifying a FreeCAD tracer build.

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
  - **KiCad** (later): `kicad-cli` netlist export and `pcb export stats`; see "Does the design hold for KiCad?".
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

Reference Models and code: branches `prototype/live-freecad` (the pipeline as one program, `prototypes/live-freecad/`) and `prototype/design-values-ui` (the GUI variants).

## Not designed here

- Later, on top of this design: writing to the Model.
- Out of scope: Tool Links on defs, vector Design Values, pushing values into CAD or EDA, connectivity checks against KiCad nets, choosing where the Model is stored, the Sireum/AADL code-generation flow.
