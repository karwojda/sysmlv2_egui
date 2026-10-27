//! Design Values (ADR-0003 to ADR-0006): checks a parsed Model with the
//! values its Tool Links read from Engineering Tools in place of its
//! Model Values. The plan asks the tools (`plan.rs`), the edit writes
//! the values into the Model in memory and parses it again (`edit.rs`),
//! and the checks' reasons about them are put in words (`words.rs`).

mod edit;
mod plan;
mod words;

pub(crate) use edit::with_design_values;
pub(crate) use plan::{HeldLink, LinkedProperty, ToolLinks, plan};
pub(crate) use words::Worded;
