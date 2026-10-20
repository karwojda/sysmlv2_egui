# Spec: One Verdict module

Status: done

## Problem Statement

Satisfaction Status, Constraint Results and Analysis Results are three kinds of the same thing: what an element shows because of what OpenSysML checked. Each is written out separately in about seven places: its status values, mark, label, worst-of ordering, mapping from a check's result, and detail text; the Diagram badge; the Properties rows; a Style condition and its editor picker; a `check` report. Adding a kind means touching all of them, and they already disagree: an analysis that wasn't run is Undecided on the badge and in Styles, but "Not run" in `check` and Properties.

## Solution

One Verdict module answers, for any element, "what is its Verdict, and why". The badge, Properties, Styles and `check` ask it instead of each kind's module. One four-value scale (holds, violated, undecided, absent) replaces the two status enums, each kind keeping its own words.

## User Stories

1. As a systems engineer, I want an analysis that wasn't run to read ○ "Not run" everywhere (badge, Style, Properties, `check`), so that I never mistake it for an undecided result.
2. As a systems engineer, I want an asserted constraint nobody checked to read ○ "Not checked", so that it isn't confused with a check that couldn't decide.
3. As a systems engineer, I want a Style rule for analyses not run and constraints not checked, so that I can colour them.
4. As a systems engineer, I want everything else — marks, labels, Legend text, `check` output, existing Style files — unchanged, so that the refactor costs me nothing.
5. As a maintainer, I want the next kind of Verdict (verification cases) to be added in one module, so that it can't drift from the others.

## Implementation Decisions

- **Term:** Verdict, in `CONTEXT.md`, with Satisfaction Status, Constraint Results and Analysis Results as its kinds.
- **Scale:** holds, violated, undecided, absent. Worst first: violated, undecided, absent, holds. Marks ✓ ✗ ? ○.
- **Labels per kind:**
  - requirement: Satisfied / Violated / Undecided / Not claimed;
  - constraint: Holds / Violated / Undecided / Not checked;
  - analysis: Holds / Violated / Undecided / Not run.
- **Interface:** `of(model, element) -> Option<Verdict>` and `all(model)`, every element with a Verdict. A Verdict has its kind, status and findings. A finding is what one check found: what it is about (a claim's satisfier, a part, the objective), its status, and its detail (the failing condition or the reason).
- **Who has one:** requirement usages, asserted constraint usages, analysis usages with an objective. Parts and analyses without objective have none.
- **Sources stay put:** `satisfaction` keeps claims, nesting and roll-up; `constraint` keeps checks on a part, row text and the part-side `constraints` condition; `analysis` keeps objectives and result rows. They use the shared scale for their marks.
- **Absent:** a requirement nothing claims; an asserted constraint with no checks; an analysis not run. A check the service tried and failed (`Unchecked`) is undecided.
- **Styles:** the file keeps `satisfaction`, `constraint` and `analysis`; constraint gains `not-checked`, analysis gains `not-run`. Internally one condition: a Verdict of kind K with status S. The editor offers each kind's four values. Legend: "Constraint not checked", "Analysis not run".
- **CLI:** `check` and `requirements` output byte-identical, except where an absent value now reads differently.
- **Replace, don't layer:** `SatisfactionStatus`, `ConstraintStatus` and their helpers are deleted.
- **Out of the module:** `ClaimResult` and the backend's mapping of service answers (candidate 2 of the architecture review).

## Testing Decisions

- Tests cross the Verdict interface: status, labels, findings and worst-of for each kind, absent cases.
- Existing satisfaction, constraint and analysis tests move to it; scenes, goldens, properties, styles and CLI tests stay as they are, changed only where a mark changes.
- Prior art: `viewer-core/tests/satisfaction.rs`, `constraints.rs`, `analysis.rs`, `viewer-cli/tests/check.rs`.

## Out of Scope

- The backend's three verdict mappings.
- Notation's compartment table (candidate 3).

## Further Notes

- Settled with the user on 2026-10-20 (Q1–Q13), from the architecture review's top candidate.
