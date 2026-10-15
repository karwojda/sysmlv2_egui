//! The watcher thread asks for a reload when Model files change on disk.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use tempfile::TempDir;
use viewer_app::watcher::Watcher;

/// Long enough for a change to be seen twice at the watcher's pace.
const WITHIN: Duration = Duration::from_secs(5);
/// Long enough that a reload would have been asked for by now.
const QUIET: Duration = Duration::from_millis(1800);

fn watching(folder: &TempDir) -> (Watcher, mpsc::Receiver<PathBuf>) {
    let (send, reloads) = mpsc::channel();
    let watcher = Watcher::new(move |folder| {
        let _ = send.send(folder);
    });
    watcher.watch(folder.path().to_path_buf());
    // Let the watcher take the folder's current files as loaded.
    std::thread::sleep(Duration::from_millis(1200));
    (watcher, reloads)
}

#[test]
fn editing_a_model_file_asks_for_one_reload_of_that_folder() {
    let folder = TempDir::new().unwrap();
    std::fs::write(folder.path().join("a.sysml"), "package A;").unwrap();
    let (_watcher, reloads) = watching(&folder);

    std::fs::write(folder.path().join("a.sysml"), "package A { part def B; }").unwrap();

    assert_eq!(reloads.recv_timeout(WITHIN).unwrap(), folder.path());
    assert!(reloads.recv_timeout(QUIET).is_err(), "only one reload");
}

#[test]
fn adding_and_deleting_model_files_asks_for_reloads() {
    let folder = TempDir::new().unwrap();
    std::fs::write(folder.path().join("a.sysml"), "package A;").unwrap();
    let (_watcher, reloads) = watching(&folder);

    std::fs::write(folder.path().join("b.sysml"), "package B;").unwrap();
    assert!(reloads.recv_timeout(WITHIN).is_ok(), "a new file");

    std::fs::remove_file(folder.path().join("b.sysml")).unwrap();
    assert!(reloads.recv_timeout(WITHIN).is_ok(), "a deleted file");
}

#[test]
fn saving_a_diagram_file_asks_for_no_reload() {
    let folder = TempDir::new().unwrap();
    std::fs::write(folder.path().join("a.sysml"), "package A;").unwrap();
    let (_watcher, reloads) = watching(&folder);

    std::fs::create_dir_all(folder.path().join("diagrams")).unwrap();
    std::fs::write(folder.path().join("diagrams/overview.diagram.json"), "{}").unwrap();

    assert!(reloads.recv_timeout(QUIET).is_err());
}

#[test]
fn dropping_the_watcher_stops_it() {
    let folder = TempDir::new().unwrap();
    std::fs::write(folder.path().join("a.sysml"), "package A;").unwrap();
    let (watcher, reloads) = watching(&folder);

    drop(watcher);
    std::fs::write(folder.path().join("a.sysml"), "package A { part def B; }").unwrap();

    assert!(reloads.recv_timeout(QUIET).is_err());
}
