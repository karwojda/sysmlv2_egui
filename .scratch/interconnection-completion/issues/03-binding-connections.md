# 03: Binding connections drawn

**What to build:** Binding connections between drawn ports or parts are plain connectors labelled `=`. Bindings with an end that isn't drawn (such as an attribute value) are left out of Interconnection Diagrams. The General Diagram lists bindings in its "connections" compartment as `bind a = b`.

**Blocked by:** 01: Unnamed connections, flows and bindings in the Model

**Status:** done

- [x] A binding between two drawn ports is a connector labelled `=`
- [x] A binding between attributes isn't drawn in an Interconnection Diagram, and is listed in the Browser and Properties panel
- [x] General compartment rows `bind a = b`
- [x] Seam 1 tests, and the `flows` golden file updated

## Comments

- Done. `resolve` now also reports whether an end is *exactly* the chain's last feature, rather than the nearest drawn port or box containing it. Bindings are drawn only when both ends are exact, so `bind tank.level = engine.fuelLevel` (attributes inside closed parts) is left out, while `bind tank.fuelOut = engine.fuelIn` is drawn. Connections and flows still attach to the nearest drawn port or box.
- A binding's label is `=` (`connector_label`), in Interconnection and General Diagrams.
- General compartment rows: `bind a = b` (with the binding's name after `bind` if it has one).
- The golden `flows-interconnection.svg` was updated.
