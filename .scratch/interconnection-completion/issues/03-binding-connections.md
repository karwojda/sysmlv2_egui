# 03: Binding connections drawn

**What to build:** Binding connections between drawn ports or parts are plain connectors labelled `=`. Bindings with an end that isn't drawn (such as an attribute value) are left out of Interconnection Diagrams. The General Diagram lists bindings in its "connections" compartment as `bind a = b`.

**Blocked by:** 01: Unnamed connections, flows and bindings in the Model

**Status:** ready-for-agent

- [ ] A binding between two drawn ports is a connector labelled `=`
- [ ] A binding between attributes isn't drawn in an Interconnection Diagram, and is listed in the Browser and Properties panel
- [ ] General compartment rows `bind a = b`
- [ ] Seam 1 tests, and the `flows` golden file updated
