# How do Design Values appear in the viewer?

**Status:** resolved
**Type:** prototype
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

How does the GUI show that an element has a Tool Link, which features carry a Design Value, the Model Value beside it ("212 kg, Model says 200"), and why a feature has no Design Value? In Diagrams (General, Interconnection) and in the Properties panel. How are a link to a missing tool or file, and a Verdict that rests on Design Values, told apart from ordinary ones?

## Answer

**Inline in Diagrams and Properties (variant A), plus a Model-wide Design Values table (variant C).** Chosen by the user from three variants of a UI prototype on local branch `prototype/design-values-ui` (ecc5f68; not pushed: no GitHub credentials). Run it with `cargo run -p viewer-app -- prototypes/design-values-ui/model`, open the General Diagram of `Drive`, switch variants with the bottom bar or ←/→.

- **Diagrams:** a compartment row of a linked usage carries its tool and object: `engine:Engine ⚙ FreeCAD Engine`; a link with any missing Design Value ends in `⚠`. A linked feature's own row shows the Design Value with the Model Value after it: `volume = 1000000.0 [mm³] ⚙ (Model: 900000.0 [mm³])`, or `volume = ⚠ no Design Value (Model: …)`. No new compartments: the drawing stays Standard Notation plus marks.
- **Properties panel:** a linked element gets a **Tool Link** row (`⚙ FreeCAD · cad/engine.FCStd · Engine`). For a linked feature, the Value row becomes **Design Value** (with its tool, object and property, or `⚠ none: <reason>`) followed by **Model Value**. A Verdict that rests on Design Values says so.
- **Design Values tab** beside Properties and Style: every linked feature in the Model with Design Value, Model Value, difference (absolute and %), source, or the failure reason; then the Verdicts resting on Design Values. Rows select their element.
- **Rejected:** variant B, a "design values" compartment on the owning node and a separate Properties section. It shows values without selecting anything, but adds a non-standard compartment that would also appear in Exports.
- **Found on the way:**
  - A General Diagram draws nested usages only as rows in their owner's compartments (`engine:Engine`), so a linked feature's value is mostly seen in Properties and the table, not on a node.
  - The viewer's Model does contain unnamed redefinitions (`:>> volume`) under the redefined name (`Drive::drive::engine::volume`), so they can be selected.
  - The "Computed" row and computed values in rows must show results from the Design Values run; the fraction-valued gap (`Rational` shows nothing, e.g. `drive.mass`) has to be fixed for roll-ups to show at all.
