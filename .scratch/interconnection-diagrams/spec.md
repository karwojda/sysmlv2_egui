# Spec: Interconnection Diagrams

Status: done

## Problem Statement

I'm a systems engineer documenting how my system is wired. The General Diagram shows what parts a definition has, but not how they connect: which port of the fuel tank feeds which port of the engine, and how torque flows through the drivetrain. That is the picture my reviewers and my documents need most, and today I have to draw it by hand. Hand-drawn wiring pictures go stale the moment the Model changes, and nobody notices.

## Solution

A new Diagram Kind, the Interconnection Diagram, following the SysML v2 Standard Notation. It is rooted at a part definition or a part usage.

- **Frame:** the root part's box. Its parts are drawn inside it as rounded boxes, with their ports as small squares on their borders.
- **Connections and interfaces** are drawn port to port (or part to part) as right-angled connectors.
- **Layering:** parts are laid out top to bottom following the connections. `in` ports sit on the top edge, `out` ports on the bottom edge, and undirected ports face whatever they connect to.
- **Opening parts:** any part can be opened in place to show its own parts and connections inside its box, as deep as the document needs.
- **Adjusting:** like General Diagrams, the user can move, resize and hide parts, drag ports along their border, and bend connectors. All of it is saved in the Diagram File, and the CLI Exports it like any other Diagram.

The layout approach was chosen by a spike (variant E, top to bottom) on the throwaway branch `prototype/interconnection-layout`. Comparison page: https://claude.ai/artifact/EGBz5JtFmGJ155tFDE5dTn

## User Stories

### Opening

1. As a systems engineer, I want to open an Interconnection Diagram of a part definition, so that I can see how its parts are wired.
2. As a systems engineer, I want to open an Interconnection Diagram of a part usage, showing the parts of the definition it's typed by, so that I can start from the part I'm looking at.
3. As a systems engineer, I want to double-click a part, or a part definition, in a General or Interconnection Diagram to open its Interconnection Diagram, so that I can go from structure to wiring in one gesture.
4. As a systems engineer, I want "Open Interconnection Diagram" in the Browser's context menu and the Properties panel, so that I can open one from anywhere I find a part.
5. As a systems engineer, I want `sysml-viewer-gui <folder> <root>` to be able to open an Interconnection Diagram directly, so that I can script my review setup.
6. As a systems engineer, I want an Interconnection Diagram of something without parts to say so, so that an empty picture isn't mistaken for a bug.

### What is drawn

7. As a systems engineer, I want the root drawn as the Frame, labelled `«part» name:Type` (or `«part def» Name`), with the parts inside it, so that the picture reads like the reference notation.
8. As a systems engineer, I want each part drawn as a rounded box labelled `«part» name:Type [multiplicity]`, so that I can tell parts and their definitions apart.
9. As a systems engineer, I want each part's ports drawn as small squares on its border, labelled `name : Type` (with `~` for a conjugated type), so that I can see where connections attach.
10. As a systems engineer, I want the root's own ports drawn on the Frame's border, with connections from inner parts to them, so that I can see what the system exposes.
11. As a systems engineer, I want connections drawn between the ports (or parts) they join, labelled with their name and type, so that I can trace each wire.
12. As a systems engineer, I want interfaces drawn the same way as connections, labelled with their interface definition, so that I can see which interface governs a link.
13. As a systems engineer, I want several connections over the same ports to keep separate, readable labels, so that parallel wires don't turn into a smudge.
14. As a systems engineer, I want parts and ports of elements I haven't modelled in the supported subset left out, so that only what the tool draws correctly appears.

### Layout

15. As a systems engineer, I want parts arranged top to bottom following the connections (from `out` ports to `in` ports), so that flow reads downward.
16. As a systems engineer, I want `in` ports on a part's top edge and `out` ports on its bottom edge, so that connectors run straight down between layers.
17. As a systems engineer, I want undirected ports to face whatever they connect to, so that their connectors stay short.
18. As a systems engineer, I want unconnected ports placed by direction too, with undirected ones on the bottom edge, so that every port has a predictable place.
19. As a systems engineer, I want boxes wide enough for their port labels, so that labels never overlap each other.
20. As a systems engineer, I want connectors drawn with right angles, so that the picture looks like a block diagram.
21. As a systems engineer, I want connectors to route around parts rather than through them, so that a wire is never hidden behind a box.
22. As a systems engineer, I want labels to stay clear of box titles and of each other, so that every label is readable.
23. As a systems engineer, I want an Interconnection Diagram to stay compact, roughly page-shaped, so that I can put it in a document.

