# Spec: Constraints

Status: done

## Problem Statement

My Models assert constraints that a consistent design must meet: a tank's fuel mass within its maximum, a power-to-weight ratio under a limit. They also define reusable constraints with parameters. The viewer draws none of this. Constraint definitions and usages are Unsupported Elements, and an asserted constraint inside a part leaves no trace on the part's node. I can't see which constraints my design breaks, or which part breaks them, and CI can't tell me when a change makes the Model inconsistent.

OpenSysML can check every asserted constraint in a part tree against the Model's values. Requirements already show their status this way; constraints should too.

## Solution

- **Asserted constraints on parts.** Parts and part defs get an `assert constraints` compartment, as on slides 85–86:
  - Each row is the constraint's text, `{fuel.fuelMass <= fuelMassMax}`.
  - On a part, each row is marked with what checking it found: ✓ holds, ✗ violated, ? undecided.
  - Constraints a part inherits from its definition are listed on the part prefixed `^`, with that part's own result.
- **Constraint definitions and usages as nodes** (slide 87):
  - `«constraint def» MaxRatio` with `parameters` and `expression` compartments.
  - `«assert constraint» powerToWeight:MaxRatio` with its bound parameters (`numerator = mass`) and a status badge.
- **Checking.** Constraints are checked on every load, by OpenSysML, for every top-level part and everything beneath it. The Properties panel explains each result.
- **Styles.** A Style can colour asserted constraint nodes by result, and parts that have a violated constraint of their own.
- **CLI.** A `check` command reports requirements and asserted constraints together and fails on any violation, as one gate for CI.
- **Demo.** The mass roll-up demo gains asserted constraints: the failing variant breaks one, the passing variant meets both.

## User Stories

1. As a systems engineer, I want a part's asserted constraints listed in an `assert constraints` compartment as text (`{mass <= 1500.0}`), so that I can see what the design must meet.
2. As a systems engineer, I want each asserted constraint on a part marked ✓, ✗ or ?, so that I can see at a glance which hold.
3. As a systems engineer, I want constraints a part inherits from its definition shown on the part with `^` and that part's own result, so that a definition's rule is checked where the values are.
4. As a systems engineer, I want a part def's asserted constraints shown without a mark, so that a definition without values doesn't look checked.
5. As a systems engineer, I want constraint definitions drawn with their parameters and expression, so that reusable constraints are readable.
6. As a systems engineer, I want an asserted constraint usage drawn as `«assert constraint» name:Def` with its bound parameters and a status badge, so that a parameterised check reads like slide 87.
7. As a systems engineer, I want unasserted constraint usages drawn without a mark, so that named conditions don't look like claims.
8. As a systems engineer, I want constraints checked automatically on load and live reload, so that the marks are always current.
9. As a systems engineer, I want the Properties panel to show each result with its failing condition, or the reason it's undecided, and on which part, so that I can fix the design.
10. As a systems engineer, I want a clear warning, and every result undecided, when the service can't check, so that unknown never reads as fine.
11. As a systems engineer, I want a Style condition on a constraint's result, so that violated constraint nodes can be red.
12. As a systems engineer, I want a Style condition for parts with a violated constraint of their own, so that the offending part stands out in a big tree.
13. As a systems engineer, I want those conditions in the Legend ("Constraint violated", "Part with a violated constraint"), so that readers know what the colours mean.
14. As a documentation maintainer, I want all marks, compartments and badges in SVG Exports, identical to the viewer, so that documents show the Model's consistency.
15. As a documentation maintainer, I want `sysml-viewer check <folder>` to report every requirement and asserted constraint and exit 1 on any violation (`--strict` also on undecided and unclaimed), so that CI has one gate.
16. As a systems engineer, I want the mass roll-up demo to include asserted constraints that fail in one variant and pass in the other, so that I can see constraints at work.

## Implementation Decisions

### Model

- **New kinds:** `ConstraintDefinition` (`constraint def`) and `ConstraintUsage` (`constraint`), both drawn as General nodes.
- **New element facts:**
  - `is_asserted`, for an `assert constraint`.
  - `expression`, a constraint's own expression as text, rebuilt as for requirements (`…` where unknown).
- **Parameters:** a constraint def's parameters are its owned `in` features. Their direction and type come from the symbol tree. On a usage, a bound parameter carries the bound expression as its value (`numerator = mass`). Values already exist from the feature-values work.
- **Constraint Results:** a list on the Model of `{ constraint, part, result }`, with the result being Holds, Violated `{ condition }`, Undecided `{ reason }`, or Unchecked `{ reason }`. `part` is the part usage the constraint was checked on: the validated top-level part, or the part at the result's instance path beneath it.

