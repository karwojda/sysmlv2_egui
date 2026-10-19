# 04: `check` CLI command

**What to build:** `sysml-viewer check <model-folder> [--strict]` prints the requirements report, then one line per Constraint Result (`✗ Violated  C::Tank::fits  on C::vehicle::tank: fuelMass <= fuelMassMax`), then a summary.

- **Exit status:** 1 on any violated requirement or constraint; with `--strict`, also on undecided or unclaimed ones.
- **Broken Model:** a Model with errors fails, saying nothing was checked.
- `requirements` is unchanged.

See `.scratch/constraints/spec.md`.

**Blocked by:** 02

**Status:** done

- [x] The report format, in a stable order
- [x] Exit codes, with and without `--strict`
- [x] A broken Model fails with its diagnostics

## Comments

- Done.
  - **Command:** `sysml-viewer check <folder> [--strict]` prints the `requirements` report unchanged, then one line per Constraint Result (sorted by constraint, then part), then `N constraint results: h hold, v violated, u undecided`.
  - **Exit status:** fails on any violated requirement or constraint, naming both (`1 requirement is violated; 1 constraint is violated`). `--strict` also fails on undecided ones and on unclaimed requirements. A Model with errors fails with "nothing was checked".
  - **Refactor:** the requirements report is now a reusable `requirements_report` returning counts, and `requirements` behaves as before.
  - **Docs:** README updated.
  - **Tests:** `viewer-cli/tests/check.rs` (6).
