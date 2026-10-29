# 02: Align, same size and distribute

**What to build:** The node context menu offers Align (left, right, top, bottom, horizontal centres, vertical centres) and Same size (width, height, both) against the right-clicked node. It also offers Distribute (horizontally, vertically) with equal gaps, where the outermost nodes stay put. See `.scratch/selection/spec.md` (Arrange commands).

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Right-clicking a node outside the Selection replaces the Selection with that node
- [ ] Align and Same size need 2+ selected nodes, and Distribute needs 3+; otherwise the entries are greyed out
- [ ] Align moves the other nodes on one axis only; the reference stays put
- [ ] Same size keeps each node's top-left, skips collapsed nodes, and is greyed out when the reference is collapsed
- [ ] Distribute keeps the order and makes the gaps between boxes equal
- [ ] Each command is a single `Edit` in `editing.rs`; results may overlap, and positions are clamped at zero
- [ ] Unit tests for every variant, including nodes inside a box in an Interconnection Diagram
