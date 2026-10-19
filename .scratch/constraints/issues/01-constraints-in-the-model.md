# 01: Constraints in the Model, and drawn

**What to build:** Constraint definitions and usages stop being Unsupported Elements.

- **Model:** constraint defs and usages carry `is_asserted`, their own expression as text, and their parameters (owned `in` features) with any bound values.
- **Part compartments:** parts and part defs list their own asserted constraints in an `assert constraints` compartment (`{mass <= 1500.0}`), with no marks yet.
- **Nodes:**
  - constraint defs are drawn as `«constraint def»` with `parameters` and `expression` compartments;
  - constraint usages are drawn as `«assert constraint»` or `«constraint»` `name:Def` with bound parameters.

See `.scratch/constraints/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `ElementKind::ConstraintDefinition` / `ConstraintUsage`; `is_asserted` and `expression` facts read by the backend
- [ ] Parameters with direction and type; bound parameter values on usages
- [ ] The `assert constraints` compartment on parts and part defs (after `attributes`, before `requirements`)
- [ ] Constraint def and usage nodes with `parameters` and `expression` compartments
- [ ] An `assertions` `.sysml` fixture and its fake twin pass a contract test
