# 04: Reference parts dashed

**What to build:** Reference parts (`ref part`) are drawn as dashed rounded boxes in Interconnection and General Diagrams. The scene's rectangle gains a dashed style, honoured by the SVG and egui renderers. The Properties panel shows "Reference: yes" for reference parts.

**Blocked by:** 01: Unnamed connections, flows and bindings in the Model

**Status:** done

- [x] `Shape::Rect` can be dashed; SVG writes it with a dash array, and the canvas draws it dashed
- [x] Reference part boxes are dashed in Interconnection Diagrams, and reference part usage nodes in General Diagrams
- [x] The Properties panel lists "Reference: yes" for reference parts
- [x] Seam 1 tests and the golden file updated

## Comments

- Done. `Shape::Rect` gains `dashed`. The SVG writes `stroke-dasharray="6 4"`, and the canvas fills the box and then draws its rounded outline as a dashed path (egui has no dashed rectangle).
- Reference parts (`Element::is_reference`) are dashed in Interconnection Diagrams (never the Frame) and as usage nodes in General Diagrams. The Properties panel lists "Reference: yes".
- The golden `flows-interconnection.svg` was updated (`driver` dashed). Other golden files are unchanged.
- The canvas rendering of the dashed outline gets checked on screen with ticket 05.
