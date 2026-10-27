//! FreeCAD as an Engineering Tool: `freecadcmd` runs an embedded script
//! once per `.FCStd` file and reports each object's `Volume` in mm³. An
//! object is named by its `Name` (`Body`) or by a dotted assembly path
//! from a top-level object (`Assembly.Link.`).

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::{Value, json};
use viewer_core::engineering_tools::{Answer, EngineeringTool, ReadValue, Wanted};

use crate::process;

/// The name Tool Links give FreeCAD: `@ToolLink { tool = "FreeCAD"; … }`.
pub const NAME: &str = "FreeCAD";

/// The environment variable naming the `freecadcmd` to run, ahead of
/// `PATH` and the usual install locations:
/// `SYSML_VIEWER_FREECAD=/opt/freecad/bin/freecadcmd`.
pub const EXECUTABLE_VARIABLE: &str = "SYSML_VIEWER_FREECAD";

/// The environment variable giving how many seconds a FreeCAD run may
/// take before it is killed: `SYSML_VIEWER_FREECAD_TIMEOUT=120`.
pub const TIMEOUT_VARIABLE: &str = "SYSML_VIEWER_FREECAD_TIMEOUT";

/// How long a FreeCAD run may take when `SYSML_VIEWER_FREECAD_TIMEOUT`
/// isn't set.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// The variable the script reads its request from. An environment
/// variable rather than an argument, so a Windows `freecadcmd` called
/// from WSL gets it too (through `WSLENV`).
const REQUEST_VARIABLE: &str = "SYSML_VIEWER_FREECAD_REQUEST";

/// The FreeCAD side of a read: opens the document, resolves each object,
/// prints one `FCRESULT {…}` line.
const SCRIPT: &str = include_str!("freecad/query.py");

/// How `freecadcmd` is looked for: by either of its names, `freecadcmd`
/// in current builds and `FreeCADCmd` in older ones, and in an install
/// folder under `FreeCAD*/bin` or a macOS bundle's `Contents/Resources/bin`.
const FREECADCMD: process::Executable = process::Executable {
    tool: NAME,
    variable: EXECUTABLE_VARIABLE,
    names: &["freecadcmd", "FreeCADCmd"],
    installs_in: |dir| {
        process::entries(dir)
            .into_iter()
            .filter(|install| {
                let name = install.file_name().unwrap_or_default().to_string_lossy();
                name.to_lowercase().starts_with("freecad")
            })
            .collect()
    },
    bins: &["bin", "Contents/Resources/bin"],
};

/// Where FreeCAD may be found on a machine, and how long it may take:
/// what `FreeCad::from_machine` reads. `Machine::current` reads this
/// machine; tests give their own.
#[derive(Clone, Debug, Default)]
pub struct Machine {
    /// `SYSML_VIEWER_FREECAD`, the executable to run.
    pub executable: Option<OsString>,
    /// `SYSML_VIEWER_FREECAD_TIMEOUT`, in seconds (`30`, `2.5`).
    pub timeout: Option<OsString>,
    /// `PATH`, searched for `freecadcmd` and `FreeCADCmd`.
    pub path: Option<OsString>,
    /// Folders FreeCAD installs into, each searched for a `FreeCAD*`
    /// folder holding `bin/freecadcmd`: `/opt`, and for a Windows install
    /// seen from WSL `/mnt/c/Program Files`.
    pub install_dirs: Vec<PathBuf>,
}

impl Machine {
    /// This machine: its environment variables and the usual install
    /// locations on Linux, WSL, macOS and Windows.
    pub fn current() -> Self {
        let mut install_dirs: Vec<PathBuf> = [
            "/mnt/c/Program Files",
            "/opt",
            "/usr/lib",
            "/usr/local/lib",
            "/Applications",
        ]
        .map(PathBuf::from)
        .into();
        install_dirs.extend(std::env::var_os("ProgramFiles").map(PathBuf::from));
        Self {
            executable: std::env::var_os(EXECUTABLE_VARIABLE),
            timeout: std::env::var_os(TIMEOUT_VARIABLE),
            path: std::env::var_os("PATH"),
            install_dirs,
        }
    }

    /// The `freecadcmd` to run, or why there is none. A set
    /// `SYSML_VIEWER_FREECAD` is final: when it names nothing, `PATH` is
    /// not searched, so a mistyped setting isn't hidden by another
    /// install.
    fn find(&self) -> Result<PathBuf, String> {
        FREECADCMD.find(
            self.executable.as_ref(),
            self.path.as_ref(),
            &self.install_dirs,
        )
    }

