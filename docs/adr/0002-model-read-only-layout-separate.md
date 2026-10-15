# The Model is read-only; Layouts live in separate files

The tool never writes to `.sysml` files. The user's diagram adjustments (positions, routes, hidden and collapsed elements) are saved as Layout files beside the Model and committed with it, so Exports can be reproduced in CI. We considered storing layout in the Model (for example as metadata annotations), but that would turn a viewer into an editor of the user's source and clutter the textual notation.
