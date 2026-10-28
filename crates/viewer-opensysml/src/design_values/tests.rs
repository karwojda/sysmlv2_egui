//! Design Values across `check`, with a fake tool and a `Store` whose
//! checkers answer from tables: what the tools are asked, what is written
//! into the Model, what the checks' reasons say, and what is judged when
//! nothing can be written.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use tempfile::TempDir;
use viewer_core::engineering_tools::{Answer, ReadValue, Wanted};
use viewer_core::model::{AnalysisResult, ElementKind};

use super::plan::{EditorEdit, ValuePlace};
use super::*;
use crate::checks::tests::{Answers, Elements, qn, violated};
use crate::checks::{ClaimOutcome, ConstraintOutcome, PartCheck};

const DOCUMENT: &str = "/model/drive.sysml";

fn undecided(reason: &str) -> ClaimResult {
    ClaimResult::Undecided {
        reason: reason.to_owned(),
    }
}

/// `@ToolLink { tool; file; object; }` on `element`, in [`DOCUMENT`].
fn link(element: &str, tool: &str, file: &str, object: &str) -> HeldLink {
    HeldLink {
        link: ToolLink {
            element: qn(element),
            tool: tool.to_owned(),
            file: file.to_owned(),
            object: object.to_owned(),
        },
        document: DOCUMENT.to_owned(),
    }
}

/// `@ToolProperty { name = property; }` on `feature`, of `owner`'s link,
/// writing a value of its own.
fn property(feature: &str, owner: &str, property: &str) -> LinkedProperty {
    LinkedProperty {
        feature: qn(feature),
        owner: qn(owner),
        property: property.to_owned(),
        has_value: true,
        redefines: Vec::new(),
        named_at: None,
        document: DOCUMENT.to_owned(),
        source: None,
    }
}

fn links(links: Vec<HeldLink>, properties: Vec<LinkedProperty>) -> ToolLinks {
    ToolLinks { links, properties }
}

/// The engine linked to FreeCAD for its volume, as `property` has it.
fn engine_volume(property: LinkedProperty) -> ToolLinks {
    links(
        vec![link("D::engine", "FreeCAD", "cad/engine.FCStd", "Engine")],
        vec![property],
    )
}

/// Every call a fake tool was given: the file and what was wanted of it.
type Calls = Arc<Mutex<Vec<(PathBuf, Wanted)>>>;

/// Answers from a table of objects' properties (a value, or why there is
/// none), or that the whole file can't be read, and records each call.
#[derive(Clone, Default)]
struct Tool {
    values: Vec<(
        &'static str,
        &'static str,
        Result<&'static str, &'static str>,
    )>,
    unreadable: Option<&'static str>,
    calls: Calls,
}

impl Tool {
    fn answering(
        values: Vec<(
            &'static str,
            &'static str,
            Result<&'static str, &'static str>,
        )>,
    ) -> Self {
        Self {
            values,
            ..Self::default()
        }
    }
}

impl EngineeringTool for Tool {
    fn read(&mut self, file: &Path, wanted: &Wanted) -> Result<Answer, String> {
        self.calls
            .lock()
            .unwrap()
            .push((file.to_path_buf(), wanted.clone()));
        if let Some(reason) = self.unreadable {
            return Err(reason.to_owned());
        }
        let mut answer = Answer::new();
        for (object, property, value) in &self.values {
            answer.entry((*object).to_owned()).or_default().insert(
                (*property).to_owned(),
                value.map(ReadValue::new).map_err(str::to_owned),
            );
        }
        Ok(answer)
    }
}

impl Checker for &Answers {
    fn verify_claims(&self) -> Result<Vec<ClaimOutcome>, String> {
        (**self).verify_claims()
    }

    fn check_part(&self, part: &QualifiedName) -> Result<PartCheck, String> {
        (**self).check_part(part)
    }

    fn run_analysis(&self, analysis: &QualifiedName) -> (Result<AnalysisResult, String>, Duration) {
        (**self).run_analysis(analysis)
    }
}

