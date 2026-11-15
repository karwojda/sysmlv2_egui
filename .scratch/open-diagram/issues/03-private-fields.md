# 03: The Open Diagram's fields are private

**What to build:** Every field of the Open Diagram is private, behind the reads in the spec. The Selection changes only through `select_only`, `toggle_selected`, `select_in` and `clear_selection`, which use the current Scene. `Scene::movable(key)` and `Scene::container_of(key)` replace the `movable`/`container` helpers and the app's inline copy. See `.scratch/open-diagram/spec.md`.

**Blocked by:** 02

**Status:** done

- [x] No `pub` fields on `OpenDiagram`
- [x] The app reads through accessors and changes the Selection through the four methods
- [x] `Scene::movable` and `Scene::container_of` exist; nothing outside `scene.rs` and `editing.rs` reads `origins` to guess the Diagram Kind
- [x] Edits and Selection changes do nothing while the Diagram can't be drawn; saving still works (tested)

## Comments

- Done. All fields are private; the app reads `source()`, `kind()`, `layout()`, `style()`, `scene()`, `selection()`, `name()` and `is_dirty()`. `Selection` keeps only its reads public, so it changes only through `select_only`, `toggle_selected`, `select_in` and `clear_selection`. The canvas used to change the Selection while holding the Scene for the whole frame; it now collects `Pick`s while handling input and applies them before painting and the context menu, so nothing lags a frame. `Scene::movable` and `Scene::container_of` replace the free helpers and the app's inline copy; only `interconnection.rs` (which fills it), `scene.rs` and `editing.rs`'s `pin` touch `origins` now. The Selection tests drive an Open Diagram instead of a bare `Selection`. New test: `a_diagram_that_cannot_be_drawn_ignores_edits_and_selections_but_still_saves`.
