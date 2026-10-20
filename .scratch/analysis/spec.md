# Spec: Computed values and analysis cases

Status: done

## Problem Statement

My Diagrams show how a value is computed (`mass = powertrain.mass + body.mass + …`) but not what it comes to. To learn that the vehicle weighs 1363 kg I have to work it out by hand. My Models also contain analysis cases, a subject with inputs, a result and an objective, and the viewer ignores them. OpenSysML can evaluate both: the values of every feature of a part, and the outcome of an analysis. I want both on the Diagram, checked as part of my CI gate.

## Solution

- **Computed values.** Every row whose value is an expression shows what it evaluates to: `mass = powertrain.mass + body.mass + fuelTank.mass + sum(wheels.mass) = 1363.0`. The values come from the part check the viewer already runs, now on every load.
- **Analysis cases.** Analysis definitions and usages are drawn as on slide 99, with `subject`, `objective`, `parameters` and `attributes` compartments. The `return` result shows its computed value, and a `«subject»` edge points to the part under analysis.
- **Running analyses.** Every analysis usage is run on load. Its objective gives a badge, ✓ holds, ✗ violated or ? undecided, and Properties lists its outputs.
- **The off switch.** A per-Model setting can turn automatic runs off, for slow Models. When it's off, a "Run now" button runs one analysis, and the viewer warns when a load's analyses take more than three seconds.
- **Styles and CLI.** An `analysis` Style condition colours analyses by outcome, and `check` reports analyses and fails on a violated objective.
- **Demo.** The mass roll-up demo gains a mass analysis and shows its computed totals.

## User Stories

1. As a systems engineer, I want an expression-valued attribute row to show its computed value after the expression, so that I see both how a total is made and what it comes to.
2. As a systems engineer, I want literal values left as written, so that rows don't repeat themselves.
3. As a systems engineer, I want a part drawn for several instances (the four `wheels`) to show one value when they agree and the distinct values when they don't, so that the row stays truthful.
4. As a systems engineer, I want the Properties panel to show a "Computed" row, so that I can read the value without the Diagram.
5. As a systems engineer, I want values computed on every load and live reload, so that they are always current.
6. As a systems engineer, I want rows to keep their expressions when the service can't compute, with the usual warning, so that I'm never shown a stale or invented number.
7. As a systems engineer, I want analysis definitions and usages drawn with `subject`, `objective`, `parameters` and `attributes` compartments, so that analyses read like slide 99.
8. As a systems engineer, I want an analysis usage's `return` result shown with its computed value, so that I see the analysis's answer on the Diagram.
9. As a systems engineer, I want a `«subject»` edge from an analysis to the part it analyses, so that I can see what is being analysed.
10. As a systems engineer, I want an analysis with an objective to wear a ✓ / ✗ / ? badge, so that a failing analysis stands out like a failing requirement.
11. As a systems engineer, I want the Properties panel of an analysis to list each output with its value and the objective's result, with the failing condition or reason, so that I can act on it.
12. As a systems engineer, I want analyses run automatically on load, so that results are always current.
13. As a systems engineer, I want a per-Model setting to turn automatic analysis runs off, saved beside the Model, so that a slow Model stays usable for everyone.
14. As a systems engineer, I want a "Run analyses on load" checkbox in the viewer and a "Run now" button on an analysis, so that I can control runs without editing files.
15. As a systems engineer, I want a warning when a load's analyses take more than three seconds, pointing at the switch, so that I know why the viewer is slow.
16. As a systems engineer, I want an analysis that wasn't run to read ? "not run", so that I can't mistake it for a result.
17. As a systems engineer, I want an `analysis` Style condition, so that violated analyses can be coloured.
18. As a documentation maintainer, I want `check` to list each analysis with its result and fail on a violated objective, so that CI covers analyses.
19. As a documentation maintainer, I want `check` to follow the Model's setting, with `--analyses` and `--no-analyses` to override, and skipped analyses counting as undecided, so that CI behaves predictably.
20. As a systems engineer, I want the mass roll-up demo to show computed totals and a mass analysis, so that I can see this in action.

## Implementation Decisions

### Computed Values

- **Every top-level part checked.** The part check (`validate_instance`) runs for every top-level part usage on every load, not only when constraints are asserted. Constraint Results and Computed Values come from the same call.
- **Values from instances.** Each instance in the result names its type symbol (the part usage, e.g. `MassRollup::vehicle::powertrain`) and its feature values by name. The value of feature `f` belongs to our element `<type symbol>::f`.
- **Several instances.** A part with several instances contributes one value per instance. The element keeps the distinct values, in first-seen order.
- **Formatting.** Values are written like literals: reals with a decimal point (`1363.0`), integers, booleans, strings in quotes. Anything else is left out.
- **Model:** elements gain `computed: Vec<String>`.
- **Notation:** a row whose `value` isn't a plain literal appends ` = <computed>`, with distinct values joined by `, `. Literal rows don't, nor do rows already equal to their value.
- **Properties:** a "Computed" row.

