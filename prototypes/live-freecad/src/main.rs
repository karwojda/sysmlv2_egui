// PROTOTYPE, throwaway: "Does a live FreeCAD query work from this tool?"
// Model with Tool Links -> read links from api-json -> freecadcmd.exe per file -> set_value in memory
// -> re-parse inline -> checks again. Prints every step and its time.
// Usage: cargo run -- [--freecad <path to freecadcmd>]
use opensysml::{Connection, SourceDocument, SourcesOptions, VerifyOptions};
use serde_json::{Value, json};
use std::{collections::BTreeMap, process::Command, time::Instant};

const MODEL: [&str; 2] = ["model/EngineeringTools.sysml", "model/drive.sysml"];

#[derive(Debug)]
struct Link { element: String, tool: String, file: String, object: String }
#[derive(Debug)]
struct Prop { feature: String, owner: String, name: String, own: String, has_value: bool }

fn report(m: &opensysml::Model) {
    for d in m.diagnostics() { println!("  DIAG {}: {}", d.severity, d.message); }
    match m.verify_satisfaction(None, &VerifyOptions::default()) {
        Ok(s) => for v in &s.verdicts { println!("  SATISFY {} by {}: {:?} {}", v.requirement_id, v.instance_type_id, v.status, v.error); },
        Err(e) => println!("  SATISFY ERR {e}"),
    }
    match m.validate_instance("Drive::drive", None) {
        Ok(val) => {
            for v in &val.verdicts { println!("  CONSTRAINT {} at {}: {:?} {}", v.element_id, v.instance_path, v.status, v.error); }
            for i in val.instances() {
                for (n, f) in i.feature_values() {
                    if ["mass", "volume"].contains(&n.as_str()) { println!("  VALUE {}.{} = {:?}", i.type_symbol_id(), n, f.value()); }
                }
            }
        }
        Err(e) => println!("  VALIDATE ERR {e}"),
    }
}

fn links(m: &opensysml::Model) -> Result<(Vec<Link>, Vec<Prop>), Box<dyn std::error::Error>> {
    let j: Value = serde_json::from_str(&m.to_api_json()?.content)?;
    let items: BTreeMap<String, Value> = j.as_array().unwrap().iter()
        .map(|i| i.get("payload").unwrap_or(i).clone())
        .map(|i| (i["@id"].as_str().unwrap_or_default().to_owned(), i)).collect();
    let qn = |id: &Value| items.get(id["@id"].as_str().unwrap_or_default()).and_then(|i| i["qualifiedName"].as_str()).unwrap_or("?").to_owned();
    let field = |meta: &Value, name: &str| -> Option<String> {
        let id = format!("{}__{}_pvalue", meta["@id"].as_str()?, name);
        items.get(&id)?["value"].as_str().map(str::to_owned)
    };
    let (mut ls, mut ps) = (vec![], vec![]);
    for i in items.values().filter(|i| i["@type"] == "MetadataUsage") {
        let ty = i["type"][0]["@id"].as_str().unwrap_or_default();
        let owner = &i["owner"];
        match ty {
            "EngineeringTools__ToolLink" => ls.push(Link {
                element: qn(owner), tool: field(i, "tool").unwrap_or_default(),
                file: field(i, "file").unwrap_or_default(), object: field(i, "object").unwrap_or_default() }),
            "EngineeringTools__ToolProperty" => {
                let f = &items[owner["@id"].as_str().unwrap()];
                if std::env::var("DUMP").is_ok() { println!("FEATURE {f}"); for r in f["ownedRedefinition"].as_array().into_iter().flatten() { println!("  RD {}", items.get(r["@id"].as_str().unwrap()).map(|x| x.to_string()).unwrap_or_default()); } }
                // `:>> volume` is unnamed: take its name from the feature it redefines
                // (a chain: Gearbox's own `:>> volume default = …` is unnamed too)
                let mut g = f;
                let own = loop {
                    if let Some(n) = g["declaredName"].as_str() { break n.to_owned() }
                    let Some(next) = g["redefines"]["@id"].as_str().and_then(|id| items.get(id)) else { break "?".into() };
                    g = next;
                };
                let fid = f["@id"].as_str().unwrap_or_default();
                let has_value = items.values().any(|x| x["@type"] == "FeatureValue" && x["owningRelatedElement"]["@id"] == fid);
                ps.push(Prop { feature: format!("{}::{own}", qn(&f["owner"])), owner: qn(&f["owner"]), name: field(i, "name").unwrap_or_default(), own, has_value })
            }
            _ => {}
        }
    }
    Ok((ls, ps))
}

