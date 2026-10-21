//! FreeCAD as an Engineering Tool: `freecadcmd` runs an embedded script
//! once per `.FCStd` file and reports each object's `Volume` in mm³. An
//! object is named by its `Name` (`Body`) or by a dotted assembly path
//! from a top-level object (`Assembly.Link.`).

use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use viewer_core::engineering_tools::{Answer, EngineeringTool, Wanted};

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

/// The names `freecadcmd` goes by: `freecadcmd` in current builds,
/// `FreeCADCmd` in older ones.
const NAMES: [&str; 2] = ["freecadcmd", "FreeCADCmd"];

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
        if let Some(executable) = &self.executable {
            let executable = PathBuf::from(executable);
            return if executable.is_file() {
                Ok(executable)
            } else {
                Err(format!(
                    "FreeCAD not available: {EXECUTABLE_VARIABLE} names {}, which does not exist",
                    executable.display()
                ))
            };
        }
        self.on_path().or_else(|| self.installed()).ok_or_else(|| {
            format!(
                "FreeCAD not available: no freecadcmd in {EXECUTABLE_VARIABLE}, on PATH \
                     or in the usual install locations"
            )
        })
    }

    fn on_path(&self) -> Option<PathBuf> {
        let path = self.path.as_ref()?;
        std::env::split_paths(path).find_map(|dir| executable_in(&dir))
    }

    /// The newest `FreeCAD*` install in the install folders: `FreeCAD 1.1`
    /// before `FreeCAD 1.0`.
    fn installed(&self) -> Option<PathBuf> {
        self.install_dirs.iter().find_map(|dir| {
            let mut installs: Vec<PathBuf> = std::fs::read_dir(dir)
                .ok()?
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    let name = entry.file_name().to_string_lossy().to_lowercase();
                    name.starts_with("freecad")
                })
                .map(|entry| entry.path())
                .collect();
            installs.sort();
            installs.iter().rev().find_map(|install| {
                ["bin", "Contents/Resources/bin"]
                    .iter()
                    .find_map(|bin| executable_in(&install.join(bin)))
            })
        })
    }

    fn timeout(&self) -> Result<Duration, String> {
        let Some(timeout) = &self.timeout else {
            return Ok(DEFAULT_TIMEOUT);
        };
        let text = timeout.to_string_lossy();
        text.trim()
            .parse::<f64>()
            .ok()
            .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
            .filter(|duration| !duration.is_zero())
            .ok_or_else(|| format!("{TIMEOUT_VARIABLE} is {text}, not a number of seconds"))
    }
}

/// `freecadcmd` (or `FreeCADCmd`, either with `.exe`) in `dir`.
fn executable_in(dir: &Path) -> Option<PathBuf> {
    NAMES
        .iter()
        .flat_map(|name| [PathBuf::from(name), PathBuf::from(format!("{name}.exe"))])
        .map(|name| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// The FreeCAD Engineering Tool: register it with a backend as `"FreeCAD"`
/// (`OpenSysmlBackend::new().with_tool("FreeCAD", FreeCad::new())`).
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

        let windows = is_windows_executable(executable);
        let path_for_freecad = |path: &Path| -> Result<String, String> {
            if windows {
                windows_path(path)
            } else {
                Ok(path.to_string_lossy().into_owned())
            }
        };
        let request = json!({"file": path_for_freecad(file)?, "objects": wanted}).to_string();

        let mut command = Command::new(executable);
        command
            .arg(path_for_freecad(&script)?)
            .env(REQUEST_VARIABLE, request)
            .current_dir(dir.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        if windows {
            let shared = match std::env::var("WSLENV") {
                Ok(existing) if !existing.is_empty() => format!("{existing}:{REQUEST_VARIABLE}"),
                _ => REQUEST_VARIABLE.to_owned(),
            };
            command.env("WSLENV", shared);
        }
        let child = command.spawn().map_err(|e| {
            format!(
                "FreeCAD not available: {} could not be started: {e}",
                executable.display()
            )
        })?;
        let (stdout, stderr) = finish(child, timeout)?;

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
                            None => value_text(&found[property]),
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

/// Waits for `child` up to `timeout`, then kills it. Gives its stdout and
/// stderr.
fn finish(mut child: Child, timeout: Duration) -> Result<(String, String), String> {
    let stdout = drain(child.stdout.take());
    let stderr = drain(child.stderr.take());
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                // The readers are left behind: a process the run started
                // may still hold the pipes open.
                return Err(format!(
                    "FreeCAD did not answer within {} s",
                    timeout.as_secs_f64()
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(format!("FreeCAD could not be waited for: {e}")),
        }
    }
    let text = |reader: JoinHandle<String>| reader.join().unwrap_or_default();
    Ok((text(stdout), text(stderr)))
}

/// Reads a pipe to its end on its own thread, so a chatty run never
/// blocks on a full pipe.
fn drain(pipe: Option<impl Read + Send + 'static>) -> JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        if let Some(mut pipe) = pipe {
            let _ = pipe.read_to_end(&mut bytes);
        }
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

/// A Windows `freecadcmd.exe` run from Linux, which is WSL: it needs
/// Windows paths and its request passed through `WSLENV`.
fn is_windows_executable(executable: &Path) -> bool {
    cfg!(unix)
        && executable
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
}

/// `path` as Windows sees it from WSL: `/tmp/x` as
/// `\\wsl.localhost\Ubuntu\tmp\x`, `/mnt/c/x` as `C:\x`.
fn windows_path(path: &Path) -> Result<String, String> {
    let output = Command::new("wslpath")
        .arg("-w")
        .arg(path)
        .output()
        .map_err(|e| format!("wslpath could not be run for a Windows FreeCAD: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "wslpath could not convert {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
