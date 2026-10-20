//! Throwaway experiment: can external values reach the check calls?
use opensysml::{AnalysisOptions, Connection, SourceDocument, SourcesOptions, Value, VerifyOptions};

fn report(label: &str, model: &opensysml::Model, part: &str, analysis: &str, subject: Option<&str>, scope: Option<&str>) {
    println!("=== {label}");
    for d in model.diagnostics() {
        println!("  DIAG {:?}", d);
    }
    match model.verify_satisfaction(scope, &VerifyOptions::default()) {
        Ok(s) => for v in &s.verdicts {
            println!("  SATISFY req={} by={} status={:?} holds={} err={:?} cond={:?}", v.requirement_id, v.instance_type_id, v.status, v.holds, v.error, v.condition);
        },
        Err(e) => println!("  SATISFY ERR {e}"),
    }
    match model.validate_instance(part, None) {
        Ok(val) => {
            for v in &val.verdicts {
                println!("  VALIDATE {} kind={} path={} status={:?} err={:?}", v.element_id, v.kind, v.instance_path, v.status, v.error);
            }
            for i in val.instances() {
                for (n, f) in i.feature_values() {
                    println!("  VALUE {}::{} = {:?}", i.type_symbol_id(), n, f.value());
                }
            }
        }
        Err(e) => println!("  VALIDATE ERR {e}"),
    }
    let opts = AnalysisOptions { subject: subject.map(str::to_owned), ..Default::default() };
    match model.run_analysis(analysis, &opts) {
        Ok(r) => for o in &r.outputs { println!("  ANALYSIS {} = {:?}", o.name, o.value); },
        Err(e) => println!("  ANALYSIS ERR {e}"),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = std::env::args().nth(1).unwrap();
    let text = std::fs::read_to_string(&file)?;
    let c = Connection::private()?;
    let opts = SourcesOptions::default();

    // 0. Baseline
    let base = c.parse_sources(&[SourceDocument::file(&file)], &opts)?;
    report("baseline", &base, "Car::vehicle", "Car::rollup", None, None);

    // 1. Request parameters: named argument for an `in` parameter of the analysis
    let mut named = std::collections::BTreeMap::new();
    named.insert("extra".to_owned(), Value::Real(5.0));
    match base.run_analysis("Car::rollup", &AnalysisOptions { named_arguments: named.clone(), ..Default::default() }) {
        Ok(r) => println!("=== named in-arg extra=5\n  {:?}", r.outputs.iter().map(|o| (&o.name, &o.value)).collect::<Vec<_>>()),
        Err(e) => println!("=== named in-arg ERR {e}"),
    }
    // named argument for a non-parameter feature
    let mut bogus = std::collections::BTreeMap::new();
    bogus.insert("v.engine.mass".to_owned(), Value::Real(170.0));
    match base.run_analysis("Car::rollup", &AnalysisOptions { named_arguments: bogus, ..Default::default() }) {
        Ok(r) => println!("=== named non-param arg\n  {:?}", r.outputs.iter().map(|o| (&o.name, &o.value)).collect::<Vec<_>>()),
        Err(e) => println!("=== named non-param arg ERR {e}"),
    }

    // 2. In-memory edit (ApplyEdits SetValue) then reparse inline
    let mut ed = base.edit();
    ed.set_value("Car::vehicle::engine::mass", "170.0");
    match ed.apply() {
        Ok(res) => {
            println!("=== edited text differs only at value: {}", res.content.contains("= 170.0") && !res.content.contains("180.0"));
            let edited = c.parse_sources(&[SourceDocument::inline(file.clone(), res.content.clone())], &opts)?;
            report("in-memory SetValue reparsed", &edited, "Car::vehicle", "Car::rollup", None, None);
            println!("  disk unchanged: {}", std::fs::read_to_string(&file)? == text);
        }
        Err(e) => println!("=== edit ERR {e}"),
    }

    // 3. Sidecar overlay package as an extra inline document
    let sidecar = r#"package Overlay {
    private import Car::*;
    part measured :> vehicle {
        part :>> engine { attribute :>> mass = 170.0; }
    }
    satisfy massReq by measured;
}"#;
    let side = c.parse_sources(&[SourceDocument::file(&file), SourceDocument::inline("overlay.sysml", sidecar)], &opts)?;
    report("sidecar subset-of-vehicle", &side, "Overlay::measured", "Car::rollup", Some("Overlay::measured"), Some("Overlay"));
    println!("  -- unscoped satisfy in sidecar model:");
    if let Ok(s) = side.verify_satisfaction(None, &VerifyOptions::default()) {
        for v in &s.verdicts { println!("  SATISFY req={} by={} status={:?} err={:?}", v.requirement_id, v.instance_type_id, v.status, v.error); }
    }

    // 4. Sidecar re-opening the Model's own package
    let reopen = r#"package Car {
    part :>> vehicle { part :>> engine { attribute :>> mass = 170.0; } }
}"#;
    match c.parse_sources(&[SourceDocument::file(&file), SourceDocument::inline("reopen.sysml", reopen)], &opts) {
        Ok(m) => report("sidecar reopening Car", &m, "Car::vehicle", "Car::rollup", None, None),
        Err(e) => println!("=== reopen ERR {e}"),
    }

    // 5. Evaluate with subject on the sidecar
    match side.evaluate("engine.mass", &opensysml::EvalOptions { context: None, subject: Some("Overlay::measured".into()) }) {
        Ok(v) => println!("=== evaluate engine.mass on Overlay::measured = {:?}", v.result),
        Err(e) => println!("=== evaluate ERR {e}"),
    }
    Ok(())
}
