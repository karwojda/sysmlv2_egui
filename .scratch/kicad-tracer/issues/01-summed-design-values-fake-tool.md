# 01: Summed Design Values from a fake tool to the CLI and Properties

**What to build:** A **Summed Design Value** travels from a tool's answer to everything the user sees. A tool's read value can now carry its parts: the parts counted and the parts left uncounted, by reference designator. The backend carries them into the feature's Design Value outcome. Checks use the value as they use any Design Value (ADR 0003).

`check` and `requirements` print a summed feature's Design Values line with `(summed N of M parts; K uncounted: r1, r2, … and J more)`. The list shows at most ten refs, in natural order (`C2` before `C10`). It reads `(summed N of N parts)` when nothing is uncounted.

The Properties Design Value row shows the value, `⚙ <tool> <object>.<property>` and "summed N of M parts", then every uncounted ref beneath. A value with no parts, such as FreeCAD's, looks exactly as it does today.

Proven with a fake `EngineeringTool` answering a summed `Power` for `board`: a power requirement that held on the Model Value is now violated. No KiCad is involved.

See `.scratch/kicad-tracer/spec.md` (stories 18, 19, 31, 32, 35, 36), ADRs 0003 and 0006, and **Summed Design Value** in `CONTEXT.md`.

**Blocked by:** None (can start immediately)

**Status:** ready-for-agent

- [ ] A read value in the trait's answer and in `DesignValue::Read` can carry counted and uncounted parts; FreeCAD answers carry none and its tests pass unchanged
- [ ] Fixture Model loaded through the real backend with a fake tool: a summed `Power` flips a power requirement to violated, and the outcome records its counted and uncounted parts
- [ ] A reason for one summed property leaves only its dependent checks Undecided
- [ ] CLI: the summed line, the ten-ref cut with "and J more", natural order, and the form with nothing uncounted (tests with `FakeBackend`)
- [ ] CLI output for a Model whose Design Values have no parts is unchanged
- [ ] Properties: the summed Design Value row with every uncounted ref beneath; rows for a value without parts unchanged
