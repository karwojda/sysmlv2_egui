# Spec: SysML v2 Viewer: first slice

Status: ready-for-agent

## Problem Statement

I'm a systems engineer working with SysML v2 models in textual notation. I want to see my models as diagrams and put those diagrams in documentation. The good visualization tools (for example Tom Sawyer) are paid. The free ones either don't render the Standard Notation well, or produce pictures I can't regenerate when the Model changes. As a result my documentation diagrams are drawn by hand, drift away from the Model, and cost time to maintain.

## Solution

A Linux desktop viewer, plus a headless command-line tool, that open a folder of SysML v2 files as a Model.

- The viewer shows the Model as a Browser Diagram (tree) and as General Diagrams in the Standard Notation, arranged automatically.
- When the automatic arrangement is off, I drag nodes into place and save the Diagram. My adjustments go into a Diagram File committed next to the Model. The Model itself is never touched.
- The command-line tool Exports every saved Diagram to SVG, so CI can regenerate documentation images whenever the Model changes.

This spec covers the first vertical slice:

1. Open a folder and load the Model.
2. Browser Diagram.
3. General Diagram with auto-layout.
4. Adjust the Layout and save the Diagram File.
5. `export --all` to SVG.

It also includes the click and navigation behaviour on these Diagrams.

## User Stories

### Opening a Model

1. As a systems engineer, I want to open a folder as a Model, so that every `.sysml` and `.kerml` file under it is loaded together, the way my model lives in git.
2. As a systems engineer, I want the viewer to start the parsing service itself, so that I don't have to launch anything by hand before viewing a model.
3. As a systems engineer, I want to point the viewer at a specific parsing-service binary through configuration, so that I can use a version I installed myself.
4. As a systems engineer, I want the viewer to fetch and cache the matching parsing-service release on first use, or else use one on my `PATH`, so that it works without configuration in the common case.
5. As an advanced user, I want to connect to a parsing service I already have running at a URL, so that I can share one service or debug it.
6. As a systems engineer, I want the parsing service stopped when I close the viewer, so that no stray processes are left behind.
7. As a systems engineer, I want a clear message when the parsing service can't be found or fails to start, so that I know what to fix.
8. As a systems engineer, I want the Model's parse and resolution errors listed with file and line, so that I can fix my sources.
9. As a systems engineer, I want the viewer to reload automatically when files in the Model folder change, so that I can edit in my text editor and see the result alongside.
10. As a systems engineer, I want the last good Diagrams to stay visible while my edit has errors, so that a half-typed change doesn't blank the screen.
11. As a systems engineer, I want the standard library resolved but hidden from the Browser by default, so that my own elements aren't buried.
12. As a systems engineer, I want to be able to show the standard library in the Browser, so that I can inspect library definitions I use.

### Browser Diagram

13. As a systems engineer, I want a Browser Diagram showing the Model's package and element tree, so that I can find my way around the Model.
14. As a systems engineer, I want each Browser entry to show the element's name and kind (for example `part def`, `port`), so that I can tell elements apart at a glance.
15. As a systems engineer, I want Unsupported Elements listed in the Browser with their kind, so that nothing in my Model is silently missing.
16. As a systems engineer, I want to expand and collapse Browser entries, so that I can focus on one area.
17. As a systems engineer, I want to filter the Browser by name, so that I can find an element in a large Model quickly.

### General Diagram

18. As a systems engineer, I want to open a General Diagram for a chosen package or definition, so that I can see its definitions and usages as boxes and edges.
19. As a systems engineer, I want part, attribute and port definitions and usages drawn in the Standard Notation (keywords such as `«part def»`, compartments), so that the pictures read the way readers of SysML expect.
20. As a systems engineer, I want specialization drawn as a hollow-triangle edge, so that type hierarchies are visible.
21. As a systems engineer, I want typing and multiplicity shown on usages, so that I can see what each usage is and how many there are.
22. As a systems engineer, I want composition (a definition owning part usages) drawn as edges in the Standard Notation, so that the structure is visible.
23. As a systems engineer, I want ports shown with their direction, so that I can read flow direction.
24. As a systems engineer, I want connections and interfaces between usages drawn as edges, so that I can see what is connected.
25. As a systems engineer, I want Unsupported Elements left out of General Diagrams, so that the picture contains only what the tool can draw correctly.
26. As a systems engineer, I want the General Diagram arranged automatically when first opened, so that I get a usable picture without any manual work.
27. As a systems engineer, I want to pan and zoom a Diagram, so that I can work with large pictures.

### Adjusting the Layout

