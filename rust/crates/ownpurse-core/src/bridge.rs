//! The TUI ↔ agent bridge's shared pieces (v0.10): where the sockets and the session registry live, the
//! registry itself, and the client side of the protocol. The running TUI is the server (it never links
//! the api crate); the CLI's `ownpurse ui …` commands are clients. Local only: a Unix socket, mode 0600,
//! in a 0700 directory. Never TCP.
//!
//! Protocol: one newline-terminated JSON request per connection, one newline-terminated JSON response.
//! Requests carry `{"v": 1, "kind": "view", "op": …}`; `kind` is the extension point (`annotate` and
//! `task` are reserved and answer `not_available` in v0.10). Responses carry `"ok"`, and on failure the
//! CLI's error shape (`error`, `message`, `help`) plus the exit code to use (`exit`).

use serde_json::{Map, Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// The protocol version every request and response carries.
pub const PROTOCOL: i64 = 1;

/// Request kinds: `view` (the view verbs) and, from v0.10.3, `upgrade` (op `upgrade`: the ONE non-view
/// action an agent may send: re-exec the TUI into a newer installed build with the exact view restored);
/// from v0.11.0, `classify` (local classification writes, `core::classify::request`, schema `cv` 1); the
/// others are reserved and answer `not_available`.
pub const KINDS: [&str; 5] = ["view", "upgrade", "classify", "annotate", "task"];

/// The allow-list: view verbs, with how many arguments each takes, plus (v0.12.0) the cosmetic theme verbs
/// `theme`, `accent` and `overlay` (theme spec v2 §2.7). [`REST`] = all the words after it (`overlay set
/// a=b c=d`, `overlay clear [token…]`), so it is the last verb of a list.
pub const VERBS: [(&str, usize); 29] = [
    ("tab", 1),
    ("scope", 1),
    ("asof", 1),
    ("filter", 1),
    ("clear-filter", 0),
    ("select", 1),
    ("drill", 0),
    ("up", 0),
    ("back", 0),
    ("forward", 0),
    ("basis", 1),
    ("differs", 1),
    ("compare", 1),
    ("reload", 0),
    // "Open in Xero": the selected document's page in the browser (a link, never an API call)
    ("open", 0),
    ("caption", 1),
    ("label", 1),
    ("theme", 1),
    ("accent", 1),
    ("overlay", REST),
    // manifests: show one (or `main`), and tab 7's mode; entries are added with the CLI (`manifest … --client`)
    ("manifest", 1),
    ("mode", 1),
    // walkthroughs §4: open an explainer panel (`viz <spec-json>`) or close it (`viz close`); cosmetic
    ("viz", 1),
    // overlays (walkthroughs design §3): cosmetic, never refused while the person types
    ("hint", REST),
    ("highlight", 1),
    ("spotlight", REST),
    ("clear-hints", REST),
    ("narrate", REST),
    // walkthroughs: play one (view verbs only inside), step through it, record one
    ("walkthrough", REST),
];

/// The view verbs a walkthrough step may use (never overlays, narration, walkthroughs, theme or session verbs).
pub const VIEW_ONLY: [&str; 13] = [
    "tab",
    "scope",
    "asof",
    "filter",
    "clear-filter",
    "select",
    "drill",
    "up",
    "basis",
    "differs",
    "compare",
    "manifest",
    "mode",
];

/// The arity of a verb that takes every remaining word (`overlay`).
pub const REST: usize = usize::MAX;

/// The cosmetic verbs: never refused for `user_busy` (they move no focus and touch no input).
pub const COSMETIC: [&str; 9] = [
    "theme",
    "accent",
    "overlay",
    "hint",
    "highlight",
    "spotlight",
    "clear-hints",
    "narrate",
    "viz",
];

/// The words after a verb that takes the rest of the list: `overlay`, `hint`, `spotlight`, `clear-hints`,
/// `narrate`, `walkthrough`.
fn check_rest(verb: &str, args: &[String]) -> Result<(), VerbError> {
    let flags = |a: &[String]| a.iter().filter(|w| !w.starts_with("--")).count();
    match verb {
        "overlay" => check_overlay_words(args),
        "hint" if flags(args) >= 2 => Ok(()),
        "hint" => Err((
            "usage",
            "`hint` takes an anchor and the text: `hint row:code=6140 \"text\" [title=T] [avoid=A,B] [no-arrow] [no-ring] [spotlight]`.".into(),
        )),
        "spotlight" if !args.is_empty() => Ok(()),
        "spotlight" => Err(("usage", "`spotlight` takes one or more anchors.".into())),
        "clear-hints" if args.len() <= 1 => Ok(()),
        "clear-hints" => Err(("usage", "`clear-hints` takes a layer id, `all`, or nothing.".into())),
        "narrate" if args.len() == 2 || (args.len() == 1 && args[0] == "clear") => Ok(()),
        "narrate" => Err((
            "usage",
            "`narrate` takes a title and a body (two words; quote them), or `clear`.".into(),
        )),
        "walkthrough" => match (args.first().map(String::as_str), args.get(1).map(String::as_str)) {
            (Some("play"), Some(_)) if args.len() == 2 => Ok(()),
            (Some("step"), Some(n)) if args.len() == 2 && n.parse::<usize>().is_ok_and(|n| n >= 1) => Ok(()),
            (Some("next" | "prev" | "stop"), None) => Ok(()),
            (Some("record"), Some("start")) if args.len() >= 3 => Ok(()),
            (Some("record"), Some("step" | "stop")) if args.len() == 2 => Ok(()),
            _ => Err((
                "usage",
                "`walkthrough` takes `play <id>`, `step <n>`, `next`, `prev`, `stop`, `record start <id> [title=T]`, `record step` or `record stop`.".into(),
            )),
        },
        _ => Ok(()),
    }
}

/// `overlay`'s words: `set token=value…` or `clear [token…]`.
fn check_overlay_words(args: &[String]) -> Result<(), VerbError> {
    match args.first().map(String::as_str) {
        Some("set") if args.len() > 1 => match args[1..].iter().find(|a| !a.contains('=')) {
            Some(bad) => Err((
                "usage",
                format!("`overlay set` takes token=value pairs; `{bad}` has no `=`."),
            )),
            None => Ok(()),
        },
        Some("set") => Err(("usage", "`overlay set` needs token=value pairs.".into())),
        Some("clear") => Ok(()),
        _ => Err((
            "usage",
            "`overlay` takes `set token=value…` or `clear [token…]`.".into(),
        )),
    }
}

/// Words that name things the bridge never does (network, confirm, edits, quit, config): refused with a
/// message saying why, not just "unknown".
pub const FORBIDDEN: [&str; 18] = [
    "sync",
    "login",
    "refetch",
    "fetch",
    "history",
    "attachments",
    "quit",
    "confirm",
    "enter",
    "key",
    "keys",
    "edit",
    "config",
    "write",
    "copy",
    "palette",
    "apply",
    "approve",
];

/// A refused verb list: (error code, message, exit code 2).
pub type VerbError = (&'static str, String);

/// Why a word isn't a verb: `forbidden` (named, view-only rule) or `unknown_verb`.
pub fn verb_error(word: &str) -> VerbError {
    if word.eq_ignore_ascii_case("upgrade") {
        return (
            "forbidden",
            "`upgrade` is not a view verb: send it as its own request (`ownpurse ui upgrade`, kind `upgrade`)."
                .into(),
        );
    }
    if matches!(word.to_lowercase().as_str(), "apply" | "approve") {
        return (
            "forbidden",
            format!(
                "`{word}` is never an agent verb: a manifest is approved by the owner at his own terminal and applied from the CLI (the TUI shows the exact commands)."
            ),
        );
    }
    if FORBIDDEN.contains(&word.to_lowercase().as_str()) {
        (
            "forbidden",
            format!(
                "`{word}` is not allowed: the agent bridge is view-only in v0.10 (no network, confirm, edit, quit or config)."
            ),
        )
    } else {
        (
            "unknown_verb",
            format!(
                "Unknown verb `{word}`. Verbs: {}.",
                VERBS.iter().map(|(v, _)| *v).collect::<Vec<_>>().join(", ")
            ),
        )
    }
}

/// Split a flat word list (`tab 2 scope design drill`) into verbs with their arguments.
pub fn split_verbs(words: &[String]) -> Result<Vec<Vec<String>>, VerbError> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < words.len() {
        let w = &words[i];
        let Some((_, n)) = VERBS.iter().find(|(v, _)| v == w) else {
            return Err(verb_error(w));
        };
        if *n == REST {
            check_rest(w, &words[i + 1..])?;
            out.push(words[i..].to_vec());
            break;
        }
        if i + n >= words.len() + usize::from(*n == 0) && *n > 0 {
            return Err((
                "usage",
                format!(
                    "`{w}` needs {n} argument{}.",
                    if *n == 1 { "" } else { "s" }
                ),
            ));
        }
        out.push(words[i..=i + n].to_vec());
        i += n + 1;
    }
    if out.is_empty() {
        return Err(("usage", "No verb given.".into()));
    }
    Ok(out)
}

/// Check one verb (from the protocol's `verbs` array): an allowed name with the right number of arguments.
pub fn check_verb(words: &[String]) -> Result<(), VerbError> {
    let Some(w) = words.first() else {
        return Err(("usage", "An empty verb.".into()));
    };
    let Some((_, n)) = VERBS.iter().find(|(v, _)| v == w) else {
        return Err(verb_error(w));
    };
    if *n == REST {
        return check_rest(w, &words[1..]);
    }
    if words.len() != n + 1 {
        return Err((
            "usage",
            format!(
                "`{w}` takes {n} argument{}.",
                if *n == 1 { "" } else { "s" }
            ),
        ));
    }
    Ok(())
}

/// The longest socket path used as is (macOS `sun_path` holds 104 bytes including the NUL).
pub const SOCKET_PATH_MAX: usize = 100;

/// Where the bridge keeps its files.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Dirs {
    /// `$XDG_STATE_HOME/ownpurse`, else `~/.local/state/ownpurse`: the registry (`sessions.json`).
    pub state: PathBuf,
    /// `$XDG_RUNTIME_DIR/ownpurse`, else the state directory: the sockets.
    pub sockets: PathBuf,
}

