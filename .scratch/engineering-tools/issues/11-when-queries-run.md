# When do Tool Link queries run?

**Status:** resolved
**Type:** grilling
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

Queries run inside every load (see "Where does the integration code run?"), and the GUI reloads when a `.sysml` file changes. Should a change to a linked `.FCStd` file also trigger a reload, and how is it watched? Are Design Values cached between loads (keyed by file and modification time, or a content hash) so a `.sysml` edit doesn't rerun FreeCAD? Does a Tool Link need a stored fingerprint to detect that the design changed since someone last looked (prior art: Cameo suspect links), or is a live query enough? What does the CLI do differently, if anything?

## Answer

**On every load, with linked files watched and answers cached in memory.** Resolved by grilling; no ADR (each point is cheap to reverse and follows from ADRs 0003 and 0005).

1. **Saving a linked file reloads the Model.** The GUI watcher's fingerprint (`viewer_core::watch::fingerprint`, modification time plus size, polled every 500 ms) also covers every file the last load's Tool Links named, wherever it lives (`../cad/engine.FCStd`). Only linked files, not FreeCAD's `.FCBak` backups or whole CAD folders.
2. **Design Values come from saved files only.** Unsaved changes in an open FreeCAD window are invisible; the saved file is what CI and other engineers see.
3. **In-memory cache in the GUI**, keyed by file path, modification time and size, plus the objects and properties asked for. A `.sysml` edit reuses the last answers unless a link now asks for something new. Nothing on disk; the CLI never caches, each run asks live. Failed runs aren't cached.
4. **No fingerprint on the Tool Link.** The Model is read-only to this tool (ADR 0002), and the live query shows the current design beside the Model Value: that difference is the staleness signal. "Changed since accepted" belongs to a future write.
5. **Timeout per tool run: 30 s by default** (about 45× the measured run), changeable per machine by environment variable like the tool's location. A timeout means no Design Value for that tool's links, so dependent checks are Undecided with "FreeCAD did not answer within 30 s"; not cached, so the next load retries.
