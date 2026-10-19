# 03: Satisfaction Claims and «satisfy» edges

**What to build:** Each `satisfy` in the Model becomes a Satisfaction Claim. The requirement usage is the one the `satisfy` subsets, or the end of its feature chain. The satisfier is its `by` subject, or else its owning part.

- **Edges:** claims are drawn in General Diagrams as `«satisfy»` edges (solid line, open arrowhead at the requirement), keyed `satisfy <claim>` so bends can be saved.
- **Parts:** parts gain a `requirements` compartment (`satisfy massReq`).
- **Properties:** "Satisfied by" links on a requirement and "Satisfies" links on a part.

This ticket covers structure only; statuses come in 04. See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** 01

**Status:** done

- [x] Claims are read for both forms (`satisfy R by x`, and `satisfy R;` inside a part)
- [x] `«satisfy»` edges are drawn when both ends are nodes, and saved routes apply
- [x] The part `requirements` compartment
- [x] The Properties links
- [x] The contract twin matches the claims

## Comments

- Done.
  - **Model:** a Satisfaction Claim is an unnamed element of kind `SatisfyRequirement` (`satisfy`) under the service's generated name (`Req::@8`, `Req::car::@1`). It has `satisfies` (the requirement usage) and `satisfier`.
    - The Browser names it `massReq by vehicle`.
    - `Model::claims()` lists claims in declaration order, sorted by the generated number, not as text, so `@10` comes after `@9`.
  - **Backend:** reads `SatisfyRequirementUsage` items.
    - The requirement is the end of `subsets`, following a feature chain for `vehicleSpec.massReq`.
    - The satisfier is the end of `subject` (`by x`), else the owner (the inside-a-part form).
    - A claim's own `SubjectMembership` no longer counts as a requirement subject.
  - **Notation:**
    - `Depicts::Satisfy(claim)`, keyed `satisfy <claim>`, is drawn when both ends are nodes. It's a solid line, an open arrowhead at the requirement, and a `«satisfy»` label, placed beside steep straight edges so the line doesn't strike through it.
    - In layering, requirements sit above their satisfiers.
    - A satisfier lists `satisfy <requirement>` rows in its `requirements` compartment.
  - **Properties:**
    - A requirement shows "Satisfied by" rows, and a satisfier "Satisfies" rows, both as links.
    - A claim itself shows "Requirement" and "Satisfier".
  - **Viewer:** clicking a `«satisfy»` edge selects its claim.
  - **Scope note:** in a package-rooted Diagram the nested requirements aren't nodes, so edges appear when a Diagram (a view, or an element set) holds both ends. The golden `requirements-satisfy.svg` draws exactly that.
  - **Tests:** 8 new tests in `requirements.rs`, and the twin plus contract test now include the four claims.
