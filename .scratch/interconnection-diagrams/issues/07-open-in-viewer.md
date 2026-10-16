# 07: Opening Interconnection Diagrams in the viewer

**What to build:**
- Opening: double-clicking a part or part def on any canvas opens its Interconnection Diagram. So does "Open Interconnection Diagram" in the Browser context menu and the Properties panel.
- Command line: `sysml-viewer-gui <folder> <root>` opens an Interconnection Diagram for a part usage root, or with `--interconnection`.
- Tabs read `Name (Interconnection)`. Clicking parts, ports and connectors shows their properties. Saving works as for General Diagrams.

**Blocked by:** 01: Interconnection tracer

**Status:** done

- [x] Double-click on a part (General or Interconnection) or a part def opens or switches to its Interconnection Diagram
- [x] Browser and Properties panel offer "Open Interconnection Diagram" for parts and part defs
- [x] The CLI argument opens the right kind
- [x] Ports and connectors can be selected and show their properties
- [x] Save as / Save write `"kind": "interconnection"`, and saved ones reopen from the list

## Comments

- Done. `open_diagram(kind, root)` replaces the General-only opening and switches to an open tab with the same kind and root.
- Ways in:
  - Double-click a part or part def on any canvas (not the Diagram's own root).
  - Browser double-click: General for packages and definitions, Interconnection for part usages.
  - Browser and Properties: "Open Interconnection Diagram" next to "Open General Diagram".
  - Canvas context menu on a node.
  - `sysml-viewer-gui <folder> <root> [--interconnection]`. The pure function `startup_kind` picks the kind and is unit-tested.
- Selection: a click selects a port (`Scene::port_at`, core), else the node, else the connection under the pointer. The Properties panel shows it, and a selected port or connection is outlined on the canvas.
- **Interconnection Layouts aren't editable yet (ticket 08).** On an Interconnection canvas, drags only pan, the resize handle is hidden, and the context menu offers only "Open Interconnection Diagram". This avoids edits that mark the Diagram unsaved but change nothing.
- Save as / Save keep the kind (tested), and saved Interconnection Diagrams reopen from the list.
- Fixed while verifying: the Properties panel's button row could force the panel wider than the window and cover the canvas. It wraps now.
- Verified on screen over X11 with `SCREENSHOT_INTERCONNECTION=1` on the screenshot example: the `Vehicle (Interconnection)` tab, the Diagram, and the Properties buttons. Canvas gestures (double-click, port selection) still need the user's hands-on check.
