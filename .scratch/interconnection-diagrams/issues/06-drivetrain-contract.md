# 06: Drivetrain fixture through OpenSysML (contract)

**What to build:** The drivetrain as a `.sysml` fixture for the backend's opt-in contract tests. The real Model must match the fake-backend twin used by the Interconnection tests, including connection ends three features deep and ends at the definition's own ports.

**Blocked by:** 01: Interconnection tracer

**Status:** done

- [x] `viewer-opensysml/tests/fixtures/drivetrain/` models the spike's drivetrain in SysML v2
- [x] A contract test asserts the OpenSysML backend returns the same Model as the fake twin
- [x] Any gap (e.g. chains through nested parts not resolved) is recorded in the ticket's Comments section

## Comments

- Done. `viewer-opensysml/tests/fixtures/drivetrain/drivetrain.sysml` mirrors the core tests' twin. `contract.rs` includes the twin by path (`#[path = "../../viewer-core/tests/common/mod.rs"]`), so there's one definition. Source locations are cleared before comparing, since the twin doesn't carry them.
- No gaps. The real Model matches: ends at the definition's own ports (`AxleAssembly::drive` from `torqueIn`, `Vehicle::supply` from `power`), the interface's `supply ::>` / `demand ::>` ends, and a three-feature end (`Vehicle::leftDrive` to `rearAxleAssembly.leftWheel.hub`), added for ticket 04.
- The contract test caught a bug in the twin: `part def Wheel` was missing, so the wheels had no ports. Fixed. `FuelInterface` got `supply`/`demand` ends in both, because the service rejects an interface definition without ends.
