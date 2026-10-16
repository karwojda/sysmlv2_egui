# 03: Styles in the viewer, with per-Diagram overrides

**What to build:** The viewer paints styled Diagrams, and `style.json` reloads live. A Diagram File can carry a `style` block (`model_style`, `rules`) whose rules win over the Model's, or which switches the Model Style off. Both the viewer and the CLI honour it. Unsaved Diagrams and views without a file use the Model Style. A broken Style file shows as a warning in the viewer. See `.scratch/styles/spec.md`.

**Blocked by:** 02

**Status:** done

- [x] The canvas paints fills and text colours like the SVG
- [x] Editing `style.json` restyles the open Diagrams live
- [x] The Diagram File `style` block round-trips, and old files still load
- [x] Diagram rules beat Model rules; `model_style: false` drops the Model's
- [x] CLI Export honours the Diagram's style block

## Comments

- Done.
  - **Diagram File:** gains `style: DiagramStyle { model_style (default true), rules, legend }`, written after `layout` and left out while it's the default, so existing files are unchanged.
  - **`style::apply_styles(scene, model, model_style, diagram_style)`:** layers the Diagram's rules over the Model's, or uses the Diagram's alone when `model_style` is false.
  - **Live reload:** `diagrams/style.json` is part of the watch fingerprint, so editing it triggers the usual reload. The app re-reads it on every load and pushes it to each open Diagram.
  - **Viewer:**
    - `OpenDiagram` holds `style` and `model_style`. `set_model_style` restyles without marking the Diagram unsaved; `set_style` is an unsaved change. `save_as` writes the style block.
    - A broken Style file shows as a warning in the top bar, and Diagrams are drawn without it.
  - **CLI:** honours each Diagram File's style block.
  - **Tests:** core `diagram_file`, `style` and `watch`; app `styles.rs`; CLI `export_styled.rs`. A GUI screenshot was checked by eye.
