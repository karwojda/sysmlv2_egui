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

**Status:** done

- [x] Committed `.FCStd` fixture: a 100 mm cube body, and a cylinder reached through an `App::Link` in an `App::Part`
- [x] Through the trait: cube `Volume` is `1000000.0 [mm^3]`; the cylinder resolves by its dotted assembly path
- [x] Unknown property gives a reason
- [x] Missing object names the object and the file; missing first object of a path likewise
- [x] Missing file names the file
- [x] Missing executable gives "FreeCAD not available"
- [x] Timeout gives "FreeCAD did not answer within 30 s" (tested with a stub executable that sleeps, and a short timeout via the environment variable)
- [x] Discovery order: `SYSML_VIEWER_FREECAD`, `PATH`, known install locations including WSL
- [x] Tests needing FreeCAD are skipped with a printed note when it can't be found

## Comments

- Done.
  - **Crate `viewer-tools`** (no SysML, no OpenSysML). `viewer_tools::FreeCad` implements `EngineeringTool`. Register it with `OpenSysmlBackend::new().with_tool("FreeCAD", FreeCad::new())`. `FreeCad::new()` reads this machine (`Machine::current()`); `FreeCad::from_machine(Machine { executable, timeout, path, install_dirs })` lets tests give their own. Finding no FreeCAD isn't an error at construction: every `read` then returns the reason, and `executable()` / `timeout()` expose what was found.
  - **Environment variables:** `SYSML_VIEWER_FREECAD` (the executable; when set but naming nothing it is final, PATH isn't searched, so a mistyped setting isn't hidden), `SYSML_VIEWER_FREECAD_TIMEOUT` (seconds, fractions allowed, default 30). The request reaches the script through `SYSML_VIEWER_FREECAD_REQUEST`, added to `WSLENV` for a Windows `.exe`. Constants: `freecad::EXECUTABLE_VARIABLE`, `TIMEOUT_VARIABLE`, `DEFAULT_TIMEOUT`.
  - **Discovery:** variable, then `PATH` (`freecadcmd`, `FreeCADCmd`, either with `.exe`), then the newest `FreeCAD*` folder holding `bin/` or `Contents/Resources/bin/` under `/mnt/c/Program Files`, `/opt`, `/usr/lib`, `/usr/local/lib`, `/Applications` and `%ProgramFiles%`. An `.exe` run from Linux is taken as WSL: file and script paths go through `wslpath -w`.
  - **Script** (`src/freecad/query.py`, `include_str!`, written to a temp dir per run): the prototype's contract. Only `Volume` (`mm^3`); others give `FreeCAD has no property X (it reads Volume)`.
  - **Reasons:** `FreeCAD not available: …` (names the variable or what was searched), `file <path> not found`, `no object named 'Engine' in <path>`, `no object named 'Gearbox' in <path>` (first object of a path), `no object at 'Assembly.Shaft.' in <path>`, `<path>: FreeCAD could not open it: …` (whole file), `FreeCAD did not answer within 30 s`, `FreeCAD gave no answer: <last stderr line>`, `SYSML_VIEWER_FREECAD_TIMEOUT is soon, not a number of seconds`.
  - **Fixture:** `tests/fixtures/parts.FCStd`, built by `tests/fixtures/make_parts.py` (header says how, also from WSL) with FreeCAD 1.1.4: `Body` (PartDesign body with an additive 100 mm cube) and `Assembly.Link.` (an `App::Link` in an `App::Part` to a cylinder r 10, h 100).
  - **Tests:** `tests/freecad.rs` (7, real FreeCAD, not `#[ignore]`d: they run in the plain `cargo test` step whenever FreeCAD is found and print `note: skipping …` otherwise) and `tests/finding_freecad.rs` (11, stub shell scripts, unix only).
  - **Deviation, rounding:** OCCT gives the PartDesign cube as 999999.9999999999 mm³, so the script rounds to 12 significant digits (`1000000.0`, the cylinder `31415.9265359`). Values print with Rust's `{:?}`, so always with a decimal point.
  - **Deviation, default timeout:** tested through `FreeCad::timeout()` returning 30 s; the message is tested with a 1 s timeout from the variable.
  - **For 07/08:** build the tool with `FreeCad::new()` per backend; it holds no process between reads. Killing on timeout kills the WSL interop process; whether the Windows `freecadcmd.exe` dies with it was not tested.
