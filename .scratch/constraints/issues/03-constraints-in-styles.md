# 03: Constraints in Styles

**What to build:** Two Style Rule conditions:

- `{"constraint": "holds" | "violated" | "undecided"}` matches asserted constraint usage nodes by their worst result.
- `{"constraints": "violated"}` matches parts with a violated result of their own, inherited ones included but not nested parts.

Both rank with `satisfaction`. They come with default Legend labels ("Constraint violated", "Part with a violated constraint") and are offered in the Style editor's picker. See `.scratch/constraints/spec.md`.

**Blocked by:** 02

**Status:** done

- [x] Both conditions match as specified and round-trip in Style files
- [x] Rank above metadata
- [x] Default Legend labels
- [x] Offered in the editor's condition picker

## Comments

- Done.
  - **`Condition::Constraint(ConstraintResult)`**, written `{"constraint": "holds" | "violated" | "undecided"}`, matches asserted constraint usages by `constraint::status`.
  - **`Condition::Constraints(PartConstraints::Violated)`**, written `{"constraints": "violated"}`, matches part usages with `constraint::has_violation`: their own checks, inherited included, nested parts not.
  - **Rank and Legend:** both rank 3, with `satisfaction`. Legend defaults are "Constraint holds / violated / undecided" and "Part with a violated constraint".
  - **Editor:** the picker offers "Constraint" (with a result picker) and "Part constraints".
  - **Tests:** 6 in `constraints.rs` (matching, own-only parts, rank over metadata, Style file round trip, Legend), and the golden `constraints-styled.svg`.
