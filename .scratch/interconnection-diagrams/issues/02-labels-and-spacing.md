# 02: Room for labels: wide-enough boxes, layer spacing, stacked connector labels

**What to build:** Every label in an Interconnection Diagram is readable. Closed parts are wide enough for the labels of their top and bottom ports. Layers leave a label line of room. Connections over the same ports stack their labels. Connector labels keep clear of port labels and box titles. Adds the spike's drivetrain (after slide 40) as a fake-backend fixture with a golden file.

**Blocked by:** 01: Interconnection tracer

**Status:** needs-triage

- [ ] A closed part's width fits its title and the labels of its top-edge ports side by side, and the same for its bottom edge
- [ ] Layers are separated by at least one label line beyond the port labels
- [ ] Connections joining the same pair of ports have stacked, non-overlapping labels
- [ ] No label overlaps another label or a box title (tested by estimated text extents)
- [ ] The drivetrain fixture (fuel tank, engine, transmission, driveshaft, rear axle assembly, controller) has a golden SVG, closed
