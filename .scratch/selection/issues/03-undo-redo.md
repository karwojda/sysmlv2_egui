# 03: Undo and redo Layout edits

**What to build:** Each open Diagram keeps an undo/redo history of its Layout. One drag or one command is one step, every Layout edit including Reset can be undone, and the shortcuts are Ctrl+Z, Ctrl+Shift+Z and Ctrl+Y. See `.scratch/selection/spec.md` (Undo).

**Blocked by:** None (can start immediately; 01 and 02's edits are covered once they go through `OpenDiagram::edit`)

**Status:** ready-for-agent

- [ ] `OpenDiagram` keeps undo and redo stacks of Layouts
- [ ] A drag from press to release is one step; edits that change nothing record no step
- [ ] Context-menu and top-bar edits, including Reset, are one step each
- [ ] A new edit clears the redo stack
- [ ] Undo/redo set the unsaved flag correctly, and it clears when back at the saved Layout
- [ ] The history survives saving and is dropped when the Diagram closes or its Layout is replaced from outside
- [ ] Unit tests on `OpenDiagram` for all of the above
