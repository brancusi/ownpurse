//! The in-place upgrade (v0.10.3): when a newer `ownpurse-tui` is installed while this one runs, the user
//! (palette `Upgrade to vX`) or an agent (`ownpurse ui upgrade`, request kind `upgrade`) can re-exec into it
//! with the view restored exactly, mid-typing included, from the canonical state (v0.10.2, V21).
//!
//! The handover, in order (`handover`):
//! 1. A running child (sync / login) defers it (`deferred`); it applies by itself once the run ends,
//!    after its reload (`Runtime::send`).
//! 2. The state and the live frame go to a temp file, 0600, in the bridge's private 0700 socket directory.
//! 3. Pre-flight: the NEW binary runs headless with `--check-state FILE`: it imports the state (record and
//!    config must be unchanged: any drift fails; a schema it can't migrate fails), renders it at the
//!    state's size and compares it with the live frame. Any failure aborts: the old process carries on
//!    unchanged, the file is deleted, the reason goes to the TUI note and the agent's answer.
//! 4. The terminal is released only as far as exec needs (`Handover::release`): synchronized output,
//!    mouse reporting, bracketed paste and the keyboard-enhancement flags are popped and raw mode is left,
//!    but the alternate screen is KEPT (no flash of the shell's screen; the new process redraws it). The
//!    bridge socket closes and its registry entry goes. Then `exec` in the same terminal, same pid:
//!    `<args> --resume FILE [--resume-reply FD]`.
//! 5. The new process imports the file and deletes it, re-registers the socket (same pid, same session id,
//!    same start time), shows `updated to vX`, and answers the agent on the inherited connection.
//! 6. If exec fails, the old process takes the terminal back, re-opens its socket and reports why.

use crate::app::App;
use crate::canonical::{AllowDrift, ImportError};
use crate::runtime::Runtime;
use ratatui::backend::Backend;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// `update_available.action` when the installed build can take over in place.
pub const ACTION: &str = "upgrade_in_place";

/// The old build's full identity, passed to the new one across the handover.
pub const FROM_ENV: &str = "OWNPURSE_UPGRADED_FROM";

/// How long the pre-flight may take (the bridge answers within 10 s).
pub const PREFLIGHT_TIMEOUT: Duration = Duration::from_secs(8);

/// Environment variable carrying the session's original start time across exec (`ui ls` keeps it).
pub const STARTED_ENV: &str = "OWNPURSE_SESSION_STARTED";

/// This build's render epoch: bumped deliberately when the visual design changes (every frame does), so an
/// in-place upgrade across it checks the state rather than the frame. Builds before v0.12.0 are epoch 0,
/// v0.12.0 to v0.14.0 epoch 1; v0.15.0 (overlay placement at the row's text) is epoch 2.
pub const RENDER_EPOCH: u32 = 2;

/// The render epoch a handed-over canonical block was drawn in: its `render_epoch`, else derived from the
/// build that wrote it (`env.version_label` below 0.12.0 → 0, else 1).
pub fn epoch_of(canonical: &Value) -> u32 {
    if let Some(e) = canonical.get("render_epoch").and_then(Value::as_u64) {
        return u32::try_from(e).unwrap_or(u32::MAX);
    }
    let v = canonical["env"]["version_label"].as_str().unwrap_or("");
    let n: Vec<u64> = v
        .split(['.', '-', '+'])
        .take(3)
        .map(|p| p.parse().unwrap_or(0))
        .collect();
    let at_least_0_12 = (
        n.first().copied().unwrap_or(0),
        n.get(1).copied().unwrap_or(0),
    ) >= (0, 12);
    // every build from v0.12.0 writes `render_epoch`; a block without it predates that
    if at_least_0_12 { 1 } else { 0 }
}

/// The epochs of a state file (its canonical block → this build), when they differ.
pub fn epoch_change(text: &str) -> Option<(u32, u32)> {
    let v: Value = serde_json::from_str(text).ok()?;
    let c = v
        .get("state")
        .and_then(|s| s.get("canonical"))
        .or_else(|| v.get("canonical"))?;
    let from = epoch_of(c);
    (from != RENDER_EPOCH).then_some((from, RENDER_EPOCH))
}