/// A Model checked from tables: `parsed` as parsed, `designed` with the
/// edits written into it, or why they could not be. Records the edits.
#[derive(Default)]
struct Tables {
    parsed: Answers,
    designed: Option<Result<Answers, String>>,
    edits: Option<Vec<ValueEdit>>,
}

impl Store for Tables {
    fn checker(&self) -> Box<dyn Checker + '_> {
        Box::new(&self.parsed)
    }

    fn with_edits(&mut self, edits: &[ValueEdit]) -> Result<Box<dyn Checker + '_>, String> {
        self.edits = Some(edits.to_vec());
        match self.designed.get_or_insert_with(|| Ok(Answers::default())) {
            Ok(designed) => Ok(Box::new(&*designed)),
            Err(reason) => Err(reason.clone()),
        }
    }
}

/// What a load found: what `check` returned, the elements afterwards, the
/// edits written, the calls the tool was given and what each checker was
/// asked.
struct Loaded {
    checked: Checked,
    elements: BTreeMap<QualifiedName, Element>,
    edits: Option<Vec<ValueEdit>>,
    calls: Vec<(PathBuf, Wanted)>,
    parsed_asked: Vec<String>,
    designed_asked: Vec<String>,
}

impl Loaded {
    fn messages(&self) -> Vec<&str> {
        self.checked
            .diagnostics
            .iter()
            .map(|d| d.message.as_str())
            .collect()
    }

    fn design_values(&self) -> Vec<(&str, &DesignValue)> {
        self.checked
            .outcomes
            .iter()
            .map(|o| (o.feature.as_str(), &o.design_value))
            .collect()
    }

    fn written(&self) -> Vec<(&str, &str)> {
        self.edits
            .iter()
            .flatten()
            .map(|e| (e.feature.as_str(), e.value.as_str()))
            .collect()
    }

    fn claim(&self, name: &str) -> ClaimResult {
        self.elements[&qn(name)].claim_result.clone().unwrap()
    }
}

/// Checks `elements` with the Tool Links `found`, FreeCAD being `tool`.
fn load(elements: Elements, found: ToolLinks, tool: Tool, mut tables: Tables) -> Loaded {
    let calls = tool.calls.clone();
    let mut tools: BTreeMap<String, Box<dyn EngineeringTool>> = [(
        "FreeCAD".to_owned(),
        Box::new(tool) as Box<dyn EngineeringTool>,
    )]
    .into();
    let mut elements = elements.0;
    let folder = TempDir::new().unwrap();
    let checked = check(
        &mut tables,
        &found,
        &mut tools,
        &mut elements,
        folder.path(),
        &Analyses::AsSet,
    );
    let designed_asked = match tables.designed {
        Some(Ok(designed)) => designed.asked.into_inner(),
        _ => Vec::new(),
    };
    Loaded {
        checked,
        elements,
        edits: tables.edits,
        calls: calls.lock().unwrap().clone(),
        parsed_asked: tables.parsed.asked.into_inner(),
        designed_asked,
    }
}

fn claimed() -> Elements {
    Elements::default()
        .package("D")
        .claim("D::@1", "D::r", "D::spare")
}

/// The claim checked by `designed`, answering `result` for it.
fn claim_answers(result: ClaimResult) -> Tables {
    Tables {
        designed: Some(Ok(Answers {
            claims: Some(Ok(vec![ClaimOutcome {
                requirement: qn("D::r"),
                satisfier: Some(qn("D::spare")),
                result,
            }])),
            ..Answers::default()
        })),
        ..Tables::default()
    }
}

mod plan {
    use super::*;

