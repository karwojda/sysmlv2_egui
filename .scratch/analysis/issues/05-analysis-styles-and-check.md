# 05: Analysis in Styles and `check`

**What to build:**

- **Style condition:** `{"analysis": "holds" | "violated" | "undecided"}` matches analysis usages by objective outcome, ranks with `satisfaction`, has Legend labels "Analysis holds / violated / undecided", and is offered in the Style editor.
- **`check`:** reports one line per analysis usage after constraints:
  - ✓ Holds or ✗ Violated, each with its outputs, plus the condition when violated, or ? Not run.
  - It fails on a violated objective; `--strict` also on undecided or not run.
  - It follows the setting, with `--analyses` / `--no-analyses` to override.

See `.scratch/analysis/spec.md`.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] Style condition, rank, Legend and editor picker
- [ ] `check` analysis lines and exit codes, with and without `--strict`
- [ ] `--analyses` / `--no-analyses` and the setting
