# Can OpenSysML evaluate with values supplied from outside the Model?

Research for `.scratch/engineering-tools/issues/02-opensysml-external-values.md` (map: Engineering Tools). Date: 2026-10-20. Service: `sysml-grpc` v0.10.0 (`~/.opensysml/bin/sysml-grpc.json`), client crate `opensysml` 0.10.0.

## Answer

Yes, without touching any `.sysml` file on disk, in two ways that were both run against the service. There is **no** request parameter or binding overlay that sets an arbitrary feature (such as `vehicle.engine.mass`) on a check call; request arguments reach only `in` parameters.

1. **In-memory edit, then re-parse (best fit).** `ApplyEdits` with a `SetValueEdit` returns the edited notation as text and writes nothing. Sending that text back as an *inline* `SourceDocument` (the other files still by path) gives a new `model_hash` whose Satisfaction Status, Constraint Results, Computed Values and Analysis Results all use the new value, under the Model's own names.
2. **Sidecar package as an extra inline document.** A generated `package Overlay` that specializes the Model's part and redefines the feature works for `validate_instance`, `run_analysis` (with the subject overridden) and `evaluate`, but every Satisfaction Claim must be restated in the sidecar against the new part, results come back under the sidecar's names, and a Model value written with `=` (not `default =`) makes the service report an error diagnostic.

Re-opening the Model's package in a second document does not work: it is a separate namespace.

## The request surface (what the wire allows)

| Call | Value-carrying fields | Source |
|---|---|---|
| `VerifySatisfaction` | none: `model_hash`, `symbol_id` (scope), `engine`, `question`. The client refuses a `subject` ("a satisfaction assertion names its own subject") | `opensysml-0.10.0/src/proto/sysml/sysml.rs` 253-267; `src/operations.rs` 777-787 |
| `ValidateInstance` | none: `model_hash`, `symbol_id`, `engine` | `sysml.rs` 293-302 |
| `VerifyConstraint` | `subject_symbol_id`; `arguments` / `named_arguments` for the constraint's **`in` parameters** only, anything else is `INVALID_ARGUMENT` | `sysml.rs` 126-162 |
| `RunAnalysis` | `subject_symbol_id`; `arguments` / `named_arguments` for the case's **input parameters** | `sysml.rs` 425-445; `src/operations.rs` 98-109 |
| `Evaluate` | `context_symbol_id`, `subject_symbol_id`; no values | `sysml.rs` 802-816 |
| `RunDocumentQuery` | `bindings` for a document query's entry parameters, not feature values | `sysml.rs` ~2620-2650 |

Nothing in the proto is named or documented as an override or overlay of feature values (`grep -i 'override\|overlay'` finds nothing; `binding` only in the comments above). Experiment: `named_arguments {"v.engine.mass": 170.0}` on `RunAnalysis` fails with `unknown parameter: analysis Car::rollup has no input parameter "v.engine.mass"`, while `{"extra": 5.0}` for a declared `in extra : Real` works (1180 to 1185).

So request arguments only help if the Model is written to take the value as an `in` parameter of an analysis or constraint. They cannot reach `VerifySatisfaction` or `ValidateInstance` at all.

## What makes options 1 and 2 possible

- A model is a list of `SourceDocument`s, each either a `file_path` or inline `content` with a `name`; one `model_hash` names them all together (`sysml.rs` 689-727; client `SourceDocument::file` / `SourceDocument::inline`, `src/sources.rs` 26-46).
- `ParseSourcesRequest.base_model_hash` makes a re-parse incremental: the answer lists the affected documents, and the rest reuse the base's results ("parse_sources_affected" capability; `sysml.rs` 722-726, 742-751).
- `ApplyEditsRequest` takes a `model_hash`, operations and a document name, and answers with the edited notation; `SetValueEdit { target, value }` takes the value in SysML notation (`"1050.0[SI::kg]"`, `"mass * 2"`) and requires it to resolve in the feature's scope (`sysml.rs` 1280-1299, 1693-1701). Client: `Model::edit()` then `Editor::set_value(target, value)` and `apply()` (`src/model.rs` 75-81, `src/edit.rs` 892-897). `EditResult` holds `content` (one document) or `documents` (several, needs `accept_documents`), and touches disk only if `write` is called (`src/edit.rs` 291-327).
- Today the viewer sends every Model file by path and nothing else (`crates/viewer-opensysml/src/lib.rs` 148-151), so either option is a change at that point only.