impl Dirs {
    /// From an environment lookup (the process's, or a test's).
    pub fn from_env(env: &dyn Fn(&str) -> Option<String>) -> Dirs {
        let nonempty = |k: &str| env(k).filter(|v| !v.is_empty()).map(PathBuf::from);
        let home = nonempty("HOME").unwrap_or_else(|| PathBuf::from("/"));
        let state = nonempty("XDG_STATE_HOME")
            .unwrap_or_else(|| home.join(".local/state"))
            .join("ownpurse");
        let sockets = nonempty("XDG_RUNTIME_DIR")
            .map(|d| d.join("ownpurse"))
            .unwrap_or_else(|| state.clone());
        Dirs { state, sockets }
    }

    pub fn process() -> Dirs {
        Dirs::from_env(&|k| std::env::var(k).ok())
    }

    pub fn registry(&self) -> PathBuf {
        self.state.join("sessions.json")
    }

    /// `tui-<pid>.sock` in the socket directory; when that path is longer than a Unix socket address
    /// allows (104 bytes on macOS, 108 on Linux), `/tmp/ownpurse-<uid>/tui-<pid>.sock` instead (also 0700).
    pub fn socket_for(&self, pid: u32) -> PathBuf {
        let p = self.sockets.join(format!("tui-{pid}.sock"));
        if p.as_os_str().len() < SOCKET_PATH_MAX {
            return p;
        }
        // SAFETY: getuid never fails and has no preconditions.
        let uid = unsafe { libc::getuid() };
        PathBuf::from(format!("/tmp/ownpurse-{uid}")).join(format!("tui-{pid}.sock"))
    }
}

