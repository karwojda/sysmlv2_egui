# Spec: Styles

Status: ready-for-agent

## Problem Statement

All my Diagrams are black on white. In a large picture I can't tell at a glance which parts are safety-critical, which come from a supplier, or which subsystem they belong to. My documents lose information I've already put in the Model as metadata and package structure. I want colour, but I don't want to give up the standard notation, and I don't want to colour every Diagram by hand.

## Solution

A Style colours node fills by Style Rules, layered on the Standard Notation.

- **Rules.** Each rule colours elements of a kind (`part def`), elements inside a package (`Vehicle::Powertrain`), or elements annotated with a metadata def (`@Safety`). When several rules match, the most specific wins: metadata beats package, which beats kind. Within one level, the deeper package wins, or for metadata the rule listed first.
- **Model Style.** It lives in `diagrams/style.json`, applies to every Diagram, and reloads live when the file changes.
- **Diagram Style.** A Diagram can add rules of its own, which win over the Model's, or switch the Model Style off.
- **Legend.** A Diagram can show a Legend bottom-right that keys each colour it uses.
- **Editing.** A Style panel in the viewer edits both the Model Style and the open Diagram's Style.
- **Export.** CLI Export applies the same Styles, so documents match the viewer.

Strokes, text and connectors stay as the Standard Notation draws them. The one exception is that text inside a dark fill turns white so it stays readable.

## User Stories

1. As a systems engineer, I want to colour all elements of a kind (for example every `part def`), so that kinds stand apart at a glance.
2. As a systems engineer, I want to colour every element inside a package, so that subsystems are visible in mixed Diagrams.
3. As a systems engineer, I want to colour elements annotated with a metadata def (`@Safety`, `#Safety`, `metadata Safety`), so that what I've marked in the Model shows in the picture.
4. As a systems engineer, I want a metadata rule to beat a package rule and a package rule to beat a kind rule, so that the most specific statement about an element decides its colour.
5. As a systems engineer, I want a deeper package's rule to beat an enclosing package's, so that I can colour a subsystem differently inside its system.
6. As a systems engineer, I want the first listed rule to win when an element has two coloured metadata annotations, so that I control the outcome by ordering.
7. As a systems engineer, I want a Model Style that applies to every Diagram of the Model, so that colours are consistent without repeating them.
8. As a systems engineer, I want the Model Style kept in a readable file beside my Diagram Files, so that I can commit it, diff it and share it.
9. As a systems engineer, I want edits to the Style file picked up live, so that I see the result as I change it.
10. As a systems engineer, I want a broken Style file reported, with Diagrams still drawn without it, so that a typo doesn't block me.
11. As a systems engineer, I want a Diagram to add rules of its own that win over the Model's, so that one picture can emphasise something different.
12. As a systems engineer, I want a Diagram to switch the Model Style off, so that I can produce a plain Standard Notation picture.
13. As a systems engineer, I want unsaved Diagrams and views without a Diagram File to use the Model Style, so that everything is coloured consistently.
14. As a systems engineer, I want part boxes and port squares in Interconnection Diagrams coloured by the same rules, so that both Diagram Kinds tell the same story.
15. As a systems engineer, I want text inside a dark fill drawn white, so that labels stay readable whatever colour I pick.
16. As a systems engineer, I want strokes, connectors and markers left in the Standard Notation, so that the Diagrams still read as SysML.
17. As a systems engineer, I want to turn a Legend on for a Diagram, so that readers know what the colours mean.
18. As a systems engineer, I want each Legend entry labelled from its rule (`«part def»`, `Powertrain`, `@Safety`) unless I give it a label of my own ("Safety-critical"), so that the key is meaningful.
19. As a systems engineer, I want the Legend to list only the colours actually used in that Diagram, so that it doesn't show irrelevant entries.
20. As a systems engineer, I want the Legend placed bottom-right, outside the nodes, so that it never covers anything.
21. As a systems engineer, I want a Style panel where I can add, remove and reorder rules and pick colours, for the Model Style and for the open Diagram, so that I don't have to edit JSON.
22. As a systems engineer, I want the panel to offer the Model's element kinds, packages and metadata defs to choose from, so that I can't mistype a name.
23. As a systems engineer, I want a rule naming a package or metadata def no longer in the Model to be kept and marked in the panel, so that renames don't silently delete my rules.
24. As a systems engineer, I want the Properties panel to show an element's metadata, so that I can see why it's coloured.
25. As a documentation maintainer, I want CLI Export to apply the Model Style and each Diagram's Style, Legend included, identical to the viewer, so that documents look the same.
26. As a documentation maintainer, I want styled Export to stay deterministic, so that unchanged inputs give unchanged files.

## Implementation Decisions

### Model

- Elements gain `metadata`: the metadata defs they are annotated with, in order. It is read from the API JSON's `MetadataUsage` items, which are owned by the annotated element and typed by the metadata def. `@X`, `#X` and `metadata X` all count. Values inside the annotation aren't read.
- The Properties panel shows a "Metadata" row per annotation (`@Safety`), which can be followed to the def.

### Style

