# 03: Frame ports and part-level connection ends

**What to build:** The root's own ports are drawn on the Frame's border, and connections from inner parts to them are drawn. Connections whose end is a part itself (no port) attach to that part's border, on the side facing the other end.

**Blocked by:** 01: Interconnection tracer

**Status:** ready-for-agent

- [ ] The root's ports sit on the Frame border: `in` top, `out` bottom, undirected facing their inner peer
- [ ] A connection from a part's port to a root port is drawn to the Frame border
- [ ] A connection ending at a part (chain of one feature) attaches to that part's border, facing the other end
- [ ] Seam 1 tests for both, and the drivetrain's `power` supply connection in its golden file
