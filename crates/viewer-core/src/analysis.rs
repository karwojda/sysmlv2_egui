//! Analysis Results: what running analysis usages found. An analysis's
//! Verdict, from its objective, is in `verdict`.

use crate::model::{Element, Model, Objective};

/// An analysis's objective: its own, or its definition's.
pub fn objective<'a>(model: &'a Model, analysis: &'a Element) -> Option<&'a Objective> {
    analysis.objective.as_ref().or_else(|| {
        analysis
            .typed_by
            .as_ref()
            .and_then(|d| model.element(d))
            .and_then(|d| d.objective.as_ref())
    })
}

/// The rows of an analysis usage's `results` compartment: each output with
/// its value, `return` marking its definition's result parameter.
pub fn result_rows(model: &Model, element: &Element) -> Vec<String> {
    let Some(result) = &element.analysis_result else {
        return Vec::new();
    };
    let definition = element.typed_by.as_ref();
    result
        .outputs
        .iter()
        .map(|(name, value)| {
            let parameter = definition
                .map(|d| crate::model::QualifiedName::new(format!("{d}::{name}")))
                .and_then(|p| model.element(&p).cloned());
            let returned = parameter.as_ref().is_some_and(|p| p.is_return);
            let typed = parameter
                .as_ref()
                .and_then(|p| p.typed_by.as_ref())
                .map(|t| format!(":{}", t.simple_name()))
                .unwrap_or_default();
            let keyword = if returned { "return " } else { "" };
            format!("{keyword}{name}{typed} = {value}")
        })
        .collect()
}
