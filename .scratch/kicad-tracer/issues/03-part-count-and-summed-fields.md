# 03: Part count and summed fields from the netlist

**What to build:** On a KiCad `board` link, `@ToolProperty { name = "ComponentCount"; }` gives the number of placed parts. Any other name that isn't a built-in (`Power`, `Price`) gives a **Summed Design Value** of that symbol field over the placed parts. Each one carries its counted and uncounted parts, so the CLI and Properties show "summed 9 of 71 parts" through ticket 01.

- **Source:** `sch export netlist --format kicadxml` lists every component on every sheet with its ref, fields, and `dnp` / `exclude_from_bom` properties.
- **Placed parts** are neither do-not-populate nor excluded from the BOM. Parts that aren't placed are neither counted nor uncounted.
- **Field names** match exactly, case included.
- **Field rules (ADR 0006):**
  - A missing or empty field means the part is uncounted.
  - A present field reads as a number with an optional unit, where the unit is an SI prefix (`p n u µ m k M G`) and a base symbol. Mixed prefixes add up, and the sum is given in the base unit (`0.412 [W]`).
  - If every field is a bare number, the sum is unitless.
  - An unreadable field, two base symbols, or bare numbers mixed with units leave no Design Value, with a reason naming the parts.
  - When no placed part has the field, the sum is 0 with every placed part uncounted.
- **Exports:** only the ones the wanted properties need run (netlist for sums and counts, stats for dimensions), in parallel.

The example gains illustrative `Power` and `Price` fields on the active parts only: the nRF54L15, the debugger MCU, the nPM1300, the LEDs and the crystal. Use plausible datasheet figures, and say in the NOTICE that they were added and are illustrative. The Model gains power, cost and part-count requirements, with Model Values as the pre-design estimates. `docs/engineering-tools.md` replaces "KiCad (later)" with the adapter as built.

See `.scratch/kicad-tracer/spec.md` (stories 5–17, 29, 37), ADRs 0005 (amendment) and 0006.

**Blocked by:** 01 (Summed Design Values from a fake tool), 02 (Board envelope from KiCad)

**Status:** done

- [x] Stub `kicad-cli` with a recorded netlist: `ComponentCount`; sums with mixed prefixes; a bare sum; mixed kinds; missing, empty and unreadable fields; DNP and excluded-from-BOM parts; no part with the field
- [x] Only the needed export runs: with only sums wanted, a stub that fails on `pcb export stats` still answers, and vice versa
- [x] Real `kicad-cli` on the example: the count and the `Power`/`Price` sums with their uncounted parts; skipped with a note when KiCad can't be found
- [x] `sysml-viewer check examples/feather-budget` judges power, cost, count and envelope from KiCad and lists the uncounted parts
- [x] Human: open the edited Feather project in KiCad once and confirm it loads with no errors and the added fields show
- [x] `docs/engineering-tools.md` describes the KiCad adapter as built
