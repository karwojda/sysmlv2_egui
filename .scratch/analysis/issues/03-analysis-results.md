# 03: Analysis Results

**What to build:** On load, the backend runs `run_analysis` for every analysis usage. Each gets an Analysis Result:

- outputs, which fill the matching parameters' Computed Values, so `return totalMass … = 1363.0`;
- the objective, as Holds, Violated (with condition) or Undecided (with reason).

A failed run gives Unchecked, with a warning. Analysis usages with an objective wear the badge, and Properties lists outputs and the objective. See `.scratch/analysis/spec.md`.

**Blocked by:** 01, 02

**Status:** ready-for-agent

- [ ] Results recorded by a pure, unit-tested function (outputs, objective outcomes, failure)
- [ ] `return` and other output parameters show computed values
- [ ] Badge only with an objective; Properties outputs and objective
- [ ] The contract test covers the fixture's result; golden of an analysis node with its result