### Opened Parts

24. As a systems engineer, I want parts closed by default, so that a first picture stays readable.
25. As a systems engineer, I want to open a part in place, from its context menu, to see its own parts and connections inside its box, so that I can show nested structure where my document needs it.
26. As a systems engineer, I want to close an Opened Part again, so that I can go back to the overview.
27. As a systems engineer, I want which parts are open saved in the Diagram File, so that my Export shows the same depth every time.
28. As a systems engineer, I want a connection whose end is inside a closed part to attach to that part's border, so that I can still see that something connects there.
29. As a systems engineer, I want opening that part to re-attach the connection to the real port inside, so that the picture becomes exact as I go deeper.

### Adjusting and saving

30. As a systems engineer, I want to move, resize and hide parts in an Interconnection Diagram as in a General Diagram, so that I can fix the automatic arrangement.
31. As a systems engineer, I want moving a part inside an Opened Part to keep it inside that part, so that nesting stays true.
32. As a systems engineer, I want to drag a port along its part's border, saved in the Layout, so that I can untangle connectors myself.
33. As a systems engineer, I want to bend connectors by hand as in General Diagrams, saved in the Layout, so that I control the final routing.
34. As a systems engineer, I want to save an Interconnection Diagram as a Diagram File with `"kind": "interconnection"`, so that it is listed and reopened like any other.
35. As a documentation maintainer, I want `sysml-viewer export` to Export Interconnection Diagrams like any other Diagram File, with the same guarantees (identical to the viewer, deterministic, failing on a missing root), so that my CI keeps documents current.
36. As a documentation maintainer, I want an Interconnection Diagram whose root is no longer a part to fail the Export clearly, so that stale documents are noticed.

### Navigation

37. As a systems engineer, I want clicking a part, port or connection to show its properties, so that I can inspect it like in the General Diagram.
38. As a systems engineer, I want double-clicking a part in an Interconnection Diagram to open that part's own Interconnection Diagram, so that I can walk down the system one level at a time, as an alternative to opening it in place.

## Implementation Decisions

### Model and Diagram Kind

- `DiagramKind::Interconnection`, serialised as `"interconnection"`. `accepts_root` takes a part definition, or a part usage typed by a part definition.
- No new Model facts are needed. Parts, ports (direction, type, conjugation), multiplicity, and connection and interface ends as feature chains are already in the Model (slice 1, ticket 04). The root's parts come from its definition: for a part usage, the definition it is typed by. A closed or opened part's ports and parts come from its own type in the same way.
- Connection ends are resolved against the drawn boxes. An end is a chain of features (`rearAxleAssembly.differential.torqueIn`). It attaches to the deepest drawn box on the chain: the port if every part on the way is open, otherwise the border of the innermost closed part. A chain that starts at one of the root's own ports attaches to the Frame.
- Connections are drawn in the box that owns them: the root definition's connections inside the Frame, and an Opened Part's type's connections inside that part.

### Building and layout

- **Interconnection builder:** a new module beside the General builder, producing the same renderer-neutral scene, so both renderers, Export determinism and golden tests work unchanged.
- **Bottom-up layout:** closed parts are sized first, then each Opened Part, then the Frame.
  - A closed part's width fits its title and the labels of its top and bottom ports.
  - Inside a box, parts are layered top to bottom with the existing layered-layout library. Edges run from the `out` end to the `in` end, with a label line of room between layers.
  - An Opened Part's size is its content plus padding and a title band. Its own ports stay clear of the title band.
- **Ports:** `in` on the top edge, `out` on the bottom edge. Undirected ports face their peer: the side of the box nearest the other end. Unconnected undirected ports go on the bottom edge. Ports on each edge are spaced evenly, in the order of their peers, to avoid crossings.
- **Connectors:** right angles. Out of the port, along the gap between layers, into the other port. They are routed around boxes using the same obstacle-aware approach as General Diagram edges (adapted to right angles). Connections over the same ports stack their labels. Connector labels sit beside their middle segment, clear of port labels and box titles.
- **Scene identity:** the scene depicts parts as elements, by usage qualified name, and ports as elements too, so hit-testing and the Properties panel work for both. Connectors depict their connection element.

