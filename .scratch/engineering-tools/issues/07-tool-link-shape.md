# What does a Tool Link look like?

**Status:** resolved
**Type:** grilling
**Blocked by:** 01, 04, 06
**Map:** [Engineering Tools](../map.md)

## Question

What metadata def expresses a Tool Link in the Model, and where does that def live (a library package shipped with this tool)? What does it address: the file (relative path), the object inside it, and which property maps to which feature? How are units declared and converted? Can one element link to several Engineering Tools? Since FreeCAD's mass is volume × a density that may be missing or wrong (see the answer to "What can FreeCAD report headless?"), does a Tool Link pull mass, or pull volume and take the density from the Model?

## Answer

**Two metadata defs from a library copied into the Model.** Resolved by grilling, recorded in ADR 0004 (`docs/adr/0004-tool-links-are-metadata-from-a-copied-library.md`).

```sysml
part engine : Engine {
    @ToolLink { tool = "FreeCAD"; file = "cad/engine.FCStd"; object = "Assembly.Link.Engine."; }
    attribute volume :>> volume { @ToolProperty { name = "Volume"; } }
    attribute mass :>> mass = volume * density;     // Computed Value from a Design Value
}
```

1. **Split:** `ToolLink { tool; file; object; }` on the element, `ToolProperty { name; tool? }` on each linked feature, as the standard library splits `ToolExecution`/`ToolVariable`, so feature names are checked by the parser.
2. **Our own defs**, shaped like `AnalysisTooling` but not reusing it: `ToolExecution` means an action is carried out by a tool.
3. **Where:** a library package `EngineeringTools` in a `.sysml` file users copy into their Model. The tool ships it (for example `sysml-viewer library`), recognises the defs by qualified name, and never writes it.
4. **Addressing:** `tool` is explicit (not inferred from the extension). `file` is relative to the `.sysml` file holding the link. `object` is one string whose syntax the tool's adapter defines: for FreeCAD the object `Name` plus the assembly path; for KiCad sheet path plus UUID, or a net name.
5. **Units:** none on the link. The adapter knows the unit of each property it reports and writes the Design Value with it (`1.0e6 [mm^3]`); OpenSysML converts. Probed against sysml-grpc 0.10.0: cross-unit comparisons and derived units come out exact (`1.0e6 [mm^3] * 7850 [kg/m^3]` = 7.85 kg); a missing unit or wrong dimension makes the check Undecided with a clear error, never a wrong answer; `set_value(…, "212 [kg]")` works. Write units with `^` in brackets (`[kg/m^3]`), not superscripts.
6. **FreeCAD mass:** the recommended route is linking `Volume` and computing mass in the Model from a density Model Value. `Mass` is allowed, but gives no Design Value (so Undecided) when the object's material is "Default" or not in the local material library.
7. **Several links:** an element may carry several Tool Links, one per tool; `ToolProperty.tool` picks one only when ambiguous. Two links to the same tool on one element is a diagnostic.
8. **Where links go:** on usages of any kind (part, item, port, connection); each adapter decides which of its objects fit. Not on defs. A `@ToolProperty` must sit on a feature owned or redefined by the linked element, otherwise a diagnostic (no search upward).
9. **Scalars only:** vector properties are reported per component (`CenterOfMass.x`, …).

Probe Models and client: session scratchpad `units/` (not kept).
