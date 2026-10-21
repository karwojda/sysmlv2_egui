//! The OpenSysML Model backend (ADR-0001): loads a Model through the
//! `sysml-grpc` service using the `opensysml` client.
//!
//! The client locates the service binary itself: `$OPENSYSML_GRPC_BINARY`,
//! then its cache in `~/.opensysml/bin`, then a pinned download, then `$PATH`.
//! The private service it starts stops when the last connection is dropped.
//!
//! Elements come from two views of the parsed Model. The symbol tree gives
//! kinds, owners, typing (library types included), multiplicity,
//! specializations and port direction. The standard API's JSON element form,
//! which the service marks experimental, adds what the symbol tree lacks:
//! connection ends, documentation, port conjugation and source lines.

mod api_json;
mod checks;
mod design_values;
mod expression;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use opensysml::{Connection, SourceDocument, SourcesOptions, Symbol};
use viewer_core::backend::Analyses;
use viewer_core::engineering_tools::EngineeringTool;
use viewer_core::model::{
    Diagnostic, Direction, Element, ElementKind, Model, Multiplicity, QualifiedName, Severity,
    SourceLocation,
};
use viewer_core::{BackendError, ModelBackend};

/// The environment variables the `opensysml` client reads for a binary and
/// a running service; flags set the same values.
pub const BINARY_VARIABLE: &str = "OPENSYSML_GRPC_BINARY";
pub const SERVICE_VARIABLE: &str = "OPENSYSML_SERVICE";

/// How to reach `sysml-grpc`: a specific binary to start, or a service
/// already running at `host:port` (which is never started or stopped). With
/// neither, the client finds or downloads a binary itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServiceConfig {
    pub binary: Option<PathBuf>,
    pub service: Option<String>,
}

impl ServiceConfig {
    /// From [`BINARY_VARIABLE`] and [`SERVICE_VARIABLE`]; empty values count
    /// as unset.
    pub fn from_env() -> Self {
        let var = |name| std::env::var(name).ok().filter(|v| !v.trim().is_empty());
        Self {
            binary: var(BINARY_VARIABLE).map(PathBuf::from),
            service: var(SERVICE_VARIABLE),
        }
    }

    fn connect(&self) -> Result<Connection, BackendError> {
        if let Some(service) = &self.service {
            let (host, port) = service
                .rsplit_once(':')
                .and_then(|(host, port)| Some((host, port.parse::<u16>().ok()?)))
                .filter(|(host, _)| !host.is_empty())
                .ok_or_else(|| {
                    BackendError::Unavailable(format!(
                        "the sysml-grpc service address `{service}` is not host:port"
                    ))
                })?;
            return Connection::external(host, port).map_err(|e| {
                BackendError::Unavailable(format!(
                    "could not reach the sysml-grpc service at {service}: {e}"
                ))
            });
        }
        // The client skips a configured binary that doesn't exist and
        // downloads one instead; a typo should fail, not surprise.
        if let Some(binary) = &self.binary
            && !binary.is_file()
        {
            return Err(BackendError::Unavailable(format!(
                "the configured sysml-grpc binary {} does not exist",
                binary.display()
            )));
        }
        Connection::private().map_err(|e| BackendError::Unavailable(explain_start_failure(&e)))
    }
}

/// What went wrong starting the private service, and what to do about it.
fn explain_start_failure(error: &opensysml::Error) -> String {
    let remedy = "Set a sysml-grpc binary with --sysml-binary <path> (or OPENSYSML_GRPC_BINARY), \
                  or connect to a running service with --sysml-service <host:port> (or OPENSYSML_SERVICE).";
    match error {
        opensysml::Error::BinaryNotFound { .. }
        | opensysml::Error::BinaryDownload(_)
        | opensysml::Error::UnsupportedPlatform { .. }
        | opensysml::Error::ChecksumMismatch(_) => {
            format!("could not get the sysml-grpc service: {error}. {remedy}")
        }
        opensysml::Error::ServiceStart(_) => {
            format!("the sysml-grpc service did not start: {error}. {remedy}")
        }
        other => format!("could not start the sysml-grpc service: {other}"),
    }
}

