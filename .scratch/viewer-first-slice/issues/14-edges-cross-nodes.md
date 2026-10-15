# 14: Straight edges can run behind other nodes

**What to build:** After nodes are moved by hand, an edge drawn straight between two node centers can pass behind a third node, or leave a node at almost the same point as another edge, so their markers overlap. Seen while verifying ticket 09: in a saved `Structure` Diagram, `vehicle → Vehicle` (defined-by) runs behind `Tank`, and the `Vehicle → Tank` composition diamond sits under that edge's triangle.

Two directions, possibly both:
1. **Let the user route edges:** drag on an edge to add or move a bend point, double-click a bend to remove it. Layout already stores bends per edge (`Layout::edges`, ticket 06), so this is UI plus an `Edit` variant.
2. **Route around nodes automatically:** e.g. orthogonal routing that avoids node rects, or at least spread edges that share a border point.

**Blocked by:** None (can start immediately)

**Status:** needs-triage

- [ ] Decide: manual bends, automatic avoidance, or both
- [ ] (if manual) A bend can be added, moved and removed on the canvas, is saved in the Diagram File, and is honoured by the CLI Export
- [ ] (if automatic) No edge passes through a node rect in the golden diagrams, and saved bends still take precedence
