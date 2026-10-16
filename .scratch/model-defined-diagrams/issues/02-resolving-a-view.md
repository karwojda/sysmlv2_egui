# 02: Resolving a view

**What to build:** A view resolves to a Diagram Kind and its contents, or to an Unsupported View with a reason.

- **Kind:** the view's own rendering decides it. Failing that, use the first rendering found on its type chain, breadth first. With no rendering anywhere, standard view-definition typing decides: `GeneralView` gives General, and `InterconnectionView` gives Interconnection, except `ActionFlowView` and `StateTransitionView`.
- **General contents:** the union of the expose scopes, keeping only nodes. A package exposed alone shows its members.
- **Interconnection root:** exactly one distinct expose target, which must be a part or part def. Anything else is an error that names the problem.

The Properties panel shows the resolved kind, or the reason a view is unsupported. See `.scratch/model-defined-diagrams/spec.md`.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Own render, inherited render, `GeneralView` / `InterconnectionView` typing → the right kind
- [ ] `ActionFlowView`, a table rendering, and no rendering → unsupported, each with its own reason
- [ ] `X`, `X::*`, `X::**`, a package alone, and overlapping exposes give the right node set with no duplicates; Unsupported Elements are dropped
- [ ] An Interconnection root with `::**` works; no target, two targets and a non-part target are errors
- [ ] The Properties panel shows the resolved kind or the reason
