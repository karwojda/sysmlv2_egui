# 02: Align, same size and distribute

**What to build:** The node context menu offers Align (left, right, top, bottom, horizontal centres, vertical centres) and Same size (width, height, both) against the right-clicked node. It also offers Distribute (horizontally, vertically) with equal gaps, where the outermost nodes stay put. See `.scratch/selection/spec.md` (Arrange commands).

**Blocked by:** 01

**Status:** done

- [x] Right-clicking a node outside the Selection replaces the Selection with that node
- [x] Align and Same size need 2+ selected nodes, and Distribute needs 3+; otherwise the entries are greyed out
- [x] Align moves the other nodes on one axis only; the reference stays put
- [x] Same size keeps each node's top-left, skips collapsed nodes, and is greyed out when the reference is collapsed
- [x] Distribute keeps the order and makes the gaps between boxes equal
- [x] Each command is a single `Edit` in `editing.rs`; results may overlap, and positions are clamped at zero
- [x] Unit tests for every variant, including nodes inside a box in an Interconnection Diagram

## Comments

- Done. Three new edits: `Edit::Align`, `Edit::MatchSize` and `Edit::Distribute`, with `Alignment`, `Dimension` and `Axis`. They're offered as submenus of the node context menu, above Hide. Disabled entries say why on hover. Aligning and distributing leave manual bends where they are, as moving one node does. Not checked by hand in the GUI (no way to drive the context menu from the agent's shell). Tests: `viewer-core/tests/editing.rs` and `interconnection_editing.rs`.
