# 02: The Open Diagram holds its Model

**What to build:** The Open Diagram holds an `Arc<Model>` and the Model Style. `open`, `open_saved` and `reload` take them together, and the other methods stop taking `&Model`, so a Scene is always built from the Model it holds and is never drawn unstyled. See `.scratch/open-diagram/spec.md`.

**Blocked by:** 01

**Status:** done

- [x] `reload(model, model_style)` replaces `refresh` and `set_model_style`
- [x] `edit`, `undo`, `redo` and `set_style` take no Model
- [x] Opening draws with the Model Style at once
- [x] Undo history survives a reload
- [x] The app holds the Model in an `Arc` and reloads every open Diagram with it

## Comments

- Done. `OpenDiagram::open(model, model_style, source)` and `open_saved(model, model_style, file)` replace the three old constructors; `reload(model, model_style)` replaces `refresh` and `set_model_style`, and the private `redraw` does the rebuild. `watch::Shown` now holds an `Arc<Model>`, so a stale Model kept after a failed reload is the same `Arc` the Diagrams hold. The style panel's live edits to the Model Style reload each Diagram with the same Model. The core tests open Diagrams through `common::open_tool`; the app's tests have a local copy. New tests: `a_diagram_opens_already_colored_by_the_model_style`, `undo_history_survives_a_model_reload`.
