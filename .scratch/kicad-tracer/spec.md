# Spec: Board budgets from KiCad (tracer build)

Status: ready-for-agent

## Problem Statement

My Model gives the controller board a power budget of 500 mW, a cost limit, a part-count limit, and an envelope it must fit (width ≤ 60 mm, thickness ≤ 1.6 mm). The board itself is designed in KiCad. The power each part draws and its price live there as symbol fields filled in from datasheets and quotes, and the board's size lives in the layout. The viewer judges my requirements only against the estimates I typed into the Model. To find out whether the real board fits, I export a BOM, add up columns in a spreadsheet, measure the outline, and edit the `.sysml` files by hand. CI can't tell me when a new part pushes the board over budget. KiCad can't compute the board's power for me either: its simulator needs a SPICE model for every part and runs only in the GUI.

## Solution

I link the part that is the board to its KiCad project: `@ToolLink { tool = "KiCad"; file = "ecad/feather.kicad_pro"; object = "board"; }`. On each feature that should come from the design I put `@ToolProperty`, naming either a board dimension (`Width`, `Height`, `Thickness`, `Area`), the number of placed parts (`ComponentCount`), or a symbol field (`Power`, `Price`) for KiCad to add up over every placed part. On every load the viewer and the CLI ask KiCad for these through `kicad-cli` and use them in every check as **Design Values**, in place of my **Model Values**, exactly as FreeCAD's volume is used today.

A field added up over the parts is a **Summed Design Value**. A part with no such field is left out and named as uncounted ("summed 9 of 71 parts"), because most passives draw no power worth stating. A field I filled in but that can't be read (`lots`, or `mA` among `mW`) leaves the sum with no Design Value, and the checks that depend on it are Undecided with the reason (ADR 0006).

This is the tracer build for KiCad: the design in `docs/engineering-tools.md` and ADRs 0003 to 0006, carried through every seam once, at board level.

## User Stories

