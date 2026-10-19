# 05: Satisfaction in Styles

**What to build:** A `satisfaction` Style Rule condition (`satisfied` / `violated` / `undecided` / `not-claimed`) matches requirement usages by status and ranks above metadata. Its default Legend labels are `Satisfied`, `Violated`, `Undecided` and `Not claimed`, and the Style editor offers it in the condition picker. See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** 04

**Status:** done

- [x] The condition matches by status, ranks above metadata, and round-trips in the Style file
- [x] Default Legend labels
- [x] Offered in the editor's condition picker

## Comments

- Done.
  - **Condition:** `Condition::Satisfaction(Satisfaction)`, written `{"satisfaction": "violated"}` in Style files, with values `satisfied` / `violated` / `undecided` / `not-claimed`. Rank 3, above metadata.
  - **Matching** needs the Model (a status comes from a requirement's claims), so `Condition::matches` and `style::rule_for` now take the Model too. The rule only matches requirement usages with that status.
  - **Legend:** default labels are the status names. A satisfaction rule is never marked ⚠ missing.
  - **Editor:** the condition picker offers "Satisfaction", with a status picker showing each mark.
  - **Tests:** 7 in `satisfaction.rs` (matching, rank over metadata, Style file round trip, Legend labels, badge mark turning white on a dark fill), and the golden `requirements-status-styled.svg`, a traffic-light tree with Legend.
