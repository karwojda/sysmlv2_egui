# 02: Item flows drawn

**What to build:** Item flows are drawn as connectors with a filled arrowhead at the midpoint of their longest run, pointing from the `from` end to the `to` end, labelled `name:Type` (payload), or `:Type` when unnamed. A flow end at an item inside a port (`tank.fuelOut.fuel`) attaches to that port. The General Diagram lists flows in its "connections" compartment (`flow fuelFlow:Fuel (tank.fuelOut → engine.fuelIn)`) and draws flow edges between usage nodes with their arrowhead.

**Blocked by:** 01: Unnamed connections, flows and bindings in the Model

**Status:** done

- [x] An Interconnection flow connector has a filled triangle at the midpoint of its longest run, pointing toward its target
- [x] Labels are `name:Payload`, or `:Payload` for an anonymous flow
- [x] A chain passing a port into an item resolves to the port
- [x] General compartment rows for flows, and flow edges with their arrowhead between General usage nodes
- [x] Seam 1 tests and a golden file for the `flows` fixture; no overlapping labels; connectors clear of boxes

## Comments

- Done. `notation::flow_arrow(points)` draws a filled triangle at the middle of the path's longest run, pointing along the path (from the flow's first end to its second). Both Diagram kinds use it. `notation::connector_label` gives a flow's `name:Payload` (`:Payload` unnamed), otherwise `name:Type`.
- Chain resolution: a chain that reaches a port and continues (into an item it carries) now resolves to that port, at any depth.
- General: the "connections" compartment includes flows (`flow fuelFlow:Fuel (tank.fuelOut → engine.fuelIn)`, `flow :Fuel (…)`), and flow edges between usage nodes get the arrowhead.
- Golden `flows-interconnection.svg`, with no overlapping labels.
- Known limit: wires over the same pair of ports share one line, so parallel flows' arrowheads coincide; their stacked labels tell them apart.
