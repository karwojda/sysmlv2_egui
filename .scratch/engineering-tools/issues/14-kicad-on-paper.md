# Does the design hold for KiCad?

**Status:** resolved
**Type:** grilling
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

Walk KiCad through the decisions so far on paper: Tool Link addressing (sheet path plus UUID, net name), which SysML usages link to components, nets and the board, which properties an adapter would report (board size and thickness from `kicad-cli pcb export stats`, symbol fields), how text fields like `"10k"` or `"100nF"` become quantities with units, and whether reading S-expressions directly (no KiCad installed) is acceptable. Note anything in ADRs 0003 to 0005 that would have to change.

## Answer

**The design holds for KiCad; ADRs 0003 to 0005 need no change.** Resolved by grilling, on paper only (KiCad not installed). ADR 0004 already makes `object` a per-tool string with an explicit `tool`; ADR 0005's adapter trait and per-machine lookup cover `kicad-cli` (`SYSML_VIEWER_KICAD`, `PATH`, known locations); ADR 0003 is untouched.

```sysml
part controller : MotorController {
    @ToolLink { tool = "KiCad"; file = "ecad/controller.kicad_pro"; object = "board"; }
    attribute :>> thickness { @ToolProperty { name = "BoardThickness"; } }
    part r12 : Resistor {
        @ToolLink { tool = "KiCad"; file = "ecad/controller.kicad_pro"; object = "R12"; }
        attribute :>> resistance { @ToolProperty { name = "Value"; } }
    }
}
```

1. **File:** a KiCad Tool Link points at the project (`.kicad_pro`); the adapter finds the schematic and board. The modeller needn't know which file holds which fact.
2. **Object:** a reference designator (`R12`), a stable `uuid:/<sheet>/<symbol>` path, `board`, or `net:<name>` (a labelled net, e.g. `net:/power/VBUS`). The Design Values tab shows the other form of each resolved component, so a modeller can switch to the UUID once the design settles (reference designators change on re-annotation).
3. **Text fields to quantities:** the adapter knows KiCad's standard symbols (`Device:R` → Ω, `Device:C` → F, `Device:L` → H); explicit unit letters always win (`100nF`); RKM notation is understood (`4k7`). Anything else gives no Design Value with a reason ("no unit in '10k' for symbol MyLib:Res"). Keeps "no unit on the link" and keeps the adapter free of SysML.
4. **Always `kicad-cli`:** `sch export netlist` (components, fields, nets) and `pcb export stats --format json` (board width, height, area, thickness; KiCad 10). No direct S-expression reading: the format changed more than ten times in the 10.0 cycle and nets need `kicad-cli` anyway. Requiring KiCad matches requiring FreeCAD. Query speed unmeasured.
5. **Nets** can be linked (identity, the ⚙ mark on a connection) but report no Design Values. Checking connectivity against the Model is out of scope.
