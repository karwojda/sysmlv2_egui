# 01: Unnamed connections, flows and bindings in the Model

**What to build:** The Model gains every connection-like element, named or not, and the facts the new notation needs. The OpenSysML backend reads unnamed connections, interfaces, flows and bindings from the API JSON element form, using the service's generated qualified names (`Vehicle::@6`). It also reads flow ends (a reference to a part, plus the port or item the nested reference redefines), item types (`payload`, from the flow's type or its `of X` payload) and `is_reference`. There are new element kinds `FlowUsage` (`flow`) and `BindingConnection` (`bind`). Unnamed connections are drawn in Interconnection and General Diagrams without a name label, which closes the gap that loses wiring today. See `.scratch/interconnection-completion/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] `ElementKind::FlowUsage` and `ElementKind::BindingConnection` exist and count as connection-like, with ends drawn as connectors
- [ ] `Element` gains `is_reference: bool` and `payload: Option<QualifiedName>`
- [ ] The backend adds unnamed connection, interface, flow and binding usages from the API JSON, with generated qualified name, empty name, owner, type, ends and source line
- [ ] Flow ends resolve to their full path (`tank.fuelOut` → `[Vehicle::tank, Tank::fuelOut]`), and payloads and `is_reference` are read
- [ ] Unnamed connectors are drawn in both Diagram kinds with no name label (`:Type` if typed)
- [ ] A `flows` `.sysml` fixture and its fake twin (unnamed connection, named and anonymous flow, port and attribute bindings, a reference part, a nested port) pass a contract test
