# 11: Live reload

**What to build:** The viewer watches the Model folder and reloads when `.sysml`/`.kerml` files change, so the user can edit in a text editor and see the result alongside. A reload with errors keeps the last good Model for drawing and updates the diagnostics list.

**Blocked by:** 07: Viewer shell

**Status:** done

- [x] Saving a change to a Model file updates the Browser and open Diagrams without user action
- [x] Adding or deleting Model files is picked up
- [x] Bursts of changes (for example an editor save that writes several files) trigger a single reload
- [x] A reload with errors keeps showing the last good Diagrams and updates the diagnostics list. The next good reload replaces them
- [x] Open Diagrams keep their Layout across reloads, with merge rules from 06 when available
- [x] Changes to Diagram Files made outside the viewer don't trigger a Model reload
- [x] The keep-last-good behaviour is tested through the core with the fake backend

## Comments

- Done. It polls rather than using inotify, because inotify doesn't fire for Windows drives (`/mnt/c`) under WSL, where the user's models live. `viewer_app::watcher::Watcher` fingerprints the Model files (path, modification time, size) every 500ms and reloads once a change has held for one more poll, so a burst of saves gives one reload.
- Core `viewer_core::watch`:
  - `model_files` is the single rule for which files make up a Model (`.sysml`/`.kerml`, recursively). The OpenSysML backend now uses it too.
  - `fingerprint` and `ChangeDetector` decide when to reload.
  - `accept(previous, loaded)` keeps the last good Model drawn (`stale`) while the new one has errors, until a good reload.
- Diagram Files aren't Model files, so saving one never triggers a reload.
- Viewer: open Diagrams are rebuilt with their Layout on every reload. While stale, a banner says the files have errors and the last good Model is shown. Diagnostics always list the latest load's problems. The watcher is dropped before the loader, so the loader's request channel closes on exit.
- Tests: 10 in `viewer-core/tests/watch.rs`; 4 in `viewer-app/tests/watcher.rs`, which drive the real thread against a temp folder (about 4s).
- Verified on screen: broke `structure.sysml` while the viewer was open. The banner appeared, the last good Diagram and Browser stayed, and the new error was listed.
