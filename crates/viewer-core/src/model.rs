//! The Model as the rest of the app sees it: a read-only snapshot of
//! elements and diagnostics, independent of the backend that produced it.

use std::collections::BTreeMap;
use std::fmt;
use std::path::PathBuf;

/// A `::`-separated element path such as `Vehicles::Car`.
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
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
        self.0
            .rsplit_once("::")
            .map(|(parent, _)| QualifiedName::new(parent))
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
    FlowUsage,
    BindingConnection,
    ViewDefinition,
    ViewUsage,
    RequirementDefinition,
    RequirementUsage,
    /// A Satisfaction Claim: `satisfy requirement by satisfier`.
    SatisfyRequirement,
    ConstraintDefinition,
    ConstraintUsage,
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
            ElementKind::FlowUsage => "flow",
            ElementKind::BindingConnection => "bind",
            ElementKind::ViewDefinition => "view def",
            ElementKind::ViewUsage => "view",
            ElementKind::RequirementDefinition => "requirement def",
            ElementKind::RequirementUsage => "requirement",
            ElementKind::SatisfyRequirement => "satisfy",
            ElementKind::ConstraintDefinition => "constraint def",
            ElementKind::ConstraintUsage => "constraint",
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
                | ElementKind::RequirementDefinition
                | ElementKind::ConstraintDefinition
        )
    }

    /// Connections, interfaces, flows and bindings: elements with ends,
    /// drawn as connectors between the usages they join.
    pub fn is_connection(&self) -> bool {
        matches!(
            self,
            ElementKind::ConnectionUsage
                | ElementKind::InterfaceUsage
                | ElementKind::FlowUsage
                | ElementKind::BindingConnection
        )
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

/// What checking a Satisfaction Claim against the Model's values found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimResult {
    Holds,
    /// The condition, as the service wrote it, evaluated to false.
    Violated {
        condition: String,
    },
    /// The claim couldn't be evaluated, for example a value is missing.
    Undecided {
        reason: String,
    },
    /// The claim was never checked: the service couldn't check at all.
    Unchecked {
        reason: String,
    },
}

/// What checking an asserted constraint on one part found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstraintCheck {
    /// The part the constraint was checked on: the one that owns it, or one
    /// that inherits it from its definition.
    pub part: QualifiedName,
    pub result: ClaimResult,
}

/// How much of its target an `expose` shows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExposeScope {
    /// `expose X`: X itself.
    Element,
    /// `expose X::*`: X's members.
    Members,
    /// `expose X::**`: X and everything beneath it.
    ElementAndDescendants,
    /// `expose X::*::**`: everything beneath X, but not X.
    Descendants,
}

/// One `expose` of a view: an element and how much of it is shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Expose {
    pub target: QualifiedName,
    pub scope: ExposeScope,
}

impl fmt::Display for Expose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = match self.scope {
            ExposeScope::Element => "",
            ExposeScope::Members => "::*",
            ExposeScope::ElementAndDescendants => "::**",
            ExposeScope::Descendants => "::*::**",
        };
        write!(f, "{}{suffix}", self.target)
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
    /// The ends a connection-like usage joins, in order (for a flow, from
    /// then to).
    pub ends: Vec<ConnectionEnd>,
    /// Whether a usage is a reference (`ref part`) rather than owned.
    pub is_reference: bool,
    /// What a flow carries: its item type.
    pub payload: Option<QualifiedName>,
    /// What a view shows, in order.
    pub exposes: Vec<Expose>,
    /// The library rendering a view or view definition names with
    /// `render`, such as `Views::asTreeDiagram`.
    pub rendering: Option<QualifiedName>,
    /// The metadata defs the element is annotated with (`@Safety`,
    /// `#Safety`, `metadata Safety`), in order.
    pub metadata: Vec<QualifiedName>,
    /// The short id written in angle brackets (`<'1'> massReq`).
    pub short_name: Option<String>,
    /// A requirement's subject: the owned feature it constrains.
    pub subject: Option<QualifiedName>,
    /// A requirement's own `require` constraints, as text, in order.
    pub require: Vec<String>,
    /// A feature's value, as text: `180.0`, `engine.mass + transmission.mass`,
    /// or `default 5` for a default.
    pub value: Option<String>,
    /// Whether a constraint usage is asserted (`assert constraint`).
    pub is_asserted: bool,
    /// A constraint's own expression, as text (`fuelMass <= fuelMassMax`).
    pub expression: Option<String>,
    /// For an asserted constraint, what checking it on each part found.
    pub checks: Vec<ConstraintCheck>,
    /// For a Satisfaction Claim, the requirement usage it claims is
    /// satisfied.
    pub satisfies: Option<QualifiedName>,
    /// For a Satisfaction Claim, the part (or other usage or definition)
    /// that satisfies it.
    pub satisfier: Option<QualifiedName>,
    /// For a Satisfaction Claim, what checking it found; `None` until
    /// checked.
    pub claim_result: Option<ClaimResult>,
    pub documentation: Option<String>,
    pub source: Option<SourceLocation>,
}

