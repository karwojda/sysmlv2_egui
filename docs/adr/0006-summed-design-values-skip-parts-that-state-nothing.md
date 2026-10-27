# A Summed Design Value leaves out parts that state nothing

When an Engineering Tool adds a value up over the parts of a design (the power of every placed component on a KiCad board), a part with no such field is left out of the sum and listed as uncounted; the Design Values tab and CLI show "summed 12 of 94 parts" with the uncounted parts named. A part whose field is present but unreadable, or whose unit doesn't match the others, leaves the sum with no Design Value, so the dependent checks are Undecided as ADR 0003 says. This bends ADR 0003's "no fallback" for sums only: most parts (resistors, capacitors) draw no power worth counting, and making engineers write `0W` on every passive would make them fill fields mechanically, which hides the same mistakes. We considered treating any missing field as no Design Value, which matches ADR 0003 exactly, but a power budget would stay Undecided on every real board; we considered treating missing as zero silently, but then a forgotten MCU undercounts and nobody sees it.

## Consequences

- A Summed Design Value carries, beside its value, the parts counted and the parts left out; the viewer and CLI show both.
- Parts marked do-not-populate or excluded from the BOM are not parts of the sum at all, counted or uncounted.
- A single linked value (one component's field, the board's thickness) still follows ADR 0003 unchanged.
