# 01: Computed Values

**What to build:** The part check (`validate_instance`) runs for every top-level part usage on every load. Each instance's feature values become Computed Values on our elements: the value of feature `f` on an instance of type symbol `T` belongs to `T::f`. A part with several instances keeps the distinct values.

- **Rows:** expression-valued rows append ` = <computed>` (`mass = powertrain.mass + … = 1363.0`); literal rows stay as written.
- **Properties:** a "Computed" row.
- **Failure:** if the check fails, rows keep their expressions, with the usual warning.

See `.scratch/analysis/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Every top-level part is checked, whether or not constraints are asserted
- [x] Instance values mapped to elements; distinct values kept for multi-instance parts
- [x] Values formatted like literals
- [x] Rows and Properties show computed values; literal rows unchanged
- [x] An `analysis` `.sysml` fixture (roll-up parts to start with) and its twin pass a contract test including computed values

## Comments

- Done.
  - **Every part checked:** the part check (`validate_instance`) runs for every top-level part usage on every load, not only when constraints are asserted. One call yields Constraint Results and Computed Values.
  - **Mapping:** instance feature values go to `<type symbol>::<feature>`, kept distinct and in first-seen order. Values equal to the element's own written value are skipped, since `mass = 1800.0` computing to 1800.0 says nothing new. That also left existing twins valid.
  - **Formatting:** like literals (`1363.0`, `4`, `true`, `"x"`, `1650 [kg]`, the unit as written). Instance references and sequences are dropped.
  - **Display:** rows whose value is an expression append ` = <computed>`, distinct values joined with `, `. Literal rows don't, and neither do rows without a computed value. Properties shows "Computed".
  - **Side effects:** constraint parameters now show their bound values' results (`in numerator = mass = 1400.0`). The demo pictures show the totals (`= 1363.0`, `= 265.0`) and were refreshed.
  - **Tests:** the `analysis` fixture (a car rolling up to 230.0) with its twin and a contract test; the `assertions` twin's computed parameters; backend unit tests for mapping and formatting; `viewer-core/tests/computed.rs` (5).
