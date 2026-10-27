# 04: KiCad in CI

**What to build:** CI installs KiCad 10.0.7 on the Linux runner and runs the workspace tests with `SYSML_VIEWER_KICAD` pointing at its `kicad-cli`. That way the real-KiCad tests run against `examples/feather-budget/` on every push instead of skipping. Prefer the official KiCad 10 release PPA, or the `kicad/kicad:10.0.7` image if the PPA proves slow or unavailable, and cache what can be cached. Running headless on the runner with no display set is untested (`.scratch/engineering-tools/issues/05-kicad-headless.md`); confirm it.

See `.scratch/kicad-tracer/spec.md` (story 34) and the existing workflow's `contract` job for how a real external program is brought into CI.

**Blocked by:** 03 (Part count and summed fields from the netlist)

**Status:** ready-for-human

- [ ] A CI job installs KiCad 10.0.7 and sets `SYSML_VIEWER_KICAD`
- [ ] The real-KiCad tests run in that job and pass, without the "skipped" note in the log
- [x] The existing jobs are unchanged and stay green

## Notes

The `kicad` job in `.github/workflows/ci.yml` adds `ppa:kicad/kicad-10.0-releases`, installs `kicad=10.0.7~ubuntu24.04.1` without recommends, prints `kicad-cli version`, and runs `cargo test -p viewer-tools --test kicad -- --nocapture` with `SYSML_VIEWER_KICAD=/usr/bin/kicad-cli`. It fails when "note: skipping" is in the log. The runner is pinned to `ubuntu-24.04` rather than `ubuntu-latest`, because the package version is built for 24.04. Only cargo is cached (rust-cache); the apt packages are downloaded on every run.

Checked locally:

- The PPA's noble index lists `kicad 10.0.7~ubuntu24.04.1`.
- The downloaded `.deb` matches the index's SHA256 and contains `/usr/bin/kicad-cli`.
- Without KiCad, the test command prints the skip note twice, so the gate would fail the job.
- The YAML parses, the `build` and `contract` jobs are unchanged, and `scripts/check` passes.

Not checked: Docker Desktop was paused, so the install never ran in an `ubuntu:24.04` container. Only a real CI run (or an unpaused Docker) can confirm the remaining points:

- the PPA install works on the runner;
- `kicad-cli` runs headless with no display set;
- the two real-KiCad tests pass against 10.0.7.
