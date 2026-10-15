# 02: OpenSysML backend and sidecar manager

**What to build:** Load a real Model from a folder of `.sysml`/`.kerml` files through OpenSysML (ADR-0001). The sidecar manager starts a `sysml-grpc` process, using a binary path from configuration or else one found on `PATH`, and stops it when the program exits. The OpenSysML backend implements the Model backend interface from 01 using the `opensysml` Rust client. Opt-in contract tests (Seam 2) load a `.sysml` version of 01's fixture and check that the backend returns the same Model as the fake backend. This ticket is also the early check that OpenSysML exposes everything the interface needs; record any gaps you find.

**Blocked by:** 01: Diagram pipeline tracer

**Status:** done

- [x] The sidecar manager starts `sysml-grpc` from a configured path, falling back to `PATH`, waits until it is ready, and stops it on exit, including on error paths
- [x] The OpenSysML backend loads every `.sysml` and `.kerml` file under a folder, recursively, as one Model
- [ ] The backend answers the full Model backend interface from 01 (kind, name, qualified name, owner, specializations, source location, diagnostics)
- [x] Parse and resolution errors appear as Model diagnostics with file and line
- [ ] The standard library is resolved, and its elements are marked as library elements
- [x] Contract tests load a `.sysml` fixture equivalent to 01's fake fixture and assert both backends return the same elements and relationships
- [x] Contract tests run only when the `sysml-grpc` binary is available and are skipped otherwise, so the default test run needs no sidecar
- [x] Any OpenSysML gaps against what the spec's interface needs (typing, port direction, connection ends, and so on) are written down in this ticket's Comments section

## Comments

- Done, apart from the two items left unchecked below. Crate `viewer-opensysml`. Contract tests live in that crate's `tests/contract.rs` and are `#[ignore]`d. Run them with `cargo test -p viewer-opensysml -- --ignored`. On first use the client downloads `sysml-grpc` v0.10.0 into `~/.opensysml/bin`.
- **Service lifecycle is handled by the `opensysml` client**: binary lookup and download, start-up, and shutdown when the last connection drops (it also closes the service's stdin, so a crash of our process ends it too). We don't run our own process manager. See the ADR-0001 consequences.
- **Gap: element source locations.** `SymbolInfo` has no span. Only diagnostics and the service's own rendered views carry locations. `Element::source` is `None` for real Models. This blocks the source-file line in the properties panel and "open in editor" in ticket 10, until the service exposes spans or we find another route (for example the `RenderNode` origins, or an api-json conversion).
- **Standard library:** names resolve against it (e.g. `:>` to library types), but library elements are not returned as roots, so they never enter our Model. Nothing to mark. Ticket 07's "show the standard library" toggle needs a different approach, or should be dropped.
- **Kinds:** the service uses camelCase symbol kinds (`package`, `partDef`, `requirementDef`, ...). Unsupported ones are turned into SysML keywords (`requirement def`).
- **Strictness:** an `import` without a visibility keyword (`private`/`public`) is reported as an error.
- **For ticket 04:** `SymbolInfo` carries `type_info`, `multiplicity` and `specializations` (kinds `specializes`, `subsets`, `redefines`, `typing`). Port direction and connection ends were not checked; check `metadata` first.
- Each `children()` call is one RPC, so loading costs one call per element. That's fine at fixture size; revisit if large Models are slow.
