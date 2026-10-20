# Spec: The backend's checks behind one seam

Status: done

## Problem Statement

On every load the OpenSysML backend checks the Model three ways: it verifies Satisfaction Claims, checks each top-level part tree (Constraint Results and Computed Values) and runs analysis usages. Each check lives in its own module that calls the service, translates its answer, decides which of our elements the answer is about and records it. The translation of a service verdict into a result is written three times, warnings come back in three shapes, and the Model's analysis setting is read in one place and second-guessed in another. The part with the real logic (which parts, which instance paths, which claim, what happens on failure) is only exercised by tests that need the service.

## Solution

One `checks` module owns all of it behind a single entry point. Its only dependency is a `Checker` seam with three questions to the service, answered in our own terms. The real adapter wraps the service and translates its vocabulary once; a table-driven test adapter lets every rule be tested without the service. Nothing a user sees changes.

## Implementation Decisions

- **Seam:** a crate-private `Checker` trait: verify the claims; check a part (constraint results with their instance paths, and computed values); run an analysis (outputs, objective result, and how long it took). Each answers with our results or a failure reason. The service's answer types can't be built outside its crate, which is why the seam answers in our types.
- **Real adapter:** wraps the parsed service Model. One function turns a service verdict into a result (error → undecided; holds; violated with condition; the `holds` flag when there's no status text; anything else undecided). Value formatting (`1363.0`, `1650 [kg]`) lives here.
- **Interface:** `checks::run(checker, elements, folder, analyses) -> Vec<Diagnostic>`. Behind it: claims verified only when there are any, the settings file and its warning, top-level parts, instance paths, declaration-order matching, Unchecked on failure, "not run", the slow-load warning.
- **Order:** claims, analyses, parts, as now. Warning texts unchanged.
- **Layout:** one `checks` module with private claims, parts, analyses and adapter parts; the old `satisfaction`, `constraints` and `analyses` modules go.

## Testing Decisions

- The `record()` unit tests are replaced by tests across `checks::run` with a table-driven `Checker`: claim matching, unmatched claims, failed verify; instance paths, inherited constraints, failed part check, computed values; analysis selection (all, none, one, from the setting, broken setting), not run, failed run, slow warning.
- Contract tests stay as the real adapter's tests and pass unchanged, as do the demo pictures.

## Out of Scope

- The JSON reader (`api_json::enrich`) and its stale warning text.
- The Model-errors path of a load.

## Further Notes

- Settled with the user on 2026-10-20 (Q1–Q10), candidate 2 of the architecture review.
