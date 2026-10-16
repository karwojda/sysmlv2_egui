# 05: Style editor panel

**What to build:** A Style panel with "Model" and "This Diagram" tabs.

- **Rules:** add, remove, move up and move down rules. Each rule has a condition picker (kind; package or metadata def from the Model), a colour picker, and a label field.
- **Diagram switches:** the Diagram tab also has "Use Model Style" and "Legend".
- **Saving:** Model edits write `style.json` straight away. Diagram edits mark the Diagram unsaved.
- **Missing elements:** rules naming elements no longer in the Model are kept and marked.

See `.scratch/styles/spec.md`.

**Blocked by:** 03

**Status:** done

- [x] The rule-list operations (add, remove, move, set) are tested as plain functions
- [x] The panel edits the Model Style and saves `style.json`
- [x] The panel edits the Diagram Style and marks the Diagram unsaved
- [x] Rules naming missing elements are marked
- [ ] Checked by hand with a screenshot

## Comments

- Done. The last box (a hands-on check) is left for the user. A screenshot shows the panel rendering, but clicking through it can't be scripted here.
  - **Core:** `style::KINDS` (the kinds a rule can name), `Condition::exists_in`, `move_up` / `move_down` and `new_rule`, all tested in `style.rs`.
  - **App (`style_panel::rule_list`):** each row has a colour button, a condition type (Kind, Package or Metadata) and a value picked from the Model's kinds, packages or `metadata def`s. A ⚠ marks a rule whose package or def is missing. There's a label field whose hint shows the automatic Legend label, ↑ / ↓ / ✕ buttons, and "Add rule".
  - **Right panel:** now has Properties and Style tabs.
    - **Model tab:** restyles every open Diagram at once, and writes `diagrams/style.json` as each change finishes. Label edits are written when the field loses focus, so typing doesn't trigger a reload per keystroke. A broken file is replaced when edited here.
    - **This Diagram tab:** "Use Model Style" and "Legend" switches plus its own rules, as unsaved changes.
  - **Screenshots:** the example's `SCREENSHOT_STYLE=1` opens the Style tab.

- [ ] Checked by hand (user)