/// The widths the cross-epoch check renders at (V21: export → import → render equals the render).
pub const EPOCH_WIDTHS: [u16; 4] = [60, 80, 100, 140];

/// Within this build: the imported state renders, round-trips (export → import) and renders the same, at
/// each width, without a panic.
pub fn self_consistent(text: &str, source: &crate::model::Source) -> Result<(), String> {
    for w in EPOCH_WIDTHS {
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<(), String> {
            let mut a = App::from_state_json_with(text, source.clone(), AllowDrift::default())
                .map_err(|e| format!("{}: {}", e.code(), e.message()))?;
            let h = a.ui.size.1.max(20);
            let first = crate::render::ansi(&crate::snapshot::render(&mut a, w, h));
            let exported = a.view_state(a.rev).canonical();
            let mut b = App::from_state_json_with(&exported, source.clone(), AllowDrift::default())
                .map_err(|e| format!("{}: {}", e.code(), e.message()))?;
            let again = crate::render::ansi(&crate::snapshot::render(&mut b, w, h));
            if first != again {
                return Err(format!(
                    "the new build's own round trip differs at {w} columns"
                ));
            }
            Ok(())
        }));
        match r {
            Ok(r) => r?,
            Err(_) => {
                return Err(format!(
                    "the new build panicked rendering the state at {w} columns"
                ));
            }
        }
    }
    Ok(())
}

/// The reply field and the note for an epoch change.
pub const EPOCH_CHANGED: &str = "changed (visual redesign)";

/// What an upgrade request finds.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Ask {
    /// No newer build installed (or one that can't take over in place).
    UpToDate,
    /// A child is running: the upgrade to this version applies when it ends.
    Deferred(String),
    /// Go: hand over to this version now.
    Go(String),
}

/// Why a handover didn't happen (a successful one never returns: the process is replaced).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Outcome {
    PreflightFailed(String),
    ExecFailed(String),
}

impl Outcome {
    pub fn code(&self) -> &'static str {
        match self {
            Outcome::PreflightFailed(_) => "preflight_failed",
            Outcome::ExecFailed(_) => "exec_failed",
        }
    }
    pub fn reason(&self) -> &str {
        match self {
            Outcome::PreflightFailed(r) | Outcome::ExecFailed(r) => r,
        }
    }
    /// The bridge's answer (exit 1; the TUI is unchanged).
    pub fn doc(&self, from: &str, to: &str) -> Value {
        let mut d = ownpurse_core::bridge::error_doc(
            self.code(),
            1,
            match self {
                Outcome::PreflightFailed(r) => {
                    format!(
                        "The upgrade to v{to} was not started: the pre-flight failed ({r}). The TUI is unchanged."
                    )
                }
                Outcome::ExecFailed(r) => {
                    format!(
                        "The upgrade to v{to} failed to start ({r}). The TUI carries on as v{from}."
                    )
                }
            },
            &[],
        );
        d["status"] = json!(self.code());
        d["reason"] = json!(self.reason());
        d["from"] = json!(from);
        d["to"] = json!(to);
        d
    }
}

/// The success answer (written by the NEW process on the inherited connection).
pub fn upgraded_doc(from: &str, to: &str, rev: u64, epoch: Option<(u32, u32)>) -> Value {
    let mut d = json!({"v": ownpurse_core::bridge::PROTOCOL, "ok": true, "status": "upgraded", "from": from, "to": to, "rev": rev});
    if let Some((a, b)) = epoch {
        d["render_epoch"] = json!(EPOCH_CHANGED);
        d["render_epoch_from"] = json!(a);
        d["render_epoch_to"] = json!(b);
    }
    d
}

/// The answers that don't hand over: `up_to_date`, `deferred`, and `upgrading` (the loop holds that one
/// and hands over; the agent sees `upgraded` or a failure instead).
pub fn ask_doc(ask: &Ask, from: &str, rev: u64) -> Value {
    let p = ownpurse_core::bridge::PROTOCOL;
    match ask {
        Ask::UpToDate => {
            json!({"v": p, "ok": true, "status": "up_to_date", "version": from, "rev": rev})
        }
        Ask::Deferred(to) => json!({
            "v": p, "ok": true, "status": "deferred", "from": from, "to": to, "rev": rev,
            "message": format!("Deferred until the sync finishes: the upgrade to v{to} applies by itself after it."),
        }),
        Ask::Go(to) => {
            json!({"v": p, "ok": true, "status": "upgrading", "from": from, "to": to, "rev": rev})
        }
    }
}

