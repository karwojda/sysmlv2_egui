# Spec: Design Values from FreeCAD (tracer build)

Status: ready-for-agent

## Problem Statement

My Model says the engine has a volume of 0.9e6 mm³, and from that my requirements and asserted constraints say the drive is within its mass budget. The real engine is designed in FreeCAD, and its body there is 1.0e6 mm³. The viewer judges my requirements only against the numbers I typed into the Model, so a requirement shows Satisfied while the design actually violates it. To see the truth I have to read values out of FreeCAD by hand and edit the `.sysml` files, and CI can't catch a design that has outgrown its budget.

## Solution

I write a **Tool Link** in the Model that says where an element is designed: `@ToolLink { tool = "FreeCAD"; file = "cad/engine.FCStd"; object = "Engine"; }` on the usage, and `@ToolProperty { name = "Volume"; }` on each feature that should come from there. On every load, the viewer and the CLI ask FreeCAD for those values and use them in every check as **Design Values**, in place of the **Model Values** I wrote. My `.sysml` files are never changed. I see each Design Value with its Model Value beside it. If FreeCAD, the file or the object can't be read, the checks that depend on it are Undecided with the reason, and `check` fails. They never quietly fall back to my old estimate.

This is the tracer build of the design in `docs/engineering-tools.md` (ADRs 0003, 0004 and 0005): one thin slice through every seam, with FreeCAD and the `Volume` property.

## User Stories

