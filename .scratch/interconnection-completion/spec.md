# Spec: Interconnection completion

Status: done

## Problem Statement

My Interconnection Diagrams show parts, ports and named connections, but the wiring in real models is often missing or wrong:

- **Unnamed connections are missing.** `connect a to b;` written without a name, the usual way, isn't drawn at all, because the viewer never reads it from the Model.
- **Missing notation.** Item flows (what moves along a connection), binding connections (`=`), reference parts (parts the system uses but doesn't own) and nested ports (ports inside ports) don't appear, though the reference notation (slides 40–44) shows all of them.

My documents can't show how things actually flow through the system.

## Solution

Complete the Interconnection Diagram notation and fill the gap in the Model:

- **Unnamed connections**, flows and bindings are read into the Model and drawn. Unnamed ones carry no name label.
- **Item flows** are connectors with a filled arrowhead in the middle, pointing from source to target, labelled with the item (`fuel:Fuel`, or `:Fuel` when anonymous).
- **Binding connections** are plain connectors marked `=`. They're drawn when both ends are drawn parts or ports.
- **Reference parts** are dashed rounded boxes, in Interconnection and General Diagrams alike.
- **A port with nested ports** is drawn as a bar along its box's border, with its nested ports as small squares on the bar's outer edge. Connections attach to the nested port they name, or to the bar.
- **General Diagrams** list flows and bindings in their "connections" compartment, alongside unnamed connections.

## User Stories

1. As a systems engineer, I want connections I wrote without a name (`connect a to b;`) drawn like named ones, so that my wiring isn't silently missing.
2. As a systems engineer, I want unnamed connectors drawn without a name label (with their type if they have one), so that the picture doesn't show machine-made names.
3. As a systems engineer, I want unnamed connections, flows and bindings listed in the Browser and inspectable in the Properties panel, so that everything in my Model can be found.
4. As a systems engineer, I want item flows drawn as connectors with a filled arrowhead in the middle pointing from source to target, so that I can see which way things move.
5. As a systems engineer, I want an item flow labelled with its item (`fuel:Fuel`, or `:Fuel` for `flow of Fuel`), so that I can see what moves.
6. As a systems engineer, I want a flow from an item inside a port (`tank.fuelOut.fuel`) to attach to that port, so that flows written at item level still show between the right ports.
7. As a systems engineer, I want binding connections between ports or parts drawn as connectors marked `=`, so that I can see which features are bound equal.
8. As a systems engineer, I want bindings between attribute values left out of Interconnection Diagrams but listed in the Properties panel and the Browser, so that the picture stays about wiring and nothing is lost.
9. As a systems engineer, I want reference parts (`ref part driver`) drawn as dashed rounded boxes, so that I can tell what the system uses from what it owns.
10. As a systems engineer, I want reference parts dashed in General Diagrams too, so that the distinction holds everywhere.
11. As a systems engineer, I want the Properties panel to say when a part is a reference, so that I can check it.
12. As a systems engineer, I want a port that has ports of its own drawn as a bar along its box's border, with its nested ports as small squares on the bar's outer edge, so that composite ports read like the reference notation.
13. As a systems engineer, I want a connection to a nested port (`tank.fuelOut.inner`) to attach to that nested port's square, and one to the composite port itself to the bar, so that I can see exactly where each wire goes.
14. As a systems engineer, I want nested ports labelled `name : Type` like other ports, so that each can be identified.
15. As a systems engineer, I want the General Diagram's "connections" compartment to list flows (`flow fuelFlow:Fuel (tank.fuelOut → engine.fuelIn)`) and bindings (`bind tank.level = engine.fuelLevel`), so that a definition's wiring is visible there too.
16. As a documentation maintainer, I want all of this in the CLI Export, deterministic and identical to the viewer, so that my documents show the complete wiring.
17. As a systems engineer, I want a hand-placed bend on an unnamed connector to stay with it while the file isn't edited, and to become an Orphaned Layout Entry if an edit renumbers it, so that nothing breaks silently.

## Implementation Decisions

### Model

- New element kinds: `FlowUsage` (keyword `flow`) and `BindingConnection` (keyword `bind`). Connections, interfaces, flows and bindings are all "connection-like": they have ends and are drawn as connectors.
- New element facts:
  - `is_reference` (a `ref` part).
  - `payload`: a flow's item type, from its own type or, for `flow of X`, its payload feature.
- Unnamed connection-like elements are read from the API JSON element form (they aren't in the symbol tree). Each gets the service's generated qualified name (`Vehicle::@6`, numbered by position), an empty name, its owner, type, ends and source line.
- Flow ends are stored differently from connection ends. A flow end references a part (or a chain), and its nested reference redefines the port or item reached, so the end's path is the reference followed by the redefined feature. A binding's ends follow the same reference-subsetting chains as connections.
- `is_reference` comes from the API JSON's `isReference`.
- ADR-0001 still holds. The symbol tree stays the source of kinds and typing for named elements; the API JSON adds unnamed elements and the facts above.

### Interconnection Diagram

- Wires are built from every connection-like owned element. A binding is kept only if both ends resolve to drawn ports or parts. A chain that passes a port and continues into an item or attribute (`tank.fuelOut.fuel`) resolves to that port.
- **Item flows:** the connector gets a filled triangle at the midpoint of its longest run, pointing toward the target end. Source and target are the flow's `from` and `to`, its first and second end. The label is `name:Type` from the payload, or `:Type` when the flow has no name.
- **Bindings:** a plain connector whose label is `=`.
- **Unnamed connections:** no name label, only `:Type` if typed.
- **Reference parts:** their boxes are dashed. The scene's rectangle gains a dashed style, honoured by both renderers.
- **Nested ports:** a port whose type owns ports is drawn as a bar along the border, long enough for its nested ports. Nested ports are small squares on the bar's outer edge, labelled outside. Port keys are feature paths (`engine.fuelIn.inner`). Ends resolve down the chain to the nested port, or stop at the bar. Ports nest only one level in this slice; a nested port that itself has ports draws as a plain square.
- Routing, label placement and Layout behave as before. Bends, port moves and hiding apply to all new connector kinds and to nested ports' parents.

### General Diagram

- **Connections compartment:** flow rows read `flow name:Payload (a → b)` and binding rows `bind a = b`. Unnamed rows drop the name.
- **Connection edges** between usage nodes include flows (with their arrowhead) and bindings (`=`).
- **Reference part usages** are drawn dashed.

### Persistence

- Unchanged formats. Layout keys for unnamed connectors use their generated qualified name, so an edit that renumbers them makes their entries Orphaned Layout Entries.

## Testing Decisions

- **Same seams as before.** Seam 1: scenes and SVG from fake-backend Models, plus golden files. Seam 2: contract tests against OpenSysML.
- **Seam 1:**
  - unnamed connections drawn without a name
  - a flow's arrowhead (filled, midpoint, pointing to the target) and label
  - flow ends at items resolving to their port
  - bindings drawn `=` between ports, and left out between attributes
  - reference parts dashed (Interconnection and General)
  - nested ports drawn as a bar with squares, with ends attaching to the right square or the bar
  - General compartment rows
  - no overlapping labels, and connectors still clear of boxes
  - golden files for a "flows" fixture
- **Seam 2:** a `.sysml` fixture with unnamed connections, named and anonymous flows, a binding between ports, a binding between attributes, a reference part and a nested port. Its fake twin must match, including generated names, payloads, `is_reference` and the flow end paths.
- **Prior art:** `viewer-core/tests/interconnection.rs`, `interconnection_editing.rs`, `general_subset.rs`; `viewer-opensysml/tests/contract.rs`.

## Out of Scope

- Items drawn inside parts (`«item» fuel:Fuel`).
- Flow definitions.
- Succession flows and message flows.
- Ports nested more than one level.
- Stable identities for unnamed connectors across edits.

## Further Notes

- Decisions settled with the user on 2026-10-16 (Q1–Q7). The unnamed-connection gap, found by probing the backend, comes first, because it affects what already ships.
- Notation reference: slides 40–44 of the Intro to the SysML v2 Language — Graphical Notation.
