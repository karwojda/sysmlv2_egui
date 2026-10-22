//! The KiCad adapter through the `EngineeringTool` trait, against the real
//! `kicad-cli` and the RoyalBlue54L Feather in `examples/feather-budget`:
//! a 58.42 × 22.86 mm board, 1.6 mm thick. When KiCad can't be found
//! these tests print a note and pass.

use std::path::{Path, PathBuf};

use viewer_core::engineering_tools::{EngineeringTool, Wanted};
use viewer_tools::kicad::{KiCad, Machine};

fn feather() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/feather-budget/ecad/RoyalBlue54L-Feather.kicad_pro")
}

/// KiCad as this machine has it, or `None` after a note saying the test
/// is skipped.
fn kicad(test: &str) -> Option<KiCad> {
    let kicad = KiCad::from_machine(Machine::current());
    match kicad.executable() {
        Ok(_) => Some(kicad),
        Err(reason) => {
            eprintln!("note: skipping {test}: {reason}");
            None
        }
    }
}

#[test]
fn the_feather_board_gives_its_dimensions() {
    let Some(mut kicad) = kicad("the_feather_board_gives_its_dimensions") else {
        return;
    };
    let wanted = Wanted::from([(
        "board".into(),
        ["Width", "Height", "Thickness", "Area"]
            .map(String::from)
            .into(),
    )]);

    let read = kicad.read(&feather(), &wanted).unwrap();

    let board = &read["board"];
    assert_eq!(board["Width"], Ok("58.42 [mm]".into()));
    assert_eq!(board["Height"], Ok("22.86 [mm]".into()));
    assert_eq!(board["Thickness"], Ok("1.6 [mm]".into()));
    assert_eq!(board["Area"], Ok("1329.932 [mm^2]".into()));
}
