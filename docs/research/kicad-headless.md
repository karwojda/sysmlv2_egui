# What can KiCad report headless?

Research for `.scratch/engineering-tools/issues/05-kicad-headless.md` (map: Engineering Tools).
Gathered 2026-10-20. At that date KiCad 10 is the current stable series (10.0.0 released
2026-03-20 [R10], Docker tags reach 10.0.7 [DH]), and KiCad 11 is in development on `master`.
Facts marked **(master)** come from the development branch and are expected in KiCad 11. They
are not in any stable release yet.

Nothing was installed or run. Query speed is **not measured**; see "Open points".

## Short answer

- **`kicad-cli` is the headless workhorse.** It is a `wxAppConsole` program, not a GUI one
  [CLI-SRC]. It reads `.kicad_sch` and `.kicad_pcb` files and writes BOM (CSV), netlists
  (KiCad S-expression or XML, plus other formats), ERC and DRC reports (JSON), and in KiCad 10
  board statistics as JSON (`pcb export stats`), which include width, height, area and
  thickness [CLI10]. It reports by exporting files, so there is no query interface. Each call
  is a fresh process.
- **The IPC API needs a running KiCad GUI in 9 and 10**, covers only the PCB editor, and must
  be turned on in Preferences > Plugins [ADDON] [PYPI]. A headless server,
  `kicad-cli api-server`, and schematic support are KiCad 11 features [ADDON] [API-SRC].
- **The SWIG `pcbnew` Python module works without a GUI.** It loads a board from file
  [SWIG]. It has been deprecated since 9.0, is planned for removal in 11.0, and is not a
  stable API [SWIG].
- **Stable addressing uses UUIDs.** Each schematic symbol has a UUID, and each instance is
  identified by a sheet-path of UUIDs with its own reference designator [SCH-FMT]. A board
  footprint stores that same path in `(path …)` [PCB-SRC10]. Reference designators are
  readable but can be re-annotated. **Nets are addressed by name.** KiCad 10 stopped writing
  net codes to files, and kicad-python says not to rely on them [PCB-H10] [KIPY-TYPES].
- **Reading the files directly is feasible** for symbols, fields, footprints, board outline
  and the stored net names. The schematic does not store nets: connectivity has to be computed
  from wires and labels, so a netlist needs `kicad-cli sch export netlist`. Board dimensions
  from raw Edge.Cuts geometry mean redoing KiCad's outline logic.

## 1. kicad-cli (headless, file in, file out)

`kicad-cli` derives from `wxAppConsole` [CLI-SRC], and the official Docker images exist only
to run it. "Using the GUI environment via Docker is not supported" [DOCKER].

Commands relevant to pulling values (KiCad 9 [CLI9] and KiCad 10 [CLI10]):

| Need | Command | Output | Notes |
|---|---|---|---|
| BOM | `sch export bom` | CSV (delimiters configurable) | `--fields` (default `Reference,Value,Footprint,${QUANTITY},${DNP}`), `--labels`, `--group-by`, `--sort-field`, `--filter`, `--exclude-dnp`, `--preset`, `--format-preset` [CLI9]. Any symbol field can be named in `--fields`. No JSON BOM. `--variant` appears in 10 [CLI10]. |
| Netlist (components + fields + nets) | `sch export netlist -f kicadsexpr\|kicadxml\|…` | S-expr or XML | Formats: `kicadsexpr` (default), `kicadxml`, `cadstar`, `orcadpcb2`, `spice`, `spicemodel`, `pads`, `allegro` [CLI9]. See §4 for contents. |
| Legacy XML for BOM scripts | `sch export python-bom` | XML | Legacy [CLI9]. |
| Electrical rule check | `sch erc --format json` | JSON | `--exit-code-violations` [CLI9]. |
| Design rule check | `pcb drc --format json [--schematic-parity]` | JSON | [CLI9]. |
| Board size, area, thickness, counts | `pcb export stats --format json --units mm\|in` | JSON | **KiCad 10 only**, not in 9 [CLI10] [FORUM-STATS]. |
| Placement (ref, value, footprint, X/Y/rot, side) | `pcb export pos` | CSV/ASCII | [CLI9]. |
| Full fabrication data model | `pcb export ipc2581`, `pcb export odb` | XML / ODB++ | [CLI9]. |
| Many commands in one run | `jobset run` | per job | Jobsets are new in 9 and also run from the GUI [CLI9] [R9]. |
| Format migration | `sch upgrade`, `pcb upgrade` | file | KiCad 10 [CLI10]. |

