//! Styles: Style Rules that color node fills, layered on the Standard
//! Notation and applied to a built scene, so they never move anything.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::diagram_file::FOLDER;
use crate::model::{Element, Model, QualifiedName};
use crate::scene::{Depicts, Fill, Rect, Rgb, Scene, Shape};

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
}

impl Condition {
    /// How strongly the condition singles an element out: metadata over
    /// package over kind.
    fn rank(&self) -> u8 {
        match self {
            Condition::Kind(_) => 0,
            Condition::Package(_) => 1,
            Condition::Metadata(_) => 2,
        }
    }

    pub fn matches(&self, element: &Element) -> bool {
        match self {
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
        }
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
    Unreadable { path: PathBuf, source: std::io::Error },
    #[error("invalid Style file {path}: {source}")]
    Invalid { path: PathBuf, source: serde_json::Error },
    #[error("Style file {path} has format version {found}, expected {FORMAT_VERSION}")]
    UnsupportedVersion { path: PathBuf, found: u32 },
}

impl Style {
    pub fn new(rules: Vec<StyleRule>) -> Self {
        Self { format_version: FORMAT_VERSION, rules }
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
    let style: Style = serde_json::from_str(&text).map_err(|source| StyleError::Invalid { path: path.clone(), source })?;
    if style.format_version != FORMAT_VERSION {
        return Err(StyleError::UnsupportedVersion { path, found: style.format_version });
    }
    Ok(Some(style))
}

/// The rule that colors `element`, from rule `levels` checked in order (a
/// Diagram's own rules, then the Model's): the first level with a match
/// decides, and within it the most specific rule wins. Among package rules
/// the deepest package wins; otherwise the first listed.
pub fn rule_for<'a>(element: &Element, levels: &[&'a [StyleRule]]) -> Option<&'a StyleRule> {
    levels.iter().find_map(|rules| {
        let depth = |rule: &StyleRule| match &rule.condition {
            Condition::Package(package) => package.as_str().split("::").count(),
            _ => 0,
        };
        let mut best: Option<&StyleRule> = None;
        for rule in rules.iter().filter(|r| r.condition.matches(element)) {
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

/// Colors the scene's nodes by the rule `levels` (see [`rule_for`]): each
/// node's own rect takes its rule's fill, and text inside that rect turns
/// white when the fill is dark. Returns the rules that colored something,
/// in the order they were first used.
pub fn apply<'a>(scene: &mut Scene, model: &Model, levels: &[&'a [StyleRule]]) -> Vec<&'a StyleRule> {
    let mut used: Vec<&StyleRule> = Vec::new();
    // Each node: what it depicts, its rect, and its fill.
    let mut nodes: Vec<(Depicts, Rect, Rgb)> = Vec::new();
    let mut seen: Vec<&Depicts> = Vec::new();
    for item in &mut scene.items {
        let (Some(element), Shape::Rect { rect, fill, .. }) = (node_element(&item.depicts), &mut item.shape) else {
            continue;
        };
        // Only a node's first rect is its box.
        if seen.contains(&&item.depicts) {
            continue;
        }
        seen.push(&item.depicts);
        let Some(rule) = model.element(element).and_then(|e| rule_for(e, levels)) else { continue };
        *fill = Fill::Color(rule.fill);
        nodes.push((item.depicts.clone(), *rect, rule.fill));
        if !used.iter().any(|u| std::ptr::eq(*u, rule)) {
            used.push(rule);
        }
    }
    for item in &mut scene.items {
        let Shape::Text { position, fill, .. } = &mut item.shape else { continue };
        let dark = nodes
            .iter()
            .any(|(depicts, rect, color)| *depicts == item.depicts && rect.contains(*position) && color.is_dark());
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