1. As a systems engineer, I want to link a part usage to a KiCad project with `@ToolLink { tool = "KiCad"; file = "….kicad_pro"; object = "board"; }`, so that the Model records where my board is designed.
2. As a systems engineer, I want the link to point at the `.kicad_pro` rather than the schematic or board file, so that I don't need to know which file holds which fact.
3. As a systems engineer, I want `file` relative to the `.sysml` file that holds the link, as for FreeCAD, so that moving my Model folder doesn't break it.
4. As a systems engineer, I want `@ToolProperty { name = "Width"; }`, `"Height"`, `"Thickness"` and `"Area"` to give the board's outline width, height, thickness and area with their units, so that envelope requirements are judged against the real layout.
5. As a systems engineer, I want `@ToolProperty { name = "ComponentCount"; }` to give the number of placed parts, so that a part-count limit is judged against the design.
6. As a systems engineer, I want any other `@ToolProperty` name on the board to add up that symbol field over the placed parts, so that I can sum whatever field my libraries use (`Power`, `Price`, `Current_3V3`) without matching a spelling chosen by this tool.
7. As a systems engineer, I want the field name matched exactly as KiCad stores it, case included, so that `Power` and `power` are not silently merged.
8. As a systems engineer, I want to write power with its unit in the field (`45mW`, `1.2 W`, `350uW`, `350µW`), so that the sum carries a unit and OpenSysML converts it to my feature's unit.
9. As a systems engineer, I want parts in mixed prefixes (`mW` and `W`) added correctly, so that I can copy figures from datasheets as they are written.
10. As a systems engineer, I want a field written as a bare number (`0.12`) on every part to give a unitless sum, so that I can sum prices, whose currency SysML's quantity library can't express.
11. As a systems engineer, I want a sum whose parts mix units of different kinds, or mix bare numbers with units, to give no Design Value with a reason naming the parts, so that a wrong field is never silently added.
12. As a systems engineer, I want a part that has no such field to be left out of the sum and listed as uncounted, so that I don't have to write `0W` on every resistor.
13. As a systems engineer, I want a field that is present but unreadable (`lots`, `TBD`) to leave the sum with no Design Value and name the part, so that a half-filled field never yields a too-small total.
14. As a systems engineer, I want a field that is present but empty to count as missing, so that a column KiCad shows for every symbol doesn't make every sum unreadable.
15. As a systems engineer, I want parts marked do-not-populate or excluded from the BOM left out of sums and counts entirely, neither counted nor uncounted, so that the budget reflects what is assembled.
16. As a systems engineer, I want the sums and counts taken from the project's default variant, so that the numbers are stable until variants are supported.
17. As a systems engineer, I want parts on every hierarchical sheet included, so that a board split into sheets sums the same as a flat one.
18. As a systems engineer, I want a Summed Design Value to replace the Model Value in every check, exactly as FreeCAD's values do (ADR 0003), so that my power and cost requirements reflect the design.
19. As a systems engineer, I want each Summed Design Value to record which parts were counted and which were not, so that I can see whether a total is complete.
20. As a systems engineer, when KiCad isn't installed or can't be found, I want the dependent checks Undecided with "KiCad not available", so that I know why nothing was judged.
21. As a systems engineer, when the installed KiCad is older than 10, I want the reason to say "KiCad 10 or newer needed; found 9.0.x", so that I know to upgrade rather than hunt for a broken link.
22. As a systems engineer, when the `.kicad_pro` doesn't exist, or its schematic or board file is missing, I want the reason to name the missing file, so that I can fix the path.
23. As a systems engineer, when the object isn't `board`, I want the reason to say only `board` can be linked for now, so that I don't wait for a value that will never come.
24. As a systems engineer, when `kicad-cli` hangs, I want the load to give up after 30 seconds with "KiCad did not answer within 30 s", and to set that timeout through `SYSML_VIEWER_KICAD_TIMEOUT`, so that neither the GUI nor CI waits forever.
25. As a systems engineer, I want to point the tool at `kicad-cli` with `SYSML_VIEWER_KICAD`, so that CI and unusual installs work.
26. As a systems engineer, I want `kicad-cli` found on `PATH` or in its usual install locations, newest version first, including a Windows install seen from WSL, so that it works on my machine without configuration.
27. As a systems engineer whose KiCad is set to another language, I want the same values as an English install, so that a decimal comma or translated report keys never break a link.
28. As a systems engineer, I want my own KiCad settings left untouched by the tool's runs, so that querying the design doesn't change my KiCad.
29. As a systems engineer, I want only the exports my links need to run (the netlist for sums and counts, the board statistics for dimensions), and those in parallel, so that a load is slowed by the slowest export, not the sum of them.
30. As a systems engineer, I want a Model without KiCad links to load exactly as before, with KiCad never started, so that this costs nothing when unused.
31. As a CI maintainer, I want the "Design Values" section of `check` and `requirements` to show a Summed Design Value as `Power = 0.412 W (summed 9 of 71 parts; 62 uncounted: C1, C2, … and 52 more)` with `(Model says …)`, so that the log explains a changed Verdict.
32. As a CI maintainer, I want the uncounted list cut after the first ten refs, in natural order (`C2` before `C10`), so that a large board doesn't flood the log.
33. As a CI maintainer, I want an unreadable KiCad link to fail `check` and `requirements` exactly as an unreadable FreeCAD link does, so that a machine without KiCad can't turn a violated budget into a pass.
34. As a CI maintainer, I want CI to install KiCad 10 and run the real-KiCad tests, so that the parsing of `kicad-cli` output is checked against the real program and not only against recordings.
35. As a systems engineer using the GUI, I want a summed feature's Design Value row in Properties to show the value with `⚙ KiCad board.Power` and "summed 9 of 71 parts", with every uncounted ref listed beneath, so that I can see which parts still lack a figure.
36. As a systems engineer using the GUI, I want a feature without a Design Value to show `⚠ none: <reason>` as it does for FreeCAD, so that I know why its checks are Undecided.
37. As a systems engineer, I want an example Model with a real KiCad board, a power budget, an envelope, a cost limit and a part-count limit, so that I can see the feature work end to end and copy its links.
38. As a maintainer, I want each adapter to say which files its answer depends on (for KiCad, every schematic and board file in the project folder), so that watching linked files can reload on a saved sheet when that slice is built.

## Implementation Decisions

- **Glossary.** Use **Engineering Tool**, **Tool Link**, **Design Value**, **Summed Design Value** and **Model Value** as `CONTEXT.md` defines them. ADR 0006 settles counted and uncounted parts; the amendment at the end of ADR 0005 allows more than one tool process per load.
- **Domain (`viewer-core`), one interface change.**
  - A value a tool reads may carry its parts: the parts counted and the parts left uncounted, each by reference designator.
  - The trait's `Answer` keeps its shape (per object and property, a read value or a reason), but a read value becomes the value text plus optional parts.
  - `DesignValue::Read` gains the same optional parts, so the CLI and Properties can show them.
  - FreeCAD's answers carry none, and its behaviour is unchanged.
  - The trait gains a way for an adapter to name the files an answer for a given file depends on. By default that is the file itself; KiCad names every `.kicad_sch` and `.kicad_pcb` in the project's folder, recursively. Nothing consumes this until watching linked files is built.