/// Create a directory (and parents) and make it 0700.
pub fn private_dir(dir: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700))
}

/// One running TUI.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    /// A short stable id (`s` + the pid in base 36), for `--session`.
    pub id: String,
    /// Optional label (`--label`, `tui.label`, agent `label`).
    pub label: Option<String>,
    /// What it shows now: scope (`all`, an alias, `group:x`), tab, theme, and whether an agent is attached.
    pub scope: String,
    pub tab: String,
    pub theme: String,
    pub agent: bool,
    pub pid: u32,
    pub socket: String,
    /// UTC, `YYYY-MM-DDTHH:MM:SSZ`.
    pub started: String,
    pub cwd: String,
    pub profile: String,
    pub version: String,
}

impl Session {
    pub fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "label": self.label,
            "scope": self.scope,
            "tab": self.tab,
            "theme": self.theme,
            "agent": self.agent,
            "pid": self.pid,
            "socket": self.socket,
            "started": self.started,
            "cwd": self.cwd,
            "profile": self.profile,
            "version": self.version,
        })
    }

    pub fn from_json(v: &Value) -> Option<Session> {
        let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
        let pid = u32::try_from(v.get("pid")?.as_u64()?).ok()?;
        Some(Session {
            id: s("id").unwrap_or_else(|| session_id(pid)),
            label: s("label"),
            scope: s("scope").unwrap_or_default(),
            tab: s("tab").unwrap_or_default(),
            theme: s("theme").unwrap_or_default(),
            agent: v.get("agent").and_then(Value::as_bool).unwrap_or(false),
            pid,
            socket: s("socket")?,
            started: s("started").unwrap_or_default(),
            cwd: s("cwd").unwrap_or_default(),
            profile: s("profile").unwrap_or_default(),
            version: s("version").unwrap_or_default(),
        })
    }

    /// Live: the process exists and its socket is still there.
    pub fn live(&self) -> bool {
        pid_alive(self.pid) && Path::new(&self.socket).exists()
    }
}

/// A session's id: `s` + the pid in base 36 (short, stable for the process's life).
pub fn session_id(pid: u32) -> String {
    let mut n = pid;
    let mut d = Vec::new();
    loop {
        d.push(std::char::from_digit(n % 36, 36).unwrap_or('0'));
        n /= 36;
        if n == 0 {
            break;
        }
    }
    d.reverse();
    format!("s{}", d.into_iter().collect::<String>())
}

/// Update this session's entry in place (scope, tab, label, theme, agent), atomically; a no-op when it's
/// missing (the registry was cleaned up).
pub fn update(dirs: &Dirs, s: &Session) -> std::io::Result<()> {
    let _l = Lock::take(dirs)?;
    let mut all = read_all(dirs);
    let Some(slot) = all.iter_mut().find(|o| o.pid == s.pid) else {
        return Ok(());
    };
    if slot == s {
        return Ok(());
    }
    *slot = s.clone();
    write_all(dirs, &all)
}

/// Which live session `--session` names: an id, a pid, a label, or `scope:<alias|group:x|all>`. Err with
/// the matches when it's ambiguous, or none.
pub fn find_session<'a>(all: &'a [Session], key: &str) -> Result<&'a Session, Vec<&'a Session>> {
    let hits: Vec<&Session> = if let Some(sc) = key.strip_prefix("scope:") {
        all.iter().filter(|s| s.scope == sc).collect()
    } else {
        let by_id: Vec<&Session> = all
            .iter()
            .filter(|s| s.id == key || s.pid.to_string() == key)
            .collect();
        if by_id.is_empty() {
            all.iter()
                .filter(|s| s.label.as_deref() == Some(key))
                .collect()
        } else {
            by_id
        }
    };
    match hits.as_slice() {
        [one] => Ok(one),
        _ => Err(hits),
    }
}

