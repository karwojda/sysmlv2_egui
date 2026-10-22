# Feather budget demo

A board designed in KiCad, and the envelope it must fit. `part feather`
in [`feather.sysml`](feather.sysml) is linked to the KiCad project in
[`ecad/`](ecad), the RoyalBlue54L Feather from KiCad 10's demos
(CERN-OHL-P-2.0, see [`ecad/NOTICE`](ecad/NOTICE)):

```sysml
@ToolLink { tool = "KiCad"; file = "ecad/RoyalBlue54L-Feather.kicad_pro"; object = "board"; }
attribute :>> width = 50.8 [mm] { @ToolProperty { name = "Width"; } }
```

On every load the viewer and `sysml-viewer check` ask `kicad-cli` for the
board's `Width`, `Height`, `Thickness` and `Area`, and judge the
requirements against those Design Values rather than the estimates the
Model writes:

| Requirement | Limit | Model says | KiCad says |
|---|---|---|---|
| `<1> boardWidth` | 60 mm | 50.8 mm | ✓ 58.42 mm |
| `<2> boardThickness` | 1.6 mm | 1.6 mm | ✓ 1.6 mm |

KiCad 10 or newer is needed. It is found through `SYSML_VIEWER_KICAD`,
then `PATH`, then the usual install locations (including a Windows install
seen from WSL). Without it the checks that depend on the board are
Undecided, and `check` fails.

[`EngineeringTools.sysml`](EngineeringTools.sysml) is the library
`sysml-viewer library` prints, copied in so the links resolve.
