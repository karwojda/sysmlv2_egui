# 01: Selection and group move

**What to build:** A canvas Selection of nodes in one box. You build it with Ctrl/Shift-click and Shift-drag, and dragging any selected node moves the whole Selection. Moving it shifts the bends of edges whose two ends are both selected, and all nodes stop together at the top/left limit. See `.scratch/selection/spec.md` (Selection, Moving).

**Blocked by:** None (can start immediately)

**Status:** done

- [x] The open Diagram's canvas keeps a Selection of node keys, separate from the app's `selected` element
- [x] Click, Ctrl/Shift-click, Shift-drag selection box, plain click on empty canvas and Escape behave as the spec says, and a plain drag on empty canvas still pans
- [x] Every node in a Selection is in the same box; adding a node from another box starts a new Selection
- [x] Ports, edges and the Interconnection Frame can't be selected
- [x] The Selection empties on Reset and on switching tabs; nodes that are no longer drawn drop out
- [x] Selected nodes are outlined
- [x] `Edit::Move` now takes several nodes; it clamps all nodes together and shifts the bends of edges with both ends selected
- [x] Unit tests for the edit, in General and Interconnection Diagrams, and for the pure selection rules

## Comments

- Done. Group moves generalise `Edit::Move` to `elements: Vec<_>` rather than adding a second move edit. An edge's bends move when the node nearest each end of its line is moved, or is inside a moved node. Nearest rather than touching, because a General Diagram's edges stop short of the node at an arrowhead. The Selection is `canvas::Selection` on each `OpenDiagram`. A selection box picks the outermost nodes wholly inside it, in one box (see the spec). It is outlined, and the box is drawn while dragging. Not checked by hand in the GUI, since clicks can't be driven from the agent's shell. Tests: `viewer-core/tests/editing.rs`, `interconnection_editing.rs`, `viewer-app/tests/selection.rs`.
