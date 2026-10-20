# Research: what FreeCAD can report headless

Ticket: `.scratch/engineering-tools/issues/01-freecad-headless.md` (map: Engineering Tools).
Researched 2026-10-20 against FreeCAD source at tags `1.1.4` (latest stable) and `26.3rc2`, the GitHub releases API, the FreeCAD wiki (read through its GitHub mirror, `FreeCAD/FreeCAD-documentation`, because wiki.freecad.org returned a bot-wall), conda-forge and Flathub. FreeCAD was **not** installed or run, so no figure below was measured locally.

Source shorthand: `src@1.1.4:path` means `https://github.com/FreeCAD/FreeCAD/blob/1.1.4/path`.

## Answer in brief

- **Yes, headless works.** `FreeCADCmd` (installed as `freecadcmd` in current builds) or `FreeCAD -c` gives a Python shell with no GUI. An external Python can also `import FreeCAD`, but only if it is ABI-compatible. Opening a `.FCStd` file and reading shapes needs no GUI.
- **Volume and centre of mass come from geometry alone**, through `Shape.Volume`, `Solid.CenterOfMass`, `Solid.MatrixOfInertia` and the related properties.
- **Mass needs a density, and FreeCAD's own `Shape.Mass` ignores material.** It returns the volume, because OCCT's density is 1. Real mass means reading the object's `ShapeMaterial` (added in 1.0) and multiplying by its `Density`. The `.FCStd` file stores only the material's **UUID**, and the density is looked up in the material library of the machine that opens the file.
- **Query speed is not known.** No primary source publishes timings, so they must be measured (ticket 10, the live FreeCAD prototype).
- **Versions:** 1.0.0 (2024-11-19), 1.1.0 (2026-03-25), **1.1.4 stable (2026-09-28)**, and 26.3 in RC. FreeCAD now uses calendar versions with three releases a year.
- **Footprint:** the official x86_64 AppImage is about 780 MiB (820,812,280 bytes) and bundles Python 3.11. Flathub carries 1.1.4. conda-forge carries 1.1.3 and a 2026.09.16 build. Distro packages lag behind.
- **Addressing:** `Name` is unique and immutable, so it is the stable key. `Label` is user-editable and only unique by default. Objects inside assemblies are addressed by a dot-separated **subname path** resolved with `getSubObject`.

## 1. Running without a GUI

