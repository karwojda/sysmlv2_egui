# Where is the Model stored, and does it matter for integration?

Research for `.scratch/engineering-tools/issues/03-model-storage.md`. Checked 2026-10-20. Nothing was installed or run. Claims carry their source; **[unverified]** marks inference or claims no primary source confirmed.

## Short answer

All three options can hold the same SysML v2 Model, but they differ in **what the Model is**:

- **`.sysml` files in git**: the Model is text, and git is the version store. Fully local.
- **API repository** (OMG Systems Modeling API, the pilot, Flexo MMS): the Model is a graph of elements with UUIDs, versioned by the server's own commits and branches. A server is required.
- **SysGit**: commercial, but in shape it is option 1. Text in git, with a GUI and review workflow on top. The Git provider is the store, and no database ships.

For an Engineering Tool integration, the important difference is **element identity**, more than where the bytes live. A Tool Link written as metadata on the element works the same in all three. Writing values back is a text edit plus a commit in options 1 and 3, and an API commit of element versions in option 2. Local-first with `.sysml` in git is a legitimate, standard-sanctioned way to use SysML v2: the textual notation is normative, and SysGit sells exactly this model. Nothing here argues against it. The API matters only if someone later needs to exchange with a tool that speaks only the API.

## Comparison

| | `.sysml` in git | API repository (pilot / Flexo MMS) | SysGit |
|---|---|---|---|
| Single source of truth | The `.sysml` files on a git branch | The server's database (PostgreSQL for the pilot, an RDF quadstore for Flexo) | Your Git provider (GitHub, GitLab, Gitea) |
| Versioning | git commits and branches | API `Commit`, `Branch`, `Tag`, server-side `diff` and `merge` | git, through the provider's API |
| Review | Pull requests on text diffs | No review concept in the API; merge reports conflicts as element ids **[unverified: any review UI]** | Pull requests ("Reviews and Pull Requests") |
| Fully local | Yes | Only by running the server stack yourself (JVM service plus DB, or six containers for Flexo) | Desktop executable, but it still needs a reachable Git provider **[unverified: works against a plain local repo]** |
| Element identity | Qualified names (no ids in text) | UUID `@id` per element | Text, so as for files **[unverified]** |
| Write-back | Edit text, commit | `POST /commits` with changed `DataVersion`s | Edit text (or via its UI or Python library), commit through the provider |
| Licence | – | Pilot EPL-2.0; Flexo Apache-2.0 | Proprietary |

## 1. `.sysml` files in git (today)

