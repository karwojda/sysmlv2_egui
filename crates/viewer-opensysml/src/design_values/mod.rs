//! Design Values (ADR-0003 to ADR-0006): checks a parsed Model with the
//! values its Tool Links read from Engineering Tools in place of its
//! Model Values. The plan asks the tools (`plan.rs`), the edit writes
//! the values into the Model in memory and parses it again (`edit.rs`),
//! and the checks' reasons about them are put in words (`words.rs`).
//!
//! What is checked comes through the `Store` seam: the service's parse in
//! a load, a test's tables in the tests.

mod edit;
mod plan;
#[cfg(test)]
mod tests;
mod words;

use std::collections::BTreeMap;
use std::path::Path;

use opensysml::{Connection, SourceDocument};
use viewer_core::backend::Analyses;
use viewer_core::engineering_tools::{DesignValue, DesignValueOutcome, EngineeringTool, ToolLink};
use viewer_core::model::{ClaimResult, Diagnostic, Element, QualifiedName, Severity};

use crate::checks::{self, Checker, Service};
use plan::ValueEdit;
use words::Worded;

pub(crate) use plan::{HeldLink, LinkedProperty, ToolLinks};

/// The Model checked, as one with Design Values written into it can be.
pub(crate) trait Store {
    /// Checks the Model as parsed, Model Values only.
    fn checker(&self) -> Box<dyn Checker + '_>;
    /// Checks the Model with `edits` written into it, or says why they
    /// could not be.
    fn with_edits(&mut self, edits: &[ValueEdit]) -> Result<Box<dyn Checker + '_>, String>;
}

/// A Model the service parsed from `documents`, which Design Values are
/// written into by the service's editor and a parse of the edited text.
pub(crate) struct Parsed<'a> {
    connection: &'a Connection,
    model: &'a opensysml::Model,
    documents: &'a [SourceDocument],
    /// The Model with Design Values written into it, once they are.
    designed: Option<opensysml::Model>,
}

impl<'a> Parsed<'a> {
    pub fn new(
        connection: &'a Connection,
        model: &'a opensysml::Model,
        documents: &'a [SourceDocument],
    ) -> Self {
        Self {
            connection,
            model,
            documents,
            designed: None,
        }
    }
}

impl Store for Parsed<'_> {
    fn checker(&self) -> Box<dyn Checker + '_> {
        Box::new(Service::new(self.model))
    }

    fn with_edits(&mut self, edits: &[ValueEdit]) -> Result<Box<dyn Checker + '_>, String> {
        self.designed =
            edit::with_design_values(self.connection, self.model, self.documents, edits)?;
        Ok(Box::new(Service::new(
            self.designed.as_ref().unwrap_or(self.model),
        )))
    }
}

/// What checking a Model with its Design Values found.
#[derive(Debug, Default)]
pub(crate) struct Checked {
    /// What is wrong with the Tool Links, and what couldn't be checked.
    pub diagnostics: Vec<Diagnostic>,
    /// Every Tool Link, for the Model to record.
    pub tool_links: Vec<ToolLink>,
    /// Every linked feature's Design Value outcome, for the Model to
    /// record.
    pub outcomes: Vec<DesignValueOutcome>,
}

/// Checks the Model of `store` and records the results on `elements` (see
/// [`checks::run`]), with the Design Values the Tool Links `found` read
/// from `tools` in place of their Model Values. The elements keep what
/// the Model writes, Model Values included. If the Design Values can't be
/// written into the Model, nothing is judged: which checks rest on them
/// can't be told.
pub(crate) fn check(
    store: &mut dyn Store,
    found: &ToolLinks,
    tools: &mut BTreeMap<String, Box<dyn EngineeringTool>>,
    elements: &mut BTreeMap<QualifiedName, Element>,
    folder: &Path,
    analyses: &Analyses,
) -> Checked {
    if found.is_empty() {
        return Checked {
            diagnostics: checks::run(&*store.checker(), elements, folder, analyses),
            ..Checked::default()
        };
    }
    let plan = plan::plan(found, tools, elements);
    let mut diagnostics = plan.diagnostics;
    let mut outcomes = plan.outcomes;
    let failed = match store.with_edits(&plan.edits) {
        Ok(checker) => {
            diagnostics.extend(checks::run(
                &Worded {
                    checker: &*checker,
                    written: &plan.written,
                },
                elements,
                folder,
                analyses,
            ));
            None
        }
        Err(error) => Some(error),
    };
    if let Some(error) = failed {
        diagnostics.push(Diagnostic {
            severity: Severity::Error,
            message: format!("Design Values could not be written into the Model: {error}"),
            location: None,
        });
        diagnostics.extend(checks::run(&*store.checker(), elements, folder, analyses));
        let reason = format!("could not be written into the Model: {error}");
        not_judged(
            elements,
            &format!("Design Values {reason}, so nothing was judged"),
        );
        for outcome in &mut outcomes {
            if let DesignValue::Read { .. } = outcome.design_value {
                outcome.design_value = DesignValue::Missing {
                    reason: reason.clone(),
                };
            }
        }
    }
    Checked {
        diagnostics,
        tool_links: plan.tool_links,
        outcomes,
    }
}

/// Makes every check result on `elements` Undecided with `reason`, and
/// drops every value the checks computed: none of them can be trusted.
fn not_judged(elements: &mut BTreeMap<QualifiedName, Element>, reason: &str) {
    for element in elements.values_mut() {
        element.computed.clear();
        if let Some(analysis) = &mut element.analysis_result {
            analysis.outputs.clear();
        }
        for result in check_results(element) {
            *result = ClaimResult::Undecided {
                reason: reason.to_owned(),
            };
        }
    }
}

/// Every check result recorded on `element`: its claim's, its constraint
/// checks', its analysis objective's.
fn check_results(element: &mut Element) -> impl Iterator<Item = &mut ClaimResult> {
    element
        .claim_result
        .iter_mut()
        .chain(element.checks.iter_mut().map(|c| &mut c.result))
        .chain(
            element
                .analysis_result
                .iter_mut()
                .flat_map(|a| a.objective.as_mut()),
        )
}
