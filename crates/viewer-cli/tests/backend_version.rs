//! Every command that loads a Model says, once on stderr, which backend
//! loaded it, so a disagreement with another tool can be traced to a
//! version. Driven with the fake backend.

#[path = "../../viewer-core/tests/common/mod.rs"]
mod common;

use std::ffi::OsString;

use tempfile::TempDir;
use viewer_cli::run_with_output;
use viewer_core::FakeBackend;

const VERSION: &str = "OpenSysML sysml-grpc v0.11.0";

struct Outcome {
    code: u8,
    stdout: String,
    stderr: String,
}

fn run(backend: &mut FakeBackend, command: &[&str]) -> Outcome {
    let folder = TempDir::new().unwrap();
    let out = TempDir::new().unwrap();
    let mut args: Vec<OsString> = vec!["sysml-viewer".into(), command[0].into()];
    args.push(folder.path().into());
    for arg in &command[1..] {
        args.push(match *arg {
            "OUT" => out.path().into(),
            other => other.into(),
        });
    }
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());
    let code = run_with_output(args, backend, &mut stdout, &mut stderr);
    // Each run has its own folder; the reports name it.
    let folder = folder.path().to_string_lossy().into_owned();
    Outcome {
        code,
        stdout: String::from_utf8(stdout).unwrap().replace(&folder, "MODEL"),
        stderr: String::from_utf8(stderr).unwrap(),
    }
}

#[test]
fn check_says_once_on_stderr_which_backend_loaded_the_model() {
    let mut unnamed = FakeBackend::new(common::requirements());
    let mut named = FakeBackend::new(common::requirements()).loaded_by(VERSION);

    let without = run(&mut unnamed, &["check"]);
    let with = run(&mut named, &["check"]);

    assert_eq!(with.stderr.matches(VERSION).count(), 1, "{}", with.stderr);
    assert_eq!(with.stderr.lines().next(), Some(VERSION));
    assert_eq!(with.stdout, without.stdout);
    assert_eq!(with.code, without.code);
}

#[test]
fn requirements_and_export_say_it_too() {
    for command in [
        &["requirements"][..],
        &["export", "--all", "--out-dir", "OUT"][..],
    ] {
        let mut backend = FakeBackend::new(common::requirements()).loaded_by(VERSION);
        let outcome = run(&mut backend, command);
        assert_eq!(
            outcome.stderr.matches(VERSION).count(),
            1,
            "{command:?}: {}",
            outcome.stderr
        );
    }
}

#[test]
fn a_backend_with_nothing_to_name_adds_no_line() {
    let mut unnamed = FakeBackend::new(common::requirements());
    let mut named = FakeBackend::new(common::requirements()).loaded_by(VERSION);

    let without = run(&mut unnamed, &["check"]);
    let with = run(&mut named, &["check"]);

    assert_eq!(
        with.stderr.replacen(&format!("{VERSION}\n"), "", 1),
        without.stderr
    );
}
