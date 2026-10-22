//! Running a tool's executable as the adapters do: found by version among
//! install folders, given a time limit, and, for a Windows executable
//! called from WSL, given Windows paths and its variables through
//! `WSLENV`.

use std::ffi::OsString;
use std::io::Read;
use std::path::Path;
use std::process::{Child, Command};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

/// What a finished run printed: its stdout and its stderr.
pub(crate) struct Output {
    pub stdout: String,
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
