# 07: CLI: Design Values section and failing on unreadable links

**What to build:** `sysml-viewer check` and `requirements` load with the FreeCAD adapter. When the Model has Tool Links, they print a "Design Values" section first: one line per linked feature with its Design Value, its source and `(Model says …)`, or the reason there is none. Then the usual output, with Undecided lines carrying the translated reasons. They fail when any Tool Link couldn't be read, with its own line `error: N Tool Link(s) could not be read`, even without `--strict`, so a CI machine without FreeCAD can't pass a design that violates its budget. A Model without Tool Links prints exactly what it does today.

See `.scratch/freecad-tracer/spec.md` (stories 29–32, 34, 35).

**Blocked by:** 03 (Design Values replace Model Values in load), 06 (FreeCAD adapter)

**Status:** ready-for-agent

- [ ] `check` and `requirements` pass the FreeCAD adapter to the backend
- [ ] "Design Values" section printed first, only when the Model has Tool Links
- [ ] Each line: Design Value, source (tool, file, object, property), `(Model says …)`; or the reason
- [ ] Any unreadable link fails the command with `error: N Tool Link(s) could not be read`, independent of `--strict`; singular/plural correct
- [ ] Existing failure lines and exit codes otherwise unchanged
- [ ] Output for a Model without Tool Links unchanged
- [ ] Tests with `FakeBackend` returning a `Model` with Design Value outcomes and Tool Links (prior art: `viewer-cli/tests/check.rs`, `requirements.rs`)
