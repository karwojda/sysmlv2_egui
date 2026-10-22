# How do existing tools link SysML to CAD and EDA?

**Status:** resolved
**Type:** research
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

Prior art for linking SysML (v1 and v2) models to CAD/EDA tools and pulling values back: SysML v2 API, OSLC, Syndeia-style digital-thread tools, vendor integrations (e.g. Cameo, Capella), and SysML v2 metadata or library conventions for referencing external artifacts. What do they link (files, objects, properties), how do they handle units, staleness and missing tools, and what has proven to work or not?

## Answer

**No standard Tool Link exists, but the SysML v2 standard library has a close precedent, and the integrations that work are small one-way pulls.** The note cites 25 sources and marks anything it couldn't check as unverified.

- **Closest standard precedent: the standard library's `AnalysisTooling` package.** `ToolExecution { toolName; uri }` goes on an action, and `@ToolVariable { name }` goes on each parameter (the official example uses ModelCenter). It is metadata stored in the Model, which is the same shape as our Tool Link. The tool's unit isn't recorded; it is only implied by the SysML type.
- **Systems Modeling API 1.0 has `ExternalRelationship`:** an element linked to an IRI, with a free-form mapping. It is stored on the server, not in the text, and is under-specified: an OMG issue is open on what it means, and SysON doesn't support it.
  - The two research notes disagree on this spec's adoption date: this one says March 2026, the Model storage one September 2025.
- **Syndeia** (commercial digital thread): links a SysML block to a CAD part or assembly, element to element. It pulls a fixed set of mass properties, compares before syncing, and syncs from CAD to SysML only. In local mode the links are stored in the Model, which matches our design.
- **Staleness precedents:** Cameo "suspect links" store a fingerprint and version on the link (partly unverified); Syndeia compares explicitly before syncing; OSLC Configuration Management versions resources but needs a server.
- **OSLC** has no CAD or EDA domain, and OSLC SysML 2.0 is only a draft. Its link guidance stores a link once, on the downstream artifact, which is the opposite of our Model-side Tool Link.
- **Units:** no link mechanism found carries units. The one open FreeCAD + SysML v2 prototype (Fraunhofer IPK, 2023) drops the unit and, by the note's reading, gets its box 10× too small.
- **Missing tools:** no source addresses what happens when the tool isn't installed. We will have to design it.
- **Open source:** no maintained SysML↔FreeCAD or SysML↔KiCad integration exists.
- **Proven:** one-way pulls of a few physical properties, links to named objects inside a file, comparing before accepting, and fingerprints on the link. **Not proven:** generic link semantics, two-way CAD sync, and units at the tool boundary.

Findings: branch `research/prior-art` (commit c31582a, not pushed: no GitHub credentials), `.scratch/engineering-tools/research/04-prior-art.md`.
