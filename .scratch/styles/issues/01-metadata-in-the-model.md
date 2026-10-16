# 01: Metadata in the Model

**What to build:** Elements know which metadata defs they are annotated with (`@X`, `#X` or `metadata X`), read from OpenSysML's API JSON `MetadataUsage` items. The Properties panel shows a "Metadata" row per annotation (`@Safety`), which can be followed to the def. See `.scratch/styles/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `Element` gains `metadata`: the annotating metadata defs, in order
- [x] The backend reads all three annotation forms
- [x] The Properties panel lists metadata rows that link to their defs
- [x] A `metadata` `.sysml` fixture and its fake twin pass a contract test

## Comments

- Done. `Element::metadata` is read from the `MetadataUsage` items each element owns in the API JSON, in declaration order, through their `type`. Metadata defs themselves stay Unsupported Elements (`metadata def`). The Properties panel shows `@Name` rows that link to the def. Tests: the `metadata` fixture with `common::metadata()`, a contract test, and a properties test.