- A Style is an ordered list of Style Rules. Each rule has exactly one condition (`kind`, `package` or `metadata`), a `fill` colour (`#rrggbb`) and an optional `label`.
- **Matching an element:**
  1. The open Diagram's rules come first, then the Model's (unless the Diagram switched the Model Style off). The first level with a matching rule decides.
  2. Within a level, specificity decides: metadata beats package, which beats kind.
  3. Among package rules, the deepest matching package wins. Among metadata rules, the first listed wins. Kind rules can't tie.
- A kind rule names a keyword the tool draws (`part def`, `part`, `port def`, `port`, `attribute def`, `attribute`, `connection def`, `interface def`, …). A package rule matches any element whose qualified name lies inside the package, at any depth. A metadata rule matches elements whose `metadata` contains the def.
- Rules naming something not in the Model never match, but they are kept.

### Applying a Style

- A Style is applied to a built scene. Builders and layout are unchanged, so a Style never moves anything.
- **What gets coloured:** each node's own rect (General nodes, Interconnection parts, port squares) gets the matched fill. Text drawn inside that rect switches to white when the fill is dark, by relative luminance. Nothing else changes.
- **Scene changes:** fills gain a colour option, and text gains a colour. Both renderers honour them, so SVG and canvas agree.
- **Legend:** when enabled, a Legend box is placed below and to the right of the nodes, and the page grows to hold it. Each row is a swatch and the rule's label, or its condition as text (`«part def»`, `Powertrain`, `@Safety`). Rows appear only for rules that coloured at least one node, in rule order, Diagram rules first.

### Persistence

- **Model Style:** `diagrams/style.json`, pretty-printed with a fixed key order:
  `{ "format_version": 1, "rules": [ { "metadata": "M::Safety", "fill": "#f4cccc", "label": "Safety-critical" }, { "kind": "part def", "fill": "#dbe8f6" } ] }`.
  A missing file means no Model Style. An unreadable one is reported (in the viewer as a diagnostic-style warning, in the CLI as a warning) and ignored.
- **Diagram Style:** an optional `"style"` block in the Diagram File: `{ "model_style": true, "rules": [...], "legend": false }`. Without the block, the Diagram uses the Model Style and shows no Legend. The format version stays 1.
- The Style file takes part in the change fingerprint, so editing it triggers the same live reload as a Model edit.

### Viewer

- The canvas paints styled scenes.
- A "Style" panel offers two tabs, "Model" and "This Diagram".
  - Each tab has a rule list with add, remove, move up and move down. Each rule has a condition picker (kind; package or metadata from the Model), a colour button, and a label field.
  - The Diagram tab adds the "Use Model Style" and "Legend" switches.
  - Editing the Model tab writes `style.json` straight away. Editing the Diagram tab marks the Diagram unsaved, like a Layout edit.
- Rules naming missing elements are shown marked.

### CLI

- `export` applies the Model Style and the Diagram File's Style, including the Legend. Views exported without a Diagram File get the Model Style.

## Testing Decisions

- **Seams as before.** Seam 1: Style matching and styled scenes and SVGs from fake-backend Models, plus golden files. Seam 2: a contract test for metadata against OpenSysML.
- **Seam 1:**
  - **Matching:** each condition; specificity across levels (metadata over package over kind); the deeper package; the first metadata rule; Diagram rules before Model rules; the Model Style switched off; rules naming missing elements.
  - **Scene:** fills on General nodes, Interconnection parts and ports; white text on dark fills only; strokes, connectors and positions unchanged.
  - **Legend:** shows only used rules, custom and automatic labels, placement outside the nodes, and page growth.
  - **Persistence:** Style file round trip and key order; the Diagram File style block round trip; old files still loading; a broken Style file.
  - **CLI:** styled Export equals the styled scene's SVG, a broken Style file gives a warning, and output is deterministic.
  - **Goldens:** a styled General Diagram with a Legend, and a styled Interconnection Diagram.
- **Seam 2:** a `metadata` fixture with `@X`, `#X` and `metadata X` annotations, plus its fake twin.
- The editor panel is checked by hand with screenshots. The model operations behind it (add, remove, move, set fill) are tested as plain functions.
- **Prior art:** `viewer-core/tests/view_diagram.rs`, `diagram_file.rs`, `viewer-cli/tests/export_views.rs`, `viewer-opensysml/tests/contract.rs`.

## Out of Scope

- **PNG Export:** deferred by the user. The plan was to rasterise our own SVG at 2× scale, picking the format from the extension.
- Matching on metadata attribute values (`isMandatory = true`).
- Matching a usage through its definition's metadata or package.
- Rules on connectors, strokes or line styles.
- Simplified domain-specific notations.
- Style libraries shared across Models.
- A movable Legend.

## Further Notes

- Decisions settled with the user on 2026-10-16 (Q1–Q14). Q3 chose specificity over list order, and Q11 has Diagram rules always beat Model rules.
- Backend probe: metadata arrives as `MetadataUsage` items in the API JSON (with `sysx:declaredKeyword` `@` or `#`), owned by the annotated element and typed by the metadata def. The symbol tree doesn't list them.
- The first slice kept element identity in the scene so that Styles could be applied after building. This spec relies on that.
