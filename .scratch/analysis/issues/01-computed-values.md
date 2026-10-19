# 01: Computed Values

**What to build:** The part check (`validate_instance`) runs for every top-level part usage on every load. Each instance's feature values become Computed Values on our elements: the value of feature `f` on an instance of type symbol `T` belongs to `T::f`. A part with several instances keeps the distinct values.

- **Rows:** expression-valued rows append ` = <computed>` (`mass = powertrain.mass + … = 1363.0`); literal rows stay as written.
- **Properties:** a "Computed" row.
- **Failure:** if the check fails, rows keep their expressions, with the usual warning.

See `.scratch/analysis/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] Every top-level part is checked, whether or not constraints are asserted
- [ ] Instance values mapped to elements; distinct values kept for multi-instance parts
- [ ] Values formatted like literals
- [ ] Rows and Properties show computed values; literal rows unchanged
- [ ] An `analysis` `.sysml` fixture (roll-up parts to start with) and its twin pass a contract test including computed values
