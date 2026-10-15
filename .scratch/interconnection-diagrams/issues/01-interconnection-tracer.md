# 01: Interconnection tracer: Frame, closed parts, ports, connectors → SVG

**What to build:** The thinnest end-to-end path for the new Diagram Kind. `DiagramKind::Interconnection` (`"interconnection"` in Diagram Files) rooted at a part definition, or a part usage typed by one, builds a scene with:
- the Frame
- the root's parts as closed rounded boxes, layered top to bottom
- their ports on the borders by direction (`in` top, `out` bottom, undirected facing the peer)
- connections and interfaces as right-angled connectors between the ports they join

The CLI Exports it from a Diagram File. See the spec at `.scratch/interconnection-diagrams/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `DiagramKind::Interconnection` exists, serialises as `interconnection`, and `accepts_root` takes part defs and part usages typed by a part def
- [x] The Frame is labelled `«part def» Name` for a definition root, or `«part» name:Type` for a usage root, and holds the definition's parts
- [x] Each part is a rounded box labelled `«part» name:Type [mult]`, with its type's ports as squares on its border, labelled `name : [~]Type`
- [x] Ports: `in` on the top edge, `out` on the bottom, undirected on the side facing the other end of their connection (bottom if unconnected), spaced evenly in peer order
- [x] Parts are layered top to bottom, with edges from the `out` end to the `in` end
- [x] Connections and interfaces between two parts' ports are right-angled connectors labelled `name:Type`
- [x] Scene items depict parts and ports as elements and connectors as their connection, so hit-testing works
- [x] A root that isn't a part fails like a missing root
- [x] `sysml-viewer export` Exports a Diagram File of kind `interconnection`
- [x] Seam 1 tests for the above, and a golden SVG of the `structure` fixture's `Vehicle`

## Comments

- Done ahead of breakdown approval, while the user was away: its design was fully settled by the spike (variant E) and Q1–Q7. The other tickets stay `needs-triage` until the user approves the breakdown.
- Core module `viewer_core::interconnection`, dispatched from `build_scene_with_layout` for `DiagramKind::Interconnection`. It produces the same scene as General Diagrams, so SVG Export, the canvas, hit-testing and golden tests work unchanged.
- `DiagramKind::accepts_root` now gates every kind, so a root of the wrong kind fails with `DiagramError::UnsuitableRoot`. Interconnection takes a part def, or a part usage (which must be typed by a part def).
- Ports are depicted as `Depicts::Port { part, port }`, since two parts of the same type share port names. Parts and the Frame are `Depicts::Element`.
- Layout: the existing top-to-bottom layered layout, on part sizes padded by one label line above and below. Edges run from each wire's `out` end to its `in` end.
- Connectors are right-angled. Ports nearly in line get a straight connector instead of a 1px jog. Labels go beside the longest run, and stacks of labels over the same ports are centred on it.
- Already in, ahead of later tickets: stacked labels (planned for 02) and closed parts widened for their port labels (02).
- Tests: 13 in `viewer-core/tests/interconnection.rs` (golden `vehicle-interconnection.svg`), and 2 in `viewer-cli/tests/export.rs`.
- The viewer only got the tab title for the new kind. Opening Interconnection Diagrams from the UI is ticket 07.