1. As a systems engineer, I want to copy a small `EngineeringTools` library file into my Model, so that `@ToolLink` and `@ToolProperty` resolve in this tool and in any other SysML v2 tool.
2. As a systems engineer, I want to get that library file from the tool itself, so that I always use the version this tool understands.
3. As a systems engineer, I want to annotate a part usage with `@ToolLink { tool; file; object; }`, so that the Model records where that part is designed.
4. As a systems engineer, I want `file` to be relative to the `.sysml` file holding the link, so that moving my Model folder doesn't break links.
5. As a systems engineer, I want to name a FreeCAD object by its `Name`, so that renaming its label in FreeCAD doesn't break the link.
6. As a systems engineer, I want to reach an object inside a FreeCAD assembly by a dotted path (`Assembly.Link.Body.`), so that I can link parts that only exist through an assembly link.
7. As a systems engineer, I want to annotate a feature with `@ToolProperty { name = "Volume"; }`, so that its value comes from the linked object's volume.
8. As a systems engineer, I want to put `@ToolProperty` on a feature that has a value (`= 0.9e6 [mm^3]`), on one that has a `default`, or on one that has none, so that I can link a feature at whatever stage my Model is in.
9. As a systems engineer, I want to link a feature that only redefines one inherited from its def (`attribute :>> volume { @ToolProperty … }`), so that the def's `default` stays my estimate while the usage takes the design's value.
10. As a systems engineer, I want the Design Value to replace the Model Value in every check (Satisfaction Status, Constraint Results, Computed Values, Analysis Results), so that my Verdicts reflect the design as it is.
11. As a systems engineer, I want roll-ups such as `mass = volume * density` and `drive.mass = engine.mass + gearbox.mass` recomputed from Design Values, so that a change in one CAD body shows up in the budgets above it.
12. As a systems engineer, I want a Design Value to affect only the usage that carries the link, not its def or sibling usages, so that one linked engine doesn't change every engine.
13. As a systems engineer, I want the Design Value written with its unit (FreeCAD reports mm³), so that OpenSysML converts it to whatever unit my feature and constraints use.
14. As a systems engineer, I want a Design Value whose dimension doesn't fit the feature (volume linked to a mass) to make the check Undecided with a clear reason, so that a wrong link is never silently wrong.
15. As a systems engineer, I want my `.sysml` files left untouched by all of this, so that the Model stays read-only to this tool (ADR 0002).
16. As a systems engineer, when FreeCAD isn't installed or can't be found, I want the dependent checks Undecided with "FreeCAD not available", so that I know why nothing was judged.
17. As a systems engineer, when the linked file doesn't exist, I want the reason to name the file, so that I can fix the path.
18. As a systems engineer, when the linked object isn't in the file (including a missing first object of an assembly path), I want the reason to name the object and the file, so that I can fix the link.
19. As a systems engineer, when FreeCAD hangs, I want the load to give up after 30 seconds with "FreeCAD did not answer within 30 s", so that neither the GUI nor CI waits forever.
20. As a systems engineer, I want to change that timeout for my machine through an environment variable, so that a slow machine or a huge assembly still works.
21. As a systems engineer, when one link fails, I want only the checks that depend on it to be Undecided, and the others still judged, so that one broken link doesn't hide every result.
22. As a systems engineer, I want a missing Design Value never to fall back to the Model Value or a def's `default`, so that no Verdict rests on an estimate without my knowing.
23. As a systems engineer, I want a diagnostic when a `@ToolProperty` sits on a feature whose owner has no Tool Link, so that a misplaced annotation doesn't just do nothing.
24. As a systems engineer, I want a diagnostic when one element has two Tool Links to the same tool, so that I know which link would be ambiguous.
25. As a systems engineer, I want to point the tool at FreeCAD with `SYSML_VIEWER_FREECAD`, so that CI and unusual installs work.
26. As a systems engineer, I want the tool to find `freecadcmd` on `PATH` or in its usual install locations without configuration, including a Windows install seen from WSL, so that it works on my machine out of the box.
27. As a systems engineer, I want all of a Model's FreeCAD links answered by one FreeCAD run per load, so that a Model with many links loads about as fast as one with a single link.
28. As a systems engineer, I want a Model without Tool Links to load exactly as fast as today, without FreeCAD ever starting, so that this feature costs nothing when unused.
29. As a CI maintainer, I want `sysml-viewer check` to print a "Design Values" section first when the Model has Tool Links, with each linked feature's Design Value, its source and its Model Value, so that a changed Verdict is explained right above it.
30. As a CI maintainer, I want failed links listed in that section with their reasons, so that the log shows what to fix.
31. As a CI maintainer, I want `check` to fail when any Tool Link couldn't be read, even without `--strict`, so that a CI machine without FreeCAD can't turn "violated by the design" into a pass.
32. As a CI maintainer, I want that failure to have its own error line (`error: 2 Tool Links could not be read`), so that it isn't confused with a violated requirement.
33. As a CI maintainer, I want Undecided lines to say `no Design Value for drive::gearbox::volume (FreeCAD: …)`, never the service's raw type-mismatch text, so that the reason reads plainly.
34. As a CI maintainer, I want `requirements` to behave like `check` for Design Values (section, failure rule, translated reasons), so that both gates agree.
35. As a CI maintainer, I want the output of a Model without Tool Links unchanged, so that existing pipelines and their expectations keep working.
36. As a systems engineer using the GUI, I want a linked element's Properties to show a Tool Link row (`⚙ FreeCAD · cad/engine.FCStd · Engine`), so that I can see where it is designed.
37. As a systems engineer using the GUI, I want a linked feature's Properties to show Design Value (with tool, object and property) followed by Model Value, in place of the plain Value row, so that I see both numbers and where the first came from.
38. As a systems engineer using the GUI, I want a feature without a Design Value to show `⚠ none: <reason>` in its Design Value row, so that I know why its checks are Undecided.
39. As a systems engineer, I want fraction-valued results (a mass of 157/20 kg from `volume * density`) shown as numbers, so that roll-ups computed from Design Values aren't blank.
40. As a systems engineer, I want units printed readably (`mm³`, `kg/m³`), not as the service writes them (`mm**3`, `SI::'kg⋅m⁻³'`), so that values read like my Model.
41. As a systems engineer using the GUI, I want the Model, with its Design Values, to arrive in one load on the existing worker thread, so that I never see Verdicts from Model Values first and the window never freezes.

## Implementation Decisions

