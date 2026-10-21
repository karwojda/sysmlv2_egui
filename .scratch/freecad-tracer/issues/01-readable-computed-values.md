# 01: Readable computed values

**What to build:** Everywhere a computed value is shown (Computed Values, Analysis Results, constraint parameters, CLI output), a `Rational` or `BigInteger` magnitude from the service appears as a decimal number (`157/20 kg` → `7.85 [kg]`), and units read like the Model rather than the service's spelling (`mm**3` → `mm³`, `SI::'kg⋅m⁻³'` → `kg/m³`, `SI::kg` → `kg`). This works for today's Models and keeps roll-ups computed from Design Values from showing blank.

See `.scratch/freecad-tracer/spec.md` (stories 39, 40).

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] A fraction-valued result (such as a mass from `volume * density`) shows as a decimal number, not blank
- [ ] `BigInteger` magnitudes show as numbers
- [ ] Power units show with superscripts (`mm³`); `SI::` prefixes and quoting are dropped; `⋅…⁻³` compound units show as `kg/m³`
- [ ] Applies to every place computed values are shown, not only Design Values
- [ ] Tested where computed values are tested today (`viewer-core/tests/computed.rs` and the service translation tests)
