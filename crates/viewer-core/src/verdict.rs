//! Verdicts: what an element shows because of what OpenSysML checked.
//! Satisfaction Status, Constraint Results and Analysis Results are its
//! kinds; each is judged on one scale and read in its own words.

use crate::model::{ClaimResult, Element, ElementKind, Model, QualifiedName};

/// Where a Verdict stands. With several, the worst decides: Violated, then
/// Undecided, then Absent, then Holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Status {
    Holds,
    Violated,
    Undecided,
    /// Nothing to judge yet: not claimed, not checked, not run.
    Absent,
}

impl Status {
    /// In the order pickers offer them.
    pub const ALL: [Status; 4] = [
        Status::Holds,
        Status::Violated,
        Status::Undecided,
        Status::Absent,
    ];

    /// The mark drawn on Diagrams: ✓ ✗ ? ○.
    pub fn mark(self) -> &'static str {
        match self {
            Status::Holds => "✓",
            Status::Violated => "✗",
            Status::Undecided => "?",
            Status::Absent => "○",
        }
    }

    /// What one check's result counts as; a check that couldn't run is
    /// Undecided.
    pub fn of(result: &ClaimResult) -> Status {
        match result {
            ClaimResult::Holds => Status::Holds,
            ClaimResult::Violated { .. } => Status::Violated,
            ClaimResult::Undecided { .. } | ClaimResult::Unchecked { .. } => Status::Undecided,
        }
    }

    fn severity(self) -> u8 {
        match self {
            Status::Holds => 0,
            Status::Absent => 1,
            Status::Undecided => 2,
            Status::Violated => 3,
        }
    }
}

/// The worst of `statuses`, `None` if there are none.
pub fn worst(statuses: impl IntoIterator<Item = Status>) -> Option<Status> {
    statuses.into_iter().max_by_key(|s| s.severity())
}

/// Which kind of Verdict, and so which words it is read in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// A requirement usage's Satisfaction Status.
    Requirement,
    /// An asserted constraint usage's Constraint Results.
    Constraint,
    /// An analysis usage's objective.
    Analysis,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::Requirement, Kind::Constraint, Kind::Analysis];

    /// `Satisfied`, `Holds`, `Not claimed`, `Not checked`, `Not run`, ….
    pub fn label(self, status: Status) -> &'static str {
        match (self, status) {
            (Kind::Requirement, Status::Holds) => "Satisfied",
            (_, Status::Holds) => "Holds",
            (_, Status::Violated) => "Violated",
            (_, Status::Undecided) => "Undecided",
            (Kind::Requirement, Status::Absent) => "Not claimed",
            (Kind::Constraint, Status::Absent) => "Not checked",
            (Kind::Analysis, Status::Absent) => "Not run",
        }
    }

    /// How a Style rule on this kind reads in a Legend: `Violated`,
    /// `Constraint violated`, `Analysis not run`.
    pub fn legend(self, status: Status) -> String {
        let label = self.label(status);
        match self {
            Kind::Requirement => label.to_owned(),
            Kind::Constraint => format!("Constraint {}", label.to_lowercase()),
            Kind::Analysis => format!("Analysis {}", label.to_lowercase()),
        }
    }
}

/// What one check behind a Verdict found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// What it is about: a claim's satisfier, the part a constraint was
    /// checked on; `None` for an analysis's objective.
    pub about: Option<QualifiedName>,
    pub status: Status,
    /// The failing condition, or why it couldn't be decided.
    pub detail: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    pub kind: Kind,
    pub status: Status,
    /// The element's own checks, in order. A requirement's nested
    /// requirements are rolled into `status` but are not findings.
    pub findings: Vec<Finding>,
}

impl Verdict {
    pub fn mark(&self) -> &'static str {
        self.status.mark()
    }

    pub fn label(&self) -> &'static str {
        self.kind.label(self.status)
    }
}

/// The Verdict of `element`: for a requirement usage, an asserted
/// constraint usage, or an analysis usage with an objective. `None` for
/// anything else.
pub fn of(model: &Model, element: &Element) -> Option<Verdict> {
    match element.kind {
        ElementKind::RequirementUsage => Some(Verdict {
            kind: Kind::Requirement,
            status: crate::satisfaction::status(model, element),
            findings: crate::satisfaction::claims_on(model, &element.qualified_name)
                .map(|claim| Finding {
                    about: claim.satisfier.clone(),
                    status: crate::satisfaction::claim_status(claim),
                    detail: claim
                        .claim_result
                        .as_ref()
                        .and_then(detail)
                        .map(str::to_owned),
                })
                .collect(),
        }),
        ElementKind::ConstraintUsage if element.is_asserted => {
            let findings: Vec<Finding> = element
                .checks
                .iter()
                .map(|check| Finding {
                    about: Some(check.part.clone()),
                    status: Status::of(&check.result),
                    detail: detail(&check.result).map(str::to_owned),
                })
                .collect();
            Some(Verdict {
                kind: Kind::Constraint,
                status: worst(findings.iter().map(|f| f.status)).unwrap_or(Status::Absent),
                findings,
            })
        }
        ElementKind::AnalysisUsage => {
            crate::analysis::objective(model, element)?;
            let findings: Vec<Finding> = element
                .analysis_result
                .iter()
                .map(|result| match &result.objective {
                    Some(outcome) => Finding {
                        about: None,
                        status: Status::of(outcome),
                        detail: detail(outcome).map(str::to_owned),
                    },
                    None => Finding {
                        about: None,
                        status: Status::Undecided,
                        detail: Some("no verdict".to_owned()),
                    },
                })
                .collect();
            Some(Verdict {
                kind: Kind::Analysis,
                status: findings.first().map_or(Status::Absent, |f| f.status),
                findings,
            })
        }
        _ => None,
    }
}

/// Every element with a Verdict, in qualified-name order.
pub fn all(model: &Model) -> impl Iterator<Item = (&Element, Verdict)> {
    model.elements().filter_map(|e| Some((e, of(model, e)?)))
}

/// The failing condition, or why a check couldn't be decided; `None` when
/// it holds.
pub fn detail(result: &ClaimResult) -> Option<&str> {
    match result {
        ClaimResult::Holds => None,
        ClaimResult::Violated { condition } => Some(condition),
        ClaimResult::Undecided { reason } | ClaimResult::Unchecked { reason } => Some(reason),
    }
}

/// How one check's result reads in the Properties panel: `✓ holds`,
/// `✗ violated: v.range >= 500.0`, `? undecided: no value for v.cost`,
/// `? not checked` when there is no result yet.
pub fn describe(result: Option<&ClaimResult>) -> String {
    match result {
        Some(ClaimResult::Holds) => "✓ holds".to_owned(),
        Some(ClaimResult::Violated { condition }) => format!("✗ violated: {condition}"),
        Some(ClaimResult::Undecided { reason }) => format!("? undecided: {reason}"),
        Some(ClaimResult::Unchecked { reason }) => format!("? not checked: {reason}"),
        None => "? not checked".to_owned(),
    }
}
