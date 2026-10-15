# 13: Parallel connections overlap and their labels collide

**What to build:** When two or more connections or interfaces join the same pair of nodes, each edge and its label must be readable on its own. Reported by the user: in the General Diagram of `Vehicle` (fixture `structure`), `line:FuelLine` and `iface:FuelInterface` both join `tanks` and `engine`. They're drawn as the same straight line, and both labels sit at its midpoint on top of each other.

**Cause:** the notation draws every connection as a straight line between the two node borders, labelled at the midpoint. Edges between the same pair of nodes (connections, or a connection alongside a composition or defined-by edge) are never told apart.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Parallel edges between the same pair of nodes are fanned out (offset from each other, curved or bent), so each line is visible on its own
- [x] Each parallel edge's label sits next to its own line, and no two labels overlap
- [x] A single edge between two nodes is drawn as before, so existing golden files only change where parallel edges exist
- [x] Self-consistent in both renderers (the change is in the scene, not in egui or SVG)
- [x] A Seam 1 test with two connections between the same usages asserts the two edges' lines and labels don't coincide. A golden file of `Vehicle`'s General Diagram covers it visually.

## Comments

- Repro: `sysml-viewer-gui crates/viewer-opensysml/tests/fixtures/structure Structure::Vehicle`, or `export` a Diagram File rooted at `Structure::Vehicle`.
- Consider whether Layout (ticket 06) should be able to store a user-chosen route for such edges. Saved edge routes are already in 06's scope, so a fanned-out default plus manual routes may be enough.
- Done. Edges joining the same two nodes, with no saved route, fan out: each bends once at the middle, 36px apart (the middle one of an odd count stays straight). Connection labels sit beside their bend on the outer side (end-aligned on the left, start-aligned on the right, or above and below for horizontal pairs). Saved routes from ticket 06 take precedence over fanning.
- Second bug found while verifying: the page only covered nodes, so a label beside an edge could be cut off at the edge. The scene now has an origin (`Scene::x`/`y`, possibly negative) and covers everything drawn. SVG writes it as the `viewBox` origin, and the canvas and Fit honour it. Older golden files were unaffected.
- New golden: `vehicle-general.svg`. Checked by eye in both the SVG and the viewer.
