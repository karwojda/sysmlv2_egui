# What can KiCad report headless?

**Status:** resolved
**Type:** research
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

What can `kicad-cli` and KiCad's scripting (the pcbnew Python module, and the IPC API introduced in KiCad 9) report without a GUI: BOM, symbol and footprint fields and properties, netlist, board dimensions? How are components and nets addressed stably (reference designators, UUIDs)? Install footprint on Linux, and query speed.

## Answer

**`kicad-cli` exports most of it headless, and much of it can be read straight from the files.** These findings come from documentation and source only; nothing was installed or run. KiCad 10 is the current stable release and 11 is in development.

- **`kicad-cli` runs headless.** Each call is a new process that exports from the files: BOM CSV with any symbol fields, netlists (S-expression or XML), ERC and DRC as JSON, and in KiCad 10 `pcb export stats` as JSON (board width, height, area and thickness, plus counts). There is no query interface, and no export of board-side footprint fields as data.
- **The IPC API needs a running GUI** in KiCad 9 and 10 and covers the PCB editor only. A headless `kicad-cli api-server` and schematic support arrive in KiCad 11. The SWIG `pcbnew` module works headless but was deprecated in 9 and is planned for removal in 11.
- **Stable addresses:**
  - A component is its sheet path plus its symbol UUID. This key joins schematic and board and survives re-annotation; reference designators are what people read, but they can change.
  - A net is its name. Net codes are internal and were dropped from the board file in KiCad 10, so a Tool Link should point at a labelled net.
- **Reading files without KiCad:**
  - Fields, DNP and BOM flags, UUIDs, stackup and pad nets are plain S-expressions.
  - Board size needs a bounding box computed over the Edge.Cuts shapes.
  - A netlist needs `kicad-cli`, because the schematic stores no net objects.
  - The format changes every major version, so a reader must check the version header.
- **Disk footprint:** the Debian package is about 164 MB installed plus GTK, wx and OpenCASCADE, and there is no CLI-only package. The `kicad/kicad:10.0.7` Docker image is about 812 MB.
- **Not measured:** query speed, the exact keys in the KiCad 10 stats JSON, and running on a bare host with no display set.

Findings: branch `research/kicad-headless` (commit aabdae7, not pushed: no GitHub credentials), `docs/research/kicad-headless.md`.
