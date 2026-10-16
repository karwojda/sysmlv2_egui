# 03: General Diagram over an element set, with parts trees

**What to build:** A General Diagram can be drawn from an explicit set of elements, as a resolved view supplies it. Tool-defined Diagrams pass their root's owned members and stay unchanged. When a node's owner is also drawn, a composition edge joins them, and the node is left out of the owner's compartments. That way `expose vehicle::**` reads as a parts tree. See `.scratch/model-defined-diagrams/spec.md`.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] The General builder accepts an explicit node set, and existing General goldens are unchanged
- [ ] Usage-to-usage composition edges are drawn when both ends are nodes
- [ ] A drawn child isn't repeated in its owner's compartments
- [ ] Layout edits (move, hide, route) work on an element-set Diagram
- [ ] Golden SVG of a tree view
