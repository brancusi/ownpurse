//! Sync and login from the app (spec P6 network §2): the TUI never links the api crate (V1). It runs the
//! installed `ownpurse` CLI as a child process: the binary next to `ownpurse-tui` first, then `PATH`; argv
//! exactly `<cmd> [<target>] --profile <p> [--config <path>] --json --progress jsonl`; no shell; stdin null;
//! its own session (`setsid`), so terminal signals aimed at the TUI never reach it; the TUI never signals
//! it. stderr lines (progress JSON, or raw text) and the exit (stdout envelope) come back as events.

use std::ffi::OsStr;
use std::io::{BufRead, BufReader, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Instant;

/// What a run sends back to the loop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RunEvent {
    /// One stderr line: a progress event (JSON) or raw text for the failure band.
    Line(String),
    /// The child exited: its code (None when killed by a signal), stdout (the `--json` envelope), and how
    /// long it ran (shown only in the result).
    Exit {
        code: Option<i32>,
        stdout: String,
        elapsed_secs: u64,
    },
    /// The child couldn't start (the OS error).
    SpawnFailed(String),
}

/// The CLI's file name.
pub const CLI_NAME: &str = "ownpurse";

/// Spec copy when neither a sibling nor a PATH binary exists.
pub const NO_CLI: &str = "ownpurse isn't installed next to the app or on PATH. Run ./install.sh.";

fn executable(p: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(p).is_ok_and(|m| m.is_file() && m.permissions().mode() & 0o111 != 0)
    }
    #[cfg(not(unix))]
    {
        p.is_file()
    }
}

/// The `ownpurse` next to the running app (`exe_dir`) first, then the first on `path` (PATH's value).
pub fn resolve_cli(exe_dir: Option<&Path>, path: Option<&OsStr>) -> Option<PathBuf> {
    if let Some(d) = exe_dir {
        let sib = d.join(CLI_NAME);
        if executable(&sib) {
            return Some(sib);
        }
    }
    std::env::split_paths(path?)
        .map(|d| d.join(CLI_NAME))
        .find(|p| executable(p))
}

/// `resolve_cli` for this process: its executable's directory and `$PATH`.
pub fn resolve_cli_for_process() -> Option<PathBuf> {
    let exe = std::env::current_exe().ok();
    let dir = exe.as_deref().and_then(Path::parent);
    resolve_cli(dir, std::env::var_os("PATH").as_deref())
}

/// The child's arguments (after the program): exactly
/// `<cmd> [<target>] --profile <profile> [--config <path>] --json --progress jsonl`.
pub fn argv(cmd: &str, target: Option<&str>, profile: &str, config: Option<&Path>) -> Vec<String> {
    let mut v = vec![cmd.to_string()];
    if let Some(t) = target {
        v.push(t.to_string());
    }
    v.push("--profile".into());
    v.push(profile.to_string());
    if let Some(c) = config {
        v.push("--config".into());
        v.push(c.display().to_string());
    }
    v.extend(["--json", "--progress", "jsonl"].map(String::from));
    v
}

/// `<cli> --version` once (System's `cli:` row): the version word, e.g. `0.9.0`.
pub fn cli_version(cli: &Path) -> Option<String> {
    let out = Command::new(cli)
        .arg("--version")
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    let text = String::from_utf8_lossy(&out.stdout);
    text.split_whitespace().last().map(str::to_string)
}

/// `<tui> --version` within `timeout`: the version word of a line `ownpurse-tui <version>` (anything else, an
/// error or no answer in time: None). The child is killed when it doesn't answer.
pub fn tui_version_within(bin: &Path, timeout: std::time::Duration) -> Option<String> {
    use std::io::Read;
    let mut child = Command::new(bin)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    let mut out = child.stdout.take()?;
    let reader = std::thread::spawn(move || {
        let mut s = String::new();
        let _ = out.read_to_string(&mut s);
        s
    });
    let start = std::time::Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(st)) if st.success() => break,
            Ok(Some(_)) | Err(_) => return None,
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return None;
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(10)),
        }
    }
    let text = reader.join().ok()?;
    let line = text.lines().find(|l| l.starts_with("ownpurse-tui "))?;
    line.split_whitespace().nth(1).map(str::to_string)
}

/// Start the child and stream its events to `sink` from a background thread. Returns once the child
/// is running (or the OS refused to start it).
pub fn spawn(
    program: &Path,
    args: &[String],
    sink: impl Fn(RunEvent) + Send + Sync + 'static,
) -> Result<(), String> {
    spawn_with_env(program, args, &[], sink)
}

/// `spawn`, with extra environment variables for the child (tests drive the fake CLI this way).
pub fn spawn_with_env(
    program: &Path,
    args: &[String],
    envs: &[(String, String)],
    sink: impl Fn(RunEvent) + Send + Sync + 'static,
) -> Result<(), String> {
    let mut cmd = Command::new(program);
    cmd.envs(envs.iter().map(|(k, v)| (k.as_str(), v.as_str())));
    cmd.args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // SAFETY: setsid is async-signal-safe; nothing else runs between fork and exec.
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
    }
    let mut child = cmd.spawn().map_err(|e| e.to_string())?;
    let started = Instant::now();
    let stderr = child.stderr.take();
    let stdout = child.stdout.take();
    std::thread::spawn(move || {
        let out = std::thread::spawn(move || {
            let mut s = String::new();
            if let Some(mut o) = stdout {
                let _ = o.read_to_string(&mut s);
            }
            s
        });
        if let Some(e) = stderr {
            for line in BufReader::new(e).lines() {
                match line {
                    Ok(l) => sink(RunEvent::Line(l)),
                    Err(_) => break,
                }
            }
        }
        let stdout = out.join().unwrap_or_default();
        let code = child.wait().ok().and_then(|s| s.code());
        sink(RunEvent::Exit {
            code,
            stdout,
            elapsed_secs: started.elapsed().as_secs(),
        });
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn argv_is_exact() {
        assert_eq!(
            argv("sync", Some("design"), "demo", None),
            [
                "sync",
                "design",
                "--profile",
                "demo",
                "--json",
                "--progress",
                "jsonl"
            ]
        );
        assert_eq!(
            argv("login", None, "demo", Some(Path::new("/w/ownpurse.json"))),
            [
                "login",
                "--profile",
                "demo",
                "--config",
                "/w/ownpurse.json",
                "--json",
                "--progress",
                "jsonl"
            ]
        );
    }
}
