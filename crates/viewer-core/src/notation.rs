//! The Standard Notation: how nodes and edges are drawn as scene primitives.

use crate::model::{Element, ElementKind, Model, QualifiedName};
use crate::scene::{Depicts, Fill, Font, Item, Point, Rect, Shape, TextAnchor};
use crate::verdict::Status;

pub(crate) const KEYWORD_FONT: Font = Font {
    size: 12.0,
    bold: false,
    italic: false,
};
pub(crate) const NAME_FONT: Font = Font {
    size: 14.0,
    bold: true,
    italic: false,
};
/// Usage titles are not bold, unlike definition names.
pub(crate) const USAGE_NAME_FONT: Font = Font {
    size: 14.0,
    bold: false,
    italic: false,
};
const COMPARTMENT_TITLE_FONT: Font = Font {
    size: 11.0,
    bold: false,
    italic: true,
};
const ROW_FONT: Font = Font {
    size: 12.0,
    bold: false,
    italic: false,
};
pub(crate) const LABEL_FONT: Font = Font {
    size: 11.0,
    bold: false,
    italic: false,
};
pub(crate) const PADDING: f32 = 10.0;
pub(crate) const LINE_GAP: f32 = 4.0;
pub(crate) const MIN_NODE_WIDTH: f32 = 120.0;
pub(crate) const USAGE_CORNER_RADIUS: f32 = 10.0;
const ARROW_LENGTH: f32 = 14.0;
const ARROW_HALF_WIDTH: f32 = 8.0;
const DIAMOND_LENGTH: f32 = 18.0;
const DIAMOND_HALF_WIDTH: f32 = 6.0;
pub(crate) const LABEL_OFFSET: f32 = 6.0;
/// How far apart edge ends on the same side of a node must be.
const END_SEPARATION: f32 = 14.0;
/// How far edges keep from nodes they pass, and how a detour searches.
const OBSTACLE_CLEARANCE: f32 = 4.0;
const DETOUR_STEP: f32 = 24.0;
const DETOUR_STEPS: usize = 16;
/// How far outside an obstacle's corner a corner bend sits.
const CORNER_CLEARANCE: f32 = 14.0;
/// Sideways distance between parallel edges joining the same two nodes.
const PARALLEL_SPREAD: f32 = 36.0;
const COLON_DOT_RADIUS: f32 = 1.8;
/// A flow's filled arrowhead.
const FLOW_ARROW_LENGTH: f32 = 12.0;
const FLOW_ARROW_HALF_WIDTH: f32 = 6.0;
/// How far the defined-by colon sits before the triangle, and how far
/// apart its two dots are.
const COLON_GAP: f32 = 5.0;
const COLON_HALF_SPREAD: f32 = 4.0;

/// Text width estimated from character count, so layout never depends on
/// the fonts installed on the machine doing the Export.
pub(crate) fn text_width(text: &str, font: Font) -> f32 {
    let average = if font.bold { 0.62 } else { 0.56 };
    // An em space, as a wrapped row's indent, is a whole em wide.
    let ems = text.chars().filter(|&c| c == '\u{2003}').count() as f32;
    (text.chars().count() as f32 - ems) * font.size * average + ems * font.size
}

/// The area `shape` covers, with text estimated like node sizes are.
pub fn extent(shape: &Shape) -> Rect {
    let around = |points: &[Point], pad: f32| {
        let min_x = points.iter().map(|p| p.x).fold(f32::INFINITY, f32::min) - pad;
        let min_y = points.iter().map(|p| p.y).fold(f32::INFINITY, f32::min) - pad;
        let max_x = points.iter().map(|p| p.x).fold(f32::NEG_INFINITY, f32::max) + pad;
        let max_y = points.iter().map(|p| p.y).fold(f32::NEG_INFINITY, f32::max) + pad;
        Rect {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        }
    };
    match shape {
        Shape::Rect { rect, .. } => *rect,
        Shape::Polyline { points, .. } | Shape::Polygon { points, .. } => around(points, 0.0),
        Shape::Circle { center, radius, .. } => around(&[*center], *radius),
        Shape::Text {
            position,
            text,
            anchor,
            font,
            ..
        } => {
            let width = text_width(text, *font);
            let x = match anchor {
                TextAnchor::Start => position.x,
                TextAnchor::Middle => position.x - width / 2.0,
                TextAnchor::End => position.x - width,
            };
            Rect {
                x,
                y: position.y - font.size,
                width,
                height: font.size * 1.3,
            }
        }
    }
}

