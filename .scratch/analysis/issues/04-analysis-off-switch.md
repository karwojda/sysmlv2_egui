# 04: The off switch

**What to build:** `diagrams/settings.json` (`{ "format_version": 1, "run_analyses": true }`) controls automatic analysis runs. A missing file means defaults; a broken one gives a warning and defaults.

- **Viewer:** a "Run analyses on load" checkbox writes the file and reloads. "Run now" in an analysis's Properties runs it once when switched off.
- **Not run:** an analysis that wasn't run reads ? "not run".
- **Slow loads:** a warning appears when a load's analyses take over 3 s, naming the setting.

See `.scratch/analysis/spec.md`.

**Blocked by:** 03

**Status:** done

- [x] Settings file round trip, missing and broken
- [x] The backend loads with or without analyses, and can run one on demand
- [x] The checkbox and "Run now" in the viewer; "not run" shown when skipped
- [x] The slow-load warning

## Comments

- Done.
  - **Settings:** `viewer_core::settings` reads and writes `diagrams/settings.json` (`run_analyses`, default true; a missing field takes its default). The file is part of the watch fingerprint.
  - **Backend interface:** `ModelBackend::load_with(folder, &Analyses)`, where `Analyses` is `AsSet`, `All`, `None` or `Only(name)`. `load` defaults to `AsSet`. `OpenSysmlBackend` reads the settings for `AsSet` (a broken file warns and runs analyses), runs only the selected analyses, and warns when they take over 3 s.
  - **Viewer:**
    - The loader's requests carry the option.
    - The top bar has a "Run analyses on load" checkbox that saves the setting and reloads.
    - An analysis usage that wasn't run offers "Run now", which reloads running only it.
  - **Not run:** Properties reads "Objective ? not run" (or "Result ? not run" without an objective), and the badge is ?.
  - **Tests:** `viewer-core/tests/settings.rs` (4), the watch fingerprint, the Properties "not run" row, and contract tests for settings off / Run now / override and for a broken settings file. A screenshot checked the checkbox.
