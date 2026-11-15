# 04: The CLI exports through the Open Diagram; edits only go through it

**What to build:** The CLI's `render_to` opens the Diagram File as an Open Diagram and Exports it. The core's `Editing` test harnesses are deleted and their tests drive an Open Diagram. `editing::apply` and `apply_with` become `pub(crate)`. The app's Export of an unsaved Diagram suggests `default_name()`. See `.scratch/open-diagram/spec.md`.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] `render_to` goes through the Open Diagram
- [ ] One test keeps the CLI's Export identical to the Open Diagram's
- [ ] No `Editing` harness left in `viewer-core/tests`
- [ ] `editing::apply` and `apply_with` are `pub(crate)`
- [ ] The app's Export and Save suggest the same name

## Comments
