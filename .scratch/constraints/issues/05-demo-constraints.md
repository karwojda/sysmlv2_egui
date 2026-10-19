# 05: Demo constraints

**What to build:** Both mass-roll-up variants gain asserted constraints:

- the tank's fuel mass within its maximum, asserted on its part def and so inherited by `vehicle.fuelTank`;
- a power-to-weight constraint def asserted on the vehicle with bound parameters.

The failing variant breaks one and the passing variant meets both. The demo Styles get constraint rules, the README pictures and text are refreshed, and the service tests cover the new results. See `.scratch/constraints/spec.md`.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] Constraints in both variants: failing breaks one, passing meets both (checked against the service)
- [ ] Demo Styles colour constraint results and offending parts
- [ ] README text and pictures updated; the picture test passes
- [ ] The `check` command fails on `failing/` and passes on `passing/`
