# 08: General Diagram on the canvas, plus GUI Export

**What to build:** In the viewer, the user picks a package or definition and opens a General Diagram for it on a canvas, painted from the scene by an egui renderer, with pan and zoom. The user can Export the current Diagram to SVG, and the result is byte-identical to the CLI output for the same Diagram.

**Blocked by:** 01: Diagram pipeline tracer; 07: Viewer shell

**Status:** done

- [x] The user can open a General Diagram for a package or definition selected in the Browser
- [x] The egui renderer paints the scene (nodes, compartments, keywords, edges, end markers), matching the SVG visually
- [x] The canvas supports pan and zoom
- [x] The egui renderer maps a pointer position to the element under it, for later interaction tickets
- [x] Exporting from the viewer writes an SVG byte-identical to the CLI Export of the same Diagram and Layout
- [x] Several Diagrams can be open at once, in tabs or an equivalent

## Comments

- Done. Double-click a package or definition in the Browser (or use its context menu, "Open General Diagram") to open a tab. `sysml-viewer-gui <folder> <root>` opens one directly. Opening an already-open root switches to its tab.
- The canvas (`viewer_app::canvas`) paints the scene's primitives on a white page, like the Export.
  - Pan: drag or scroll. Zoom: ctrl+scroll or pinch, about the pointer, 10%–800%. "Fit" re-fits the view.
  - Hovering highlights a node and shows its qualified name; a click selects it.
  - The default egui font has no bold face, so bold text is overprinted.
- `Scene::element_at` (core) maps a point to the topmost node. `DiagramKind::accepts_root` decides what can be a General Diagram's root (packages and definitions).
- Export writes `svg::render` of the same scene. `tests/export.rs` proves it's byte-identical to `sysml-viewer export --diagram` for the same Diagram.
- When a Model reloads, open Diagrams are rebuilt and keep their view. Opening a different folder closes them.
- Verified by eye via the screenshot example (`.../structure ... Structure`). Not exercised by hand: drag, zoom, hover and double-click. The screenshot tool can't send input.
