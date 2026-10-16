# 05: Exporting views from the CLI

**What to build:** CLI export covers views.

- `export --all` writes every supported view without a saved Diagram File as `<view simple name>.svg`. A saved model-defined Diagram File is exported under its own name with its Layout.
- Each Unsupported View gets a warning on stderr, and the exit status is still zero.
- Duplicate simple names, or a name that clashes with a Diagram File, fail before anything is written, naming both.
- `export --view <name> --out <file>` exports one view by simple or qualified name. It fails for an unsupported, erroneous, unknown or ambiguous name.

See `.scratch/model-defined-diagrams/spec.md`.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] `--all` includes supported views and warns about unsupported ones without failing
- [ ] Saved Layout is used for a view that has a Diagram File
- [ ] Duplicate names fail, naming both
- [ ] `--view` works, and each failure case gives a clear message
