# 01: Move the Open Diagram into the core

**What to build:** Move `OpenDiagram`, `Selection`, `movable`, `container`, `UNDO_STEPS` and `check_name` from `viewer-app/src/canvas.rs` into `viewer_core::open_diagram`, with the same behaviour. `View`, `needs_fit` and `fitted_to` stay in the app, which wraps the Open Diagram with them. `title()` stays in the app. See `.scratch/open-diagram/spec.md`.

**Blocked by:** None

**Status:** done

- [x] `viewer_core::open_diagram` exists, and `OpenDiagram` and `Selection` are re-exported from the crate root
- [x] The app keeps its viewing state beside the Open Diagram, not in it
- [x] `save_as` and `export_svg` return `OpenDiagramError`
- [x] The app's `undo`, `selection` and `saving` tests (and `styles`, `views`) run in `viewer-core`; `export` waits for 04
- [x] The app behaves as before

## Comments

- Done. `viewer_core::open_diagram` holds `OpenDiagram`, `Selection`, `OpenDiagramError`, `movable`, `check_name` and `UNDO_STEPS`. The app wraps each one in a `canvas::Tab` with `View`, `needs_fit`, `fitted_to` and `title()`; `ViewerApp.diagrams` is now `tabs`, and the tab-bar method became `tab_bar` so the two names don't collide. `DiagramError` derives `Clone` so `NotDrawn` can carry it. Tests moved to `viewer-core/tests/{undo,selection,saving,open_diagram_styles,open_views}.rs`; their `title()` asserts now check `name`/`dirty`, and the titles have their own test in `viewer-app/tests/tabs.rs`. `export.rs` stays in the app until 04 because it needs `viewer_cli`.
