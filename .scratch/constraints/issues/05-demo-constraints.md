# 05: Demo constraints

**What to build:** Both mass-roll-up variants gain asserted constraints:

- the tank's fuel mass within its maximum, asserted on its part def and so inherited by `vehicle.fuelTank`;
- a power-to-weight constraint def asserted on the vehicle with bound parameters.

The failing variant breaks one and the passing variant meets both. The demo Styles get constraint rules, the README pictures and text are refreshed, and the service tests cover the new results. See `.scratch/constraints/spec.md`.

**Blocked by:** 03

**Status:** done

- [x] Constraints in both variants: failing breaks one, passing meets both (checked against the service)
- [x] Demo Styles colour constraint results and offending parts
- [x] README text and pictures updated; the picture test passes
- [x] The `check` command fails on `failing/` and passes on `passing/`

## Comments

- Done.
  - **Constraints:** both variants gain `FuelTank::fuelFits { fuelMass <= fuelMassMax }`, asserted on the definition and so inherited by `vehicle.fuelTank`, and `vehicle::powerToWeight : MassPerPower` (≤ 16 kg per kW) with bound parameters. Failing overfills the tank (50 > 45); passing carries 40. Power-to-weight holds in both (≈ 15.1 and 14.8).
  - **Demo Styles:** new rules colour parts with a violated constraint, and violated constraint nodes, red.
  - **README:** a constraints table, updated descriptions and a "make the tank's fuel fit" step. The CLI examples now use `check`.
  - **Pictures:** refreshed, and the picture test passes. The service tests check both constraints in both variants.
