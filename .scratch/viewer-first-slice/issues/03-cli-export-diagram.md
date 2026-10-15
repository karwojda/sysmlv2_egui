# 03: CLI `export --diagram` from a Diagram File

**What to build:** A headless command-line tool. `export <model-folder> --diagram <name> --out <file.svg>` loads the Model through the OpenSysML backend, reads the named Diagram File, and writes the SVG Export. This ticket defines the minimal Diagram File: format version, Diagram name, Diagram Kind, root element qualified name, and an empty Layout. Diagram Files live in a dedicated folder inside the Model folder.

**Blocked by:** 01: Diagram pipeline tracer; 02: OpenSysML backend and sidecar manager

**Status:** ready-for-agent

- [ ] The Diagram File is human-readable text with stable key ordering and holds format version, name, Diagram Kind, root element qualified name and an (empty) Layout
- [ ] `export --diagram <name> --out <file>` writes the SVG for that Diagram, identical to what the core pipeline produces
- [ ] The command runs without a display
- [ ] It exits non-zero and lists diagnostics when the Model has errors
- [ ] It exits non-zero and names the Diagram when its root element no longer exists in the Model
- [ ] It exits non-zero with a clear message when the named Diagram File doesn't exist
- [ ] Pipeline tests (fake backend) cover the exit-code cases. One opt-in test runs the real command against a `.sysml` fixture
