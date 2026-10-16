# 02: Model Style in Exports

**What to build:** A Model Style in `diagrams/style.json` colours node fills in every Exported Diagram.

- **Rules:** each Style Rule matches by kind, package or metadata.
- **Precedence:** the most specific wins: metadata beats package, which beats kind. Within a level the deeper package wins, or for metadata the first listed rule.
- **Scene:** the scene and the SVG carry fill colours and text colours. General nodes, Interconnection parts and port squares are all coloured. Text inside a dark fill turns white.
- **CLI:** the CLI applies the Model Style. A broken Style file gives a warning and is ignored.

See `.scratch/styles/spec.md`.

**Blocked by:** 01

**Status:** done

- [x] Style and Style Rule types, with a pretty-printed Style file round trip
- [x] Matching covers each condition, specificity, the deeper package, the first metadata rule, and rules naming missing elements
- [x] Applying a Style recolours only node fills and their inner text; positions, strokes and connectors are unchanged
- [x] SVG renders the colours, and unstyled goldens are unchanged
- [x] CLI Export uses `diagrams/style.json`; a broken file warns and is ignored
- [x] Goldens for a styled General Diagram and a styled Interconnection Diagram

## Comments

- Done.
  - **Scene:** `Fill::Color(Rgb)`, and `Shape::Text` gains `fill`: `Stroke` for black (unwritten in SVG, so unstyled goldens are byte-identical) or `Background` for white. `Rgb` parses and serializes `#rrggbb`, and `is_dark` uses relative luminance below 0.179. The canvas paints both.
  - **`viewer_core::style`:**
    - Types: `Condition` (`kind` / `package` / `metadata`), `StyleRule { condition, fill, label }` and `Style { format_version, rules }`.
    - File: `path_for`, `load` (a missing file gives `None`) and `to_text`.
    - `rule_for(element, levels)`: the first level with a match decides; within it, rank (metadata > package > kind), then package depth, then list order.
    - `apply(scene, model, levels)`: colours each node's first rect (General node, Interconnection part, port) and whitens the text inside a dark rect. It returns the rules used, for the Legend.
  - **CLI:** loads `diagrams/style.json` for `--diagram`, `--view` and `--all`. A broken file gives `warning: …; Exporting without the Model Style`.
  - **Tests:** `viewer-core/tests/style.rs`, `viewer-cli/tests/export_styled.rs`, and the goldens `styled-general.svg` and `styled-interconnection.svg`.
