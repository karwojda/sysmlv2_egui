# 04: The CLI exports through the Open Diagram; edits only go through it

**What to build:** The CLI's `render_to` opens the Diagram File as an Open Diagram and Exports it. The core's `Editing` test harnesses are deleted and their tests drive an Open Diagram. `editing::apply` and `apply_with` become `pub(crate)`. The app's Export of an unsaved Diagram suggests `default_name()`. See `.scratch/open-diagram/spec.md`.

**Blocked by:** 03

**Status:** done

- [x] `render_to` goes through the Open Diagram
- [x] One test keeps the CLI's Export identical to the Open Diagram's
- [x] No `Editing` harness left in `viewer-core/tests`
- [x] `editing::apply_with` is `pub(crate)`; `editing::apply` is deleted (nothing called it)
- [x] The app's Export and Save suggest the same name

## Comments

- Done. `render_to` opens the Diagram File with `OpenDiagram::open_saved` and calls `export_svg`; the CLI holds its Model in an `Arc`. One wording change: failing to create the output folder now says "cannot write <out>: …" rather than "cannot create <folder>: …" (no test pinned it). The app's `export.rs` is split: the CLI-vs-Open-Diagram byte-identity test is `viewer-cli/tests/open_diagram_export.rs`, the other two are `viewer-core/tests/open_diagram_export.rs`, and `viewer-app` no longer dev-depends on `viewer-cli`. Both `Editing` harnesses are gone: `editing.rs` and `interconnection_editing.rs` use small `open`/`bounds`/`part`/`port`/`saved` helpers over an Open Diagram, and tests that seeded a Layout by hand open a Diagram File with it instead. `view_diagram.rs`'s move-and-hide test also goes through an Open Diagram. The app's Export path now uses `default_name()`, as saving does (new test in `saving.rs`). Test counts per file are unchanged (20 and 12); workspace 631 passed, and the 50 service-backed `--ignored` tests pass.
