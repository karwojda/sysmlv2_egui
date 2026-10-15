# 12: Sidecar options and failure handling

**What to build:** Make the sidecar usable outside the defaults, for both the viewer and the CLI. The `opensysml` client already finds, downloads and runs `sysml-grpc` (see ADR-0001), so this ticket passes the user's settings through and makes failures clear. A configured binary path sets `$OPENSYSML_GRPC_BINARY` for the client. A configured service URL uses an external connection and never starts or stops a process. Failures to find, download or start the service produce actionable messages.

**Blocked by:** 02: OpenSysML backend and sidecar manager

**Status:** ready-for-agent

- [ ] A configured binary path is used instead of the cache or a download
- [ ] A configured service URL (`host:port`) connects to an existing `sysml-grpc` and never starts or stops a process
- [ ] A failed download with no binary on `PATH` produces a message saying how to configure a path or URL
- [ ] A binary that exits or never becomes ready produces a message that includes its output
- [ ] The viewer shows these messages in the UI; the CLI prints them and exits non-zero
- [ ] No sidecar process is left running after the viewer or CLI exits, including after a crash in the app's own code