/// One freecadcmd run per file. Returns object -> property -> Ok("<value> [unit]") | Err(reason).
fn query(freecad: &str, file: &str, objects: &BTreeMap<String, Vec<String>>) -> BTreeMap<String, BTreeMap<String, Result<String, String>>> {
    let win = |p: &str| String::from_utf8(Command::new("wslpath").args(["-w", p]).output().unwrap().stdout).unwrap().trim().to_owned();
    let all_err = |why: String| objects.iter().map(|(o, ps)| (o.clone(), ps.iter().map(|p| (p.clone(), Err(why.clone()))).collect())).collect();
    if !std::path::Path::new(file).exists() { return all_err(format!("file {file} not found")); }
    let req = json!({"file": win(file), "objects": objects}).to_string();
    let out = match Command::new(freecad).arg(win("fc/query.py")).env("FC_REQ", &req).env("WSLENV", "FC_REQ").current_dir("/tmp").output() {
        Ok(o) => o,
        Err(e) => return all_err(format!("FreeCAD not available ({freecad}: {e})")),
    };
    let text = String::from_utf8_lossy(&out.stdout);
    let Some(line) = text.lines().find_map(|l| l.split_once("FCRESULT ").map(|x| x.1)) else {
        return all_err(format!("FreeCAD gave no answer: {}", String::from_utf8_lossy(&out.stderr)));
    };
    let v: Value = serde_json::from_str(line).unwrap();
    if let Some(e) = v["__file__"]["error"].as_str() { return all_err(e.to_owned()); }
    objects.iter().map(|(o, ps)| (o.clone(), ps.iter().map(|p| {
        let r = &v[o];
        let r = if r["error"].is_string() { r } else { &r[p] };
        (p.clone(), match r { Value::Array(a) => Ok(format!("{} [{}]", a[0], a[1].as_str().unwrap())), _ => Err(r["error"].as_str().unwrap_or("?").to_owned()) })
    }).collect())).collect()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let freecad = args.iter().position(|a| a == "--freecad").map(|i| args[i + 1].clone())
        .unwrap_or("/mnt/c/Program Files/FreeCAD 1.1/bin/freecadcmd.exe".into());
    let t = Instant::now();
    let c = Connection::private()?;
    println!("connect: {:?}", t.elapsed());

    let t = Instant::now();
    let docs = MODEL.map(SourceDocument::file);
    let base = c.parse_sources(&docs, &SourcesOptions::default())?;
    println!("=== Model Values (parse {:?})", t.elapsed());
    let t = Instant::now();
    report(&base);
    println!("  (checks {:?})", t.elapsed());

    let t = Instant::now();
    let (ls, ps) = links(&base)?;
    println!("=== Tool Links (read from api-json {:?})", t.elapsed());
    for l in &ls { println!("  {l:?}"); }
    for p in &ps { println!("  {p:?}"); }

    // group by file; file is relative to the .sysml holding the link (here: all in model/)
    let mut per_file: BTreeMap<String, BTreeMap<String, Vec<String>>> = BTreeMap::new();
    let mut wants = vec![];
    for p in &ps {
        let Some(l) = ls.iter().find(|l| l.element == p.owner) else { println!("  DIAG {}: @ToolProperty but no Tool Link on {}", p.feature, p.owner); continue };
        let file = format!("model/{}", l.file);
        per_file.entry(file.clone()).or_default().entry(l.object.clone()).or_default().push(p.name.clone());
        wants.push((p, file, l.object.clone(), p.name.clone()));
    }
    let t = Instant::now();
    let answers: BTreeMap<_, _> = per_file.iter().map(|(f, objs)| (f.clone(), query(&freecad, f, objs))).collect();
    println!("=== FreeCAD ({} file(s), {:?})", answers.len(), t.elapsed());

    let mut ed = base.edit();
    ed.in_document("model/drive.sysml"); // one editor per document: links in several files need several edits
    let mut undecided = vec![];
    let mut any = false;
    for (p, file, object, name) in &wants {
        let feature = &p.feature;
        match &answers[file][object][name] {
            Ok(v) if p.has_value => { println!("  Design Value {feature} = {v}  ({object}.{name}) via set_value"); ed.set_value(feature, v); any = true; }
            Ok(v) => {
                // set_value can't add a value to a declaration with a body, and a @ToolProperty feature always has one:
                // replace the declaration with a bare redefinition carrying the value
                println!("  Design Value {feature} = {v}  ({object}.{name}) via delete + add_member");
                ed.delete(feature, false);
                ed.add_member(&p.owner, "attribute", "", opensysml::MemberOptions { value: Some(v.clone()), redefines: vec![p.own.clone()], ..opensysml::MemberOptions::new() });
                any = true;
            }
            Err(why) => {
                // No fallback to the Model Value. Stripping it isn't enough (a def's `default` shows through a
                // valueless redefinition), so write a string that can't be a quantity: OpenSysML says Undecided, with our reason
                let marker = format!("\"no Design Value: {}\"", why.replace('"', "'"));
                println!("  no Design Value for {feature}: {why}; writing marker {marker}");
                ed.delete(feature, false);
                ed.add_member(&p.owner, "attribute", "", opensysml::MemberOptions { value: Some(marker), redefines: vec![p.own.clone()], ..opensysml::MemberOptions::new() });
                undecided.push((feature.clone(), why.clone()));
                any = true;
            }
        }
    }
    if !any { return Ok(()); }

    let t = Instant::now();
    let edited = ed.apply()?;
    println!("=== edit applied ({:?})", t.elapsed());
    for d in &edited.documents {
        for l in d.content.lines().filter(|l| l.contains("volume") && l.contains(":>>")) { println!("  {}> {}", d.name, l.trim()); }
    }
    let t = Instant::now();
    // `documents` holds only what the edit touched: send the rest from disk
    let docs: Vec<_> = MODEL.iter().map(|f| match edited.documents.iter().find(|d| d.name == *f) {
        Some(d) => SourceDocument::inline(d.name.clone(), d.content.clone()),
        None => SourceDocument::file(f),
    }).collect();
    let designed = c.parse_sources(&docs, &SourcesOptions::default())?;
    println!("=== Design Values (re-parse {:?})", t.elapsed());
    let t = Instant::now();
    report(&designed);
    println!("  (checks {:?})", t.elapsed());
    Ok(())
}
