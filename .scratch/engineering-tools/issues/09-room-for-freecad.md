# Make room for FreeCAD

**Status:** resolved
**Type:** task
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

The Windows host drive under WSL has about 5 GB free, which isn't enough to safely install FreeCAD (and later KiCad). Free enough space: for example `cargo clean` (target/ is 25 GB) followed by compacting the WSL virtual disk, or install FreeCAD on the Windows side and call it from WSL. HITL: the user is preparing this. Record what was done and where FreeCAD ended up.

## Answer

Done by the user. `cargo clean` brought `target/` to 2.2 GB, and C: now has 50 GB free. FreeCAD **1.1.4** is installed on the Windows side at `C:\Program Files\FreeCAD 1.1\`. Its headless binary runs from WSL: `"/mnt/c/Program Files/FreeCAD 1.1/bin/freecadcmd.exe" -c "<python>"` (checked: prints version 1.1.4). It is a Windows program, so file paths passed to it need Windows form (`wslpath -w`).
