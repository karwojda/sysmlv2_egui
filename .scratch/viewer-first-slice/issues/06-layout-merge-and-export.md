# 06: Layout merge and Layout-aware Export

**What to build:** Diagram Files carry a real Layout: node positions and sizes, edge routes, hidden elements and collapsed elements, keyed by element qualified name. When a Diagram is built, its saved Layout is merged with the current Model. Saved positions are kept as fixed points, elements without a saved position are auto-placed around them, and entries that match no element are kept as Orphaned Layout Entries. CLI Exports honour the Layout. ADR-0002 applies: the Model is never written.

**Blocked by:** 03: CLI `export --diagram` from a Diagram File

**Status:** ready-for-agent

- [ ] The Diagram File Layout holds positions, sizes, edge routes, hidden elements, collapsed elements and Orphaned Layout Entries, one entry per element with stable ordering, so diffs stay reviewable
- [ ] Elements with saved positions and sizes are drawn exactly there
- [ ] Elements added to the Model since the Layout was saved are auto-placed without moving saved elements
- [ ] Layout entries whose qualified name matches nothing become Orphaned Layout Entries, are not drawn, and are preserved when the Diagram File is written back
- [ ] Hidden elements are left out of the scene, and collapsed elements are drawn without compartments
- [ ] Saved edge routes are used, and edges without a saved route are auto-routed
- [ ] CLI `--diagram` and `--all` Exports reflect the Layout
- [ ] Pipeline tests cover each merge case: saved, new, orphaned, hidden, collapsed and routed
