# 05: Right-angled routing around parts

**What to build:** Interconnection connectors never pass through a box they don't connect. Right-angled routes go around obstacles. Undirected ports don't produce the hooks seen in the spike, and connectors leaving the same side spread apart. This settles the spike's rough spots.

**Blocked by:** 01: Interconnection tracer; 03: Frame ports and part-level connection ends

**Status:** ready-for-agent

- [ ] No connector segment passes through a box other than its two ends' boxes (and the boxes enclosing them), checked over all golden fixtures, closed and opened
- [ ] Connectors leave and enter ports perpendicular to the border
- [ ] The drivetrain's `supply` connector (from the Frame's `power` port) routes around the fuel tank
- [ ] Saved bends still take precedence over automatic routing