    #[test]
    fn a_model_without_tool_links_is_checked_as_parsed_and_asks_no_tool() {
        let tables = Tables {
            parsed: Answers {
                claims: Some(Ok(vec![ClaimOutcome {
                    requirement: qn("D::r"),
                    satisfier: None,
                    result: ClaimResult::Holds,
                }])),
                ..Answers::default()
            },
            ..Tables::default()
        };

        let loaded = load(claimed(), ToolLinks::default(), Tool::default(), tables);

        assert_eq!(loaded.claim("D::@1"), ClaimResult::Holds);
        assert!(loaded.calls.is_empty());
        assert!(loaded.edits.is_none(), "nothing is written");
        assert!(loaded.checked.tool_links.is_empty() && loaded.checked.outcomes.is_empty());
    }

    #[test]
    fn each_tool_is_asked_once_per_file_for_every_object_and_property() {
        let found = links(
            vec![
                link("D::engine", "FreeCAD", "cad/drive.FCStd", "Engine"),
                link("D::pump", "FreeCAD", "cad/drive.FCStd", "Pump"),
            ],
            vec![
                property("D::engine::volume", "D::engine", "Volume"),
                property("D::engine::area", "D::engine", "Area"),
                property("D::pump::volume", "D::pump", "Volume"),
            ],
        );

        let loaded = load(
            Elements::default(),
            found,
            Tool::default(),
            Tables::default(),
        );

        let wanted: Wanted = [
            (
                "Engine".to_owned(),
                ["Area".to_owned(), "Volume".to_owned()].into(),
            ),
            ("Pump".to_owned(), ["Volume".to_owned()].into()),
        ]
        .into();
        assert_eq!(
            loaded.calls,
            [(PathBuf::from("/model/cad/drive.FCStd"), wanted)],
            "the file is relative to the document holding the link"
        );
        assert_eq!(loaded.checked.tool_links.len(), 2);
    }

    #[test]
    fn features_take_the_values_read_and_markers_where_there_are_none() {
        let found = links(
            vec![link("D::engine", "FreeCAD", "cad/engine.FCStd", "Engine")],
            vec![
                property("D::engine::volume", "D::engine", "Volume"),
                property("D::engine::area", "D::engine", "Area"),
                property("D::engine::mass", "D::engine", "Mass"),
            ],
        );
        let tool = Tool::answering(vec![
            ("Engine", "Volume", Ok("1000000.0 [mm^3]")),
            ("Engine", "Area", Err("no property Area")),
        ]);

        let loaded = load(Elements::default(), found, tool, Tables::default());

        assert_eq!(
            loaded.written(),
            [
                ("D::engine::volume", "1000000.0 [mm^3]"),
                ("D::engine::area", "\"no Design Value #0#\""),
                ("D::engine::mass", "\"no Design Value #1#\""),
            ]
        );
        assert_eq!(
            loaded.design_values(),
            [
                (
                    "D::engine::volume",
                    &DesignValue::Read {
                        value: "1000000.0 [mm^3]".to_owned(),
                        parts: None,
                    }
                ),
                (
                    "D::engine::area",
                    &DesignValue::Missing {
                        reason: "no property Area".to_owned()
                    }
                ),
                (
                    "D::engine::mass",
                    &DesignValue::Missing {
                        reason: "gave no answer for Mass of Engine".to_owned()
                    }
                ),
            ]
        );
    }

    #[test]
    fn a_file_the_tool_cannot_read_leaves_every_feature_linked_to_it_without_a_value() {
        let tool = Tool {
            unreadable: Some("FreeCAD not available"),
            ..Tool::default()
        };

        let loaded = load(
            Elements::default(),
            engine_volume(property("D::engine::volume", "D::engine", "Volume")),
            tool,
            Tables::default(),
        );

        assert_eq!(
            loaded.design_values(),
            [(
                "D::engine::volume",
                &DesignValue::Missing {
                    reason: "FreeCAD not available".to_owned()
                }
            )]
        );
    }

