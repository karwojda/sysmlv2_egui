# 02: Constraint text

**What to build:** A requirement's `require` constraints are rebuilt as text from the API JSON expression tree and shown in a `constraints` compartment (`require {v.mass <= 2000.0}`), plus a "Requires" row in Properties.

- **Covered:** comparisons, arithmetic, `and` / `or` / `not` / `xor` / `implies`, feature chains, feature references, and literals (including unit-qualified ones where the tree shows them).
- **Fallback:** anything else is `{…}`. Parentheses are added only where precedence needs them.
- **`assume`:** read but not shown.

See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] The `require` text is rebuilt for each covered construct, with the `{…}` fallback
- [ ] Minimal parentheses by precedence
- [ ] The `constraints` compartment and the Properties "Requires" row
- [ ] The contract twin matches the real backend's constraint text
