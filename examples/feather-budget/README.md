# Feather budget demo

A board designed in KiCad, and the budgets it must meet. `part feather`
in [`feather.sysml`](feather.sysml) is linked to the KiCad project in
[`ecad/`](ecad), the RoyalBlue54L Feather from KiCad 10's demos
(CERN-OHL-P-2.0, see [`ecad/NOTICE`](ecad/NOTICE)):

```sysml
@ToolLink { tool = "KiCad"; file = "ecad/RoyalBlue54L-Feather.kicad_pro"; object = "board"; }
attribute :>> width = 50.8 [mm] { @ToolProperty { name = "Width"; } }
attribute :>> power = 0.3 [W] { @ToolProperty { name = "Power"; } }
attribute :>> componentCount = 40 { @ToolProperty { name = "ComponentCount"; } }
```

On every load the viewer and `sysml-viewer check` ask `kicad-cli` for the
board's `Width`, `Height`, `Thickness` and `Area` from the layout, its
`ComponentCount` (the placed parts, leaving out do-not-populate and
excluded-from-BOM parts), and the `Power` and `Price` symbol fields added
up over the placed parts. They judge the requirements against those
Design Values rather than the estimates the Model writes:

| Requirement | Limit | Model says | KiCad says |
|---|---|---|---|
| `<1> boardWidth` | 60 mm | 50.8 mm | ✓ 58.42 mm |
| `<2> boardThickness` | 1.6 mm | 1.6 mm | ✓ 1.6 mm |
| `<3> boardPower` | 0.5 W | 0.3 W | ✓ 0.060953 W, summed 7 of 48 parts |
| `<4> boardCost` | 15 | 10 | ✓ 12.8, summed 7 of 48 parts |
| `<5> boardPartCount` | 60 | 40 | ✓ 48 |

Only the seven active parts have `Power` and `Price` fields, with
illustrative figures (see `ecad/NOTICE`); the other 41 placed parts are
listed as uncounted. A part's field may carry a unit with an SI prefix
(`45mW`, `350µW`, `1.2 W`); the sum is in the base unit. Prices are bare
numbers, so their sum has no unit.

KiCad 10 or newer is needed. It is found through `SYSML_VIEWER_KICAD`,
then `PATH`, then the usual install locations (including a Windows install
seen from WSL). Without it the checks that depend on the board are
Undecided, and `check` fails.

[`EngineeringTools.sysml`](EngineeringTools.sysml) is the library
`sysml-viewer library` prints, copied in so the links resolve.
