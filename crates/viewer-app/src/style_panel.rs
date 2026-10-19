//! The Style panel's rule list: add, remove, reorder and change Style
//! Rules, choosing kinds, packages and metadata defs from the Model.

use eframe::egui;
use viewer_core::scene::Rgb;
use viewer_core::style::{self, Condition, KINDS, Satisfaction, StyleRule};
use viewer_core::{ElementKind, Model, QualifiedName};

/// What an edit of the rule list did: `changed` the rules at all, and
/// whether the change is finished (`commit`) rather than a label still
/// being typed.
#[derive(Default)]
pub struct Edited {
    pub changed: bool,
    pub commit: bool,
}

impl Edited {
    fn finished(&mut self) {
        self.changed = true;
        self.commit = true;
    }
}

/// Shows `rules` for editing; `salt` keeps two lists' widgets apart.
pub fn rule_list(
    ui: &mut egui::Ui,
    salt: &str,
    rules: &mut Vec<StyleRule>,
    model: &Model,
) -> Edited {
    let mut edited = Edited::default();
    let packages = names_of(model, |kind| *kind == ElementKind::Package);
    let metadata = names_of(
        model,
        |kind| matches!(kind, ElementKind::Unsupported(k) if k == "metadata def"),
    );
    if rules.is_empty() {
        ui.weak("No rules.");
    }
    let mut action: Option<(usize, Action)> = None;
    for (index, rule) in rules.iter_mut().enumerate() {
        ui.push_id((salt, index), |ui| {
            ui.horizontal_wrapped(|ui| {
                let mut color = [rule.fill.0, rule.fill.1, rule.fill.2];
                if ui.color_edit_button_srgb(&mut color).changed() {
                    rule.fill = Rgb(color[0], color[1], color[2]);
                    edited.finished();
                }
                if condition_picker(ui, &mut rule.condition, &packages, &metadata) {
                    edited.finished();
                }
                if !rule.condition.exists_in(model) {
                    ui.colored_label(ui.visuals().warn_fg_color, "⚠")
                        .on_hover_text(
                            "Not in the Model: this rule matches nothing until it is back",
                        );
                }
            });
            ui.horizontal(|ui| {
                let mut label = rule.label.clone().unwrap_or_default();
                let field = ui.add(
                    egui::TextEdit::singleline(&mut label)
                        .desired_width(140.0)
                        .hint_text(style::legend_label(&StyleRule {
                            label: None,
                            ..rule.clone()
                        })),
                );
                if field.changed() {
                    rule.label = Some(label.trim().to_owned()).filter(|l| !l.is_empty());
                    edited.changed = true;
                }
                if field.lost_focus() {
                    edited.commit = true;
                }
                if ui
                    .small_button("↑")
                    .on_hover_text("Earlier: wins ties")
                    .clicked()
                {
                    action = Some((index, Action::Up));
                }
                if ui.small_button("↓").on_hover_text("Later").clicked() {
                    action = Some((index, Action::Down));
                }
                if ui.small_button("✕").on_hover_text("Remove").clicked() {
                    action = Some((index, Action::Remove));
                }
            });
            ui.separator();
        });
    }
    match action {
        Some((index, Action::Up)) => style::move_up(rules, index),
        Some((index, Action::Down)) => style::move_down(rules, index),
        Some((index, Action::Remove)) => {
            rules.remove(index);
        }
        None => {}
    }
    if action.is_some() {
        edited.finished();
    }
    if ui.button("Add rule").clicked() {
        rules.push(style::new_rule());
        edited.finished();
    }
    edited
}

enum Action {
    Up,
    Down,
    Remove,
}

fn names_of(model: &Model, kind: impl Fn(&ElementKind) -> bool) -> Vec<QualifiedName> {
    model
        .elements()
        .filter(|e| kind(&e.kind))
        .map(|e| e.qualified_name.clone())
        .collect()
}

/// The condition's type, then what it names; true if either changed.
fn condition_picker(
    ui: &mut egui::Ui,
    condition: &mut Condition,
    packages: &[QualifiedName],
    metadata: &[QualifiedName],
) -> bool {
    let mut changed = false;
    let type_name = match condition {
        Condition::Kind(_) => "Kind",
        Condition::Package(_) => "Package",
        Condition::Metadata(_) => "Metadata",
        Condition::Satisfaction(_) => "Satisfaction",
    };
    egui::ComboBox::from_id_salt("type")
        .selected_text(type_name)
        .width(80.0)
        .show_ui(ui, |ui| {
            let first = |names: &[QualifiedName]| {
                names
                    .first()
                    .cloned()
                    .unwrap_or_else(|| QualifiedName::new(""))
            };
            let options = [
                ("Kind", Condition::Kind(KINDS[0].to_owned())),
                ("Package", Condition::Package(first(packages))),
                ("Metadata", Condition::Metadata(first(metadata))),
                (
                    "Satisfaction",
                    Condition::Satisfaction(Satisfaction::Violated),
                ),
            ];
            for (name, fresh) in options {
                if ui.selectable_label(type_name == name, name).clicked() && type_name != name {
                    *condition = fresh;
                    changed = true;
                }
            }
        });
    match condition {
        Condition::Kind(keyword) => {
            egui::ComboBox::from_id_salt("kind")
                .selected_text(format!("«{keyword}»"))
                .show_ui(ui, |ui| {
                    for kind in KINDS {
                        if ui
                            .selectable_label(keyword == kind, format!("«{kind}»"))
                            .clicked()
                            && keyword != kind
                        {
                            *keyword = kind.to_owned();
                            changed = true;
                        }
                    }
                });
        }
        Condition::Package(name) => changed |= name_picker(ui, name, packages),
        Condition::Metadata(name) => changed |= name_picker(ui, name, metadata),
        Condition::Satisfaction(wanted) => {
            egui::ComboBox::from_id_salt("satisfaction")
                .selected_text(format!(
                    "{} {}",
                    wanted.status().mark(),
                    wanted.status().label()
                ))
                .show_ui(ui, |ui| {
                    for option in Satisfaction::ALL {
                        let text =
                            format!("{} {}", option.status().mark(), option.status().label());
                        if ui.selectable_label(*wanted == option, text).clicked()
                            && *wanted != option
                        {
                            *wanted = option;
                            changed = true;
                        }
                    }
                });
        }
    }
    changed
}

/// Chooses `name` among `choices`; true if it changed.
fn name_picker(ui: &mut egui::Ui, name: &mut QualifiedName, choices: &[QualifiedName]) -> bool {
    let mut changed = false;
    let shown = if name.as_str().is_empty() {
        "(none in the Model)".to_owned()
    } else {
        name.to_string()
    };
    egui::ComboBox::from_id_salt("name")
        .selected_text(shown)
        .show_ui(ui, |ui| {
            for choice in choices {
                if ui
                    .selectable_label(name == choice, choice.as_str())
                    .clicked()
                    && name != choice
                {
                    *name = choice.clone();
                    changed = true;
                }
            }
        });
    changed
}
