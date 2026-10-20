# Prior art: linking SysML to CAD/EDA and pulling values back

Research for ticket `04-prior-art` (map: Engineering Tools). Gathered 2026-10-20.
Sources are primary (specs, vendor docs, standard library source, papers) unless marked otherwise.
**[unverified]** marks claims I could not check against a primary source.

## Short answer

- Nobody standardises a "Tool Link" yet. The closest standard pieces are:
  - **SysML v2 standard library `AnalysisTooling`**: `ToolExecution { toolName; uri }` on an action, plus `ToolVariable { name }` on each parameter. This is metadata in the Model, which is the same shape as our Tool Link. [S1, S2]
  - **Systems Modeling API 1.0 `ExternalRelationship`**: an element linked to an IRI, with an optional mapping `specification` written in a named `language`. It is stored by the API server beside the Model, not in the text. [S3]
- Commercial digital-thread tools such as Syndeia link **elements to elements**: a SysML block to a CAD part or assembly. They do not link to files. In practice they pull **a fixed set of properties** (mass properties) **one way, from CAD to SysML**, on demand, through compare-then-sync. [S5, S6]
- **Staleness** is handled in two ways. One is on-demand compare (Syndeia). The other is fingerprints stored on the link (Cameo "suspect links"). OSLC delegates it to configuration management (concept vs version resources). [S6, S9, S11]
- **Units:** the SysML v2 side has full ISQ/SI typing. None of the link mechanisms carries the *tool's* unit: `ToolVariable` has only a name, and `ExternalRelationship` leaves the mapping to a free-form expression. Cameo converts units only between compatible QUDV units. The one FreeCAD example we found drops the unit entirely. [S2, S3, S10, S12]
- **Missing tools:** none of the sources says what happens when the tool is not installed. This is a gap, not a solved problem.
- **Open source:** no maintained open-source SysML↔FreeCAD or SysML↔KiCad integration was found. There is one paper prototype that drives FreeCAD from the SysML v2 API (Fraunhofer IPK, 2023) [S12], and FreeCAD appears as a box in a Galois/OpenMBEE slide [S14].

## 1. SysML v2 language and library conventions

### `AnalysisTooling` (standard library, Domain Libraries/Analysis)

From the SysML-v2-Release repo at commit `fb97b75` (2026-08 release) [S1]:

```sysml
metadata def ToolExecution { attribute toolName : String; attribute uri : String; }
metadata def ToolVariable  { attribute name : String; }
```

- `ToolExecution` "identifies an external analysis tool to be used to implement the annotated action". `ToolVariable` annotates "a parameter or other feature of the action with the name of the variable in the tool". [S1]
- Official example `AnalysisAnnotation.sysml` [S2]: an `action def ComputeDynamics` is annotated with `ToolExecution { toolName = "ModelCenter"; uri = "aserv://localhost/Vehicle/Equation1"; }`, and its `in`/`out` parameters are typed `MassValue`, `SpeedValue` and so on, each with `@ToolVariable { name = "mass"; }`.
- What this tells us:
  - The link target is a **tool name plus a URI string with a tool-specific scheme**. There is no structured file/object/property split.
  - It annotates **actions and their parameters**, not parts. It is a "run this computation" link rather than a "this part is designed there" link.
  - The **unit is implied by the SysML type** (`MassValue`). The tool's own unit is not recorded.
- It is the only standard-library metadata aimed at external tools. The other Metadata libraries are `ModelingMetadata` (StatusInfo, Rationale, Issue, Refinement), `ImageMetadata`, `ParametersOfInterestMetadata` and `RiskMetadata` [S1]. `ImageMetadata::Image.location` is "A URI for the location of a resource containing the image content". That is the only other library precedent for pointing at an external file by URI string [S4].

### Geometry library

`Domain Libraries/Geometry/ShapeItems.sysml` and `SpatialItems.sysml` exist [S1]. The Fraunhofer paper uses `ShapeItems::Box` with `length/width/height` in `[cm]` as the "CAD domain library for simple geometry" [S12].

## 2. Systems Modeling API and Services 1.0 (OMG, formal, document dated March 2026)

