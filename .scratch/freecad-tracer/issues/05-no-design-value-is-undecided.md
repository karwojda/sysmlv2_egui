# 05: No Design Value means Undecided, never a fallback

**What to build:** When a link can't be answered (the tool gives a reason for the whole file or for one property), the dependent checks are Undecided and say why, as `no Design Value for drive::gearbox::volume (FreeCAD: <reason>)`, never the service's raw type-mismatch text. Checks that don't depend on that feature are still judged. A missing Design Value never falls back to the Model Value or to a def's `default`. A Design Value whose dimension doesn't fit the feature (a volume linked to a mass) makes the check Undecided with a clear reason. Two diagnostics appear: a `@ToolProperty` whose owner has no Tool Link, and one element with two Tool Links to the same tool.

The mechanism (from the prototype): the same edit as ticket 03/04 writes a marker string in place of the value, so OpenSysML reports dependents Undecided with the marker in its error chain; the backend translates the marker in every check reason. Tests assert only the translated text, never the marker.

See `.scratch/freecad-tracer/spec.md` (stories 14, 21–24, 33).

**Blocked by:** 04 (Every feature shape takes a Design Value)

**Status:** ready-for-agent

- [ ] A failed link makes exactly the dependent checks Undecided, with the translated reason; independent checks keep their Verdicts
- [ ] A failed link on an inherited-`default` redefinition doesn't let the `default` show through
- [ ] A whole-file error applies to every property asked of that file
- [ ] A dimension mismatch is Undecided with a clear reason
- [ ] The Model records the reason as the feature's Design Value outcome
- [ ] Diagnostic for `@ToolProperty` on a feature whose owner has no Tool Link
- [ ] Diagnostic for two Tool Links to one tool on one element
- [ ] All through the real backend with a fake tool
