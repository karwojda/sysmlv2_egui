//! Running a tool's executable as the adapters do: found by version among
//! install folders, given a time limit, and, for a Windows executable
//! called from WSL, given Windows paths and its variables through
//! `WSLENV`.

use std::ffi::OsString;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// What a finished run printed: its stdout and its stderr.
pub(crate) struct Output {
    /// Everything the run wrote to stdout, decoded lossily as UTF-8:
    /// `10.0.7` for `kicad-cli version`.
    pub stdout: String,
    /// Everything the run wrote to stderr, decoded the same way: where a
    /// tool says why it failed.
    pub stderr: String,
}

impl Output {
    /// The last line the run printed that isn't blank, stderr first:
    /// what a tool says when it fails (`Failed to load board: …`).
    pub fn last_said(&self) -> Option<&str> {
        [&self.stderr, &self.stdout].into_iter().find_map(|text| {
            text.lines()
                .rev()
                .map(str::trim)
                .find(|line| !line.is_empty())
        })
    }
}

/// How one tool's executable is looked for on a machine, and what the
/// reasons for not finding it say.
pub(crate) struct Executable {
    /// The tool as reasons name it: `KiCad not available: …`.
    pub tool: &'static str,
    /// The variable naming the executable, `SYSML_VIEWER_KICAD`.
    pub variable: &'static str,
    /// The names it goes by, each also with `.exe`; reasons give the
    /// first: `kicad-cli`.
    pub names: &'static [&'static str],
    /// The installs in a folder installs go into, each a folder named
    /// for its version: `KiCad/10.0` in `/mnt/c/Program Files`.
    pub installs_in: fn(&Path) -> Vec<PathBuf>,
    /// The folders in an install that may hold it: `bin`,
    /// `Contents/MacOS`.
    pub bins: &'static [&'static str],
}

impl Executable {
    /// The executable to run, or why there is none: the one `named` by
    /// the variable, else the first on `path`, else the newest install in
    /// `install_dirs`. A set variable is final: when it names nothing,
    /// `PATH` is not searched, so a mistyped setting isn't hidden by
    /// another install.
    pub fn find(
        &self,
        named: Option<&OsString>,
        path: Option<&OsString>,
        install_dirs: &[PathBuf],
    ) -> Result<PathBuf, String> {
        let Self { tool, variable, .. } = self;
        if let Some(named) = named {
            let named = PathBuf::from(named);
            return if named.is_file() {
                Ok(named)
            } else {
                Err(format!(
                    "{tool} not available: {variable} names {}, which does not exist",
                    named.display()
                ))
            };
        }
        let on_path = || std::env::split_paths(path?).find_map(|dir| self.in_dir(&dir));
        on_path()
            .or_else(|| self.installed(install_dirs))
            .ok_or_else(|| {
                format!(
                    "{tool} not available: no {} in {variable}, on PATH \
                     or in the usual install locations",
                    self.names[0]
                )
            })
    }

    /// The newest install in the first of `install_dirs` that has one, by
    /// the version numbers in its folder's name: `KiCad/10.0` before
    /// `KiCad/9.0`, `FreeCAD 1.10` before `FreeCAD 1.9`.
    fn installed(&self, install_dirs: &[PathBuf]) -> Option<PathBuf> {
        install_dirs.iter().find_map(|dir| {
            let mut installs = (self.installs_in)(dir);
            installs.sort_by_cached_key(|install| {
                version_numbers(&install.file_name().unwrap_or_default().to_string_lossy())
            });
            installs.iter().rev().find_map(|install| {
                self.bins
                    .iter()
                    .find_map(|bin| self.in_dir(&install.join(bin)))
            })
        })
    }

    /// The executable in `dir`, by any of its names, with or without
    /// `.exe`.
    fn in_dir(&self, dir: &Path) -> Option<PathBuf> {
        self.names
            .iter()
            .flat_map(|name| [PathBuf::from(name), PathBuf::from(format!("{name}.exe"))])
            .map(|name| dir.join(name))
            .find(|candidate| candidate.is_file())
    }
}

/// What is in `dir`: nothing when it can't be read.
pub(crate) fn entries(dir: &Path) -> Vec<PathBuf> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|entry| entry.ok())
                .map(|entry| entry.path())
                .collect()
        })
        .unwrap_or_default()
}

/// The version numbers in a name, in order: `FreeCAD 1.10` is `[1, 10]`,
/// `10.0` is `[10, 0]`, a plain `FreeCAD` is `[]`.
pub(crate) fn version_numbers(name: &str) -> Vec<u64> {
    name.split(|c: char| !c.is_ascii_digit())
        .filter_map(|digits| digits.parse().ok())
        .collect()
}

/// The time limit a setting such as `SYSML_VIEWER_FREECAD_TIMEOUT=2.5`
/// gives, `default` when it is not set, or why it can't be used.
pub(crate) fn timeout(
    setting: Option<&OsString>,
    variable: &str,
    default: Duration,
) -> Result<Duration, String> {
    let Some(setting) = setting else {
        return Ok(default);
    };
    let text = setting.to_string_lossy();
    text.trim()
        .parse::<f64>()
        .ok()
        .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
        .filter(|duration| !duration.is_zero())
        .ok_or_else(|| format!("{variable} is {text}, not a number of seconds"))
}

/// Waits for `child` up to `timeout`, then kills it (`KiCad did not
/// answer within 30 s`, for `tool` KiCad).
pub(crate) fn finish(mut child: Child, timeout: Duration, tool: &str) -> Result<Output, String> {
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
                    "{tool} did not answer within {} s",
                    timeout.as_secs_f64()
                ));
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(format!("{tool} could not be waited for: {e}")),
        }
    }
    let text = |reader: JoinHandle<String>| reader.join().unwrap_or_default();
    Ok(Output {
        stdout: text(stdout),
        stderr: text(stderr),
    })
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

/// A Windows `.exe` run from Linux, which is WSL: it needs Windows paths
/// and its variables passed through `WSLENV`.
pub(crate) fn is_windows_executable(executable: &Path) -> bool {
    cfg!(unix)
        && executable
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("exe"))
}

/// `path` as `executable` sees it: unchanged, or for a Windows executable
/// seen from WSL as Windows sees it (`/tmp/x` as
/// `\\wsl.localhost\Ubuntu\tmp\x`, `/mnt/c/x` as `C:\x`).
pub(crate) fn path_for(executable: &Path, path: &Path, tool: &str) -> Result<String, String> {
    if !is_windows_executable(executable) {
        return Ok(path.to_string_lossy().into_owned());
    }
    let output = Command::new("wslpath")
        .arg("-w")
        .arg(path)
        .output()
        .map_err(|e| format!("wslpath could not be run for a Windows {tool}: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "wslpath could not convert {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Sets `variable` to `value` for `command`, and when `executable` is a
/// Windows one seen from WSL also lists it in `WSLENV` so it gets there.
pub(crate) fn set_variable(command: &mut Command, executable: &Path, variable: &str, value: &str) {
    command.env(variable, value);
    if is_windows_executable(executable) {
        let shared = match std::env::var("WSLENV") {
            Ok(existing) if !existing.is_empty() => format!("{existing}:{variable}"),
            _ => variable.to_owned(),
        };
        command.env("WSLENV", shared);
    }
}