- §7.1.3 [S3]:
  - **ExternalRelationship** "represents the relationship between a KerML Element … available from a service provider to ExternalData available over the web using an IRI". "A hyperlink between a KerML Element to a web resource is the most primitive example."
  - Attributes:
    - `specification : String [0..1]` is "the formal representation of the semantics". It "can be a collection of mathematical expressions … to map the attributes of a KerML Element to the attributes of an ExternalData".
    - `language : String [0..1]`.
  - **ExternalData** has `resourceIdentifier : IRI`. It "is defined only for the purpose of defining an ExternalRelationship".
- The service (§7.2.5) offers `getExternalRelationships`, `getExternalRelationshipsByElement`, `getExternalRelationshipById`, `createExternalRelationship` and `deleteExternalRelationship`. All of them are versioned by project/commit/branch like other data. [S3]
- The OSLC PSM (§8.2.4) says "OSLC does/can not differentiate between properties/relationships/elements that are external or otherwise". It lists create/delete of external relationships as "Not available". [S3]
- How the spec handles each concern:
  - **Units:** not addressed. They would live inside the free-form `specification`.
  - **Staleness:** nothing beyond commit versioning of the link itself.
  - **Missing tools:** nothing.
- An open OMG issue (SYSMOAS11-27) asks for an example `specification` and for the meaning of a null one [S15, seen via search; issues.omg.org refused connection, **[unverified text]**]. In other words, the feature is under-specified.
- Implementation status: SysON documents `resourceIdentifier` as "not supported" [S16, via search snippet, **[unverified]**].

## 3. OSLC (OASIS)

- **OSLC SysML 2.0** (Project Specification Draft 01, 25 April 2024) [S7] defines "a RESTful web services interface for the SysML v2 resources and relationships between those and related resources such as product change requests, activities, tasks, requirements or test cases". It builds on OSLC AM link types and Resource Preview. It has no CAD/EDA domain.
- **Resource Preview** (OSLC Core 3.0, OASIS Standard, Aug 2021) [S8] gives a "Compact" representation of any linked resource: `dcterms:title`, `oslc:shortTitle`, `oslc:icon`, and small/large preview documents shown in iframes. This is the standard answer to "show something useful about the far end of a link" without understanding its format.
- **Link Guidance** (Project Note) [S9]:
  - "link information should be captured in one place to avoid information redundancy".
  - Store the link "in the resource that is developed later (the 'downstream' artifact)".
  - Inverse links are discouraged.
  - Our choice (the Tool Link lives in the Model, the CAD file knows nothing) puts the link on the *upstream* side. The guidance would say the opposite. This is worth a sentence in the ADR, but local-first viewing makes the Model side the only practical one.
- **Configuration Management 1.0** (OASIS Standard, 2023) [S11] separates *concept resources* from *version resources* and resolves links through (global) configurations. This is the OSLC answer to staleness: a link names a concept, and a configuration picks which version you see. It is heavy machinery that assumes servers.

## 4. Commercial digital-thread tools

### Syndeia (Intercax; now listed with Dassault on the OMG SysML v2 tool page)

- **What it links:**
  - Elements to elements. NX `.prt` and Creo `.asm`/`.prt` files are "expanded" to show part/assembly structure, and connections go from a SysML block to an NX/Creo part or assembly [S5].
  - Two kinds [S6]:
    - **Reference connection**: "the most basic type … establishes a traceability link".
    - **Model transform connection**: "map the structure of a SysML model element with that of a repository element such that Syndeia can compare and bi-directionally sync the structure on-demand".
- **What it pulls:**
  - Generating SysML from NX/Creo brings in **mass, volume, surface area, density, center of gravity and bounding box**, plus optional NX feature expressions/attributes and Creo model attributes [S5].
  - **Compare covers mass properties only**.
  - **Sync is CAD → SysML only** ("Sync mass property values from NX -> SysML model") [S5].
  - The search summary of the NX docs gives the reason: system engineers rarely edit CAD [S5; reason from search snippet, **[unverified wording]**].
