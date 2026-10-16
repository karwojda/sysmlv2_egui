# 04: Reference parts dashed

**What to build:** Reference parts (`ref part`) are drawn as dashed rounded boxes in Interconnection and General Diagrams. The scene's rectangle gains a dashed style, honoured by the SVG and egui renderers. The Properties panel shows "Reference: yes" for reference parts.

**Blocked by:** 01: Unnamed connections, flows and bindings in the Model

**Status:** ready-for-agent

- [ ] `Shape::Rect` can be dashed; SVG writes it with a dash array, and the canvas draws it dashed
- [ ] Reference part boxes are dashed in Interconnection Diagrams, and reference part usage nodes in General Diagrams
- [ ] The Properties panel lists "Reference: yes" for reference parts
- [ ] Seam 1 tests and the golden file updated
