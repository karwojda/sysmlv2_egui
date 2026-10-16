# 02: Model Style in Exports

**What to build:** A Model Style in `diagrams/style.json` colours node fills in every Exported Diagram.

- **Rules:** each Style Rule matches by kind, package or metadata.
- **Precedence:** the most specific wins: metadata beats package, which beats kind. Within a level the deeper package wins, or for metadata the first listed rule.
- **Scene:** the scene and the SVG carry fill colours and text colours. General nodes, Interconnection parts and port squares are all coloured. Text inside a dark fill turns white.
- **CLI:** the CLI applies the Model Style. A broken Style file gives a warning and is ignored.

See `.scratch/styles/spec.md`.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Style and Style Rule types, with a pretty-printed Style file round trip
- [ ] Matching covers each condition, specificity, the deeper package, the first metadata rule, and rules naming missing elements
- [ ] Applying a Style recolours only node fills and their inner text; positions, strokes and connectors are unchanged
- [ ] SVG renders the colours, and unstyled goldens are unchanged
- [ ] CLI Export uses `diagrams/style.json`; a broken file warns and is ignored
- [ ] Goldens for a styled General Diagram and a styled Interconnection Diagram
