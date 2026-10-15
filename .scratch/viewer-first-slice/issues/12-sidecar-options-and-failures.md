# 12: Sidecar options and failure handling

**What to build:** Make the sidecar usable outside the defaults, for both the viewer and the CLI. The `opensysml` client already finds, downloads and runs `sysml-grpc` (see ADR-0001), so this ticket passes the user's settings through and makes failures clear. A configured binary path sets `$OPENSYSML_GRPC_BINARY` for the client. A configured service URL uses an external connection and never starts or stops a process. Failures to find, download or start the service produce actionable messages.

**Blocked by:** 02: OpenSysML backend and sidecar manager

**Status:** done

- [x] A configured binary path is used instead of the cache or a download
- [x] A configured service URL (`host:port`) connects to an existing `sysml-grpc` and never starts or stops a process
- [x] A failed download with no binary on `PATH` produces a message saying how to configure a path or URL
- [x] A binary that exits or never becomes ready produces a message that includes its output
- [x] The viewer shows these messages in the UI; the CLI prints them and exits non-zero
- [x] No sidecar process is left running after the viewer or CLI exits, including after a crash in the app's own code

## Comments

- Done. `viewer_opensysml::ServiceConfig { binary, service }`, from `--sysml-binary <PATH>` / `--sysml-service <HOST:PORT>` on both programs. Flags win over `OPENSYSML_GRPC_BINARY` / `OPENSYSML_SERVICE`.
- **Binary:** the client reads it from the environment only, so each program's `main` sets the variable before any thread starts. The backend first checks that a configured binary exists, because the client otherwise ignores a missing path silently and downloads a release.
- **Service:** `Connection::external`. A service we didn't start is never started or stopped by us (tested: it keeps running after the backend is dropped).
- **Messages:** a missing binary, a failed download or an unsupported platform says how to configure a binary or service. A start failure includes the client's report. An unreachable service names its address, and a malformed address is rejected. The CLI prints them and exits 1; the viewer shows them in the top bar.
- **No orphaned service:** the client starts `sysml-grpc -exit-with-parent`, which exits when our process's pipe closes, even after SIGKILL. A test SIGKILLs a child holding the private service and checks the service is gone within 5s.
- Tests: `viewer-opensysml/tests/service.rs` (3 fast, 3 opt-in), plus 2 for the CLI flags.
