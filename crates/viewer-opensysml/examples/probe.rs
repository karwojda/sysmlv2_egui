//! Shows what the service makes of a SysML file: its symbol tree, and the
//! items of its API JSON element form whose `@type` contains one of the
//! given words (all of them when none are given). For finding out how a
//! construct is encoded before reading it in `api_json.rs`:
//!
//! `cargo run -p viewer-opensysml --example probe -- model.sysml FeatureValue Subject`

use opensysml::{
    Connection, ConvertOptions, ConvertSource, SourceDocument, SourcesOptions, Symbol,
};
use serde_json::Value;

/// The properties worth printing; the rest is noise for most questions.
const SHOWN: [&str; 16] = [
    "@id",
    "@type",
    "declaredName",
    "qualifiedName",
    "owner",
    "memberElement",
    "featureWithValue",
    "type",
    "subject",
    "subsets",
    "redefinedFeature",
    "referent",
    "chainingFeature",
    "direction",
    "kind",
    "isDefault",
];

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let file = args
        .next()
        .ok_or("usage: probe <model.sysml> [@type word …]")?;
    let wanted: Vec<String> = args.collect();
    let connection = Connection::private()?;
    let parsed =
        connection.parse_sources(&[SourceDocument::file(&file)], &SourcesOptions::default())?;
    for root in parsed.roots() {
        tree(root, 0)?;
    }
    let conversion = connection.convert(
        "api-json",
        &ConvertSource::Model(parsed.hash().to_owned()),
        &ConvertOptions::default(),
    )?;
    let items: Vec<Value> = serde_json::from_str(&conversion.content)?;
    for item in &items {
        let kind = item["@type"].as_str().unwrap_or_default();
        if !wanted.is_empty() && !wanted.iter().any(|w| kind.contains(w.as_str())) {
            continue;
        }
        let shown: Vec<String> = SHOWN
            .iter()
            .filter_map(|&key| Some(format!("{key}={}", item.get(key).filter(|v| !v.is_null())?)))
            .collect();
        println!("JSON {}", shown.join(" | "));
    }
    Ok(())
}

/// Prints `symbol` and its descendants, one per line, indented by depth.
fn tree(symbol: &Symbol, depth: usize) -> Result<(), Box<dyn std::error::Error>> {
    let info = symbol.wire();
    if !info.id.is_empty() {
        println!(
            "TREE {}{} [{}] type={:?} meta={:?}",
            "  ".repeat(depth),
            info.id,
            info.kind,
            info.type_info.as_ref().map(|t| &t.resolved_id),
            info.metadata
        );
    }
    for child in symbol.children()? {
        tree(&child, depth + 1)?;
    }
    Ok(())
}
