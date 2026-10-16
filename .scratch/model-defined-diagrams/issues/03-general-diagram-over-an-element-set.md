# 03: General Diagram over an element set, with parts trees

**What to build:** A General Diagram can be drawn from an explicit set of elements, as a resolved view supplies it. Tool-defined Diagrams pass their root's owned members and stay unchanged. When a node's owner is also drawn, a composition edge joins them, and the node is left out of the owner's compartments. That way `expose vehicle::**` reads as a parts tree. See `.scratch/model-defined-diagrams/spec.md`.

**Blocked by:** 02

**Status:** done

- [x] The General builder accepts an explicit node set, and existing General goldens are unchanged
- [x] Usage-to-usage composition edges are drawn when both ends are nodes
- [x] A drawn child isn't repeated in its owner's compartments
- [x] Layout edits (move, hide, route) work on an element-set Diagram
- [x] Golden SVG of a tree view

## Comments

- Done.
  - **Builder:** `build_general_scene(model, elements, layout)` draws exactly the given elements (those General draws as nodes). It draws connections owned by the nodes or their owners. Tool-defined General Diagrams pass the root's owned members and the root's connections, and their goldens are unchanged.
  - **Parts trees:** a node whose owner is drawn gets a composition edge from the owner (a filled diamond with no label, since the child's node names it), keyed like other compositions (`composition <owner> <usage>`) so it can be routed. It is also left out of the owner's compartments.
  - **API:** `view::build_scene(model, view, layout)` draws any supported view, and `view::layout_key_exists` gives the "known" test for editing and Orphaned Layout Entries.
  - **Tests:** `viewer-core/tests/view_diagram.rs` and the golden `views-tree.svg`.
