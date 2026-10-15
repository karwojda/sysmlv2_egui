# 11: Live reload

**What to build:** The viewer watches the Model folder and reloads when `.sysml`/`.kerml` files change, so the user can edit in a text editor and see the result alongside. A reload with errors keeps the last good Model for drawing and updates the diagnostics list.

**Blocked by:** 07: Viewer shell

**Status:** ready-for-agent

- [ ] Saving a change to a Model file updates the Browser and open Diagrams without user action
- [ ] Adding or deleting Model files is picked up
- [ ] Bursts of changes (for example an editor save that writes several files) trigger a single reload
- [ ] A reload with errors keeps showing the last good Diagrams and updates the diagnostics list. The next good reload replaces them
- [ ] Open Diagrams keep their Layout across reloads, with merge rules from 06 when available
- [ ] Changes to Diagram Files made outside the viewer don't trigger a Model reload
- [ ] The keep-last-good behaviour is tested through the core with the fake backend
