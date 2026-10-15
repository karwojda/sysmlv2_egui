# OpenSysML as a sidecar process for parsing and resolution

We parse and resolve SysML v2 through OpenSysML's `sysml-grpc` service, a separate Go process the app talks to with the `opensysml` Rust client, rather than through a Rust-native parser. Drawing connections needs real name resolution and the standard library, and today only OpenSysML provides both. The Rust alternatives are syster-base (alpha, unknown semantic coverage) and tree-sitter-sysml (syntax only). The rest of the app depends only on our own model-loading interface, never on `opensysml` types, so the backend can be swapped later.

## Consequences

- The app has to ship or locate a Go binary and manage its lifecycle.
- A future web (wasm) build would reach the service over gRPC-Web/Connect rather than running a parser in-process.
- We let the `opensysml` client find, download and run `sysml-grpc` instead of bundling it with the app. It looks at `$OPENSYSML_GRPC_BINARY`, then its cache in `~/.opensysml/bin`, then downloads the matching release from GitHub (checked against a pinned digest), then falls back to `PATH`. So the first run on a machine, CI included, needs network access unless a binary is configured.
- The backend reads two views of a parsed Model. The stable symbol tree gives kinds, owners, typing (including standard-library types, which the other view names only by opaque ids), multiplicity, specializations and port direction. The standard API's JSON element form, which the service marks experimental, adds connection ends, documentation, port conjugation and source lines. If that conversion fails, the Model still loads, with a warning, and without those properties. The contract tests catch any change in its vocabulary when the pinned service version is upgraded.
