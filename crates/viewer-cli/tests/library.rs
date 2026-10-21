//! The `library` command: prints the `EngineeringTools` library that users
//! copy into their Model, so `@ToolLink` and `@ToolProperty` resolve.

use std::ffi::OsString;

use viewer_cli::{EXIT_OK, run_with_output};
use viewer_core::{FakeBackend, Model};

#[test]
fn prints_the_engineering_tools_library_and_succeeds() {
    let args: Vec<OsString> = vec!["sysml-viewer".into(), "library".into()];
    let (mut stdout, mut stderr) = (Vec::new(), Vec::new());

    let code = run_with_output(
        args,
        &mut FakeBackend::new(Model::default()),
        &mut stdout,
        &mut stderr,
    );

    let stdout = String::from_utf8(stdout).unwrap();
    assert_eq!(code, EXIT_OK);
    assert_eq!(String::from_utf8(stderr).unwrap(), "");
    assert!(stdout.starts_with("package EngineeringTools {"), "{stdout}");
    for line in [
        "metadata def ToolLink {",
        "doc /* The annotated usage is designed in `object` inside `file`, owned by Engineering Tool `tool`. */",
        "attribute tool : String;",
        "attribute file : String;",
        "attribute object : String;",
        "metadata def ToolProperty {",
        "doc /* The annotated feature takes its Design Value from property `name` of its owner's Tool Link. */",
        "attribute name : String;",
        "attribute tool : String[0..1];",
    ] {
        assert!(
            stdout.lines().any(|l| l.trim() == line),
            "missing {line:?} in\n{stdout}"
        );
    }
}
