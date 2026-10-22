# Where is the Model stored, and does it matter for integration?

**Status:** resolved
**Type:** research
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

Compare three ways of storing a SysML v2 Model: `.sysml` files in git (today), a SysML v2 API and Services repository such as OpenMBEE Flexo MMS, and SysGit's approach. For each: where does the single source of truth live, how do branches and reviews work, can it run fully locally, how would an Engineering Tool integration and Tool Links work against it, and how would writing values back to the Model work?

## Answer

**Storing `.sysml` files in git is a sound way to use SysML v2, and the storage choice matters less for integration than element identity does.**

- **`.sysml` in git:** the textual notation is normative (OMG adopted SysML v2 formally on 2025-06-30), so the files are the Model, not an export. It runs fully locally, and reviews are pull requests on text. Writing values back means editing the text and committing, which would mean revisiting ADR 0002 for that case.
- **SysML v2 API repository:**
  - OMG Systems Modeling API and Services 1.0 is formal (September 2025). It has projects, branches, commits, diff and merge (a conflict returns 409), and queries.
  - A commit is a list of per-element JSON versions keyed by UUID. Writing back is clean, but it needs a server, and the API has no review or pull-request concept.
  - The pilot implementation is a proof of concept on Java and PostgreSQL. Flexo MMS keeps RDF in Fuseki; its SysML v2 layer is a separate service with no releases, and running it locally takes six containers. That conflicts with the map's local-first default.
- **SysGit** (commercial; vendor claims only):
  - It keeps data in the customer's Git provider, with no database, and offers pull-request reviews, a code editor, a Python library, and desktop or Kubernetes installs.
  - In shape it is the same as `.sysml` in git. Not verified: that the files are plain `.sysml`, that it implements the OMG API, that it works without a Git provider, and any CAD or EDA integration.
- **Element identity matters most.** The text carries no UUIDs, and OpenSysML's ids are built from the qualified name, so they change on a rename. API elements have stable UUIDs. A Tool Link is metadata on the element itself, so it moves with a rename; identity matters mainly for a future write back.
- **Bridge between files and a server:** the pilot's Jupyter `%publish` already pushes text to an API server as a new commit, so files can stay the source of truth with a server as an optional mirror.
- **The note recommends** (input to "Where does the integration code run?", not yet decided):
  - Keep `.sysml` files in git.
  - Key Tool Links by qualified name, behind the model-loading interface from ADR 0001.
  - Design a future write as a store-neutral change set (element, feature, value).
  - Treat an API server as a publish target, not as storage.

Findings: branch `research/model-storage` (commit c9f233c, not pushed: no GitHub credentials), `docs/research/model-storage.md`.
