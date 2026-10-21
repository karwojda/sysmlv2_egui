# What can FreeCAD report headless?

**Status:** resolved
**Type:** research
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

Can `freecadcmd` (or FreeCAD's Python module) open an `.FCStd` file without a GUI and return, per body or part: volume, mass, centre of gravity and other physical properties? Does mass need a material or density set in the file, and how is that stored? How long does a query take (startup plus per-file)? Which FreeCAD versions, install footprint (disk size), and Linux packaging (AppImage, Flatpak, distro, conda)? How are objects addressed stably inside a file (Name vs Label vs path through assemblies)?

## Answer

**Yes, FreeCAD can report these headless, but its mass number can't be trusted as-is.** These findings come from FreeCAD source and documentation only; nothing was installed or measured.

- **Running headless:** `FreeCADCmd` or `FreeCAD -c` runs a script with no GUI. The AppImage can do the same with `FreeCAD.AppImage freecadcmd script.py`. Embedding FreeCAD in another Python is fragile because the interpreter must match; the wiki recommends a subprocess.
- **The file:** `.FCStd` is a zip holding `Document.xml` and one BREP file per shape. `openDocument` loads the saved shapes without recomputing them.
- **What it reports:** volume, plus centre of mass (global coordinates), matrix of inertia and principal properties per solid.
- **Mass is the main trap.** `Shape.Mass` equals the volume, because the density is fixed at 1. Real mass is volume times the density of the object's material.
  - Since 1.0 each Part feature has a material, but the file stores only its UUID. The density is looked up in the material library of whichever machine opens the file, so CI needs the same materials installed.
  - A body with no material assigned gets "Default" at 1 kg/m³, which makes mass off by roughly 1000x.
  - FreeCAD 26.3 (still RC) adds a Mass Properties tool that reads densities and follows links, but it is C++ only, with no Python binding.
- **Versions:** 1.1.4 is the current stable release; 26.3 is in RC. FreeCAD now ships three calendar-versioned releases a year.
- **Disk footprint:** the AppImage is about 780 MiB, Flathub carries 1.1.4, and the conda-forge package is about 122 MB. Distro packages lag behind.
- **Addressing objects:** use the object's `Name`, which is unique and immutable. `Label` is user-editable and not reliably unique. For an object inside an assembly, use a dotted path such as `Assembly.Link.Body.` resolved with `getSubObject`, and `getLinkedObject` to reach the original object that holds the material. So a Tool Link addresses the file, plus the object's `Name`, plus that path when the object is inside an assembly.
- **Still open, for the prototype:** startup and per-file timing, what happens when a material UUID is missing locally, and running the AppImage in CI (it needs FUSE or `--appimage-extract`).

Findings: branch `research/freecad-headless` (commit 69ebf2c, not pushed: no GitHub credentials), `docs/research/freecad-headless.md`.
