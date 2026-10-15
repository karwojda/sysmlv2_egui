# 07: Viewer shell: open a Model, Browser Diagram, diagnostics

**What to build:** The egui/eframe desktop viewer. The user opens a folder as a Model, and the viewer starts the sidecar and loads the Model. It shows the Browser Diagram (the package and element tree) and a diagnostics list.

**Blocked by:** 02: OpenSysML backend and sidecar manager

**Status:** done

- [x] The user can open a folder as a Model from the viewer
- [x] The viewer starts the sidecar on open and stops it on exit
- [x] The Browser Diagram shows the Model tree, with each entry's name and kind
- [x] Unsupported Elements are listed in the Browser with their kind
- [x] Entries can be expanded and collapsed
- [x] A name filter narrows the Browser to matching entries and their ancestors
- [x] A diagnostics list shows Model errors with file and line
- [x] Browser content (tree, filter) is built in the core and tested through the fake backend. The egui layer only paints it

## Comments

- Done. Crate `viewer-app`, binary `sysml-viewer-gui [model-folder]` (eframe 0.37). The Model is opened from a path field or the command line. There's no native folder picker yet: rfd needs GTK or an xdg portal, and neither is reliable on WSL.
- Models load on a worker thread that owns the backend. Dropping the app joins the worker and drops the backend, which stops the sidecar. `tests/loader.rs` covers this.
- Browser content is `viewer_core::browser::tree`, tested through the fake backend. Top-level entries start open, and while filtering every entry is open.
- The standard-library toggle was dropped: library elements never reach the Model (see ticket 02).
- Verified by eye with `cargo run -p viewer-app --example screenshot -- <folder> <out.png>`. On this WSL machine it needs `WAYLAND_DISPLAY=/mnt/wslg/runtime-dir/wayland-0`.
- The backend fix this ticket uncovered: one file with a syntax error made the service refuse the API JSON conversion for **every** file. Now each file falls back to converting on its own, so only the broken file loses its source lines, ends and documentation. It also doesn't repeat that file's error as a warning.
- Diagnostics show file paths relative to the Model folder.
