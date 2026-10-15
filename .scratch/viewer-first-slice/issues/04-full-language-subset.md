# 04: Full first-slice language subset in General Diagrams

**What to build:** General Diagrams draw the whole first-slice language subset in the Standard Notation. That means packages and imports; part, attribute and port definitions and usages; typing and multiplicity on usages; port direction; composition (a definition owning usages); and connections and interfaces with `connect`. Unsupported Elements are left out of General Diagrams. The Model backend interface, the fake backend and the OpenSysML backend grow to supply the extra facts.

**Blocked by:** 01: Diagram pipeline tracer; 02: OpenSysML backend and sidecar manager

**Status:** ready-for-agent

- [ ] The Model backend interface exposes type, multiplicity, port direction, connection and interface ends, imports and documentation
- [ ] `attribute def`/`attribute`, `port def`/`port`, and `part` usages are drawn with Standard Notation keywords and compartments
- [ ] Usages show their type and multiplicity
- [ ] Ports show their direction (`in` / `out` / `inout`)
- [ ] Composition is drawn as an edge in the Standard Notation
- [ ] Connections and interfaces between usages are drawn as edges
- [ ] Elements reached through imports resolve correctly in Diagrams
- [ ] Unsupported Elements never appear in a General Diagram
- [ ] Golden SVG tests cover each construct, and Unsupported Element exclusion, through the fake backend
- [ ] The contract fixtures are extended so the OpenSysML backend matches the fake backend for every new construct
