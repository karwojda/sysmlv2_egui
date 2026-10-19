# SysML v2 Viewer

[![CI](https://github.com/karwojda/sysmlv2_egui/actions/workflows/ci.yml/badge.svg)](https://github.com/karwojda/sysmlv2_egui/actions/workflows/ci.yml)

A desktop viewer for SysML v2 Models, written in Rust with [egui](https://github.com/emilk/egui). Open a folder of `.sysml` files, browse the Model, and see it as Diagrams in the standard SysML v2 graphical notation. Adjust the automatic layout where it's off, and export Diagrams to SVG for documentation, either from the viewer or headless from the command line.

The Model is only ever read. Layout, saved Diagrams and Styles live in their own files beside it.

## Features

- **Browser and Properties:** the Model's ownership tree with a name filter, and each element's kind, type, multiplicity, metadata, documentation and source location ("Open in editor").
- **General Diagrams:** definitions and usages with compartments, plus specialization, defined-by, composition and connection edges.
- **Interconnection Diagrams:** nested parts and ports, connections, item flows, bindings, reference parts and composite ports. Parts can be opened in place.
- **Model-defined Diagrams:** `view` usages that render `asTreeDiagram` or `asInterconnectionDiagram` (or are typed by `GeneralView` / `InterconnectionView`) are drawn from what they `expose`.
- **Editable layout:** move, resize, hide and collapse nodes, bend edges, and move ports. Save the result as a Diagram File.
- **Requirements:** requirement nodes with their constraints, `«satisfy»` edges, and a status badge on each requirement (✓ satisfied, ✗ violated, ? undecided, ○ not claimed), checked by OpenSysML against the Model's values.
- **Styles:** colour node fills by element kind, package or metadata, with an optional Legend. Set them once for the Model and override them per Diagram.
- **Live reload:** edits to the Model or its Style file show up in the open Diagrams.
- **Export:** deterministic SVG, identical between the viewer and the CLI.

For a quick tour, open [`examples/mass-rollup`](examples/mass-rollup): a vehicle whose mass rolls up from its parts, with requirements that turn green or red as you edit the masses.

## Requirements

- Rust (stable, edition 2024).
- Parsing uses OpenSysML's [sysml-grpc](https://github.com/Open-MBEE/OpenSysML) service through the `opensysml` crate. On first use it downloads a release into `~/.opensysml/bin`. To use another binary, or a service already running, set `OPENSYSML_GRPC_BINARY` / `OPENSYSML_SERVICE` or pass `--sysml-binary` / `--sysml-service`.
- Linux is the supported platform for now.

## Usage

### Viewer

```sh
cargo run --release -p viewer-app -- <model-folder> [element] [--interconnection]
```

The optional element opens straight away: a General Diagram for a package or definition, an Interconnection Diagram for a part, or a view's own Diagram. Double-click an element in the Browser to open its Diagram, or pick one from the Views list.

On WSL, start it over X11 if WSLg's Wayland compositor crashes: `WAYLAND_DISPLAY= cargo run …`. WSLg's X11 bridge may show a maximised window shifted or cropped. `--renderer glow` (OpenGL) avoids that, but WSLg then hides the mouse cursor over the window, so the default renderer stays Vulkan.

### Command line

```sh
# one saved Diagram, or one view
cargo run --release -p viewer-cli -- export <model-folder> --diagram overview --out docs/overview.svg
cargo run --release -p viewer-cli -- export <model-folder> --view "vehicle tree" --out docs/tree.svg

# every saved Diagram and every supported view
cargo run --release -p viewer-cli -- export <model-folder> --all --out-dir docs/img

# requirement status, checked by OpenSysML; exits 1 if any is violated
# (--strict also fails on undecided and unclaimed ones), for CI
cargo run --release -p viewer-cli -- requirements <model-folder> [--strict]
```

The binaries are `sysml-viewer-gui` and `sysml-viewer`.

### Files beside the Model

```
my-model/
├── *.sysml
└── diagrams/
    ├── overview.diagram.json   # a saved Diagram: what it shows, its Layout and its Style
    └── style.json              # the Model Style
```

Both are pretty-printed JSON, so they diff and review well. Commit them with the Model.

## Development

```sh
cargo test --workspace                                  # fake-backend tests and golden SVGs
cargo test -p viewer-opensysml -- --ignored             # contract tests against sysml-grpc
UPDATE_GOLDEN=1 cargo test -p viewer-core               # regenerate golden SVGs after an intended change
```

The crates:

| Crate | Contents |
|---|---|
| `viewer-core` | The Model, Diagram builders, layout, Styles, the SVG renderer. No GUI and no parser. |
| `viewer-opensysml` | The backend that loads a Model through sysml-grpc. |
| `viewer-cli` | The headless `sysml-viewer` Export tool. |
| `viewer-app` | The egui desktop viewer. |

- **Vocabulary:** the domain terms used throughout (Diagram, Layout, Style, …) are defined in [CONTEXT.md](CONTEXT.md).
- **Design decisions:** recorded in [docs/adr](docs/adr).
- **Notation:** follows the *Intro to the SysML v2 Language – Graphical Notation* slides from the SysML v2 release.
