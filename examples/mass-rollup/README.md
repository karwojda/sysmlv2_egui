# Mass roll-up demo

A vehicle whose mass rolls up from its parts, with a mass budget it has to
meet. OpenSysML computes the totals and checks the requirements; the viewer
shows each requirement green (✓ satisfied), red (✗ violated) or grey (○ not
claimed).

```
vehicle      = powertrain + body + fuelTank + 4 × wheel
powertrain   = engine + transmission
```

| Requirement | Limit | Claimed by | As shipped |
|---|---|---|---|
| `<1> vehicleMass` | 1400 kg | `vehicle` | ✓ 1363 kg |
| `<2> powertrainMass` | 250 kg | `vehicle.powertrain` | ✗ 265 kg |
| `<3> tankMass` | 70 kg | nothing yet | ○ |

## See it

```sh
# from the repository root
WAYLAND_DISPLAY= cargo run -p viewer-app -- examples/mass-rollup
```

Open **mass-overview** under Saved Diagrams: the parts tree, the requirements,
their `«satisfy»` edges and a Legend, coloured by the Model Style in
`diagrams/style.json`.

## Make it change

Edit `vehicle.sysml` with the viewer open and save. It reloads and re-checks.

- **Turn the powertrain green:** set the engine's mass to `160.0`
  (160 + 85 = 245 ≤ 250).
- **Turn the vehicle red:** set the body's mass to `1000.0`
  (265 + 1000 + 60 + 88 = 1413 > 1400).
- **Claim the tank requirement:** add
  `satisfy massSpec.tankMass by vehicle.fuelTank;` and it turns green (60 ≤ 70).

## From the command line

```sh
cargo run -p viewer-cli -- requirements examples/mass-rollup     # exits 1: powertrainMass is violated
cargo run -p viewer-cli -- export examples/mass-rollup --diagram mass-overview --out mass-overview.svg
```
