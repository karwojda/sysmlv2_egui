# 03: Styles in the viewer, with per-Diagram overrides

**What to build:** The viewer paints styled Diagrams, and `style.json` reloads live. A Diagram File can carry a `style` block (`model_style`, `rules`) whose rules win over the Model's, or which switches the Model Style off. Both the viewer and the CLI honour it. Unsaved Diagrams and views without a file use the Model Style. A broken Style file shows as a warning in the viewer. See `.scratch/styles/spec.md`.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] The canvas paints fills and text colours like the SVG
- [ ] Editing `style.json` restyles the open Diagrams live
- [ ] The Diagram File `style` block round-trips, and old files still load
- [ ] Diagram rules beat Model rules; `model_style: false` drops the Model's
- [ ] CLI Export honours the Diagram's style block
