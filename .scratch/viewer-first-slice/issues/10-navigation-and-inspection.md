# 10: Navigation and inspection

**What to build:** The Diagrams become a way to move around the Model. Clicking an element shows a properties panel. Selection stays in sync between the Browser and the open Diagram. Double-clicking a type reference goes to its definition. "Open in editor" jumps to the element's source line in VS Code.

**Blocked by:** 08: General Diagram on the canvas

**Status:** ready-for-agent

- [ ] Clicking an element in a Diagram or the Browser shows its kind, name, qualified name, type, multiplicity, documentation and source file and line
- [ ] Selecting an element in the Browser highlights it in the open Diagram, and the other way round
- [ ] Double-clicking a type reference selects its definition, revealing it in the Browser, and in the Diagram when present
- [ ] "Open in editor" launches `code -g <file>:<line>` for the selected element, and shows a clear message if that fails
- [ ] Double-clicking a part does nothing yet; Interconnection Diagrams are a later slice
- [ ] The properties content and type-reference resolution are tested through the core with the fake backend
