# 02: The Open Diagram holds its Model

**What to build:** The Open Diagram holds an `Arc<Model>` and the Model Style. `open`, `open_saved` and `reload` take them together, and the other methods stop taking `&Model`, so a Scene is always built from the Model it holds and is never drawn unstyled. See `.scratch/open-diagram/spec.md`.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] `reload(model, model_style)` replaces `refresh` and `set_model_style`
- [ ] `edit`, `undo`, `redo` and `set_style` take no Model
- [ ] Opening draws with the Model Style at once
- [ ] Undo history survives a reload
- [ ] The app holds the Model in an `Arc` and reloads every open Diagram with it

## Comments