- **Domain (`viewer-core`).** Add an `EngineeringTool` trait. Input: a file (an absolute path resolved by the caller), and per object the property names wanted. Output: per object and property, either a value text with its unit (for example `1000000.0 [mm^3]`) or a reason. An error for the whole file (tool not found, file missing, timeout) is a reason that applies to every property asked of it. The `Model` gains, per linked element, its Tool Link (tool, file as written, object), and per linked feature a **Design Value outcome**: the Design Value text and source (tool, file, object, property) or the reason there is none, plus the Model Value as the Model wrote it. These carry no OpenSysML types (ADR 0001).
- **Adapters (new crate `viewer-tools`).** Holds the FreeCAD adapter, which knows nothing of SysML.
  - It finds `freecadcmd`: first `SYSML_VIEWER_FREECAD`, then `PATH` (`freecadcmd`, `FreeCADCmd`), then known install locations, including `/mnt/c/Program Files/FreeCAD */bin/freecadcmd.exe` under WSL. A Windows executable is given Windows paths (`wslpath -w`) and receives its request through an environment variable passed with `WSLENV`.
  - It embeds a Python script, writes it to a temporary directory, and runs it once per file. The script opens the document, resolves each object by `Name`, or by a dotted assembly path through `getSubObject` (checking that the first object exists), and prints one JSON answer line.
  - For the tracer, the only property is `Volume`, reported in `mm^3`. Unknown properties give a reason.
  - A run is killed after a timeout, 30 s by default, set by an environment variable.
  - The script contract was settled in the prototype (branch `prototype/live-freecad`, `fc/query.py`): request `{"file": <path>, "objects": {"<object>": ["Volume", …]}}`; answer line `FCRESULT {"<object>": {"Volume": [<number>, "mm^3"]} | {"error": "<reason>"}}`, with `"__file__": {"error": …}` when the document can't be opened.
- **Backend (`viewer-opensysml`).** `OpenSysmlBackend` is given its adapters by name (`"FreeCAD"`). Inside `load_with`, after the existing parse:
  1. Read Tool Links from the api-json form. A `MetadataUsage` whose `type` is `EngineeringTools::ToolLink` or `::ToolProperty` has an `owner` (the linked element or feature). Each field's literal is the item whose id is `<metadata id>__<field>_pvalue`. Name an unnamed redefinition by following `redefines`, recursively, as the backend already does for values.
  2. Emit the two diagnostics (a property without a link on its owner; two links to one tool on one element).
  3. Group the wanted properties per tool and file, and call each adapter once per file.
  4. Edit the Model in memory, with one editor per document (`in_document`):
     - A feature that has a value gets `set_value(feature, "<value> [<unit>]")`.
     - A feature with a body and no value gets `delete(feature)` followed by `add_member(owner, "attribute", "", redefines = [name], value)`, because `set_value` refuses such declarations.
     - When there is no Design Value, the same edit writes the string `"no Design Value: <tool>: <reason>"`. OpenSysML then reports every dependent check Undecided with that text in its error chain, and leaves independent checks alone. Removing the value isn't enough, because a def's `default` shows through a valueless redefinition.
  5. Re-parse the edited documents inline, together with the untouched ones from disk, and run the existing checks on that Model.
  6. Translate the marker in every check reason into `no Design Value for <feature> (<tool>: <reason>)`.
  7. Record the Design Value outcomes and the Tool Links on the `Model`.

  A Model with no Tool Links skips all of this.
- **Values shown.** The service value text for a `Rational` (and `BigInteger`) magnitude becomes a decimal number. Units are shown readably (`mm**3` → `mm³`, `SI::'kg⋅m⁻³'` → `kg/m³`, `SI::kg` → `kg`). This applies everywhere computed values are shown, not only for Design Values.
- **CLI.** `check` and `requirements` print a "Design Values" section first when the Model has Tool Links: one line per linked feature with the Design Value, its source and `(Model says …)`, or the reason. They fail when any Tool Link couldn't be read, with `error: N Tool Link(s) could not be read`, independent of `--strict`. The existing failure lines and exit codes are otherwise unchanged.
- **GUI Properties.** A linked element gets a `Tool Link` row. A linked feature's `Value` row is replaced by `Design Value` (value plus `⚙ <tool> <object>.<property>`, or `⚠ none: <reason>`), then `Model Value`. The look came from the UI prototype (branch `prototype/design-values-ui`, variant A).
- **Library.** `EngineeringTools.sysml` defines `metadata def ToolLink { attribute tool : String; attribute file : String; attribute object : String; }` and `metadata def ToolProperty { attribute name : String; attribute tool : String[0..1]; }`, with the doc comments from the prototype. The CLI gains a command that prints it (`sysml-viewer library`). The tool recognises the defs by qualified name and never writes the file.
- **Glossary.** Use **Tool Link**, **Engineering Tool**, **Design Value** and **Model Value** as defined in `CONTEXT.md`, and **Verdict**, **Satisfaction Status**, **Constraint Result** and **Computed Value** as already defined.