/// What a node shows: a header, then titled compartments of feature rows.
pub struct NodeContent {
    keyword: String,
    title: String,
    /// A line under the title: a requirement's `subject v:Vehicle`.
    subtitle: Option<String>,
    /// A requirement usage's Satisfaction Status mark, drawn in a badge.
    badge: Option<&'static str>,
    usage: bool,
    compartments: Vec<(&'static str, Vec<String>)>,
}

/// The status badge: a small circle in a requirement usage's top-right
/// corner.
const BADGE_SIZE: f32 = 16.0;
const BADGE_INSET: f32 = 5.0;

/// The compartments a node lists, in order, with the kinds of owned
/// feature each one holds.
const COMPARTMENTS: [(&str, &[ElementKind]); 5] = [
    ("attributes", &[ElementKind::AttributeUsage]),
    ("ports", &[ElementKind::PortUsage]),
    ("parts", &[ElementKind::PartUsage]),
    (
        "connections",
        &[
            ElementKind::ConnectionUsage,
            ElementKind::InterfaceUsage,
            ElementKind::FlowUsage,
            ElementKind::BindingConnection,
        ],
    ),
    ("requirements", &[ElementKind::RequirementUsage]),
];

/// Characters per line of a wrapped `doc` compartment, and of any other
/// compartment row too long for one.
const DOC_LINE_CHARS: usize = 40;

/// What starts the continuation lines of a wrapped row: an indent of em
/// spaces, which SVG keeps where it would collapse plain ones.
const CONTINUED: &str = "\u{2003}\u{2003}";

/// What `element`'s node shows; without `compartments` (collapsed), only
/// its header. Compartments list the owned features `listed` accepts.
pub fn node_content(
    element: &Element,
    model: &Model,
    compartments: bool,
    listed: impl Fn(&QualifiedName) -> bool,
) -> NodeContent {
    let usage = !element.kind.is_definition();
    let compartments_shown = compartments;
    let requirement = matches!(
        element.kind,
        ElementKind::RequirementDefinition | ElementKind::RequirementUsage
    );
    // A requirement's shall-statement leads its compartments.
    let doc = element
        .documentation
        .as_deref()
        .filter(|_| requirement && compartments)
        .map(|text| ("doc", wrap(text, DOC_LINE_CHARS)));
    let features = COMPARTMENTS
        .iter()
        .filter(|_| compartments)
        .filter_map(|(title, kinds)| {
            let rows: Vec<String> = model
                .owned_members(&element.qualified_name)
                .filter(|m| kinds.contains(&m.kind) && listed(&m.qualified_name))
                // The subject is on its own line in the header.
                .filter(|m| element.subject.as_ref() != Some(&m.qualified_name))
                .map(|m| match crate::verdict::of(model, m) {
                    Some(verdict) => format!("{} {}", feature_row(m), verdict.mark()),
                    None => feature_row(m),
                })
                .collect();
            (!rows.is_empty()).then_some((*title, rows))
        });
    // A requirement names its subject under its title; an analysis lists it.
    let subtitle = element
        .subject
        .as_ref()
        .filter(|_| requirement)
        .map(|subject| {
            let label = model
                .element(subject)
                .map(usage_label)
                .unwrap_or_else(|| subject.simple_name().to_owned());
            format!("subject {label}")
        });
    let mut compartments: Vec<(&'static str, Vec<String>)> =
        doc.into_iter().chain(features).collect();
    // A requirement's constraints follow its attributes, as in the notation.
    if compartments_shown && !element.require.is_empty() {
        let rows = element
            .require
            .iter()
            .map(|text| format!("require {{{text}}}"))
            .collect();
        insert_after_attributes(&mut compartments, "constraints", rows);
    }
    let analysis = matches!(
        element.kind,
        ElementKind::AnalysisDefinition | ElementKind::AnalysisUsage
    );
    if compartments_shown && analysis {
        // As on slide 99: subject, objective and parameters come first.
        let mut leading: Vec<(&'static str, Vec<String>)> = Vec::new();
        if let Some(subject) = element.subject.as_ref().and_then(|s| model.element(s)) {
            leading.push(("subject", vec![feature_row(subject)]));
        }
        if let Some(objective) = &element.objective {
            let mut rows: Vec<String> = objective
                .doc
                .as_deref()
                .map_or_else(Vec::new, |d| wrap(d, DOC_LINE_CHARS));
            rows.extend(objective.require.iter().map(|r| format!("require {{{r}}}")));
            if !rows.is_empty() {
                leading.push(("objective", rows));
            }
        }
        let mut parameters: Vec<&Element> = model
            .owned_members(&element.qualified_name)
            .filter(|m| {
                m.direction.is_some() && element.subject.as_ref() != Some(&m.qualified_name)
            })
            .collect();
        parameters.sort_by_key(|m| m.source.as_ref().map(|s| s.line));
        if !parameters.is_empty() {
            leading.push((
                "parameters",
                parameters.into_iter().map(feature_row).collect(),
            ));
        }
        // What running it found, its `return` result among them.
        let results = crate::analysis::result_rows(model, element);
        if !results.is_empty() {
            leading.push(("results", results));
        }
        compartments.splice(0..0, leading);
    }
    if compartments_shown && !analysis {
        let constraint = matches!(
            element.kind,
            ElementKind::ConstraintDefinition | ElementKind::ConstraintUsage
        );
        if constraint {
            // A constraint lists its parameters, with their bound values,
            // then its own expression.
            // Parameters are positional: keep the order they're written in,
            // where the Model knows it.
            let mut parameters: Vec<&Element> = model
                .owned_members(&element.qualified_name)
                .filter(|m| m.direction.is_some() && listed(&m.qualified_name))
                .collect();
            parameters.sort_by_key(|m| m.source.as_ref().map(|s| s.line));
            let parameters: Vec<String> = parameters.into_iter().map(feature_row).collect();
            if !parameters.is_empty() {
                compartments.push(("parameters", parameters));
            }
            if let Some(expression) = &element.expression {
                compartments.push(("expression", vec![expression.clone()]));
            }
        } else {
            // A part's constraints, asserted and plain, follow its attributes.
            let row = crate::constraint::row_text;
            let owned = || {
                model
                    .owned_members(&element.qualified_name)
                    .filter(|m| m.kind == ElementKind::ConstraintUsage && listed(&m.qualified_name))
            };
            let plain: Vec<String> = owned().filter(|c| !c.is_asserted).map(row).collect();
            if !plain.is_empty() {
                insert_after_attributes(&mut compartments, "constraints", plain);
            }
            // On a part, each asserted row ends with what checking it on this
            // part found (○ if never checked), and inherited ones follow
            // with `^`. A definition has no values to check: no marks.
            let part = &element.qualified_name;
            let checked = !element.kind.is_definition();
            let mark_on = |c: &Element| {
                c.checks
                    .iter()
                    .find(|check| &check.part == part)
                    .map_or(Status::Absent, |check| Status::of(&check.result))
                    .mark()
            };
            let mut asserted: Vec<String> = owned()
                .filter(|c| c.is_asserted)
                .map(|c| {
                    if checked {
                        format!("{} {}", row(c), mark_on(c))
                    } else {
                        row(c)
                    }
                })
                .collect();
            if checked {
                asserted.extend(
                    crate::constraint::checks_on(model, part)
                        .filter(|(c, _)| c.owner.as_ref() != Some(part))
                        .map(|(c, check)| {
                            format!("^{} {}", row(c), Status::of(&check.result).mark())
                        }),
                );
            }
            if !asserted.is_empty() {
                insert_after_attributes(&mut compartments, "assert constraints", asserted);
            }
        }
    }
    // What the element claims to satisfy joins its requirements compartment.
    let claims: Vec<String> = model
        .claims()
        .filter(|c| compartments_shown && c.satisfier.as_ref() == Some(&element.qualified_name))
        .filter_map(|c| {
            Some((
                c.satisfies.as_ref()?,
                crate::satisfaction::claim_status(c).mark(),
            ))
        })
        .map(|(requirement, mark)| format!("satisfy {} {mark}", requirement.simple_name()))
        .collect();
    if !claims.is_empty() {
        match compartments
            .iter_mut()
            .find(|(title, _)| *title == "requirements")
        {
            Some((_, rows)) => rows.extend(claims),
            None => compartments.push(("requirements", claims)),
        }
    }
    // A row too long for the node's normal width wraps, as `doc` does.
    for (_, rows) in &mut compartments {
        *rows = rows.iter().flat_map(|row| wrap_row(row)).collect();
    }
    let short_id = element
        .short_name
        .as_ref()
        .map(|id| format!("<{id}> "))
        .unwrap_or_default();
    NodeContent {
        keyword: if element.kind == ElementKind::ConstraintUsage && element.is_asserted {
            "«assert constraint»".to_owned()
        } else {
            format!("«{}»", element.kind.keyword())
        },
        title: if usage {
            format!("{short_id}{}", usage_label(element))
        } else {
            format!("{short_id}{}", element.name)
        },
        subtitle,
        badge: crate::verdict::of(model, element).map(|v| v.mark()),
        usage,
        compartments,
    }
}

/// Whether a value as written is a plain literal (`180.0`, `-1`, `true`,
/// `"x"`, `1650 [kg]`), which needs no computed value beside it.
fn is_literal(value: &str) -> bool {
    let value = value.strip_prefix("default ").unwrap_or(value);
    let number = |s: &str| s.trim().parse::<f64>().is_ok();
    let quantity = value
        .strip_suffix(']')
        .and_then(|v| v.split_once(" ["))
        .is_some_and(|(magnitude, unit)| number(magnitude) && !unit.contains(' '));
    number(value)
        || quantity
        || value == "true"
        || value == "false"
        || (value.starts_with('"') && value.ends_with('"'))
}

/// Puts a compartment after `attributes` (or after `doc`, or first).
fn insert_after_attributes(
    compartments: &mut Vec<(&'static str, Vec<String>)>,
    title: &'static str,
    rows: Vec<String>,
) {
    let after = compartments
        .iter()
        .position(|(t, _)| *t == "attributes")
        .map_or(
            usize::from(compartments.first().is_some_and(|(t, _)| *t == "doc")),
            |i| i + 1,
        );
    compartments.insert(after, (title, rows));
}

/// A compartment row as lines of at most [`DOC_LINE_CHARS`], continuation
/// lines indented by [`CONTINUED`]. It breaks between [`row_words`], after
/// an operator where that keeps at least half a line (`a + b and` / `c`);
/// a word longer than a line keeps a line of its own.
fn wrap_row(row: &str) -> Vec<String> {
    if row.chars().count() <= DOC_LINE_CHARS {
        return vec![row.to_owned()];
    }
    fit_lines(row_words(row))
        .into_iter()
        .enumerate()
        .map(|(i, words)| {
            let text = words.join(" ");
            if i == 0 {
                text
            } else {
                format!("{CONTINUED}{text}")
            }
        })
        .collect()
}

/// The words a row may break between: at spaces, but never inside a unit
/// or a quoted name (`['°C']`, `'max speed'`), nor between a value and its
/// unit (`-10 ['°C']`) or a row and its closing mark (`… ✓`). A quote or
/// bracket left open counts as plain text.
fn row_words(row: &str) -> Vec<String> {
    const MARKS: [&str; 4] = ["✓", "✗", "?", "○"];
    let mut words: Vec<String> = vec![String::new()];
    let (mut bracket, mut quote) = (0_u32, None::<char>);
    for c in row.chars() {
        if c == ' ' && bracket == 0 && quote.is_none() {
            words.push(String::new());
            continue;
        }
        match (c, quote) {
            ('\'' | '"', None) => quote = Some(c),
            (c, Some(q)) if c == q => quote = None,
            ('[', None) => bracket += 1,
            (']', None) => bracket = bracket.saturating_sub(1),
            _ => {}
        }
        words.last_mut().expect("one word at least").push(c);
    }
    if bracket > 0 || quote.is_some() {
        words = row.split(' ').map(str::to_owned).collect();
    }
    let mut glued: Vec<String> = Vec::new();
    for word in words.into_iter().filter(|w| !w.is_empty()) {
        let joins = word.starts_with('[') || MARKS.contains(&word.as_str());
        match glued.last_mut() {
            Some(last) if joins && !is_operator(last) => {
                last.push(' ');
                last.push_str(&word);
            }
            _ => glued.push(word),
        }
    }
    glued
}

/// Whether `word` is an operator a wrapped row prefers to break after.
fn is_operator(word: &str) -> bool {
    const OPERATORS: [&str; 24] = [
        "+", "-", "*", "/", "%", "**", "^", "<", ">", "<=", ">=", "==", "!=", "===", "!==", "=",
        ":=", "and", "or", "xor", "implies", "not", "??", "|",
    ];
    OPERATORS.contains(&word)
}

/// `words` as lines that fit [`DOC_LINE_CHARS`], continuation lines
/// counting their indent; each line is measured again once words carry
/// over to it.
fn fit_lines(words: Vec<String>) -> Vec<Vec<String>> {
    let width = |line: &[String], first: bool| {
        let indent = if first { 0 } else { CONTINUED.chars().count() };
        indent
            + line.iter().map(|w| w.chars().count()).sum::<usize>()
            + line.len().saturating_sub(1)
    };
    let mut lines: Vec<Vec<String>> = Vec::new();
    let mut line: Vec<String> = Vec::new();
    for word in words {
        line.push(word);
        while line.len() > 1 && width(&line, lines.is_empty()) > DOC_LINE_CHARS {
            let first = lines.is_empty();
            // As many words as fit, then back to the last operator among
            // them if that keeps at least half a line.
            let fits = (1..line.len())
                .rev()
                .find(|&n| width(&line[..n], first) <= DOC_LINE_CHARS)
                .unwrap_or(1);
            let cut = line[..fits]
                .iter()
                .rposition(|w| is_operator(w))
                .map(|at| at + 1)
                .filter(|&n| 2 * width(&line[..n], first) >= DOC_LINE_CHARS)
                .unwrap_or(fits);
            let rest = line.split_off(cut);
            lines.push(std::mem::replace(&mut line, rest));
        }
    }
    lines.push(line);
    lines
}

/// `text` broken into lines of at most `width` characters at spaces (a
/// longer word gets a line of its own).
fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for word in text.split_whitespace() {
        match lines.last_mut() {
            Some(line) if line.chars().count() + 1 + word.chars().count() <= width => {
                line.push(' ');
                line.push_str(word);
            }
            _ => lines.push(word.to_owned()),
        }
    }
    lines
}

/// A connector's label: a flow's `name:Payload` (`:Payload` unnamed),
/// otherwise `name:Type`; empty for an unnamed, untyped connector.
pub(crate) fn connector_label(element: &Element) -> String {
    match (&element.kind, &element.payload) {
        (ElementKind::FlowUsage, Some(payload)) => {
            format!("{}:{}", element.name, payload.simple_name())
        }
        (ElementKind::FlowUsage, None) => element.name.clone(),
        (ElementKind::BindingConnection, _) => "=".to_owned(),
        _ => usage_label(element),
    }
}

/// A flow's arrowhead: a filled triangle at the middle of the path's
/// longest run, pointing along the path (from its first point to its last).
pub(crate) fn flow_arrow(points: &[Point]) -> Shape {
    let (a, b) = points
        .windows(2)
        .map(|s| (s[0], s[1]))
        .max_by(|x, y| distance(x.0, x.1).total_cmp(&distance(y.0, y.1)))
        .expect("a path has a segment");
    let middle = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    let length = distance(a, b).max(f32::EPSILON);
    let (ux, uy) = ((b.x - a.x) / length, (b.y - a.y) / length);
    let tip = Point::new(
        middle.x + ux * FLOW_ARROW_LENGTH / 2.0,
        middle.y + uy * FLOW_ARROW_LENGTH / 2.0,
    );
    let base = Point::new(
        middle.x - ux * FLOW_ARROW_LENGTH / 2.0,
        middle.y - uy * FLOW_ARROW_LENGTH / 2.0,
    );
    let (nx, ny) = (-uy * FLOW_ARROW_HALF_WIDTH, ux * FLOW_ARROW_HALF_WIDTH);
    Shape::Polygon {
        points: vec![
            tip,
            Point::new(base.x + nx, base.y + ny),
            Point::new(base.x - nx, base.y - ny),
        ],
        fill: Fill::Stroke,
    }
}

/// `name:Type [multiplicity]`, with `~` for a conjugated type.
pub(crate) fn usage_label(element: &Element) -> String {
    let mut label = element.name.clone();
    if let Some(definition) = &element.typed_by {
        let conjugate = if element.conjugated { "~" } else { "" };
        label.push_str(&format!(":{conjugate}{}", definition.simple_name()));
    }
    if let Some(multiplicity) = &element.multiplicity {
        label.push_str(&format!(" {multiplicity}"));
    }
    label
}

/// A compartment row: ports lead with their direction, connections trail
/// with their ends.
fn feature_row(feature: &Element) -> String {
    let mut row = String::new();
    if feature.is_return {
        row.push_str("return ");
    } else if let Some(direction) = feature.direction {
        row.push_str(direction.keyword());
        row.push(' ');
    }
    if feature.kind == ElementKind::BindingConnection {
        // `bind a = b`, with the binding's name first if it has one.
        let ends: Vec<String> = feature.ends.iter().map(ToString::to_string).collect();
        let name = if feature.name.is_empty() {
            String::new()
        } else {
            format!("{} ", feature.name)
        };
        return format!("bind {name}{}", ends.join(" = "));
    }
    if feature.kind == ElementKind::FlowUsage {
        row.push_str("flow ");
    }
    if let Some(id) = &feature.short_name {
        row.push_str(&format!("<{id}> "));
    }
    row.push_str(&connector_label(feature));
    match feature.value.as_deref() {
        Some(value) if value.starts_with("default ") => row.push_str(&format!(" {value}")),
        Some(value) => row.push_str(&format!(" = {value}")),
        None => {}
    }
    // What an expression comes to, after it; a literal says itself.
    let expression = feature.value.as_deref().is_some_and(|v| !is_literal(v));
    if expression && !feature.computed.is_empty() {
        row.push_str(&format!(" = {}", feature.computed.join(", ")));
    }
    if feature.kind.is_connection() && !feature.ends.is_empty() {
        let ends: Vec<String> = feature.ends.iter().map(ToString::to_string).collect();
        row.push_str(&format!(" ({})", ends.join(" → ")));
    }
    row.trim_start().to_owned()
}

fn title_font(content: &NodeContent) -> Font {
    if content.usage {
        USAGE_NAME_FONT
    } else {
        NAME_FONT
    }
}

fn header_height(content: &NodeContent) -> f32 {
    let subtitle = if content.subtitle.is_some() {
        LINE_GAP + ROW_FONT.size
    } else {
        0.0
    };
    2.0 * PADDING + KEYWORD_FONT.size + LINE_GAP + NAME_FONT.size + subtitle
}

fn compartment_height(rows: usize) -> f32 {
    PADDING + COMPARTMENT_TITLE_FONT.size + rows as f32 * (LINE_GAP + ROW_FONT.size) + PADDING
}

pub fn node_size(content: &NodeContent) -> (f32, f32) {
    let mut widest = text_width(&content.keyword, KEYWORD_FONT)
        .max(text_width(&content.title, title_font(content)))
        .max(
            content
                .subtitle
                .as_deref()
                .map_or(0.0, |s| text_width(s, ROW_FONT)),
        );
    for (title, rows) in &content.compartments {
        widest = widest.max(text_width(title, COMPARTMENT_TITLE_FONT));
        for row in rows {
            widest = widest.max(text_width(row, ROW_FONT));
        }
    }
    // Room for the badge on both sides keeps the title centred clear of it.
    let badge = if content.badge.is_some() {
        2.0 * (BADGE_SIZE + BADGE_INSET)
    } else {
        0.0
    };
    let width = (widest + 2.0 * PADDING + badge).max(MIN_NODE_WIDTH).ceil();
    let height = header_height(content)
        + content
            .compartments
            .iter()
            .map(|(_, rows)| compartment_height(rows.len()))
            .sum::<f32>();
    (width, height.ceil())
}

/// A node: square corners for definitions, rounded for usages; the keyword
/// in guillemets above the bold title; then each compartment under a
/// separator, with an italic title and left-aligned rows.
pub fn node(element: &Element, content: &NodeContent, rect: Rect) -> Vec<Item> {
    let depicts = Depicts::Element(element.qualified_name.clone());
    let corner_radius = if content.usage {
        USAGE_CORNER_RADIUS
    } else {
        0.0
    };
    let keyword_baseline = rect.y + PADDING + KEYWORD_FONT.size;
    let mut shapes = vec![
        Shape::Rect {
            rect,
            fill: Fill::Background,
            corner_radius,
            dashed: element.is_reference,
        },
        Shape::Text {
            position: Point::new(rect.center_x(), keyword_baseline),
            text: content.keyword.clone(),
            anchor: TextAnchor::Middle,
            font: KEYWORD_FONT,
            fill: Fill::Stroke,
        },
        Shape::Text {
            position: Point::new(
                rect.center_x(),
                keyword_baseline + LINE_GAP + NAME_FONT.size,
            ),
            text: content.title.clone(),
            anchor: TextAnchor::Middle,
            font: title_font(content),
            fill: Fill::Stroke,
        },
    ];
    if let Some(subtitle) = &content.subtitle {
        shapes.push(Shape::Text {
            position: Point::new(
                rect.center_x(),
                keyword_baseline + 2.0 * LINE_GAP + NAME_FONT.size + ROW_FONT.size,
            ),
            text: subtitle.clone(),
            anchor: TextAnchor::Middle,
            font: ROW_FONT,
            fill: Fill::Stroke,
        });
    }
    if let Some(mark) = content.badge {
        let badge = Rect {
            x: rect.right() - BADGE_INSET - BADGE_SIZE,
            y: rect.y + BADGE_INSET,
            width: BADGE_SIZE,
            height: BADGE_SIZE,
        };
        // Unfilled, so a Style's fill shows through and its text colour applies.
        shapes.push(Shape::Rect {
            rect: badge,
            fill: Fill::None,
            corner_radius: BADGE_SIZE / 2.0,
            dashed: false,
        });
        shapes.push(Shape::Text {
            position: Point::new(
                badge.center_x(),
                badge.y + BADGE_SIZE / 2.0 + LABEL_FONT.size / 3.0,
            ),
            text: mark.to_owned(),
            anchor: TextAnchor::Middle,
            font: LABEL_FONT,
            fill: Fill::Stroke,
        });
    }
    let mut top = rect.y + header_height(content);
    for (title, rows) in &content.compartments {
        shapes.push(Shape::Polyline {
            points: vec![Point::new(rect.x, top), Point::new(rect.right(), top)],
            dashed: false,
        });
        let mut baseline = top + PADDING + COMPARTMENT_TITLE_FONT.size;
        shapes.push(Shape::Text {
            position: Point::new(rect.center_x(), baseline),
            text: (*title).to_owned(),
            anchor: TextAnchor::Middle,
            font: COMPARTMENT_TITLE_FONT,
            fill: Fill::Stroke,
        });
        for row in rows {
            baseline += LINE_GAP + ROW_FONT.size;
            shapes.push(Shape::Text {
                position: Point::new(rect.x + PADDING, baseline),
                text: row.clone(),
                anchor: TextAnchor::Start,
                font: ROW_FONT,
                fill: Fill::Stroke,
            });
        }
        top += compartment_height(rows.len());
    }
    tag(depicts, shapes)
}

/// Specialization: a solid line from the specific node ending in a hollow
/// triangle on the general node.
pub fn specialization(depicts: Depicts, path: Vec<Point>) -> Vec<Item> {
    let (before, end) = last_segment(&path);
    let mut line = path.clone();
    *line.last_mut().expect("a route has two points") = back_from(end, before, ARROW_LENGTH);
    tag(
        depicts,
        vec![
            Shape::Polyline {
                points: line,
                dashed: false,
            },
            triangle(end, before),
        ],
    )
}

/// Defined by: a solid line from the usage ending in a hollow triangle on
/// its definition, with a colon across the line just before the triangle.
pub fn defined_by(depicts: Depicts, path: Vec<Point>) -> Vec<Item> {
    let (before, end) = last_segment(&path);
    let base = back_from(end, before, ARROW_LENGTH);
    let colon = back_from(base, before, COLON_GAP);
    let (nx, ny) = normal(before, end, COLON_HALF_SPREAD);
    let dot = |sign: f32| Shape::Circle {
        center: Point::new(colon.x + sign * nx, colon.y + sign * ny),
        radius: COLON_DOT_RADIUS,
        fill: Fill::Stroke,
    };
    let mut line = path.clone();
    *line.last_mut().expect("a route has two points") = base;
    tag(
        depicts,
        vec![
            Shape::Polyline {
                points: line,
                dashed: false,
            },
            triangle(end, before),
            dot(1.0),
            dot(-1.0),
        ],
    )
}

/// Composition: a solid line from the owner, starting in a filled diamond,
/// to the part's definition, labelled with the `usage`'s name and
/// multiplicity near the definition. Without a usage (a line straight to
/// the part's own node, which names it) there is no label.
pub fn composition(depicts: Depicts, usage: Option<&Element>, path: Vec<Point>) -> Vec<Item> {
    let (start, after) = (path[0], path[1]);
    let diamond_back = back_from(start, after, DIAMOND_LENGTH);
    let middle = back_from(start, after, DIAMOND_LENGTH / 2.0);
    let (nx, ny) = normal(start, after, DIAMOND_HALF_WIDTH);
    let mut label = usage.map(|u| u.name.clone()).unwrap_or_default();
    if let Some(multiplicity) = usage.and_then(|u| u.multiplicity.as_ref()) {
        label.push_str(&format!(" {multiplicity}"));
    }
    let (before, end) = last_segment(&path);
    let near_end = back_from(end, before, 2.0 * LABEL_OFFSET + LABEL_FONT.size);
    let mut line = path.clone();
    line[0] = diamond_back;
    let mut shapes = vec![
        Shape::Polyline {
            points: line,
            dashed: false,
        },
        Shape::Polygon {
            points: vec![
                start,
                Point::new(middle.x + nx, middle.y + ny),
                diamond_back,
                Point::new(middle.x - nx, middle.y - ny),
            ],
            fill: Fill::Stroke,
        },
    ];
    if !label.is_empty() {
        shapes.push(Shape::Text {
            position: Point::new(near_end.x + LABEL_OFFSET, near_end.y),
            text: label,
            anchor: TextAnchor::Start,
            font: LABEL_FONT,
            fill: Fill::Stroke,
        });
    }
    tag(depicts, shapes)
}

/// A connection or interface: a plain solid line between the usages it
/// joins, labelled with its name and type. A straight one is labelled above
/// its middle; a bent one beside its middle bend, on the outer side, so
/// fanned-out parallel connections keep their labels apart.
pub fn connection(depicts: Depicts, element: &Element, path: Vec<Point>) -> Vec<Item> {
    let (position, anchor) = label_place(&path);
    let text = connector_label(element);
    let mut shapes = vec![Shape::Polyline {
        points: path.clone(),
        dashed: false,
    }];
    if element.kind == ElementKind::FlowUsage {
        shapes.push(flow_arrow(&path));
    }
    // An unnamed, untyped connection has no label.
    if !text.is_empty() {
        shapes.push(Shape::Text {
            position,
            text,
            anchor,
            font: LABEL_FONT,
            fill: Fill::Stroke,
        });
    }
    tag(depicts, shapes)
}

/// A Satisfaction Claim: a solid line from the satisfier ending in an open
/// arrowhead on the requirement, labelled `«satisfy»` like a connection.
pub fn satisfy(depicts: Depicts, mark: &str, path: Vec<Point>) -> Vec<Item> {
    labelled_arrow(depicts, &format!("«satisfy» {mark}"), false, path)
}

/// An analysis's subject: a dashed line from the analysis ending in an open
/// arrowhead on the part it analyses, labelled `«subject»`.
pub fn subject(depicts: Depicts, path: Vec<Point>) -> Vec<Item> {
    labelled_arrow(depicts, "«subject»", true, path)
}

/// A line ending in an open arrowhead, labelled beside its middle.
fn labelled_arrow(depicts: Depicts, label: &str, dashed: bool, path: Vec<Point>) -> Vec<Item> {
    // Beside a steep straight edge, so the line doesn't strike through it.
    let (position, anchor) = match path.as_slice() {
        [a, b] if (b.y - a.y).abs() > 2.0 * (b.x - a.x).abs() => (
            Point::new(
                (a.x + b.x) / 2.0 + LABEL_OFFSET,
                (a.y + b.y) / 2.0 + LABEL_FONT.size / 3.0,
            ),
            TextAnchor::Start,
        ),
        _ => label_place(&path),
    };
    let (before, end) = last_segment(&path);
    let base = back_from(end, before, ARROW_LENGTH);
    let (nx, ny) = normal(before, end, ARROW_HALF_WIDTH);
    tag(
        depicts,
        vec![
            Shape::Polyline {
                points: path,
                dashed,
            },
            Shape::Polyline {
                points: vec![
                    Point::new(base.x + nx, base.y + ny),
                    end,
                    Point::new(base.x - nx, base.y - ny),
                ],
                dashed: false,
            },
            Shape::Text {
                position,
                text: label.to_owned(),
                anchor,
                font: LABEL_FONT,
                fill: Fill::Stroke,
            },
        ],
    )
}

/// Where an edge's label goes: above a straight edge's middle, or beside a
/// bent one's middle bend on the outer side, so fanned-out parallel edges
/// keep their labels apart.
fn label_place(path: &[Point]) -> (Point, TextAnchor) {
    let (first, last) = (path[0], path[path.len() - 1]);
    let chord_middle = Point::new((first.x + last.x) / 2.0, (first.y + last.y) / 2.0);
    if path.len() % 2 == 1 {
        let bend = path[path.len() / 2];
        let (dx, dy) = (bend.x - chord_middle.x, bend.y - chord_middle.y);
        if dx.abs() < 0.5 && dy.abs() < 0.5 {
            (
                Point::new(bend.x, bend.y - LABEL_OFFSET),
                TextAnchor::Middle,
            )
        } else if dx.abs() >= dy.abs() {
            let side = dx.signum() * LABEL_OFFSET;
            let anchor = if dx > 0.0 {
                TextAnchor::Start
            } else {
                TextAnchor::End
            };
            (
                Point::new(bend.x + side, bend.y + LABEL_FONT.size / 3.0),
                anchor,
            )
        } else if dy > 0.0 {
            (
                Point::new(bend.x, bend.y + LABEL_OFFSET + LABEL_FONT.size),
                TextAnchor::Middle,
            )
        } else {
            (
                Point::new(bend.x, bend.y - LABEL_OFFSET),
                TextAnchor::Middle,
            )
        }
    } else {
        let segment = (path.len() - 1) / 2;
        let (a, b) = (path[segment], path[segment + 1]);
        (
            Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0 - LABEL_OFFSET),
            TextAnchor::Middle,
        )
    }
}

/// Bends that fan out `count` parallel edges between `a` and `b`: edge
/// `index` bends at the middle, offset sideways from the straight line.
/// The middle edge of an odd count stays straight.
pub fn fan_bends(a: Rect, b: Rect, index: usize, count: usize) -> Vec<Point> {
    let offset = (index as f32 - (count as f32 - 1.0) / 2.0) * PARALLEL_SPREAD;
    if offset == 0.0 {
        return Vec::new();
    }
    let (ca, cb) = (center(a), center(b));
    let (nx, ny) = normal(ca, cb, offset);
    vec![Point::new(
        (ca.x + cb.x) / 2.0 + nx,
        (ca.y + cb.y) / 2.0 + ny,
    )]
}

/// The path of an edge from `from` to `to` through `bends`: it leaves
/// `from`'s border heading for the first bend (or `to`'s center) and enters
/// `to`'s border from the last bend (or `from`'s center).
pub fn route(from: Rect, to: Rect, bends: &[Point]) -> Vec<Point> {
    let first_target = bends.first().copied().unwrap_or(center(to));
    let last_source = bends.last().copied().unwrap_or(center(from));
    let mut path = vec![border_point(from, first_target)];
    path.extend_from_slice(bends);
    path.push(border_point(to, last_source));
    path
}

/// A bend that takes the straight edge from `from` to `to` around the
/// rects in `obstacles` it would otherwise cross. Candidates are points
/// sideways from the middle and points just off each obstacle's corners;
/// the shortest that crosses nothing wins, else the one crossing fewest, as
/// long as it improves on the straight edge. Empty if the straight edge is
/// already clear or nothing improves on it.
pub fn detour(from: Rect, to: Rect, obstacles: &[Rect]) -> Vec<Point> {
    let crossings = |path: &[Point]| {
        obstacles
            .iter()
            .filter(|r| path.windows(2).any(|s| crosses(s[0], s[1], **r)))
            .count()
    };
    let length = |path: &[Point]| path.windows(2).map(|s| distance(s[0], s[1])).sum::<f32>();
    let straight = crossings(&route(from, to, &[]));
    if straight == 0 {
        return Vec::new();
    }

    let (a, b) = (center(from), center(to));
    let middle = Point::new((a.x + b.x) / 2.0, (a.y + b.y) / 2.0);
    let mut candidates = Vec::new();
    for step in 1..=DETOUR_STEPS {
        for side in [1.0, -1.0] {
            let (nx, ny) = normal(a, b, side * step as f32 * DETOUR_STEP);
            candidates.push(Point::new(middle.x + nx, middle.y + ny));
        }
    }
    let off = CORNER_CLEARANCE;
    for r in obstacles {
        candidates.extend([
            Point::new(r.x - off, r.y - off),
            Point::new(r.right() + off, r.y - off),
            Point::new(r.x - off, r.bottom() + off),
            Point::new(r.right() + off, r.bottom() + off),
        ]);
    }
    // Bends inside the two end nodes would make no sense.
    candidates.retain(|c| !from.contains(*c) && !to.contains(*c));

    let best = |options: Vec<Vec<Point>>| {
        options
            .into_iter()
            .map(|bends| {
                let path = route(from, to, &bends);
                (crossings(&path), length(&path), bends)
            })
            .filter(|(count, _, _)| *count < straight)
            .min_by(|x, y| x.0.cmp(&y.0).then(x.1.total_cmp(&y.1)))
    };
    let single = best(candidates.iter().map(|c| vec![*c]).collect());
    if let Some((0, _, bends)) = single {
        return bends;
    }
    // A pocket can need two bends: out of it, then around what blocks it.
    let pairs = candidates
        .iter()
        .flat_map(|a| {
            candidates
                .iter()
                .filter(move |b| *b != a)
                .map(move |b| vec![*a, *b])
        })
        .collect();
    match (single, best(pairs)) {
        (_, Some((0, _, bends))) => bends,
        (Some((_, _, bends)), _) => bends,
        (None, Some((_, _, bends))) => bends,
        (None, None) => Vec::new(),
    }
}

fn distance(a: Point, b: Point) -> f32 {
    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt()
}

/// Whether the segment from `a` to `b` passes through `rect`, kept a few
/// units clear of its border (Liang–Barsky clipping).
fn crosses(a: Point, b: Point, rect: Rect) -> bool {
    let (x0, y0) = (rect.x - OBSTACLE_CLEARANCE, rect.y - OBSTACLE_CLEARANCE);
    let (x1, y1) = (
        rect.right() + OBSTACLE_CLEARANCE,
        rect.bottom() + OBSTACLE_CLEARANCE,
    );
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let (mut t0, mut t1) = (0.0_f32, 1.0_f32);
    for (p, q) in [
        (-dx, a.x - x0),
        (dx, x1 - a.x),
        (-dy, a.y - y0),
        (dy, y1 - a.y),
    ] {
        if p == 0.0 {
            if q < 0.0 {
                return false;
            }
        } else {
            let t = q / p;
            if p < 0.0 {
                t0 = t0.max(t);
            } else {
                t1 = t1.min(t);
            }
            if t0 > t1 {
                return false;
            }
        }
    }
    true
}

/// Spreads edge ends that land close together on the same side of `rect`,
/// so their markers don't stack. `ends` are the points to adjust; each is
/// moved along its side only.
pub fn spread_ends(rect: Rect, ends: &mut [&mut Point]) {
    let side_of = |p: &Point| {
        if (p.y - rect.y).abs() < 0.5 {
            0
        } else if (p.y - rect.bottom()).abs() < 0.5 {
            1
        } else if (p.x - rect.x).abs() < 0.5 {
            2
        } else {
            3
        }
    };
    for side in 0..4 {
        let horizontal = side < 2;
        let along = |p: &Point| if horizontal { p.x } else { p.y };
        let (low, high) = if horizontal {
            (rect.x, rect.right())
        } else {
            (rect.y, rect.bottom())
        };
        let mut on_side: Vec<&mut &mut Point> =
            ends.iter_mut().filter(|p| side_of(p) == side).collect();
        on_side.sort_by(|a, b| along(a).total_cmp(&along(b)));
        for i in 1..on_side.len() {
            let previous = along(on_side[i - 1]);
            let point = &mut on_side[i];
            if along(point) - previous < END_SEPARATION {
                let moved = (previous + END_SEPARATION).min(high).max(low);
                if horizontal {
                    point.x = moved
                } else {
                    point.y = moved
                }
            }
        }
    }
}

fn last_segment(path: &[Point]) -> (Point, Point) {
    (path[path.len() - 2], path[path.len() - 1])
}

fn tag(depicts: Depicts, shapes: Vec<Shape>) -> Vec<Item> {
    shapes
        .into_iter()
        .map(|shape| Item {
            depicts: depicts.clone(),
            shape,
        })
        .collect()
}

fn center(rect: Rect) -> Point {
    Point::new(rect.center_x(), rect.y + rect.height / 2.0)
}

/// Where the ray from the center of `rect` toward `toward` crosses its
/// border. The coordinate on the crossed side is exact, so edges meet
/// borders precisely.
fn border_point(rect: Rect, toward: Point) -> Point {
    let c = center(rect);
    let (dx, dy) = (toward.x - c.x, toward.y - c.y);
    let (half_w, half_h) = (rect.width / 2.0, rect.height / 2.0);
    if dx == 0.0 && dy == 0.0 {
        return c;
    }
    if dx.abs() * half_h <= dy.abs() * half_w {
        let y = if dy > 0.0 { rect.bottom() } else { rect.y };
        Point::new(c.x + dx * (half_h / dy.abs()), y)
    } else {
        let x = if dx > 0.0 { rect.right() } else { rect.x };
        Point::new(x, c.y + dy * (half_w / dx.abs()))
    }
}

/// The point `distance` back from `tip` toward `from`.
fn back_from(tip: Point, from: Point, distance: f32) -> Point {
    let (dx, dy) = (from.x - tip.x, from.y - tip.y);
    let length = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    Point::new(
        tip.x + dx / length * distance,
        tip.y + dy / length * distance,
    )
}

/// A vector of `length` perpendicular to the line from `a` to `b`.
fn normal(a: Point, b: Point, length: f32) -> (f32, f32) {
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let norm = (dx * dx + dy * dy).sqrt().max(f32::EPSILON);
    (-dy / norm * length, dx / norm * length)
}

/// A hollow triangle with its tip at `tip`, pointing away from `from`.
fn triangle(tip: Point, from: Point) -> Shape {
    let base = back_from(tip, from, ARROW_LENGTH);
    let (nx, ny) = normal(from, tip, ARROW_HALF_WIDTH);
    Shape::Polygon {
        points: vec![
            tip,
            Point::new(base.x + nx, base.y + ny),
            Point::new(base.x - nx, base.y - ny),
        ],
        fill: Fill::Background,
    }
}
