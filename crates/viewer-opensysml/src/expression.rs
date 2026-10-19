//! Rebuilds the text of an expression from its API JSON element form, for
//! showing a requirement's `require` constraints. The service sends
//! expressions as trees, not text; anything not understood here becomes
//! `…` rather than something misleading.

use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::Value;

/// What a construct the rebuilder doesn't know shows as.
pub const UNKNOWN: &str = "…";

/// The text of the expression with id `id`, looking up its parts in `items`.
pub fn text(items: &HashMap<String, (Value, PathBuf)>, id: &str) -> String {
    render(items, id).0
}

/// How a feature is named in text: its short name, else its name, else,
/// for a redefinition with no name of its own (`attribute :>> mass = 180.0`),
/// the name of the feature it redefines.
fn feature_name(items: &HashMap<String, (Value, PathBuf)>, feature: &Value) -> Option<String> {
    if let Some(name) = feature
        .get("declaredShortName")
        .or_else(|| feature.get("declaredName"))
    {
        return name.as_str().map(str::to_owned);
    }
    let redefinition = feature
        .get("ownedRedefinition")?
        .as_array()?
        .first()?
        .get("@id")?
        .as_str()?;
    let (redefinition, _) = items.get(redefinition)?;
    let redefined = redefinition.get("redefinedFeature")?.get("@id")?.as_str()?;
    feature_name(items, &items.get(redefined)?.0)
}

/// Binding strength, weakest first; `[` (a unit) binds tightest.
fn precedence(operator: &str, unary: bool) -> u8 {
    if unary {
        return 10;
    }
    match operator {
        "??" => 0,
        "implies" => 1,
        "or" | "|" => 2,
        "xor" => 3,
        "and" | "&" => 4,
        "==" | "!=" | "===" | "!==" => 5,
        "<" | ">" | "<=" | ">=" => 6,
        "+" | "-" => 7,
        "*" | "/" | "%" => 8,
        "**" | "^" => 9,
        "[" => 11,
        _ => 12,
    }
}

/// Exponentiation groups to the right; every other binary operator to the
/// left.
fn right_associative(operator: &str) -> bool {
    matches!(operator, "**" | "^" | "implies")
}