    #[test]
    fn an_element_with_two_links_to_one_tool_takes_no_design_value_and_warns() {
        let found = links(
            vec![
                link("D::engine", "FreeCAD", "cad/a.FCStd", "A"),
                link("D::engine", "FreeCAD", "cad/b.FCStd", "B"),
            ],
            vec![property("D::engine::volume", "D::engine", "Volume")],
        );

        let loaded = load(
            Elements::default(),
            found,
            Tool::default(),
            Tables::default(),
        );

        assert!(loaded.calls.is_empty(), "{:?}", loaded.calls);
        assert_eq!(
            loaded.messages(),
            ["D::engine has 2 Tool Links to FreeCAD: its features take no Design Value"]
        );
        assert_eq!(
            loaded.design_values(),
            [(
                "D::engine::volume",
                &DesignValue::Missing {
                    reason: "D::engine has 2 Tool Links to FreeCAD".to_owned()
                }
            )]
        );
    }

    #[test]
    fn a_tool_property_without_a_tool_link_warns_and_takes_nothing() {
        let found = links(
            Vec::new(),
            vec![property("D::engine::volume", "D::engine", "Volume")],
        );

        let loaded = load(
            Elements::default(),
            found,
            Tool::default(),
            Tables::default(),
        );

        assert_eq!(
            loaded.messages(),
            [
                "@ToolProperty on D::engine::volume, but D::engine has no Tool Link: it takes no Design Value"
            ]
        );
        assert!(loaded.checked.outcomes.is_empty());
        assert_eq!(loaded.written(), []);
    }

    #[test]
    fn a_tool_this_program_does_not_know_gives_no_design_value() {
        let found = links(
            vec![link("D::board", "Eagle", "pcb/board.brd", "Board")],
            vec![property("D::board::power", "D::board", "Power")],
        );

        let loaded = load(
            Elements::default(),
            found,
            Tool::default(),
            Tables::default(),
        );

        assert_eq!(
            loaded.design_values(),
            [(
                "D::board::power",
                &DesignValue::Missing {
                    reason: "no Engineering Tool named Eagle".to_owned()
                }
            )]
        );
    }

    #[test]
    fn the_model_value_is_the_feature_s_own_or_the_one_it_redefines() {
        let elements = Elements::default()
            .add("D::Engine::volume", ElementKind::AttributeUsage, |e| {
                e.value = Some("0.2e6 [mm^3]".to_owned())
            })
            .add("D::engine::volume", ElementKind::AttributeUsage, |_| {})
            .add("D::pump::volume", ElementKind::AttributeUsage, |e| {
                e.value = Some("0.9e6 [mm^3]".to_owned())
            });
        let found = links(
            vec![
                link("D::engine", "FreeCAD", "cad/drive.FCStd", "Engine"),
                link("D::pump", "FreeCAD", "cad/drive.FCStd", "Pump"),
            ],
            vec![
                LinkedProperty {
                    redefines: vec![qn("D::Engine::volume")],
                    ..property("D::engine::volume", "D::engine", "Volume")
                },
                property("D::pump::volume", "D::pump", "Volume"),
            ],
        );

        let loaded = load(elements, found, Tool::default(), Tables::default());

        let model_values: Vec<Option<&str>> = loaded
            .checked
            .outcomes
            .iter()
            .map(|o| o.model_value.as_deref())
            .collect();
        assert_eq!(model_values, [Some("0.2e6 [mm^3]"), Some("0.9e6 [mm^3]")]);
    }

    #[test]
    fn a_value_is_set_or_written_into_the_text_or_redefines_by_how_the_feature_is_declared() {
        let found = links(
            vec![link("D::engine", "FreeCAD", "cad/engine.FCStd", "Engine")],
            vec![
                property("D::engine::volume", "D::engine", "Volume"),
                LinkedProperty {
                    has_value: false,
                    named_at: Some((4, 9)),
                    ..property("D::engine::bore", "D::engine", "Bore")
                },
                LinkedProperty {
                    has_value: false,
                    ..property("D::engine::mass", "D::engine", "Mass")
                },
            ],
        );

        let loaded = load(
            Elements::default(),
            found,
            Tool::default(),
            Tables::default(),
        );

        let places: Vec<&ValuePlace> = loaded.edits.iter().flatten().map(|e| &e.place).collect();
        assert_eq!(
            places,
            [
                &ValuePlace::Editor(EditorEdit::SetValue),
                &ValuePlace::InText { at: (4, 9) },
                &ValuePlace::Editor(EditorEdit::Redefine {
                    owner: qn("D::engine"),
                    redefines: "mass".to_owned(),
                }),
            ]
        );
    }
}

