# How OpenSysML encodes a Model

What the backend reads from the sysml-grpc service, where each fact comes from, and the quirks that cost a probe to find. The code that reads it is `crates/viewer-opensysml/src/lib.rs` (`collect`), `api_json.rs`, `expression.rs`, `checks/service.rs` and `design_values/` (`Parsed`, `edit.rs`, `words.rs`). It describes sysml-grpc 0.11; what changed from 0.10 is listed at the end.

## Probe first

Before reading a new construct, look at what the service sends for it:

```sh
cargo run -p viewer-opensysml --example probe -- model.sysml FeatureValue Subject
```

It prints the symbol tree (`TREE` lines) and the JSON items whose `@type` contains one of the words (`JSON` lines, all items when no word is given). Write a small `.sysml` file with just the construct, probe it, then write the reader and a contract test in `tests/contract.rs`.

## Three inputs

1. **Symbol tree** (`parse_sources`, stable). Kinds (`partDef`, `analysisCaseUsage`, …), owners, typing including standard-library types (`type_info.resolved_id`), multiplicity, specializations, and `metadata["direction"]` for parameters. Everything named is here; unnamed elements are not.
2. **API JSON element form** (`convert("api-json")`, marked experimental). Adds what the tree lacks: unnamed elements, documentation, values, subjects, objectives, connector ends, conjugation, metadata usages, source lines. Read defensively: a missing property leaves the element as it was. Every property is written, `null` when not set; `enrich` drops the `null`s on reading, so a property that is there is set.
3. **Check calls**: `verify_satisfaction`, `validate_instance`, `run_analysis`. Their answers are translated once, in `checks/service.rs`.

## Names and ids

- A JSON `@id` is the qualified name with `::` written `__` and `@` written `_40`: `A::MassCheck::@2` is `A__MassCheck___402`. A membership's id is its member's id plus `_om`. Use `qualifiedName` to get our names (`name_items` sets it, below); only `expose` falls back on the `_om` rule.
- `qualifiedName` quotes a name that needs it (`V::'vehicle tree'`); `sysx:mappingQualifiedName` has the symbol tree's spelling (`V::vehicle tree`), and `name_items` prefers it.
- An unnamed element has no `qualifiedName`, only its owner and its `sysx:memberIndex`. `name_items` names it after both, `P::@1`, `Req::@10`, as the ids do. Sort them by that number, not as text, to keep declaration order. A member with a `declaredName` is named under its owner's name, generated or not. The fields a metadata body sets (`@ToolLink { tool = … }`) have neither name nor redefinition name and come out `engine::@0::@0`, where 0.10 said `engine::@0::tool`; no reader names them.
- A redefinition without a name of its own (`attribute :>> mass = 180.0`) is named after the feature it redefines, in the element form (`engine::mass`, id `engine__mass`, with a `name` but no `declaredName`) as in the symbol tree. So is a `render asTreeDiagram` (`TreeView::asTreeDiagram`). Its owner may be an unnamed element (`engine::@0`), so resolve names recursively (`valued_element`). The feature it redefines may be unnamed as well (a usage's `:>> volume` redefines its def's own `:>> volume default = …`), so follow `Redefinition` → `redefinedFeature` until one has a `declaredName` (`declared_name`).
- An unnamed redefinition with a body and no value (`attribute :>> volume { @ToolProperty … }`) refuses `set_value`; to give it one, `delete` it and `add_member(owner, "attribute", "", redefines = ["volume"], value)`.
- A named feature with a body and no value (`attribute bore : LengthValue { @ToolProperty … }`) can't be given one through the editor: `set_value` refuses the body, `delete` and `rename` refuse while anything refers to it, `add_member` refuses its name while it exists, `move_to` respells the references, and the body's unnamed members (`bore::@0`, named only in the JSON form) are unknown to it. The service does parse a value written before the body (`attribute bore : LengthValue = 8.0 [mm] { … }`), so `design_values::write_value` writes it into the text, at the declaration's `sysx:sourceLine`/`sysx:sourceColumn` (both from 1, the column counted in UTF-8 bytes: `ö` before it on its line counts 2, `😀` 4; `sysx:sourceEndLine`/`sysx:sourceEndColumn`, `sysx:hasBody` and `sysx:sourceText` are there too), before the service's edits.
- A string written as a quantity feature's value (`"no Design Value #0#"`) is accepted by the edit; exactly the checks that depend on the feature come back Undecided with the string somewhere in their error chain, and the rest are judged as before. Removing the value instead would let a def's `default` show through. A quantity of the wrong dimension comes back as `feature value pump.mass: type mismatch: cannot write … to a feature typed by MassValue (dimension M)`, naming the feature only from its part, so two linked `engine.mass` under different parents read the same. Neither text is shown as is: `design_value_reason` in `checks/service.rs` puts both into words, telling same-named features apart by the part the check was about and by the value the service could not write.
- An unnamed redefinition's `sysx:sourceLine` sits on its own item (`engine::mass`); read the line from that item.
- Standard-library elements are not in the form. A reference to one is a bare id, a name-based uuid: uuid v5 of the qualified name (`Views::asTreeDiagram`) under the root package's uuid, itself uuid v5 of `https://www.omg.org/spec/SysML/<package>` (or `KerML/` for `ScalarValues` and the other KerML packages) in the URL namespace. A name we can guess is checked against the id (`is_library_id`); `name_items` does it for renderings. A reference in an expression reads as it was written, from the reference's own `sysx:sourceText` (`kg`), or for a function from the invocation's text up to `(` (`sum`).

