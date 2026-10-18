# 04: Satisfaction Status

**What to build:** On every load and live reload, the backend asks OpenSysML to check all claims (`verify_satisfaction`) and stores each claim's result.

- **Results:** each claim gets `Holds`, `Violated { condition }`, `Undecided { reason }`, or `Unchecked` when the check fails. A failed check also adds one Model warning.
- **Status:** a requirement usage's Satisfaction Status is derived: Not claimed when it has no claims, otherwise the worst result.
- **Marks:**
  - badges (✓ ✗ ? ○) in the corner of requirement usages, not definitions;
  - the claim's mark on its `«satisfy»` edge label and in the part's compartment row;
  - marks on nested requirement rows.
- **Properties:** "Status", "Failing condition" or "Undecided because", and a result on each link.

See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] Results are matched to claims, by requirement and satisfier, else by order
- [ ] A failed check gives a warning and marks every claim Unchecked
- [ ] Status derivation is tested for: no claims, all holding, one violated, undecided, unchecked
- [ ] Badges and marks appear in the viewer and SVG, and the canvas font has the marks
- [ ] Properties shows the status details
- [ ] The contract test covers held, violated, undecided and not-claimed usages, including the inside-a-part case (Undecided while the service gap lasts)
- [ ] Goldens: a requirement tree with statuses, and parts with `«satisfy»` edges