impl Element {
    /// How the element is named to a reader: its name, or for an unnamed
    /// connection-like element its ends (`a → b`, or `a = b` for a binding).
    pub fn display_name(&self) -> String {
        if let (true, Some(requirement), Some(satisfier)) =
            (self.name.is_empty(), &self.satisfies, &self.satisfier)
        {
            return format!(
                "{} by {}",
                requirement.simple_name(),
                satisfier.simple_name()
            );
        }
        if !self.name.is_empty() || self.ends.is_empty() {
            return self.name.clone();
        }
        let ends: Vec<String> = self.ends.iter().map(ToString::to_string).collect();
        let joiner = if self.kind == ElementKind::BindingConnection {
            " = "
        } else {
            " → "
        };
        ends.join(joiner)
    }

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
            is_reference: false,
            payload: None,
            exposes: Vec::new(),
            rendering: None,
            metadata: Vec::new(),
            short_name: None,
            subject: None,
            require: Vec::new(),
            value: None,
            is_asserted: false,
            expression: None,
            checks: Vec::new(),
            satisfies: None,
            satisfier: None,
            claim_result: None,
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

    /// Every Satisfaction Claim, in declaration order within each owner:
    /// generated names number them (`Req::@8`, `Req::@10`), so they sort by
    /// that number, not as text.
    pub fn claims(&self) -> impl Iterator<Item = &Element> {
        let mut claims: Vec<&Element> = self
            .elements
            .values()
            .filter(|e| e.kind == ElementKind::SatisfyRequirement)
            .collect();
        claims.sort_by_key(|c| {
            let name = c.qualified_name.as_str();
            let numbered = name
                .rsplit_once("::@")
                .and_then(|(owner, n)| Some((owner, n.parse::<u64>().ok()?)));
            match numbered {
                Some((owner, n)) => (owner.to_owned(), n),
                None => (name.to_owned(), 0),
            }
        });
        claims.into_iter()
    }