/// `kill(pid, 0)`: the process exists (EPERM also means it exists, owned by someone else).
pub fn pid_alive(pid: u32) -> bool {
    let Ok(p) = libc::pid_t::try_from(pid) else {
        return false;
    };
    if p <= 0 {
        return false;
    }
    // SAFETY: signal 0 only checks for existence and permission; nothing is delivered.
    let r = unsafe { libc::kill(p, 0) };
    r == 0 || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// `now` as `YYYY-MM-DDTHH:MM:SSZ`.
pub fn utc_stamp(secs: i64) -> String {
    let (y, m, d) = crate::dates::civil_from_days(secs.div_euclid(86400));
    let r = secs.rem_euclid(86400);
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        r / 3600,
        r % 3600 / 60,
        r % 60
    )
}

/// An advisory lock on the registry (flock on `sessions.lock`), held until dropped.
struct Lock(std::fs::File);

impl Lock {
    fn take(dirs: &Dirs) -> std::io::Result<Lock> {
        private_dir(&dirs.state)?;
        let f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(dirs.state.join("sessions.lock"))?;
        use std::os::fd::AsRawFd;
        // SAFETY: a valid open descriptor; LOCK_EX blocks until the other holder releases it.
        if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error());
        }
        Ok(Lock(f))
    }
}

impl Drop for Lock {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;
        // SAFETY: the descriptor is still open (we own the File).
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

fn read_all(dirs: &Dirs) -> Vec<Session> {
    std::fs::read_to_string(dirs.registry())
        .ok()
        .and_then(|t| serde_json::from_str::<Value>(&t).ok())
        .and_then(|v| v.get("sessions").and_then(Value::as_array).cloned())
        .unwrap_or_default()
        .iter()
        .filter_map(Session::from_json)
        .collect()
}

/// Write the registry atomically (a temp file in the same directory, then rename), mode 0600.
fn write_all(dirs: &Dirs, sessions: &[Session]) -> std::io::Result<()> {
    private_dir(&dirs.state)?;
    let doc = json!({
        "v": PROTOCOL,
        "sessions": sessions.iter().map(Session::to_json).collect::<Vec<_>>(),
    });
    let tmp = dirs
        .state
        .join(format!("sessions.json.tmp-{}", std::process::id()));
    {
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&tmp)?;
        f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
        f.write_all(serde_json::to_string_pretty(&doc)?.as_bytes())?;
        f.write_all(b"\n")?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, dirs.registry())
}

/// Add (or replace) this session's entry, pruning stale ones.
pub fn register(dirs: &Dirs, s: &Session) -> std::io::Result<()> {
    let _l = Lock::take(dirs)?;
    let mut all: Vec<Session> = read_all(dirs)
        .into_iter()
        .filter(|o| o.pid != s.pid && o.live())
        .collect();
    all.push(s.clone());
    write_all(dirs, &all)
}

/// Remove a session's entry (on exit).
pub fn unregister(dirs: &Dirs, pid: u32) -> std::io::Result<()> {
    let _l = Lock::take(dirs)?;
    let all = read_all(dirs);
    let kept: Vec<Session> = all.iter().filter(|o| o.pid != pid).cloned().collect();
    if kept.len() != all.len() {
        write_all(dirs, &kept)?;
    }
    Ok(())
}

/// The live sessions; stale entries (process gone or socket missing) are pruned from the file.
pub fn sessions(dirs: &Dirs) -> Vec<Session> {
    if !dirs.registry().exists() {
        return vec![];
    }
    let Ok(_l) = Lock::take(dirs) else {
        return read_all(dirs).into_iter().filter(Session::live).collect();
    };
    let all = read_all(dirs);
    let live: Vec<Session> = all.iter().filter(|s| s.live()).cloned().collect();
    if live.len() != all.len() {
        let _ = write_all(dirs, &live);
    }
    live
}

/// A request envelope: `{"v": 1, "kind": kind, "op": op, …fields}`.
pub fn request_doc(kind: &str, op: &str, fields: Map<String, Value>) -> Value {
    let mut m = Map::new();
    m.insert("v".into(), json!(PROTOCOL));
    m.insert("kind".into(), json!(kind));
    m.insert("op".into(), json!(op));
    m.extend(fields);
    Value::Object(m)
}

/// An error response: the CLI's error shape plus the exit code the client should use. Without a help line of
/// its own, the code's line from [`UI_ERRORS`] is used, so every refusal says what to do next.
pub fn error_doc(code: &str, exit: i32, message: impl Into<String>, help: &[&str]) -> Value {
    let help: Vec<String> = if help.is_empty() {
        vec![default_help(code).to_string()]
    } else {
        help.iter().map(|h| h.to_string()).collect()
    };
    json!({
        "v": PROTOCOL,
        "ok": false,
        "error": code,
        "message": message.into(),
        "help": help,
        "exit": exit,
    })
}

