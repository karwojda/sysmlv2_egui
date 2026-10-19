# 02: Constraint text

**What to build:** A requirement's `require` constraints are rebuilt as text from the API JSON expression tree and shown in a `constraints` compartment (`require {v.mass <= 2000.0}`), plus a "Requires" row in Properties.

- **Covered:** comparisons, arithmetic, `and` / `or` / `not` / `xor` / `implies`, feature chains, feature references, and literals (including unit-qualified ones where the tree shows them).
- **Fallback:** anything else is `{…}`. Parentheses are added only where precedence needs them.
- **`assume`:** read but not shown.

See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** 01

**Status:** done

- [x] The `require` text is rebuilt for each covered construct, with the `{…}` fallback
- [x] Minimal parentheses by precedence
- [x] The `constraints` compartment and the Properties "Requires" row
- [x] The contract twin matches the real backend's constraint text

## Comments

- Done.
  - **Rebuilder (`viewer-opensysml/src/expression.rs`):** builds text from the API JSON trees.
    - Covers operator expressions (binary, unary `not` / `-`, and `[` for units), feature chains, feature references, integer, rational, string and boolean literals, `*`, `null`, and invocations. An invocation's function is named by its declared name, or by the name it was written with when unresolved.
    - Parentheses appear only where precedence or associativity needs them (`(v.a + v.b) * 2`, `v.a - (v.b - 1)`).
    - Anything unknown, or a dangling reference, becomes `…`.
  - **Model:** `Element::require` holds the text of each `require` constraint a requirement owns, in declaration order. A constraint with no result expression (one given by reference) shows `…`. `assume` constraints are skipped.
  - **Notation:** a `constraints` compartment (`require {…}`) after `attributes` (or after `doc`, or first), hidden when collapsed. Properties gets a "Requires" row per constraint.
  - **Units read in full:** library elements arrive without their short names, so a unit reads as its full name, `1650 [kilogram]`, not `[kg]` as written. It's still correct and unambiguous; fixing it would need the service to send short names for library elements.
  - **Tests:**
    - 7 unit tests of the rebuilder in-crate, so CI covers them.
    - A `constraints` fixture with a contract test of every probed form against the real service.
    - The `requirements` twin carries constraint texts.
    - `requirements.rs` tests for placement, collapsing and Properties, plus the golden `requirements-package.svg`.
