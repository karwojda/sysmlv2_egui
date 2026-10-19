# Mass roll-up demo

A vehicle whose mass rolls up from its parts, with a mass budget it has to
meet. OpenSysML computes the totals and checks the requirements. The viewer
shows each requirement green (✓ satisfied), red (✗ violated) or grey (○ not
claimed), and the budget as a whole takes the worst of its parts.

```
vehicle      = powertrain + body + fuelTank + 4 × wheel
powertrain   = engine + transmission
```

There are two variants of the same Model, each in its own folder:

| Requirement | Limit | [`failing/`](failing) | [`passing/`](passing) |
|---|---|---|---|
| `<1> vehicleMass` | 1400 kg | ✓ 1363 kg | ✓ 1333 kg |
| `<2> powertrainMass` | 250 kg | ✗ 265 kg (engine 180) | ✓ 235 kg (engine 150) |
| `<3> tankMass` | 70 kg | ○ nothing claims it | ✓ 60 kg |
| `massSpec` (all three) | | ✗ | ✓ |

## See it

```sh
# from the repository root
WAYLAND_DISPLAY= cargo run -p viewer-app -- examples/mass-rollup/failing
WAYLAND_DISPLAY= cargo run -p viewer-app -- examples/mass-rollup/passing
```

Open **mass-overview** under Saved Diagrams. It shows the parts tree with each
mass, the requirements with their limits, their `«satisfy»` edges and a
Legend, coloured by the Model Style in `diagrams/style.json`.

## Make it change

With the viewer open on `failing/`, edit its `vehicle.sysml` and save. The
viewer reloads and re-checks.

- **Turn the powertrain green:** set the engine's mass to `150.0`
  (150 + 85 = 235 ≤ 250).
- **Claim the tank requirement:** add
  `satisfy massSpec.tankMass by vehicle.fuelTank;` (60 ≤ 70).
- With both, `failing/` becomes `passing/` and `massSpec` turns green.
- **Turn the vehicle red:** set the body's mass to `1000.0`
  (265 + 1000 + 60 + 88 = 1413 > 1400).

## From the command line

```sh
cargo run -p viewer-cli -- requirements examples/mass-rollup/failing   # exits 1
cargo run -p viewer-cli -- requirements examples/mass-rollup/passing   # exits 0
cargo run -p viewer-cli -- export examples/mass-rollup/passing --diagram mass-overview --out mass-overview.svg
```
