//! KiCad as an Engineering Tool: `kicad-cli` exports what a project's
//! links ask for. A link names the project's `.kicad_pro`; the adapter
//! reads the `.kicad_sch` and `.kicad_pcb` of the same name beside it.
//! The one object is `board`, whose `Width`, `Height`, `Thickness` and
//! `Area` come from `pcb export stats`.
//!
//! Every run gets its own temporary `KICAD_CONFIG_HOME` whose
//! `kicad_common.json` sets the language to English: a KiCad set to
//! another language writes its statistics with decimal commas
//! (`58,4200 mm`), and locale variables don't change that. The user's own
//! KiCad settings are never read or written.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;
use viewer_core::engineering_tools::{Answer, EngineeringTool, Wanted};

use crate::process;

/// The name Tool Links give KiCad: `@ToolLink { tool = "KiCad"; … }`.
pub const NAME: &str = "KiCad";

/// The environment variable naming the `kicad-cli` to run, ahead of
/// `PATH` and the usual install locations:
/// `SYSML_VIEWER_KICAD=/usr/bin/kicad-cli`.
pub const EXECUTABLE_VARIABLE: &str = "SYSML_VIEWER_KICAD";

/// The environment variable giving how many seconds a `kicad-cli` run may
/// take before it is killed: `SYSML_VIEWER_KICAD_TIMEOUT=60`.
pub const TIMEOUT_VARIABLE: &str = "SYSML_VIEWER_KICAD_TIMEOUT";

/// How long a `kicad-cli` run may take when `SYSML_VIEWER_KICAD_TIMEOUT`
/// isn't set.
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// The one object a KiCad link can name for now: `object = "board"`.
pub const BOARD: &str = "board";

/// The oldest KiCad whose `kicad-cli` has `pcb export stats`.
const OLDEST_VERSION: u64 = 10;

/// The board's properties, each with the key of `pcb export stats` it is
/// read from (`board.width`).
const DIMENSIONS: [(&str, &str); 4] = [
    ("Width", "width"),
    ("Height", "height"),
    ("Thickness", "board_thickness"),
    ("Area", "area"),
];

/// Where KiCad may be found on a machine, and how long it may take: what
/// `KiCad::from_machine` reads. `Machine::current` reads this machine;
/// tests give their own.
#[derive(Clone, Debug, Default)]
pub struct Machine {
    /// `SYSML_VIEWER_KICAD`, the executable to run.
    pub executable: Option<OsString>,
    /// `SYSML_VIEWER_KICAD_TIMEOUT`, in seconds (`30`, `2.5`).
    pub timeout: Option<OsString>,
    /// `PATH`, searched for `kicad-cli`.
    pub path: Option<OsString>,
    /// Folders KiCad installs into, each searched for a `KiCad` folder
    /// holding one folder per version with `bin/kicad-cli` in it (newest
    /// first), or `KiCad.app`: `/Applications`, and for a Windows install
    /// seen from WSL `/mnt/c/Program Files`
    /// (`/mnt/c/Program Files/KiCad/10.0/bin/kicad-cli.exe`).
    pub install_dirs: Vec<PathBuf>,
}

