# 05: Exporting views from the CLI

**What to build:** CLI export covers views.

- `export --all` writes every supported view without a saved Diagram File as `<view simple name>.svg`. A saved model-defined Diagram File is exported under its own name with its Layout.
- Each Unsupported View gets a warning on stderr, and the exit status is still zero.
- Duplicate simple names, or a name that clashes with a Diagram File, fail before anything is written, naming both.
- `export --view <name> --out <file>` exports one view by simple or qualified name. It fails for an unsupported, erroneous, unknown or ambiguous name.

See `.scratch/model-defined-diagrams/spec.md`.

**Blocked by:** 04

**Status:** done

- [x] `--all` includes supported views and warns about unsupported ones without failing
- [x] Saved Layout is used for a view that has a Diagram File
- [x] Duplicate names fail, naming both
- [x] `--view` works, and each failure case gives a clear message

## Comments

- Done.
  - **`export --all`:**
    - Loads the Model even without Diagram Files, since views count. Without Diagram Files or views it says "no Diagram Files or views … nothing to Export".
    - Each supported view without a saved Diagram File is written as `<view::file_name>.svg` (`vehicle tree` → `vehicle-tree.svg`). A view saved as a Diagram File is exported once, under the file's name and with its Layout.
    - Unsupported Views give `warning: view `X` is not Exported: <reason>`, and the exit status stays zero.
    - An invalid view, such as an Interconnection view exposing nothing, is reported like a failed Diagram, and the rest are still exported.
    - A name clash (view against view, or view against Diagram File) fails before anything is written, naming both.
  - **`export --view <name> --out <file>`:** takes a qualified name, or a simple name when it's unique. It uses a saved Diagram File's Layout if one refers to the view. Unsupported, unknown and ambiguous names each fail with their own message.
  - **Tests:** `viewer-cli/tests/export_views.rs`. The real binary was checked on the `views` fixture: 4 SVGs plus the table warning.