- **Source of truth:** the files. SysML v2 and KerML define a normative textual notation, so the text *is* the Model, not an export of it. OMG formally adopted the specs on 30 June 2025 ([SysML-v2-Release README](https://github.com/Systems-Modeling/SysML-v2-Release)).
- **Branching and review:** plain git. Diffs are readable text, and Diagram Files and Layouts sit beside the Model and are committed with it (ADR 0002).
- **Local:** fully. Parsing goes through the OpenSysML sidecar (ADR 0001), which needs no server, only a one-off binary download.
- **Identity:** the text has no element UUIDs. OpenSysML's API-JSON `@id` is derived from the qualified name (`A::MassCheck::@2` becomes `A__MassCheck___402`, see `docs/opensysml-encoding.md`), so a rename changes the id. For the *standard library*, KerML fixed the same problem with name-based (v5) UUIDs computed from an element's qualified name or path ([OMG issue KERML_-36](https://issues.omg.org/issues/KERML_-36), resolved in 1.0b4; the page was unreachable when checked, so this is from the issue summary in search results **[partly unverified]**).
- **Tool Links:** metadata on the element in the text. This option fits best, because the link is reviewed in the same pull request as the element it annotates.
- **Write-back:** a text edit (for example, replacing a value expression), then a commit. The hard part is a precise, formatting-preserving edit to the source span. OpenSysML already reports source lines (ADR 0001). That would mean revisiting ADR 0002 for the values case only.
- **Bridge to the API when needed:** the pilot's Jupyter kernel already publishes text to an API server: `%publish [--project=…] [--branch=…] <NAME>` makes a new commit on the project, `%load` fetches models back, and `%repo` sets the server ([SysMLInteractiveHelp.java](https://github.com/Systems-Modeling/SysML-v2-Pilot-Implementation/blob/master/org.omg.sysml.interactive/src/org/omg/sysml/interactive/SysMLInteractiveHelp.java)). Files can stay the source of truth, with the API as a published mirror.

## 2. SysML v2 API and Services repository

**The standard.** *Systems Modeling API and Services* 1.0, formal, September 2025 ([OMG](https://www.omg.org/spec/SystemsModelingAPI/)). It has a platform-independent model plus a REST/HTTP PSM published as [OpenAPI.json](https://www.omg.org/spec/SystemsModelingAPI/20250201/OpenAPI.json) and OSLC shapes and vocabularies. From the OpenAPI file, the endpoints are:

- `/projects`, `/projects/{id}/branches`, `/tags`, `/commits`
- `/commits/{id}/elements`, `/roots`, `/elements/{id}/relationships`, `/commits/{id}/changes`
- `/commits/{compareCommitId}/diff`
- `POST /branches/{targetBranchId}/merge?sourceCommitId=…`, which returns **409 with the conflicting `DataIdentity` list**
- `/queries` and `/query-results`

A commit (`CommitRequest`) is a list of `DataVersion`s, each with an `identity` (`@id`, a UUID) and a `payload` (the element JSON, or null to delete).

**Implementations.**

- *Pilot* [SysML-v2-API-Services](https://github.com/Systems-Modeling/SysML-v2-API-Services): a "proof-of-concept pilot implementation" built on Java 11, Play and sbt, with PostgreSQL behind it. It runs locally on `localhost:9000`, licence EPL-2.0.
- *OpenMBEE Flexo MMS* is "a version control system for model data" that stores native RDF ([overview](https://flexo-mms-deployment-guide.readthedocs.io/en/latest/overview.html)).
  - The Layer 1 service keeps all state in a SPARQL 1.1 quadstore (Fuseki) ([layer1 README](https://github.com/Open-MBEE/flexo-mms-layer1-service)).
  - Its own model is orgs, repos, branches, locks and collections, plus SPARQL query endpoints scoped to a ref ([OpenMBEE wiki](https://openmbee.atlassian.net/wiki/spaces/OPENMBEE/pages/623378450)).
  - The SysML v2 API is a *separate* microservice on top, [flexo-mms-sysmlv2](https://github.com/Open-MBEE/flexo-mms-sysmlv2) (JDK 21, no releases).
  - The local docker-compose runs six containers: OpenLDAP, Fuseki, MinIO, auth, store and layer1 ([compose file](https://github.com/Open-MBEE/flexo-mms-deployment/blob/develop/docker-compose/docker-compose.yml)), plus the sysmlv2 service.
  - Which API endpoints the adapter supports is not listed **[unverified]**.

**Against the questions:**

- **Source of truth:** the server. Files become an import or export format. Text formatting is not part of the element JSON **[inference]**.
- **Branching and review:** the server has branches and merge with element-level conflicts. Pull-request-style review, comments and approvals are not part of the API, so they would need a separate tool **[inference from the OpenAPI paths]**.
- **Local:** possible but heavy: a JVM service and a database, or six or more containers for Flexo. That conflicts with "no required server" in the map.
- **Tool Links:** still metadata on the element. The element has a stable UUID that survives renames, which makes a stronger link target than a qualified name. The integration would query elements over REST (or SPARQL in Flexo) instead of parsing.
- **Write-back:** the cleanest of the three. `POST /projects/{id}/commits` with one `DataVersion` for the changed feature value, usually on a branch, then `merge`. That is a structured, typed write with no text editing. It does require the server, plus UUIDs for every element, which text does not carry.

## 3. SysGit

From SysGit's own site; vendor claims, not tested.

- "Everything is stored in Git and open standards" and "built on SysMLv2" ([home](https://sysgit.io)).
- "SysGit relies on underlying Git providers as the storage mechanism for data, including GitHub, GitLab, and Gitea." It "does not ship with a database of any kind." "Any creation, modification, or deletion of data is handled through the Git Provider's API." On an air-gapped network, "the Git provider must be on the same airgapped network" ([FAQ](https://www.sysgit.io/frequently-asked-questions)).
- It comes as a desktop "self-contained executable" or an IT-managed k8s package, with "no cloud services required". It has a SysML v2 code editor ("make model changes in code"), "Reviews and Pull Requests", a Python library, import from DOORS, Jama and spreadsheets, and lists "Proprietary APIs" as a drawback of other platforms ([product](https://www.sysgit.io/product)).
- **[Unverified]:**
  - That the files in git are plain `.sysml` textual notation (strongly implied by "textual notation … hardware as code", but no file format is named).
  - Whether it implements the OMG SysML v2 API.
  - Whether it works on a local repo with no provider.
  - Any CAD or EDA integration (none is named).

**Against the questions:** the source of truth, branching and write-back are the same as option 1, done through the Git provider. Tool Links would be metadata in text, as in option 1. SysGit is evidence that a commercial product built on "SysML v2 text in git with PR reviews" is a credible way to use SysML v2, not a hobbyist shortcut.

## What this means for the Engineering Tools design

1. Keep `.sysml` in git as the source of truth. It matches local-first, the standard, and the closest commercial product.
2. Design Tool Links and pulled values around **element identity through qualified names**, behind our own model-loading interface (ADR 0001), so a later move to UUIDs (API `@id`) changes only the backend.
3. Shape a future write-back as a *change set* (element, feature, new value). Applied to files, that becomes a text edit plus a commit. Applied to an API server, it becomes a `Commit` of `DataVersion`s. Either way the design doesn't depend on the store.
4. Treat an API server as an optional publish target (as the pilot's `%publish` does), not as storage.
