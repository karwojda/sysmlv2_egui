# 05: `export --all`

**What to build:** `export <model-folder> --all --out-dir <dir>` Exports every Diagram File in the Model to `<dir>/<diagram-name>.svg`, so CI can regenerate every documentation image with no extra configuration.

**Blocked by:** 03: CLI `export --diagram` from a Diagram File

**Status:** ready-for-agent

- [ ] Every Diagram File in the Model is Exported to `<out-dir>/<diagram-name>.svg`, and the output directory is created if missing
- [ ] Running the command twice on unchanged inputs produces byte-identical files
- [ ] If any Diagram fails (for example a missing root element), the command still reports every failure, names each failed Diagram, and exits non-zero
- [ ] Model errors cause a non-zero exit, as with `--diagram`
- [ ] A Model with no Diagram Files exits zero and says so
- [ ] Pipeline tests cover naming, determinism and the failure cases
