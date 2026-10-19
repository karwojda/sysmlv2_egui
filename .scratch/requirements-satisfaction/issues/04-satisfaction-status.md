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

**Status:** done

- [x] Results are matched to claims, by requirement and satisfier, else by order
- [x] A failed check gives a warning and marks every claim Unchecked
- [x] Status derivation is tested for: no claims, all holding, one violated, undecided, unchecked
- [x] Badges and marks appear in the viewer and SVG, and the canvas font has the marks
- [x] Properties shows the status details
- [x] The contract test covers held, violated, undecided and not-claimed usages, including the inside-a-part case (Undecided while the service gap lasts)
- [x] Goldens: a requirement tree with statuses, and parts with `«satisfy»` edges

## Comments

- Done.
  - **Model:** claims carry `claim_result: Option<ClaimResult>`, which is `Holds`, `Violated { condition }`, `Undecided { reason }` or `Unchecked { reason }`.
    - `viewer_core::satisfaction::status(model, requirement)` derives a requirement usage's status: Not claimed with no claims, otherwise the worst result. Unchecked and missing results count as Undecided.
    - It returns `None` for anything that isn't a requirement usage.
    - The module also gives `mark()` / `label()`, `claim_status` and `describe_claim`.
  - **Backend (`viewer-opensysml/src/satisfaction.rs`):** one `verify_satisfaction(None, …)` per load, and only when the Model has claims.
    - Verdicts go to claims by requirement and satisfier. When the service names no satisfier (the inside-a-part form), the next claim on that requirement in declaration order takes it.
    - An evaluation error becomes Undecided with the service's text.
    - A claim with no verdict is Unchecked, and a failed check makes every claim Unchecked plus one Model warning.
    - The matching is a pure function with 5 unit tests that run in CI.
  - **Notation:**
    - **Badge:** an unfilled 16 px circle with the mark in the top-right corner of requirement usages, never definitions. The node widens by the badge on both sides so the title stays centred and clear of it.
    - **Styles:** the badge is a node's second rect, so a Style fill shows through it and the mark turns white on a dark fill like other inner text.
    - **Marks:** on `«satisfy» ✓` labels, on `satisfy massReq ✓` rows and on nested requirement rows (`<2> rangeReq:RangeTarget ✗`).
  - **Properties:**
    - **Requirement usage:** "Status" (`✗ Violated`), "Satisfied by" with each claim's mark, and per claim "Failing condition" or "Undecided because".
    - **Satisfier:** "Satisfies" rows with marks.
    - **Claim:** a "Result" row.
  - **Fixture:** `car` now satisfies its own `<5> carMassReq`, so the fixture covers every status: massReq ✓, rangeReq ✗, costReq ?, carMassReq ? (the service gap), spareReq ○. The twin carries the service's exact results, and the contract test confirms them.
  - **ADR-0001** gains a consequence for the verification input.
  - **Checked by eye:** the goldens, and a GUI screenshot showing the egui font draws ✓ ✗ ? ○.
