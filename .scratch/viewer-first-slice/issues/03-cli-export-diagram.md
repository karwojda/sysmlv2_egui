# 03: CLI `export --diagram` from a Diagram File

**What to build:** A headless command-line tool. `export <model-folder> --diagram <name> --out <file.svg>` loads the Model through the OpenSysML backend, reads the named Diagram File, and writes the SVG Export. This ticket defines the minimal Diagram File: format version, Diagram name, Diagram Kind, root element qualified name, and an empty Layout. Diagram Files live in a dedicated folder inside the Model folder.

**Blocked by:** 01: Diagram pipeline tracer; 02: OpenSysML backend and sidecar manager

**Status:** done

- [x] The Diagram File is human-readable text with stable key ordering and holds format version, name, Diagram Kind, root element qualified name and an (empty) Layout
- [x] `export --diagram <name> --out <file>` writes the SVG for that Diagram, identical to what the core pipeline produces
- [x] The command runs without a display
- [x] It exits non-zero and lists diagnostics when the Model has errors
- [x] It exits non-zero and names the Diagram when its root element no longer exists in the Model
- [x] It exits non-zero with a clear message when the named Diagram File doesn't exist
- [x] Pipeline tests (fake backend) cover the exit-code cases. One opt-in test runs the real command against a `.sysml` fixture

## Comments

- Done. Crate `viewer-cli`, binary `sysml-viewer`. Command logic is `viewer_cli::run(args, backend, stderr)`, so tests drive it with the fake backend. One `#[ignore]`d test runs the real binary headless against a real Model.
- Diagram Files live at `<model>/diagrams/<name>.diagram.json`: pretty JSON in a fixed key order, `format_version: 1`. An unknown version is rejected.
- Exit codes: 0 on success, 1 on any failure, 2 on a usage error (from clap). Diagnostics print as `file:line: severity: message`. Warnings print but don't fail the command.
- A missing output folder is created.
