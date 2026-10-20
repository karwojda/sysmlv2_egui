# Coding standards

The judgement calls a reviewer checks a change against. Formatting and lints are enforced by `scripts/check` and CI, so they are not repeated here.

## Words

- **Domain terms.** Names, doc comments, messages and test names use the terms in `CONTEXT.md` in the sense it gives them, and none of its _Avoid_ words. A Verdict is what an element shows; what one check found is its result or outcome. A change that introduces a new concept adds its term to `CONTEXT.md` in the same change.
- **Service vocabulary stays in its adapters.** `opensysml` types and the service's own words (its verdicts, symbol kinds such as `partDef`, JSON `@type` strings) appear only where the service is read: `lib.rs` (`collect`), `api_json.rs`, `expression.rs` and `checks/service.rs` in `viewer-opensysml`. Everything else speaks the Model's terms (ADR-0001).

## Shape

- **One meaning, one place.** A rule, a word table or a user-visible text lives in one module and is asked for, not rewritten: marks and labels in `verdict`, a service verdict's translation in `checks/service.rs`. Two copies of the same text drift.
- **Deep modules at real seams.** Prefer a small interface with the behaviour behind it over helpers each caller strings together. A new seam needs two adapters (the real one and a test one); otherwise keep the code direct.
- **The Model is read-only** (ADR-0002). Nothing writes `.sysml` files; the user's adjustments go in files under `diagrams/`.

## Doc comments

- Every public and crate-public item has one. It says what the item is or gives, in plain words, with a concrete example where one helps: `` /// `✓ holds`, `✗ violated: x <= 1` ``, `` /// `MassRollup::vehicle::powertrain` ``.
- Comments explain why or what a reader can't see in the code; they match the density of the code around them.

## Tests

- **Through the interface.** Tests cross the seam callers use: the fake backend and its twins in `viewer-core/tests/common` (seam 1), the contract tests against the service (seam 2), the test `Checker` for the backend's checks. A private helper gets its own test only when no interface test can reach the case.
- **Replace, don't layer.** When a module's interface changes, its tests move to the new interface; old tests are not kept alongside wrappers.
- **Twins stay twins.** A fixture under `viewer-opensysml/tests/fixtures` and its fake-backend twin change together, and the contract test comparing them stays green.

## Interfaces users rely on

The CLI's report lines and exit codes, the Style file's vocabulary, the Diagram File and settings formats, and SVG Exports are interfaces. A change to one is deliberate: its tests or golden files change in the same commit, existing files keep loading (or `format_version` is bumped), and the commit message says what users will see.