    /// Elements directly owned by `owner`, ordered by qualified name.
    pub fn owned_members<'a>(
        &'a self,
        owner: &'a QualifiedName,
    ) -> impl Iterator<Item = &'a Element> {
        self.elements
            .values()
            .filter(move |e| e.owner.as_ref() == Some(owner))
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
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

    pub fn flow(self, name: &str) -> Self {
        self.element(name, ElementKind::FlowUsage)
    }

    pub fn binding(self, name: &str) -> Self {
        self.element(name, ElementKind::BindingConnection)
    }

    /// An unnamed connection-like element, as the service reports one: a
    /// generated qualified name (`Vehicle::@6`) and an empty name.
    pub fn unnamed(mut self, name: &str, kind: ElementKind) -> Self {
        self = self.element(name, kind);
        self.last().name = String::new();
        self
    }

    pub fn reference(mut self) -> Self {
        self.last().is_reference = true;
        self
    }

    pub fn payload(mut self, item: &str) -> Self {
        self.last().payload = Some(QualifiedName::new(item));
        self
    }

    pub fn requirement_def(self, name: &str) -> Self {
        self.element(name, ElementKind::RequirementDefinition)
    }

    pub fn requirement(self, name: &str) -> Self {
        self.element(name, ElementKind::RequirementUsage)
    }

    pub fn short_name(mut self, id: &str) -> Self {
        self.last().short_name = Some(id.to_owned());
        self
    }

    /// Makes `feature` (an owned feature, added separately) the subject.
    pub fn subject(mut self, feature: &str) -> Self {
        self.last().subject = Some(QualifiedName::new(feature));
        self
    }

    /// A Satisfaction Claim as the service reports one: a generated
    /// qualified name (`Req::@10`), no name, the requirement usage it
    /// claims and its satisfier.
    pub fn satisfy(mut self, name: &str, requirement: &str, satisfier: &str) -> Self {
        self = self.unnamed(name, ElementKind::SatisfyRequirement);
        self.last().satisfies = Some(QualifiedName::new(requirement));
        self.last().satisfier = Some(QualifiedName::new(satisfier));
        self
    }

    pub fn constraint_def(self, name: &str) -> Self {
        self.element(name, ElementKind::ConstraintDefinition)
    }

    pub fn constraint(self, name: &str) -> Self {
        self.element(name, ElementKind::ConstraintUsage)
    }

    /// Makes the last constraint usage asserted (`assert constraint`).
    pub fn asserted(mut self) -> Self {
        self.last().is_asserted = true;
        self
    }

    /// Records what checking the last constraint on `part` found.
    pub fn checked_on(mut self, part: &str, result: ClaimResult) -> Self {
        self.last().checks.push(ConstraintCheck {
            part: QualifiedName::new(part),
            result,
        });
        self
    }

    /// Sets the last constraint's own expression, as text.
    pub fn expression(mut self, text: &str) -> Self {
        self.last().expression = Some(text.to_owned());
        self
    }

    /// Sets the last feature's value, as text (`180.0`).
    pub fn value(mut self, text: &str) -> Self {
        self.last().value = Some(text.to_owned());
        self
    }

    /// Sets what checking the last Satisfaction Claim found.
    pub fn result(mut self, result: ClaimResult) -> Self {
        self.last().claim_result = Some(result);
        self
    }

    /// Adds a `require` constraint, as its text (`v.mass <= limit`).
    pub fn requires(mut self, constraint: &str) -> Self {
        self.last().require.push(constraint.to_owned());
        self
    }

    pub fn view_def(self, name: &str) -> Self {
        self.element(name, ElementKind::ViewDefinition)
    }

    pub fn view(self, name: &str) -> Self {
        self.element(name, ElementKind::ViewUsage)
    }

    pub fn exposes(mut self, target: &str, scope: ExposeScope) -> Self {
        self.last().exposes.push(Expose {
            target: QualifiedName::new(target),
            scope,
        });
        self
    }

    pub fn renders(mut self, rendering: &str) -> Self {
        self.last().rendering = Some(QualifiedName::new(rendering));
        self
    }

    pub fn annotated(mut self, metadata_def: &str) -> Self {
        self.last().metadata.push(QualifiedName::new(metadata_def));
        self
    }

    pub fn unsupported(self, name: &str, keyword: &str) -> Self {
        self.element(name, ElementKind::Unsupported(keyword.to_owned()))
    }

    pub fn element(mut self, name: &str, kind: ElementKind) -> Self {
        self.elements
            .push(Element::new(QualifiedName::new(name), kind));
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
        self.last().multiplicity = Some(Multiplicity {
            lower: lower.to_owned(),
            upper: upper.to_owned(),
        });
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
        self.last().source = Some(SourceLocation {
            file: file.into(),
            line,
        });
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