/// Loads Models through `sysml-grpc`, connecting on first use: a private
/// service the client starts (and stops when the backend is dropped), or a
/// running one if configured.
pub struct OpenSysmlBackend {
    config: ServiceConfig,
    connection: Option<Connection>,
    /// The Engineering Tools Tool Links can name, by name (`FreeCAD`).
    tools: BTreeMap<String, Box<dyn EngineeringTool>>,
}

impl Default for OpenSysmlBackend {
    fn default() -> Self {
        Self::with_config(ServiceConfig::from_env())
    }
}

impl OpenSysmlBackend {
    /// Configured from the environment; see [`ServiceConfig::from_env`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: ServiceConfig) -> Self {
        Self {
            config,
            connection: None,
            tools: BTreeMap::new(),
        }
    }

    /// Lets Tool Links naming `name` (`FreeCAD`) read their Design Values
    /// from `tool`.
    pub fn with_tool(
        mut self,
        name: impl Into<String>,
        tool: impl EngineeringTool + 'static,
    ) -> Self {
        self.tools.insert(name.into(), Box::new(tool));
        self
    }

    fn connection(&mut self) -> Result<&Connection, BackendError> {
        if self.connection.is_none() {
            self.connection = Some(self.config.connect()?);
        }
        Ok(self.connection.as_ref().expect("connection was just set"))
    }
}

impl ModelBackend for OpenSysmlBackend {
    fn load_with(&mut self, folder: &Path, analyses: &Analyses) -> Result<Model, BackendError> {
        if !folder.is_dir() {
            return Err(BackendError::FolderNotFound(folder.to_path_buf()));
        }
        let files = viewer_core::watch::model_files(folder)
            .map_err(|e| BackendError::Unavailable(e.to_string()))?;
        if files.is_empty() {
            return Ok(Model::default());
        }
        let documents: Vec<SourceDocument> = files.iter().map(SourceDocument::file).collect();
        let connection = self.connection()?.clone();

        let parsed = match connection.parse_sources(&documents, &SourcesOptions::default()) {
            Ok(parsed) => parsed,
            // The service refused the whole Model; its diagnostics say why.
            Err(opensysml::Error::ModelErrors {
                message,
                diagnostics,
            }) => {
                let mut diagnostics: Vec<Diagnostic> = diagnostics.iter().map(diagnostic).collect();
                if diagnostics.is_empty() {
                    diagnostics.push(Diagnostic {
                        severity: Severity::Error,
                        message,
                        location: None,
                    });
                }
                return Ok(Model::new([], diagnostics));
            }
            Err(error) => return Err(unavailable(error)),
        };

        let mut elements = BTreeMap::new();
        for root in parsed.roots() {
            collect(root, None, &mut elements)?;
        }
        let mut diagnostics: Vec<Diagnostic> =
            parsed.diagnostics().iter().map(diagnostic).collect();
        let (failures, tool_links) = api_json::enrich(&connection, &parsed, &mut elements);
        for (document, error) in failures {
            // A file with its own errors already says why; don't repeat it.
            let file = PathBuf::from(&document);
            let has_errors = diagnostics.iter().any(|d| {
                d.severity == Severity::Error && d.location.as_ref().is_some_and(|l| l.file == file)
            });
            if has_errors {
                continue;
            }
            diagnostics.push(Diagnostic {
                severity: Severity::Warning,
                message: format!(
                    "connection ends, documentation and source lines are missing for this file: {error}"
                ),
                location: Some(SourceLocation { file, line: 0 }),
            });
        }
        if tool_links.is_empty() {
            diagnostics.extend(checks::run(
                &checks::Service(&parsed),
                &mut elements,
                folder,
                analyses,
            ));
            return Ok(Model::new(elements.into_values(), diagnostics));
        }

        // The checks run on the Model with its Design Values; the elements
        // keep what the Model writes, Model Values included.
        let plan = design_values::plan(&tool_links, &mut self.tools, &elements);
        let designed = match with_design_values(&connection, &parsed, &documents, &plan.edits) {
            Ok(designed) => designed,
            Err(error) => {
                diagnostics.push(Diagnostic {
                    severity: Severity::Error,
                    message: format!("Design Values could not be written into the Model: {error}"),
                    location: None,
                });
                None
            }
        };
        diagnostics.extend(checks::run(
            &checks::Service(designed.as_ref().unwrap_or(&parsed)),
            &mut elements,
            folder,
            analyses,
        ));
        Ok(Model::new(elements.into_values(), diagnostics)
            .with_design_values(plan.tool_links, plan.outcomes))
    }
}

