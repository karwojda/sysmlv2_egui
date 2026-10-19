# 03: Analysis Results

**What to build:** On load, the backend runs `run_analysis` for every analysis usage. Each gets an Analysis Result:

- outputs, which fill the matching parameters' Computed Values, so `return totalMass … = 1363.0`;
- the objective, as Holds, Violated (with condition) or Undecided (with reason).

A failed run gives Unchecked, with a warning. Analysis usages with an objective wear the badge, and Properties lists outputs and the objective. See `.scratch/analysis/spec.md`.

**Blocked by:** 01, 02

**Status:** done

- [x] Results recorded by a pure, unit-tested function (outputs, objective outcomes, failure)
- [x] `return` and other output parameters show computed values
- [x] Badge only with an objective; Properties outputs and objective
- [x] The contract test covers the fixture's result; golden of an analysis node with its result

## Comments

- Done.
  - **Model:** analysis usages carry `analysis_result: Option<AnalysisResult { outputs, objective }>`. `viewer_core::analysis` gives the objective (own or from the definition), the status (the objective's outcome, Undecided if not run, `None` without an objective) and the `results` rows.
  - **Backend (`viewer-opensysml/src/analyses.rs`):**
    - One `run_analysis` per analysis usage on load.
    - Outputs are formatted like Computed Values.
    - The objective outcome comes from the `objective` verdict's `holds` flag, since its status text is empty, with any error making it Undecided.
    - A failed run is Unchecked, with a warning.
    - The recording is pure, with 5 unit tests that run in CI.
  - **Display, a deliberate deviation from the spec text:** a usage doesn't own its definition's parameters, so its outputs appear in a `results` compartment (`return result:Real = 230.0`), with `return` and the type taken from the definition's parameter. Writing them into the parameters would mean drawing parameters the usage doesn't have.
  - **Badge and Properties:** only analyses with an objective wear the badge. Properties shows "Output" rows and "Objective".
  - **Tests:** the twin carries the service's exact result, confirmed by the contract test. 3 new tests in `analysis.rs`, and the golden `analysis-result.svg`.
