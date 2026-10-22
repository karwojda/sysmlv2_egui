# 04: Every feature shape takes a Design Value

**What to build:** A systems engineer can put `@ToolProperty` on a feature at any stage of their Model and it takes the Design Value: a feature with a value (ticket 03), a feature with a body and no value, and a usage that only redefines a feature inherited from its def (`attribute :>> volume { @ToolProperty … }`), where the def's `default` stays the estimate and the usage takes the design's value. Unnamed redefinitions are named by following `redefines`, recursively, as the backend already does for values. Because `set_value` refuses declarations with a body, such features are replaced in memory (delete, then add a redefining member with the value).

See `.scratch/freecad-tracer/spec.md` (stories 8, 9).

**Blocked by:** 03 (Design Values replace Model Values in load)

**Status:** done

- [x] A feature with a body and no value takes the Design Value, and its dependent checks use it
- [x] A `:>>` redefinition of an inherited feature with a def `default` takes the Design Value; the def and other usages keep the `default`
- [x] The Model's Design Value outcomes name these features by their resolved names
- [x] Fixture cases for all three shapes, through the real backend with a fake tool

## Comments

- Done.
  - **Edit:** `design_values::Edit` has `how: How`, either `SetValue` (feature with a value, as in 03) or `Redefine { owner, redefines }` (feature with a body and no value). `with_design_values` in `lib.rs` turns `Redefine` into `delete(feature)` + `add_member(owner, "attribute", "", redefines = [name], value)`, in the same per-document editor as the `set_value`s. The `has_value` filter in `plan` is gone, so every `@ToolProperty` feature is asked for and gets an outcome.
  - **Names:** `api_json::valued_element` now names an unnamed redefinition by following `Redefinition` → `redefinedFeature` recursively (`declared_name`): the gearbox's `:>> volume` redefines Gearbox's own unnamed `:>> volume default`, which used to make it unresolvable and silently dropped.
  - **Model Value of a valueless feature:** `LinkedProperty.redefines` holds the chain of features it redefines, nearest first, and the outcome's `model_value` is the first value along the feature then that chain. So the gearbox's Model Value is Gearbox's `default 200000.0 [millimetre ** 3]` (with the `default` word, as the element's value text has it), and the engine's, with no value anywhere, is `None`.
  - **Fixture:** a new `design_value_shapes/` folder: the engine's `:>> volume { @ToolProperty }` has no value and no default; the gearbox's redefines Gearbox's `default` and a sibling `spare : Gearbox` keeps it (checked by a new `gearboxMass ≤ 0.6 kg` constraint on Gearbox). Three new tests in `viewer-opensysml/tests/design_values.rs`. `docs/opensysml-encoding.md` records the redefinition chain and the delete + `add_member` edit.
  - **Deviation, scope of "body and no value":** only unnamed redefinitions (`:>> volume { … }`), the shapes the ticket and the design name. A named new feature with a body and no value (`attribute bore : LengthValue { @ToolProperty … }`) would get `redefines = ["bore"]` after its own delete, which is wrong; it is untested and would come out as an edit failure diagnostic. Supporting it needs `add_member` with its own name and type.
  - **For ticket 05:** a `Missing` outcome still writes nothing. The marker can reuse `How`: `SetValue` for features with a value, `Redefine` for the rest (the delete + `add_member` path is what makes a def's `default` not show through). The `tool_links/` fixture, loaded without a tool, now records a `Missing` outcome for the gearbox too.
- Review fixes.
  - **Byte columns:** `sysx:sourceColumn` counts UTF-8 bytes (probed), so `write_value` now places a named feature's value by bytes. A part named `'Welle⚙…'` on the feature's line in `named_design_values/` covers it.
