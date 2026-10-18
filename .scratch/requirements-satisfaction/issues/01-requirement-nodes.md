# 01: Requirement nodes

**What to build:** Requirements stop being Unsupported Elements.

- **Model:** requirement definitions (`requirement def`) and usages (`requirement`) carry their short id, subject and `doc`.
- **Notation:** General Diagrams draw them in the standard notation:
  - header `«requirement»` / `«requirement def»`, then `<id> name:Type`, then `subject v:Vehicle`;
  - `doc` and `attributes` compartments, with the subject feature not repeated.
- **Trees:** a requirement usage can be the root of a General Diagram. Nested requirement usages are listed in a `requirements` compartment (no status marks yet).

See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `ElementKind::RequirementDefinition` / `RequirementUsage`, plus `short_name` and `subject` facts read by the backend
- [ ] Requirement nodes drawn with header, subject line, `doc` and attributes compartments
- [ ] Requirement usages accepted as General Diagram roots, with the nested `requirements` compartment
- [ ] Properties shows "Short id" and "Subject"
- [ ] A `requirements` `.sysml` fixture and its fake twin pass a contract test
