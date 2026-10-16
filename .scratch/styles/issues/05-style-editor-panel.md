# 05: Style editor panel

**What to build:** A Style panel with "Model" and "This Diagram" tabs.

- **Rules:** add, remove, move up and move down rules. Each rule has a condition picker (kind; package or metadata def from the Model), a colour picker, and a label field.
- **Diagram switches:** the Diagram tab also has "Use Model Style" and "Legend".
- **Saving:** Model edits write `style.json` straight away. Diagram edits mark the Diagram unsaved.
- **Missing elements:** rules naming elements no longer in the Model are kept and marked.

See `.scratch/styles/spec.md`.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] The rule-list operations (add, remove, move, set) are tested as plain functions
- [ ] The panel edits the Model Style and saves `style.json`
- [ ] The panel edits the Diagram Style and marks the Diagram unsaved
- [ ] Rules naming missing elements are marked
- [ ] Checked by hand with a screenshot
