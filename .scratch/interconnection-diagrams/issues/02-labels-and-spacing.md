# 02: Room for labels: wide-enough boxes, layer spacing, stacked connector labels

**What to build:** Every label in an Interconnection Diagram is readable. Closed parts are wide enough for the labels of their top and bottom ports. Layers leave a label line of room. Connections over the same ports stack their labels. Connector labels keep clear of port labels and box titles. Adds the spike's drivetrain (after slide 40) as a fake-backend fixture with a golden file.

**Blocked by:** 01: Interconnection tracer

**Status:** done

- [x] A closed part's width fits its title and the labels of its top-edge ports side by side, and the same for its bottom edge
- [x] Layers are separated by at least one label line beyond the port labels
- [x] Connections joining the same pair of ports have stacked, non-overlapping labels
- [x] No label overlaps another label or a box title (tested by estimated text extents)
- [x] The drivetrain fixture (fuel tank, engine, transmission, driveshaft, rear axle assembly, controller) has a golden SVG, closed

## Comments

- Done.
  - The space under each part grows with the deepest stack of connector labels leaving it (connectors sharing their ports).
  - Stacks are spaced by the estimated label height, so even estimated boxes don't touch.
  - Labels on horizontal runs sit above the line; on vertical runs, beside it.
  - Closed parts were already widened for their port labels in ticket 01.
- `Shape::extent()` (public, in core) gives the same text estimate the layout uses. The test `no_two_labels_overlap` checks every pair of labels in `Vehicle` and the drivetrain.
- Shared fixture: `viewer-core/tests/common/mod.rs::drivetrain()`, after slide 40, the twin for ticket 06's `.sysml` fixture. Golden `drivetrain-interconnection.svg` (closed). `vehicle-interconnection.svg` was refreshed (taller gaps).
- Left for later tickets: the `engineStatus` hook between undirected side ports (05), and the Frame's `power` port with its `supply` connection (03).
