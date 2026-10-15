# 01: Diagram pipeline tracer: fake Model → General Diagram → SVG

**What to build:** The thinnest end-to-end path through the core, without a sidecar or a GUI. A small fixture Model (a package with a few `part def`s related by specialization) is served by a fake backend and turned into a General Diagram rooted at that package. The Diagram is auto-laid out, built into a renderer-neutral scene, and Exported as SVG. This creates the Model backend interface (ADR-0001), the fake backend, the Diagram builder, auto-layout using an existing pure-Rust layered-layout library, the scene, and the deterministic SVG renderer. It also sets up the Seam 1 testing pattern. See the spec at `.scratch/viewer-first-slice/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] A Rust project exists, set up so the core (Model backend interface, Diagram builder, auto-layout, scene, SVG renderer) is separate from any GUI or command-line code
- [x] The Model backend interface exposes, for each element: kind, name, qualified name, owner, specializations and source location, plus Model diagnostics. Nothing outside the backend depends on a parsing library's types
- [x] A fake backend builds a Model from in-memory fixture data
- [x] The Diagram builder produces a General Diagram for a root package containing its `part def`s and the specialization edges between them
- [x] Auto-layout arranges the nodes with no overlaps
- [x] The scene describes shapes, text and edges (with end markers) and has no egui or SVG concepts
- [x] Nodes follow the Standard Notation (`«part def»` keyword plus name), and specialization edges end in a hollow triangle
- [x] SVG Export is deterministic: the same inputs give byte-identical output (stable ordering, fixed number formatting, no timestamps)
- [x] A golden-file test Exports the fixture's General Diagram and compares it to a committed SVG
- [x] Tests assert only on the scene and SVG output, never on internal data structures

## Comments

- Done. Core crate `viewer-core`; 12 tests at Seam 1, including a golden SVG (regenerate with `UPDATE_GOLDEN=1 cargo test`).
- Layered layout uses `rust-sugiyama` 0.4. It uses `HashMap` internally, but output proved identical across 50 in-process runs and 6 separate processes. If a golden test ever flakes, suspect this first.
- The scene is made of primitives (rect, polyline, polygon, text) tagged with what they depict, so renderers add no notation of their own.
- Text width is estimated from character count, so Exports don't depend on installed fonts.
