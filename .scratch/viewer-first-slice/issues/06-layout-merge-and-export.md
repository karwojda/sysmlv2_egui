# 06: Layout merge and Layout-aware Export

**What to build:** Diagram Files carry a real Layout: node positions and sizes, edge routes, hidden elements and collapsed elements, keyed by element qualified name. When a Diagram is built, its saved Layout is merged with the current Model. Saved positions are kept as fixed points, elements without a saved position are auto-placed around them, and entries that match no element are kept as Orphaned Layout Entries. CLI Exports honour the Layout. ADR-0002 applies: the Model is never written.

**Blocked by:** 03: CLI `export --diagram` from a Diagram File

**Status:** done

- [x] The Diagram File Layout holds positions, sizes, edge routes, hidden elements, collapsed elements and Orphaned Layout Entries, one entry per element with stable ordering, so diffs stay reviewable
- [x] Elements with saved positions and sizes are drawn exactly there
- [x] Elements added to the Model since the Layout was saved are auto-placed without moving saved elements
- [x] Layout entries whose qualified name matches nothing become Orphaned Layout Entries, are not drawn, and are preserved when the Diagram File is written back
- [x] Hidden elements are left out of the scene, and collapsed elements are drawn without compartments
- [x] Saved edge routes are used, and edges without a saved route are auto-routed
- [x] CLI `--diagram` and `--all` Exports reflect the Layout
- [x] Pipeline tests cover each merge case: saved, new, orphaned, hidden, collapsed and routed

## Comments

- Done. `Layout` (in `diagram_file`) holds:
  - `nodes`: qualified name → `{x, y, width, height}`
  - `edges`: edge key → bend points `[[x, y], ...]`
  - `hidden` and `collapsed`: sets of qualified names

  Empty sections are left out of the file, and everything is sorted, so diffs stay clean. Edge keys come from `Depicts::edge_key`, e.g. `specialization P::Car P::Vehicle` or `connection P::link`.
- `build_scene_with_layout` applies it. `build_scene` is the same with an empty Layout, so every existing golden file is byte-identical.
- Placement: saved nodes are drawn exactly as saved. Every other node is auto-laid out among themselves (keeping the edges between them) and placed below the saved ones, so nothing saved moves and nothing new overlaps. The scene grows to fit saved nodes. Coordinates are assumed non-negative; ticket 09 should clamp drags at 0.
- Routes: an edge leaves its source's border toward its first bend and enters its target's border from its last bend. Markers and labels use the end segments. A connection label sits on its middle segment.
- Orphaned Layout Entries are never deleted, so they survive a save round trip. `Layout::orphaned(&model)` lists those in `nodes`, `hidden` and `collapsed`. Edge keys aren't checked for orphans; they're only ignored when nothing matches.
- The CLI's `--diagram` and `--all` both honour the Layout (tested in `viewer-cli/tests/export.rs`).
