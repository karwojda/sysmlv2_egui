# 03: Constraints in Styles

**What to build:** Two Style Rule conditions:

- `{"constraint": "holds" | "violated" | "undecided"}` matches asserted constraint usage nodes by their worst result.
- `{"constraints": "violated"}` matches parts with a violated result of their own, inherited ones included but not nested parts.

Both rank with `satisfaction`. They come with default Legend labels ("Constraint violated", "Part with a violated constraint") and are offered in the Style editor's picker. See `.scratch/constraints/spec.md`.

**Blocked by:** 02

**Status:** ready-for-agent

- [ ] Both conditions match as specified and round-trip in Style files
- [ ] Rank above metadata
- [ ] Default Legend labels
- [ ] Offered in the editor's condition picker
