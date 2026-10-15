# 09: Layout editing and saving Diagrams in the viewer

**What to build:** The user adjusts a Diagram on the canvas and saves it. Interactions become editing commands applied to the open Diagram's Layout in the core: move, resize, hide, show again, collapse, expand and reset to automatic. The egui layer only translates input into commands. The user saves a Diagram under a name as a Diagram File, sees saved Diagrams listed when opening the Model, can keep ad hoc Diagrams unsaved, and is warned about unsaved changes.

**Blocked by:** 06: Layout merge and Layout-aware Export; 08: General Diagram on the canvas

**Status:** done

- [x] Dragging a node moves it, and resizing a node changes its size
- [x] An element can be hidden from a Diagram, and hidden elements can be shown again
- [x] A node's compartments can be collapsed and expanded
- [x] Reset returns the Diagram to the automatic arrangement
- [x] Save as writes a named Diagram File; save updates it
- [x] Saved Diagrams are listed when a Model is opened and can be reopened
- [x] Opening a Diagram without saving it creates no file
- [x] An unsaved-changes indicator is shown, and closing a Diagram or the viewer with unsaved changes asks for confirmation
- [x] The viewer never writes `.sysml` or `.kerml` files
- [x] Each editing command is tested through the core: apply the command, then assert on the resulting scene and the Diagram File written

## Comments

- Done. Core `viewer_core::editing`: `Edit` = Move, Resize, Hide, Show, ShowAll, Collapse, Expand, Reset, applied by `apply(layout, model, scene, edit)`.
  - Any real edit first pins every drawn node where it is, so changing one node never rearranges the others.
  - An edit that changes nothing leaves the Layout untouched, so the Diagram isn't marked unsaved.
  - Nodes stop at 0,0. The minimum size is 40×24.
  - Collapse and Expand drop any saved size (back to natural size).
  - Reset keeps Orphaned Layout Entries.
- A saved node size is now optional (`NodePlacement { x, y, size? }`): positions are saved alone until the user resizes.
- Canvas:
  - Drag a node to move it, or drag its corner handle (shown on hover or selection) to resize it. Dragging empty space pans.
  - Right-click a node: Hide, Collapse/Expand. Right-click empty space: Show all hidden, Reset layout.
  - Toolbar: Save (saved Diagrams), Save as `<name>`, Hidden (n) with Show and Show all, Reset layout.
- The Saved Diagrams list sits above the Browser. Click to open, or switch to the tab if it's open. Ad hoc Diagrams leave no file. Tabs show `•` when unsaved. Closing a tab or the window with unsaved changes asks first.
- Names are limited to letters, digits, `-`, `_` and `.`. "Save as" refuses another Diagram's name. Saving never touches Model files (tested).
- Tests: `viewer-core/tests/editing.rs` (10) and `viewer-app/tests/saving.rs` (8).
- **Not exercised by hand:** drag, resize, context menus and the close dialogs. The screenshot tool can't send input. Checked by eye: the toolbar and Saved Diagrams list, plus the Export of a saved Layout (positions, hidden, collapsed).
- Follow-up: ticket 14 (straight edges can run behind nodes; adding bends in the UI).
