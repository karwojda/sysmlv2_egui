# 04: Full first-slice language subset in General Diagrams

**What to build:** General Diagrams draw the whole first-slice language subset in the Standard Notation. That means packages and imports; part, attribute and port definitions and usages; typing and multiplicity on usages; port direction; composition (a definition owning usages); and connections and interfaces with `connect`. Unsupported Elements are left out of General Diagrams. The Model backend interface, the fake backend and the OpenSysML backend grow to supply the extra facts.

**Blocked by:** 01: Diagram pipeline tracer; 02: OpenSysML backend and sidecar manager

**Status:** done

- [x] The Model backend interface exposes type, multiplicity, port direction, connection and interface ends, imports and documentation
- [x] `attribute def`/`attribute`, `port def`/`port`, and `part` usages are drawn with Standard Notation keywords and compartments
- [x] Usages show their type and multiplicity
- [x] Ports show their direction (`in` / `out` / `inout`)
- [x] Composition is drawn as an edge in the Standard Notation
- [x] Connections and interfaces between usages are drawn as edges
- [x] Elements reached through imports resolve correctly in Diagrams
- [x] Unsupported Elements never appear in a General Diagram
- [x] Golden SVG tests cover each construct, and Unsupported Element exclusion, through the fake backend
- [x] The contract fixtures are extended so the OpenSysML backend matches the fake backend for every new construct

## Comments

- Done. Element facts come from the symbol tree plus the API JSON conversion (see ADR-0001's consequences). That also fills in element **source lines**, closing the gap ticket 02 recorded, so ticket 10 is unblocked on that front.
- Notation choices (Standard Notation, simplified):
  - Definitions have square corners and usages rounded ones.
  - Compartments, in this order: `attributes`, `ports`, `parts`, `connections`. Each row reads `[direction] name:[~]Type [multiplicity]`, and connection rows add `(a.b → c.d)`.
  - Composition is a filled diamond at the owner, labelled `name [mult]` near the part's definition.
  - Defined-by (usage node to its definition) is a solid line, a hollow triangle and a colon `:`.
  - Package-level connections between usage nodes are plain labelled lines.
  - Connections owned by definitions are listed in the `connections` compartment, not drawn. Drawing them belongs to the Interconnection Diagram.
- Typing edge style was resolved against the user's notation reference (Intro to the SysML v2 Language — Graphical Notation, slides 12 and 32). Defined-by is a **solid** line with a hollow triangle and a colon across the line before it. Usage labels are `name:Type`, without spaces and not bold.
- `end x : PortDef` inside a `connection def` is a *reference* end (an Unsupported Element), not a port. Interface-def ends are ports.
- Layout: connections now take part in layering, so connected usages sit together. Unconnected groups wrap into rows at 900px.
- Edges now run center to center and are clipped to node borders. This changed the vehicles golden file (diagonal edges only).
