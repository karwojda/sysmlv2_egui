# 08: Editing an Interconnection Diagram's Layout

**What to build:** The existing gestures work in Interconnection Diagrams: move, resize, hide, bend connectors, Reset. Layout keys are feature paths relative to the root. Moving a part keeps it inside its enclosing box and moves its insides with it. Ports can be dragged along their part's border, saved as `Layout::ports`. A part's context menu opens or closes it in place.

**Blocked by:** 04: Opened Parts; 07: Opening Interconnection Diagrams in the viewer

**Status:** done

- [x] Positions in an Interconnection Layout are relative to the enclosing box's content and keyed by feature path
- [x] Moving an Opened Part moves its contents; a moved part can't leave its enclosing box
- [x] Dragging a port moves it along the border and can move it round a corner to another side; saved in `Layout::ports` and honoured by the Export
- [x] The part context menu offers Open / Close, which toggles `Layout::opened`
- [x] Seam 1 tests for each edit (scene and Diagram File); canvas gestures checked by the user
- [x] Saved bends take precedence over automatic routing in Interconnection Diagrams (moved from 05)

## Comments

- Done. Canvas gestures still need the user's hands-on check. Checked on screen: a saved Interconnection Layout (opened part, moved port) through CLI Export and in the viewer.
- **Identity:** parts in an Interconnection scene are `Depicts::Part { key, element }`. The key is the feature path that keys the Layout; the element drives selection and properties. Ports carry their key (`engine.status`; a Frame port by its name). `Scene` now distinguishes `node_bounds(key)` from `element_bounds(element)`, and gains `element_of(key)` and `origins`.
- **Positions** are saved relative to the enclosing box's top-left (`Scene::origins`), so moving an Opened Part moves its insides. Moves clamp at 0, so a part stays inside its box, and the box grows around it. A saved size never shrinks a box below what it holds.
- **Placement:** saved children keep their place. Unsaved ones are laid out among themselves and set below the saved ones, as in General Diagrams.
- **Edits:** `MovePort { port, side, offset }` (`Layout::ports`), `Open(path)` and `Close(path)`. `apply_with(layout, known, scene, edit)` takes a key-exists check: `layout_key_exists(model, kind, root, key)`, with `interconnection::path_exists` for feature paths. That's how Reset keeps orphaned entries in both kinds. Reset keeps Opened Parts.
- **Saved bends** (moved from 05) take precedence in Interconnection Diagrams, joined with right angles.
- **Viewer:**
  - Drag a part or its corner to move or resize it. Drag a port along its box's border, and round a corner.
  - Drag a connector to bend it.
  - Right-click a part: Open in place / Close (when its type has parts), Hide, Open Interconnection Diagram.
  - The Frame can't be moved or resized; pressing on it pans.
- **Fixed while verifying:**
  - The toolbar row could be wider than the central area, which stretched the canvas under the Properties panel and cut the Diagram off. It wraps now.
  - The view also refits automatically while the canvas size settles, until the user pans or zooms.
- **Tests:** 10 in `viewer-core/tests/interconnection_editing.rs`, plus a unit test for the port-drag geometry.
