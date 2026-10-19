# 04: The off switch

**What to build:** `diagrams/settings.json` (`{ "format_version": 1, "run_analyses": true }`) controls automatic analysis runs. A missing file means defaults; a broken one gives a warning and defaults.

- **Viewer:** a "Run analyses on load" checkbox writes the file and reloads. "Run now" in an analysis's Properties runs it once when switched off.
- **Not run:** an analysis that wasn't run reads ? "not run".
- **Slow loads:** a warning appears when a load's analyses take over 3 s, naming the setting.

See `.scratch/analysis/spec.md`.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] Settings file round trip, missing and broken
- [ ] The backend loads with or without analyses, and can run one on demand
- [ ] The checkbox and "Run now" in the viewer; "not run" shown when skipped
- [ ] The slow-load warning
