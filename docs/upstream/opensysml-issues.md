# Draft issues for OpenSysML (sysml-grpc)

Status: draft, not posted. Review before filing at https://github.com/Open-MBEE/OpenSysML.
Two separate bugs, meant to be filed as two issues. The reproductions are in `repro/` beside this file.
Found 2026-10-18 with sysml-grpc 0.10.0 through the `opensysml` Rust client 0.10.0.

---

## Issue 1: A `satisfy` nested in a usage is evaluated without binding the containing usage as its subject

### Summary

A satisfy requirement usage declared inside a part, without `by`, should take the containing part as its satisfying feature. `VerifySatisfaction` doesn't bind it: the requirement's subject has no value, and the verdict is always `undecided`. The same claim written as `satisfy … by part` at package level evaluates correctly.

### Specification

SysML v2 Part 1, 7.21.4:

> A satisfy requirement usage can be declared without an explicit satisfying feature if it is nested in definition or usage. In this case, the satisfying feature is considered to be given by the containing definition or usage […]

Its example is `part vehicle3 : Vehicle { … }` with the comment `// "vehicle3" is implicitly the satisfying feature.` The semantics in 8.4 says the same: without an explicit satisfying feature, the binding targets `Base::things::that`.

### Reproduction

`repro/nested-satisfy.sysml`:

```sysml
package NestedSatisfy {
    private import ScalarValues::*;

    part def Vehicle {
        attribute mass : Real;
    }
    requirement def MassLimit {
        subject v : Vehicle;
        require constraint { v.mass <= 1600.0 }
    }
    requirement byForm : MassLimit;
    requirement nestedForm : MassLimit;

    // The same part, with the same values, claims both requirements.
    part car : Vehicle {
        attribute :>> mass = 1500.0;
        satisfy nestedForm;          // satisfier implied: car (SysML v2 Part 1, 7.21.4)
    }
    satisfy byForm by car;           // satisfier named
}
```

Parse the file, then call `VerifySatisfaction` over the whole model (no scope, default engine and question).

### Expected

Both verdicts `holds`: `car.mass` is 1500.0 and the limit is 1600.0. The nested verdict names `NestedSatisfy::car` as its instance, like the `by` form does.

### Actual

```
requirement=NestedSatisfy::byForm      instance_type_id="NestedSatisfy::car"  status=holds      error=""
requirement=NestedSatisfy::nestedForm  instance_type_id=""                    status=undecided  error="satisfaction satisfy nestedForm: require condition evaluation failed: no value for feature v"
```

The `api-json` conversion shows the same gap. The nested `SatisfyRequirementUsage` has no `subject`, only its owner:

```
NestedSatisfy::car::@1  owner=NestedSatisfy__car  subject=(none)              subsets=NestedSatisfy__nestedForm
NestedSatisfy::@6       owner=NestedSatisfy       subject=NestedSatisfy__car  subsets=NestedSatisfy__byForm
```

### Impact

Every claim written in the nested style evaluates as `undecided`, whatever the values. That's the natural style when a part lists its own obligations, as on slide 92 of the *Intro to the SysML v2 Language – Graphical Notation*. Tools that report requirement status, or gate CI on it, can't use that style.

### Workaround

Declare the claim outside the part with an explicit satisfying feature: `satisfy nestedForm by car;`.

---

## Issue 2: `api-json` omits `declaredShortName` for standard-library elements

### Summary

In the `api-json` conversion, standard-library elements arrive in a reduced form (`@id`, `@type`, `declaredName`, `qualifiedName`, `isLibraryElement`) without their `declaredShortName`. User elements keep theirs. Clients therefore can't show library references as written: a unit like `[kg]` can only be shown as `[kilogram]`.

### Reproduction

`repro/library-short-name.sysml`:

```sysml
package LibraryShortName {
    private import ISQ::*;
    private import SI::*;

    part def Vehicle {
        attribute <m> mass :> ISQ::mass;
    }
    requirement def MassLimit {
        subject v : Vehicle;
        require constraint { v.m <= 1650 [kg] }
    }
}
```

Parse the file and convert the model to `api-json`.

### Expected

The item for `SI::kilogram` (declared `attribute <kg> kilogram : MassUnit` in `SI.sysml`) carries `"declaredShortName": "kg"`, as user elements do.

### Actual

```
LibraryShortName::Vehicle::mass: declaredName="mass"     declaredShortName="m"        isLibraryElement=false
SI::kilogram:                    declaredName="kilogram" declaredShortName=(missing)  isLibraryElement=true
```

The constraint's `FeatureReferenceExpression` for the unit points at that library item, so the short name the user wrote can't be recovered.

### Impact

Cosmetic but widespread: every reference to a library element written by its short name (most units, `kg`, `m`, `s`, `N` …) can only be shown by its full name.

### Suggestion

Include `declaredShortName` (and, ideally, `shortName`) in the reduced form of library elements.

---

## Notes for whoever files these

- Both were checked against the real service, not inferred. Re-run the reproductions with the current sysml-grpc before filing, in case a newer release fixed either.
- The viewer already copes with both. Nested claims are drawn from their owning part, and shown Undecided with the service's reason. Library references fall back to the full name. Once the service is fixed, the viewer shows correct results with no change.
- Smaller observations, not worth an issue on their own: the symbol tree leaves a requirement's `subject` untyped (the `api-json` form has the type), and lists elements that have a short name twice under the same id.
