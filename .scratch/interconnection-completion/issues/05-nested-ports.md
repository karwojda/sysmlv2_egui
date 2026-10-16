# 05: Nested ports

**What to build:** A port whose type owns ports is drawn as a bar along its box's border, long enough for its nested ports, which are small squares on the bar's outer edge, labelled `name : Type` outside. Nested ports are keyed by feature path (`engine.fuelIn.inner`). Connection ends resolve down the chain to the nested square, or stop at the bar for the composite port itself. One level of nesting; deeper nested ports draw as plain squares.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] A composite port is a bar along the border; its nested ports are squares on its outer edge with labels
- [x] The bar's length fits its nested ports and their labels; the box grows to fit the bar
- [x] An end at `part.port.inner` attaches to the inner square; an end at `part.port` attaches to the bar
- [x] Nested ports are selectable (port keys `part.port.inner`); dragging moves the composite port's bar as one, and nested ports keep their order on it
- [x] Seam 1 tests and a golden file; no overlapping labels

## Comments

- Done. `PortSlot::nested` holds a port's own ports (from its type, one level). A composite port is drawn as a bar straddling the border:
  - It is long enough for each nested port's square and label side by side on a top or bottom edge, or one per row on a side edge.
  - Nested ports are 8px squares on the bar's outer edge, labelled outside, with keys like `tank.fuelOut.inner`.
  - The composite port's own label sits just beyond the bar's end.
- Ends: `End::Port` gains `nested: Option<usize>`. Chains resolve to the nested square (`tank.fuelOut.inner`, exact), or to the bar (`tank.fuelOut`). That includes the Frame's own composite ports.
- Box width counts a composite port's bar plus its label.
- Viewer: grabbing a nested square grabs its composite port (`canvas::grab_at`, unit-tested), so the bar moves as one.
- Checked on screen with the real backend (`flows` fixture), including ticket 04's dashed outline on the canvas.
- Tests: 3 in `viewer-core/tests/flows.rs`, 1 in `viewer-app/src/canvas.rs`. The golden `flows-interconnection.svg` was updated.
