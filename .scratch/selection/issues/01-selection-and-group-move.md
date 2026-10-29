# 01: Selection and group move

**What to build:** A canvas Selection of nodes in one box. You build it with Ctrl/Shift-click and Shift-drag, and dragging any selected node moves the whole Selection. Moving it shifts the bends of edges whose two ends are both selected, and all nodes stop together at the top/left limit. See `.scratch/selection/spec.md` (Selection, Moving).

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] The open Diagram's canvas keeps a Selection of node keys, separate from the app's `selected` element
- [ ] Click, Ctrl/Shift-click, Shift-drag selection box, plain click on empty canvas and Escape behave as the spec says, and a plain drag on empty canvas still pans
- [ ] Every node in a Selection is in the same box; adding a node from another box starts a new Selection
- [ ] Ports, edges and the Interconnection Frame can't be selected
- [ ] The Selection empties on Reset and on switching tabs; nodes that are no longer drawn drop out
- [ ] Selected nodes are outlined
- [ ] A new group-move `Edit` clamps all nodes together and shifts the bends of edges with both ends selected
- [ ] Unit tests for the edit, in General and Interconnection Diagrams, and for the pure selection rules
