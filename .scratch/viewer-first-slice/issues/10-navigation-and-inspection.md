# 10: Navigation and inspection

**What to build:** The Diagrams become a way to move around the Model. Clicking an element shows a properties panel. Selection stays in sync between the Browser and the open Diagram. Double-clicking a type reference goes to its definition. "Open in editor" jumps to the element's source line in VS Code.

**Blocked by:** 08: General Diagram on the canvas

**Status:** done

- [x] Clicking an element in a Diagram or the Browser shows its kind, name, qualified name, type, multiplicity, documentation and source file and line
- [x] Selecting an element in the Browser highlights it in the open Diagram, and the other way round
- [x] Double-clicking a type reference selects its definition, revealing it in the Browser, and in the Diagram when present
- [x] "Open in editor" launches `code -g <file>:<line>` for the selected element, and shows a clear message if that fails
- [x] Double-clicking a part does nothing yet; Interconnection Diagrams are a later slice
- [x] The properties content and type-reference resolution are tested through the core with the fake backend

## Comments

- Done in code; tested through the core; not yet seen on screen (see the last note).
- Core `viewer_core::properties::properties(model, element)` lists Kind, Name, Qualified name, Owner, Type (with `~`), Multiplicity, Direction, Specializes, Connects/To (ends), Documentation and Source, each only when present. Owner, Type and Specializes refer to their element when the Model has it. Seven tests in `viewer-core/tests/properties.rs`.
- The Properties panel is on the right. Its references are links, and following one selects that element (this is how a type reference leads to its definition). There's also an "Open General Diagram" button for packages and definitions.
- Selection is shared. A click on the canvas, or a followed link, selects the element, opens its owners in the Browser and scrolls it into view. A Browser click highlights it in the open Diagram.
- "Open in editor" runs `code -g <file>:<line>`. On failure it says so and asks whether `code` is on PATH. It's disabled, with a tooltip, when the element has no source location.
- Double-clicking a part on the canvas does nothing (the Interconnection Diagram comes later). Double-click on the canvas only removes bends.
- `sysml-viewer-gui <folder> <root>` also selects the root, so its properties show on start.
- **Unverified on screen:** from 23:02 on, every Wayland launch made the WSLg compositor (Weston) segfault (`/mnt/wslg/stderr.log`, "terminated with signal 11"). Every crash since 22:12 lines up with a viewer launch. The workaround is X11 via XWayland, which needs `libxkbcommon-x11-0`.