mod words {
    use super::*;

    /// The engines of the drive and of the spare linked to FreeCAD for
    /// their volume, read as `drive` and `spare`.
    fn two_engines(drive: &'static str, spare: &'static str) -> (ToolLinks, Tool) {
        let found = links(
            vec![
                link("D::drive::engine", "FreeCAD", "cad/drive.FCStd", "Drive"),
                link("D::spare::engine", "FreeCAD", "cad/drive.FCStd", "Spare"),
            ],
            vec![
                property("D::drive::engine::volume", "D::drive::engine", "Volume"),
                property("D::spare::engine::volume", "D::spare::engine", "Volume"),
            ],
        );
        let tool = Tool::answering(vec![
            ("Drive", "Volume", Ok(drive)),
            ("Spare", "Volume", Ok(spare)),
        ]);
        (found, tool)
    }

    /// What the service says after `type mismatch: `.
    const MISMATCH: &str = "cannot write 1000000.0 [mm**3] (dimension L^3) to a feature typed by MassValue (dimension M)";

    fn mismatch() -> String {
        format!("feature value engine.volume: type mismatch: {MISMATCH}")
    }

    #[test]
    fn a_reason_naming_a_marker_says_which_design_value_is_missing() {
        let tool = Tool::answering(vec![("Engine", "Volume", Err("no object Engine"))]);

        let loaded = load(
            claimed(),
            engine_volume(property("D::engine::volume", "D::engine", "Volume")),
            tool,
            claim_answers(undecided(
                "cannot compare \"no Design Value #0#\" with 1.5 [m^3]",
            )),
        );

        assert_eq!(
            loaded.claim("D::@1"),
            undecided("no Design Value for D::engine::volume (FreeCAD: no object Engine)")
        );
    }

    #[test]
    fn a_reason_naming_two_markers_names_both() {
        let found = links(
            vec![link("D::engine", "FreeCAD", "cad/engine.FCStd", "Engine")],
            vec![
                property("D::engine::volume", "D::engine", "Volume"),
                property("D::engine::area", "D::engine", "Area"),
            ],
        );

        let loaded = load(
            claimed(),
            found,
            Tool::default(),
            claim_answers(undecided(
                "\"no Design Value #1#\" + \"no Design Value #0#\" is not a number",
            )),
        );

        assert_eq!(
            loaded.claim("D::@1"),
            undecided(
                "no Design Value for D::engine::volume (FreeCAD: gave no answer for Volume of Engine); \
                 no Design Value for D::engine::area (FreeCAD: gave no answer for Area of Engine)"
            )
        );
    }

    #[test]
    fn a_type_mismatch_names_the_design_value_beneath_the_satisfier() {
        let (found, tool) = two_engines("1000000.0 [mm^3]", "1000000.0 [mm^3]");

        let loaded = load(
            claimed(),
            found,
            tool,
            claim_answers(undecided(&mismatch())),
        );

        assert_eq!(
            loaded.claim("D::@1"),
            undecided(&format!(
                "Design Value for D::spare::engine::volume (FreeCAD: Spare.Volume) does not fit it: {MISMATCH}"
            ))
        );
    }

