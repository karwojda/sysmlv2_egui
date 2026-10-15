# 01: Interconnection tracer: Frame, closed parts, ports, connectors → SVG

**What to build:** The thinnest end-to-end path for the new Diagram Kind. `DiagramKind::Interconnection` (`"interconnection"` in Diagram Files) rooted at a part definition, or a part usage typed by one, builds a scene with:
- the Frame
- the root's parts as closed rounded boxes, layered top to bottom
- their ports on the borders by direction (`in` top, `out` bottom, undirected facing the peer)
- connections and interfaces as right-angled connectors between the ports they join

The CLI Exports it from a Diagram File. See the spec at `.scratch/interconnection-diagrams/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** needs-triage

- [ ] `DiagramKind::Interconnection` exists, serialises as `interconnection`, and `accepts_root` takes part defs and part usages typed by a part def
- [ ] The Frame is labelled `«part def» Name` for a definition root, or `«part» name:Type` for a usage root, and holds the definition's parts
- [ ] Each part is a rounded box labelled `«part» name:Type [mult]`, with its type's ports as squares on its border, labelled `name : [~]Type`
- [ ] Ports: `in` on the top edge, `out` on the bottom, undirected on the side facing the other end of their connection (bottom if unconnected), spaced evenly in peer order
- [ ] Parts are layered top to bottom, with edges from the `out` end to the `in` end
- [ ] Connections and interfaces between two parts' ports are right-angled connectors labelled `name:Type`
- [ ] Scene items depict parts and ports as elements and connectors as their connection, so hit-testing works
- [ ] A root that isn't a part fails like a missing root
- [ ] `sysml-viewer export` Exports a Diagram File of kind `interconnection`
- [ ] Seam 1 tests for the above, and a golden SVG of the `structure` fixture's `Vehicle`
