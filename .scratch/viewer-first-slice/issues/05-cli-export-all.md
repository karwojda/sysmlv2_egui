# 05: `export --all`

**What to build:** `export <model-folder> --all --out-dir <dir>` Exports every Diagram File in the Model to `<dir>/<diagram-name>.svg`, so CI can regenerate every documentation image with no extra configuration.

**Blocked by:** 03: CLI `export --diagram` from a Diagram File

**Status:** done

- [x] Every Diagram File in the Model is Exported to `<out-dir>/<diagram-name>.svg`, and the output directory is created if missing
- [x] Running the command twice on unchanged inputs produces byte-identical files
- [x] If any Diagram fails (for example a missing root element), the command still reports every failure, names each failed Diagram, and exits non-zero
- [x] Model errors cause a non-zero exit, as with `--diagram`
- [x] A Model with no Diagram Files exits zero and says so
- [x] Pipeline tests cover naming, determinism and the failure cases

## Comments

- Done. `export <model> --all --out-dir <dir>`. The Model is loaded once. Every failing Diagram is reported, then a summary line (`N of M Diagrams failed to Export`) and exit 1. Diagrams that succeed are still written.
- With no Diagram Files, the Model isn't loaded at all (so the service never starts), and the command exits 0.
- clap enforces that `--diagram`/`--out` and `--all`/`--out-dir` aren't mixed.