/// `parsed` with `edits` made in memory, one editor per document, and
/// parsed again: the edited documents inline, the rest from `documents`.
/// `None` when there is nothing to edit. The files are never written.
fn with_design_values(
    connection: &Connection,
    parsed: &opensysml::Model,
    documents: &[SourceDocument],
    edits: &[design_values::Edit],
) -> Result<Option<opensysml::Model>, opensysml::Error> {
    let mut per_document: BTreeMap<&str, Vec<&design_values::Edit>> = BTreeMap::new();
    for edit in edits {
        per_document.entry(&edit.document).or_default().push(edit);
    }
    if per_document.is_empty() {
        return Ok(None);
    }
    let mut edited: BTreeMap<String, String> = BTreeMap::new();
    for (document, edits) in per_document {
        let mut editor = parsed.edit();
        editor.in_document(document);
        for edit in edits {
            match &edit.how {
                design_values::How::SetValue => {
                    editor.set_value(edit.feature.as_str(), &edit.value);
                }
                design_values::How::Redefine { owner, redefines } => {
                    editor.delete(edit.feature.as_str(), false);
                    editor.add_member(
                        owner.as_str(),
                        "attribute",
                        "",
                        opensysml::MemberOptions {
                            value: Some(edit.value.clone()),
                            redefines: vec![redefines.clone()],
                            ..opensysml::MemberOptions::new()
                        },
                    );
                }
            }
        }
        for result in editor.apply()?.documents {
            edited.insert(result.name, result.content);
        }
    }
    let sources: Vec<SourceDocument> = documents
        .iter()
        .map(|d| {
            let name = d.document_name();
            match edited.remove(&name) {
                Some(content) => SourceDocument::inline(name, content),
                None => d.clone(),
            }
        })
        .collect();
    connection
        .parse_sources(&sources, &SourcesOptions::default())
        .map(Some)
}

/// Converts `symbol` and its descendants. Each document's root namespace has
/// an empty id and is not itself an element; connector ends are part of
/// their connection, not elements of their own.
fn collect(
    symbol: &Symbol,
    owner: Option<&QualifiedName>,
    out: &mut BTreeMap<QualifiedName, Element>,
) -> Result<(), BackendError> {
    let info = symbol.wire();
    if info.kind == "connectorEnd" {
        return Ok(());
    }
    // A view's `render` is a fact of the view, not an element of its own.
    let owner_kind = owner.and_then(|o| out.get(o)).map(|e| &e.kind);
    if info.kind == "renderingUsage"
        && matches!(
            owner_kind,
            Some(ElementKind::ViewUsage | ElementKind::ViewDefinition)
        )
    {
        return Ok(());
    }
    let this = if info.id.is_empty() {
        None
    } else {
        let qualified_name = QualifiedName::new(info.id.clone());
        let mut element = Element::new(qualified_name.clone(), element_kind(&info.kind));
        element.name = info.name.clone();
        element.owner = owner.cloned();
        element.specializes = info
            .specializations
            .iter()
            .filter(|s| s.kind == "specializes" && !s.target_id.is_empty())
            .map(|s| QualifiedName::new(s.target_id.clone()))
            .collect();
        element.typed_by = info
            .type_info
            .as_ref()
            .filter(|t| !t.resolved_id.is_empty())
            .map(|t| QualifiedName::new(t.resolved_id.clone()));
        element.multiplicity = info.multiplicity.as_ref().map(|m| Multiplicity {
            lower: m.lower.clone(),
            upper: m.upper.clone(),
        });
        element.direction = info.metadata.get("direction").and_then(|d| direction(d));
        out.insert(qualified_name.clone(), element);
        Some(qualified_name)
    };
    for child in symbol.children().map_err(unavailable)? {
        collect(&child, this.as_ref().or(owner), out)?;
    }
    Ok(())
}

