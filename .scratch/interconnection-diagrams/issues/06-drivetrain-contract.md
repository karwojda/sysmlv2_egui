# 06: Drivetrain fixture through OpenSysML (contract)

**What to build:** The drivetrain as a `.sysml` fixture for the backend's opt-in contract tests. The real Model must match the fake-backend twin used by the Interconnection tests, including connection ends three features deep and ends at the definition's own ports.

**Blocked by:** 01: Interconnection tracer

**Status:** needs-triage

- [ ] `viewer-opensysml/tests/fixtures/drivetrain/` models the spike's drivetrain in SysML v2
- [ ] A contract test asserts the OpenSysML backend returns the same Model as the fake twin
- [ ] Any gap (e.g. chains through nested parts not resolved) is recorded in the ticket's Comments section
