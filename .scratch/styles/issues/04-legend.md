# 04: Legend

**What to build:** A Diagram can show a Legend. It is switched on per Diagram from a toolbar toggle and stored in the Diagram File's style block. It sits bottom-right outside the nodes, and the page grows to hold it. It lists only the rules that coloured something, Diagram rules first, each with a swatch and its custom label or an automatic one (`«part def»`, `Powertrain`, `@Safety`). The viewer and Export draw it identically. See `.scratch/styles/spec.md`.

**Blocked by:** 03

**Status:** done

- [x] The Legend lists used rules only, with custom or automatic labels
- [x] It sits outside the nodes and the page grows to hold it
- [x] The toolbar toggle marks the Diagram unsaved, and the setting is saved in its style block
- [x] Golden of a styled Diagram with a Legend

## Comments

- Done.
  - **Drawing:** `style::legend(scene, rules)` draws a box of swatch-and-label rows, tagged `Depicts::Legend`, so it is never a node or an edge. It sits below everything else, right-aligned within the page margin, and the page grows to hold it, leftwards too if the Legend is wider.
  - **Labels:** `style::legend_label` gives the rule's label, else `«kind»`, the package's simple name, or `@Metadata`.
  - **Rules listed:** `apply_styles` draws the Legend when `DiagramStyle::legend` is set, listing the rules used in rule order, Diagram rules first. With no colours used, there's no Legend.
  - **Viewer:** a "Legend" checkbox in the toolbar is an unsaved change, saved in the style block.
  - **Tests:** core `style.rs` (Legend tests and the golden `styled-legend.svg`) and CLI `export_styled.rs`.
