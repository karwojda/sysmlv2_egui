# Spec: Requirements satisfaction

Status: ready-for-agent

## Problem Statement

My Model states requirements, with constraints such as "mass ≤ 2000 kg". It also states which parts satisfy them (`satisfy massReq by vehicle`). The viewer shows none of it. Requirements are Unsupported Elements, so they are never drawn, and I can't see which requirements are met, which are violated, which can't be decided yet, and which nothing claims to satisfy.

OpenSysML can evaluate these claims against the Model's values. Today I'd have to run that by hand and read raw results. I want the answer on my Diagrams and in my documents, and I want CI to tell me when a change breaks a requirement.

## Solution

- **Drawing requirements.** Requirement definitions and usages are drawn in General Diagrams in the standard notation: `«requirement»`, short id, name and type, a subject line, and `doc`, attributes and constraints compartments. Each `satisfy` in the Model (a Satisfaction Claim) is drawn as a `«satisfy»` edge from the part to the requirement.
- **Checking.** On every load the viewer asks OpenSysML to check every claim. Each requirement usage gets a Satisfaction Status, shown as a badge in its corner: ✓ Satisfied, ✗ Violated, ? Undecided, ○ Not claimed. Each `«satisfy»` edge carries its own result.
- **Parts and trees.** Parts list what they satisfy in a "requirements" compartment. Requirement trees open as General Diagrams rooted at a requirement usage.
- **Details.** The Properties panel explains each status: the failing condition, or why it couldn't be decided.
- **Styles.** A `satisfaction` condition lets Styles colour requirements by status, with a Legend.
- **CLI.** `sysml-viewer requirements` prints a status table and fails on violations, for CI.

## User Stories

1. As a systems engineer, I want requirement definitions and usages drawn in General Diagrams in the standard notation, so that my requirements are visible beside the design.
2. As a systems engineer, I want a requirement's short id (`<1>`), name and type in its header, so that I can identify it as in my specifications.
3. As a systems engineer, I want a requirement's subject shown as `subject v:Vehicle`, so that I can see what it constrains.
4. As a systems engineer, I want a requirement's `doc` text in a compartment, so that the shall-statement is readable on the Diagram.
5. As a systems engineer, I want a requirement's `require` constraints shown as text (`require {v.mass <= 2000.0}`), so that I can see the condition being checked.
6. As a systems engineer, I want a constraint the viewer can't render shown as `{…}` rather than something wrong, so that I'm never misled.
7. As a systems engineer, I want each `satisfy` drawn as a `«satisfy»` edge from the satisfying part to the requirement, so that traceability is visible.
8. As a systems engineer, I want a `satisfy` written inside a part (`part car { satisfy massReq; }`) drawn from that part, so that both ways of writing it look the same.
9. As a systems engineer, I want every requirement usage to show its Satisfaction Status as a badge (✓ ✗ ? ○), so that I can see at a glance what is met.
10. As a systems engineer, I want requirements nothing claims to satisfy marked "Not claimed", so that gaps in coverage stand out.
11. As a systems engineer, I want each `«satisfy»` edge marked with its own result, so that I can tell which satisfier fails when there are several.
12. As a systems engineer, I want a requirement's badge to summarise all its claims, worst first, so that one failing satisfier isn't hidden by a passing one.
13. As a systems engineer, I want requirement definitions left without a badge, so that only things that can be satisfied carry one.
14. As a systems engineer, I want part nodes to list what they satisfy in a "requirements" compartment (`satisfy massReq ✓`), so that I can see a part's obligations from the part.
15. As a systems engineer, I want to open a General Diagram rooted at a requirement usage, so that I can see a requirement specification as a tree.
16. As a systems engineer, I want nested requirements listed in a "requirements" compartment with their marks (`<1> massReq ✗`), so that a specification's state is visible on one node.
17. As a systems engineer, I want statuses checked automatically when the Model loads and on live reload, so that the picture is always current.
18. As a systems engineer, I want a clear warning, and every status Undecided, when the service can't check, so that I never mistake "unknown" for "met".
19. As a systems engineer, I want the Properties panel of a requirement to show its status, the failing condition or the reason it's undecided, and "Satisfied by" links with each result, so that I can understand and fix it.
20. As a systems engineer, I want the Properties panel of a part to list "Satisfies" rows with results, so that I can check a part's obligations.
21. As a systems engineer, I want a Style Rule condition on Satisfaction Status that beats every other condition, so that I can colour violated requirements red without other rules overruling it.
22. As a systems engineer, I want satisfaction colours to appear in the Legend like other rules, so that readers understand them.
23. As a documentation maintainer, I want badges, edges and compartments in SVG Exports, deterministic and identical to the viewer, so that my documents show requirement status.
24. As a documentation maintainer, I want `sysml-viewer requirements <folder>` to print each requirement usage with its status and satisfiers, so that I get a textual report.
25. As a documentation maintainer, I want that command to exit with failure when any requirement is Violated, and with `--strict` also on Undecided and Not claimed, so that CI can gate on requirement status.
26. As a systems engineer, I want views exposing requirements (`expose vehicleSpec::**`, `asTreeDiagram`) to show requirement trees with statuses, so that Model-defined Diagrams work for requirements too.