impl Machine {
    /// This machine: its environment variables and the usual install
    /// locations on WSL, macOS and Windows. On Linux, KiCad's packages put
    /// `kicad-cli` on `PATH`.
    pub fn current() -> Self {
        let mut install_dirs: Vec<PathBuf> = ["/mnt/c/Program Files", "/Applications"]
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

    /// The `kicad-cli` to run, or why there is none. A set
    /// `SYSML_VIEWER_KICAD` is final: when it names nothing, `PATH` is not
    /// searched, so a mistyped setting isn't hidden by another install.
    fn find(&self) -> Result<PathBuf, String> {
        if let Some(executable) = &self.executable {
            let executable = PathBuf::from(executable);
            return if executable.is_file() {
                Ok(executable)
            } else {
                Err(format!(
                    "KiCad not available: {EXECUTABLE_VARIABLE} names {}, which does not exist",
                    executable.display()
                ))
            };
        }
        self.on_path().or_else(|| self.installed()).ok_or_else(|| {
            format!(
                "KiCad not available: no kicad-cli in {EXECUTABLE_VARIABLE}, on PATH \
                     or in the usual install locations"
            )
        })
    }

    fn on_path(&self) -> Option<PathBuf> {
        let path = self.path.as_ref()?;
        std::env::split_paths(path).find_map(|dir| executable_in(&dir))
    }

    /// The newest install in the install folders, by the version numbers
    /// in its folder's name: `KiCad/10.0` before `KiCad/9.0`.
    fn installed(&self) -> Option<PathBuf> {
        self.install_dirs.iter().find_map(|dir| {
            let mut installs: Vec<PathBuf> = std::fs::read_dir(dir.join("KiCad"))
                .ok()?
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect();
            installs.sort_by_cached_key(|install| {
                process::version_numbers(&install.file_name().unwrap_or_default().to_string_lossy())
            });
            installs.iter().rev().find_map(|install| {
                ["bin", "Contents/MacOS"]
                    .iter()
                    .find_map(|bin| executable_in(&install.join(bin)))
            })
        })
    }
}

/// `kicad-cli` (or `kicad-cli.exe`) in `dir`.
fn executable_in(dir: &Path) -> Option<PathBuf> {
    ["kicad-cli", "kicad-cli.exe"]
        .iter()
        .map(|name| dir.join(name))
        .find(|candidate| candidate.is_file())
}

/// The KiCad Engineering Tool, which a backend is given under [`NAME`]
/// (see [`crate::built_in`]). Nothing is run until a link is read.
pub struct KiCad {
    executable: Result<PathBuf, String>,
    timeout: Result<Duration, String>,
    /// Whether `kicad-cli` has said it is new enough, so later reads
    /// don't ask again.
    version_checked: bool,
}

impl KiCad {
    /// KiCad as this machine has it (`Machine::current`). Finding no
    /// KiCad isn't an error here: every read then gives the reason.
    pub fn new() -> Self {
        Self::from_machine(Machine::current())
    }

    /// KiCad as `machine` has it: `SYSML_VIEWER_KICAD` first, then
    /// `PATH`, then the install folders.
    pub fn from_machine(machine: Machine) -> Self {
        Self {
            executable: machine.find(),
            timeout: process::timeout(machine.timeout.as_ref(), TIMEOUT_VARIABLE, DEFAULT_TIMEOUT),
            version_checked: false,
        }
    }

    /// The `kicad-cli` this adapter runs, or why there is none
    /// (`KiCad not available: …`).
    pub fn executable(&self) -> Result<&Path, &str> {
        self.executable.as_deref().map_err(String::as_str)
    }

    /// How long a run may take before it is killed, or why the setting
    /// can't be used.
    pub fn timeout(&self) -> Result<Duration, &str> {
        self.timeout.as_ref().copied().map_err(String::as_str)
    }

    /// Asks `kicad-cli` its version, once: anything older than KiCad 10
    /// is a reason (`KiCad 10 or newer needed; found 9.0.3`).
    fn check_version(&mut self, executable: &Path) -> Result<(), String> {
        if self.version_checked {
            return Ok(());
        }
        let workspace = Workspace::new()?;
        let output = self.run(executable, &workspace, &["version".into()])?;
        let found = output.stdout.trim();
        let major = process::version_numbers(found).first().copied();
        match major {
            Some(major) if major >= OLDEST_VERSION => {
                self.version_checked = true;
                Ok(())
            }
            Some(_) => Err(format!(
                "KiCad {OLDEST_VERSION} or newer needed; found {found}"
            )),
            None => Err(format!(
                "KiCad did not say its version: {}",
                output.last_said().unwrap_or("it printed nothing")
            )),
        }
    }

    /// The board statistics of `pcb` (`pcb export stats`), as KiCad
    /// writes them in English and millimetres.
    fn board_statistics(&self, executable: &Path, pcb: &Path) -> Result<Value, String> {
        let workspace = Workspace::new()?;
        let report = workspace.dir.path().join("stats.json");
        let args = [
            "pcb".into(),
            "export".into(),
            "stats".into(),
            "--format".into(),
            "json".into(),
            "--units".into(),
            "mm".into(),
            "-o".into(),
            process::path_for(executable, &report, NAME)?,
            process::path_for(executable, pcb, NAME)?,
        ];
        let output = self.run(executable, &workspace, &args)?;
        let text = std::fs::read_to_string(&report).map_err(|_| {
            format!(
                "KiCad gave no board statistics for {}: {}",
                pcb.display(),
                output.last_said().unwrap_or("it said nothing")
            )
        })?;
        serde_json::from_str(&text)
            .map_err(|e| format!("KiCad's board statistics can't be read: {e}"))
    }

