# 07: Opening Interconnection Diagrams in the viewer

**What to build:**
- Opening: double-clicking a part or part def on any canvas opens its Interconnection Diagram. So does "Open Interconnection Diagram" in the Browser context menu and the Properties panel.
- Command line: `sysml-viewer-gui <folder> <root>` opens an Interconnection Diagram for a part usage root, or with `--interconnection`.
- Tabs read `Name (Interconnection)`. Clicking parts, ports and connectors shows their properties. Saving works as for General Diagrams.

**Blocked by:** 01: Interconnection tracer

**Status:** needs-triage

- [ ] Double-click on a part (General or Interconnection) or a part def opens or switches to its Interconnection Diagram
- [ ] Browser and Properties panel offer "Open Interconnection Diagram" for parts and part defs
- [ ] The CLI argument opens the right kind
- [ ] Ports and connectors can be selected and show their properties
- [ ] Save as / Save write `"kind": "interconnection"`, and saved ones reopen from the list
