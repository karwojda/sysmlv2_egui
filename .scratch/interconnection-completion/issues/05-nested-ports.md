# 05: Nested ports

**What to build:** A port whose type owns ports is drawn as a bar along its box's border, long enough for its nested ports, which are small squares on the bar's outer edge, labelled `name : Type` outside. Nested ports are keyed by feature path (`engine.fuelIn.inner`). Connection ends resolve down the chain to the nested square, or stop at the bar for the composite port itself. One level of nesting; deeper nested ports draw as plain squares.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] A composite port is a bar along the border; its nested ports are squares on its outer edge with labels
- [ ] The bar's length fits its nested ports and their labels; the box grows to fit the bar
- [ ] An end at `part.port.inner` attaches to the inner square; an end at `part.port` attaches to the bar
- [ ] Nested ports are selectable (port keys `part.port.inner`); dragging moves the composite port's bar as one, and nested ports keep their order on it
- [ ] Seam 1 tests and a golden file; no overlapping labels
