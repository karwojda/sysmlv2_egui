# 03: Satisfaction Claims and «satisfy» edges

**What to build:** Each `satisfy` in the Model becomes a Satisfaction Claim. The requirement usage is the one the `satisfy` subsets, or the end of its feature chain. The satisfier is its `by` subject, or else its owning part.

- **Edges:** claims are drawn in General Diagrams as `«satisfy»` edges (solid line, open arrowhead at the requirement), keyed `satisfy <claim>` so bends can be saved.
- **Parts:** parts gain a `requirements` compartment (`satisfy massReq`).
- **Properties:** "Satisfied by" links on a requirement and "Satisfies" links on a part.

This ticket covers structure only; statuses come in 04. See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Claims are read for both forms (`satisfy R by x`, and `satisfy R;` inside a part)
- [ ] `«satisfy»` edges are drawn when both ends are nodes, and saved routes apply
- [ ] The part `requirements` compartment
- [ ] The Properties links
- [ ] The contract twin matches the claims
