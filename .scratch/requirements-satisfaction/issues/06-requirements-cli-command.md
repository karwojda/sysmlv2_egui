# 06: `requirements` CLI command

**What to build:** `sysml-viewer requirements <model-folder> [--strict]` loads the Model and prints diagnostics to stderr. It writes one line per requirement usage to stdout, in qualified-name order: the mark, the status, the name, and the satisfiers with their failing condition or reason.

- **Exit status:** 1 on any Violated; with `--strict`, also on Undecided or Not claimed.
- **Broken Model:** a Model with errors fails, as Export does.

See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** 04

**Status:** done

- [x] The table format, in a stable order
- [x] Exit codes, with and without `--strict`
- [x] A broken Model fails with its diagnostics

## Comments

- Done.
  - **Command:** `sysml-viewer requirements <model-folder> [--strict]`.
  - **Report:** one line per requirement usage in qualified-name order, columns aligned: mark, status, name, then each claim as `by <satisfier> <mark>` plus its failing condition or reason, joined with `; `. A summary line follows (`6 requirements: 1 satisfied, 1 violated, 2 undecided, 2 not claimed`). A Model without requirement usages says so and passes.
  - **Exit status:** 1 with `error: N requirement(s) is/are violated`; with `--strict`, any requirement not Satisfied fails (`… not satisfied`). A Model with errors fails as Export does.
  - **API:** the table goes to stdout, through a new `viewer_cli::run_with_output(args, backend, stdout, stderr)`; `run` keeps its signature and writes to the real stdout.
  - **Docs:** README updated.
  - **Tests:** `viewer-cli/tests/requirements.rs` (7, using the core `requirements` twin). Checked against the real service on the fixture.