    fn timeout(&self) -> Result<Duration, String> {
        process::timeout(self.timeout.as_ref(), TIMEOUT_VARIABLE, DEFAULT_TIMEOUT)
    }
}

/// The FreeCAD Engineering Tool, which a backend is given under [`NAME`]
/// (see [`crate::built_in`]).
pub struct FreeCad {
    executable: Result<PathBuf, String>,
    timeout: Result<Duration, String>,
}

impl FreeCad {
    /// FreeCAD as this machine has it (`Machine::current`). Finding no
    /// FreeCAD isn't an error here: every read then gives the reason.
    pub fn new() -> Self {
        Self::from_machine(Machine::current())
    }

    /// FreeCAD as `machine` has it: `SYSML_VIEWER_FREECAD` first, then
    /// `PATH`, then the install folders.
    pub fn from_machine(machine: Machine) -> Self {
        Self {
            executable: machine.find(),
            timeout: machine.timeout(),
        }
    }

    /// The `freecadcmd` this adapter runs, or why there is none
    /// (`FreeCAD not available: …`).
    pub fn executable(&self) -> Result<&Path, &str> {
        self.executable.as_deref().map_err(String::as_str)
    }

    /// How long a run may take before it is killed, or why the setting
    /// can't be used.
    pub fn timeout(&self) -> Result<Duration, &str> {
        self.timeout.as_ref().copied().map_err(String::as_str)
    }

    /// Runs the script on `file` and returns what it printed after
    /// `FCRESULT`.
    fn run(&self, executable: &Path, file: &Path, wanted: &Wanted) -> Result<Value, String> {
        let timeout = self.timeout()?;
        let dir =
            tempfile::tempdir().map_err(|e| format!("no temporary folder for FreeCAD: {e}"))?;
        let script = dir.path().join("query.py");
        std::fs::write(&script, SCRIPT)
            .map_err(|e| format!("could not write {}: {e}", script.display()))?;

        let path_for_freecad = |path: &Path| process::path_for(executable, path, NAME);
        let request = json!({"file": path_for_freecad(file)?, "objects": wanted}).to_string();

        let mut command = Command::new(executable);
        command
            .arg(path_for_freecad(&script)?)
            .current_dir(dir.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        process::set_variable(&mut command, executable, REQUEST_VARIABLE, &request);
        let child = command.spawn().map_err(|e| {
            format!(
                "FreeCAD not available: {} could not be started: {e}",
                executable.display()
            )
        })?;
        let process::Output { stdout, stderr } = process::finish(child, timeout, NAME)?;

        let Some(line) = stdout
            .lines()
            .find_map(|line| line.strip_prefix("FCRESULT "))
        else {
            let said = stderr.lines().rev().find(|l| !l.trim().is_empty());
            return Err(match said {
                Some(said) => format!("FreeCAD gave no answer: {}", said.trim()),
                None => "FreeCAD gave no answer".to_owned(),
            });
        };
        serde_json::from_str(line).map_err(|e| format!("FreeCAD's answer can't be read: {e}"))
    }
}

impl Default for FreeCad {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineeringTool for FreeCad {
    fn read(&mut self, file: &Path, wanted: &Wanted) -> Result<Answer, String> {
        let executable = self.executable.clone()?;
        if !file.is_file() {
            return Err(format!("file {} not found", file.display()));
        }
        let result = self.run(&executable, file, wanted)?;
        if let Some(reason) = result["__file__"]["error"].as_str() {
            return Err(format!("{}: {reason}", file.display()));
        }
        Ok(wanted
            .iter()
            .map(|(object, properties)| {
                let found = &result[object];
                let answers = properties
                    .iter()
                    .map(|property| {
                        let value = match found["error"].as_str() {
                            Some(reason) => Err(format!("{reason} in {}", file.display())),
                            None => value_text(&found[property]).map(ReadValue::new),
                        };
                        (property.clone(), value)
                    })
                    .collect();
                (object.clone(), answers)
            })
            .collect())
    }
}

/// `[1000000.0, "mm^3"]` as `1000000.0 [mm^3]`, or the reason the script
/// gave instead.
fn value_text(answer: &Value) -> Result<String, String> {
    if let Some(reason) = answer["error"].as_str() {
        return Err(reason.to_owned());
    }
    match (answer[0].as_f64(), answer[1].as_str()) {
        (Some(number), Some(unit)) => Ok(format!("{number:?} [{unit}]")),
        _ => Err("FreeCAD gave no answer for it".to_owned()),
    }
}
