//! The KiCad adapter through the `EngineeringTool` trait, against the real
//! `kicad-cli` and the RoyalBlue54L Feather in `examples/feather-budget`:
//! a 58.42 × 22.86 mm board, 1.6 mm thick, with 48 placed parts. When
//! KiCad can't be found these tests print a note and pass.

use std::path::{Path, PathBuf};

use viewer_core::engineering_tools::{EngineeringTool, Parts, ReadValue, Wanted};
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
    let read = |property: &str| board[property].clone();
    assert_eq!(read("Width"), Ok(ReadValue::new("58.42 [mm]")));
    assert_eq!(read("Height"), Ok(ReadValue::new("22.86 [mm]")));
    assert_eq!(read("Thickness"), Ok(ReadValue::new("1.6 [mm]")));
    assert_eq!(read("Area"), Ok(ReadValue::new("1329.932 [mm^2]")));
}

#[test]
fn the_feather_gives_its_part_count_and_summed_fields() {
    let Some(mut kicad) = kicad("the_feather_gives_its_part_count_and_summed_fields") else {
        return;
    };
    let wanted = Wanted::from([(
        "board".into(),
        ["ComponentCount", "Power", "Price"]
            .map(String::from)
            .into(),
    )]);

    let read = kicad.read(&feather(), &wanted).unwrap();

    // The seven active parts the example gives illustrative figures to;
    // the other 41 placed parts state none.
    let counted = ["D1", "D3", "U1", "U2", "U5", "Y1", "Y2"];
    let uncounted = [
        "C1", "C2", "C3", "C4", "C5", "C6", "C7", "C8", "C9", "C10", "C11", "C12", "C13", "C14",
        "C15", "C16", "C17", "C18", "C19", "C20", "C21", "C22", "C23", "C24", "C25", "J3", "J4",
        "J5", "J6", "L1", "R1", "R4", "R5", "R6", "R7", "R8", "SW1", "SW2", "U3", "U4", "U6",
    ];
    let board = &read["board"];
    assert_eq!(board["ComponentCount"], Ok(ReadValue::new("48")));
    assert_eq!(
        board["Power"],
        Ok(ReadValue::summed(
            "0.060953 [W]",
            Parts::new(counted, uncounted)
        ))
    );
    assert_eq!(
        board["Price"],
        Ok(ReadValue::summed("12.8", Parts::new(counted, uncounted)))
    );
}
