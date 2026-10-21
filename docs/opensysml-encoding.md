# How OpenSysML encodes a Model

What the backend reads from the sysml-grpc service, where each fact comes from, and the quirks that cost a probe to find. The code that reads it is `crates/viewer-opensysml/src/lib.rs` (`collect`), `api_json.rs`, `expression.rs` and `checks/service.rs`.

## Probe first

Before reading a new construct, look at what the service sends for it:

```sh
cargo run -p viewer-opensysml --example probe -- model.sysml FeatureValue Subject
```

It prints the symbol tree (`TREE` lines) and the JSON items whose `@type` contains one of the words (`JSON` lines, all items when no word is given). Write a small `.sysml` file with just the construct, probe it, then write the reader and a contract test in `tests/contract.rs`.

## Three inputs

1. **Symbol tree** (`parse_sources`, stable). Kinds (`partDef`, `analysisCaseUsage`, …), owners, typing including standard-library types (`type_info.resolved_id`), multiplicity, specializations, and `metadata["direction"]` for parameters. Everything named is here; unnamed elements are not.
2. **API JSON element form** (`convert("api-json")`, marked experimental). Adds what the tree lacks: unnamed elements, documentation, values, subjects, objectives, connector ends, conjugation, metadata usages, source lines. Read defensively: a missing property leaves the element as it was.
3. **Check calls**: `verify_satisfaction`, `validate_instance`, `run_analysis`. Their answers are translated once, in `checks/service.rs`.

## Names and ids

- A JSON `@id` is the qualified name with `::` written `__` and `@` written `_40`: `A::MassCheck::@2` is `A__MassCheck___402`. A membership's id is its member's id plus `_om`. Use `qualifiedName` to get our names; only `expose` falls back on the `_om` rule.
- Unnamed elements get generated names, numbered within their owner: `P::@1`, `Req::@10`. Sort them by that number, not as text, to keep declaration order.
- A redefinition without a name of its own (`attribute :>> mass = 180.0`) is unnamed in the element form (`engine::@0`) but appears in the symbol tree under the name of the feature it redefines (`engine::mass`). Its owner may be an unnamed redefinition too (`part :>> tank { … }`), so resolve names recursively (`valued_element`).

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
- A computed magnitude an `f64` doesn't hold exactly is a `Rational` (`1.0e6 [mm^3] * 7850 [kg/m^3]` is `157/20`), and one beyond `i64` a `BigInteger`. Units come back in the service's spelling: `SI::kg`, `SI::'m³'` and `SI::'kg⋅m⁻³'` from products and quotients, `mm**3` from sums. `value_text` writes them as decimals in readable units (`7.85 [kg]`, `kg/m³`, `mm³`).

## Quirks

- The service refuses to convert any document of a Model with a syntax error anywhere. Converting the file on its own with `tolerate_syntax_errors` still works, losing only ends that reach into other files.
- A `satisfy` nested inside a part is evaluated without its subject, and the JSON form omits `declaredShortName` for standard-library elements (`kg`). Both are written up, not yet filed, in `docs/upstream/opensysml-issues.md`.
