# 08: General Diagram on the canvas, plus GUI Export

**What to build:** In the viewer, the user picks a package or definition and opens a General Diagram for it on a canvas, painted from the scene by an egui renderer, with pan and zoom. The user can Export the current Diagram to SVG, and the result is byte-identical to the CLI output for the same Diagram.

**Blocked by:** 01: Diagram pipeline tracer; 07: Viewer shell

**Status:** ready-for-agent

- [ ] The user can open a General Diagram for a package or definition selected in the Browser
- [ ] The egui renderer paints the scene (nodes, compartments, keywords, edges, end markers), matching the SVG visually
- [ ] The canvas supports pan and zoom
- [ ] The egui renderer maps a pointer position to the element under it, for later interaction tickets
- [ ] Exporting from the viewer writes an SVG byte-identical to the CLI Export of the same Diagram and Layout
- [ ] Several Diagrams can be open at once, in tabs or an equivalent
