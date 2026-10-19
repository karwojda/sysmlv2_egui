# 06: Demo

**What to build:** Both mass roll-up variants gain `massAnalysis`: subject the vehicle, returning its total mass, with an objective of ≤ 1400 kg. The `Vehicle` definition declares the parts the analysis reads, so the Model has no errors.

The pictures show computed totals (`= 1363.0`) and the analysis node with its badge. The demo Styles colour the analysis, the README explains it, and the service tests cover it. See `.scratch/analysis/spec.md`.

**Blocked by:** 05

**Status:** ready-for-agent

- [ ] `massAnalysis` in both variants, with no Model errors, holding in both (1363, 1333)
- [ ] Pictures show computed totals and the analysis node; the picture test passes
- [ ] README updated; `check` lists the analysis
