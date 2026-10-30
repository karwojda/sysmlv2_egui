# Spec: Model-defined Diagrams

Status: done

## Problem Statement

My SysML models already say which pictures matter. I write `view` usages that `expose` the elements I want and `render` them as a tree or interconnection diagram. The viewer ignores them. To get those pictures I have to recreate each one as a Tool-defined Diagram, picking a kind and a root. A Tool-defined Diagram can't show an arbitrary set of elements, such as a parts tree for one subsystem or a handful of definitions from different packages. When I change a view in the model, nothing follows.

## Solution

The viewer draws a `view` usage as a Model-defined Diagram:

- **Diagram Kind.** The view's rendering decides it, or failing that its view definition: a tree rendering or a General View becomes a General Diagram, and an interconnection rendering or an Interconnection View becomes an Interconnection Diagram. Views of any other kind are Unsupported Views. They're listed and marked, but not drawn.
- **Contents.** A General Diagram shows exactly the Exposed Elements. `X` means X alone, `X::*` its members, `X::**` everything beneath it, and several exposes combine. Where a node's owner is also drawn, a composition edge joins them, which gives the parts tree. An Interconnection Diagram is rooted at the one part or part def the view exposes.
- **Finding them.** The viewer lists the Model's views. Double-clicking a view in the Browser opens it. `export --all` writes every supported view as `<view name>.svg`.
- **Layout.** Saving keeps a Diagram File that refers to the view by name. The kind and contents are re-read from the Model on every load, so changing the view's exposes or rendering updates the Diagram while the hand-made Layout stays.

## User Stories

