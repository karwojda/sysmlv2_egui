# Does an Engineering Tool's value replace or challenge the Model's?

**Status:** resolved
**Type:** grilling
**Blocked by:** 02
**Map:** [Engineering Tools](../map.md)

## Question

If the Model says `mass = 200 [kg]` for a part and FreeCAD reports 212 kg for the linked body, what happens? Is FreeCAD's value used in the checks in place of the Model's, is the mismatch a finding (a Verdict) in its own right, or should a linked feature carry no value in the Model at all? What does each mean for requirements and budgets?

## Answer

**Replace.** Resolved by grilling, recorded in ADR 0003 (`docs/adr/0003-design-values-replace-model-values.md`); terms **Design Value** and **Model Value** added to `CONTEXT.md`.

1. Checks (Satisfaction Status, Constraint Results, Computed Values, Analysis Results) use the **Design Value** in place of the **Model Value**, applied by editing the Model in memory and re-parsing, never on disk. In the example, `engine.mass` is 212, `vehicle.mass` rolls up to 262, and the requirement is Violated.
2. Methodology: Model Values carry estimates and budgets before the design exists; adding a Tool Link is the step that says the design now decides. Budgets belong in requirements and Asserted Constraints, not attribute values.
3. The difference between Design Value and Model Value is shown beside the feature ("212 kg, Model says 200"), never a Verdict. Whether it matters is for requirements and constraints to judge.
4. A Tool Link whose Design Value can't be had (tool missing, file or object missing, query failed) makes the dependent checks Undecided with the reason. No fallback to the Model Value.
5. All three ways of writing the feature are accepted (`=`, `default =`, no value), with no rule or warning for the modeller.
6. A Design Value applies only to the usage carrying the Tool Link, not its def or sibling usages. When the Model Value is only inherited from the def, the in-memory edit must add a redefinition on the usage (untested in "Can OpenSysML evaluate with values supplied from outside?").
7. Checks run once, with Design Values. A run on Model Values only could be a CLI option later.
