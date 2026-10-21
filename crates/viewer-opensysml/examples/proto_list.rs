// PROTOTYPE: list element qualified names of a Model folder.
use viewer_core::backend::ModelBackend;
fn main() {
    let folder = std::env::args().nth(1).unwrap();
    let mut b = viewer_opensysml::OpenSysmlBackend::new();
    let m = b.load(std::path::Path::new(&folder)).unwrap();
    for e in m.elements() {
        if e.qualified_name.to_string().contains("drive") {
            println!(
                "{} {:?} value={:?} computed={:?}",
                e.qualified_name, e.kind, e.value, e.computed
            );
        }
    }
}
