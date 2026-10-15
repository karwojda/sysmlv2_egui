# 07: Viewer shell: open a Model, Browser Diagram, diagnostics

**What to build:** The egui/eframe desktop viewer. The user opens a folder as a Model, and the viewer starts the sidecar and loads the Model. It shows the Browser Diagram (the package and element tree) and a diagnostics list.

**Blocked by:** 02: OpenSysML backend and sidecar manager

**Status:** ready-for-agent

- [ ] The user can open a folder as a Model from the viewer
- [ ] The viewer starts the sidecar on open and stops it on exit
- [ ] The Browser Diagram shows the Model tree, with each entry's name and kind
- [ ] Unsupported Elements are listed in the Browser with their kind
- [ ] Entries can be expanded and collapsed
- [ ] A name filter narrows the Browser to matching entries and their ancestors
- [ ] A diagnostics list shows Model errors with file and line
- [ ] Browser content (tree, filter) is built in the core and tested through the fake backend. The egui layer only paints it