/// The note while an upgrade waits for a running child (it applies by itself): `sync design`, `login`.
pub fn deferred_note(_to: &str, what: &str) -> String {
    format!("Upgrade will run when {what} finishes")
}

/// What a failed pre-flight means for the user, and what to do: `(reason, remedy)`. The full reason
/// goes to Activity and the agent's answer.
pub fn preflight_remedy(reason: &str) -> (&'static str, &'static str) {
    let code = reason.split_once(": ").map_or(reason, |(c, _)| c);
    match code {
        "record_changed" => ("record changed", "r reload, then retry"),
        "config_changed" => ("config changed", "fix config, then retry"),
        "classify_changed" => ("labels changed", "r reload, then retry"),
        // a schema the new build can't read, or a frame it draws differently: a plain restart is safe
        "bad_state" => ("state not readable", "quit and restart to update"),
        _ if reason.starts_with("the view state would not survive") => {
            ("view state would be lost", "quit and restart to update")
        }
        _ if reason.starts_with("the restored frame differs") => {
            ("view would change", "quit and restart to update")
        }
        _ if reason.starts_with("the state file carries no live frame") => {
            ("state incomplete", "retry")
        }
        // the new build didn't run, timed out, or doesn't know --check-state
        _ => ("new build failed its check", "reinstall, then retry"),
    }
}

/// The note after a handover that didn't happen.
pub fn failed_note(o: &Outcome, from: &str, _to: &str) -> String {
    match o {
        Outcome::PreflightFailed(r) => {
            let (why, fix) = preflight_remedy(r);
            format!("Upgrade not started: {why} · {fix}")
        }
        Outcome::ExecFailed(r) => format!("Upgrade failed · still v{from} · {r}"),
    }
}

/// A shorter form of an upgrade note for a narrow footer: the reason goes first, the remedy stays
/// (`Upgrade not started · r reload, then retry`). None when there is nothing to drop.
pub fn short_note(n: &str) -> Option<String> {
    let rest = n.strip_prefix("Upgrade not started: ")?;
    let (_, fix) = rest.split_once(" · ")?;
    Some(format!("Upgrade not started · {fix}"))
}

/// The remedy's short form for 60–79 columns: `r reload, retry`, `fix config, retry`, `reinstall, retry`.
pub fn narrow_note(n: &str) -> String {
    n.replace(", then retry", ", retry")
}

/// An upgrade note (the footer keeps all of it; key hints drop first).
pub fn is_note(n: &str) -> bool {
    n.starts_with("Upgrade ") || n.starts_with("updated v")
}

/// The note the new build shows after it took over (`updated v0.10.2 → v0.10.3 · state kept`).
pub fn updated_note(from: &str, to: &str) -> String {
    format!("updated v{from} → v{to} · state kept")
}

/// The same across a render epoch: `updated v0.11.1 → v0.12.0 · new look · state kept`.
pub fn updated_note_new_look(from: &str, to: &str) -> String {
    format!("updated v{from} → v{to} · new look · state kept")
}

/// What the terminal loop provides (tests pass fakes).
pub trait Handover {
    /// Release the terminal as far as exec needs and close the bridge (socket file and registry entry).
    fn release(&mut self) -> std::io::Result<()>;
    /// Replace the process. Returns only on failure.
    fn exec(&mut self, program: &Path, args: &[String], env: &[(String, String)])
    -> std::io::Error;
    /// After a failed exec: the terminal and the bridge back as they were.
    fn reenter(&mut self) -> std::io::Result<()>;
}

/// Where and how to hand over.
#[derive(Clone, Debug)]
pub struct Plan {
    /// The new binary (this binary's path on disk; a reinstall replaced it).
    pub program: PathBuf,
    /// The arguments this process was started with, minus the one-shot ones (`base_args`).
    pub args: Vec<String>,
    /// The private directory for the state file (the bridge's socket directory, 0700).
    pub dir: PathBuf,
    /// The agent's connection, inherited across exec (`--resume-reply FD`).
    pub reply_fd: Option<i32>,
    /// Extra environment for the new process (the session start time, synchronized-output support).
    pub env: Vec<(String, String)>,
    /// How long the pre-flight may take ([`PREFLIGHT_TIMEOUT`] live; tests on a loaded machine give more).
    pub preflight_timeout: Duration,
}

