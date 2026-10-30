# 03: Undo and redo Layout edits

**What to build:** Each open Diagram keeps an undo/redo history of its Layout. One drag or one command is one step, every Layout edit including Reset can be undone, and the shortcuts are Ctrl+Z, Ctrl+Shift+Z and Ctrl+Y. See `.scratch/selection/spec.md` (Undo).

**Blocked by:** None (can start immediately; 01 and 02's edits are covered once they go through `OpenDiagram::edit`)

**Status:** done

- [x] `OpenDiagram` keeps undo and redo stacks of Layouts
- [x] A drag from press to release is one step; edits that change nothing record no step
- [x] Context-menu and top-bar edits, including Reset, are one step each
- [x] A new edit clears the redo stack
- [x] Undo/redo set the unsaved flag correctly, and it clears when back at the saved Layout
- [x] The history survives saving and is dropped when the Diagram closes or its Layout is replaced from outside
- [x] Unit tests on `OpenDiagram` for all of the above

## Comments

- Done. `OpenDiagram` keeps up to `UNDO_STEPS` (200) Layouts to go back to, and the redo stack. `begin_drag`/`end_drag` group a drag into one step. The canvas calls them when a primary-button drag starts and stops. The toolbar has Undo and Redo buttons, and the shortcuts work unless a text field has focus. Diagram Files are only read when a Diagram is opened, so nothing currently replaces a Layout from outside; the history ends with the open Diagram. Tests: `crates/viewer-app/tests/undo.rs`.
