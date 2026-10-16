# 04: Opening views in the viewer

**What to build:** The viewer can open and save Model-defined Diagrams.

- A Views list sits beside saved Diagrams. Unsupported Views are greyed out, with their reason as a tooltip.
- Double-clicking a view in the Browser opens it.
- A Model-defined Diagram can be edited and saved like any other. Saving writes a model-defined Diagram File (`name`, `view`, `layout`), named after the view by default. The kind and contents are resolved again on load and on live reload.
- A file whose view has gone missing or become unsupported opens with a message and isn't modified.
- Entries for elements that are no longer exposed become Orphaned Layout Entries.

See `.scratch/model-defined-diagrams/spec.md`.

**Blocked by:** 03

**Status:** done

- [x] Diagram File has a model-defined form; old tool-defined files still load; round-trip test
- [x] A Views list opens supported views and explains unsupported ones
- [x] Browser double-click opens a view
- [x] Save, reload and live reload pick up changes to the view's exposes and rendering
- [x] A missing view gives a message, and the file is untouched
- [x] Golden SVG of an interconnection view

## Comments

- Done.
  - **Core:** `DiagramSource` is `Tool { kind, root }` or `View { view }`, flattened (untagged) into `DiagramFile`. Old files read as `Tool`, and new view files carry `"view"` with no kind or root. `DiagramSource::build_scene`, `layout_key_exists` and `kind` cover both forms. `DiagramError::View` carries view errors.
  - **Orphaned Layout Entries:** in a view's General Diagram, a key is "known" only while its element is exposed. So un-exposing something keeps its entries, even through Reset.
  - **Default name:** `view::file_name` makes a view's name safe for Diagram and Export file names (`vehicle tree` → `vehicle-tree`).
  - **App:**
    - `OpenDiagram` holds a `source` and the `kind` it last resolved to.
    - A "Views" list sits under Saved Diagrams. Unsupported Views are disabled, with the reason on hover.
    - Double-clicking a view in the Browser opens it; for an Unsupported View the status says why.
    - The Properties panel has an "Open view" button, and a view given as the startup root opens.
    - The tab title for an unsaved view is `<name> (view)`.
  - **Tests:** `viewer-app/tests/views.rs`, the core Diagram File round trip, and the golden `views-wiring.svg`. A GUI screenshot on the `views` fixture was checked by eye.
