# 05: Right-angled routing around parts

**What to build:** Interconnection connectors never pass through a box they don't connect. Right-angled routes go around obstacles. Undirected ports don't produce the hooks seen in the spike, and connectors leaving the same side spread apart. This settles the spike's rough spots.

**Blocked by:** 01: Interconnection tracer; 03: Frame ports and part-level connection ends

**Status:** done

- [x] No connector segment passes through a box other than its two ends' boxes (and the boxes enclosing them), checked over all golden fixtures, closed and opened
- [x] Connectors leave and enter ports perpendicular to the border
- [x] The drivetrain's `supply` connector (from the Frame's `power` port) routes around the fuel tank
- [ ] Saved bends still take precedence over automatic routing (moved to ticket 08: Interconnection Layouts aren't editable yet, so there are no saved bends to honour)

## Comments

- Done, except the saved-bends criterion, moved to 08.
- Routing (`interconnection::route`): a cheapest right-angled path between the two ports' stubs, over a sparse grid of lines through both stubs and just outside every obstacle's edges, with a cost per bend. It's used only when the simple route (straight, or one elbow across the gap) isn't clear.
- Obstacles (`obstacles`): inside a box, every other box inflated by 10, plus the title band of every open box (the owning box's own band included). The boxes along each end's path are passable, so a connector can reach a port deep inside an Opened Part.
- Undirected ports lean to the top or bottom edge unless their peer is mostly level with them, which removes the spike's `engineStatus` hook.
- Ticket 04 already moved own top ports off titles and made inward connectors clear the title band.
- Fixed while verifying: the "simple route is clear" check skipped straight connectors, so `supply` still ran through the fuel tank. Also, open boxes' title bands had no clearance, so routes hugged their borders.
- Test `connectors_are_right_angled_and_never_pass_through_a_box_they_dont_connect` covers `Vehicle`, the drivetrain, and the drivetrain opened.