## Implementation Decisions

### Model

- **New kinds:** `RequirementDefinition` (`requirement def`) and `RequirementUsage` (`requirement`). Both draw as General nodes.
- **New element facts:**
  - `short_name`, from `declaredShortName`, shown as `<id>`.
  - `subject`: the owned feature that is the requirement's subject (a `SubjectMembership`). It is shown on the subject line, not in a compartment.
  - `require`: the `require` constraints as display text, in order.
- **Constraint text** is rebuilt from the API JSON expression tree:
  - Covered: operators (comparison, arithmetic, `and` / `or` / `not` / `xor`, `implies`), feature chains (`v.mass`), feature references, literals (numbers, strings, booleans), and unit-qualified literals where the tree shows them.
  - Anything else becomes `{…}`. Parentheses are added only where precedence needs them.
  - `assume` constraints are read but not shown in this slice.
- **Satisfaction Claims** come from the API JSON `SatisfyRequirementUsage` items:
  - The requirement usage is the one it `subsets`, or the end of its feature chain for the inside-a-part form.
  - The satisfier is its `subject` reference (`by x`), or else its owning part.
  - Each claim has a generated identity, like unnamed connections.
- **Claim results:** `Holds`, `Violated { condition }`, `Undecided { reason }`, or `Unchecked { reason }` when the service can't check at all.
  - The backend fills them from the service's satisfaction check over the whole Model, run once per load after parsing. Results are matched to claims by requirement and satisfier; where the service gives no satisfier (the inside-a-part form), by order.
  - A failed check marks every claim Unchecked and adds one Model warning naming the service's reason.
- **Satisfaction Status** of a requirement usage is derived, not stored: Not claimed when it has no claims; otherwise the worst of its claims' results (Violated, then Undecided or Unchecked, then Satisfied).
- ADR-0001 still holds. The service's verification RPC becomes a third backend input, after the symbol tree and the API JSON. ADR-0001 gains a consequence noting it.

### Notation (General Diagram)

- **Requirement node:** a rounded box for usages and a square one for definitions, as with parts.
  - Header: `«requirement»` or `«requirement def»`, then `<id> name:Type` (not bold for usages), then `subject v:Vehicle`.
  - Compartments: `doc`, `attributes`, `constraints` (`require {…}`), and `requirements` (nested requirement usages, as `<id> name:Type` plus a mark).
- **Badge:** a small circle in the top-right corner of requirement usages, holding ✓, ✗, ? or ○. It isn't coloured by default: Styles do colour.
- **`«satisfy»` edge:** a solid line with an open arrowhead at the requirement, labelled `«satisfy»` plus the claim's mark. It's drawn when both ends are nodes, keyed `satisfy <claim>` for Layout routes.
- **Part nodes** gain a `requirements` compartment: one row per claim the part is the satisfier of, `satisfy massReq ✓`.
- **Roots:** a General Diagram can be rooted at a requirement usage, showing its owned members.
- **Marks:** ✓ ✗ ? ○ come from the scene's ordinary text. Both renderers must draw them (font coverage is checked in the viewer).

