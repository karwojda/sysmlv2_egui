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
A Diagram derived from a SysML `view` usage in the Model, using what it exposes and renders.

**Layout**:
The user's adjustments to a Diagram's automatic arrangement (positions, sizes, edge routes, hidden and collapsed elements), stored outside the Model.
_Avoid_: view, styling, diagram data

**Diagram File**:
A saved Diagram: its Diagram Kind, its root element, and its Layout, in one file beside the Model. Diagrams that are opened but never saved leave no Diagram File.
_Avoid_: layout file, diagram definition

**Orphaned Layout Entry**:
A Layout entry whose element qualified name no longer matches anything in the Model, for example after a rename. It is kept in the Diagram File, not deleted.

**Standard Notation**:
The SysML v2 specification's graphical notation, which every Diagram follows by default.

**Style**:
A set of coloring rules (by element kind, package, or metadata) layered on the Standard Notation. It is defined once for the Model and can be overridden per Diagram.
_Avoid_: theme, color scheme, skin

### Output

**Export**:
Rendering a Diagram, with its Layout, to an SVG or PNG file, either from the GUI or from the headless command line.
_Avoid_: print, publish, save
