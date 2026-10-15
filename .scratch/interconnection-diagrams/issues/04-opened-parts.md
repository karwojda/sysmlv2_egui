# 04: Opened Parts

**What to build:** A part can be opened in place: drawn with its type's parts and connections inside its box, laid out with the same rules, recursively. Which parts are open is saved in the Layout as feature paths relative to the root (`rearAxleAssembly`, `rearAxleAssembly.differential`). Connection ends that run into a closed part attach to its border. Opening it re-attaches them to the real port.

**Blocked by:** 01: Interconnection tracer; 03: Frame ports and part-level connection ends

**Status:** needs-triage

- [ ] `Layout::opened` (a set of feature paths) is saved in Diagram Files and omitted when empty
- [ ] An Opened Part is sized bottom up: its content plus padding and a title band, with its own ports clear of the title
- [ ] Its type's connections are drawn inside it, including those to its own ports (from the inside)
- [ ] An end like `rearAxleAssembly.differential.torqueIn` attaches to the port when every part on the way is open, otherwise to the innermost closed part's border
- [ ] Two usages of the same definition can be opened independently
- [ ] Orphaned `opened` entries are kept
- [ ] Golden SVG of the drivetrain with `rearAxleAssembly` opened
