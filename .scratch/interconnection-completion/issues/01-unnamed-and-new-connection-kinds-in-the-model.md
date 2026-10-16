# 01: Unnamed connections, flows and bindings in the Model

**What to build:** The Model gains every connection-like element, named or not, and the facts the new notation needs. The OpenSysML backend reads unnamed connections, interfaces, flows and bindings from the API JSON element form, using the service's generated qualified names (`Vehicle::@6`). It also reads flow ends (a reference to a part, plus the port or item the nested reference redefines), item types (`payload`, from the flow's type or its `of X` payload) and `is_reference`. There are new element kinds `FlowUsage` (`flow`) and `BindingConnection` (`bind`). Unnamed connections are drawn in Interconnection and General Diagrams without a name label, which closes the gap that loses wiring today. See `.scratch/interconnection-completion/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `ElementKind::FlowUsage` and `ElementKind::BindingConnection` exist and count as connection-like, with ends drawn as connectors
- [x] `Element` gains `is_reference: bool` and `payload: Option<QualifiedName>`
- [x] The backend adds unnamed connection, interface, flow and binding usages from the API JSON, with generated qualified name, empty name, owner, type, ends and source line
- [x] Flow ends resolve to their full path (`tank.fuelOut` → `[Vehicle::tank, Tank::fuelOut]`), and payloads and `is_reference` are read
- [x] Unnamed connectors are drawn in both Diagram kinds with no name label (`:Type` if typed)
- [x] A `flows` `.sysml` fixture and its fake twin (unnamed connection, named and anonymous flow, port and attribute bindings, a reference part, a nested port) pass a contract test

## Comments

- Done. Model: `ElementKind::FlowUsage` (`flow`) and `BindingConnection` (`bind`) count as connection-like (`is_connection`). `Element` gains `is_reference` and `payload`. `Element::display_name()` names an unnamed element by its ends (`a → b`, or `a = b` for a binding); the Browser uses it, and the Properties panel shows "(unnamed)".
- Backend (`api_json`):
  - Unnamed `ConnectionUsage`, `InterfaceUsage`, `FlowUsage` and `BindingConnectorAsUsage` items are added as elements under their generated names, with an empty name and their type.
  - A flow end's path is its reference subsetting chain plus the feature its nested `ReferenceUsage` redefines (`tank.fuelOut.fuel` → `[tank, fuelOut, fuel]`).
  - A flow's payload comes from its `PayloadFeature`, else its type. `isReference` sets `is_reference`.
  - The symbol kinds `flowUsage` and `bindingConnector(AsUsage)` are mapped.
- Drawing: connectors without a name or type have no label item, in either Diagram kind. Compartment rows for unnamed connections start with their ends.
- Fixture `viewer-opensysml/tests/fixtures/flows/flows.sysml` has an anonymous item flow, a named flow (`flow fuelFlow of Fuel ...`; `: X` would type it by a flow *definition*, which the service rejects), port and attribute bindings, an unnamed connection between nested ports, and a reference part. Its twin is `viewer-core/tests/common/mod.rs::flows()`; the contract test passes.
- Until tickets 02, 03 and 05: flows and bindings are drawn as plain connectors. Ends that pass a port into an item, attribute or nested port attach to the part's border.
- Tests: 5 in `viewer-core/tests/flows.rs`, plus the contract test.
- Side note: a throwaway probe resolved `opensysml` 0.10.1, which swapped the shared service cache in `~/.opensysml/bin`. The project still pins 0.10.0 and swaps it back on its next run.
