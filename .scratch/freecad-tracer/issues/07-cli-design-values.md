# 07: CLI: Design Values section and failing on unreadable links

**What to build:** `sysml-viewer check` and `requirements` load with the FreeCAD adapter. When the Model has Tool Links, they print a "Design Values" section first: one line per linked feature with its Design Value, its source and `(Model says …)`, or the reason there is none. Then the usual output, with Undecided lines carrying the translated reasons. They fail when any Tool Link couldn't be read, with its own line `error: N Tool Link(s) could not be read`, even without `--strict`, so a CI machine without FreeCAD can't pass a design that violates its budget. A Model without Tool Links prints exactly what it does today.

See `.scratch/freecad-tracer/spec.md` (stories 29–32, 34, 35).

**Blocked by:** 03 (Design Values replace Model Values in load), 06 (FreeCAD adapter)

**Status:** done

- [x] `check` and `requirements` pass the FreeCAD adapter to the backend
- [x] "Design Values" section printed first, only when the Model has Tool Links
- [x] Each line: Design Value, source (tool, file, object, property), `(Model says …)`; or the reason
- [x] Any unreadable link fails the command with `error: N Tool Link(s) could not be read`, independent of `--strict`; singular/plural correct
- [x] Existing failure lines and exit codes otherwise unchanged
- [x] Output for a Model without Tool Links unchanged
- [x] Tests with `FakeBackend` returning a `Model` with Design Value outcomes and Tool Links (prior art: `viewer-cli/tests/check.rs`, `requirements.rs`)

## Comments

- Done.
  - **Section (`viewer-cli/src/lib.rs`, `design_values_report`):** printed before the requirements report by both `check` and `requirements` when the Model has Tool Links or Design Value outcomes. Each line is `⚙ Read        <feature>  <value> from <tool>: <property> of <object> in <file> (Model says <model value>)` or `⚠ Not read    <feature>  <reason>, for <tool>: <property> of <object> in <file> (Model says …)`, followed by `N Design Value(s): R read, M not read`. `(Model says …)` is left out when the Model writes no value. Values and Model Values go through the shared unit formatter, so `[mm^3]` prints as `[mm³]` and `[millimetre ** 3]` as `[mm³]`.
  - **Failing:** a Tool Link counts as unreadable when any of its features has no Design Value, with links told apart by (tool, file, object). `error: N Tool Link(s) could not be read` is a stderr line of its own, before the existing failure line, and it fails the command whatever `--strict` says. The existing lines and exit codes are unchanged. `Failure` can now carry several lines, and each one is printed with an `error: ` prefix.
  - **Adapter:** `main.rs` builds `OpenSysmlBackend::with_config(config).with_tool("FreeCAD", viewer_tools::FreeCad::new())` for every command, so `export` loads Design Values too. FreeCAD starts only when the Model has Tool Links. `viewer-cli` now depends on `viewer-tools`.
  - **Unit formatter:** moved from `viewer-opensysml/src/checks/service.rs` to `viewer_core::units` (`readable_unit`, plus `readable_value`, which formats the bracketed unit of a value text). It now also handles spaced `**`. When `freecad-tracer` was merged in, ticket 08's private copy in `properties.rs` was folded into it. That copy contributed unit names written as symbols (`millimetre` → `mm`), now matched as whole words only. The Properties rows, the CLI section and Computed Values all go through `viewer_core::units`.
  - **Tests:** `viewer-cli/tests/design_values.rs` has 7 tests on `FakeBackend`: the read line, the not-read lines, the failure without `--strict`, singular before the other failures, read values failing nothing, `requirements`, and no output change without links. `viewer_core::units` has its own unit tests. A live run against `viewer-tools/tests/fixtures/parts.FCStd` (`Body`) printed `1000000.0 [mm³] from FreeCAD: Volume of Body …`.
  - **Deviation:** the not-read line names the tool as well (`for FreeCAD: Volume of …`), to match the read line.
  - **With ticket 05 merged:** Undecided lines carry its translated reason. In the live run, a link to a missing object made the drive's mass requirement `? Undecided … no Design Value for Drive::drive::gearbox::volume (FreeCAD: no object named 'Gearbox' in …)`.
