# 01: The backend's checks behind one seam

**What to build:** the OpenSysML backend's claim verification, part checks and analysis runs go through one `checks` module and a `Checker` seam, so every rule about which elements get which results is tested without the service. Loads behave exactly as before. See `.scratch/checks/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `checks::run` is the backend's only call for checks; the settings file is read there
- [x] One service-verdict translation, in the real adapter
- [x] Tests across `checks::run` with a test `Checker` replace the `record()` tests
- [x] Contract tests and demo pictures pass unchanged
