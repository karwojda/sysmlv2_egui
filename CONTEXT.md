# SysML v2 Viewer

A desktop tool for systems engineers to open SysML v2 models, see them as diagrams, and export those diagrams into documentation.

## Language

### Model

**Model**:
The set of SysML v2 textual-notation files the user opens; the single source of truth, which this tool reads but never writes.
_Avoid_: project, workspace, repository

**Model Order**:
The order a Model declares its elements in: by file, then by where each starts in it. A Diagram draws a parent's children left to right in Model Order.
_Avoid_: declaration order, source order

**Unsupported Element**:
A Model element outside the language subset this tool draws; it is listed in the Browser Diagram but never appears in other Diagrams.
_Avoid_: unknown element, opaque element

### Diagrams

**Diagram**:
A picture of part of the Model, of one Diagram Kind, drawn by this tool.
_Avoid_: view (a SysML `view` is a Model element, not a Diagram)

**Diagram Kind**:
The SysML v2 standard visualization a Diagram follows: Browser, General, or Interconnection (others later).
_Avoid_: diagram type, chart

**Tool-defined Diagram**:
A Diagram the user creates by choosing a Diagram Kind and a root element.

**Model-defined Diagram**:
A Diagram derived from a SysML `view` usage in the Model, using what it exposes and renders. Its Diagram Kind and elements come from the Model each time it is drawn; only its Layout is stored.

**Exposed Element**:
An element a `view` names with `expose`, so it appears in the view's Model-defined Diagram (`X` itself, `X::*` its members, `X::**` all its descendants).

**Unsupported View**:
A `view` whose rendering or definition has no matching Diagram Kind in this tool (for example a table or a sequence view). It is listed but can't be drawn.

**Layout**:
The user's adjustments to a Diagram's automatic arrangement (positions, sizes, edge routes, port positions, hidden, collapsed and opened elements), stored outside the Model.
_Avoid_: view, styling, diagram data

**Frame**:
In an Interconnection Diagram, the box of the root part that every other part is drawn inside; the root's own ports sit on its border.
_Avoid_: container, boundary, root box

**Opened Part**:
A part in an Interconnection Diagram drawn with its own parts and connections inside its box; parts are closed (drawn as plain boxes with their ports) unless opened.
_Avoid_: expanded part, drilled-down part (collapsing is about compartments, not parts)

**Diagram File**:
A saved Diagram: its Diagram Kind and root element (or, for a Model-defined Diagram, the `view` it comes from), and its Layout, in one file beside the Model. Diagrams that are opened but never saved leave no Diagram File.
_Avoid_: layout file, diagram definition

**Orphaned Layout Entry**:
A Layout entry whose element qualified name no longer matches anything in the Model, for example after a rename. It is kept in the Diagram File, not deleted.

**Standard Notation**:
The SysML v2 specification's graphical notation, which every Diagram follows by default.

**Satisfaction Claim**:
A `satisfy` in the Model stating that a part (the satisfier) satisfies a requirement usage. It is drawn as a `«satisfy»` edge from the satisfier to the requirement.

**Satisfaction Status**:
Whether a requirement usage is met, as checked by evaluating its Satisfaction Claims against the Model's values: Satisfied (every claim holds), Violated (a claim's condition is false), Undecided (a claim couldn't be evaluated, for example a value is missing), or Not claimed (nothing claims to satisfy it). With several claims, the worst one decides. A composite requirement also rolls up its nested requirements' statuses, worst first: Violated, Undecided, Not claimed, Satisfied. A claim on a composite with no constraint of its own has nothing to check, so only the nested requirements count.
_Avoid_: verified, passed (verification cases are a separate thing)

**Asserted Constraint**:
A constraint the Model asserts must hold (`assert constraint { mass <= 1500 }`), in a part or part def, either written out or as a usage of a constraint def with its parameters bound. A violated one makes the Model inconsistent.

**Constraint Result**:
What checking an Asserted Constraint on one part found: holds, violated (with the failing condition), or undecided (with the reason). An inherited constraint has a result for each part that inherits it. A plain, unasserted constraint has none.

**Computed Value**:
The value OpenSysML evaluates for a feature of a part, from the Model's values and expressions (`powertrain.mass` = 265.0). It is shown beside the expression it comes from, never instead of it.

**Analysis Result**:
What running an analysis usage found: the values of its outputs (its `return` result among them), and, if it has an objective, whether the objective holds, is violated or is undecided.

**Verdict**:
What an element shows because of what OpenSysML checked: holds, violated, undecided, or absent (nothing to judge yet). Satisfaction Status, the Constraint Results of an Asserted Constraint and an Analysis Result's objective are its three kinds, each in its own words: a requirement is Satisfied or Not claimed, an asserted constraint nobody checked Not checked, an analysis Not run. An analysis without an objective has no Verdict, nor does a part. With several, the worst decides: violated, undecided, absent, holds.
_Avoid_: status (too generic), check result

**Style**:
A set of Style Rules layered on the Standard Notation, coloring node fills only. It is defined once for the Model and can be overridden per Diagram: a Diagram's own rules come on top of the Model's, or replace them when it switches the Model Style off.
_Avoid_: theme, color scheme, skin

**Style Rule**:
One condition (the element's kind, a package it is inside, or metadata it is annotated with) and the fill color it gives. When several rules match an element, the most specific wins: metadata over package over kind.

**Legend**:
A key drawn on a Diagram, and in its Export, pairing each fill color used with the Style Rule that gave it. Shown only when the Diagram asks for it.

### Engineering Tools

**Engineering Tool**:
An external program that owns part of the design in its own files, such as FreeCAD for mechanical geometry or KiCad for circuit boards. The Model states intent; an Engineering Tool holds a design that should meet it.
_Avoid_: domain tool, external tool, CAD (too narrow), domain (means something else here)

**Tool Link**:
A statement in the Model that one of its elements is designed in a specific place inside an Engineering Tool, for example that `part engine` is the body `Engine` in `engine.FCStd`. It is written as metadata from the `EngineeringTools` library (ADR 0004): `@ToolLink` on the element, and `@ToolProperty` on each of its features that takes a Design Value, naming the tool's property (`Volume`). A feature with a `@ToolProperty` is a linked feature; `@ToolProperty` is part of the Tool Link, not a concept of its own.
_Avoid_: binding (a SysML binding connects features), mapping, trace

**Design Value**:
The value an Engineering Tool reports, through a Tool Link, for a feature of the linked element (the mass FreeCAD reports for body `Engine`). Checks use it in place of the Model Value; a difference between the two is shown, not judged.
_Avoid_: pulled value, actual value, measured value

**Summed Design Value**:
A Design Value an Engineering Tool adds up over the parts of a design, such as the power of every placed component on a KiCad board. A part that doesn't state the value is left out and named as uncounted, not treated as unknown; a part that states it unreadably leaves the sum with no Design Value (ADR 0006).
_Avoid_: rollup (a SysML rollup sums Model features), total, aggregate

**Model Value**:
The value the Model itself writes for a feature. Once a Tool Link gives that feature a Design Value, the Model Value is kept and shown beside it, typically as the estimate or budget from before the design existed.
_Avoid_: estimate (it isn't always one), nominal value

### Output

**Export**:
Rendering a Diagram, with its Layout, to an SVG or PNG file, either from the GUI or from the headless command line.
_Avoid_: print, publish, save
