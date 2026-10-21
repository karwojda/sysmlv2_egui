# Map: Engineering Tools

**Label:** wayfinder:map

## Destination

An agreed architecture for how this tool connects to Engineering Tools, recorded as ADRs and a short design note. FreeCAD is the worked example, and the design is checked on paper against KiCad. It is ready to hand to `/to-spec` for a FreeCAD tracer build.

## Notes

- Domain: SysML v2 Viewer. Vocabulary in `CONTEXT.md` (**Engineering Tool**, **Tool Link**, **Verdict**, …); ADR 0002 says the tool never writes the Model.
- Skills: `grilling` + `domain-modeling` for grilling tickets; `research` for research tickets; `prototype` for prototype tickets.
- For now, values flow one way: they are pulled from an Engineering Tool into checks (Satisfaction Status, Constraint Results, Analysis Results), queried live, and not stored. Writing to the Model is expected later, so the design must leave room for it, but it isn't built here.
- Tool Links are written in the Model as metadata on the element.
- Local-first is the default: no required server or cloud service; everything runs on the engineer's machine and in CI. Server-backed options are still researched.
- Prototypes are allowed where a question needs a runnable answer; they produce decisions, not features.

## Decisions so far

- [What can FreeCAD report headless?](issues/01-freecad-headless.md): `freecadcmd` gives volume, centre of mass and inertia per solid, but mass = volume × density from a material the file stores only as a UUID; address objects by `Name` plus the path through any assembly.
- [Can OpenSysML evaluate with values supplied from outside?](issues/02-opensysml-external-values.md): yes, by editing the Model in memory (`set_value`) and re-parsing it inline, with nothing written to disk; the API has no value overlay.
- [Where is the Model stored, and does it matter for integration?](issues/03-model-storage.md): `.sysml` in git is a sound, normative choice; API servers (OMG API 1.0, Flexo) give UUIDs and structured commits but need a server; SysGit is git-shaped. Element identity matters more than storage.
- [How do existing tools link SysML to CAD and EDA?](issues/04-prior-art.md): no standard Tool Link; the standard library's `AnalysisTooling` (`ToolExecution`/`ToolVariable` metadata) is the closest precedent; working integrations pull a few properties one way and compare before accepting; units and missing tools are unsolved everywhere.
- [What can KiCad report headless?](issues/05-kicad-headless.md): `kicad-cli` exports BOM, netlist and board stats (KiCad 10) with no GUI; the IPC API needs a GUI until KiCad 11; components are addressed by sheet path plus UUID, nets by name; most fields can be read from the S-expression files directly.
- [Does an Engineering Tool's value replace or challenge the Model's?](issues/06-replace-or-challenge.md): replace; checks run once on the Design Value, the Model Value is shown beside it and the difference is never a Verdict; no Design Value means Undecided, never a fallback; only the linked usage is affected (ADR 0003).
- [Make room for FreeCAD](issues/09-room-for-freecad.md): FreeCAD 1.1.4 is installed on Windows; `freecadcmd.exe` runs from WSL.
- [What does a Tool Link look like?](issues/07-tool-link-shape.md): `@ToolLink { tool; file; object; }` on a usage plus `@ToolProperty { name; }` on each linked feature, from an `EngineeringTools` library copied into the Model; no unit on the link (the adapter states it, OpenSysML converts); FreeCAD mass via linked volume × Model density (ADR 0004).
- [Does a live FreeCAD query work from this tool?](issues/10-live-freecad-prototype.md): yes, ~1 s on load (FreeCAD ~0.65 s per file, one process for all files, off the UI thread); body features need delete + add_member instead of `set_value`; a missing Design Value is written as a string marker so exactly the dependent checks go Undecided with the reason.
- [Where does the integration code run?](issues/08-where-integration-runs.md): built-in adapters behind an `EngineeringTool` trait in a new `viewer-tools` crate, run inside `load_with` once per load; tools found by env var, `PATH`, known locations; the Design Value list is the change set a future write would reuse (ADR 0005).
- [When do Tool Link queries run?](issues/11-when-queries-run.md): on every load; saving a linked file reloads; Design Values from saved files only, cached in memory in the GUI by file mtime and size, never in the CLI; no fingerprint on the link; 30 s timeout per tool run means Undecided.
- [How do the CLI and CI report Design Values and missing ones?](issues/12-cli-ci-reporting.md): an unreadable Tool Link fails `check` and `requirements` without `--strict`; a "Design Values" section with Model Values beside; `--model-values` skips the tools; `export` only warns; CI uses a FreeCAD AppImage recipe.
- [How do Design Values appear in the viewer?](issues/13-design-values-in-viewer.md): inline: linked rows carry `⚙ tool object` (`⚠` when broken), Properties shows Design Value then Model Value and a Tool Link row; plus a Model-wide Design Values tab with differences and failures. No new compartments.
- [Does the design hold for KiCad?](issues/14-kicad-on-paper.md): yes, no ADR changes; links point at the `.kicad_pro`, objects by reference designator or sheet-path UUID; text fields become quantities via standard symbols and explicit units; always `kicad-cli`; nets link but carry no Design Values.
- [Write the design note](issues/15-design-note.md): `docs/engineering-tools.md`, with the FreeCAD tracer build's slices; ready for `/to-spec`.

## Not yet specified

None: the destination is reached.

## Out of scope

- Sireum, AADL and HAMR code generation from the Model: a code-generation flow, not an evidence pull; its own effort.
- Pushing Model values into CAD or EDA designs.
- Choosing where the Model is stored (files plus git, SysML v2 API server, SysGit): only researched here, in "Where is the Model stored, and does it matter for integration?".
- Building the KiCad integration.
- Tool Links on defs, so a library of standard parts could be linked once: not needed for the FreeCAD tracer build; a later effort.
- Vector Design Values (centre of mass, inertia): not needed until a check uses one; a later effort.
- Checking the Model's connections against KiCad's nets: a structural check, not a Design Value; ruled out in "Does the design hold for KiCad?".