- **Where links live:**
  - In local mode, "connections between SysML artifacts and non-SysML artifacts" are **stored in the SysML model**, and repository details sit in a per-user `~/.syndeia` folder.
  - Cloud mode (recommended) stores everything on Syndeia Cloud [S17].
  - This matches our split: Tool Link in the Model, and machine-specific locations kept out of it.
- **Staleness:** handled by explicit **compare** (a diff view) before sync. The reference link "can be used to detect changes in a Teamwork Cloud versioned element" [S6]. Syndeia "is aware of the versions of connected models" in PLM/ALM [S5-intro].
- **Units / missing tool:** the pages I read do not cover them.

### Cameo / CATIA Magic (Dassault)

- **Hyperlinks** on elements (Address / File / Element tabs) are stored as tagged values [S18]. This is the simplest SysML v1 practice: a file link with no semantics.
- **Suspect links**: tracking applies a `«track»` stereotype whose **fingerprint** tag records the state of the tracked end, plus a **version** tag. A change makes the relationship "suspect". It applies by default to Satisfy/Verify/Refine/deriveReq relationships and can be configured to other dependency types and properties via `«SuspectLinkConfiguration»` [S10b; fingerprint detail from search summary of 2021x/2024x docs, **[partly unverified]**]. This is a proven, model-stored staleness pattern: store a hash of what you last saw on the link.
- **Units:** Cameo Simulation Toolkit converts across binding connectors "only if both sides share compatible units of the same quantity and kind" per QUDV. On failure, values are mirrored with Auto Convert Units off [S10, via search summary]. A third-party issue log (Webel) reports that the 2026x SysML v2 evaluation plugin mishandles mixed-unit arithmetic [S19, **third-party**].
- **CAD:** 3DS marketing describes SysML v2 REST API integration and a "SysML v2 Geometry Viewer – Spatial Requirements From System Model to CAD" demo [S20]. No technical docs were found, **[unverified]**.

### Capella (Eclipse)

- Ships a "Basic Mass" viewpoint: mass boxes with current/max values on physical diagrams [S21].
- The Eclipse project states that developing "bridges towards other specialty engineering tools" is out of scope [S21]. No open-source Capella↔CAD bridge was found. The open-source `capellambse` gives Python read/write access to Capella models [S22].

## 5. Research and open-source prototypes

- **Fraunhofer IPK / CONTACT, "Praktische Anwendung der SysML v2 API am Beispiel von MCAD und Simulation" (Tag des Systems Engineering 2023)** [S12]:
  - The paper notes that MCAD integration "so far mostly through export/import or custom SysML profiles and translation tools".
  - A **FreeCAD Python macro** calls the SysML v2 REST API, walks the element tree and collects `{length: 5, width: 15, height: 5}`, then creates a `Part::Box`.
  - A second script reads `parMaxRPM`, computes torque and **commits a value back** via the API.
  - Observation (my reading of the listing, not stated in the paper): the walker keeps only `LiteralInteger` values and discards the `[cm]` unit. FreeCAD lengths are in mm, so the box comes out 10× too small. This is the units trap in its plainest form.