28. As a systems engineer, I want to drag nodes to new positions, so that I can fix an automatic arrangement I don't like.
29. As a systems engineer, I want to resize nodes, so that long names and compartments fit.
30. As a systems engineer, I want to hide an element from a Diagram, so that the picture shows only what my document is about.
31. As a systems engineer, I want to show hidden elements again, so that hiding is never a dead end.
32. As a systems engineer, I want to collapse a node's compartments, so that I can make a summary picture.
33. As a systems engineer, I want to reset a Diagram to the automatic arrangement, so that I can start over.
34. As a systems engineer, I want to save a Diagram under a name as a Diagram File, so that my adjustments persist and can be committed with the Model.
35. As a systems engineer, I want saved Diagrams listed when I open the Model, so that I can reopen them directly.
36. As a systems engineer, I want to open a Diagram ad hoc without saving it, so that exploring the Model doesn't litter the folder with files.
37. As a systems engineer, I want the viewer to tell me when a Diagram has unsaved changes, so that I don't lose work.
38. As a systems engineer, I want Diagram Files to be readable text that diffs cleanly, so that layout changes can be reviewed in pull requests.
39. As a systems engineer, I want newly added Model elements auto-placed around my saved positions, so that a Model change doesn't destroy my manual Layout.
40. As a systems engineer, I want Layout entries for renamed or deleted elements kept as Orphaned Layout Entries rather than deleted, so that nothing is lost silently and they can be re-attached later.
41. As a systems engineer, I want the viewer to never modify my `.sysml` files, so that I can trust it with my source.

### Navigation and inspection

42. As a systems engineer, I want to click an element in any Diagram and see a properties panel (kind, name, qualified name, type, multiplicity, documentation, source file and line), so that I can inspect it without opening the source.
43. As a systems engineer, I want selecting an element in the Browser to highlight it in the open Diagram, and the other way round, so that the two stay in sync.
44. As a systems engineer, I want to double-click a type reference to go to its definition, so that I can follow the Model.
45. As a systems engineer, I want an "open in editor" action that opens the element's source line in VS Code, so that I can jump straight to editing it.
46. As a systems engineer, I want double-clicking a part to open its Interconnection Diagram once that Diagram Kind exists, so that navigation reaches into structure (until then the action is unavailable).

### Export

47. As a systems engineer, I want to Export the current Diagram to SVG from the viewer, so that I can paste it into a document.
48. As a systems engineer, I want the GUI Export and the command-line Export of the same Diagram to be identical, so that what I see is what my documents get.
49. As a documentation maintainer, I want a command that Exports one named Diagram File to an SVG path, so that I can script individual images.
50. As a documentation maintainer, I want a command that Exports every Diagram File in a Model to a directory as `<diagram-name>.svg`, so that CI can regenerate all document images with no extra configuration.
51. As a documentation maintainer, I want Exports to be deterministic (the same inputs give a byte-identical SVG), so that regenerated images only show up in diffs when something actually changed.
52. As a documentation maintainer, I want the command to exit non-zero when the Model has errors, so that CI fails instead of publishing stale pictures.
53. As a documentation maintainer, I want the command to exit non-zero and name the Diagram when a Diagram File's root element no longer exists, so that I notice documentation that refers to removed design.
54. As a documentation maintainer, I want the command to run without a display, so that it works in headless CI.
55. As a documentation maintainer, I want the command-line tool to find and manage the parsing service the same way the viewer does, so that CI setup is just "make the binary available".

## Implementation Decisions

### Architecture and modules

- **Model backend interface.** The app owns a small interface for loading a Model from a folder and querying it: elements with kind, name, qualified name, owner, type, multiplicity, specializations, port direction, connection ends, documentation, source location, and diagnostics. Every other module depends only on this interface (ADR-0001). There are two implementations in this slice:
  - the **OpenSysML backend**, which talks to `sysml-grpc` through the `opensysml` Rust client;
  - a **fake backend**, which serves a Model from in-memory fixture data for tests.
- **Sidecar manager.** Starts and stops `sysml-grpc`, delegating to the `opensysml` client. The client looks for the binary in this order: a configured path (`$OPENSYSML_GRPC_BINARY`), its cache in `~/.opensysml/bin`, a digest-pinned download of the matching release from GitHub, then `PATH`. A configured URL skips process management entirely. The viewer and the command-line tool share it.
- **Model watcher.** Watches the Model folder and triggers a reload. A reload that produces errors keeps the last good Model for drawing and publishes the new diagnostics.
- **Diagram builder.** Given a Model, a Diagram Kind and a root element, it decides which elements and relationships appear. It covers the language subset: packages, imports, part/attribute/port definitions and usages, connections and interfaces with `connect`, specialization, typing and multiplicity. Everything outside the subset is an Unsupported Element: listed in the Browser Diagram, excluded from General Diagrams.
- **Auto-layout.** Arranges a General Diagram with an existing pure-Rust layered-layout library. Elements with saved positions in the Layout keep them and act as fixed points. Only elements without a saved position are placed.
- **Layout merge.** Combines a Diagram File's Layout with the current Model. Entries are keyed by element qualified name. Entries that match nothing become Orphaned Layout Entries and are kept in the file. New elements go to auto-layout.
- **Scene.** A renderer-neutral description of a drawn Diagram: shapes, text, edges with routes and end markers, all positioned. Diagram builder, auto-layout and Layout together produce a scene. Nothing upstream of the scene knows about egui or SVG.
- **Renderers.** The egui renderer paints a scene on screen and maps pointer hits back to elements. The SVG renderer serialises a scene deterministically: stable element order, fixed number formatting, no timestamps.
- **Editing commands.** GUI interactions (move, resize, hide, show, collapse, reset, save) are commands applied to the open Diagram's Layout in the core. The egui layer only translates input into commands. Every command produces a new scene.
- **Viewer app (egui/eframe).** Browser panel, Diagram canvas with pan and zoom, properties panel, diagnostics list, saved-Diagram list, unsaved-changes indicator, and "open in editor" (launches `code -g <file>:<line>`).
- **Command-line tool.** `export <model-folder> --diagram <name> --out <file.svg>` and `export <model-folder> --all --out-dir <dir>`. It exits non-zero on Model errors or when a Diagram File's root element is missing, naming the Diagram.

