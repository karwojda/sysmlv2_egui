//! Styles: Style Rules that color node fills, layered on the Standard
//! Notation and applied to a built scene, so they never move anything.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::diagram_file::{DiagramStyle, FOLDER};
use crate::model::{Element, Model, QualifiedName};
use crate::notation::{LABEL_FONT, text_width};
use crate::satisfaction::{self, SatisfactionStatus};
use crate::scene::{Depicts, Fill, Item, Point, Rect, Rgb, Scene, Shape, TextAnchor};

/// Legend geometry.
const LEGEND_GAP: f32 = 16.0;
const LEGEND_PADDING: f32 = 8.0;
const LEGEND_ROW: f32 = 18.0;
const SWATCH: f32 = 12.0;
/// Room kept around the nodes, as the builders keep.
const PAGE_MARGIN: f32 = 20.0;

/// The Model Style's file name, inside the Diagram File folder.
const FILE_NAME: &str = "style.json";
const FORMAT_VERSION: u32 = 1;

/// What a Style Rule matches.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Condition {
    /// Elements of a kind, by its keyword (`part def`).
    Kind(String),
    /// Elements anywhere inside a package.
    Package(QualifiedName),
    /// Elements annotated with a metadata def.
    Metadata(QualifiedName),
    /// Requirement usages with a Satisfaction Status.
    Satisfaction(Satisfaction),
}

/// The Satisfaction Status a satisfaction rule matches, as written in the
/// Style file.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Satisfaction {
    Satisfied,
    Violated,
    Undecided,
    NotClaimed,
}

impl Satisfaction {
    pub const ALL: [Satisfaction; 4] = [
        Satisfaction::Satisfied,
        Satisfaction::Violated,
        Satisfaction::Undecided,
        Satisfaction::NotClaimed,
    ];

    pub fn status(self) -> SatisfactionStatus {
        match self {
            Satisfaction::Satisfied => SatisfactionStatus::Satisfied,
            Satisfaction::Violated => SatisfactionStatus::Violated,
            Satisfaction::Undecided => SatisfactionStatus::Undecided,
            Satisfaction::NotClaimed => SatisfactionStatus::NotClaimed,
        }
    }
}

impl Condition {
    /// How strongly the condition singles an element out: satisfaction
    /// over metadata over package over kind.
    fn rank(&self) -> u8 {
        match self {
            Condition::Kind(_) => 0,
            Condition::Package(_) => 1,
            Condition::Metadata(_) => 2,
            Condition::Satisfaction(_) => 3,
        }
    }

    pub fn matches(&self, model: &Model, element: &Element) -> bool {
        match self {
            Condition::Satisfaction(wanted) => {
                satisfaction::status(model, &element.qualified_name) == Some(wanted.status())
            }
            Condition::Kind(keyword) => element.kind.keyword() == keyword,
            Condition::Package(package) => element
                .qualified_name
                .as_str()
                .strip_prefix(package.as_str())
                .is_some_and(|rest| rest.starts_with("::")),
            Condition::Metadata(definition) => element.metadata.contains(definition),
        }
    }

    /// How the condition reads in a Legend: `«part def»`, `Powertrain`,
    /// `@Safety`.
    pub fn describe(&self) -> String {
        match self {
            Condition::Kind(keyword) => format!("«{keyword}»"),
            Condition::Package(package) => package.simple_name().to_owned(),
            Condition::Metadata(definition) => format!("@{}", definition.simple_name()),
            Condition::Satisfaction(wanted) => wanted.status().label().to_owned(),
        }
    }
}

/// The kinds a kind rule can name: those drawn as nodes.
pub const KINDS: [&str; 12] = [
    "part def",
    "part",
    "port def",
    "port",
    "attribute def",
    "attribute",
    "connection def",
    "interface def",
    "view def",
    "view",
    "requirement def",
    "requirement",
];

impl Condition {
    /// Whether what the condition names is in the Model; a rule whose
    /// package or metadata def is gone never matches, but is kept.
    pub fn exists_in(&self, model: &Model) -> bool {
        match self {
            Condition::Kind(keyword) => KINDS.contains(&keyword.as_str()),
            Condition::Package(name) | Condition::Metadata(name) => model.element(name).is_some(),
            Condition::Satisfaction(_) => true,
        }
    }
}

