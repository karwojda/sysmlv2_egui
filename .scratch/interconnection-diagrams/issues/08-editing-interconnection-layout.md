# 08: Editing an Interconnection Diagram's Layout

**What to build:** The existing gestures work in Interconnection Diagrams: move, resize, hide, bend connectors, Reset. Layout keys are feature paths relative to the root. Moving a part keeps it inside its enclosing box and moves its insides with it. Ports can be dragged along their part's border, saved as `Layout::ports`. A part's context menu opens or closes it in place.

**Blocked by:** 04: Opened Parts; 07: Opening Interconnection Diagrams in the viewer

**Status:** needs-triage

- [ ] Positions in an Interconnection Layout are relative to the enclosing box's content and keyed by feature path
- [ ] Moving an Opened Part moves its contents; a moved part can't leave its enclosing box
- [ ] Dragging a port moves it along the border and can move it round a corner to another side; saved in `Layout::ports` and honoured by the Export
- [ ] The part context menu offers Open / Close, which toggles `Layout::opened`
- [ ] Seam 1 tests for each edit (scene and Diagram File); canvas gestures checked by the user