### Diagram File

- There is one Diagram File per saved Diagram, stored beside the Model in a dedicated folder inside the Model folder, and meant to be committed.
- Contents: format version, Diagram name, Diagram Kind, root element qualified name, and the Layout. The Layout is node positions and sizes, edge routes, hidden elements, collapsed elements, and Orphaned Layout Entries.
- The format is a human-readable text format with stable key ordering and one entry per element, so diffs stay reviewable.
- The Diagram name gives the Export file name (`<diagram-name>.svg`).

### Notation

- Diagrams follow the SysML v2 Standard Notation. Simplifications are allowed where faithful rendering is expensive, but shapes, keywords and edge markers must match the spec's intent.
- Style (coloring rules) comes in a later slice. The scene should carry element identity and kind so Styles can be applied later without changing the builder.

### Platform

- Linux only for both the viewer and the command-line tool in this slice. The core must not assume a filesystem or threads in ways that would block a later wasm build.

## Testing Decisions

- **Good tests check external behaviour:** given a Model and a Diagram File (or a Kind and root), what scene and what SVG come out, and what the command-line tool returns. They never assert on internal data structures or private helpers, so the internals can be refactored freely.
- **Seam 1, the Diagram pipeline (the main seam).** Tests build a Model with the fake backend from fixtures, run the pipeline, and assert on the scene and on Exported SVG. This covers:
  - Browser and General content, and Unsupported Element handling
  - Standard Notation shapes and markers
  - auto-layout placing only unplaced elements
  - Layout merge: Orphaned Layout Entries kept, new elements placed, hidden and collapsed elements honoured
  - editing commands, by applying a command and checking the resulting scene and the Diagram File written
  - Export determinism
  - command-line `--all` file naming and exit codes for Model errors and missing root elements

  Exported SVGs are compared to committed golden files. A deliberate notation change updates the golden files in the same commit.
- **Seam 2, the Model backend contract.** Opt-in tests (they need the `sysml-grpc` binary) load fixture `.sysml` folders through the OpenSysML backend and check that the result matches what the fake backend serves for the same fixtures. That keeps the fake honest. They run in CI where the binary is available and are skipped otherwise.
- **Not tested automatically:** egui painting and input translation. That layer stays thin enough to check by eye.
- **Prior art:** none. This is a new codebase, and these tests set the pattern.

## Out of Scope

- Interconnection Diagrams. They're the next slice, possibly preceded by a spike on nested layout with ports on borders. Until then, double-clicking a part doesn't open anything.
- Model-defined Diagrams built from SysML `view` usages.
- Action Flow, State Transition, Sequence and Grid Diagram Kinds.
- Styles (coloring rules) and simplified domain-specific notations.
- PNG Export.
- Generating whole documents (Markdown, HTML, Word) with diagrams and element tables.
- Re-attaching Orphaned Layout Entries to elements.
- Loading Models from the SysML v2 REST API or Flexo MMS, and any alternative backend such as syster-base.
- Editing the Model in any way.
- Windows, macOS and web (wasm) builds.

## Further Notes

- Vocabulary follows the root `CONTEXT.md`: Model, Unsupported Element, Diagram, Diagram Kind, Tool-defined Diagram, Model-defined Diagram, Layout, Diagram File, Orphaned Layout Entry, Standard Notation, Style, Export.
- Relevant ADRs: ADR-0001 (OpenSysML sidecar behind our own Model backend interface), ADR-0002 (the Model is read-only; Layouts live in separate files).
- OpenSysML is young (v0.x) and its Rust client covers parse, diagnostics, symbols and evaluation. Before building on it, confirm that its queries expose everything the Model backend interface needs (owners, typing, specializations, port direction, connection ends, source locations). Gaps should be found early, through the backend contract tests.
- The scene is the shared output of the GUI and the command line. Keeping it free of renderer concerns is what makes "what you see is what your documents get" hold.
