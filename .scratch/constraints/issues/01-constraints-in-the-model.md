# 01: Constraints in the Model, and drawn

**What to build:** Constraint definitions and usages stop being Unsupported Elements.

- **Model:** constraint defs and usages carry `is_asserted`, their own expression as text, and their parameters (owned `in` features) with any bound values.
- **Part compartments:** parts and part defs list their own asserted constraints in an `assert constraints` compartment (`{mass <= 1500.0}`), with no marks yet.
- **Nodes:**
  - constraint defs are drawn as `«constraint def»` with `parameters` and `expression` compartments;
  - constraint usages are drawn as `«assert constraint»` or `«constraint»` `name:Def` with bound parameters.

See `.scratch/constraints/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `ElementKind::ConstraintDefinition` / `ConstraintUsage`; `is_asserted` and `expression` facts read by the backend
- [x] Parameters with direction and type; bound parameter values on usages
- [x] The `assert constraints` compartment on parts and part defs (after `attributes`, before `requirements`)
- [x] Constraint def and usage nodes with `parameters` and `expression` compartments
- [x] An `assertions` `.sysml` fixture and its fake twin pass a contract test

## Comments

- Done.
  - **Model:**
    - `ElementKind::ConstraintDefinition` (a definition) and `ConstraintUsage` (a General node), plus `is_asserted` and `expression`.
    - The service's Model data says `AssertConstraintUsage` for asserted usages, while the symbol tree calls both kinds `constraintUsage`.
    - The expression is the constraint's result expression, rebuilt as text.
    - Parameters are `in` reference features: Unsupported Elements with a direction and type, as they arrive.
    - A usage's bindings (`in numerator = mass`) are the values on its same-named parameters.
  - **Backend fix:** names generated for unnamed elements are now resolved recursively, so attributes inside a redefinition inside another (`part :>> tank { attribute :>> fuelMass = 50.0; }`) get their values. They used to get none.
  - **Notation:**
    - `assert constraints` and `constraints` compartments on parts and part defs, after `attributes`. Rows read `{expression}`, or `name:Def` for a usage of a definition.
    - Constraint nodes have `parameters` (in source order where known, since parameters are positional) and `expression` compartments.
    - `«assert constraint»` is the keyword of asserted usages.
  - **Tests:** the `assertions` fixture with its twin and a contract test; `viewer-core/tests/constraints.rs` (8); the goldens `constraints-package.svg` and `constraints-usage.svg`.