    /// Runs `kicad-cli` with `args` in `workspace`, its config folder.
    fn run(
        &self,
        executable: &Path,
        workspace: &Workspace,
        args: &[String],
    ) -> Result<process::Output, String> {
        let timeout = self.timeout.clone()?;
        let config = process::path_for(executable, &workspace.config, NAME)?;
        let mut command = Command::new(executable);
        command
            .args(args)
            .current_dir(workspace.dir.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        process::set_variable(&mut command, executable, "KICAD_CONFIG_HOME", &config);
        let child = command.spawn().map_err(|e| {
            format!(
                "KiCad not available: {} could not be started: {e}",
                executable.display()
            )
        })?;
        process::finish(child, timeout, NAME)
    }
}

impl Default for KiCad {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineeringTool for KiCad {
    fn read(&mut self, file: &Path, wanted: &Wanted) -> Result<Answer, String> {
        let executable = self.executable.clone()?;
        let pcb = file.with_extension("kicad_pcb");
        for needed in [file, &file.with_extension("kicad_sch"), &pcb] {
            if !needed.is_file() {
                return Err(format!("file {} not found", needed.display()));
            }
        }
        self.check_version(&executable)?;
        let wants_dimensions = wanted.get(BOARD).is_some_and(|properties| {
            properties
                .iter()
                .any(|property| dimension_key(property).is_some())
        });
        let statistics = if wants_dimensions {
            Some(self.board_statistics(&executable, &pcb)?)
        } else {
            None
        };
        Ok(wanted
            .iter()
            .map(|(object, properties)| {
                let answers = properties
                    .iter()
                    .map(|property| {
                        let value = if object != BOARD {
                            Err(format!(
                                "only {BOARD} can be linked in KiCad for now, not '{object}'"
                            ))
                        } else {
                            match (dimension_key(property), &statistics) {
                                (Some(key), Some(statistics)) => dimension(statistics, key),
                                _ => Err(format!(
                                    "KiCad has no {BOARD} property {property} \
                                     (it reads Width, Height, Thickness and Area)"
                                )),
                            }
                        };
                        (property.clone(), value)
                    })
                    .collect();
                (object.clone(), answers)
            })
            .collect())
    }

    /// Every schematic and board file in the project's folder and the
    /// folders below it: a change to any sheet may change an answer.
    fn dependent_files(&self, file: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        if let Some(folder) = file.parent() {
            design_files_in(folder, &mut files);
        }
        files.sort();
        files
    }
}

/// Adds the `.kicad_sch` and `.kicad_pcb` files in `folder` and the
/// folders below it to `files`.
fn design_files_in(folder: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entries.filter_map(|entry| entry.ok()) {
        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            design_files_in(&path, files);
        } else if path
            .extension()
            .is_some_and(|extension| extension == "kicad_sch" || extension == "kicad_pcb")
        {
            files.push(path);
        }
    }
}

/// The key of `pcb export stats` a board property is read from:
/// `Width` is `board.width`.
fn dimension_key(property: &str) -> Option<&'static str> {
    DIMENSIONS
        .iter()
        .find(|(name, _)| *name == property)
        .map(|(_, key)| *key)
}

/// `board.<key>` of the statistics, `"58.4200 mm"`, as SysML writes it:
/// `58.42 [mm]`.
fn dimension(statistics: &Value, key: &str) -> Result<String, String> {
    let Some(text) = statistics["board"][key].as_str() else {
        return Err(format!("KiCad's board statistics have no board.{key}"));
    };
    let unreadable = || format!("KiCad gave board.{key} as '{text}', which can't be read");
    let (number, unit) = text.trim().split_once(' ').ok_or_else(unreadable)?;
    let number: f64 = number.parse().map_err(|_| unreadable())?;
    let unit = match unit.trim() {
        "mm" => "mm",
        "mm²" => "mm^2",
        _ => return Err(unreadable()),
    };
    Ok(format!("{number:?} [{unit}]"))
}

/// A run's temporary folder, holding the config folder KiCad is pointed
/// at, and any file the run writes. It is removed when dropped.
struct Workspace {
    dir: tempfile::TempDir,
    /// The `KICAD_CONFIG_HOME` of the run, whose
    /// `10.0/kicad_common.json` sets the language to English.
    config: PathBuf,
}

impl Workspace {
    fn new() -> Result<Self, String> {
        let dir = tempfile::tempdir().map_err(|e| format!("no temporary folder for KiCad: {e}"))?;
        let config = dir.path().join("config");
        let common = config.join("10.0");
        std::fs::create_dir_all(&common)
            .and_then(|()| {
                std::fs::write(
                    common.join("kicad_common.json"),
                    r#"{"system":{"language":"English"}}"#,
                )
            })
            .map_err(|e| format!("could not write a KiCad config folder: {e}"))?;
        Ok(Self { dir, config })
    }
}
