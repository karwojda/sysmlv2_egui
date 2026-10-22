# Write the design note

**Status:** resolved
**Type:** task
**Blocked by:** 11, 12, 13, 14
**Map:** [Engineering Tools](../map.md)

## Question

Write the short design note the destination asks for, linking ADRs 0003 to 0005 and the closed tickets: the pipeline from Tool Link to Verdict, the code layout, the decided edge cases, and what a FreeCAD tracer build must include, so it can go to `/to-spec`.

## Answer

Written: `docs/engineering-tools.md`. It gives the pipeline from Tool Link to Verdict, the code layout, the decided behaviour, the slices of a FreeCAD tracer build (library, link reading, FreeCAD adapter with `Volume`, Design Values and the marker, CLI `check`, Properties rows, and fixing fraction-valued results), the follow-on slices, and links to ADRs 0003 to 0005 and both prototype branches. Ready for `/to-spec`.