### Checking

- On every load, the backend calls `validate_instance` once per top-level part usage, one owned by a package. It maps each verdict of kind `constraint` to a Constraint Result: the constraint from `element_id`, the part from the validated root plus the verdict's `instance_path` (`tank` → `vehicle::tank`).
- Holds, violated (with condition) or undecided (with the error) follow the verdict, as for requirements.
- If a part can't be validated, its constraints are Unchecked and one Model warning says why.
- Constraint defs and unasserted constraint usages are never checked.

### Notation

- **Parts:** parts and part defs get an `assert constraints` compartment, placed after `attributes` and before `requirements`.
  - On a def: one row per asserted constraint usage it owns, `{text}`.
  - On a part usage: its own asserted constraints, then inherited ones, i.e. results on this part whose constraint is owned elsewhere, as `^{text}`. Each row ends with the result's mark.
  - Owned asserted constraints are listed even when unchecked, marked ?.
- **Constraint def node:** `«constraint def» Name`, then a `parameters` compartment (`in numerator : Real`) and an `expression` compartment.
- **Constraint usage node:** `«assert constraint»` or `«constraint»`, then `name:Def`. Its `parameters` compartment shows the bound parameters (`numerator = mass`), then `expression` if it has its own. An asserted usage gets the status badge, the worst of its results.
- A constraint usage owned by a part is listed in the part's compartment, not drawn as a node, unless a view exposes it. Package-level constraint defs and usages are nodes.

### Styles

- `{"constraint": "holds" | "violated" | "undecided"}` matches asserted constraint usage nodes by their worst result. Legend: "Constraint holds", "Constraint violated", "Constraint undecided".
- `{"constraints": "violated"}` matches parts with a violated result of their own (inherited included; nested parts not). Legend: "Part with a violated constraint".
- Both rank with `satisfaction`, above metadata. The Style editor offers them.

### Properties

- **A part:** one "Asserts" row per result on it (`{fits} ✗`), plus a "Failing condition" or "Undecided because" row.
- **An asserted constraint usage:** "Result", one per part, with the details.
- **A constraint def:** its "Parameters".

### CLI

- `sysml-viewer check <model-folder> [--strict]`: the `requirements` report, then one line per Constraint Result (`✗ Violated  C::Tank::fits  on C::vehicle::tank: fuelMass <= fuelMassMax`), then a summary. It exits 1 on any violated requirement or constraint; with `--strict`, also on undecided or unclaimed ones. `requirements` stays as it is.

## Testing Decisions

- **Seams as before.** Seam 1: fake-backend Models carrying Constraint Results, driving scenes, SVGs, goldens, properties, styles and the CLI. Seam 2: contract tests of the real backend.
- **Seam 1:**
  - compartment rows (own, inherited `^`, def without marks);
  - constraint def and usage nodes, with bound parameters;
  - badge only on asserted usages;
  - result mapping (instance paths);
  - both Style conditions, their rank and Legend labels;
  - `check` output, exit codes and `--strict`;
  - goldens for parts with assertions and for a constraint def and usage.
- **Seam 2:** an `assertions` fixture with:
  - an inline assertion that holds and one that's violated;
  - one inherited from a part def;
  - a constraint def used with bound parameters;
  - an unasserted constraint.

  The twin must match the facts and results. Separately, the demo's tests are extended and its pictures refreshed.
- **Prior art:** `viewer-core/tests/satisfaction.rs`, `requirements.rs`, `viewer-cli/tests/requirements.rs`, `viewer-opensysml/tests/contract.rs`.

## Out of Scope

- The parametric view (slide 88): constraint boxes with parameter ports bound to attributes.
- `assume` constraints, derived parameters (`/x_out`), analysis cases.
- Showing computed values (`= 265`); the service's calculation call could provide them later.
- Rolling part results up to enclosing parts (Q8).

## Further Notes

- Decisions settled with the user on 2026-10-19 (Q1–Q9).
- Backend probe: `validate_instance("C::vehicle")` returned, for the probe Model, `light` holds, `powerToWeight` violated (`numerator / denominator <= limit`, 1400 / 90 > 12), and the inherited `Tank::fits` violated at instance path `tank`. `verify_constraint` on a definition-level constraint is undecided ("fuelMass has no value"), which is why parts, not constraints, are what gets checked.
- Notation: slides 85–88 of the Graphical Notation deck.