    #[test]
    fn a_type_mismatch_names_the_design_value_beneath_the_part_at_the_constraint_s_path() {
        let found = links(
            vec![
                link("D::drive::engine", "FreeCAD", "cad/drive.FCStd", "Drive"),
                link(
                    "D::drive::spare::engine",
                    "FreeCAD",
                    "cad/drive.FCStd",
                    "Spare",
                ),
            ],
            vec![
                property("D::drive::engine::volume", "D::drive::engine", "Volume"),
                property(
                    "D::drive::spare::engine::volume",
                    "D::drive::spare::engine",
                    "Volume",
                ),
            ],
        );
        let tool = Tool::answering(vec![
            ("Drive", "Volume", Ok("1000000.0 [mm^3]")),
            ("Spare", "Volume", Ok("1000000.0 [mm^3]")),
        ]);
        let elements = Elements::default()
            .package("D")
            .part("D::drive")
            .asserted("D::drive::fits");
        let tables = Tables {
            designed: Some(Ok(Answers {
                parts: [(
                    qn("D::drive"),
                    Ok(PartCheck {
                        constraints: vec![ConstraintOutcome {
                            constraint: qn("D::drive::fits"),
                            path: "spare".to_owned(),
                            result: undecided(&mismatch()),
                        }],
                        values: Vec::new(),
                    }),
                )]
                .into(),
                ..Answers::default()
            })),
            ..Tables::default()
        };

        let loaded = load(elements, found, tool, tables);

        let results: Vec<&ClaimResult> = loaded.elements[&qn("D::drive::fits")]
            .checks
            .iter()
            .map(|c| &c.result)
            .collect();
        assert_eq!(
            results,
            [&undecided(&format!(
                "Design Value for D::drive::spare::engine::volume (FreeCAD: Spare.Volume) does not fit it: {MISMATCH}"
            ))]
        );
    }

    #[test]
    fn a_type_mismatch_beneath_no_part_names_the_value_refused_or_every_one_that_could_be() {
        let analysis = |reason: String| Tables {
            designed: Some(Ok(Answers {
                analyses: [(
                    qn("D::mass"),
                    Ok(AnalysisResult {
                        outputs: Vec::new(),
                        objective: Some(undecided(&reason)),
                    }),
                )]
                .into(),
                ..Answers::default()
            })),
            ..Tables::default()
        };
        let objective = |loaded: &Loaded| {
            loaded.elements[&qn("D::mass")]
                .analysis_result
                .clone()
                .unwrap()
                .objective
                .unwrap()
        };
        let elements = || Elements::default().package("D").analysis("D::mass");

        let (found, tool) = two_engines("2.0 [mm^3]", "1000000.0 [mm^3]");
        let refused = load(elements(), found, tool, analysis(mismatch()));
        let (found, tool) = two_engines("1000000.0 [mm^3]", "1000000.0 [mm^3]");
        let either = load(elements(), found, tool, analysis(mismatch()));

        assert_eq!(
            objective(&refused),
            undecided(&format!(
                "Design Value for D::spare::engine::volume (FreeCAD: Spare.Volume) does not fit it: {MISMATCH}"
            ))
        );
        assert_eq!(
            objective(&either),
            undecided(&format!(
                "Design Value for D::drive::engine::volume (FreeCAD: Drive.Volume) or \
                 Design Value for D::spare::engine::volume (FreeCAD: Spare.Volume) does not fit it: {MISMATCH}"
            ))
        );
    }

    #[test]
    fn a_reason_about_no_design_value_is_left_as_it_is() {
        let (found, tool) = two_engines("1.0 [mm^3]", "2.0 [mm^3]");

        let loaded = load(
            claimed(),
            found,
            tool,
            claim_answers(undecided("feature value pump.mass: no value")),
        );

        assert_eq!(
            loaded.claim("D::@1"),
            undecided("feature value pump.mass: no value")
        );
    }

    #[test]
    fn a_check_that_fails_is_put_in_words_too() {
        let tool = Tool::answering(vec![("Engine", "Volume", Err("no object Engine"))]);
        let tables = Tables {
            designed: Some(Ok(Answers {
                claims: Some(Err("verify failed at \"no Design Value #0#\"".to_owned())),
                ..Answers::default()
            })),
            ..Tables::default()
        };

        let loaded = load(
            claimed(),
            engine_volume(property("D::engine::volume", "D::engine", "Volume")),
            tool,
            tables,
        );

        assert_eq!(
            loaded.messages(),
            [
                "requirements could not be checked, so their status is Undecided: \
                 no Design Value for D::engine::volume (FreeCAD: no object Engine)"
            ]
        );
    }
}

