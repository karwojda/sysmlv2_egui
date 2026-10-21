# Can OpenSysML evaluate with values supplied from outside?

**Status:** resolved
**Type:** research
**Blocked by:** –
**Map:** [Engineering Tools](../map.md)

## Question

Can a value for a feature (for example `engine.mass`) that comes from outside the Model reach OpenSysML's check and evaluation calls (the ones behind Satisfaction Status, Constraint Results, Computed Values and Analysis Results) without editing `.sysml` files? Options to check: a request parameter or binding overlay, an in-memory extra file or snippet sent to the service, redefinition in a generated sidecar package, anything else. See `docs/opensysml-encoding.md` and the `opensysml` crate.

## Answer

**Yes, without editing any `.sysml` file on disk, but not through a request parameter.** Verified against sysml-grpc v0.10.0 with a small experiment.

- **No overlay in the API.** The check calls (`VerifySatisfaction`, `ValidateInstance`) take no values. `RunAnalysis` and `VerifyConstraint` accept arguments, but only for declared `in` parameters; supplying `v.engine.mass` is rejected as an unknown parameter.
- **Route 1, edit in memory and re-parse (best fit).** `Model::edit().set_value("Car::vehicle::engine::mass", "170.0")` returns edited text without writing to disk. Sending that text back as an inline `SourceDocument` gives a new model. Satisfaction Status, Constraint Results, Computed Values and Analysis Results all change, under the Model's own names. This works whether the value is written with `=`, with `default =`, or left unset. `base_model_hash` should keep the re-parse incremental (not measured).
- **Route 2, sidecar package as an extra inline document.** A generated `part measured :> vehicle { part :>> engine { attribute :>> mass = 170.0; } }` works, but results come back under the sidecar's names. Claims and analysis subjects have to be restated for it. If the Model writes the value with `=`, the service reports the error `feature-value-overriding`.
- **Re-opening the Model's package** in a second document fails, because the second package is a separate namespace and its references don't resolve.
- **Plug-in point:** today the viewer sends every Model file by path (`crates/viewer-opensysml/src/lib.rs` 148-151). Either route changes only that spot.
- **Not tested:** setting a feature the Model doesn't redeclare at that usage. That needs an add-member edit rather than `SetValue`.

Findings and the experiment: branch `research/opensysml-external-values`, `docs/research/opensysml-external-values.md`. The branch is committed locally (328ebd3) but not pushed: there are no GitHub credentials in this environment.
