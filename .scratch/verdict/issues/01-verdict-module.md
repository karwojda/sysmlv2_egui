# 01: One Verdict module

**What to build:** every badge, Properties row, Style match and `check` line about a Satisfaction Status, Constraint Result or Analysis Result comes from one Verdict module, on one scale with each kind's own words. Unrun analyses and unchecked constraints read ○ absent, and Styles can match them. See `.scratch/verdict/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] Verdict module with `of` / `all`, kinds, scale and findings; old status types deleted
- [x] Badge, Properties, Styles, Style editor and CLI ask it
- [x] Analysis not run and constraint not checked read ○, with `not-run` / `not-checked` Style values and Legend labels
- [x] Tests moved to the Verdict interface; other output unchanged

Note: a part's `assert constraints` row for a constraint never checked on that part now reads ○ too, following Q9.
