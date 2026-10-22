# Engineering Tools are built-in adapters run during load

Each Engineering Tool is reached through an adapter built into this program behind one Rust trait (`EngineeringTool`: a file and the properties wanted per object in, values with units or reasons out). The OpenSysML backend runs the adapters inside `load_with`, once per load and one process per tool for all its files, then applies the Design Values by an in-memory edit and re-parses (ADR 0003), so a load returns one result and the GUI's existing worker thread keeps it off the UI. Tool executables are found per machine (an environment variable per tool, then `PATH`, then known install locations), never through the Model. We considered a plug-in protocol, where any executable speaking our JSON could be an adapter, but that makes us own a public protocol for two tools we write ourselves; it can be drawn out of the trait if an outside tool ever needs it. We considered putting the integration behind a Model server, but that breaks local-first and keeping `.sysml` files in git. We considered keeping tool processes alive between loads, and a two-phase load that shows the Model before its checks; both add lifecycle or in-between states that a measured ~0.65 s per FreeCAD run doesn't yet justify.

## Consequences

- The list of Design Values is a store-neutral change set (feature qualified name, value text). A future write to the Model applies the same change set and saves the edited text instead of only re-parsing it; that needs a new ADR amending ADR 0002.
- Adapters know nothing of SysML; the edit, the "no Design Value" marker and the re-parse are OpenSysML-specific and stay in the OpenSysML backend.
- A Model with Tool Links loads about 0.7 s slower (one FreeCAD run); one without loads as before.
