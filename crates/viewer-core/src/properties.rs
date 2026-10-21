//! What the properties panel shows for an element: labelled values, some of
//! which refer to other elements and can be followed.

use crate::constraint;
use crate::diagram::DiagramKind;
use crate::model::{ElementKind, Model, QualifiedName, SourceLocation};
use crate::satisfaction;
use crate::verdict::{self, Kind, Status};
use crate::view::ViewError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Property {
    pub label: &'static str,
    pub value: String,
    /// The element the value refers to, for a type, owner or general.
    pub refers_to: Option<QualifiedName>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Properties {
    pub rows: Vec<Property>,
    /// Where the element is declared, for "open in editor".
    pub source: Option<SourceLocation>,
}

/// The properties of `element`, or `None` if the Model has no such
/// element. Only properties the element has are listed.
pub fn properties(model: &Model, element: &QualifiedName) -> Option<Properties> {
    let e = model.element(element)?;
    let text = |label, value: String| Property {
        label,
        value,
        refers_to: None,
    };
    let link = |label, target: &QualifiedName, value: String| Property {
        label,
        value,
        refers_to: model.element(target).map(|_| target.clone()),
    };

    let mut rows = vec![
        text("Kind", format!("«{}»", e.kind.keyword())),
        text(
            "Name",
            if e.name.is_empty() {
                "(unnamed)".to_owned()
            } else {
                e.name.clone()
            },
        ),
        text("Qualified name", e.qualified_name.to_string()),
    ];
    if let Some(id) = &e.short_name {
        rows.push(text("Short id", id.clone()));
    }
    if let Some(owner) = &e.owner {
        rows.push(link("Owner", owner, owner.to_string()));
    }
    if let Some(definition) = &e.typed_by {
        let conjugate = if e.conjugated { "~" } else { "" };
        rows.push(link("Type", definition, format!("{conjugate}{definition}")));
    }
    if let Some(subject) = &e.subject {
        let label = model
            .element(subject)
            .map(crate::notation::usage_label)
            .unwrap_or_else(|| subject.to_string());
        rows.push(link("Subject", subject, label));
    }
    if let Some(l) = model.tool_link(element) {
        rows.push(text(
            "Tool Link",
            format!("⚙ {} · {} · {}", l.tool, l.file, l.object),
        ));
    }
    if let Some(value) = &e.value {
        rows.push(text("Value", value.clone()));
    }
    if !e.computed.is_empty() {
        rows.push(text("Computed", e.computed.join(", ")));
    }
    if let Some(expression) = &e.expression {
        rows.push(text("Expression", expression.clone()));
    }
    if e.is_asserted {
        rows.push(text("Asserted", "yes".to_owned()));
    }
    let verdict = verdict::of(model, e);
    // An analysis usage: its outputs and what its objective came to.
    if e.kind == ElementKind::AnalysisUsage && e.analysis_result.is_none() {
        let label = if verdict.is_some() {
            "Objective"
        } else {
            "Result"
        };
        rows.push(text(label, format!("{} not run", Status::Absent.mark())));
    }
    if let Some(result) = &e.analysis_result {
        for (name, value) in &result.outputs {
            rows.push(text("Output", format!("{name} = {value}")));
        }
        match &result.objective {
            Some(outcome) => rows.push(text("Objective", verdict::describe(Some(outcome)))),
            // Ran, but the service said nothing about the objective.
            None => {
                for finding in verdict.iter().flat_map(|v| &v.findings) {
                    let detail = finding.detail.as_deref().unwrap_or_default();
                    rows.push(text(
                        "Objective",
                        format!("{} {detail}", finding.status.mark()),
                    ));
                }
            }
        }
    }
    // An asserted constraint: what checking it found on each part.
    for check in &e.checks {
        rows.push(link(
            "Result",
            &check.part,
            format!(
                "on {} {}",
                check.part,
                verdict::describe(Some(&check.result))
            ),
        ));
    }
    // A part: what checking each of its constraints found on it.
    for (constraint_element, check) in constraint::checks_on(model, element) {
        let status = Status::of(&check.result);
        rows.push(link(
            "Asserts",
            &constraint_element.qualified_name,
            format!("{} {}", constraint_element.name, status.mark()),
        ));
        rows.extend(reason(status, verdict::detail(&check.result)));
    }
    for constraint in &e.require {
        rows.push(text("Requires", format!("{{{constraint}}}")));
    }
    if let (Some(requirement), Some(satisfier)) = (&e.satisfies, &e.satisfier) {
        rows.push(link("Requirement", requirement, requirement.to_string()));
        rows.push(link("Satisfier", satisfier, satisfier.to_string()));
        rows.push(text("Result", verdict::describe(e.claim_result.as_ref())));
    }
    if let Some(verdict) = verdict.filter(|v| v.kind == Kind::Requirement) {
        rows.push(text(
            "Status",
            format!("{} {}", verdict.mark(), verdict.label()),
        ));
        let nested = satisfaction::nested(model, e).count();
        if nested > 0 {
            rows.push(text(
                "Rolled up from",
                format!("{nested} nested requirements"),
            ));
        }
    }
    for claim in model.claims() {
        let (Some(requirement), Some(satisfier)) = (&claim.satisfies, &claim.satisfier) else {
            continue;
        };
        let status = satisfaction::claim_status(claim);
        if requirement == element {
            rows.push(link(
                "Satisfied by",
                satisfier,
                format!("{satisfier} {}", status.mark()),
            ));
            rows.extend(reason(
                status,
                claim.claim_result.as_ref().and_then(verdict::detail),
            ));
        }
        if satisfier == element {
            rows.push(link(
                "Satisfies",
                requirement,
                format!("{requirement} {}", status.mark()),
            ));
        }
    }
    if e.is_reference {
        rows.push(text("Reference", "yes".to_owned()));
    }
    if let Some(multiplicity) = &e.multiplicity {
        rows.push(text("Multiplicity", multiplicity.to_string()));
    }
    if let Some(direction) = e.direction {
        rows.push(text("Direction", direction.keyword().to_owned()));
    }
    for general in &e.specializes {
        rows.push(link("Specializes", general, general.to_string()));
    }
    for (i, end) in e.ends.iter().enumerate() {
        let label = if i == 0 { "Connects" } else { "To" };
        rows.push(text(label, end.to_string()));
    }
    if let Some(rendering) = &e.rendering {
        rows.push(text("Renders", rendering.simple_name().to_owned()));
    }
    if e.kind == ElementKind::ViewUsage {
        let diagram = match crate::view::kind(model, element) {
            Ok(DiagramKind::General) => "General Diagram".to_owned(),
            Ok(DiagramKind::Interconnection) => "Interconnection Diagram".to_owned(),
            Err(ViewError::Unsupported { reason, .. }) => format!("unsupported: {reason}"),
            Err(error) => error.to_string(),
        };
        rows.push(text("Drawn as", diagram));
    }
    for expose in &e.exposes {
        rows.push(link("Exposes", &expose.target, expose.to_string()));
    }
    for metadata in &e.metadata {
        rows.push(link(
            "Metadata",
            metadata,
            format!("@{}", metadata.simple_name()),
        ));
    }
    if let Some(documentation) = &e.documentation {
        rows.push(text("Documentation", documentation.clone()));
    }
    if let Some(source) = &e.source {
        rows.push(text(
            "Source",
            format!("{}:{}", source.file.display(), source.line),
        ));
    }
    Some(Properties {
        rows,
        source: e.source.clone(),
    })
}

/// Why a check didn't hold: its failing condition, or why it was undecided.
fn reason(status: Status, detail: Option<&str>) -> Option<Property> {
    let label = match status {
        Status::Violated => "Failing condition",
        Status::Undecided => "Undecided because",
        Status::Holds | Status::Absent => return None,
    };
    Some(Property {
        label,
        value: detail?.to_owned(),
        refers_to: None,
    })
}