/// Every error code a `ownpurse ui …` command or an agent's classify write can answer: (code, exit, meaning,
/// what to do next). docs/TUI.md "Error codes" is this table (a test keeps them equal), and the schema of the
/// reply is docs/schema/ui-error.v1.json.
pub const UI_ERRORS: &[(&str, i32, &str, &str)] = &[
    (
        "usage",
        2,
        "The command, a verb or its argument is malformed.",
        "Check the syntax with the command's --help; values (verbs, rows, choices, groups) must exist in the current view or classify.toml.",
    ),
    (
        "unknown_verb",
        2,
        "Not a view verb.",
        "View verbs: tab, scope, asof, filter, clear-filter, select, drill, up, back, forward, basis, differs, compare, reload, open, caption, label, theme, accent, overlay, manifest, mode, viz, hint, highlight, spotlight, clear-hints, narrate, walkthrough.",
    ),
    (
        "forbidden",
        2,
        "The verb names something the bridge never does (sync, login, quit, confirm, edits, keys).",
        "Ask the user to do it in the TUI; an agent sends view verbs only.",
    ),
    (
        "not_applicable",
        2,
        "The verb means nothing on this screen (e.g. a filter on a document).",
        "Move to a screen where it applies (`up`, or another tab) and resend.",
    ),
    (
        "not_found",
        2,
        "The row, anchor, line or group is not in the current list.",
        "Read `ownpurse ui state --all-rows`; clear the filter or change the scope if the row should be there.",
    ),
    (
        "bad_caption",
        2,
        "The caption is empty, longer than 120 characters, or has control, escape or invisible characters.",
        "Send one line of plain text, at most 120 characters.",
    ),
    (
        "bad_token",
        2,
        "A theme, accent or overlay token (or its value) is unknown or invalid.",
        "Use a name the message lists; read the theme's tokens in `ownpurse ui state` (theme).",
    ),
    (
        "too_fast",
        2,
        "`ui flash` beyond the photosensitivity limits (a cycle under 334 ms, or not 1 to 5 flashes).",
        "Use --on + --off of at least 334 ms and --times from 1 to 5.",
    ),
    (
        "interrupted",
        130,
        "Ctrl-C during `ui flash`; the overlay that was there before was restored.",
        "Nothing to undo; run it again if it still makes sense.",
    ),
    (
        "too_many",
        2,
        "A classify write would touch more than 50 lines.",
        "Narrow the filter, or pass --allow-bulk when the user asked for a bulk change.",
    ),
    (
        "filter",
        2,
        "A classify filter token is malformed.",
        "Filter tokens: org:, acct:, month:, year:, <type>:<id>, conf<x, conf>=x, by:, since:, status:open|all|stale.",
    ),
    (
        "bad_state",
        2,
        "The file given to --from-state is not a saved ui state.",
        "Save one with `ownpurse --json ui state > state.json`.",
    ),
    (
        "not_available",
        2,
        "The request kind is reserved and not implemented.",
        "Use `ownpurse ui state|screen|send|wait|upgrade` or the classify write commands.",
    ),
    (
        "view_changed",
        4,
        "The view (or the classify store) moved since the revision you sent; nothing was changed.",
        "Run `ownpurse ui state` again and resend if it still makes sense.",
    ),
    (
        "view_mismatch",
        4,
        "An --expect predicate does not hold on the view; nothing was changed.",
        "Read `ownpurse ui state`, decide again, and resend with --expect for the view you now intend.",
    ),
    (
        "user_busy",
        4,
        "The user is typing, in an overlay, or has that line open in a picker; nothing was changed.",
        "Wait, then read `ownpurse ui state` again; never retry in a loop.",
    ),
    (
        "too_small",
        4,
        "The user's terminal is below the minimum size (60×20) and shows only the size warning; a change could not be announced, so nothing was changed.",
        "Ask the user to enlarge their terminal, then read `ownpurse ui state` and resend.",
    ),
    (
        "timeout",
        4,
        "`ui wait` reached its timeout before the condition held.",
        "Read the values in the reply; wait again with a longer --timeout or a different condition.",
    ),
    (
        "record_changed",
        4,
        "The saved state was taken against another record version.",
        "Pass --allow-record-drift to draw it anyway (marked as drifted), or save a new state.",
    ),
    (
        "config_changed",
        4,
        "The saved state was taken under another config.",
        "Pass --allow-config-drift to draw it anyway (marked as drifted), or save a new state.",
    ),
    (
        "classify_changed",
        4,
        "Labels were written since the state was saved.",
        "Pass --allow-classify-drift to draw it anyway (marked as drifted), or save a new state.",
    ),
    (
        "no_session",
        1,
        "No running TUI with the agent bridge.",
        "Ask the user to start `ownpurse tui` (the bridge is on unless --no-agent).",
    ),
    (
        "bridge",
        1,
        "The TUI's socket did not answer (it may have exited).",
        "Run `ownpurse ui ls`; retry once the TUI runs again.",
    ),
    (
        "closed",
        1,
        "The TUI is closing.",
        "Run `ownpurse ui ls` and talk to a running session.",
    ),
    (
        "protocol",
        2,
        "The CLI and the TUI speak different protocol versions.",
        "Install matching builds (./install.sh) and restart the TUI (or `ownpurse ui upgrade`).",
    ),
    (
        "preflight_failed",
        1,
        "The newer build could not take over the view; the old one carries on unchanged.",
        "Read the message; the user can restart the TUI instead.",
    ),
    (
        "exec_failed",
        1,
        "The newer build could not start; the old one carries on unchanged.",
        "Reinstall (./install.sh) and try `ownpurse ui upgrade` again.",
    ),
    (
        "tui_missing",
        1,
        "The ownpurse-tui binary was not found (needed for --from-state).",
        "Install both binaries with ./install.sh.",
    ),
    (
        "render",
        1,
        "Rendering a saved state failed.",
        "Save a fresh state with `ownpurse --json ui state` and retry.",
    ),
    (
        "write",
        1,
        "The output file could not be written.",
        "Check the --out path and its folder.",
    ),
    (
        "png",
        1,
        "The PNG could not be encoded.",
        "Retry with --format text or svg.",
    ),
    (
        "conflict",
        1,
        "classify.toml changed on disk since it was loaded.",
        "Reload (the TUI does it live; for the CLI just retry) and resend the edit.",
    ),
    (
        "config",
        1,
        "classify.toml is invalid, or a config edit would make it invalid.",
        "Run `ownpurse classify rules check` and fix the file, or adjust the edit.",
    ),
    (
        "store",
        1,
        "The classification store could not be read or written.",
        "Check db/classify.sqlite (permissions, disk) and retry.",
    ),
    (
        "integrity",
        1,
        "The classification store's hash chain does not verify.",
        "Stop and tell the user; do not write until the store is checked.",
    ),
    (
        "record",
        1,
        "The local record could not be read.",
        "Run `ownpurse doctor`.",
    ),
    (
        "not_set_up",
        1,
        "Classification is not set up in this TUI (no classify.toml).",
        "Create classify.toml next to ownpurse.json (see docs/CONFIG.md, classify).",
    ),
    (
        "internal",
        1,
        "A bug in ownpurse.",
        "Report it with the command and `ownpurse ui state`; rerun with OWNPURSE_DEBUG=1.",
    ),
];

