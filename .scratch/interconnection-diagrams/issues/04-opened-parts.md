# 04: Opened Parts

**What to build:** A part can be opened in place: drawn with its type's parts and connections inside its box, laid out with the same rules, recursively. Which parts are open is saved in the Layout as feature paths relative to the root (`rearAxleAssembly`, `rearAxleAssembly.differential`). Connection ends that run into a closed part attach to its border. Opening it re-attaches them to the real port.

**Blocked by:** 01: Interconnection tracer; 03: Frame ports and part-level connection ends

**Status:** done

- [x] `Layout::opened` (a set of feature paths) is saved in Diagram Files and omitted when empty
- [x] An Opened Part is sized bottom up: its content plus padding and a title band, with its own ports clear of the title
- [x] Its type's connections are drawn inside it, including those to its own ports (from the inside)
- [x] An end like `rearAxleAssembly.differential.torqueIn` attaches to the port when every part on the way is open, otherwise to the innermost closed part's border
- [x] Two usages of the same definition can be opened independently
- [x] Orphaned `opened` entries are kept
- [x] Golden SVG of the drivetrain with `rearAxleAssembly` opened

## Comments

- Done. `Layout::opened` holds feature paths from the root (`rearAxleAssembly`, `a.b`), is omitted when empty, and keeps unknown entries. `open_box` builds a child as an open box when its path is listed, recursively, with its own wires drawn inside it.
- Wire ends are now paths into the box tree (`End::Port { path, port }`, `End::Part { path }`). `resolve` follows a chain down through open parts to the port, or stops at the innermost closed part (`leftDrive` attaches to the closed `rearAxleAssembly`, and to `leftWheel.hub` once it's open).
- Labels: connector labels are now placed by collision search, stepping up or down a line until they clear box titles, port labels and earlier labels. This replaced pair-based stacking, which missed connectors leaving the same port for different targets (`axle` and `leftDrive`). The room below a part counts all wires leaving the same port.
- Titles, partly ahead of ticket 05: a box's own top ports move off its title text, and a connector entering a box from the top first clears the title band. This fixed `drive` through the axle assembly's title and `supply` through the Frame's title.
- An Opened Part's own ports are placed by its parent (direction, or facing outer peers) and are not re-aligned to inner peers. Only the Frame aligns its own ports.
- Tests: 6 new (inside, own connections, closed → opened ends, saving and unknown entries, golden `drivetrain-opened-interconnection.svg` with no overlapping labels, two usages opened independently).
- For ticket 08: two opened usages of one definition draw their inner parts with the same `Depicts::Element` (the inner usage's qualified name). Selection and Layout keys need feature paths.
- Left for ticket 05: `leftDrive` crosses the differential and the axle assembly's title band, and `supply` crosses the fuel tank.
