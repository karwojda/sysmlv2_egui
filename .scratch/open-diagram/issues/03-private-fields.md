# 03: The Open Diagram's fields are private

**What to build:** Every field of the Open Diagram is private, behind the reads in the spec. The Selection changes only through `select_only`, `toggle_selected`, `select_in` and `clear_selection`, which use the current Scene. `Scene::movable(key)` and `Scene::container_of(key)` replace the `movable`/`container` helpers and the app's inline copy. See `.scratch/open-diagram/spec.md`.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] No `pub` fields on `OpenDiagram`
- [ ] The app reads through accessors and changes the Selection through the four methods
- [ ] `Scene::movable` and `Scene::container_of` exist; nothing outside `scene.rs` and `editing.rs` reads `origins` to guess the Diagram Kind
- [ ] Edits and Selection changes do nothing while the Diagram can't be drawn; saving still works (tested)

## Comments
