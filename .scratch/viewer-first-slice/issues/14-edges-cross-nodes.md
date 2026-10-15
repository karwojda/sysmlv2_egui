# 14: Straight edges can run behind other nodes

**What to build:** After nodes are moved by hand, an edge drawn straight between two node centers can pass behind a third node, or leave a node at almost the same point as another edge, so their markers overlap. Seen while verifying ticket 09: in a saved `Structure` Diagram, `vehicle → Vehicle` (defined-by) runs behind `Tank`, and the `Vehicle → Tank` composition diamond sits under that edge's triangle.

Two directions, possibly both:
1. **Let the user route edges:** drag on an edge to add or move a bend point, double-click a bend to remove it. Layout already stores bends per edge (`Layout::edges`, ticket 06), so this is UI plus an `Edit` variant.
2. **Route around nodes automatically:** e.g. orthogonal routing that avoids node rects, or at least spread edges that share a border point.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Decide: manual bends, automatic avoidance, or both (user: both)
- [x] (if manual) A bend can be added, moved and removed on the canvas, is saved in the Diagram File, and is honoured by the CLI Export
- [x] (if automatic) No edge passes through a node rect in the golden diagrams, and saved bends still take precedence

## Comments

- Done, both.
- **Automatic (core, `notation::detour` and `spread_ends`):** an edge that would cross a node gets one bend, or two to get out of a pocket. Candidates are sideways offsets from the middle and points just off obstacle corners. The shortest route that crosses nothing wins, else the one crossing fewest (only if that beats the straight line). Ends crowding the same spot on a node's side are spread 14px apart. Saved routes skip both. No existing golden changed, because automatic layouts never needed it. A test places nodes by hand in the arrangement that showed the problem and asserts no edge passes through any node.
- **Manual (viewer):**
  - Drag an edge's line to add a bend there and move it. Drag a bend to move it. Double-click a bend to remove it.
  - Right-click an edge: "Reset route" (back to automatic).
  - The hovered edge is highlighted with its bend handles.
  - Dragging an automatically bent edge turns its bends into saved ones.
  - Core: `Edit::Route { edge, bends }` (empty = automatic). `Scene::edges()` gives each edge's path.
  - Grab priority (`canvas::grab_at`, unit-tested): resize handle, bend, node, edge line, pan.
- Bend dragging: see the user verification below.
- **Verified by the user by hand (2026-10-15):** bending edges works.