/// The help line of an error code (a generic one for a code missing from the table).
pub fn default_help(code: &str) -> &'static str {
    UI_ERRORS.iter().find(|e| e.0 == code).map_or(
        "Read `ownpurse ui state` and decide again; see docs/TUI.md (Error codes).",
        |e| e.3,
    )
}

/// Send one request and read the one response (the client side; the CLI uses it).
pub fn request(socket: &Path, req: &Value, timeout: Duration) -> Result<Value, String> {
    let mut s =
        UnixStream::connect(socket).map_err(|e| format!("connect {}: {e}", socket.display()))?;
    s.set_read_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    s.set_write_timeout(Some(timeout))
        .map_err(|e| e.to_string())?;
    let mut line = req.to_string();
    line.push('\n');
    s.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
    s.flush().map_err(|e| e.to_string())?;
    let mut out = String::new();
    BufReader::new(&s)
        .read_line(&mut out)
        .map_err(|e| format!("no answer from the TUI ({e})"))?;
    if out.trim().is_empty() {
        return Err("the TUI closed the connection without answering".into());
    }
    serde_json::from_str(out.trim()).map_err(|e| format!("bad answer from the TUI ({e})"))
}

// ── intent guards (`ui send --expect PATH=VALUE`) ──

/// One `--expect` predicate: a dotted path into the `ui state` JSON, an operator and a value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Expect {
    pub text: String,
    pub path: String,
    /// `=`, `!=` or `~` (substring, case-insensitive).
    pub op: String,
    pub value: String,
}

/// Parse `tab=ledger`, `level=2`, `breadcrumb~"Trading Clearing"`, `selected.account_code!=135`.
pub fn parse_expect(text: &str) -> Result<Expect, String> {
    let (i, op) = ["!=", "~", "="]
        .iter()
        .filter_map(|op| text.find(op).map(|i| (i, *op)))
        .min_by_key(|(i, op)| (*i, std::cmp::Reverse(op.len())))
        .ok_or_else(|| format!("`{text}`: expected PATH=VALUE, PATH!=VALUE or PATH~TEXT"))?;
    let path = text[..i].trim().to_string();
    if path.is_empty()
        || !path
            .chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '-'))
    {
        return Err(format!(
            "`{text}`: the path must be dotted field names, e.g. selected.account_code"
        ));
    }
    let raw = text[i + op.len()..].trim();
    let value = raw
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .unwrap_or(raw)
        .to_string();
    Ok(Expect {
        text: text.to_string(),
        path,
        op: op.to_string(),
        value,
    })
}

/// The value at a dotted path (`rows.0.anchor.org` indexes arrays), or `Null`.
pub fn at_path<'a>(v: &'a Value, path: &str) -> &'a Value {
    static NULL: Value = Value::Null;
    let mut cur = v;
    for part in path.split('.') {
        cur = match cur {
            Value::Object(m) => m.get(part).unwrap_or(&NULL),
            Value::Array(a) => part
                .parse::<usize>()
                .ok()
                .and_then(|i| a.get(i))
                .unwrap_or(&NULL),
            _ => &NULL,
        };
    }
    cur
}