/// The start-up arguments without the one-shot ones (`--resume`, `--resume-reply`, `--check-state`).
pub fn base_args(args: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut out = Vec::new();
    let mut it = args.into_iter();
    while let Some(a) = it.next() {
        if ["--resume", "--resume-reply", "--check-state"].contains(&a.as_str()) {
            it.next();
            continue;
        }
        out.push(a);
    }
    out
}

/// Write the state and the live frame (ANSI, styles included) to `<dir>/upgrade-<pid>.json`, 0600.
pub fn export(app: &App, frame: &ratatui::buffer::Buffer, dir: &Path) -> std::io::Result<PathBuf> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    ownpurse_core::bridge::private_dir(dir)?;
    let path = dir.join(format!("upgrade-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let doc = json!({
        "state": app.state_json(app.rev),
        "frame": crate::render::ansi(frame),
    });
    let mut f = std::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&path)?;
    f.write_all(ownpurse_core::canon::canonical(&doc).as_bytes())?;
    f.write_all(b"\n")?;
    f.sync_all()?;
    Ok(path)
}

/// Run `program <args> --check-state FILE` headless (no terminal, no socket), within [`PREFLIGHT_TIMEOUT`].
pub fn preflight(program: &Path, args: &[String], file: &Path) -> Result<(), String> {
    preflight_within(program, args, file, PREFLIGHT_TIMEOUT)
}

/// [`preflight`] with an explicit timeout. Its output is drained while it runs (a chatty build can't block
/// on a full pipe and be mistaken for a hung one).
pub fn preflight_within(
    program: &Path,
    args: &[String],
    file: &Path,
    timeout: Duration,
) -> Result<(), String> {
    use std::io::Read;
    let mut child = std::process::Command::new(program)
        .args(args)
        .arg("--check-state")
        .arg(file)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| format!("could not run {}: {e}", program.display()))?;
    let drain = |p: Option<Box<dyn Read + Send>>| {
        std::thread::spawn(move || {
            let mut s = String::new();
            if let Some(mut p) = p {
                let _ = p.read_to_string(&mut s);
            }
            s
        })
    };
    let out_t = drain(
        child
            .stdout
            .take()
            .map(|o| Box::new(o) as Box<dyn Read + Send>),
    );
    let err_t = drain(
        child
            .stderr
            .take()
            .map(|e| Box::new(e) as Box<dyn Read + Send>),
    );
    let start = Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(s)) => break s,
            Ok(None) if start.elapsed() > timeout => {
                let _ = child.kill();
                let _ = child.wait();
                return Err(format!(
                    "the new build did not answer within {}s",
                    timeout.as_secs()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(e) => return Err(e.to_string()),
        }
    };
    let out = out_t.join().unwrap_or_default();
    let err = err_t.join().unwrap_or_default();
    if status.success() {
        return Ok(());
    }
    // the check prints `error: <code>` / `message: <why>`; an older or broken build prints something else
    let message = out
        .lines()
        .find_map(|l| l.strip_prefix("message: "))
        .map(str::to_string)
        .or_else(|| {
            err.lines()
                .chain(out.lines())
                .map(str::trim)
                .find(|l| !l.is_empty())
                .map(str::to_string)
        })
        .unwrap_or_else(|| format!("the new build exited with {status}"));
    Err(message)
}

/// The live handover's import allowance: data freshness tolerated (and unmarked), config drift refused.
const LIVE: AllowDrift = AllowDrift {
    record: false,
    config: false,
    classify: false,
    manifest: false,
    live: true,
};