/// Maps a service symbol kind (`partDef`, `requirementDef`, ...) to ours.
fn element_kind(kind: &str) -> ElementKind {
    match kind {
        "package" => ElementKind::Package,
        "partDef" => ElementKind::PartDefinition,
        "partUsage" => ElementKind::PartUsage,
        "attributeDef" => ElementKind::AttributeDefinition,
        "attributeUsage" => ElementKind::AttributeUsage,
        "portDef" => ElementKind::PortDefinition,
        "portUsage" => ElementKind::PortUsage,
        "connectionDef" => ElementKind::ConnectionDefinition,
        "connectionUsage" => ElementKind::ConnectionUsage,
        "interfaceDef" => ElementKind::InterfaceDefinition,
        "interfaceUsage" => ElementKind::InterfaceUsage,
        "flowUsage" => ElementKind::FlowUsage,
        "bindingConnectorAsUsage" | "bindingConnector" => ElementKind::BindingConnection,
        "viewDef" => ElementKind::ViewDefinition,
        "requirementDef" => ElementKind::RequirementDefinition,
        "requirementUsage" => ElementKind::RequirementUsage,
        "constraintDef" => ElementKind::ConstraintDefinition,
        "constraintUsage" => ElementKind::ConstraintUsage,
        "analysisCaseDef" => ElementKind::AnalysisDefinition,
        "analysisCaseUsage" => ElementKind::AnalysisUsage,
        "viewUsage" => ElementKind::ViewUsage,
        other => ElementKind::Unsupported(keyword_of(other)),
    }
}

/// `requirementDef` → `requirement def`, `actionUsage` → `action`: the
/// SysML keyword the kind names.
fn keyword_of(kind: &str) -> String {
    let kind = kind.strip_suffix("Usage").unwrap_or(kind);
    let mut keyword = String::new();
    for c in kind.chars() {
        if c.is_uppercase() && !keyword.is_empty() {
            keyword.push(' ');
        }
        keyword.extend(c.to_lowercase());
    }
    keyword
}

fn direction(text: &str) -> Option<Direction> {
    match text {
        "in" => Some(Direction::In),
        "out" => Some(Direction::Out),
        "inout" => Some(Direction::InOut),
        _ => None,
    }
}

fn diagnostic(d: &opensysml::Diagnostic) -> Diagnostic {
    Diagnostic {
        severity: if d.severity == "error" {
            Severity::Error
        } else {
            Severity::Warning
        },
        message: d.message.clone(),
        location: d.span.as_ref().map(|span| SourceLocation {
            file: PathBuf::from(&span.file),
            line: span.start_line.max(0) as u32,
        }),
    }
}

fn unavailable(error: opensysml::Error) -> BackendError {
    BackendError::Unavailable(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_failures_say_how_to_configure_a_binary_or_service() {
        let message = explain_start_failure(&opensysml::Error::BinaryDownload("offline".into()));

        assert!(message.contains("offline"), "{message}");
        assert!(
            message.contains("--sysml-binary") && message.contains("--sysml-service"),
            "{message}"
        );
    }

    #[test]
    fn service_kinds_become_our_kinds_or_sysml_keywords() {
        assert_eq!(element_kind("partDef"), ElementKind::PartDefinition);
        assert_eq!(element_kind("interfaceUsage"), ElementKind::InterfaceUsage);
        assert_eq!(
            element_kind("requirementDef"),
            ElementKind::RequirementDefinition
        );
        assert_eq!(
            element_kind("requirementUsage"),
            ElementKind::RequirementUsage
        );
        assert_eq!(
            element_kind("useCaseDef"),
            ElementKind::Unsupported("use case def".into())
        );
        assert_eq!(
            element_kind("actionUsage"),
            ElementKind::Unsupported("action".into())
        );
    }
}
