# 03: Frame ports and part-level connection ends

**What to build:** The root's own ports are drawn on the Frame's border, and connections from inner parts to them are drawn. Connections whose end is a part itself (no port) attach to that part's border, on the side facing the other end.

**Blocked by:** 01: Interconnection tracer

**Status:** done

- [x] The root's ports sit on the Frame border: `in` top, `out` bottom, undirected facing their inner peer
- [x] A connection from a part's port to a root port is drawn to the Frame border
- [x] A connection ending at a part (chain of one feature) attaches to that part's border, facing the other end
- [x] Seam 1 tests for both, and the drivetrain's `power` supply connection in its golden file

## Comments

- Done. The builder is now a **box tree**: the Frame is the root's box, whose children are its parts. The root's ports are ordinary box ports, and wire ends are `End::Port { child: Option<usize>, port }` (`None` = the box's own port) or `End::Part { child }`. This sets up ticket 04: an Opened Part is a child box with children of its own. `Vehicle`'s golden SVG came out byte-identical through the rewrite.
- Frame ports: `in` on the top edge, `out` on the bottom, undirected facing their inner peer. Each lines up with the child port it's wired to, keeping ports on a side at least 20 apart, so its connector runs straight. Inside the box, flow from an own `in` port counts as an `out` end (and vice versa).
- Part-level ends attach to the middle of the child's side facing the other end.
- Fixes the clippy type-complexity warning from ticket 02 (wire ends are `End` now).
- Tests: 3 new in `viewer-core/tests/interconnection.rs`. The drivetrain golden now includes `power` and `supply`.
- Seen, for ticket 05: the straight `supply` connector runs through the fuel tank and through the Frame's title.
