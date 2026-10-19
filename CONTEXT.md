# SysML v2 Viewer

A desktop tool for systems engineers to open SysML v2 models, see them as diagrams, and export those diagrams into documentation.

## Language

### Model

**Model**:
The set of SysML v2 textual-notation files the user opens; the single source of truth, which this tool reads but never writes.
_Avoid_: project, workspace, repository

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

**Style**:
A set of Style Rules layered on the Standard Notation, coloring node fills only. It is defined once for the Model and can be overridden per Diagram: a Diagram's own rules come on top of the Model's, or replace them when it switches the Model Style off.
_Avoid_: theme, color scheme, skin

**Style Rule**:
One condition (the element's kind, a package it is inside, or metadata it is annotated with) and the fill color it gives. When several rules match an element, the most specific wins: metadata over package over kind.

**Legend**:
A key drawn on a Diagram, and in its Export, pairing each fill color used with the Style Rule that gave it. Shown only when the Diagram asks for it.

### Output

**Export**:
Rendering a Diagram, with its Layout, to an SVG or PNG file, either from the GUI or from the headless command line.
_Avoid_: print, publish, save
