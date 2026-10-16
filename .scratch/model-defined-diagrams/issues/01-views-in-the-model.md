# 01: Views and their facts in the Model

**What to build:** The Model knows about `view` usages and `view def`s, along with what each one exposes and how it renders.

- New element kinds `ViewUsage` (`view`) and `ViewDefinition` (`view def`), from the symbol tree.
- Each element gains `exposes`, an ordered list of `{ target, scope }` where the scope is `Element`, `Members` or `Descendants`. It is read from the API JSON's `MembershipExpose` and `NamespaceExpose`, including `isRecursive`.
- Each element gains `rendering`, the qualified name of the library rendering it references (`Views::asTreeDiagram`).

Views appear in the Browser. The Properties panel shows a view's rendering and one row per expose (`vehicle::**`). See `.scratch/model-defined-diagrams/spec.md`.

**Blocked by:** None (can start immediately)

**Status:** done

- [x] `ElementKind::ViewUsage` and `ElementKind::ViewDefinition` exist, with keywords `view` and `view def`
- [x] `Element` gains `exposes` (target and scope) and `rendering`
- [x] The backend reads exposes (`X`, `X::*`, `X::**`) and renderings on both view usages and view definitions
- [x] The Properties panel lists a view's rendering and its exposes
- [x] A `views` `.sysml` fixture and its fake twin pass a contract test. The fixture has a tree view, an interconnection view, a view inheriting its render from a user view def, a `GeneralView`-typed view without a render, a view exposing `Pkg::*` plus a single element, and a table view.

## Comments

- Done.
  - **Model:** `ElementKind::ViewDefinition` / `ViewUsage`. `Element` gains `exposes: Vec<Expose { target, scope }>` and `rendering`. The scopes are `Element` (`X`), `Members` (`X::*`), `ElementAndDescendants` (`X::**`, a recursive membership expose, which by KerML includes X) and `Descendants` (`X::*::**`). The spec's three scopes became four once the probe showed `X::**` includes X.
  - **Backend:** exposes and renderings are read from the API JSON in declaration order (by generated member index). A membership expose resolves through its membership's `memberElement`. The symbol tree's `renderingUsage` children of views are dropped, since the rendering is a fact of the view.
  - **Properties:** "Renders" (simple name) and one "Exposes" row per expose, which can be followed to the target.
  - **Tests:** `views` fixture and twin (`common::views()`), contract test, and a properties test.
