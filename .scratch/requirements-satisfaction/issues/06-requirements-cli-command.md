# 06: `requirements` CLI command

**What to build:** `sysml-viewer requirements <model-folder> [--strict]` loads the Model and prints diagnostics to stderr. It writes one line per requirement usage to stdout, in qualified-name order: the mark, the status, the name, and the satisfiers with their failing condition or reason.

- **Exit status:** 1 on any Violated; with `--strict`, also on Undecided or Not claimed.
- **Broken Model:** a Model with errors fails, as Export does.

See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] The table format, in a stable order
- [ ] Exit codes, with and without `--strict`
- [ ] A broken Model fails with its diagnostics