## Where each fact lives

| Fact | Where |
|---|---|
| Documentation | a `Documentation` item; `owner` is the documented element, `body` the text |
| A feature's value | a `FeatureValue` item: `featureWithValue` is the feature, `memberElement` the expression, `isDefault` marks `default` |
| Expression text | rebuilt from the expression tree (`expression.rs`); the service sends no text. Unknown constructs become `…` |
| A requirement's or analysis's subject | a `SubjectMembership` owned by it; `memberElement` is the subject feature, whose `type` the symbol tree leaves empty |
| A subject bound by value (`subject v = vehicle`, `satisfy r by vehicle.powertrain`) | the subject feature's own `FeatureValue`, whose expression is a `FeatureReferenceExpression` (`referent`) or a `FeatureChainExpression` (`targetFeature`) |
| A feature chain (`vehicleSpec.massReq`) | the item's `chainingFeature` array; the last entry is the feature meant |
| A Satisfaction Claim | a `SatisfyRequirementUsage`: `subsets` the requirement, `subject` the satisfier, else the subject's value, else the owner (`satisfy r;` inside a part) |
| `require` constraints | `RequirementConstraintMembership` items with `kind` = `requirement`, each holding a constraint whose `ResultExpressionMembership` is the expression |
| An asserted constraint | `@type` `AssertConstraintUsage`; its expression as above |
| An analysis's objective | an `ObjectiveMembership`; its member is an unnamed requirement with its own `Documentation` and `require` constraints |
| A `return` parameter | a `ReturnParameterMembership` owned by the analysis |
| Connector ends | `connectorEnd`; each end's `ReferenceSubsetting` names the feature or chain, and a flow end's owned `ReferenceUsage` `redefines` the port it reaches |
| Metadata | owned `MetadataUsage` items, by `type` |
| A metadata field's value (`@ToolLink { tool = "FreeCAD"; … }`) | the literal item `<metadata id>__<field>_pvalue` (`value`); a `@ToolProperty`'s `owner` is the feature, whose own `FeatureValue` says it has a value |
| `expose` | `MembershipExpose` (`importedMembership`) or `NamespaceExpose` (`importedNamespace`), with `isRecursive` |

## Check answers

- A verdict's `status` is `holds` or `violated`, with the failing `condition`; a non-empty `error` means it couldn't be evaluated. An analysis objective's verdict has no status text, only the `holds` flag.
- `validate_instance` of a top-level part returns verdicts with an `instance_path` relative to that part (`tank`, `axle.wheel`), and every instance's feature values: the value of feature `f` on an instance of type symbol `T` belongs to `T::f`.
- An analysis's output (`run_analysis` `outputs`) is a sequence of its one value: `Sequence([Real(1363.0)])`.
- A computed magnitude an `f64` doesn't hold exactly is a `Rational` (`1.0e6 [mm^3] * 7850 [kg/m^3]` is `157/20`), and one beyond `i64` a `BigInteger`. Units come back in the service's spelling: `SI::kg`, `SI::'m³'` and `SI::'kg⋅m⁻³'` from products and quotients, `mm**3` from sums. `value_text` writes them as decimals in readable units (`7.85 [kg]`, `kg/m³`, `mm³`).

## Quirks

- The service refuses to convert any document of a Model with a syntax error anywhere. Converting the file on its own with `tolerate_syntax_errors` still works, losing only ends that reach into other files.
- Parsing reports the service's checks of the Model as diagnostics, `subject-parameter-position` among them, without a separate call.

## From 0.10 to 0.11

What the move to 0.11 changed, for the next upgrade:

- `null` for every property not set, where 0.10 left it out.
- No `qualifiedName` on unnamed elements; quoted names in `qualifiedName`, with `sysx:mappingQualifiedName` added.
- Unnamed redefinitions and renderings named after what they redefine or render (`engine::mass`, not `engine::@0`), ids included.
- No standard-library items: 0.10 sent a stub for each referenced one (`@id`, `declaredName`, `qualifiedName`, `isLibraryElement`); 0.11 sends none. Units read as written (`[kg]`, where 0.10 gave `[kilogram]`).
- Analysis outputs as one-value sequences.
- A `satisfy` nested inside a part is evaluated against that part (`docs/upstream/opensysml-issues.md`, Issue 1).
- New checks reported on parse, such as `subject-parameter-position`.
