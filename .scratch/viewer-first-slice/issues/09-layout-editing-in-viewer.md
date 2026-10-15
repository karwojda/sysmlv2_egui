# 09: Layout editing and saving Diagrams in the viewer

**What to build:** The user adjusts a Diagram on the canvas and saves it. Interactions become editing commands applied to the open Diagram's Layout in the core: move, resize, hide, show again, collapse, expand and reset to automatic. The egui layer only translates input into commands. The user saves a Diagram under a name as a Diagram File, sees saved Diagrams listed when opening the Model, can keep ad hoc Diagrams unsaved, and is warned about unsaved changes.

**Blocked by:** 06: Layout merge and Layout-aware Export; 08: General Diagram on the canvas

**Status:** ready-for-agent

- [ ] Dragging a node moves it, and resizing a node changes its size
- [ ] An element can be hidden from a Diagram, and hidden elements can be shown again
- [ ] A node's compartments can be collapsed and expanded
- [ ] Reset returns the Diagram to the automatic arrangement
- [ ] Save as writes a named Diagram File; save updates it
- [ ] Saved Diagrams are listed when a Model is opened and can be reopened
- [ ] Opening a Diagram without saving it creates no file
- [ ] An unsaved-changes indicator is shown, and closing a Diagram or the viewer with unsaved changes asks for confirmation
- [ ] The viewer never writes `.sysml` or `.kerml` files
- [ ] Each editing command is tested through the core: apply the command, then assert on the resulting scene and the Diagram File written
