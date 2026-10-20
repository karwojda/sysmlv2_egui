//! Renders an SVG Export to PNG, to look at a Diagram without a browser:
//! `cargo run -p viewer-core --example svg2png -- in.svg out.png`.
//!
//! Sans-serif text is drawn in DejaVu Sans, a common Linux font close to
//! what browsers pick for the Exports.

use std::process::ExitCode;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [input, output] = args.as_slice() else {
        eprintln!("usage: svg2png <in.svg> <out.png>");
        return ExitCode::FAILURE;
    };
    match render(input, output) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

fn render(input: &str, output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut options = resvg::usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    options.fontdb_mut().set_sans_serif_family("DejaVu Sans");
    let tree = resvg::usvg::Tree::from_data(&std::fs::read(input)?, &options)?;
    let size = tree.size().to_int_size();
    let mut pixmap =
        resvg::tiny_skia::Pixmap::new(size.width(), size.height()).ok_or("the SVG has no area")?;
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );
    pixmap.save_png(output)?;
    Ok(())
}
