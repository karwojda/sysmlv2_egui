//! The Model as the rest of the app sees it: a read-only snapshot of
//! elements and diagnostics, independent of the backend that produced it.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

/// A `::`-separated element path such as `Vehicles::Car`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct QualifiedName(String);

impl QualifiedName {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The last segment, e.g. `Car` for `Vehicles::Car`.
    pub fn simple_name(&self) -> &str {
        self.0.rsplit("::").next().unwrap_or(&self.0)
    }

    /// The qualified name of the enclosing namespace, if any.
    pub fn parent(&self) -> Option<QualifiedName> {
        self.0.rsplit_once("::").map(|(parent, _)| QualifiedName::new(parent))
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<&str> for QualifiedName {
    fn from(name: &str) -> Self {
        Self::new(name)
    }
}

/// The kinds of element this tool understands. Anything else is an
/// Unsupported Element, which keeps the SysML keyword it was declared with.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ElementKind {
    Package,
    PartDefinition,
    Unsupported(String),
}

impl ElementKind {
    /// The SysML keyword, as shown in the Standard Notation and the Browser.
    pub fn keyword(&self) -> &str {
        match self {
            ElementKind::Package => "package",
            ElementKind::PartDefinition => "part def",
            ElementKind::Unsupported(keyword) => keyword,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SourceLocation {
    pub file: PathBuf,
    pub line: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Element {
    pub qualified_name: QualifiedName,
    pub name: String,
    pub kind: ElementKind,
    pub owner: Option<QualifiedName>,
    /// Elements this one specializes (`:>`), by qualified name.
    pub specializes: Vec<QualifiedName>,
    pub source: Option<SourceLocation>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub location: Option<SourceLocation>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Model {
    elements: BTreeMap<QualifiedName, Element>,
    diagnostics: Vec<Diagnostic>,
}

impl Model {
    pub fn new(elements: impl IntoIterator<Item = Element>, diagnostics: Vec<Diagnostic>) -> Self {
        Self {
            elements: elements
                .into_iter()
                .map(|e| (e.qualified_name.clone(), e))
                .collect(),
            diagnostics,
        }
    }

    pub fn builder() -> ModelBuilder {
        ModelBuilder::default()
    }

    pub fn element(&self, name: &QualifiedName) -> Option<&Element> {
        self.elements.get(name)
    }

    /// All elements, ordered by qualified name.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.elements.values()
    }

    /// Elements directly owned by `owner`, ordered by qualified name.
    pub fn owned_members<'a>(&'a self, owner: &'a QualifiedName) -> impl Iterator<Item = &'a Element> {
        self.elements
            .values()
            .filter(move |e| e.owner.as_ref() == Some(owner))
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics.iter().any(|d| d.severity == Severity::Error)
    }
}

/// Builds a Model by hand, for tests and the fake backend. Owners and names
/// are derived from qualified names.
#[derive(Default)]
pub struct ModelBuilder {
    elements: Vec<Element>,
    diagnostics: Vec<Diagnostic>,
}

impl ModelBuilder {
    pub fn package(self, name: &str) -> Self {
        self.element(name, ElementKind::Package)
    }

    pub fn part_def(self, name: &str) -> Self {
        self.element(name, ElementKind::PartDefinition)
    }

    pub fn unsupported(self, name: &str, keyword: &str) -> Self {
        self.element(name, ElementKind::Unsupported(keyword.to_owned()))
    }

    pub fn element(mut self, name: &str, kind: ElementKind) -> Self {
        let qualified_name = QualifiedName::new(name);
        self.elements.push(Element {
            name: qualified_name.simple_name().to_owned(),
            owner: qualified_name.parent(),
            qualified_name,
            kind,
            specializes: Vec::new(),
            source: None,
        });
        self
    }

    /// Makes the most recently added element specialize `general`.
    pub fn specializes(mut self, general: &str) -> Self {
        self.last().specializes.push(QualifiedName::new(general));
        self
    }

    /// Sets the source location of the most recently added element.
    pub fn at(mut self, file: &str, line: u32) -> Self {
        self.last().source = Some(SourceLocation { file: file.into(), line });
        self
    }

    pub fn diagnostic(mut self, severity: Severity, message: &str) -> Self {
        self.diagnostics.push(Diagnostic {
            severity,
            message: message.to_owned(),
            location: None,
        });
        self
    }

    pub fn build(self) -> Model {
        Model::new(self.elements, self.diagnostics)
    }

    fn last(&mut self) -> &mut Element {
        self.elements
            .last_mut()
            .expect("add an element before setting its properties")
    }
}