## Testing Decisions

- **Good tests** drive the feature from outside, through a public seam, and assert what a user sees: the Model's Design Value outcomes, the Verdicts and their reasons, CLI output and exit codes, Properties rows. They never assert how the edit was made, nor the marker string itself (only its translation).
- **Main seam: the real backend's `load` with a fake `EngineeringTool`.** Add fixture Model folders with the library and Tool Links, modelled on the drive Model from the prototype. Tests:
  - Design Values flip a requirement and an asserted constraint from holds to violated, and roll-ups recompute.
  - A feature with a value, one with a body and no value, and one only inherited from a def `default` all take Design Values.
  - A failed link makes exactly the dependent checks Undecided with the translated reason, independent checks are still judged, and a def `default` is not used.
  - Both diagnostics appear.
  - A dimension mismatch is Undecided.
  - A Model without links never calls the tool.
  - The `.sysml` files are byte-for-byte unchanged after a load.

  Prior art: `viewer-opensysml/tests/contract.rs`, which loads fixtures through the real service.
- **CLI with `FakeBackend`**: a `Model` carrying Design Value outcomes and Tool Links. Covers the Design Values section, the translated Undecided lines, failing on unreadable links without `--strict`, unchanged output without links, and `library` printing the file. Prior art: `viewer-cli/tests/check.rs` and `requirements.rs`.
- **Properties**: Tool Link, Design Value (both forms) and Model Value rows from a `Model` built in the test. Prior art: `viewer-core/tests/properties.rs`.
- **FreeCAD adapter through the trait**, against the real `freecadcmd` and a small committed `.FCStd` fixture (a 100 mm cube body, and a cylinder reached through an `App::Link` in an `App::Part`). Covers volume and unit, the assembly path, a missing object, a missing first object of a path, a missing file, a missing executable, and the timeout (a stub executable that sleeps). Tests that need FreeCAD are skipped with a printed note when it can't be found, in the same way the service tests depend on `sysml-grpc`.
- **Value text**: `Rational` and readable-unit formatting, tested where computed values are tested today (`viewer-core/tests/computed.rs` and the service translation tests).

## Out of Scope

- The Design Values tab, and the `⚙`/`⚠` marks on Diagram compartment rows (the next slice, already designed: variant A rows plus variant C's table).
- `--model-values`, watching linked files for reloads, and the in-memory cache of answers between loads.
- FreeCAD properties other than `Volume` (`Mass` with its material guard, `Area`, `CenterOfMass.*`).
- `ToolProperty.tool` disambiguation, and several Tool Links on one element (the diagnostic for two links to one tool is in).
- The CI recipe for installing FreeCAD (AppImage), and `export` warnings for unreadable links.
- KiCad, Tool Links on defs, vector Design Values, writing to the Model, pushing values into CAD, a plug-in protocol for third-party tools, and Model servers.

## Further Notes

- Design and reasoning: `docs/engineering-tools.md`, ADRs 0003 to 0005, and the wayfinder map `.scratch/engineering-tools/map.md`.
- The whole pipeline runs as one program on branch `prototype/live-freecad` (`prototypes/live-freecad/src/main.rs`), against FreeCAD 1.1.4 and sysml-grpc 0.10.0. It shows the api-json shapes, the edits and the marker working. Measured: FreeCAD about 0.6–0.75 s per run including startup; reading links about 110 ms; edit about 55 ms; re-parse about 45 ms.
- `set_value` on a declaration with a body is requested upstream (Issue 3 in `docs/upstream/opensysml-issues.md`, unposted). If OpenSysML adds it, the delete and add workaround can go.
- On this machine FreeCAD is the Windows install at `C:\Program Files\FreeCAD 1.1`, called from WSL. The adapter's WSL path handling exists for this case.