- The wiki's *Headless FreeCAD* page says FreeCAD can run "in the console without enabling the GUI" through `/path/to/FreeCAD -c` or `/path/to/FreeCADCmd`. View providers cannot be created in that mode. Source: [wiki/Headless_FreeCAD.md](https://github.com/FreeCAD/FreeCAD-documentation/blob/main/wiki/Headless_FreeCAD.md).
- The console entry point is `src@1.1.4:src/Main/MainCmd.cpp`.
- **AppImage:** the official AppImage's `AppRun` script runs any binary in its `usr/bin` that is named as the first argument (`if [ -e "$HERE/usr/bin/$1" ] ; then MAIN="$HERE/usr/bin/$1"`). So `FreeCAD_1.1.4-Linux-x86_64-py311.AppImage freecadcmd script.py` runs headless from the single file. The script also sets `PYTHONHOME` to the bundle. Source: `src@1.1.4:package/rattler-build/linux/AppDir/AppRun`.
- **Embedding in an outside Python:** append the directory that holds `FreeCAD.so` to `sys.path`, then `import FreeCAD`. The wiki warns that the module "can only be imported from a compatible Python interpreter" and recommends "the Python included with FreeCAD, run FreeCAD via command line, or as a subprocess". Source: [wiki/Embedding_FreeCAD.md](https://github.com/FreeCAD/FreeCAD-documentation/blob/main/wiki/Embedding_FreeCAD.md). For our use, a subprocess that runs `freecadcmd` on a script is the low-risk path.
- **Opening a file without recomputing:** `.FCStd` is a zip that holds `Document.xml`, which is all console mode uses, plus one BREP file per object's shape. Shapes are therefore stored "so it can be accessed by components without the need to recalculate the shape". `FreeCAD.openDocument(path)` followed by reading `obj.Shape` returns the saved geometry, and no `recompute()` is needed. Source: [wiki/File_Format_FCStd.md](https://github.com/FreeCAD/FreeCAD-documentation/blob/main/wiki/File_Format_FCStd.md).

## 2. Physical properties per body or part

From `src@1.1.4:src/Mod/Part/App/TopoShape.pyi` and `TopoShapeSolid.pyi`:

| Property | On | Meaning |
|---|---|---|
| `Volume` | any shape | "Total volume of the solids of the shape" (mm³) |
| `Area` | any shape | total face area (mm²) |
| `Mass` | `Part.Solid` | "mass of the current system" (see the warning below) |
| `CenterOfMass` | `Part.Solid` | "expressed in the absolute Cartesian coordinate system"; equals the centre of gravity in a uniform field |
| `MatrixOfInertia` | `Part.Solid` | inertia tensor about the centre of mass, axes parallel to global X/Y/Z |
| `StaticMoments`, `PrincipalProperties` | `Part.Solid` | static moments; principal moments and axes |
| `getMomentOfInertia(p, dir)`, `getRadiusOfGyration(p, dir)` | `Part.Solid` | about an arbitrary axis |

**Warning:** `TopoShapeSolidPy::getMass()` calls `BRepGProp::VolumeProperties(shape, props)` and returns `props.Mass()`, with no density argument, so the value equals the volume. Source: `src@1.1.4:src/Mod/Part/App/TopoShapeSolidPyImp.cpp`, lines 104–110. The same holds for the inertia values: they are volume-weighted, so they must be scaled by density.

A PartDesign `Body`'s `Shape` is its result solid. Units are FreeCAD-internal: mm, and kg for mass quantities.

**New in 26.3 (RC, not yet stable):** a Measure "Mass Properties" tool (`src@26.3rc2:src/Mod/Measure/App/MassPropertiesResult.cpp`). It computes volume, mass, surface area, centre of gravity, centre of volume and inertia across several objects. It applies each object's placement, follows Links to their `Part::Feature`, and weights by `ShapeMaterial` density. When the material is `Default` or has no `Density`, it **falls back to 1000 kg/m³** (`density = 1.0e-6` kg/mm³). The computation (`CalculateMassProperties`) is C++ in Measure/App, and no Python binding was found in `AppMeasurePy.cpp`. A headless script on 1.1.x has to do the same calculation itself, which is short: `GProp`-style sums of `Volume × density` and a mass-weighted centre.

## 3. Material and density: where they live

- Since **1.0**, every `Part::Feature`, and therefore every PartDesign Body, has a `ShapeMaterial` property of type `Materials::PropertyMaterial`. In 0.21.2 the property is absent. Sources: `src@1.1.4:src/Mod/Part/App/PartFeature.h` line 78, `src@1.0.0:src/Mod/Part/App/PartFeature.h` line 61, `src@0.21.2:src/Mod/Part/App/PartFeature.h`.
- New objects receive the default material (`PartFeature.cpp`, `Feature::Feature()`). Its UUID is the preference `Mod/Material/DefaultMaterial`, falling back to `7f9fd73b-…` (`MaterialManager.cpp`, `defaultMaterialUUID()`). That card, `Standard/Default.FCMat`, is described as a "Generic material with density of 1" and has `Density: "1 kg/m^3"`. Mass from an unassigned body is therefore wrong by a factor of about 1000–8000 unless that case is detected, as 26.3 does by checking for the name `Default`.
- **Serialisation:** `PropertyMaterial::Save` writes only `<PropertyMaterial uuid="…"/>`, and `Restore` looks that UUID up through `MaterialManager::getManager().getMaterial(...)`. Source: `src@1.1.4:src/Mod/Material/App/PropertyMaterial.cpp`, lines 83–97. **The density is not in the file.** It comes from the material library on the querying machine, so a user-defined material card has to be present in CI too, and a changed card silently changes the mass. What happens when a UUID is missing was not traced; it needs testing in the prototype.
- From Python, `obj.ShapeMaterial` exposes `Name`, `UUID` and `PhysicalProperties`. The property-dict form is marked "deprecated" in favour of quantity accessors. Source: `src@1.1.4:src/Mod/Material/App/Material.pyi`.
- Alternative: the Tool Link, or a property the user adds, could state the density or the mass explicitly. That is a design choice for the Tool Link ticket.

## 4. Query speed

No primary source gives startup or per-file timings. The cost is the interpreter and module start (OCCT and Qt core libraries, with no GUI), `openDocument`, which unzips the file and reads BREP but does not recompute, and the property calls, which run OCCT `BRepGProp` integration and are cheap for ordinary parts. **This has to be measured** in the live prototype, ticket 10. One process per run that queries every link, rather than one process per link, is the obvious mitigation.

## 5. Versions, footprint, Linux packaging

| Release | Date (GitHub API `published_at`) |
|---|---|
| 1.0.0 | 2024-11-19 |
| 1.1.0 | 2026-03-25 |
| 1.1.4 (latest stable) | 2026-09-28 |
| 26.3rc2 (pre-release) | 2026-10-18 |

The 26.3rc1 release notes say: "FreeCAD has moved to three releases per year on fixed dates with calendar-based version numbers, so the next releases will be 27.1, 27.2 and 27.3." Source: [GitHub release 26.3rc1](https://github.com/FreeCAD/FreeCAD/releases/tag/26.3rc1).

Packaging:

- **AppImage (official):** `FreeCAD_1.1.4-Linux-x86_64-py311.AppImage` is 820,812,280 bytes, and the aarch64 build is 755,591,688 bytes. Both are self-contained and need no installation. The 26.3rc2 x86_64 AppImage is 768,526,840 bytes. Source: [GitHub release 1.1.4](https://github.com/FreeCAD/FreeCAD/releases/tag/1.1.4). Official bundles are built with rattler-build from conda-forge packages and pin `python>=3.11,<3.12` and `occt>=7.8,<7.9` (`src@1.1.4:package/rattler-build/recipe.yaml`).
- **Flatpak:** `org.freecad.FreeCAD` on Flathub is at 1.1.4 ([Flathub API](https://flathub.org/api/v2/appstream/org.freecad.FreeCAD)). The wiki lists it as official.
- **conda-forge:** package `freecad`, with versions including 1.0.0, 1.1.0, 1.1.3 and 2026.09.16. The linux-64 package alone is about 122 MB compressed, before dependencies, with builds for py311–py314 ([anaconda.org API](https://api.anaconda.org/package/conda-forge/freecad)). This is the only route where FreeCAD comes as an importable module in a chosen Python version.
- **Snap:** listed as official. **Ubuntu PPA:** "currently unmaintained". **Distro packages** (Ubuntu Universe, Fedora, openSUSE) "may be outdated" or are "well behind the FreeCAD releases". Source: [wiki/Installing_on_Linux.md](https://github.com/FreeCAD/FreeCAD-documentation/blob/main/wiki/Installing_on_Linux.md).
- **Footprint:** plan for about 0.8 GB per installed version (the AppImage) and more once unpacked. The unpacked size was not measured.

## 6. Stable object addressing

- **`Name`** is the internal name. It "must be unique in the entire document", "is assigned at the creation time of the object; afterwards it is no longer editable", and scripts "should use the `Name` … rather than its `Label`". Lookup is `doc.getObject(name)`. Source: [wiki/Object_name.md](https://github.com/FreeCAD/FreeCAD-documentation/blob/main/wiki/Object_name.md) and `src@1.1.4:src/App/Document.pyi`.
- **`Label`** is the tree text the user sees. It is user-editable and unique by default, but the preference `BaseApp/Preferences/Document/DuplicateLabels` allows duplicates (`src@1.1.4:src/App/DocumentObject.cpp`, around line 826). `doc.getObjectsByLabel(label)` returns a list for that reason (`Document.pyi`).
- **`Label2`** is a free-text description.
- **`ID`** is an integer "unique identifier (among its document)" (`DocumentObject.pyi`). It is less readable than `Name` and no more stable.
- **Assemblies and Links:** an object inside a container or assembly is addressed from a top-level object with a dot-separated **subname**, for example `Assembly.LinkToPart.Body.`. `obj.getSubObject(subname, retType=…)` resolves it. The returned shape is "pre-transformed by accumulated transformation matrix along the object path", and `retType=3` returns the transformed placement. `obj.resolve(subname)` and `obj.getSubObjectList(subname)` give the chain. `getLinkedObject()` follows `App::Link` to the source object, which carries `ShapeMaterial`. Source: `src@1.1.4:src/App/DocumentObject.pyi`, lines 156–296.
- **Implication for a Tool Link:** use the file path plus `Name`, plus a subname path when the target sits inside an assembly. Label can be offered as a display hint or a fallback, and never as the key. A Name stays stable across saves, but copy and paste or merging documents can give an object a new suffixed Name such as `Body001`. That is inferred from the uniqueness rule, not documented explicitly.

## Open points

- Startup and per-file timing (prototype).
- Behaviour when a `ShapeMaterial` UUID is not in the local library (prototype).
- Exact headless behaviour inside the AppImage in CI, where it needs FUSE or `--appimage-extract` (prototype).
