# 04: KiCad in CI

**What to build:** CI installs KiCad 10.0.7 on the Linux runner and runs the workspace tests with `SYSML_VIEWER_KICAD` pointing at its `kicad-cli`. That way the real-KiCad tests run against `examples/feather-budget/` on every push instead of skipping. Prefer the official KiCad 10 release PPA, or the `kicad/kicad:10.0.7` image if the PPA proves slow or unavailable, and cache what can be cached. Running headless on the runner with no display set is untested (`.scratch/engineering-tools/issues/05-kicad-headless.md`); confirm it.

See `.scratch/kicad-tracer/spec.md` (story 34) and the existing workflow's `contract` job for how a real external program is brought into CI.

**Blocked by:** 03 (Part count and summed fields from the netlist)

**Status:** ready-for-agent

- [ ] A CI job installs KiCad 10.0.7 and sets `SYSML_VIEWER_KICAD`
- [ ] The real-KiCad tests run in that job and pass, without the "skipped" note in the log
- [ ] The existing jobs are unchanged and stay green
