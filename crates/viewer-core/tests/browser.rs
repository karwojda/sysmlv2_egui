//! The Browser Diagram's content, from a Model served by the fake backend.

use std::path::Path;

use viewer_core::browser::{BrowserEntry, tree};
use viewer_core::{FakeBackend, Model, ModelBackend};

fn model() -> Model {
    let model = Model::builder()
        .package("Vehicles")
        .part_def("Vehicles::Car")
        .attribute("Vehicles::Car::mass")
        .part_def("Vehicles::Truck")
        .unsupported("Vehicles::TopSpeed", "use case def")
        .package("Vehicles::Parts")
        .part_def("Vehicles::Parts::Wheel")
        .package("Fleet")
        .part("Fleet::taxi")
        .build();
    FakeBackend::new(model).load(Path::new("model")).unwrap()
}

/// Each entry as `indent «keyword» name`, depth first.
fn outline(entries: &[BrowserEntry]) -> Vec<String> {
    fn walk(entries: &[BrowserEntry], depth: usize, out: &mut Vec<String>) {
        for entry in entries {
            out.push(format!(
                "{}«{}» {}",
                "  ".repeat(depth),
                entry.keyword,
                entry.name
            ));
            walk(&entry.children, depth + 1, out);
        }
    }
    let mut out = Vec::new();
    walk(entries, 0, &mut out);
    out
}

#[test]
fn shows_the_ownership_tree_with_names_and_kinds() {
    assert_eq!(
        outline(&tree(&model(), "")),
        [
            "«package» Fleet",
            "  «part» taxi",
            "«package» Vehicles",
            "  «part def» Car",
            "    «attribute» mass",
            "  «package» Parts",
            "    «part def» Wheel",
            "  «use case def» TopSpeed",
            "  «part def» Truck",
        ]
    );
}

#[test]
fn marks_unsupported_elements() {
    let entries = tree(&model(), "TopSpeed");
    let top_speed = &entries[0].children[0];

    assert_eq!(top_speed.name, "TopSpeed");
    assert!(top_speed.unsupported);
    assert!(!entries[0].unsupported);
}

#[test]
fn a_filter_keeps_matching_entries_and_their_ancestors() {
    assert_eq!(
        outline(&tree(&model(), "wheel")),
        [
            "«package» Vehicles",
            "  «package» Parts",
            "    «part def» Wheel"
        ]
    );
}

#[test]
fn a_filter_matches_any_part_of_a_name_ignoring_case() {
    assert_eq!(
        outline(&tree(&model(), "  AR ")),
        [
            "«package» Vehicles",
            "  «part def» Car",
            "  «package» Parts"
        ]
    );
}

#[test]
fn a_filter_with_no_match_gives_an_empty_tree() {
    assert!(tree(&model(), "nothing like this").is_empty());
}