### Layout persistence

- The existing Layout (positions, sizes, edge bends, hidden) applies unchanged. Interconnection positions are relative to the enclosing box's content area, so moving an Opened Part moves its insides with it.
- New Layout entries, empty by default and omitted from the file when empty:
  - `opened`: a set of feature paths (`rearAxleAssembly`, `rearAxleAssembly.differential`), identifying a part as nested in this Diagram. Two usages of the same definition can be opened independently.
  - `ports`: port placement overrides (side and position along it) for ports moved by hand, keyed by feature path.
- Layout keys for an Interconnection Diagram are feature paths relative to the root, not qualified names, because the same part definition can appear several times in one Diagram.
- Orphaned Layout Entries are kept, as for General Diagrams.

### Viewer

- Opening: double-click a part or part def on any canvas; "Open Interconnection Diagram" in the Browser context menu and the Properties panel; `sysml-viewer-gui <folder> <root>` keeps opening a General Diagram for packages and definitions, and opens an Interconnection Diagram when the root is a part usage, or with `--interconnection`.
- Canvas: the part context menu gains Open/Close. Ports can be dragged along their border. Existing move, resize, hide, bend and Reset gestures apply to Interconnection Diagrams.
- Tabs show `Name (Interconnection)` for ad hoc Diagrams.

### CLI

- No changes beyond the Diagram Kind: `export --diagram/--all` builds whatever kind the Diagram File names. A root that's no longer a part (or a part usage without a definition) fails like a missing root, naming the Diagram.

## Testing Decisions

- **Same seams as slice 1.** Seam 1 builds the scene from a fake-backend Model (with a Layout) and checks the scene and golden SVGs. Seam 2 is the opt-in OpenSysML contract tests. Tests check external behaviour only: what is drawn where, what the Diagram File holds, what the CLI returns.
- **Seam 1 tests:**
  - Interconnection content for a part def root and a part usage root.
  - Ports per side by direction.
  - End resolution through closed and opened parts.
  - Frame ports.
  - Connectors' right angles and their clearance from boxes.
  - Stacked labels.
  - No overlapping boxes or labels.
  - Opened parts in the Layout.
  - Moving a part moves its insides.
  - Port overrides.
  - Determinism.
  - Golden files: the `structure` fixture's `Vehicle`, and a drivetrain fixture after slide 40 (closed and with `rearAxleAssembly` opened). The drivetrain Model from the spike, as a fake-backend fixture.
- **Seam 2:** add the drivetrain as a `.sysml` fixture, and check the real backend's Model matches the fake one, including chained connection ends three features deep.
- **Prior art:** `viewer-core/tests/general_diagram.rs`, `general_subset.rs`, `layout.rs`, `editing.rs`, `routing.rs`; `viewer-cli/tests/export*.rs`; `viewer-app/tests/*` for the viewer's model-level tests. Canvas gestures are checked by hand (the user verifies), as in slice 1.

## Out of Scope

- Item flows (`flow of … from … to …`) with their arrowhead and item label.
- Binding connections (`=`).
- Reference parts drawn dashed.
- Nested ports (ports inside ports).
- Items drawn inside parts (`«item» fuel:Fuel`).
- Model-defined Interconnection Diagrams from SysML `view`s.
- Automatically choosing which parts to open.

## Further Notes

- Vocabulary follows `CONTEXT.md`, including two new terms: Frame and Opened Part.
- The spike's remaining rough spots are the requirements of user stories 21 and 22: a connector from the root's own port crossing a part, a nested part's port label on its title, and the hook on undirected side ports.
- Q1–Q7 were settled with the user on 2026-10-15:
  - roots are part defs and part usages
  - parts are closed by default, opened in place, saved in the Layout
  - ports by direction, undirected ones facing their peer, draggable
  - hidden ends attach to the innermost visible box
  - connections and interfaces now; flows, bindings and references later
  - double-click opens; the kind is `interconnection`
  - the root's ports go on the Frame