/// The text, and how strongly it binds (atoms bind tightest).
fn render(items: &HashMap<String, (Value, PathBuf)>, id: &str) -> (String, u8) {
    const ATOM: u8 = 12;
    let Some((item, _)) = items.get(id) else {
        return (UNKNOWN.to_owned(), ATOM);
    };
    let arguments: Vec<&str> = item
        .get("argument")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("@id").and_then(Value::as_str))
                .collect()
        })
        .unwrap_or_default();
    let name_of = |property: &str| -> Option<String> {
        let reference = item.get(property)?;
        if let Some(id) = reference.get("@id").and_then(Value::as_str) {
            let (target, _) = items.get(id)?;
            // An anonymous chain feature reads as its links: `vehicle.powertrain`.
            if let Some(chain) = target.get("chainingFeature").and_then(Value::as_array) {
                let names: Option<Vec<String>> = chain
                    .iter()
                    .map(|link| feature_name(items, &items.get(link.get("@id")?.as_str()?)?.0))
                    .collect();
                return names.map(|n| n.join("."));
            }
            return feature_name(items, target);
        }
        // An unresolved reference keeps the name it was written with.
        reference
            .get("@ref")
            .and_then(Value::as_str)
            .map(str::to_owned)
    };
    match item
        .get("@type")
        .and_then(Value::as_str)
        .unwrap_or_default()
    {
        "LiteralInteger" | "LiteralRational" => match item.get("value") {
            Some(Value::Number(n)) => (n.to_string(), ATOM),
            _ => (UNKNOWN.to_owned(), ATOM),
        },
        "LiteralBoolean" => match item.get("value").and_then(Value::as_bool) {
            Some(b) => (b.to_string(), ATOM),
            None => (UNKNOWN.to_owned(), ATOM),
        },
        "LiteralString" => match item.get("value").and_then(Value::as_str) {
            Some(s) => (format!("\"{s}\""), ATOM),
            None => (UNKNOWN.to_owned(), ATOM),
        },
        "LiteralInfinity" => ("*".to_owned(), ATOM),
        "NullExpression" => ("null".to_owned(), ATOM),
        "FeatureReferenceExpression" => (
            name_of("referent").unwrap_or_else(|| UNKNOWN.to_owned()),
            ATOM,
        ),
        "FeatureChainExpression" => {
            let (Some(source), Some(target)) = (arguments.first(), name_of("targetFeature")) else {
                return (UNKNOWN.to_owned(), ATOM);
            };
            let (source, strength) = render(items, source);
            let source = if strength < ATOM {
                format!("({source})")
            } else {
                source
            };
            (format!("{source}.{target}"), ATOM)
        }
        "InvocationExpression" => {
            let Some(function) = name_of("function") else {
                return (UNKNOWN.to_owned(), ATOM);
            };
            let arguments: Vec<String> = arguments.iter().map(|a| render(items, a).0).collect();
            (format!("{function}({})", arguments.join(", ")), ATOM)
        }
        "OperatorExpression" => {
            let Some(operator) = item.get("operator").and_then(Value::as_str) else {
                return (UNKNOWN.to_owned(), ATOM);
            };
            match arguments.as_slice() {
                [operand] => {
                    let strength = precedence(operator, true);
                    let (operand, inner) = render(items, operand);
                    let operand = if inner < strength {
                        format!("({operand})")
                    } else {
                        operand
                    };
                    let space = if operator.chars().all(char::is_alphabetic) {
                        " "
                    } else {
                        ""
                    };
                    (format!("{operator}{space}{operand}"), strength)
                }
                [left, right] if operator == "[" => {
                    let (value, inner) = render(items, left);
                    let value = if inner < precedence("[", false) {
                        format!("({value})")
                    } else {
                        value
                    };
                    (
                        format!("{value} [{}]", render(items, right).0),
                        precedence("[", false),
                    )
                }
                [left, right] => {
                    let strength = precedence(operator, false);
                    let right_assoc = right_associative(operator);
                    let (left, l) = render(items, left);
                    let (right, r) = render(items, right);
                    let left = if l < strength || (l == strength && right_assoc) {
                        format!("({left})")
                    } else {
                        left
                    };
                    let right = if r < strength || (r == strength && !right_assoc) {
                        format!("({right})")
                    } else {
                        right
                    };
                    (format!("{left} {operator} {right}"), strength)
                }
                _ => (UNKNOWN.to_owned(), ATOM),
            }
        }
        _ => (UNKNOWN.to_owned(), ATOM),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Items from `(id, json)` pairs, as the converted element form has them.
    fn items(entries: Vec<(&str, Value)>) -> HashMap<String, (Value, PathBuf)> {
        entries
            .into_iter()
            .map(|(id, v)| (id.to_owned(), (v, PathBuf::new())))
            .collect()
    }

    fn op(operator: &str, args: &[&str]) -> Value {
        let argument: Vec<Value> = args.iter().map(|a| json!({"@id": a})).collect();
        json!({"@type": "OperatorExpression", "operator": operator, "argument": argument})
    }

    fn reference(target: &str) -> Value {
        json!({"@type": "FeatureReferenceExpression", "referent": {"@id": target}})
    }

    fn chain(source: &str, target: &str) -> Value {
        json!({"@type": "FeatureChainExpression", "argument": [{"@id": source}], "targetFeature": {"@id": target}})
    }

    fn feature(name: &str) -> Value {
        json!({"declaredName": name})
    }

    fn int(value: i64) -> Value {
        json!({"@type": "LiteralInteger", "value": value})
    }

    /// `v.a`, `v.b`, `v.ok` and `limit`, ready to combine.
    fn base() -> Vec<(&'static str, Value)> {
        vec![
            ("v", feature("v")),
            ("a", feature("a")),
            ("b", feature("b")),
            ("ok", feature("ok")),
            ("limit", feature("limit")),
            ("rv", reference("v")),
            ("va", chain("rv", "a")),
            ("vb", chain("rv", "b")),
            ("vok", chain("rv", "ok")),
            ("rlimit", reference("limit")),
        ]
    }

    fn render_with(extra: Vec<(&'static str, Value)>, root: &str) -> String {
        let mut entries = base();
        entries.extend(extra);
        text(&items(entries), root)
    }

    #[test]
    fn a_comparison_of_a_feature_chain_and_a_feature() {
        assert_eq!(
            render_with(vec![("e", op("<=", &["va", "rlimit"]))], "e"),
            "v.a <= limit"
        );
    }

    #[test]
    fn parentheses_appear_only_where_precedence_needs_them() {
        let extra = vec![
            ("sum", op("+", &["va", "vb"])),
            ("two", int(2)),
            ("times", op("*", &["sum", "two"])),
            ("ten", int(10)),
            ("gt", op(">", &["times", "ten"])),
            ("not", op("not", &["vok"])),
            ("e", op("and", &["gt", "not"])),
        ];
        assert_eq!(render_with(extra, "e"), "(v.a + v.b) * 2 > 10 and not v.ok");
    }

    #[test]
    fn a_left_associative_operator_keeps_parentheses_on_its_right() {
        let extra = vec![
            ("one", int(1)),
            ("inner", op("-", &["vb", "one"])),
            ("e", op("-", &["va", "inner"])),
            ("flat", op("-", &["inner", "one"])),
        ];
        assert_eq!(render_with(extra.clone(), "e"), "v.a - (v.b - 1)");
        assert_eq!(render_with(extra, "flat"), "v.b - 1 - 1");
    }

    #[test]
    fn logic_unary_minus_and_rationals() {
        let extra = vec![
            ("three", int(3)),
            ("eq", op("==", &["va", "three"])),
            ("half", json!({"@type": "LiteralRational", "value": 1.5})),
            ("neg", op("-", &["half"])),
            ("ne", op("!=", &["vb", "neg"])),
            ("or", op("or", &["eq", "ne"])),
            ("e", op("implies", &["or", "vok"])),
        ];
        assert_eq!(
            render_with(extra, "e"),
            "v.a == 3 or v.b != -1.5 implies v.ok"
        );
    }

    #[test]
    fn a_quantity_with_its_unit() {
        let extra = vec![
            (
                "kg",
                json!({"declaredName": "kilogram", "declaredShortName": "kg"}),
            ),
            ("value", int(1650)),
            ("unit", reference("kg")),
            ("q", op("[", &["value", "unit"])),
            ("e", op("<=", &["va", "q"])),
        ];
        assert_eq!(render_with(extra, "e"), "v.a <= 1650 [kg]");
    }

    #[test]
    fn strings_booleans_and_invocations() {
        let extra = vec![
            ("x", json!({"@type": "LiteralString", "value": "x"})),
            ("y", json!({"@type": "LiteralString", "value": "y"})),
            ("eq", op("==", &["x", "y"])),
            ("t", json!({"@type": "LiteralBoolean", "value": true})),
            ("e", op("xor", &["eq", "t"])),
            ("Cap", feature("Cap")),
            (
                "call",
                json!({"@type": "InvocationExpression", "function": {"@id": "Cap"}, "argument": [{"@id": "va"}]}),
            ),
            (
                "loose",
                json!({"@type": "InvocationExpression", "function": {"@ref": "abs"}, "argument": [{"@id": "vb"}]}),
            ),
        ];
        assert_eq!(render_with(extra.clone(), "e"), "\"x\" == \"y\" xor true");
        assert_eq!(render_with(extra.clone(), "call"), "Cap(v.a)");
        assert_eq!(
            render_with(extra, "loose"),
            "abs(v.b)",
            "an unresolved name is kept as written"
        );
    }

    #[test]
    fn an_unnamed_redefinition_reads_as_the_feature_it_redefines() {
        let extra = vec![
            ("Component::mass", feature("mass")),
            (
                "rd",
                json!({"@type": "Redefinition", "redefinedFeature": {"@id": "Component::mass"}}),
            ),
            (
                "engine::@0",
                json!({"@type": "AttributeUsage", "ownedRedefinition": [{"@id": "rd"}]}),
            ),
            ("engine", feature("engine")),
            (
                "chain",
                json!({"@type": "Feature", "chainingFeature": [{"@id": "engine"}, {"@id": "engine::@0"}]}),
            ),
            ("e", reference("chain")),
        ];
        assert_eq!(render_with(extra, "e"), "engine.mass");
    }

    #[test]
    fn a_reference_to_a_chain_reads_as_its_links() {
        let extra = vec![
            ("vehicle", feature("vehicle")),
            ("powertrain", feature("powertrain")),
            (
                "chain",
                json!({"@type": "Feature", "chainingFeature": [{"@id": "vehicle"}, {"@id": "powertrain"}]}),
            ),
            ("e", reference("chain")),
        ];
        assert_eq!(render_with(extra, "e"), "vehicle.powertrain");
    }

    #[test]
    fn anything_unknown_is_an_ellipsis_never_a_guess() {
        let extra = vec![
            (
                "odd",
                json!({"@type": "SelectExpression", "argument": [{"@id": "va"}]}),
            ),
            ("e", op("<=", &["odd", "rlimit"])),
            ("dangling", op("<=", &["va", "missing"])),
        ];
        assert_eq!(render_with(extra.clone(), "e"), "… <= limit");
        assert_eq!(render_with(extra, "dangling"), "v.a <= …");
    }
}
