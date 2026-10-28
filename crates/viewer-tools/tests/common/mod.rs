//! What the stub tests share.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// A shell script at `path` running `body`, made executable.
///
/// A child process writes it, not this one: a test thread starting a
/// tool forks a child that holds every file this process has open until
/// it execs, and running a script some process still has open for
/// writing fails with "Text file busy" (ETXTBSY).
pub fn script(path: &Path, body: &str) -> PathBuf {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let mut writer = Command::new("sh")
        .args(["-c", "cat > \"$0\" && chmod 755 \"$0\""])
        .arg(path)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    writer
        .stdin
        .take()
        .unwrap()
        .write_all(format!("#!/bin/sh\n{body}\n").as_bytes())
        .unwrap();
    assert!(writer.wait().unwrap().success(), "could not write {path:?}");
    path.to_path_buf()
}