## Experiment

Files: `docs/research/opensysml-external-values/car.sysml` (the Model) and `overlay_exp.rs` (the client, run as a temporary example of `viewer-opensysml` against sysml-grpc v0.10.0). The Model has `part vehicle : Vehicle { part :>> engine { attribute :>> mass = 180.0; } }`, an asserted constraint `engine.mass <= 175.0` on `Vehicle`, `satisfy massReq by vehicle` with `require constraint { v.engine.mass <= 175.0 }`, a derived `totalMass = engine.mass + 1000.0`, and an analysis `rollup` returning `v.totalMass + extra`. The external value is 170.0.

| Variant | Satisfaction | Constraint | `engine.mass` / `totalMass` | Analysis | Diagnostics |
|---|---|---|---|---|---|
| Baseline (`= 180.0`) | violated | violated | 180 / 1180 | 1180 | none |
| 1. `set_value("Car::vehicle::engine::mass", "170.0")`, re-parsed inline | holds | holds | 170 / 1170 | 1170 | none; file on disk unchanged |
| 2. Sidecar `part measured :> vehicle { part :>> engine { attribute :>> mass = 170.0; } } satisfy massReq by measured;` | `Car::vehicle` violated, `Overlay::measured` holds | holds (on `Overlay::measured`) | 170 / 1170 | 1170 with `subject = Overlay::measured` | **error** `feature-value-overriding`: "cannot override the binding value of Car::vehicle::engine::mass: a value written with `=` is fixed for every feature redefining it; write it as `default =` …" |
| 2, Model says `default = 180.0` | as above | as above | 170 / 1170 | 1170 | none |
| Model leaves `mass` without a value | baseline undecided ("no value for feature v.engine.mass"); options 1 and 2 both hold | as left | 170 / 1170 | baseline fails; 1170 | none |
| Second document `package Car { part :>> vehicle { … } }` | unchanged (violated) | unchanged | 180 | 1180 | `unresolved reference: vehicle — did you mean Car::vehicle?`: a second `Car` is not merged with the first |

Option 1 also worked on a feature with no value: `SetValue` on `attribute :>> mass;` added `= 170.0`. Not tested: setting a feature the Model doesn't redeclare at that usage at all (here `vehicle` would need `part :>> engine { attribute :>> mass; }`). That would need an add-member edit (`MemberOptions.value`, `src/edit.rs` 336-380) rather than `SetValue`.

## Trade-offs for the design

- **In-memory edit (1)**: results keep the Model's names, so Satisfaction Status, Constraint Results and Computed Values land on the elements the engineer sees. Works for `=` and `default =` values alike. Costs one `ApplyEdits` round trip per document plus a re-parse, which `base_model_hash` keeps incremental. The value is SysML notation, so units must be written (`170.0 [kg]`). It is also the same mechanism as the later "write to the Model" (`EditResult::write`), which leaves room for that without building it.
- **Sidecar (2)**: needs no edit capability and leaves the Model's own verdicts as they are, which is useful for showing "as modelled" next to "as measured". But it duplicates every claim under new names, needs the subject overridden per analysis, and is flagged as an error when the Model binds the value with `=`. Generated SysML must also import the Model's package and get the names right.
- **Request arguments**: only where the Model is written to take the value as an `in` parameter; not a general route.

## Sources

- `opensysml` 0.10.0 crate source, `~/.cargo/registry/src/index.crates.io-*/opensysml-0.10.0/`: `src/proto/sysml/sysml.rs` (prost code generated from the service's `sysml.proto`), `src/operations.rs`, `src/sources.rs`, `src/edit.rs`, `src/model.rs`. Line numbers as cited.
- `sysml-grpc` v0.10.0 binary from Open-MBEE/OpenSysML (sha256 `bf78a702…`), run privately by `Connection::private()`.
- This repo: `crates/viewer-opensysml/src/lib.rs`, `crates/viewer-opensysml/src/checks/service.rs`, `docs/opensysml-encoding.md`.
