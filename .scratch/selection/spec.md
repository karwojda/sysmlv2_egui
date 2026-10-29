# Spec: Selection, alignment and undo

Status: ready-for-agent

## Problem Statement

Tidying a Diagram means dragging nodes one at a time and judging by eye whether they line up. I can't move a block of nodes together, make boxes the same size, or line them up exactly. And when an edit goes wrong, the only way back is Reset, which throws away all my other adjustments.

## Solution

- **Selection.** I can pick several nodes of one Diagram, all drawn in the same box, and drag them as one. See `CONTEXT.md`.
- **Arrange commands.** From a selected node's context menu I can align the Selection to that node (six ways), give the others its width, height or size, or distribute the Selection evenly.
- **Undo.** Every Layout edit can be undone and redone, one user action at a time.

All of this changes only the Layout (ADR-0002). Exports and the CLI need no changes.

## User Stories

1. As a systems engineer, I want to Ctrl- or Shift-click nodes to add them to or remove them from a Selection, so that I can pick exactly the nodes I want.
2. As a systems engineer, I want to Shift-drag a box on empty canvas to select the nodes inside it, so that I can pick many nodes at once while a plain drag still pans.
3. As a systems engineer, I want dragging any selected node to move the whole Selection, so that a block of nodes keeps its arrangement.
4. As a systems engineer, I want the whole Selection to stop when one node reaches the top or left limit, so that a move never distorts the arrangement.
5. As a systems engineer, I want manual bends of edges between two selected nodes to move with them, so that their routes aren't left behind.
6. As a systems engineer, I want to align the Selection's left edges, right edges, tops, bottoms, horizontal centres or vertical centres to the node I right-click, so that I choose which node stays put.
7. As a systems engineer, I want to give the Selection the width, height or size of the node I right-click, so that boxes match.
8. As a systems engineer, I want to distribute three or more selected nodes horizontally or vertically with equal gaps, so that rows and columns look even.
9. As a systems engineer, I want to undo and redo any Layout edit, including Reset, so that a wrong click is never costly.
10. As a systems engineer, I want one drag or one command to be one undo step, so that undo matches what I did.

## Implementation Decisions

### Selection

- It belongs to the canvas of one open Diagram: a set of node keys (`Depicts::node_key`: elements in a General Diagram, feature paths in an Interconnection Diagram). It is never saved.
- It is separate from the app's `selected` element, which drives the Browser and Properties. Clicking a node still sets `selected` to that node's element, so Properties follows the node clicked last.
- **Same box.** All nodes in a Selection are drawn in the same box (they have the same parent in `scene.origins`, or none). Adding a node from another box starts a new Selection with only that node. A box drag only picks nodes in one box: the box the drag starts in.
- Ports, edges and the Interconnection Frame can never be selected.
- **Gestures:** a plain click on a node selects only that node. Ctrl- or Shift-click adds the node, or removes it if it's already selected. Shift-drag on empty canvas draws a selection box. A plain drag on empty canvas still pans. A plain click on empty canvas, or Escape, empties the Selection.
- It also empties on Reset and when another Diagram tab becomes active. When the scene is rebuilt (after an edit or a Model reload), keys that are no longer drawn drop out, for example hidden nodes.
- Selected nodes are outlined with the existing highlight.
- Right-clicking a node outside the Selection replaces the Selection with that node.

### Moving

- Dragging a selected node moves every selected node by the same offset. The offset is clamped so that no node goes past zero at the top or left of its page or box; all nodes stop together.
- Manual bends of an edge whose two ends are both selected shift by the same offset. Bends of edges with only one selected end stay put, as for a single-node move today.
- This is a new `Edit` (for example `MoveAll { elements, dx, dy }`), so that clamping and bends happen in one place in `editing.rs`.

### Arrange commands

- They sit in the node context menu, under "Align", "Same size" and "Distribute". The right-clicked node is the **reference**.
- **Align** (Selection of 2+): left, right, top, bottom, horizontal centres, vertical centres. Every other selected node moves on one axis so that the matching edge or centre equals the reference's. The reference doesn't move.
- **Same size** (Selection of 2+): width, height, or both, copied from the reference's drawn size. Positions don't change (the top-left stays). Collapsed nodes are skipped, and a collapsed reference greys these entries out.
- **Distribute** (Selection of 3+): horizontally or vertically. The two outermost nodes on that axis stay put. The others keep their order and are moved so that the gaps between neighbouring boxes are equal. The reference plays no part.
- Results may overlap, and nothing is nudged apart. Positions are still clamped at zero like any move.
- Each command is a single `Edit` that sets the placements of all affected nodes.

### Undo

- Each open Diagram keeps an undo and a redo stack of Layouts. `OpenDiagram::edit` (`canvas.rs`) is where every Layout change already goes through, so the history lives there.
- **One step per user action.** A drag (move, resize, bend, port), from press to release, is one step, even though it applies an `Edit` every frame. A context-menu command, a Hide/Show from the top bar, or Reset is one step. Edits that change nothing record no step.
- Every Layout edit can be undone, including Hide/Show/Show all, Collapse/Expand, routes, ports, Open/Close and Reset.
- Shortcuts: Ctrl+Z undoes. Ctrl+Shift+Z and Ctrl+Y redo. A new edit clears the redo stack. Undo and redo mark the Diagram unsaved, unless it is back at the saved Layout.
- The history survives saving. It is dropped when the Diagram is closed, or when its Layout is replaced from outside (for example by reopening its Diagram File).

## Testing Decisions

- `editing.rs` gets unit tests for the new edits: a group move with clamping and bends; each align, same-size and distribute variant; skipping collapsed nodes; nodes inside a box in an Interconnection Diagram.
- Undo/redo is tested on `OpenDiagram` (step grouping, redo cleared by a new edit, Reset undone, the unsaved flag).
- Selection rules that don't need egui (same box, dropping keys that aren't drawn, what a box drag picks) are pure functions in `canvas.rs` with unit tests, like `grab_at`.

## Out of Scope

- Lasting groups saved in the Layout.
- Moving the Selection with arrow keys, and hiding it with Delete.
- Avoiding overlaps after arranging.
- Arranging across boxes.
