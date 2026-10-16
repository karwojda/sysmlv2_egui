# 02: Item flows drawn

**What to build:** Item flows are drawn as connectors with a filled arrowhead at the midpoint of their longest run, pointing from the `from` end to the `to` end, labelled `name:Type` (payload), or `:Type` when unnamed. A flow end at an item inside a port (`tank.fuelOut.fuel`) attaches to that port. The General Diagram lists flows in its "connections" compartment (`flow fuelFlow:Fuel (tank.fuelOut → engine.fuelIn)`) and draws flow edges between usage nodes with their arrowhead.

**Blocked by:** 01: Unnamed connections, flows and bindings in the Model

**Status:** ready-for-agent

- [ ] An Interconnection flow connector has a filled triangle at the midpoint of its longest run, pointing toward its target
- [ ] Labels are `name:Payload`, or `:Payload` for an anonymous flow
- [ ] A chain passing a port into an item resolves to the port
- [ ] General compartment rows for flows, and flow edges with their arrowhead between General usage nodes
- [ ] Seam 1 tests and a golden file for the `flows` fixture; no overlapping labels; connectors clear of boxes