- **KiCad adapter (`viewer-tools`), registered as `"KiCad"` beside FreeCAD in `built_in()`.** It knows nothing of SysML.
  - **Finding `kicad-cli`:** `SYSML_VIEWER_KICAD` first and final, then `PATH`, then known install locations, newest version first. Those include `/mnt/c/Program Files/KiCad/<version>/bin/kicad-cli.exe` under WSL, whose arguments become Windows paths. It follows the FreeCAD adapter's `Machine` pattern.
  - **Version:** `kicad-cli version` must report 10 or newer, or every property gets the version reason.
  - **Timeout:** each run is killed after `SYSML_VIEWER_KICAD_TIMEOUT` seconds, 30 by default.
  - **Language:** every run gets its own temporary config folder through `KICAD_CONFIG_HOME` (passed with `WSLENV` to a Windows executable). The folder holds `10.0/kicad_common.json` with the language set to English. Locale variables have no effect on `kicad-cli`; this does. The user's own KiCad config is never read or written.
  - **Files:** the link names the `.kicad_pro`, and the adapter uses the `.kicad_sch` and `.kicad_pcb` of the same name beside it.
  - **Objects:** only `board`. Any other object gets a reason saying so.
  - **Exports:**
    - Board dimensions come from `pcb export stats --format json --units mm`, keys `board.width`, `board.height`, `board.board_thickness` and `board.area`. Their values are strings with units (`"58.4200 mm"`, `"605.284 mm²"`), which become `58.42 [mm]` and `605.284 [mm^2]`.
    - Sums and `ComponentCount` come from `sch export netlist --format kicadxml`. It lists every component on every sheet with its ref, its fields, and its do-not-populate and excluded-from-BOM properties.
    - Only the exports the wanted properties need are run, and they run in parallel.
  - **Placed parts** are those neither do-not-populate nor excluded from the BOM. `ComponentCount` is their number, unitless, with all of them counted.
  - **A Summed Design Value** is computed over the placed parts:
    - A missing or empty field means the part is uncounted.
    - Each present field must read as a number with an optional unit, where the unit is an SI prefix (`p n u µ m k M G`) and a base symbol. A number with no unit is a bare value.
    - The sum is given in the base unit (`0.412 [W]`), or with no unit when every field is bare.
    - An unreadable field, two base symbols, or a mix of bare and unit-carrying fields leaves no Design Value, with a reason naming the offending parts.
    - When no placed part has the field, the result is 0 with every placed part uncounted. That is still a Design Value, per ADR 0006.
- **Backend (`viewer-opensysml`).** No KiCad-specific code. It already groups wanted properties per tool and file, calls the adapter, and edits and re-parses; it now carries a read value's parts through to the Design Value outcome.
- **CLI.** The Design Values line of a Summed Design Value appends `(summed N of M parts; K uncounted: r1, r2, … and J more)`, with at most ten refs in natural order. When nothing is uncounted it is `(summed N of N parts)`. `ComponentCount` and the dimensions print like any other Design Value. The failure rule for unreadable links is unchanged.
- **GUI Properties.** The Design Value row of a summed feature shows the value, `⚙ KiCad board.<property>` and "summed N of M parts", and then every uncounted ref beneath. No Design Values tab yet.
- **Example and fixture.** `examples/feather-budget/`:
  - The RoyalBlue54L Feather demo from KiCad 10 (CERN-OHL-P-2.0), copied without `img/`, with its LICENSE and a NOTICE saying which fields were added.
  - Illustrative `Power` and `Price` fields on the active parts only: the nRF54L15, the debugger MCU, the nPM1300, LEDs and the crystal. Plausible datasheet figures, marked as illustrative in the NOTICE.
  - A Model with `part feather` linked to `board`, the `EngineeringTools` library, and requirements for power, width, thickness, part count and cost.
  - The project is opened once in KiCad by a person to confirm it still loads cleanly after the fields are added.
- **Design note.** `docs/engineering-tools.md` gains the KiCad adapter as built, replacing "KiCad (later)".

## Testing Decisions