1. As a systems engineer, I want each `view` usage in my Model listed in the viewer, so that I can see which pictures the Model defines.
2. As a systems engineer, I want to open a view from that list, so that I get the picture the Model describes without rebuilding it by hand.
3. As a systems engineer, I want to double-click a view in the Browser to open it, so that views behave like other elements I can open.
4. As a systems engineer, I want a view with `render asTreeDiagram` drawn as a General Diagram, so that tree views appear in the standard notation.
5. As a systems engineer, I want a view with `render asInterconnectionDiagram` drawn as an Interconnection Diagram, so that wiring views appear as nested boxes and ports.
6. As a systems engineer, I want a view without its own `render` to use its view definition's rendering (for example `view def TreeView { render asTreeDiagram; }`), so that I can define the rendering once and reuse it.
7. As a systems engineer, I want a view typed by the standard `GeneralView` or `InterconnectionView` (directly or through my own view definitions) drawn as that kind, so that the standard view definitions work.
8. As a systems engineer, I want `expose X` to show X itself, `expose X::*` its members and `expose X::**` all its descendants, so that I control exactly what the picture contains.
9. As a systems engineer, I want several `expose` statements in one view combined, so that one picture can show elements from different places.
10. As a systems engineer, I want a part drawn together with the part that owns it joined by a composition edge, so that `expose vehicle::**` gives a parts tree.
11. As a systems engineer, I want an element that is drawn as a node left out of its owner's compartments, so that it isn't shown twice.
12. As a systems engineer, I want the usual specialization, typing and connection edges drawn between the exposed nodes, so that relationships among them stay visible.
13. As a systems engineer, I want exposing a package to show its members, so that `expose MyPackage` gives a useful picture even though packages aren't drawn as nodes.
14. As a systems engineer, I want Unsupported Elements among the Exposed Elements left out, so that the picture holds only what the tool can draw.
15. As a systems engineer, I want an Interconnection view rooted at the part or part def it exposes (with or without `::*` or `::**`), so that the usual way of writing it works.
16. As a systems engineer, I want a clear message when an Interconnection view exposes nothing usable or more than one part, so that I can fix the view.
17. As a systems engineer, I want views whose rendering or definition the tool doesn't support (tables, sequence, action-flow, state views) marked as unsupported in the list, so that I know they exist but can't be drawn yet.
18. As a systems engineer, I want an empty view (nothing exposed, or nothing drawable) to open as an empty Diagram with a note, so that I see why nothing appears.
19. As a systems engineer, I want to move, resize, hide, collapse and route things in a Model-defined Diagram exactly as in a Tool-defined one, so that I can fix a poor automatic arrangement.
20. As a systems engineer, I want saving a Model-defined Diagram to keep a Diagram File that refers to the view, named after the view by default, so that my Layout survives restarts.
21. As a systems engineer, I want a saved Model-defined Diagram to pick up changes to the view's exposes or rendering when I reload, so that the Model stays the single source of what is shown.
22. As a systems engineer, I want Layout entries for elements no longer exposed kept as Orphaned Layout Entries, so that un-exposing something by mistake doesn't lose my arrangement.
23. As a systems engineer, I want a Diagram File whose view was renamed or deleted to open with a clear message instead of failing silently, so that I can fix the reference.
24. As a systems engineer, I want live reload to redraw an open Model-defined Diagram when the view changes, so that editing the view in my text editor shows up at once.
25. As a systems engineer, I want the Properties panel for a view to show its Diagram Kind (or why it's unsupported) and its exposes, so that I can check how the viewer reads it.
26. As a documentation maintainer, I want `export --all` to write every supported view as `<view name>.svg`, alongside saved Diagram Files, so that my documents follow the Model's views.
27. As a documentation maintainer, I want `export --all` to warn about each Unsupported View it skips and still exit successfully, so that one table view doesn't break the build.
28. As a documentation maintainer, I want `export --view <name>` to export one view (and fail with a clear message for an Unsupported View), so that I can script single pictures.
29. As a documentation maintainer, I want export to fail naming both views when two have the same simple name, so that no output file is silently overwritten.
30. As a documentation maintainer, I want a saved Diagram File's Layout used when a view is exported, so that the exported picture matches what I arranged in the viewer.
31. As a documentation maintainer, I want Model-defined Diagram Export deterministic and identical to the viewer, so that documents don't churn.

## Implementation Decisions

### Model

- New element kinds `ViewUsage` (keyword `view`) and `ViewDefinition` (keyword `view def`), from the symbol tree like other kinds. A view's typing and specializations come from the symbol tree as usual, including library types such as `StandardViewDefinitions::GeneralView`.
- New element facts:
  - `exposes`: an ordered list of `{ target, scope }`, where the scope is `Element` (`expose X`), `Members` (`X::*`), `ElementAndDescendants` (`X::**`, which by KerML includes X) or `Descendants` (`X::*::**`). Read from the API JSON: a `MembershipExpose` is `Element`, or `ElementAndDescendants` when `isRecursive`; a `NamespaceExpose` is `Members`, or `Descendants` when `isRecursive`.
  - `rendering`: the qualified name of the library rendering the element references (`Views::asTreeDiagram`), read from its owned `RenderingUsage`'s reference. Held on views and view definitions alike.
- ADR-0001 still holds: the symbol tree gives views and typing, the API JSON adds exposes and renderings.

### Resolving a view

- A pure function maps a view to either a Diagram Kind with its contents, or an unsupported reason.
- **Kind.** The view's own rendering wins. Otherwise use the first rendering found by walking its type and then that type's specializations, breadth first. `asTreeDiagram` → General, `asInterconnectionDiagram` → Interconnection, any other rendering → unsupported ("renders as `asElementTable`"). With no rendering anywhere, the definition chain decides: reaching `GeneralView` → General, reaching `InterconnectionView` → Interconnection, but `ActionFlowView` and `StateTransitionView` (which specialize `InterconnectionView`) → unsupported. Anything else → unsupported ("no rendering").
- **General contents.** The union of every expose's elements: `Element` adds the target, or the target's members when it's a package; `Members` adds owned members; `ElementAndDescendants` adds the target (unless a package) and all its descendants; `Descendants` adds the descendants alone. Keep only elements a General Diagram draws as nodes; Unsupported Elements and connection-likes become edges or drop out.
- **Interconnection root.** The distinct targets of the exposes, whatever their scope. There must be exactly one, and it must be a part or part def; otherwise the view resolves to an error that names the problem.
- **Unsupported** is distinct from **erroneous**. Unsupported Views are listed and skipped by `--all`. Erroneous supported views (a bad Interconnection root) open with their message and fail export.

### General Diagram over an element set

- The General builder gains an entry point that takes an explicit node set instead of a root's owned members. The existing root-based form becomes "the owned members of the root" passed to it, so Tool-defined Diagrams are unchanged.
- **Composition between usages.** When a node's owner is also a node, a composition edge (filled diamond at the owner) joins them, and the node is left out of the owner's compartments. This is in addition to the existing definition-to-part-type composition.
- Layout keys stay element qualified names. A key exists while the element is in the Model, as for General today.

### Diagram File

- A Diagram File is either tool-defined (`kind`, `root`) or model-defined (`view`). Model-defined files carry `name`, `view` (the view's qualified name) and `layout`, with no kind or root. `format_version` stays 1. The two forms are told apart by which fields are present, so older files still load.
- The default name for a saved Model-defined Diagram is the view's simple name. Naming clashes with existing Diagram Files are handled as they are today.
- When a file's view no longer exists, or no longer resolves to a supported kind, the viewer opens it with that message and keeps the file untouched.

### Viewer

- A **Views** list sits beside the saved Diagrams list. It shows every view by simple name, with Unsupported Views greyed out and their reason as a tooltip. Selecting a supported view opens it.
- Double-clicking a view in the Browser opens it the same way. For an Unsupported View it shows the reason.
- The title shows the view's name and resolved kind. Editing, saving, confirmations and live reload work as for Tool-defined Diagrams. After a reload, the view is resolved again.
- The Properties panel for a view lists its rendering, its resolved kind (or unsupported reason) and one row per expose (`vehicle::**`).

### CLI

- `export --all` writes each saved Diagram File as before, plus each supported view that has no saved Diagram File as `<view simple name>.svg`. A saved Diagram File referring to a view is exported under its own name with its Layout. Unsupported Views produce one warning line each on stderr and don't affect the exit code. Duplicate simple names among exported views, or a clash with a Diagram File name, fail before writing, naming both.
- `export --view <name>` exports one view, by simple or qualified name, to `--out`. It fails for an Unsupported View, an erroneous view, an unknown name, or an ambiguous simple name.

## Testing Decisions

- **Same seams as before.** Seam 1: scenes, resolution results and SVGs from fake-backend Models, plus golden files. Seam 2: contract tests against OpenSysML for the new facts.
- **Seam 1:**
  - Kind resolution: own render, inherited render through a user view def, standard `GeneralView` / `InterconnectionView` typing, `ActionFlowView` unsupported, table rendering unsupported, no rendering unsupported.
  - Expose scopes and unions: `X`, `X::*`, `X::**`, a package exposed alone, overlapping exposes without duplicates, Unsupported Elements dropped.
  - A parts-tree General Diagram (`vehicle::**`): composition edges between usages, and children not repeated in compartments.
  - Interconnection root: one target with `::**`, no target, two targets, and a non-part target.
  - Diagram File round trip for the model-defined form, old tool-defined files still loading, and a missing view.
  - Layout on a Model-defined Diagram (move, hide), with Orphaned Layout Entries after an expose is removed.
  - CLI: `--all` with supported and Unsupported Views, a warning that leaves exit status zero, a duplicate name failing, and `--view` for each failure case.
  - Golden SVGs for a tree view and an interconnection view.
- **Seam 2:** a `views` fixture with a tree view (`expose vehicle::**`), an interconnection view, a view inheriting its render from a user view def, a `GeneralView`-typed view without a render, a view exposing `Pkg::*` plus a single element, and a table view. Its fake twin must match the exposes (targets and scopes), renderings and typing.
- **Prior art:** `viewer-core/tests/general_subset.rs`, `interconnection.rs`, `diagram_file.rs`; `viewer-cli` tests; `viewer-opensysml/tests/contract.rs`.

## Out of Scope

- **View filters** (`filter @SysML::PartUsage;`, `expose vehicle::**[@Safety]`). Recorded as future work: filtered views are drawn unfiltered for now. The Properties panel shows the filter text, so the difference is visible.
- Viewpoints, `satisfy`, and concerns on views.
- Nested views (views owned by views) as anything beyond ordinary views in the list.
- Table, textual, sequence, action-flow, state-transition, geometry and grid renderings.
- The service's own `render_view`.
- Package nodes in General Diagrams.
- A separate tree-diagram kind. `asTreeDiagram` uses the General notation, as checked against the library and the notation slides.

## Further Notes

- Decisions settled with the user on 2026-10-16 (Q1–Q6). Q1 was checked: the standard library defines `asTreeDiagram` only as "a tree diagram, using the graphical notation defined in the SysML specification", and the notation's parts trees are usage nodes with composition edges. That is the General notation, so no separate kind is needed.
- The user asked to keep view filters for a later slice. They are the natural next step for tree views, since most examples pair `asTreeDiagram` with `filter @SysML::PartUsage`.
- Backend probe findings (from this session): view usages arrive as `viewUsage`, a usage inherits its render from its view def, and exposes appear as `MembershipExpose` / `NamespaceExpose` in the API JSON.
