# PROTOTYPE, throwaway: does a live FreeCAD query work from this tool?

Answers the wayfinder ticket "Does a live FreeCAD query work from this tool?" (`.scratch/engineering-tools/issues/10-live-freecad-prototype.md` on master). Never merged.

Run from this folder (needs FreeCAD 1.1 on Windows, called from WSL):

    cargo run                                  # Model Values, Tool Links, FreeCAD, Design Values, checks again
    cargo run -- --freecad /nope/freecadcmd.exe  # FreeCAD missing

- `model/` is a small Model with two Tool Links (`drive.sysml`) and the copied `EngineeringTools` library.
- `fc/make_cad.py` built `model/cad/engine.FCStd`; `fc/query.py` is the FreeCAD side of a query.
- `src/main.rs` is the whole pipeline; it prints each step and its time.
