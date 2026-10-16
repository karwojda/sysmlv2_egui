# 04: Opening views in the viewer

**What to build:** The viewer can open and save Model-defined Diagrams.

- A Views list sits beside saved Diagrams. Unsupported Views are greyed out, with their reason as a tooltip.
- Double-clicking a view in the Browser opens it.
- A Model-defined Diagram can be edited and saved like any other. Saving writes a model-defined Diagram File (`name`, `view`, `layout`), named after the view by default. The kind and contents are resolved again on load and on live reload.
- A file whose view has gone missing or become unsupported opens with a message and isn't modified.
- Entries for elements that are no longer exposed become Orphaned Layout Entries.

See `.scratch/model-defined-diagrams/spec.md`.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] Diagram File has a model-defined form; old tool-defined files still load; round-trip test
- [ ] A Views list opens supported views and explains unsupported ones
- [ ] Browser double-click opens a view
- [ ] Save, reload and live reload pick up changes to the view's exposes and rendering
- [ ] A missing view gives a message, and the file is untouched
- [ ] Golden SVG of an interconnection view
