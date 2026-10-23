//! KiCad as an Engineering Tool: `kicad-cli` exports what a project's
//! links ask for. A link names the project's `.kicad_pro`; the adapter
//! reads the `.kicad_sch` and `.kicad_pcb` of the same name beside it.
//! The one object is `board`, whose `Width`, `Height`, `Thickness` and
//! `Area` come from `pcb export stats`. Its `ComponentCount` and any other
//! property, a symbol field (`Power`, `Price`) added up over the placed
//! parts, come from `sch export netlist`. Only the exports the wanted
//! properties need are run, side by side.
//!
//! Every run gets its own temporary `KICAD_CONFIG_HOME` whose
//! `kicad_common.json` sets the language to English: a KiCad set to
//! another language writes its statistics with decimal commas
//! (`58,4200 mm`), and locale variables don't change that. The user's own
//! KiCad settings are never read or written.

use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use serde_json::Value;
use viewer_core::engineering_tools::{
    Answer, EngineeringTool, Parts, ReadValue, Wanted, natural_order,
};

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

/// The board property giving the number of placed parts:
/// `@ToolProperty { name = "ComponentCount"; }`.
pub const COMPONENT_COUNT: &str = "ComponentCount";

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
    /// The major version `kicad-cli` said it is, once it has said it is
    /// new enough, so later reads don't ask again: `10` for 10.0.7.
    major_version: Option<u64>,
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
            major_version: None,
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

    /// Asks `kicad-cli` its version, once, and gives its major version:
    /// anything older than KiCad 10 is a reason (`KiCad 10 or newer
    /// needed; found 9.0.3`).
    fn check_version(&mut self, executable: &Path) -> Result<u64, String> {
        if let Some(major) = self.major_version {
            return Ok(major);
        }
        // `version` prints nothing in KiCad's language, so its run needs
        // no language setting, nor a version to file one under.
        let workspace = Workspace::new(None)?;
        let output = self.run(executable, &workspace, &["version".into()])?;
        let found = output.stdout.trim();
        let major = process::version_numbers(found).first().copied();
        match major {
            Some(major) if major >= OLDEST_VERSION => {
                self.major_version = Some(major);
                Ok(major)
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
    fn board_statistics(&self, executable: &Path, major: u64, pcb: &Path) -> Result<Value, String> {
        let workspace = Workspace::new(Some(major))?;
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

    /// The placed parts of the schematic `sch`, every sheet of it, from
    /// `sch export netlist`.
    fn placed_parts(&self, executable: &Path, major: u64, sch: &Path) -> Result<Vec<Part>, String> {
        let workspace = Workspace::new(Some(major))?;
        let report = workspace.dir.path().join("netlist.xml");
        let args = [
            "sch".into(),
            "export".into(),
            "netlist".into(),
            "--format".into(),
            "kicadxml".into(),
            "-o".into(),
            process::path_for(executable, &report, NAME)?,
            process::path_for(executable, sch, NAME)?,
        ];
        let output = self.run(executable, &workspace, &args)?;
        let text = std::fs::read_to_string(&report).map_err(|_| {
            format!(
                "KiCad gave no netlist for {}: {}",
                sch.display(),
                output.last_said().unwrap_or("it said nothing")
            )
        })?;
        placed_in(&text).map_err(|e| format!("KiCad's netlist can't be read: {e}"))
    }

    /// The exports the wanted properties need, run side by side by the
    /// `kicad-cli` of major version `major`: the board statistics of
    /// `pcb` and the placed parts of `sch`, each `None` when no property
    /// needs it.
    fn export(
        &self,
        executable: &Path,
        major: u64,
        pcb: Option<&Path>,
        sch: Option<PathBuf>,
    ) -> (Exported<Value>, Exported<Vec<Part>>) {
        std::thread::scope(|scope| {
            let statistics =
                pcb.map(|pcb| scope.spawn(move || self.board_statistics(executable, major, pcb)));
            let placed =
                sch.map(|sch| scope.spawn(move || self.placed_parts(executable, major, &sch)));
            (statistics.map(finished), placed.map(finished))
        })
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
        let major = self.check_version(&executable)?;
        let board = wanted.get(BOARD);
        let wants = |dimensions: bool| {
            board.is_some_and(|properties| {
                properties
                    .iter()
                    .any(|property| dimension_key(property).is_some() == dimensions)
            })
        };
        let (statistics, placed) = self.export(
            &executable,
            major,
            wants(true).then_some(pcb.as_path()),
            wants(false).then(|| file.with_extension("kicad_sch")),
        );
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
                        } else if let Some(key) = dimension_key(property) {
                            exported(&statistics).and_then(|statistics| dimension(statistics, key))
                        } else {
                            exported(&placed).and_then(|placed| from_parts(placed, property))
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

/// What an export gave, or why it gave nothing; `None` when no wanted
/// property needed it.
type Exported<T> = Option<Result<T, String>>;

/// The export's result once its run is over.
fn finished<T>(run: std::thread::ScopedJoinHandle<'_, Result<T, String>>) -> Result<T, String> {
    run.join()
        .unwrap_or_else(|_| Err("reading KiCad's export failed".into()))
}

/// What an export gave a property that needed it.
fn exported<T>(export: &Exported<T>) -> Result<&T, String> {
    match export {
        Some(Ok(value)) => Ok(value),
        Some(Err(reason)) => Err(reason.clone()),
        None => Err("KiCad was not asked for this property's export".into()),
    }
}

/// A placed part of the board: neither do-not-populate nor excluded from
/// the BOM.
struct Part {
    /// Its reference designator, `U1`.
    reference: String,
    /// Its symbol fields by name, `Power` = `45mW`; an empty field is
    /// left out.
    fields: BTreeMap<String, String>,
}

/// The placed parts in a netlist written by `sch export netlist --format
/// kicadxml`, which lists every component on every sheet.
fn placed_in(netlist: &str) -> Result<Vec<Part>, String> {
    let document = roxmltree::Document::parse(netlist).map_err(|e| e.to_string())?;
    let named = |node: &roxmltree::Node, tag: &str| node.has_tag_name(tag);
    Ok(document
        .descendants()
        .filter(|node| named(node, "comp"))
        .filter(|comp| {
            !comp.children().any(|child| {
                named(&child, "property")
                    && matches!(child.attribute("name"), Some("dnp" | "exclude_from_bom"))
            })
        })
        .map(|comp| Part {
            reference: comp.attribute("ref").unwrap_or_default().to_string(),
            fields: comp
                .children()
                .filter(|child| named(child, "fields"))
                .flat_map(|fields| fields.children())
                .filter(|field| named(field, "field"))
                .filter_map(|field| {
                    let value = field.text().unwrap_or_default().trim();
                    (!value.is_empty())
                        .then(|| Some((field.attribute("name")?.to_string(), value.to_string())))?
                })
                .collect(),
        })
        .collect())
}

/// A board property read from the placed parts: `ComponentCount`, their
/// number, or any other name, that field added up over them (ADR-0006).
fn from_parts(placed: &[Part], property: &str) -> Result<ReadValue, String> {
    if property == COMPONENT_COUNT {
        return Ok(ReadValue::new(placed.len().to_string()));
    }
    let mut uncounted = Vec::new();
    let mut unreadable = Vec::new();
    // The counted parts by unit, `None` for bare numbers.
    let mut by_unit: BTreeMap<Option<String>, Vec<(&str, f64)>> = BTreeMap::new();
    for part in placed {
        let reference = part.reference.as_str();
        match part.fields.get(property) {
            None => uncounted.push(reference),
            Some(text) => match quantity(text) {
                Some((value, unit)) => by_unit.entry(unit).or_default().push((reference, value)),
                None => unreadable.push((reference, text.as_str())),
            },
        }
    }
    if !unreadable.is_empty() {
        unreadable.sort_by(|a, b| natural_order(a.0, b.0));
        let named: Vec<String> = unreadable
            .iter()
            .map(|(reference, text)| format!("{reference} ('{text}')"))
            .collect();
        return Err(format!(
            "the {property} field can't be read as a number with a unit on {}",
            named.join(", ")
        ));
    }
    let references = |parts: &[(&str, f64)]| {
        let mut references: Vec<&str> = parts.iter().map(|(reference, _)| *reference).collect();
        references.sort_by(|a, b| natural_order(a, b));
        references.join(", ")
    };
    if let Some(bare) = by_unit.get(&None).filter(|_| by_unit.len() > 1) {
        let with_units: Vec<(&str, f64)> = by_unit
            .iter()
            .filter(|(unit, _)| unit.is_some())
            .flat_map(|(_, parts)| parts.iter().copied())
            .collect();
        return Err(format!(
            "the {property} fields mix bare numbers ({}) with units ({})",
            references(bare),
            references(&with_units)
        ));
    }
    if by_unit.len() > 1 {
        let units: Vec<String> = by_unit
            .iter()
            .map(|(unit, parts)| {
                format!("{} on {}", unit.as_deref().unwrap_or(""), references(parts))
            })
            .collect();
        return Err(format!(
            "the {property} fields mix units: {}",
            units.join("; ")
        ));
    }
    // Folded from 0.0: an empty `sum` of floats is -0.0.
    let total = tidy(
        by_unit
            .values()
            .flatten()
            .fold(0.0, |total, (_, value)| total + value),
    );
    let value = match by_unit.keys().next() {
        Some(Some(unit)) => format!("{total:?} [{unit}]"),
        _ => format!("{total:?}"),
    };
    let counted = by_unit.values().flatten().map(|(reference, _)| *reference);
    Ok(ReadValue::summed(value, Parts::new(counted, uncounted)))
}

/// The SI prefixes a field's unit may start with, and their factors.
const PREFIXES: [(char, f64); 9] = [
    ('p', 1e-12),
    ('n', 1e-9),
    ('u', 1e-6),
    ('µ', 1e-6),
    ('μ', 1e-6),
    ('m', 1e-3),
    ('k', 1e3),
    ('M', 1e6),
    ('G', 1e9),
];

/// A field read as a number with an optional unit, given in the unit's
/// base symbol: `45mW` is 0.045 in `W`, `0.12` is 0.12 with no unit.
/// `None` when it can't be read (`lots`, `TBD`, `1,5 W`).
fn quantity(text: &str) -> Option<(f64, Option<String>)> {
    let text = text.trim();
    let end = text
        .char_indices()
        .find(|&(i, c)| !(c.is_ascii_digit() || c == '.' || (i == 0 && matches!(c, '+' | '-'))))
        .map_or(text.len(), |(i, _)| i);
    let (number, unit) = text.split_at(end);
    let number: f64 = number.parse().ok()?;
    let unit = unit.trim();
    if unit.is_empty() {
        return Some((number, None));
    }
    if !unit.chars().all(char::is_alphabetic) {
        return None;
    }
    let mut chars = unit.chars();
    let first = chars.next()?;
    let base = chars.as_str();
    match PREFIXES.iter().find(|(prefix, _)| *prefix == first) {
        Some((_, factor)) if !base.is_empty() => Some((number * factor, Some(base.to_string()))),
        _ => Some((number, Some(unit.to_string()))),
    }
}

/// `total` without the noise of adding binary fractions: `0.045 + 1.2`
/// is `1.245`, not `1.2449999999999999`.
fn tidy(total: f64) -> f64 {
    format!("{total:.12e}").parse().unwrap_or(total)
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
fn dimension(statistics: &Value, key: &str) -> Result<ReadValue, String> {
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
    Ok(ReadValue::new(format!("{number:?} [{unit}]")))
}

/// A run's temporary folder, holding the config folder KiCad is pointed
/// at, and any file the run writes. It is removed when dropped.
struct Workspace {
    dir: tempfile::TempDir,
    /// The `KICAD_CONFIG_HOME` of the run, whose
    /// `<major>.0/kicad_common.json` sets the language to English
    /// (`10.0/` for KiCad 10.0.7, `11.0/` for 11.0.1).
    config: PathBuf,
}

impl Workspace {
    /// A run's folder, with English set for a KiCad of major version
    /// `major`; with no language setting when `None`.
    fn new(major: Option<u64>) -> Result<Self, String> {
        let dir = tempfile::tempdir().map_err(|e| format!("no temporary folder for KiCad: {e}"))?;
        let config = dir.path().join("config");
        let Some(major) = major else {
            std::fs::create_dir_all(&config)
                .map_err(|e| format!("could not write a KiCad config folder: {e}"))?;
            return Ok(Self { dir, config });
        };
        let common = config.join(format!("{major}.0"));
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
