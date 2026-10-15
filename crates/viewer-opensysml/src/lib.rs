//! The OpenSysML Model backend (ADR-0001): loads a Model through the
//! `sysml-grpc` service using the `opensysml` client.
//!
//! The client locates the service binary itself: `$OPENSYSML_GRPC_BINARY`,
//! then its cache in `~/.opensysml/bin`, then a pinned download, then `$PATH`.
//! The private service it starts stops when the last connection is dropped.

use std::path::{Path, PathBuf};

use opensysml::{Connection, SourceDocument, SourcesOptions, Symbol};
use viewer_core::model::{Diagnostic, Element, ElementKind, Model, QualifiedName, Severity, SourceLocation};
use viewer_core::{BackendError, ModelBackend};

const MODEL_EXTENSIONS: [&str; 2] = ["sysml", "kerml"];

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
        let files = model_files(folder).map_err(|e| BackendError::Unavailable(e.to_string()))?;
        if files.is_empty() {
            return Ok(Model::default());
        }
        let documents: Vec<SourceDocument> = files.iter().map(SourceDocument::file).collect();

        let parsed = match self.connection()?.parse_sources(&documents, &SourcesOptions::default()) {
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

        let mut elements = Vec::new();
        for root in parsed.roots() {
            collect(root, None, &mut elements)?;
        }
        let diagnostics = parsed.diagnostics().iter().map(diagnostic).collect();
        Ok(Model::new(elements, diagnostics))
    }
}

/// Every model file under `folder`, recursively, in a stable order.
fn model_files(folder: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut pending = vec![folder.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir)? {
            let path = entry?.path();
            if path.is_dir() {
                pending.push(path);
            } else if path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| MODEL_EXTENSIONS.contains(&e))
            {
                files.push(path);
            }
        }
    }
    files.sort();
    Ok(files)
}

/// Converts `symbol` and its descendants. Each document's root namespace has
/// an empty id and is not itself an element.
fn collect(symbol: &Symbol, owner: Option<&QualifiedName>, out: &mut Vec<Element>) -> Result<(), BackendError> {
    let info = symbol.wire();
    let this = if info.id.is_empty() {
        None
    } else {
        let qualified_name = QualifiedName::new(info.id.clone());
        out.push(Element {
            qualified_name: qualified_name.clone(),
            name: info.name.clone(),
            kind: element_kind(&info.kind),
            owner: owner.cloned(),
            specializes: info
                .specializations
                .iter()
                .filter(|s| s.kind == "specializes" && !s.target_id.is_empty())
                .map(|s| QualifiedName::new(s.target_id.clone()))
                .collect(),
            // The service does not report element source locations yet.
            source: None,
        });
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
        other => ElementKind::Unsupported(keyword_of(other)),
    }
}

/// `requirementDef` → `requirement def`: the SysML keyword the kind names.
fn keyword_of(kind: &str) -> String {
    let mut keyword = String::new();
    for c in kind.chars() {
        if c.is_uppercase() && !keyword.is_empty() {
            keyword.push(' ');
        }
        keyword.extend(c.to_lowercase());
    }
    keyword
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
    fn service_kinds_become_sysml_keywords() {
        assert_eq!(element_kind("requirementDef"), ElementKind::Unsupported("requirement def".into()));
        assert_eq!(element_kind("actionUsage"), ElementKind::Unsupported("action usage".into()));
        assert_eq!(element_kind("partDef"), ElementKind::PartDefinition);
        assert_eq!(element_kind("package"), ElementKind::Package);
    }
}