- **Good tests** drive the feature through a public seam and assert what a user sees: Design Value outcomes with their parts, Verdicts and reasons, CLI lines and exit codes, Properties rows. They never assert which `kicad-cli` arguments were used, nor the temporary config folder's contents.
- **Backend `load` with a fake `EngineeringTool`** (prior art `viewer-opensysml/tests/contract.rs` and the FreeCAD tracer's fixtures):
  - A summed answer flips a power requirement from holds to violated.
  - Its counted and uncounted parts reach the Design Value outcome.
  - A reason for one property leaves only its dependent checks Undecided.
- **KiCad adapter through the `EngineeringTool` trait**, two ways:
  - **Stub `kicad-cli`:** a small script named by `SYSML_VIEWER_KICAD` that answers `version` and writes recorded outputs to the `-o` path. The recordings are a netlist and stats from the example project, plus a stats file recorded in Polish with decimal commas, which the adapter must never see once English is forced. The stub asserts it was given `KICAD_CONFIG_HOME`. Covers:
    - each dimension with its unit, and `ComponentCount`
    - sums with mixed prefixes, bare sums, and mixed kinds
    - missing, empty and unreadable fields
    - DNP and excluded-from-BOM parts
    - no part with the field
    - another object than `board`
    - a missing `.kicad_pro`, `.kicad_sch` or `.kicad_pcb`
    - a version below 10
    - a missing executable, and the timeout (a stub that sleeps)
    - only the needed export being run (a stub that fails on the other)
    - the files an answer depends on
  - **Real `kicad-cli`** on `examples/feather-budget/`: the dimensions (58.42 × 22.86 × 1.6 mm), the count, and the Power and Price sums with their uncounted parts. Skipped with a printed note when KiCad can't be found, like the FreeCAD tests (`viewer-tools/tests/freecad.rs`).
- **CLI with `FakeBackend`** (prior art `viewer-cli/tests/check.rs`): the summed line, the ten-ref cut and natural order, the no-uncounted form, and unchanged output for FreeCAD values.
- **Properties** (prior art `viewer-core/tests/properties.rs`): the summed Design Value row with uncounted refs, and unchanged rows for a FreeCAD value.
- **CI.** A job installs KiCad 10.0.7 on the Linux runner and runs the workspace tests with `SYSML_VIEWER_KICAD` set, so the real-KiCad tests run there. It is the last slice.

## Out of Scope

- Links to single components (`object = "R12"`, `uuid:/<sheet>/<symbol>`), and reading `Value` fields as quantities through the standard symbols (`Device:R` → Ω, RKM `4k7`). Designed in `.scratch/engineering-tools/issues/14-kicad-on-paper.md`; a later slice.
- Nets as linked objects, and checking the Model's connections against KiCad's nets.
- KiCad variants (`--variant`) and a variant slot on the Tool Link.
- Power by simulation (SPICE through ngspice), and per-rail current budgets.
- ERC and DRC results as evidence; board mass.
- The Design Values tab, `⚙`/`⚠` marks on Diagram rows, `--model-values`, watching linked files and the in-memory answer cache. These are already deferred by the FreeCAD tracer; the KiCad adapter only declares its dependent files.
- Reading KiCad's S-expression files directly, and `kicad-cli jobset` to fold the exports into one process.
- Writing to the Model, and pushing values into KiCad.

## Further Notes

- Grilled in the session that wrote this spec; earlier design in `.scratch/engineering-tools/map.md` (issues 05 and 14), ADRs 0003 to 0006.
- **Probe, KiCad 10.0.7** on Windows from WSL (`/mnt/c/Program Files/KiCad/10.0/bin/kicad-cli.exe`):
  - `kicad-cli` has no simulation command; ngspice ships only inside the GUI.
  - With the UI language left at "Default" on a Polish Windows, the stats JSON had `"16,5000 mm"` and Polish keys (`przelotek_na`). `LANG`, `LC_ALL` and `LANGUAGE` changed nothing. A `KICAD_CONFIG_HOME` folder whose `kicad_common.json` says English gave `"16.5000 mm"` and English keys.
  - Feather: 71 components in the netlist, of which 22 are do-not-populate. Netlist about 1.5 s, stats about 2.2 s.
  - StickHub and video demos: 1.7 to 5 s per export.
- **Demo licenses:**
  - The Feather (CERN-OHL-P-2.0) and tiny_tapeout (Apache-2.0) are the permissive ones.
  - StickHub is CC BY-NC-SA, and cm5_minima is CERN-OHL-S.
  - The rest carry no license file.
- `pcb export stats` exists from KiCad 10 on, hence the version floor.
