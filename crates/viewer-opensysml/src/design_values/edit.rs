//! The edit: a parsed Model with its Design Values (and markers) written
//! in, in memory, and parsed again. The files are never written (ADR-0002).

use std::collections::BTreeMap;

use opensysml::{Connection, SourceDocument, SourcesOptions};

use super::plan::{EditorEdit, ValueEdit, ValuePlace};

/// `parsed` with `edits` made in memory, one editor per document, and
/// parsed again: the edited documents inline, the rest from `documents`.
/// `None` when there is nothing to edit. The files are never written.
pub(crate) fn with_design_values(
    connection: &Connection,
    parsed: &opensysml::Model,
    documents: &[SourceDocument],
    edits: &[ValueEdit],
) -> Result<Option<opensysml::Model>, String> {
    if edits.is_empty() {
        return Ok(None);
    }
    let mut in_text: Vec<(&ValueEdit, (u32, u32))> = Vec::new();
    let mut per_document: BTreeMap<&str, Vec<(&ValueEdit, &EditorEdit)>> = BTreeMap::new();
    for edit in edits {
        match &edit.place {
            ValuePlace::InText { at } => in_text.push((edit, *at)),
            ValuePlace::Editor(editor_edit) => per_document
                .entry(&edit.document)
                .or_default()
                .push((edit, editor_edit)),
        }
    }
    // The values the service won't write go into the text first, the last
    // in each document first, so the places of the others stay as parsed.
    in_text.sort_by(|(a, at_a), (b, at_b)| (&b.document, at_b).cmp(&(&a.document, at_a)));
    let mut edited: BTreeMap<String, String> = BTreeMap::new();
    for (edit, at) in in_text {
        let text = match edited.remove(&edit.document) {
            Some(text) => text,
            None => std::fs::read_to_string(&edit.document)
                .map_err(|e| format!("cannot read {}: {e}", edit.document))?,
        };
        let written = write_value(&text, at, &edit.value).ok_or_else(|| {
            format!(
                "no declaration of {} at line {}, column {} of {}",
                edit.feature, at.0, at.1, edit.document
            )
        })?;
        edited.insert(edit.document.clone(), written);
    }
    let reparsed = match edited.is_empty() {
        true => None,
        false => Some(
            connection
                .parse_sources(&with_edited(documents, &edited), &SourcesOptions::default())
                .map_err(|e| e.to_string())?,
        ),
    };
    let base = reparsed.as_ref().unwrap_or(parsed);

    if per_document.is_empty() {
        return Ok(reparsed);
    }
    for (document, edits) in per_document {
        let mut editor = base.edit();
        editor.in_document(document);
        for (edit, editor_edit) in edits {
            match editor_edit {
                EditorEdit::SetValue => {
                    editor.set_value(edit.feature.as_str(), &edit.value);
                }
                EditorEdit::Redefine { owner, redefines } => {
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
        for result in editor.apply().map_err(|e| e.to_string())?.documents {
            edited.insert(result.name, result.content);
        }
    }
    connection
        .parse_sources(&with_edited(documents, &edited), &SourcesOptions::default())
        .map(Some)
        .map_err(|e| e.to_string())
}

/// `documents`, those in `edited` with their edited text inline.
fn with_edited(
    documents: &[SourceDocument],
    edited: &BTreeMap<String, String>,
) -> Vec<SourceDocument> {
    documents
        .iter()
        .map(|d| {
            let name = d.document_name();
            match edited.get(&name) {
                Some(content) => SourceDocument::inline(name, content.clone()),
                None => d.clone(),
            }
        })
        .collect()
}

/// `text` with ` = <value>` written into the declaration beginning at
/// `at` (line from 1, column from 1 in UTF-8 bytes, as the service counts
/// it), before its body or its terminating `;`.
/// `None` when neither follows: `at` is no declaration of this text.
fn write_value(text: &str, at: (u32, u32), value: &str) -> Option<String> {
    let (line, column) = at;
    let line_start = match line {
        0 => return None,
        1 => 0,
        _ => text.match_indices('\n').nth(line as usize - 2)?.0 + 1,
    };
    // The service counts the column in UTF-8 bytes, not characters.
    let start = line_start + column.checked_sub(1)? as usize;
    if !text.is_char_boundary(start) || text[line_start..start].contains('\n') {
        return None;
    }
    // The header runs to the body's `{` or the `;`; neither counts inside
    // a string, a quoted name or a comment.
    let header = &text[start..];
    let mut chars = header.char_indices();
    let end = loop {
        let (i, c) = chars.next()?;
        match c {
            '{' | ';' => break start + i,
            '"' | '\'' => {
                chars.by_ref().find(|&(_, n)| n == c)?;
            }
            '/' if header[i..].starts_with("//") => {
                chars.by_ref().find(|&(_, n)| n == '\n')?;
            }
            '/' if header[i..].starts_with("/*") => {
                chars.next();
                chars
                    .by_ref()
                    .find(|&(j, _)| header[j..].starts_with("*/"))?;
                chars.next();
            }
            _ => {}
        }
    };
    // Not past a line break, which may end a `//` comment.
    let head = text[..end].trim_end_matches([' ', '\t']);
    Some(format!("{head} = {value} {}", &text[end..]))
}

#[cfg(test)]
mod tests {
    use super::write_value;

    #[test]
    fn a_value_is_written_before_the_body() {
        let text =
            "part p {\n    attribute bore : LengthValue { @ToolProperty { name = \"D\"; } }\n}\n";
        assert_eq!(
            write_value(text, (2, 5), "8.0 [mm]").as_deref(),
            Some(
                "part p {\n    attribute bore : LengthValue = 8.0 [mm] { @ToolProperty { name = \"D\"; } }\n}\n"
            )
        );
    }

    #[test]
    fn a_value_is_written_before_the_semicolon_of_a_declaration_without_a_body() {
        assert_eq!(
            write_value(
                "attribute bore : LengthValue;",
                (1, 1),
                "\"no Design Value #0#\""
            )
            .as_deref(),
            Some("attribute bore : LengthValue = \"no Design Value #0#\" ;")
        );
    }

    #[test]
    fn braces_in_quoted_names_and_comments_are_not_the_body() {
        let text = "attribute 'b{re' /* { */ : LengthValue // {\n    { }";
        assert_eq!(
            write_value(text, (1, 1), "1 [mm]").as_deref(),
            Some("attribute 'b{re' /* { */ : LengthValue // {\n = 1 [mm] { }")
        );
    }

    #[test]
    fn a_place_that_starts_no_declaration_writes_nothing() {
        assert_eq!(
            write_value("attribute bore : LengthValue", (1, 1), "1"),
            None
        );
        assert_eq!(write_value("attribute bore;", (3, 1), "1"), None);
        assert_eq!(write_value("attribute bore;", (1, 40), "1"), None);
    }
}