/// Moves rule `index` one place earlier; the first stays put.
pub fn move_up(rules: &mut [StyleRule], index: usize) {
    if index > 0 && index < rules.len() {
        rules.swap(index - 1, index);
    }
}

/// Moves rule `index` one place later; the last stays put.
pub fn move_down(rules: &mut [StyleRule], index: usize) {
    if index + 1 < rules.len() {
        rules.swap(index, index + 1);
    }
}

/// The rule a new row starts as: every part def, in light blue.
pub fn new_rule() -> StyleRule {
    StyleRule {
        condition: Condition::Kind("part def".to_owned()),
        fill: Rgb(0xdb, 0xe8, 0xf6),
        label: None,
    }
}

/// One condition and the fill color it gives, with an optional Legend label.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StyleRule {
    #[serde(flatten)]
    pub condition: Condition,
    pub fill: Rgb,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// The Model Style, as stored in its file.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Style {
    pub format_version: u32,
    pub rules: Vec<StyleRule>,
}

#[derive(Debug, thiserror::Error)]
pub enum StyleError {
    #[error("cannot read {path}: {source}")]
    Unreadable {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid Style file {path}: {source}")]
    Invalid {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("Style file {path} has format version {found}, expected {FORMAT_VERSION}")]
    UnsupportedVersion { path: PathBuf, found: u32 },
}

impl Style {
    pub fn new(rules: Vec<StyleRule>) -> Self {
        Self {
            format_version: FORMAT_VERSION,
            rules,
        }
    }

    pub fn to_text(&self) -> String {
        let mut text = serde_json::to_string_pretty(self).expect("a Style always serializes");
        text.push('\n');
        text
    }
}

/// Where the Model Style of `model_folder` lives.
pub fn path_for(model_folder: &Path) -> PathBuf {
    model_folder.join(FOLDER).join(FILE_NAME)
}

/// The Model Style of `model_folder`, or `None` if it has none.
pub fn load(model_folder: &Path) -> Result<Option<Style>, StyleError> {
    let path = path_for(model_folder);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(source) => return Err(StyleError::Unreadable { path, source }),
    };
    let style: Style = serde_json::from_str(&text).map_err(|source| StyleError::Invalid {
        path: path.clone(),
        source,
    })?;
    if style.format_version != FORMAT_VERSION {
        return Err(StyleError::UnsupportedVersion {
            path,
            found: style.format_version,
        });
    }
    Ok(Some(style))
}

/// The rule that colors `element`, from rule `levels` checked in order (a
/// Diagram's own rules, then the Model's): the first level with a match
/// decides, and within it the most specific rule wins. Among package rules
/// the deepest package wins; otherwise the first listed.
pub fn rule_for<'a>(
    model: &Model,
    element: &Element,
    levels: &[&'a [StyleRule]],
) -> Option<&'a StyleRule> {
    levels.iter().find_map(|rules| {
        let depth = |rule: &StyleRule| match &rule.condition {
            Condition::Package(package) => package.as_str().split("::").count(),
            _ => 0,
        };
        let mut best: Option<&StyleRule> = None;
        for rule in rules.iter().filter(|r| r.condition.matches(model, element)) {
            let better = best.is_none_or(|b| {
                (rule.condition.rank(), depth(rule)) > (b.condition.rank(), depth(b))
            });
            if better {
                best = Some(rule);
            }
        }
        best
    })
}

/// Colors the scene by a Diagram's Style layered over the Model Style
/// (see [`apply`]), returning the rules used.
pub fn apply_styles<'a>(
    scene: &mut Scene,
    model: &Model,
    model_style: Option<&'a Style>,
    diagram: &'a DiagramStyle,
) -> Vec<&'a StyleRule> {
    let mut levels: Vec<&[StyleRule]> = vec![&diagram.rules];
    if let Some(style) = model_style.filter(|_| diagram.model_style) {
        levels.push(&style.rules);
    }
    let used = apply(scene, model, &levels);
    // In rule order, the Diagram's rules first.
    let used: Vec<&StyleRule> = levels
        .iter()
        .flat_map(|rules| rules.iter())
        .filter(|r| used.iter().any(|u| std::ptr::eq(*u, *r)))
        .collect();
    if diagram.legend {
        legend(scene, &used);
    }
    used
}

/// How a rule reads in the Legend: its label, else its condition.
pub fn legend_label(rule: &StyleRule) -> String {
    rule.label
        .clone()
        .unwrap_or_else(|| rule.condition.describe())
}

