# 01: Move the Open Diagram into the core

**What to build:** Move `OpenDiagram`, `Selection`, `movable`, `container`, `UNDO_STEPS` and `check_name` from `viewer-app/src/canvas.rs` into `viewer_core::open_diagram`, with the same behaviour. `View`, `needs_fit` and `fitted_to` stay in the app, which wraps the Open Diagram with them. `title()` stays in the app. See `.scratch/open-diagram/spec.md`.

**Blocked by:** None

**Status:** ready-for-agent

- [ ] `viewer_core::open_diagram` exists, and `OpenDiagram` and `Selection` are re-exported from the crate root
- [ ] The app keeps its viewing state beside the Open Diagram, not in it
- [ ] `save_as` and `export_svg` return `OpenDiagramError`
- [ ] The app's `undo`, `selection`, `saving` and `export` tests run in `viewer-core`
- [ ] The app behaves as before

## Comments
