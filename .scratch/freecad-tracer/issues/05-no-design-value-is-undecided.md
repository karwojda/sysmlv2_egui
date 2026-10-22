# 05: No Design Value means Undecided, never a fallback

**What to build:** When a link can't be answered (the tool gives a reason for the whole file or for one property), the dependent checks are Undecided and say why, as `no Design Value for drive::gearbox::volume (FreeCAD: <reason>)`, never the service's raw type-mismatch text. Checks that don't depend on that feature are still judged. A missing Design Value never falls back to the Model Value or to a def's `default`. A Design Value whose dimension doesn't fit the feature (a volume linked to a mass) makes the check Undecided with a clear reason. Two diagnostics appear: a `@ToolProperty` whose owner has no Tool Link, and one element with two Tool Links to the same tool.

The mechanism (from the prototype): the same edit as ticket 03/04 writes a marker string in place of the value, so OpenSysML reports dependents Undecided with the marker in its error chain; the backend translates the marker in every check reason. Tests assert only the translated text, never the marker.

See `.scratch/freecad-tracer/spec.md` (stories 14, 21–24, 33).

**Blocked by:** 04 (Every feature shape takes a Design Value)

**Status:** done

- [x] A failed link makes exactly the dependent checks Undecided, with the translated reason; independent checks keep their Verdicts
- [x] A failed link on an inherited-`default` redefinition doesn't let the `default` show through
- [x] A whole-file error applies to every property asked of that file
- [x] A dimension mismatch is Undecided with a clear reason
- [x] The Model records the reason as the feature's Design Value outcome
- [x] Diagnostic for `@ToolProperty` on a feature whose owner has no Tool Link
- [x] Diagnostic for two Tool Links to one tool on one element
- [x] All through the real backend with a fake tool

## Comments

- Done.
  - **Marker:** a `Missing` outcome now writes the string `"no Design Value #N#"` in place of the value, through the same `How` as a Design Value (`SetValue`, or delete + `add_member` for a valueless redefinition, so a def's `default` can't show through). OpenSysML reports exactly the dependent checks Undecided with the marker in its error chain. `Plan::translate` replaces it, in every claim, constraint check and analysis objective result, with `no Design Value for <feature> (<tool>: <reason>)`; several markers in one reason are joined with `; `.
  - **Dimension mismatch:** the service's `feature value pump.mass: type mismatch: cannot write …` is translated to `Design Value for <feature> (<tool>: <object>.<property>) does not fit it: cannot write …`, matched against the features that were given a Design Value.
  - **Diagnostics (warnings):** `@ToolProperty on <feature>, but <owner> has no Tool Link: it takes no Design Value`, at the feature's own source line (read from its unnamed `@N` item, which has no element of ours: new `LinkedProperty.source`); and `<element> has N Tool Links to <tool>: its features take no Design Value`. An element's features with an ambiguous link are not asked for and get a `Missing` outcome saying so.
  - **Fixtures:** `no_design_values/` (failed object, failed file, wrong dimension, unlinked spares) and `tool_link_mistakes/`. Five new tests in `viewer-opensysml/tests/design_values.rs` assert only the translated text. `docs/opensysml-encoding.md` records the marker, the type-mismatch text and where an unnamed redefinition's source line lives.
  - **Named feature with a body and no value** (closes ticket 04's recorded gap, spec story 8): `attribute bore : LengthValue { @ToolProperty … }` used to get `delete` + `add_member(redefines = ["bore"])`, which the service refuses (`EDIT_FAILURE_DELETE_REFERENCED` when a constraint uses it). No editor operation can value it (see `docs/opensysml-encoding.md`), so a new `How::InText` writes ` = <value>` into the text before its body or `;`, at the declaration's line and column (`LinkedProperty.named_at`, from `declaredName` + `sysx:sourceColumn`), then the service's edits run on that text. It keeps its name and type, and it takes the marker the same way. Fixture `named_design_values/`, service test `a_named_feature_with_a_body_and_no_value_takes_the_design_value_or_is_undecided` (8 mm Violated, `spare` Undecided with `no Design Value for Drive::drive::spare::bore (FreeCAD: no object Spare)`), unit tests for `write_value` (body, `;`, braces in quoted names and comments, a place that starts no declaration).