### Styles

- A new condition `satisfaction` with values `satisfied`, `violated`, `undecided`, `not-claimed`. It matches requirement usages with that status, and ranks above metadata.
- Its Legend label is `Satisfied`, `Violated`, `Undecided` or `Not claimed` unless the rule has its own.

### Properties panel

- **A requirement usage:** "Status" (with the mark), then "Failing condition" or "Undecided because", then one "Satisfied by" row per claim (a link to the satisfier, with its result).
- **A part:** one "Satisfies" row per claim (a link to the requirement, with its result).
- **Requirements:** "Subject", "Short id" and "Requires" rows.

### CLI

- `sysml-viewer requirements <model-folder> [--strict]` loads the Model, prints diagnostics to stderr, and writes one line per requirement usage to stdout, in qualified-name order:
  `✗ Violated     R::rangeReq      by R::vehicle: v.range >= 500.0`.
- **Exit status:** 0 normally; 1 if any requirement is Violated; with `--strict`, also 1 if any is Undecided or Not claimed. A Model with errors fails as Export does.

## Testing Decisions

- **Seams as before.** Seam 1: fake-backend Models (builder methods for requirement facts, claims and their results) driving scenes, SVGs, goldens, properties, Style matching and the CLI. Seam 2: contract tests of the real backend against a `requirements` fixture and its fake twin.
- **Seam 1:**
  - Requirement node content: header, subject line, compartments, and subject features not repeated.
  - Constraint text for each covered construct, plus the `{…}` fallback.
  - Status derivation: no claims, all holding, one violated among several, undecided, unchecked.
  - Badges on usages only.
  - `«satisfy»` edges with marks, and Layout routes on them.
  - The part requirements compartment, requirement-usage roots, and the nested requirements compartment.
  - Properties rows.
  - Style condition rank and Legend labels.
  - CLI table, exit codes and `--strict`.
  - Goldens for a requirement tree with statuses, and for parts with `«satisfy»` edges.
- **Seam 2:** the `requirements` fixture has:
  - a requirement definition with a short id, subject, doc and require constraint;
  - usages that hold, are violated, are undecided, and are not claimed;
  - both `satisfy` forms;
  - a nested specification.

  The twin must match facts, claims and results. The inside-a-part claim is expected Undecided while the service leaves its subject unbound.
- **Prior art:** `viewer-core/tests/style.rs`, `view_diagram.rs`, `properties.rs`, `viewer-cli/tests/export_views.rs`, `viewer-opensysml/tests/contract.rs`.

## Out of Scope

- Verification cases (`verify`, verification verdicts).
- Checking unclaimed requirements against a part chosen by the user.
- Showing `assume` constraints.
- Derived-requirement (`#derive`) and `«allocate»` edges.
- Satisfaction markers in Interconnection Diagrams.
- Rolling statuses up from nested requirements to their parents.

## Further Notes

- Decisions settled with the user on 2026-10-18 (Q1–Q13).
- **Backend probes:**
  - `Model::verify_satisfaction(None, …)` returns one verdict per `satisfy`, each holds, violated (with the condition text) or undecided (with an error).
  - `SatisfyRequirementUsage` items carry `subject` and `subsets`, or a feature chain for the inside-a-part form.
  - Requirement definitions carry `declaredShortName`.
  - Constraints arrive as expression trees, not text.
  - `satisfy` may only target requirement usages.
- **Known service gap:** a `satisfy` inside a part is evaluated without binding the part as the subject (Undecided, "no value for feature v"). Worth reporting to OpenSysML.
- **Notation:** slides 90–93 of the Graphical Notation deck; the `satisfy-edge` in SysML v2 Part 1 §8.2.3 (page 245).