/// `--check-state FILE` (in the NEW build, so an older running TUI gets the newer rule). Two checks, both
/// STRICT; either failure aborts the upgrade and the old TUI carries on:
/// 1. **State equivalence** (primary): import (record and config unchanged; an older schema migrated, or
///    refused), export again: the handed-over canonical state must survive whole. Every field it carries is
///    still there with the same value; a newer build may only ADD fields (with their defaults).
/// 2. **The frame**, within one render epoch: the restored frame equals the live one row for row, with the one
///    cross-version allowance the frozen-state test also applies (`snapshot::frame_difference`): the tab bar
///    may append new tabs. Across epochs (a visual redesign, [`RENDER_EPOCH`]) the old frame can't match, so
///    instead this build must be self-consistent: its own export → import → render equals its render at
///    60/80/100/140 columns, with no panic. Returns the epochs when they changed.
///
/// A live handover is not a file import: the record or the store may have moved since the export (another
/// process wrote). That is reloaded and the view-state equivalence re-verified on the current data (the
/// frame then can't equal the exported one, so it isn't compared); only a config that no longer fits the
/// view (`config_changed`, e.g. a removed org) or a lost view field fails.
pub fn check(text: &str, source: crate::model::Source) -> Result<Option<(u32, u32)>, String> {
    let v: Value = serde_json::from_str(text).map_err(|e| format!("not JSON: {e}"))?;
    let want = v["frame"]
        .as_str()
        .ok_or("the state file carries no live frame")?;
    let handed = v
        .get("state")
        .and_then(|s| s.get("canonical"))
        .or_else(|| v.get("canonical"))
        .ok_or("the state file carries no canonical state")?;
    let from = epoch_of(handed);
    // a live handover: data that moved since the export (another process wrote) is reloaded, never a
    // reason to fail; a config that no longer fits the view still is
    let mut app = App::from_state_json_with(text, source.clone(), LIVE)
        .map_err(|e| format!("{}: {}", e.code(), e.message()))?;
    let restored = app.canonical_value();
    let moved = handed.get("record") != restored.get("record");
    let mut handed = handed.clone();
    if moved && let (Some(h), Some(r)) = (handed.as_object_mut(), restored.get("record")) {
        // the revisions are the data's, not the view's: equivalence is checked on everything else
        h.insert("record".into(), r.clone());
    }
    if let Some(path) = lost(&handed, &restored, "canonical") {
        return Err(format!(
            "the view state would not survive the upgrade ({path} is lost or changed)"
        ));
    }
    if moved {
        // the frame shows the newer data now, so it can't equal the exported one; the view state did. The
        // dry run still draws a first frame (config, record and view all loaded): a build that can't start
        // fails here, while the old process still runs
        let (w, h) = app.ui.size;
        let _ = crate::snapshot::render(&mut app, w, h);
        return Ok(None);
    }
    if from != RENDER_EPOCH {
        self_consistent(text, &source)?;
        return Ok(Some((from, RENDER_EPOCH)));
    }
    let (w, h) = app.ui.size;
    let got = crate::render::ansi(&crate::snapshot::render(&mut app, w, h));
    if let Some((row, why)) = crate::snapshot::frame_difference(&got, want) {
        return Err(format!(
            "the restored frame differs from the live one (row {}: {why})",
            row + 1
        ));
    }
    Ok(None)
}

/// The first path of `old` that `new` doesn't carry with the same value (objects: every key of `old`, new
/// keys allowed; arrays: same length, element by element; scalars: equal). None: nothing is lost.
/// `canonical.render_epoch` is the drawing build's, not the view's: a newer epoch is not a loss.
pub fn lost(old: &Value, new: &Value, path: &str) -> Option<String> {
    if path == "canonical.render_epoch" {
        return None;
    }
    match (old, new) {
        (Value::Object(a), Value::Object(b)) => a.iter().find_map(|(k, va)| match b.get(k) {
            Some(vb) => lost(va, vb, &format!("{path}.{k}")),
            None => Some(format!("{path}.{k}")),
        }),
        (Value::Array(a), Value::Array(b)) if a.len() == b.len() => a
            .iter()
            .zip(b)
            .enumerate()
            .find_map(|(i, (x, y))| lost(x, y, &format!("{path}[{i}]"))),
        _ if old == new => None,
        _ => Some(path.to_string()),
    }
}