- **Galois INDIGO (ADEPT 2024 slides)** [S14]: an architecture diagram lists FreeCAD, Cameo (cameo2owl), STEP2OWL, AADL2OWL and DOORS NG around OpenMBEE Flexo MMS. No detail on what is extracted from FreeCAD, **[unverified beyond the label]**.
- **OpenMBEE Flexo** implements the SysML v2 API over an RDF graph store (SPARQL/GraphQL) and is open source [S23]. It is a server, relevant only to the "server-backed options" note on the map.
- **Schumacher & Inkermann (TU Clausthal, DFX 2021)**: a concept for linking SysML diagram elements with CAD models in a "heterogeneous model" [S24]. Full text is behind a login, **[unverified content]**.
- **KiCad:** no SysML integration found. Relevant tool facts [S25]:
  - KiCad 9/10 IPC API: "only supports communication with a running instance of the KiCad GUI", PCB editor only, and "no support for plotting or exporting".
  - KiCad 11 adds schematic plugins and "headless mode through `kicad-cli`".
  - The SWIG Python bindings are deprecated as of 9.0.
  - So for a local-first, CI-friendly pull, `kicad-cli` (or parsing KiCad's text files) is the realistic path on current releases.
- **FreeCAD:** the wiki was unreachable (bot wall), so tool facts are **[unverified]**:
  - Objects carry an internal `Name` (unique per document, fixed) and a user-editable `Label`. A Tool Link should target `Name`.
  - FreeCAD can run headless (`FreeCADCmd`).
  - Internal units are mm/kg/s.

## 6. What has worked, and what hasn't

Worked:

- Pulling a **small fixed set of physical properties** (mass properties, bounding box) **one way from CAD** into the system model. This is the most consistent shipping feature across vendors (Syndeia NX/Creo). [S5]
- **Element-level links** to named objects inside a file, rather than whole-file links. [S5, S6]
- **Explicit compare before sync**, so the engineer sees the difference and accepts it. [S5, S6]
- **Fingerprint on the link** to flag suspect relationships without a server. [S10b]

Did not work, or is still open:

- Generic standard link objects have **under-specified semantics**: the meaning of `ExternalRelationship.specification` is an open issue, and SysON doesn't implement `resourceIdentifier`. [S3, S15, S16]
- **Bidirectional CAD sync** is scoped down even by vendors (CAD → SysML only for NX/Creo). [S5]
- **Units at the boundary** are left to each integration. Every mechanism found carries only a name, a URI or a free expression, and drops are easy to make. [S2, S3, S12]
- **Tool absence**: no source describes it. Expect to design it ourselves (the map already leans to an Undecided Verdict with a reason).
- **Server-centred threads** (OSLC CM, Syndeia Cloud, Flexo) solve versioning but conflict with local-first. Syndeia's own local mode falls back to "link in the model, locations per user". [S11, S17]

## Implications for this project (suggestions, not decisions)

- Shape the Tool Link like `ToolExecution`/`ToolVariable`: a tool name, a location (file path relative to the Model), an object id inside the file, and per-attribute property names. Consider whether to reuse `AnalysisTooling` directly for computation-style links and define our own metadata for "part is designed in" links.
- Put an explicit **unit on the tool side** of each mapped property (or a fixed per-tool unit table), and check dimensional compatibility against the SysML attribute type. That closes the gap every precedent leaves open.
- Detect staleness by fingerprint (file hash or mtime plus object id), not by a server.
- Treat a missing tool or file as a first-class result with a reason.

## Sources

- [S1] SysML-v2-Release, `sysml.library/Domain Libraries/Analysis/AnalysisTooling.sysml` and the Metadata/Geometry directory listings, commit fb97b754 (2026-09-11, "Updated for 2026-08"). https://github.com/Systems-Modeling/SysML-v2-Release/blob/fb97b754f29588b8e9c7a35f370880cd15eb29e7/sysml.library/Domain%20Libraries/Analysis/AnalysisTooling.sysml
- [S2] Same repo, `sysml/src/examples/Analysis Examples/AnalysisAnnotation.sysml`. https://github.com/Systems-Modeling/SysML-v2-Release/blob/fb97b754f29588b8e9c7a35f370880cd15eb29e7/sysml/src/examples/Analysis%20Examples/AnalysisAnnotation.sysml
- [S3] OMG, *Systems Modeling API and Services* v1.0 (dated March 2026), §7.1.3, §7.2.5, §8.2.4. https://www.omg.org/spec/SystemsModelingAPI/1.0/PDF
- [S4] SysML-v2-Release, `Domain Libraries/Metadata/ImageMetadata.sysml` (same commit).
- [S5] Intercax, "Syndeia and Local Models (NX, Creo, Simulink, Stateflow, Excel…)", Syndeia 3.4 docs. https://intercax.atlassian.net/wiki/spaces/SYN34/pages/2407361559 ; Syndeia Introduction https://intercax.atlassian.net/wiki/spaces/SYN34/pages/2407361078/Syndeia+Introduction
- [S6] Intercax, Connection Manager (connection types) https://intercax.atlassian.net/wiki/spaces/SYN34/pages/2409178608 ; Exercise 20.1 Teamwork Cloud https://intercax.atlassian.net/wiki/spaces/SYN36/pages/3330187219 (quotes via search snippets)
- [S7] OASIS OSLC OP, *OSLC Systems Modeling Language Version 2.0 Part 1*, PSD01, 25 April 2024. https://docs.oasis-open-projects.org/oslc-op/sysml/v2.0/psd01/sysml-spec.html
- [S8] OASIS, *OSLC Core 3.0 Part 3: Resource Preview*, OASIS Standard, 26 Aug 2021. https://docs.oasis-open-projects.org/oslc-op/core/v3.0/resource-preview.html
- [S9] OASIS, *OSLC Link Guidance 1.0*, Project Note 01, §2.1.3. https://docs.oasis-open-projects.org/oslc-op/link-guidance/v1.0/pn01/link-guidance.html
- [S10] Dassault/No Magic, Cameo Simulation Toolkit 2024x, unit conversion. https://docs.nomagic.com/x/2gGfC (via search summary)
- [S10b] Dassault/No Magic, "Customizing suspect links tracking" (2026x / latest). https://docs.nomagic.com/MT/latest/magic-software-architect---magicdraw/customizing-suspect-links-tracking-303997927.html ; suspect links overview (2021x) https://docs.nomagic.com/x/-LscBQ
- [S11] OASIS, *OSLC Configuration Management 1.0*, OASIS Standard 2023. https://open-services.net/spec/config/latest
- [S12] M. Manoury, C. Muggeo, "Praktische Anwendung der SysML v2 API am Beispiel von MCAD und Simulation", Tag des Systems Engineering 2023, Fraunhofer IPK / CONTACT. https://publica-rest.fraunhofer.de/server/api/core/bitstreams/4941281c-cfde-451e-9a32-9844745419dc/content
- [S14] S. Vestal, E. Sandberg (Galois), INDIGO slides, ADEPT 2024. https://adept.univ-brest.fr/2024/doc/Vestal_INDIGO.pdf
- [S15] OMG issue SYSMOAS11-27. https://issues.omg.org/issues/SYSMOAS11-27 (not reachable; content via search)
- [S16] SysON docs, APIs. https://doc.mbse-syson.org/syson/v2024.11.0/user-manual/integration/api.html (via search snippet)
- [S17] Intercax, "Configure Syndeia mode", Syndeia 3.6. https://intercax.atlassian.net/wiki/spaces/SYN36/pages/3330093334 (via search summary)
- [S18] No Magic, MagicDraw hyperlinks / `HyperlinkUtils` Javadoc. https://jdocs.nomagic.com/2022x_Refresh1/com/nomagic/magicdraw/hyperlinks/HyperlinkUtils.html ; https://docs.nomagic.com/x/KzmKDQ
- [S19] Webel, Cameo SysML v2 issue log (third party). https://webel.com.au/view/csm/sysmlv2/issue
- [S20] Dassault Systèmes, CATIA SysML v2. https://www.3ds.com/products/catia/catia-magic/sysmlv2
- [S21] Eclipse Capella, Extensions Installation Guide and project governance. https://wiki.eclipse.org/Capella/Extensions/Installation_Guide ; https://projects.eclipse.org/projects/polarsys.capella/governance
- [S22] capellambse (DB Netz), PyPI. https://pypi.org/project/capellambse/
- [S23] OpenMBEE, Flexo SysMLv2. https://openmbee.atlassian.net/wiki/spaces/OPENMBEE/pages/1181188098/Flexo+SysMLv2
- [S24] Schumacher, Inkermann, "Heterogene Modellierung – Verknüpfung und Integration von Systemmodellen der SysML mit CAD-Modellen", DFX 2021. https://www.designsociety.org/publication/43580/
- [S25] KiCad dev docs, IPC API "For Add-on Developers" and "PCB Python Bindings". https://dev-docs.kicad.org/en/apis-and-binding/ipc-api/for-addon-developers/ ; https://dev-docs.kicad.org/en/apis-and-binding/pcbnew/
