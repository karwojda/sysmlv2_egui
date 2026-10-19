# 04: `check` CLI command

**What to build:** `sysml-viewer check <model-folder> [--strict]` prints the requirements report, then one line per Constraint Result (`✗ Violated  C::Tank::fits  on C::vehicle::tank: fuelMass <= fuelMassMax`), then a summary.

- **Exit status:** 1 on any violated requirement or constraint; with `--strict`, also on undecided or unclaimed ones.
- **Broken Model:** a Model with errors fails, saying nothing was checked.
- `requirements` is unchanged.

See `.scratch/constraints/spec.md`.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] The report format, in a stable order
- [ ] Exit codes, with and without `--strict`
- [ ] A broken Model fails with its diagnostics