/// `--resume FILE` (in the NEW build): the exported view, with the process facts of this build (its
/// version, CLI, paths, clock) from `fresh`, and the `updated to vX` note. Returns the version it came from.
pub fn resume(
    text: &str,
    source: crate::model::Source,
    fresh: &App,
) -> Result<(App, String), ImportError> {
    let mut app = App::from_state_json_with(text, source, LIVE)?;
    // the old build's full identity (`0.12.1+g5aa1401`) when it said so; else its version
    let from = std::env::var(FROM_ENV)
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| app.version_label.clone());
    app.version_label = fresh.version_label.clone();
    app.build_label = fresh.build_label.clone();
    app.cli = fresh.cli.clone();
    app.cli_version = fresh.cli_version.clone();
    app.cli_shown = fresh.cli_shown.clone();
    app.paths = fresh.paths.clone();
    app.now = fresh.now;
    // this build is the update; detection runs again from here
    app.update_available = None;
    // a different build of the same version names the builds (`0.12.1+g5aa1401 → 0.12.1+g6dc723f`)
    let to = if from.contains('+') {
        crate::app::full_version(&app.version_label, app.build_label.as_deref())
    } else {
        app.version_label.clone()
    };
    app.ui.note = Some(if epoch_change(text).is_some() {
        updated_note_new_look(&from, &to)
    } else {
        updated_note(&from, &to)
    });
    // the newest news leads the strip (an older agent notice would hide the update note)
    app.agent.notice = None;
    app.invalidate();
    app.dirty = true;
    Ok((app, from))
}

/// The handover (module doc steps 2–6). Never returns when it succeeds (the process is replaced).
pub fn handover<B: Backend>(rt: &mut Runtime<B>, plan: &Plan, h: &mut dyn Handover) -> Outcome {
    let from = crate::app::full_version(&rt.app.version_label, rt.app.build_label.as_deref());
    let to = rt
        .app
        .update_available
        .as_ref()
        .map(|u| u.version.clone())
        .unwrap_or_default();
    let o = (|| -> Outcome {
        // first pick up what other processes wrote (the normal reload path): the state then carries the
        // current revisions and the new build's check sees no drift
        rt.external_check();
        rt.config_check();
        // the terminal must show what is exported: the new build draws over it, cell for cell, without a clear
        let _ = rt.frame();
        let frame = rt.current_frame();
        let file = match export(&rt.app, &frame, &plan.dir) {
            Ok(f) => f,
            Err(e) => return Outcome::PreflightFailed(format!("could not write the state: {e}")),
        };
        // never a handover to the build that is running (the binary on disk may have been reinstalled
        // with the identical build since it was offered): refused, the running process keeps going
        if let Some(v) = crate::net::tui_version_within(&plan.program, plan.preflight_timeout)
            && v == from
        {
            let _ = std::fs::remove_file(&file);
            return Outcome::PreflightFailed(format!("already running this build (v{v})"));
        }
        if let Err(r) = preflight_within(&plan.program, &plan.args, &file, plan.preflight_timeout) {
            let _ = std::fs::remove_file(&file);
            return Outcome::PreflightFailed(r);
        }
        if let Err(e) = h.release() {
            let _ = h.reenter();
            let _ = std::fs::remove_file(&file);
            return Outcome::ExecFailed(format!("could not release the terminal: {e}"));
        }
        let mut args = plan.args.clone();
        args.push("--resume".into());
        args.push(file.display().to_string());
        if let Some(fd) = plan.reply_fd {
            args.push("--resume-reply".into());
            args.push(fd.to_string());
        }
        let err = h.exec(&plan.program, &args, &plan.env);
        let back = h.reenter();
        let _ = std::fs::remove_file(&file);
        Outcome::ExecFailed(match back {
            Ok(()) => err.to_string(),
            Err(b) => format!("{err}; the terminal could not be fully restored: {b}"),
        })
    })();
    rt.app.ui.note = Some(failed_note(&o, &from, &to));
    // the newest news leads the strip: an older agent notice would hide why the upgrade didn't start
    rt.app.agent.notice = None;
    // the whole reason, in Activity (the footer keeps the remedy)
    let sel = rt.app.ui.activity.sel;
    rt.app
        .agent_log(&format!("upgrade to v{to} · {}: {}", o.code(), o.reason()));
    if rt.app.ui.tab != crate::app::Tab::Activity {
        rt.app.ui.activity.sel = sel; // only Activity's own cursor is kept from moving
    }
    rt.app.dirty = true;
    o
}
