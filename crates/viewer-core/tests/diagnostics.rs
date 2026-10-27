//! A diagnostic as one line, as the CLI prints it and the GUI shows it.

use std::path::{Path, PathBuf};

use viewer_core::model::{Diagnostic, Severity, SourceLocation};

fn at(file: &str, line: u32) -> Diagnostic {
    Diagnostic {
        severity: Severity::Error,
        message: "unexpected ';'".to_owned(),
        location: Some(SourceLocation {
            file: PathBuf::from(file),
            line,
        }),
    }
}

#[test]
fn a_diagnostic_reads_file_line_severity_message() {
    assert_eq!(
        at("/m/drive.sysml", 12).line(None),
        "/m/drive.sysml:12: error: unexpected ';'"
    );
}

#[test]
fn line_0_is_the_whole_file_so_no_line_is_written() {
    assert_eq!(
        at("/m/drive.sysml", 0).line(None),
        "/m/drive.sysml: error: unexpected ';'"
    );
}

#[test]
fn the_file_is_relative_to_the_folder_when_it_is_inside_it() {
    let folder = Path::new("/m");
    assert_eq!(
        at("/m/parts/drive.sysml", 3).line(Some(folder)),
        "parts/drive.sysml:3: error: unexpected ';'"
    );
    assert_eq!(
        at("/elsewhere/drive.sysml", 3).line(Some(folder)),
        "/elsewhere/drive.sysml:3: error: unexpected ';'"
    );
}

#[test]
fn a_diagnostic_about_no_file_is_its_severity_and_message() {
    let warning = Diagnostic {
        severity: Severity::Warning,
        message: "analyses were not run".to_owned(),
        location: None,
    };
    assert_eq!(warning.line(None), "warning: analyses were not run");
}
