# 01: Requirement nodes

**What to build:** Requirements stop being Unsupported Elements.

- **Model:** requirement definitions (`requirement def`) and usages (`requirement`) carry their short id, subject and `doc`.
- **Notation:** General Diagrams draw them in the standard notation:
  - header `«requirement»` / `«requirement def»`, then `<id> name:Type`, then `subject v:Vehicle`;
  - `doc` and `attributes` compartments, with the subject feature not repeated.
- **Trees:** a requirement usage can be the root of a General Diagram. Nested requirement usages are listed in a `requirements` compartment (no status marks yet).

See `.scratch/requirements-satisfaction/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `ElementKind::RequirementDefinition` / `RequirementUsage`, plus `short_name` and `subject` facts read by the backend
- [x] Requirement nodes drawn with header, subject line, `doc` and attributes compartments
- [x] Requirement usages accepted as General Diagram roots, with the nested `requirements` compartment
- [x] Properties shows "Short id" and "Subject"
- [x] A `requirements` `.sysml` fixture and its fake twin pass a contract test

## Comments

- Done.
  - **Model:** `ElementKind::RequirementDefinition` (counted as a definition) and `RequirementUsage`. Elements gain `short_name` (any element with `declaredShortName`) and `subject` (from the requirement's `SubjectMembership`).
  - **Subject typing:** the symbol tree leaves a subject feature untyped, so its type is taken from the API JSON item, for subjects only.
  - **Notation:**
    - Requirement nodes have a `subject v:Vehicle` line under the title.
    - A wrapped (40-character) `doc` compartment leads the compartments, for requirement kinds only.
    - A new `requirements` compartment lists nested usages, and `<id>` prefixes appear on titles and rows.
    - The subject is excluded from compartments and from composition edges.
  - **Roots:** a requirement usage is accepted as a General root. It is drawn as the top node with its owned requirements hanging below by composition, as in a parts tree.
  - **Properties:** "Short id" and "Subject" rows. The Style editor's kind list gains `requirement def` and `requirement`.
  - **Tests updated:** the `vehicles` contract twin now reads `TopSpeed` as a requirement def. Core tests that only needed *some* Unsupported Element now use `use case def`.
  - **Tests added:** `viewer-core/tests/requirements.rs` (9 tests), the golden `requirements-tree.svg`, and the `requirements` fixture with its twin and a contract test. The fixture already holds the claims for tickets 03–04.