/// A JSON value as the text a predicate compares: strings as they are, arrays joined with ` › ` (a breadcrumb),
/// `null` for null, numbers and booleans as JSON.
pub fn expect_text(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Array(a) => a.iter().map(expect_text).collect::<Vec<_>>().join(" › "),
        Value::Null => "null".into(),
        other => other.to_string(),
    }
}

/// Check every predicate against a state; the failing ones (predicate, actual value).
pub fn check_expects<'a>(state: &Value, expects: &'a [Expect]) -> Vec<(&'a Expect, Value)> {
    expects
        .iter()
        .filter_map(|e| {
            let actual = at_path(state, &e.path);
            let t = expect_text(actual);
            let ok = match e.op.as_str() {
                "=" => t == e.value,
                "!=" => t != e.value,
                _ => t.to_lowercase().contains(&e.value.to_lowercase()),
            };
            (!ok).then(|| (e, actual.clone()))
        })
        .collect()
}

// ── captions (shared by the view bridge and classification writes) ──

/// Longest caption, in characters.
pub const CAPTION_MAX: usize = 120;

/// Why a caption is refused (`bad_caption`, exit 2): rejected, never stripped, so the agent knows.
pub fn caption_error(text: &str) -> Option<String> {
    if text.trim().is_empty() {
        return Some("A caption needs text (or `caption clear`).".into());
    }
    if text.chars().count() > CAPTION_MAX {
        return Some(format!(
            "A caption is at most {CAPTION_MAX} characters (this one has {}).",
            text.chars().count()
        ));
    }
    for c in text.chars() {
        let u = c as u32;
        let control = u < 0x20 || u == 0x7f || (0x80..=0x9f).contains(&u);
        // zero-width and bidi controls could make on-screen text read differently from what it is
        let invisible = matches!(u,
            0x200b..=0x200f | 0x202a..=0x202e | 0x2060..=0x2064 | 0x2066..=0x2069 | 0xfeff | 0x00ad | 0x061c | 0x180e);
        if control {
            return Some(format!(
                "A caption is one line of text: control character U+{u:04X} (escape sequences included) is not allowed."
            ));
        }
        if invisible {
            return Some(format!(
                "A caption can't contain invisible or direction-changing character U+{u:04X}."
            ));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("xab-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn verbs_split_and_refuse() {
        let w = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
        assert_eq!(
            split_verbs(&w("tab 2 scope design drill")).unwrap(),
            vec![w("tab 2"), w("scope design"), w("drill")]
        );
        assert_eq!(split_verbs(&w("sync design")).unwrap_err().0, "forbidden");
        assert_eq!(split_verbs(&w("dance")).unwrap_err().0, "unknown_verb");
        assert_eq!(
            split_verbs(&w("back forward caption x")).unwrap(),
            vec![w("back"), w("forward"), w("caption x")]
        );
        assert!(split_verbs(&w("drill up")).is_ok());
        assert_eq!(split_verbs(&w("tab")).unwrap_err().0, "usage");
        assert!(check_verb(&w("filter Hollowpine")).is_ok());
        assert_eq!(check_verb(&w("drill now")).unwrap_err().0, "usage");
        assert_eq!(check_verb(&w("quit")).unwrap_err().0, "forbidden");
    }

    #[test]
    fn session_ids_and_matching() {
        assert_eq!(session_id(36), "s10");
        let mk = |pid: u32, label: Option<&str>, scope: &str| Session {
            id: session_id(pid),
            label: label.map(String::from),
            scope: scope.into(),
            tab: "overview".into(),
            theme: "ledger-dark".into(),
            agent: false,
            pid,
            socket: String::new(),
            started: String::new(),
            cwd: String::new(),
            profile: String::new(),
            version: String::new(),
        };
        let all = vec![
            mk(100, Some("books"), "roast"),
            mk(200, None, "roast"),
            mk(300, Some("fees"), "hold"),
        ];
        assert_eq!(find_session(&all, "fees").unwrap().pid, 300);
        assert_eq!(find_session(&all, &session_id(200)).unwrap().pid, 200);
        assert_eq!(find_session(&all, "100").unwrap().pid, 100);
        assert_eq!(find_session(&all, "scope:hold").unwrap().pid, 300);
        assert_eq!(
            find_session(&all, "scope:roast").unwrap_err().len(),
            2,
            "ambiguous"
        );
        assert!(find_session(&all, "nobody").unwrap_err().is_empty());
    }

    #[test]
    fn dirs_follow_xdg_then_home() {
        let d = Dirs::from_env(&|k| match k {
            "HOME" => Some("/h".into()),
            _ => None,
        });
        assert_eq!(d.state, PathBuf::from("/h/.local/state/ownpurse"));
        assert_eq!(d.sockets, d.state);
        let d = Dirs::from_env(&|k| match k {
            "HOME" => Some("/h".into()),
            "XDG_STATE_HOME" => Some("/s".into()),
            "XDG_RUNTIME_DIR" => Some("/run/user/1".into()),
            _ => None,
        });
        assert_eq!(d.registry(), PathBuf::from("/s/ownpurse/sessions.json"));
        assert_eq!(
            d.socket_for(42),
            PathBuf::from("/run/user/1/ownpurse/tui-42.sock")
        );
    }

    #[test]
    fn registry_writes_atomically_prunes_stale_and_unregisters() {
        let root = tmp("reg");
        let dirs = Dirs {
            state: root.join("state"),
            sockets: root.join("state"),
        };
        private_dir(&dirs.state).unwrap();
        let me = std::process::id();
        let sock = dirs.socket_for(me);
        std::fs::write(&sock, "").unwrap(); // stands in for the socket file
        let s = Session {
            id: session_id(me),
            label: None,
            scope: "all".into(),
            tab: "overview".into(),
            theme: "ledger-dark".into(),
            agent: false,
            pid: me,
            socket: sock.display().to_string(),
            started: utc_stamp(0),
            cwd: "/w".into(),
            profile: "demo".into(),
            version: "0.10.0".into(),
        };
        register(&dirs, &s).unwrap();
        // a dead pid and a live pid whose socket is gone are both stale
        let dead = Session {
            pid: 999_999,
            socket: sock.display().to_string(),
            ..s.clone()
        };
        let gone = Session {
            pid: 1,
            socket: root.join("missing.sock").display().to_string(),
            ..s.clone()
        };
        write_all(&dirs, &[s.clone(), dead, gone]).unwrap();
        assert_eq!(sessions(&dirs), vec![s.clone()]);
        assert_eq!(read_all(&dirs), vec![s.clone()], "pruned on disk too");
        let mode = std::fs::metadata(dirs.registry())
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
        let dmode = std::fs::metadata(&dirs.state).unwrap().permissions().mode() & 0o777;
        assert_eq!(dmode, 0o700);
        assert!(
            std::fs::read_dir(&dirs.state)
                .unwrap()
                .flatten()
                .all(|e| !e.file_name().to_string_lossy().contains(".tmp-")),
            "no temp file left behind"
        );
        unregister(&dirs, me).unwrap();
        assert!(sessions(&dirs).is_empty());
    }
}

#[cfg(test)]
mod error_table_tests {
    use super::*;

    /// docs/TUI.md "Error codes" lists exactly UI_ERRORS (codes, exits, meanings, help).
    #[test]
    fn the_docs_table_is_the_code_table() {
        let doc =
            std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../../docs/TUI.md"))
                .unwrap();
        let start = doc.find("## Error codes").expect("an Error codes section");
        let rows: Vec<(String, i32, String, String)> = doc[start..]
            .lines()
            .filter(|l| l.starts_with("| `"))
            .map(|l| {
                let c: Vec<&str> = l.trim_matches('|').split(" | ").map(str::trim).collect();
                (
                    c[0].trim_matches('`').to_string(),
                    c[1].parse().unwrap(),
                    c[2].to_string(),
                    c[3].to_string(),
                )
            })
            .collect();
        let want: Vec<(String, i32, String, String)> = UI_ERRORS
            .iter()
            .map(|(c, e, m, h)| (c.to_string(), *e, m.to_string(), h.to_string()))
            .collect();
        assert_eq!(rows, want, "regenerate the docs table from UI_ERRORS");
        let mut codes: Vec<&str> = UI_ERRORS.iter().map(|e| e.0).collect();
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), UI_ERRORS.len(), "a code is listed twice");
    }

    /// Every error code the bridge, the TUI's agent side, the ui CLI and the classify write path construct is in
    /// the table (a source scan of the error constructors).
    #[test]
    fn every_constructed_code_is_in_the_table() {
        let root = concat!(env!("CARGO_MANIFEST_DIR"), "/..");
        let files = [
            "ownpurse-core/src/bridge.rs",
            "ownpurse-core/src/classify/request.rs",
            "ownpurse-core/src/classify/view.rs",
            "ownpurse-tui/src/bridge.rs",
            "ownpurse-tui/src/agent.rs",
            "ownpurse-tui/src/upgrade.rs",
            "ownpurse-tui/src/canonical.rs",
            "ownpurse-tui/src/classify/mod.rs",
            "ownpurse-cli/src/ui.rs",
        ];
        let pats = [
            regex::Regex::new(r#"(?s)(?:error_doc|CliError::new|CliError::runtime|ClassifyError::new)\(\s*&?"([a-z_]+)""#).unwrap(),
            regex::Regex::new(r#"(?s)Err\(\(\s*"([a-z_]+)""#).unwrap(),
            regex::Regex::new(r#"=> "([a-z_]+)",\s*$"#).unwrap(),
        ];
        let known: Vec<&str> = UI_ERRORS.iter().map(|e| e.0).collect();
        let mut missing = vec![];
        for f in files {
            let text = std::fs::read_to_string(format!("{root}/{f}")).unwrap();
            let body = text.split("#[cfg(test)]").next().unwrap_or(&text);
            for p in &pats[..2] {
                for c in p.captures_iter(body) {
                    if !known.contains(&&c[1]) {
                        missing.push(format!("{f}: {}", &c[1]));
                    }
                }
            }
        }
        missing.sort();
        missing.dedup();
        assert!(missing.is_empty(), "codes not in UI_ERRORS: {missing:?}");
        let _ = &pats[2];
    }
}