### Analysis cases

- **Kinds:** `AnalysisDefinition` (`analysis def`) and `AnalysisUsage` (`analysis`), from the symbol tree's `analysisCaseDef` / `analysisCaseUsage`.
- **Facts:**
  - the subject, as for requirements, plus the part it is bound to (`v = vehicle`), read from the subject's value;
  - the objective's doc and `require` constraints as text;
  - the `return` parameter (`is_return`);
  - in, out and return parameters with their values.
- **Node:**
  - Header: `«analysis def» Name` or `«analysis» name:Def`.
  - Compartments, in order: `subject` (`v = vehicle`), `objective` (its doc, then `require {…}` rows), `parameters` (`in engineMass:Real = 180.0`, `return totalMass:Real = … = 1363.0`) and `attributes`.
  - Badge: usages with an objective wear the badge.
- **`«subject»` edge:** a dashed line with an open arrowhead from the analysis node to its subject part, when both are drawn. It is keyed `subject <analysis>` so it can be routed.

### Analysis Results

- **Running:** on load, unless switched off, the backend runs `run_analysis` for every analysis usage and times the total.
- **Results:** each analysis gets `analysis_result: Option<AnalysisResult { outputs: Vec<(name, value)>, objective: Option<ClaimResult> }>`.
  - Outputs fill the matching parameters' Computed Values.
  - The objective verdict becomes Holds, Violated (with condition) or Undecided (with reason).
  - A failed run gives Unchecked, with one warning.
  - An analysis not run (switched off) has no result and reads ? "not run".
- **Status:** an analysis usage's status is its objective's result, or none without an objective.
- **Slow loads:** if a load's analyses take more than 3 s, a warning says so and names the setting.

### Settings

- **File:** `diagrams/settings.json`, `{ "format_version": 1, "run_analyses": true }`. Missing means defaults; a broken file gives a warning and defaults.
- **Viewer:** a "Run analyses on load" checkbox (in the top bar) writes it and reloads. "Run now" in an analysis's Properties runs that one analysis through the loader and shows its result until the next load.
- **Backend interface:** gains a way to load with or without analyses, and to run one analysis on the current Model.

### Styles and CLI

- **Style condition:** `{"analysis": "holds" | "violated" | "undecided"}` matches analysis usages by objective outcome. Legend: "Analysis holds" / "violated" / "undecided". It ranks with `satisfaction`.
- **`check`:** after constraints, one line per analysis usage. A violated line reads `✗ Violated  MassRollup::massAnalysis  totalMass = 1403.0: totalMass <= 1400.0`, a holding one `✓ Holds  …  totalMass = 1363.0`, and one not run `? Not run  …`.
  - It fails on a violated objective; with `--strict`, also on undecided or not-run.
  - `--analyses` / `--no-analyses` override the setting.

### Demo

- Both mass roll-up variants gain `massAnalysis`: subject the vehicle, return its total mass, objective ≤ 1400 kg. The `Vehicle` definition declares the parts the analysis reads, so the Model has no errors. Both variants hold (1363, 1333).
- The pictures show computed totals and the analysis node. The README explains them.

## Testing Decisions

- **Seams as before.** Seam 1: fake-backend Models with computed values and analysis results, driving scenes, SVGs, goldens, properties, styles, settings and the CLI. Seam 2: contract tests of the real backend.
- **Seam 1:**
  - Computed rows: expression rows, literal rows, several instances agreeing and disagreeing.
  - Analysis nodes: compartments, return value, `«subject»` edge, badge only with an objective.
  - Result mapping: outputs, objective outcomes, failure, not run.
  - Settings file: round trip, missing, broken.
  - Style condition.
  - `check`: lines, exit codes, `--strict`, `--analyses` / `--no-analyses`.
  - Goldens for computed rows and for an analysis node.
- **Seam 2:** an `analysis` fixture with an analysis def and usage (subject bound, objective, return, input with default), plus parts with roll-ups. The twin must match the computed values and the result. The demo's tests and pictures are updated.
- **Prior art:** `viewer-core/tests/constraints.rs`, `satisfaction.rs`, `viewer-cli/tests/check.rs`, `viewer-opensysml/tests/contract.rs`.

## Out of Scope

- Parameter sweeps and their charts: the next slice.
- Trade studies (`alternatives`), action and state simulation, running calc definitions on their own.
- Editing analysis inputs from the viewer.

## Further Notes

- Decisions settled with the user on 2026-10-19 (Q1–Q8). Q4 chose automatic runs with a per-Model off switch.
- **Backend probes (demo copy):**
  - `validate_instance("MassRollup::vehicle")` returned every instance's feature values: `vehicle.mass` = 1363.0, `powertrain.mass` = 265.0, and four `wheels` instances at 22.0.
  - `run_analysis` returned `totalMass` = 1363.0 with the objective holding, in about 100 ms.
  - A sweep of the engine mass over five values took about 70 ms.
- Notation: slides 99–100 of the Graphical Notation deck.
