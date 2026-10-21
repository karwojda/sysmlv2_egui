# 06: FreeCAD adapter

**What to build:** A FreeCAD Engineering Tool, in a new `viewer-tools` crate that knows nothing of SysML, answering `Volume` for objects in an `.FCStd` file. It finds `freecadcmd` with no configuration: `SYSML_VIEWER_FREECAD` first, then `PATH` (`freecadcmd`, `FreeCADCmd`), then the usual install locations, including a Windows install seen from WSL (`/mnt/c/Program Files/FreeCAD */bin/freecadcmd.exe`, given Windows paths via `wslpath -w` and its request through an environment variable passed with `WSLENV`). It runs an embedded Python script once per file, resolving each object by `Name` or by a dotted assembly path through `getSubObject` (checking the first object exists). Each failure gives a reason that names what to fix. A run is killed after 30 s by default, changeable by an environment variable.

Script contract, settled in the prototype (branch `prototype/live-freecad`, `fc/query.py`):

```
request: {"file": <path>, "objects": {"<object>": ["Volume", …]}}
answer:  FCRESULT {"<object>": {"Volume": [<number>, "mm^3"]} | {"error": "<reason>"}}
         with "__file__": {"error": …} when the document can't be opened
```

See `.scratch/freecad-tracer/spec.md` (stories 5, 6, 16–20, 25, 26).

**Blocked by:** 03 (needs the `EngineeringTool` trait)

**Status:** ready-for-agent

- [ ] Committed `.FCStd` fixture: a 100 mm cube body, and a cylinder reached through an `App::Link` in an `App::Part`
- [ ] Through the trait: cube `Volume` is `1000000.0 [mm^3]`; the cylinder resolves by its dotted assembly path
- [ ] Unknown property gives a reason
- [ ] Missing object names the object and the file; missing first object of a path likewise
- [ ] Missing file names the file
- [ ] Missing executable gives "FreeCAD not available"
- [ ] Timeout gives "FreeCAD did not answer within 30 s" (tested with a stub executable that sleeps, and a short timeout via the environment variable)
- [ ] Discovery order: `SYSML_VIEWER_FREECAD`, `PATH`, known install locations including WSL
- [ ] Tests needing FreeCAD are skipped with a printed note when it can't be found