mod judged {
    use super::*;

    #[test]
    fn with_the_values_written_the_checks_ask_the_model_that_has_them() {
        let tool = Tool::answering(vec![("Engine", "Volume", Ok("1.0 [mm^3]"))]);

        let loaded = load(
            claimed(),
            engine_volume(property("D::engine::volume", "D::engine", "Volume")),
            tool,
            claim_answers(violated("volume <= 0.5 [mm^3]")),
        );

        assert_eq!(loaded.claim("D::@1"), violated("volume <= 0.5 [mm^3]"));
        assert!(loaded.parsed_asked.is_empty(), "{:?}", loaded.parsed_asked);
        assert_eq!(loaded.designed_asked, ["claims"]);
    }

    #[test]
    fn when_the_values_cannot_be_written_nothing_is_judged() {
        let elements = claimed()
            .part("D::drive")
            .add("D::drive::mass", ElementKind::AttributeUsage, |_| {})
            .asserted("D::drive::light")
            .analysis("D::sizing");
        let found = links(
            vec![link("D::engine", "FreeCAD", "cad/engine.FCStd", "Engine")],
            vec![
                property("D::engine::volume", "D::engine", "Volume"),
                property("D::engine::area", "D::engine", "Area"),
            ],
        );
        let tool = Tool::answering(vec![
            ("Engine", "Volume", Ok("1.0 [mm^3]")),
            ("Engine", "Area", Err("no property Area")),
        ]);
        let tables = Tables {
            parsed: Answers {
                claims: Some(Ok(vec![ClaimOutcome {
                    requirement: qn("D::r"),
                    satisfier: None,
                    result: ClaimResult::Holds,
                }])),
                parts: [(
                    qn("D::drive"),
                    Ok(PartCheck {
                        constraints: vec![ConstraintOutcome {
                            constraint: qn("D::drive::light"),
                            path: String::new(),
                            result: violated("mass <= 10 [kg]"),
                        }],
                        values: vec![crate::checks::ComputedValue {
                            feature: qn("D::drive::mass"),
                            text: "12.0 [kg]".to_owned(),
                        }],
                    }),
                )]
                .into(),
                analyses: [(
                    qn("D::sizing"),
                    Ok(AnalysisResult {
                        outputs: vec![("result".to_owned(), "12.0".to_owned())],
                        objective: Some(ClaimResult::Holds),
                    }),
                )]
                .into(),
                ..Answers::default()
            },
            designed: Some(Err("cannot read drive.sysml".to_owned())),
            ..Tables::default()
        };

        let loaded = load(elements, found, tool, tables);

        let nothing = undecided(
            "Design Values could not be written into the Model: cannot read drive.sysml, so nothing was judged",
        );
        assert_eq!(
            loaded.messages(),
            ["Design Values could not be written into the Model: cannot read drive.sysml"]
        );
        assert_eq!(loaded.claim("D::@1"), nothing);
        let light = &loaded.elements[&qn("D::drive::light")].checks;
        assert_eq!(light.len(), 1);
        assert_eq!(light[0].result, nothing);
        let sizing = loaded.elements[&qn("D::sizing")]
            .analysis_result
            .clone()
            .unwrap();
        assert_eq!(sizing.objective, Some(nothing.clone()));
        assert!(sizing.outputs.is_empty(), "{:?}", sizing.outputs);
        assert!(loaded.elements[&qn("D::drive::mass")].computed.is_empty());
        assert_eq!(
            loaded.design_values(),
            [
                (
                    "D::engine::volume",
                    &DesignValue::Missing {
                        reason: "could not be written into the Model: cannot read drive.sysml"
                            .to_owned()
                    }
                ),
                (
                    "D::engine::area",
                    &DesignValue::Missing {
                        reason: "no property Area".to_owned()
                    }
                ),
            ],
            "a value read was not used; one missing keeps its own reason"
        );
    }
}