/// Draws a Legend of `rules` below the scene's content, aligned to its
/// right edge, and grows the page to hold it. No rules, no Legend.
pub fn legend(scene: &mut Scene, rules: &[&StyleRule]) {
    if rules.is_empty() {
        return;
    }
    let labels: Vec<String> = rules.iter().map(|r| legend_label(r)).collect();
    let widest = labels
        .iter()
        .map(|l| text_width(l, LABEL_FONT))
        .fold(0.0, f32::max);
    let width = 2.0 * LEGEND_PADDING + SWATCH + LEGEND_PADDING + widest;
    let height = 2.0 * LEGEND_PADDING + LEGEND_ROW * rules.len() as f32 - (LEGEND_ROW - SWATCH);
    let right = scene.x + scene.width - PAGE_MARGIN;
    let top = scene.y + scene.height - PAGE_MARGIN + LEGEND_GAP;
    let left = right - width;

    let mut push = |shape| {
        scene.items.push(Item {
            depicts: Depicts::Legend,
            shape,
        })
    };
    push(Shape::Rect {
        rect: Rect {
            x: left,
            y: top,
            width,
            height,
        },
        fill: Fill::Background,
        corner_radius: 0.0,
        dashed: false,
    });
    for (i, (rule, label)) in rules.iter().zip(labels).enumerate() {
        let row_top = top + LEGEND_PADDING + i as f32 * LEGEND_ROW;
        push(Shape::Rect {
            rect: Rect {
                x: left + LEGEND_PADDING,
                y: row_top,
                width: SWATCH,
                height: SWATCH,
            },
            fill: Fill::Color(rule.fill),
            corner_radius: 0.0,
            dashed: false,
        });
        push(Shape::Text {
            position: Point::new(left + 2.0 * LEGEND_PADDING + SWATCH, row_top + SWATCH - 1.5),
            text: label,
            anchor: TextAnchor::Start,
            font: LABEL_FONT,
            fill: Fill::Stroke,
        });
    }
    let new_left = scene.x.min((left - PAGE_MARGIN).floor());
    let new_bottom = top + height + PAGE_MARGIN;
    scene.width += scene.x - new_left;
    scene.x = new_left;
    scene.height = (new_bottom - scene.y).ceil().max(scene.height);
}

/// Colors the scene's nodes by the rule `levels` (see [`rule_for`]): each
/// node's own rect takes its rule's fill, and text inside that rect turns
/// white when the fill is dark. Returns the rules that colored something,
/// in the order they were first used.
pub fn apply<'a>(
    scene: &mut Scene,
    model: &Model,
    levels: &[&'a [StyleRule]],
) -> Vec<&'a StyleRule> {
    let mut used: Vec<&StyleRule> = Vec::new();
    // Each node: what it depicts, its rect, and its fill.
    let mut nodes: Vec<(Depicts, Rect, Rgb)> = Vec::new();
    let mut seen: Vec<&Depicts> = Vec::new();
    for item in &mut scene.items {
        let (Some(element), Shape::Rect { rect, fill, .. }) =
            (node_element(&item.depicts), &mut item.shape)
        else {
            continue;
        };
        // Only a node's first rect is its box.
        if seen.contains(&&item.depicts) {
            continue;
        }
        seen.push(&item.depicts);
        let Some(rule) = model
            .element(element)
            .and_then(|e| rule_for(model, e, levels))
        else {
            continue;
        };
        *fill = Fill::Color(rule.fill);
        nodes.push((item.depicts.clone(), *rect, rule.fill));
        if !used.iter().any(|u| std::ptr::eq(*u, rule)) {
            used.push(rule);
        }
    }
    for item in &mut scene.items {
        let Shape::Text { position, fill, .. } = &mut item.shape else {
            continue;
        };
        let dark = nodes.iter().any(|(depicts, rect, color)| {
            *depicts == item.depicts && rect.contains(*position) && color.is_dark()
        });
        if dark {
            *fill = Fill::Background;
        }
    }
    used
}

/// The element a node item stands for: a General node, an Interconnection
/// part, or a port.
fn node_element(depicts: &Depicts) -> Option<&QualifiedName> {
    match depicts {
        Depicts::Element(element) | Depicts::Part { element, .. } => Some(element),
        Depicts::Port { port, .. } => Some(port),
        _ => None,
    }
}
