# 02: Constraint Results

**What to build:** On every load, the backend checks each top-level part usage with `validate_instance`. Each `constraint` verdict becomes a Constraint Result `{ constraint, part, result }`, with the part taken from the validated root plus the verdict's instance path.

- **Results:** Holds, Violated (with the condition), Undecided (with the reason), or Unchecked, with one Model warning when a part can't be validated.
- **Marks:** each `assert constraints` row ends with its part's result mark. Constraints inherited from a definition appear on the part as `^{…}` rows. Asserted usage nodes get a badge with their worst result.
- **Properties:** the panel explains each result.

See `.scratch/constraints/spec.md`.

**Blocked by:** 01

**Status:** done

- [x] Results mapped by constraint and instance path; a failed check gives Unchecked and a warning (unit-tested, so CI covers it)
- [x] Row marks, inherited `^` rows on usages, and def rows without marks
- [x] Badges on asserted usages only
- [x] Properties shows the result, failing condition or reason, and the part
- [x] Contract test of results on the `assertions` fixture; goldens for parts with assertions and for constraint nodes

## Comments

- Done.
  - **Model:** each asserted constraint carries `checks: Vec<ConstraintCheck { part, result }>`, reusing `ClaimResult`. An inherited constraint (`Tank::fits`) holds its result for each part that inherits it (`vehicle::tank`), so no Model-wide list was needed.
  - **`viewer_core::constraint`:** `ConstraintStatus` (Holds, Undecided, Violated) with marks, `status` (an asserted usage's worst check, Undecided if never checked), `checks_on(part)`, `has_violation(part)` (ready for ticket 03), `row_text` and `describe`.
  - **Backend (`viewer-opensysml/src/constraints.rs`):**
    - One `validate_instance` per top-level part usage, and only when the Model asserts any constraint.
    - `constraint` verdicts go to their constraint, on the root or on the part at the dotted instance path.
    - An evaluation error becomes Undecided with the service's reason. A failed validation marks the tree's own asserted constraints Unchecked and adds one warning.
    - The matching is pure, with 4 unit tests that run in CI.
  - **Notation:**
    - A part's own asserted rows end with its result mark, ? if never checked. Inherited results follow as `^{…} ✗` rows. A definition's rows carry no mark.
    - Asserted usage nodes wear the badge; plain ones and parts don't.
  - **Properties:**
    - **A part:** "Asserts" rows with marks, plus the failing condition or reason.
    - **A constraint:** "Expression", "Asserted", and a "Result" row per part.
  - **Tests:** the twin carries the service's exact results, confirmed by the contract test. 14 tests in `constraints.rs`, and the golden `constraints-checked.svg`.
