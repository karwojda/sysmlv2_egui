//! The Model as the rest of the app sees it: a read-only snapshot of
//! elements and diagnostics, independent of the backend that produced it.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

/// A `::`-separated element path such as `Vehicles::Car`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize)]
#[serde(transparent)]
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
    PartUsage,
    AttributeDefinition,
    AttributeUsage,
    PortDefinition,
    PortUsage,
    ConnectionDefinition,
    ConnectionUsage,
    InterfaceDefinition,
    InterfaceUsage,
    Unsupported(String),
}

impl ElementKind {
    /// The SysML keyword, as shown in the Standard Notation and the Browser.
    pub fn keyword(&self) -> &str {
        match self {
            ElementKind::Package => "package",
            ElementKind::PartDefinition => "part def",
            ElementKind::PartUsage => "part",
            ElementKind::AttributeDefinition => "attribute def",
            ElementKind::AttributeUsage => "attribute",
            ElementKind::PortDefinition => "port def",
            ElementKind::PortUsage => "port",
            ElementKind::ConnectionDefinition => "connection def",
            ElementKind::ConnectionUsage => "connection",
            ElementKind::InterfaceDefinition => "interface def",
            ElementKind::InterfaceUsage => "interface",
            ElementKind::Unsupported(keyword) => keyword,
        }
    }

    pub fn is_definition(&self) -> bool {
        matches!(
            self,
            ElementKind::PartDefinition
                | ElementKind::AttributeDefinition
                | ElementKind::PortDefinition
                | ElementKind::ConnectionDefinition
                | ElementKind::InterfaceDefinition
        )
    }

    /// Connection and interface usages, which join other usages.
    pub fn is_connection(&self) -> bool {
        matches!(self, ElementKind::ConnectionUsage | ElementKind::InterfaceUsage)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    In,
    Out,
    InOut,
}

impl Direction {
    pub fn keyword(self) -> &'static str {
        match self {
            Direction::In => "in",
            Direction::Out => "out",
            Direction::InOut => "inout",
        }
    }
}

/// Declared bounds, as written: `lower` may be a number, `upper` a number
/// or `*`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Multiplicity {
    pub lower: String,
    pub upper: String,
}

impl fmt::Display for Multiplicity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.lower == self.upper {
            write!(f, "[{}]", self.upper)
        } else {
            write!(f, "[{}..{}]", self.lower, self.upper)
        }
    }
}

/// One end of a connection: the chain of features it reaches, such as
/// `[Vehicle::tanks, Tank::fuelOut]` for `tanks.fuelOut`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectionEnd {
    pub path: Vec<QualifiedName>,
}

impl fmt::Display for ConnectionEnd {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self.path.iter().map(QualifiedName::simple_name).collect();
        f.write_str(&names.join("."))
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
    /// The definition a usage is typed by (`:`).
    pub typed_by: Option<QualifiedName>,
    /// Whether a port is typed by the conjugate of its definition (`~`).
    pub conjugated: bool,
    pub multiplicity: Option<Multiplicity>,
    pub direction: Option<Direction>,
    /// The ends a connection or interface usage joins, in order.
    pub ends: Vec<ConnectionEnd>,
    pub documentation: Option<String>,
    pub source: Option<SourceLocation>,
}

impl Element {
    /// A plain element of `kind`; its name and owner come from the
    /// qualified name and every other property is empty.
    pub fn new(qualified_name: QualifiedName, kind: ElementKind) -> Self {
        Self {
            name: qualified_name.simple_name().to_owned(),
            owner: qualified_name.parent(),
            qualified_name,
            kind,
            specializes: Vec::new(),
            typed_by: None,
            conjugated: false,
            multiplicity: None,
            direction: None,
            ends: Vec::new(),
            documentation: None,
            source: None,
        }
    }
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
/// are derived from qualified names; property setters apply to the most
/// recently added element.
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

    pub fn part(self, name: &str) -> Self {
        self.element(name, ElementKind::PartUsage)
    }

    pub fn attribute_def(self, name: &str) -> Self {
        self.element(name, ElementKind::AttributeDefinition)
    }

    pub fn attribute(self, name: &str) -> Self {
        self.element(name, ElementKind::AttributeUsage)
    }

    pub fn port_def(self, name: &str) -> Self {
        self.element(name, ElementKind::PortDefinition)
    }

    pub fn port(self, name: &str) -> Self {
        self.element(name, ElementKind::PortUsage)
    }

    pub fn connection_def(self, name: &str) -> Self {
        self.element(name, ElementKind::ConnectionDefinition)
    }

    pub fn connection(self, name: &str) -> Self {
        self.element(name, ElementKind::ConnectionUsage)
    }

    pub fn interface_def(self, name: &str) -> Self {
        self.element(name, ElementKind::InterfaceDefinition)
    }

    pub fn interface(self, name: &str) -> Self {
        self.element(name, ElementKind::InterfaceUsage)
    }

    pub fn unsupported(self, name: &str, keyword: &str) -> Self {
        self.element(name, ElementKind::Unsupported(keyword.to_owned()))
    }

    pub fn element(mut self, name: &str, kind: ElementKind) -> Self {
        self.elements.push(Element::new(QualifiedName::new(name), kind));
        self
    }

    pub fn specializes(mut self, general: &str) -> Self {
        self.last().specializes.push(QualifiedName::new(general));
        self
    }

    pub fn typed_by(mut self, definition: &str) -> Self {
        self.last().typed_by = Some(QualifiedName::new(definition));
        self
    }

    pub fn conjugated(mut self) -> Self {
        self.last().conjugated = true;
        self
    }

    pub fn multiplicity(mut self, lower: &str, upper: &str) -> Self {
        self.last().multiplicity = Some(Multiplicity { lower: lower.to_owned(), upper: upper.to_owned() });
        self
    }

    pub fn direction(mut self, direction: Direction) -> Self {
        self.last().direction = Some(direction);
        self
    }

    /// Adds an end reaching the given feature chain, e.g.
    /// `&["Vehicle::tanks", "Tank::fuelOut"]`.
    pub fn end(mut self, path: &[&str]) -> Self {
        let path = path.iter().map(|p| QualifiedName::new(*p)).collect();
        self.last().ends.push(ConnectionEnd { path });
        self
    }

    pub fn doc(mut self, text: &str) -> Self {
        self.last().documentation = Some(text.to_owned());
        self
    }

    pub fn at(mut self, file: impl Into<PathBuf>, line: u32) -> Self {
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
