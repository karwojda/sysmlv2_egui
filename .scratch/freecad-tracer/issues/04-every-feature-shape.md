# 04: Every feature shape takes a Design Value

**What to build:** A systems engineer can put `@ToolProperty` on a feature at any stage of their Model and it takes the Design Value: a feature with a value (ticket 03), a feature with a body and no value, and a usage that only redefines a feature inherited from its def (`attribute :>> volume { @ToolProperty … }`), where the def's `default` stays the estimate and the usage takes the design's value. Unnamed redefinitions are named by following `redefines`, recursively, as the backend already does for values. Because `set_value` refuses declarations with a body, such features are replaced in memory (delete, then add a redefining member with the value).

See `.scratch/freecad-tracer/spec.md` (stories 8, 9).

**Blocked by:** 03 (Design Values replace Model Values in load)

**Status:** ready-for-agent

- [ ] A feature with a body and no value takes the Design Value, and its dependent checks use it
- [ ] A `:>>` redefinition of an inherited feature with a def `default` takes the Design Value; the def and other usages keep the `default`
- [ ] The Model's Design Value outcomes name these features by their resolved names
- [ ] Fixture cases for all three shapes, through the real backend with a fake tool