Keys in the `pcb export stats` JSON, from `FormatBoardStatisticsJson` [STATS-SRC]:
`board.{has_outline, width, height, area, board_thickness, front/back_copper_area,
min_track_width, min_track_clearance, min_drill_diameter, …}`, plus
`pads.{through_hole, smd, …}`, `vias.{through, blind, buried, micro}`,
`components.{tht, smd, unspecified, total}.{front, back, total}` and a `drill_holes[]` array.
`width`, `height` and `area` are null when there is no outline [STATS-SRC]. This source listing
is from master. Expect the KiCad 10 key names to be the same or very close, and confirm them
with a probe.

**Gap:** no `kicad-cli` command dumps footprint *properties* from the board as data. Symbol
fields come through the BOM and netlist. Board-side footprint fields need `pos` (partial),
IPC-2581, the IPC API, or reading the file directly. **(master)** registers
`pcb export bom`, `pcb export stackup`, and `sch/pcb/sym/fp diff` [CLI-SRC].

## 2. IPC API (KiCad 9+)

- Transport: Protocol Buffers over NNG, through a Unix domain socket (`api.sock` in the
  user's temp directory by default) [IPC] [ADDON]. External clients need not be launched
  from KiCad. Plugins started by KiCad get `KICAD_API_SOCKET` and `KICAD_API_TOKEN` [ADDON].
- **Needs a running GUI in 9 and 10.** "The IPC API in KiCad 9 and 10 only supports
  communication with a running instance of the KiCad GUI." "Support for running in headless
  mode through `kicad-cli` was added for KiCad 11." [ADDON]
- **PCB editor only in 9 and 10.** "KiCad 9 and 10 only support IPC API plugins in the PCB
  Editor. KiCad 11 adds support for plugins in the Schematic Editor." [ADDON]
- **No export in 9 and 10.** "The IPC API in KiCad 9 and 10 has no support for plotting or
  exporting files"; it was added for 11 [ADDON].
- It must be enabled under Preferences > Plugins: "It is not possible to use
  `kicad-python` to manipulate KiCad design files without KiCad running." [PYPI]
- It is designed as a stable interface that does not change when KiCad's internals are
  refactored [ADDON].
- Client: `kicad-python` (import `kipy`), version 0.8.0 (2026-08-30), Python 3.9+, MIT,
  requires KiCad 9.0+ [PYPI]. The `Board` class has `get_footprints`, `get_nets`,
  `get_pads`, `get_items_by_id`, `get_items_by_net`, `get_stackup`,
  `get_copper_layer_count`, `get_shapes`, `get_item_bounding_box`, `get_title_block_info`,
  `expand_text_variables` and `export_stats` (main branch, so some of these need 11) [KIPY-BOARD].
  `FootprintInstance` exposes `id`, `reference_field`, `value_field`, `texts_and_fields`,
  `attributes` (`do_not_populate`, `exclude_from_bill_of_materials`, …), `sheet_path` and
  `symbol_sheet_name` [KIPY-TYPES].
- **(master)** `kicad-cli api-server` runs the IPC server without a GUI. It can preload a
  board, schematic or project and takes a `--socket` override [API-SRC]. A forum reply calls
  it a v11 feature, not 10.x [FORUM-API].

## 3. SWIG `pcbnew` Python module

- It can be "used stand-alone, i.e. no instance of the PCB Editor is running"; the board is
  "loaded and saved from and to file" [SWIG].
- "Deprecated as of KiCad 9.0 … The current plan is to remove the SWIG bindings in KiCad
  11.0." It is "a raw binding layer … it is unstable" and "will generally change between
  KiCad major versions" [SWIG]. kicad-python's page also says SWIG is removed in 11 [PYPI].
- It covers the board only. There is no schematic equivalent.
- Verdict: usable for KiCad 9 and 10 only. It is a dead end for a design meant to last.

## 4. Stable addressing

**Schematic.** Each placed symbol has a `uuid`. Hierarchical paths are sheet UUIDs joined by
`/`, starting at the root. Under `(instances (project …) (path "/…" (reference "R1") (unit 1)))`
each sheet instance stores its *own* reference designator [SCH-FMT] [SCH-SRC10]. A symbol on a
sheet used twice therefore has one UUID, two paths and two references. Fields are
`(property "Name" "Value" (at …) …)` [SCH-SRC10] [SEXPR].

**Board.** A footprint has its own `uuid`, plus `(path "…")`, the hierarchical path of the
linked schematic symbol, and `(sheetname …)` and `(sheetfile …)` [SEXPR] [PCB-SRC10]. So
**sheet-path + symbol UUID** is the key that joins schematic and board, and it survives
re-annotation. Reference designators are what engineers read and write, but annotation can
change them.

**Netlist export.** Each `comp` carries `ref`, `value`, `footprint`, `datasheet`,
`description`, `fields/field[name]`, `libsource`, `property` (including `dnp`,
`exclude_from_bom` and `exclude_from_board` when set), `sheetpath (names, tstamps)` and
`tstamps` (the symbol UUIDs). Each `net` carries `code` (sequential: index + 1, *not* stable),
`name` and `class`, and its nodes carry `ref`, `pin` (pad number), `pinfunction` and
`pintype` [NET-SRC].

**Nets.** These are addressed by **name**. Board format 20251028 (KiCad 10) says: "Stop
writing netcodes; they're an internal implementation detail" [PCB-H10]. The KiCad 10 writer
emits `(net "NAME")` on pads, tracks and zones, where 9.0 wrote `(net N "NAME")` and a net
table [PCB-SRC10] [PCB-SRC9]. kicad-python marks `Net.code` deprecated: "API clients should
not rely on net codes" [KIPY-TYPES]. Net names are hierarchical (for example `/sheet/LABEL`).
Unlabelled nets get auto-generated names that can change when the design is edited, so
a Tool Link to a net should point at a labelled net.

## 5. Reading `.kicad_sch` / `.kicad_pcb` without KiCad

Both are documented S-expression formats [SEXPR] [SCH-FMT] [PCB-FMT], each with a
`(version YYYYMMDD)` header. The board version moved through more than ten values in the 10.0
cycle alone [PCB-H10].

| Data | Directly readable? |
|---|---|
| Symbol fields/properties, DNP/in-BOM flags, UUIDs, per-instance references | Yes. These are plain tokens [SCH-FMT]. Hierarchy means walking `(sheet …)` files and resolving `instances`. |
| Footprint fields, attributes, position, layer, `path`, `uuid` | Yes [SEXPR] [PCB-SRC10]. |
| Board thickness, stackup | Yes: `general/thickness` and `setup/stackup` [PCB-FMT]. |
| Board width/height | Partly. You must compute the bounding box of `Edge.Cuts` graphics (lines, arcs, curves) yourself. KiCad's own number is measured from the outside of the edge-cut stroke [GL-18470]. |
| Pad-to-net on the board | Yes: pad `(net …)`. This is name-only from KiCad 10, code + name before [PCB-SRC9] [PCB-SRC10]. |
| Schematic nets / netlist | **No.** The schematic has wires, junctions and labels but no net objects [SCH-FMT]. You would have to reimplement connectivity (labels, power symbols, hierarchical pins, buses), so use `kicad-cli sch export netlist`. |
| Text variables `${…}`, library-resolved defaults | No. They need the project file and libraries. |

The format changes with every major version (for example, net codes were dropped in 10), so a
direct reader must branch on `version`. This is the same coupling risk as SWIG, but with no
install needed.

## 6. Linux install footprint

- Debian trixie `kicad` 9.0.2: 40.5 MB download, 164 MB installed. It depends on GTK3,
  wxGTK 3.2, OpenCASCADE 7.8, ngspice, NNG, protobuf and Python 3.13. Libraries are
  *Recommended* (`kicad-libraries`) and 3D models are *Suggested* (`kicad-packages3d`) [DEB].
  There is no separate CLI-only package, so `kicad-cli` brings the GUI libraries with it.
- Official Docker `kicad/kicad`: `10.0.7` is about 812 MB and `10.0.7-full` (with libraries)
  is about 1.45 GB, compressed sizes from the Docker Hub API [DH]. AppImage builds come in
  Full and Lite (without 3D) [DL-LINUX]. Flatpak is also offered [DL-LINUX].
- Reading the files directly needs nothing installed.

## Open points (need a probe, not more reading)

- **Query speed:** no primary source gives `kicad-cli` cold-start or per-export times. A
  probe should time `kicad-cli version`, `sch export netlist`, `sch export bom` and
  `pcb export stats` on a small and a large project, in Docker and natively. Library-table
  loading is the suspected startup cost.
- The exact `pcb export stats` JSON keys in 10.0.x versus master.
- Whether `kicad-cli` really runs with no `DISPLAY` set on a bare Linux host. The
  `wxAppConsole` base and the official Docker images suggest it does, but neither states it.

## Sources

- [CLI9] KiCad 9.0 CLI reference — https://docs.kicad.org/9.0/en/cli/cli.html
- [CLI10] KiCad 10.0 CLI reference — https://docs.kicad.org/10.0/en/cli/cli.html
- [CLI-SRC] `kicad/kicad_cli.cpp` (master doxygen) — https://docs.kicad.org/doxygen/kicad__cli_8cpp_source.html
- [API-SRC] `command_api_server.cpp` (master doxygen) — https://docs.kicad.org/doxygen/command__api__server_8cpp_source.html
- [STATS-SRC] `board_statistics_report.cpp` (master doxygen) — https://docs.kicad.org/doxygen/board__statistics__report_8cpp_source.html
- [NET-SRC] `netlist_exporter_xml.cpp` (master doxygen) — https://docs.kicad.org/doxygen/netlist__exporter__xml_8cpp_source.html
- [IPC] IPC API overview — https://dev-docs.kicad.org/en/apis-and-binding/ipc-api/
- [ADDON] IPC API for add-on developers — https://dev-docs.kicad.org/en/apis-and-binding/ipc-api/for-addon-developers/
- [SWIG] PCB Python bindings — https://dev-docs.kicad.org/en/apis-and-binding/pcbnew/
- [PYPI] kicad-python on PyPI — https://pypi.org/project/kicad-python/
- [KIPY-BOARD] kicad-python `kipy/board.py` — https://gitlab.com/kicad/code/kicad-python/-/raw/main/kipy/board.py
- [KIPY-TYPES] kicad-python `kipy/board_types.py` — https://gitlab.com/kicad/code/kicad-python/-/raw/main/kipy/board_types.py
- [SEXPR] S-expression format intro — https://dev-docs.kicad.org/en/file-formats/sexpr-intro/
- [SCH-FMT] Schematic file format — https://dev-docs.kicad.org/en/file-formats/sexpr-schematic/
- [PCB-FMT] Board file format — https://dev-docs.kicad.org/en/file-formats/sexpr-pcb/
- [PCB-SRC9] `pcb_io_kicad_sexpr.cpp` @ 9.0 — https://gitlab.com/kicad/code/kicad/-/raw/9.0/pcbnew/pcb_io/kicad_sexpr/pcb_io_kicad_sexpr.cpp (net format lines 678, 1613, 2436)
- [PCB-SRC10] `pcb_io_kicad_sexpr.cpp` @ 10.0 — https://gitlab.com/kicad/code/kicad/-/raw/10.0/pcbnew/pcb_io/kicad_sexpr/pcb_io_kicad_sexpr.cpp (path 1278, pad net 1858, track net 2861)
- [PCB-H10] `pcb_io_kicad_sexpr.h` @ 10.0, `SEXPR_BOARD_FILE_VERSION` history — https://gitlab.com/kicad/code/kicad/-/raw/10.0/pcbnew/pcb_io/kicad_sexpr/pcb_io_kicad_sexpr.h
- [SCH-SRC10] `sch_io_kicad_sexpr.cpp` @ 10.0 — https://gitlab.com/kicad/code/kicad/-/raw/10.0/eeschema/sch_io/kicad_sexpr/sch_io_kicad_sexpr.cpp (instances 853–955, property 1019)
- [R9] KiCad 9.0.0 release — https://www.kicad.org/blog/2025/02/Version-9.0.0-Released/
- [R10] KiCad 10.0.0 release — https://www.kicad.org/blog/2026/03/Version-10.0.0-Released/
- [DOCKER] KiCad Docker images — https://www.kicad.org/download/docker/
- [DH] Docker Hub tags API — https://hub.docker.com/v2/repositories/kicad/kicad/tags?page_size=15
- [DEB] Debian trixie `kicad` — https://packages.debian.org/trixie/kicad
- [DL-LINUX] KiCad Linux downloads — https://www.kicad.org/download/linux/
- [GL-18470] Board size includes Edge.Cuts stroke — https://gitlab.com/kicad/code/kicad/-/issues/18470
- [FORUM-STATS] (maintainer reply: stats are v10+) — https://forum.kicad.info/t/kicad-cli-board-statistics/66669
- [FORUM-API] (api-server is a v11 feature) — https://forum.kicad.info/t/is-the-kicad-cli-api-server-option-going-to-be-in-kicad-v11-or-v10-x-x/69411
