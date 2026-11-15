# Spec: The Open Diagram lives in the core

Status: done

## Problem Statement

"Rebuild the styled Scene after a Layout edit" is written three times: in the app's `OpenDiagram` (`viewer-app/src/canvas.rs`), in the CLI's `render_to` (`viewer-cli/src/lib.rs`), and in the `Editing` harnesses of `viewer-core/tests/editing.rs` and `interconnection_editing.rs`. The orphan-aware `known` closure, Style layering, pinning before an edit, undo and the Selection's `keep_drawn` are each wired by hand. `viewer-app/tests/export.rs` has to assert that the app and CLI write the same bytes because they share no module. Undo and Selection hold no egui code, yet they can only be tested through the app crate.

## Solution

Move the **Open Diagram** (see `CONTEXT.md`) into `viewer_core::open_diagram`, as a deep module: a small interface over building, styling, editing, undoing, selecting, saving and exporting one Diagram. The app keeps only how it is viewed (`View`, `needs_fit`, `fitted_to`) and the tab `title()`. The CLI's Export goes through it too. Nothing users see changes, except that the app's Export suggests the same lowercased name as saving does.

## Implementation Decisions

- **Module:** `viewer_core::open_diagram` holds `OpenDiagram`, `Selection`, `OpenDiagramError`, `check_name` and `UNDO_STEPS`; `OpenDiagram` and `Selection` are re-exported from the crate root.
- **Model:** it holds an `Arc<Model>`. `open(model, model_style, source)`, `open_saved(model, model_style, file)` and `reload(model, model_style)` take the Model and the Model Style together (the app always loads them together), so no Scene is ever drawn unstyled. Undo history survives a reload.
- **Interface:** every field is private.
  - Reads: `source`, `kind`, `layout`, `style`, `scene() -> Result<&Scene, &DiagramError>`, `selection`, `name`, `is_dirty`, `default_name`, `is_interconnection_of`, `can_undo`, `can_redo`.
  - Changes: `edit`, `begin_drag`/`end_drag`, `undo`/`redo`, `set_style`, and the Selection's `select_only`, `toggle_selected`, `select_in`, `clear_selection`, which always use the current Scene.
  - Files: `save_as(folder, name)` and `export_svg(path)` do the I/O and return `OpenDiagramError` (`NameNotAllowed`, `NameTaken`, `NotDrawn(DiagramError)`, `Io`).
- **Undrawable Diagrams:** while the Scene is an error, edits and Selection changes do nothing; saving still works.
- **Scene queries:** `Scene::movable(key)` and `Scene::container_of(key)` replace the app's `movable`/`container` helpers and its inline copy, with today's implementation behind them (candidate #2 of the architecture review later changes only that).
- **Edit seam:** `editing::apply` and `apply_with` become `pub(crate)`; the Open Diagram is the only way to change a Layout. `Edit`, `Alignment`, `Dimension`, `Axis` and `MIN_NODE_SIZE` stay public.
- **CLI:** `render_to` opens the Diagram File as an Open Diagram and Exports it.
- **Naming:** the app's Export of an unsaved Diagram suggests `default_name()` (lowercased), like saving.

## Testing Decisions

- The app's `undo`, `selection`, `saving` and `export` tests become `viewer-core` tests against the Open Diagram. The app keeps tests only for `View`, `grab_at` and `nearest_side`.
- Both `Editing` harnesses are deleted; their tests drive an Open Diagram.
- Notation tests that build Scenes directly, and the golden SVGs, stay as they are.
- One test keeps the CLI's Export identical to the Open Diagram's.

## Out of Scope

- Scene answering the Interconnection structure itself (architecture candidate #2) beyond the two queries above.
- Edit availability per Diagram Kind (candidate #3), and trimming the scene builders (candidate #6).

## Issues

1. `issues/01-move-open-diagram-to-core.md`
2. `issues/02-hold-the-model.md`
3. `issues/03-private-fields.md`
4. `issues/04-cli-and-edit-seam.md`
