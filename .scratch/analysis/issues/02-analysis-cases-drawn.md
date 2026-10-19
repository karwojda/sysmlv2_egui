# 02: Analysis cases in the Model, and drawn

**What to build:** Analysis definitions and usages stop being Unsupported Elements.

- **Facts:** each carries its subject and the part it is bound to (`v = vehicle`), its objective (doc and `require` text) and its parameters (in, out and `return`, with values).
- **Node:** drawn as on slide 99: `«analysis def»` / `«analysis» name:Def` with `subject`, `objective`, `parameters` and `attributes` compartments.
- **Edge:** a dashed `«subject»` edge with an open arrowhead runs from an analysis to its subject part when both are drawn, keyed `subject <analysis>`.

See `.scratch/analysis/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `ElementKind::AnalysisDefinition` / `AnalysisUsage` with subject binding, objective, `return` flag
- [ ] Analysis nodes with their four compartments
- [ ] The `«subject»` edge, with saved routes applying
- [ ] The `analysis` fixture gains an analysis def and usage; its twin and contract test match
