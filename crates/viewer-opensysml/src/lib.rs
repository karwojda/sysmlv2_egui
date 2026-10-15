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

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use opensysml::{Connection, SourceDocument, SourcesOptions, Symbol};
use viewer_core::model::{
    Diagnostic, Direction, Element, ElementKind, Model, Multiplicity, QualifiedName, Severity, SourceLocation,
};
use viewer_core::{BackendError, ModelBackend};

/// Loads Models through a private `sysml-grpc` service, started on first use.
#[derive(Default)]
pub struct OpenSysmlBackend {
    connection: Option<Connection>,
}

impl OpenSysmlBackend {
    pub fn new() -> Self {
        Self::default()
    }

    fn connection(&mut self) -> Result<&Connection, BackendError> {
        if self.connection.is_none() {
            let connection = Connection::private().map_err(unavailable)?;
            self.connection = Some(connection);
        }
        Ok(self.connection.as_ref().expect("connection was just set"))
    }
}

impl ModelBackend for OpenSysmlBackend {
    fn load(&mut self, folder: &Path) -> Result<Model, BackendError> {
        if !folder.is_dir() {
            return Err(BackendError::FolderNotFound(folder.to_path_buf()));
        }
        let files = viewer_core::watch::model_files(folder).map_err(|e| BackendError::Unavailable(e.to_string()))?;
        if files.is_empty() {
            return Ok(Model::default());
        }
        let documents: Vec<SourceDocument> = files.iter().map(SourceDocument::file).collect();
        let connection = self.connection()?.clone();

        let parsed = match connection.parse_sources(&documents, &SourcesOptions::default()) {
            Ok(parsed) => parsed,
            // The service refused the whole Model; its diagnostics say why.
            Err(opensysml::Error::ModelErrors { message, diagnostics }) => {
                let mut diagnostics: Vec<Diagnostic> = diagnostics.iter().map(diagnostic).collect();
                if diagnostics.is_empty() {
                    diagnostics.push(Diagnostic { severity: Severity::Error, message, location: None });
                }
                return Ok(Model::new([], diagnostics));
            }
            Err(error) => return Err(unavailable(error)),
        };

        let mut elements = BTreeMap::new();
        for root in parsed.roots() {
            collect(root, None, &mut elements)?;
        }
        let mut diagnostics: Vec<Diagnostic> = parsed.diagnostics().iter().map(diagnostic).collect();
        for (document, error) in api_json::enrich(&connection, &parsed, &mut elements) {
            // A file with its own errors already says why; don't repeat it.
            let file = PathBuf::from(&document);
            let has_errors = diagnostics
                .iter()
                .any(|d| d.severity == Severity::Error && d.location.as_ref().is_some_and(|l| l.file == file));
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
        Ok(Model::new(elements.into_values(), diagnostics))
    }
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
        element.multiplicity = info
            .multiplicity
            .as_ref()
            .map(|m| Multiplicity { lower: m.lower.clone(), upper: m.upper.clone() });
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
        severity: if d.severity == "error" { Severity::Error } else { Severity::Warning },
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
    fn service_kinds_become_our_kinds_or_sysml_keywords() {
        assert_eq!(element_kind("partDef"), ElementKind::PartDefinition);
        assert_eq!(element_kind("interfaceUsage"), ElementKind::InterfaceUsage);
        assert_eq!(element_kind("requirementDef"), ElementKind::Unsupported("requirement def".into()));
        assert_eq!(element_kind("actionUsage"), ElementKind::Unsupported("action".into()));
    }
}
