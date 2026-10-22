# 01: Readable computed values

**What to build:** Everywhere a computed value is shown (Computed Values, Analysis Results, constraint parameters, CLI output), a `Rational` or `BigInteger` magnitude from the service appears as a decimal number (`157/20 kg` → `7.85 [kg]`), and units read like the Model rather than the service's spelling (`mm**3` → `mm³`, `SI::'kg⋅m⁻³'` → `kg/m³`, `SI::kg` → `kg`). This works for today's Models and keeps roll-ups computed from Design Values from showing blank.

See `.scratch/freecad-tracer/spec.md` (stories 39, 40).

**Blocked by:** None (can start immediately)

**Status:** done

- [x] A fraction-valued result (such as a mass from `volume * density`) shows as a decimal number, not blank
- [x] `BigInteger` magnitudes show as numbers
- [x] Power units show with superscripts (`mm³`); `SI::` prefixes and quoting are dropped; `⋅…⁻³` compound units show as `kg/m³`
- [x] Applies to every place computed values are shown, not only Design Values
- [x] Tested where computed values are tested today (`viewer-core/tests/computed.rs` and the service translation tests)

## Comments

- Done.
  - **Numbers:** `value_text` (`viewer-opensysml/src/checks/service.rs`) writes a `Rational` value or magnitude as the decimal its `f64` comes to (`157/20 [SI::kg]` → `7.85 [kg]`, `1/3` → `0.3333333333333333`), and a `BigInteger` as its digits. Both used to come out blank.
  - **Units:** package prefixes and quotes are dropped (`SI::kg` → `kg`, `SI::'m³'` → `m³`), `**n` and `^n` become superscripts (`mm**3` → `mm³`), and a `⋅` product with negative powers becomes a quotient (`kg⋅m⁻³` → `kg/m³`, `kg⋅m⁻¹⋅s⁻²` → `kg/(m⋅s²)`). A unit with nothing above the line stays as it is (`s⁻¹`).
  - **Everywhere:** one function feeds Computed Values (rows, Properties, constraint parameters) and Analysis Results (GUI and CLI), so all of them change. Literal rows still show the value as written.
  - **What the service sends** (probed): products and quotients come back in SI (`volume * 2` on `1.0e6 [mm^3]` is `0.002 [SI::'m³']`, `mass / volume` is `7850.0 [SI::'kg⋅m⁻³']`), and sums keep the operand unit as `mm**3`. Noted in `docs/opensysml-encoding.md`.
  - **Tests:** the service translation tests (fractions, BigIntegers, twelve unit spellings); a `quantities` fixture with a contract test through the real service (`7.85 [kg]`, `0.002 [m³]`, `7850.0 [kg/m³]`, `2000000.0 [mm³]`, a BigInteger); `viewer-core/tests/computed.rs` gains a quantity row.
  - **Deviation:** none in behaviour. Formatting lives in the adapter, which knows the service's spelling, so the `viewer-core` test documents the display rather than driving it.
