# 02: Constraint Results

**What to build:** On every load, the backend checks each top-level part usage with `validate_instance`. Each `constraint` verdict becomes a Constraint Result `{ constraint, part, result }`, with the part taken from the validated root plus the verdict's instance path.

- **Results:** Holds, Violated (with the condition), Undecided (with the reason), or Unchecked, with one Model warning when a part can't be validated.
- **Marks:** each `assert constraints` row ends with its part's result mark. Constraints inherited from a definition appear on the part as `^{…}` rows. Asserted usage nodes get a badge with their worst result.
- **Properties:** the panel explains each result.

See `.scratch/constraints/spec.md`.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Results mapped by constraint and instance path; a failed check gives Unchecked and a warning (unit-tested, so CI covers it)
- [ ] Row marks, inherited `^` rows on usages, and def rows without marks
- [ ] Badges on asserted usages only
- [ ] Properties shows the result, failing condition or reason, and the part
- [ ] Contract test of results on the `assertions` fixture; goldens for parts with assertions and for constraint nodes
