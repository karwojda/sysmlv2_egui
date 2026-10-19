# 02: Analysis cases in the Model, and drawn

**What to build:** Analysis definitions and usages stop being Unsupported Elements.

- **Facts:** each carries its subject and the part it is bound to (`v = vehicle`), its objective (doc and `require` text) and its parameters (in, out and `return`, with values).
- **Node:** drawn as on slide 99: `«analysis def»` / `«analysis» name:Def` with `subject`, `objective`, `parameters` and `attributes` compartments.
- **Edge:** a dashed `«subject»` edge with an open arrowhead runs from an analysis to its subject part when both are drawn, keyed `subject <analysis>`.

See `.scratch/analysis/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `ElementKind::AnalysisDefinition` / `AnalysisUsage` with subject binding, objective, `return` flag
- [x] Analysis nodes with their four compartments
- [x] The `«subject»` edge, with saved routes applying
- [x] The `analysis` fixture gains an analysis def and usage; its twin and contract test match

## Comments

- Done.
  - **Model:**
    - `ElementKind::AnalysisDefinition` (a definition) and `AnalysisUsage` (a General node).
    - Elements gain `subject_target` (the part a subject is bound to, resolved through the subject's value the way a chained `satisfy … by` is), `objective: Option<Objective { doc, require }>`, and `is_return`.
    - The objective is the unnamed requirement under the analysis's `ObjectiveMembership`. Return parameters come from `ReturnParameterMembership` and are marked after the element loop.
  - **Notation:**
    - Analysis nodes lead with `subject` (`c:Component`, or `c = car` on a usage), `objective` (wrapped doc, `require {…}`) and `parameters` (source order; `return` replaces `out` for the result), then the usual compartments.
    - The requirement-style `subject …` header line is now limited to requirements.
    - `Depicts::Subject(analysis)`, keyed `subject <analysis>`, is a dashed open-arrow `«subject»` edge to the bound part when both are drawn. The `«satisfy»` edge now shares its drawing (`labelled_arrow`). Clicking the edge in the viewer selects the analysis.
  - **Tests:** the `analysis` fixture and twin gain `MassCheck` and `carMass`, confirmed by the contract test. `viewer-core/tests/analysis.rs` (6) and the golden `analysis-package.svg`.
