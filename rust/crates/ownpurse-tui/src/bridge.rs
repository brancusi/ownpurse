//! The agent bridge, transport side (v0.10): the running TUI is the server. A Unix socket (mode 0600, in a
//! 0700 directory; never TCP) takes one JSON request per connection; a listener thread forwards it into
//! the loop's channel and writes back the answer the loop gives. The loop answers in `Runtime::agent`:
//! every change goes through `update` (`Msg::Agent`) like a key, then the frame is drawn as usual, so an
//! idle bridge writes nothing to the terminal and a request that changes nothing draws nothing.

use crate::agent::ViewSnap;
use crate::app::Msg;
use crate::runtime::Runtime;
use ownpurse_core::bridge::{self as wire, Dirs, Session, error_doc};
use ratatui::backend::Backend;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

/// A request from an agent, with where the answer goes.
pub struct AgentRequest {
    pub doc: Value,
    pub reply: mpsc::Sender<Value>,
    /// For an `upgrade` request: the connection itself, so the NEW process (after exec) can write the
    /// `upgraded` answer on it (the descriptor is inherited across exec).
    pub conn: Option<UnixStream>,
}

/// How long the listener waits for the loop to answer (the loop only blocks while $EDITOR runs, and the
/// listener answers `user_busy` itself then).
const ANSWER: Duration = Duration::from_secs(10);
/// `ui wait`: the default and the longest timeout.
pub const WAIT_DEFAULT_MS: u64 = 30_000;
pub const WAIT_MAX_MS: u64 = 600_000;

/// What `ui wait --until` waits for.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WaitCond {
    /// No sync or login running and no upgrade pending.
    Idle,
    /// No sync or login running.
    RunningNone,
    /// The view revision passes `n` (`>` or `>=`).
    Rev(bool, u64),
    /// The classification store revision passes `n`.
    StoreRev(bool, i64),
    /// The record version this TUI shows (`record.chain_seq`) passes `n` (a sync, here or elsewhere).
    Record(bool, i64),
}

/// A wait in flight: the condition, the deadline and where the answer goes.
pub struct Wait {
    pub until: String,
    pub cond: WaitCond,
    pub started: std::time::Instant,
    pub deadline: std::time::Instant,
    pub reply: mpsc::Sender<Value>,
}

/// Parse `idle`, `running=none`, `rev>N`, `rev>=N`, `store_rev>N`, `store_rev>=N`.
pub fn parse_until(text: &str) -> Result<WaitCond, String> {
    let t = text.trim();
    let num = |s: &str| {
        s.trim()
            .parse::<i64>()
            .map_err(|_| format!("`{t}`: not a number"))
    };
    if t == "idle" {
        return Ok(WaitCond::Idle);
    }
    if t == "running=none" {
        return Ok(WaitCond::RunningNone);
    }
    for key in ["store_rev", "record", "rev"] {
        if let Some(rest) = t.strip_prefix(key) {
            let (ge, n) = if let Some(n) = rest.strip_prefix(">=") {
                (true, num(n)?)
            } else if let Some(n) = rest.strip_prefix('>') {
                (false, num(n)?)
            } else {
                continue;
            };
            return Ok(match key {
                "store_rev" => WaitCond::StoreRev(ge, n),
                "record" => WaitCond::Record(ge, n),
                _ => WaitCond::Rev(ge, n.max(0) as u64),
            });
        }
    }
    Err(format!(
        "`{t}`: expected idle, running=none, rev>N, store_rev>N or record>N (or >=N)"
    ))
}

/// Longest request line accepted.
const MAX_REQUEST: u64 = 1 << 20;

/// What the panic hook and signal exit must clean up (each socket file and its registry entry).
static LIVE: std::sync::Mutex<Vec<(Dirs, PathBuf, u32)>> = std::sync::Mutex::new(Vec::new());

fn remove(dirs: &Dirs, path: &PathBuf, pid: u32) {
    let _ = std::fs::remove_file(path);
    let _ = wire::unregister(dirs, pid);
}

/// Remove this process's sockets and registry entries (idempotent; safe from the panic hook).
pub fn cleanup() {
    let taken: Vec<(Dirs, PathBuf, u32)> = LIVE
        .lock()
        .map(|mut g| std::mem::take(&mut *g))
        .unwrap_or_default();
    for (dirs, path, pid) in taken {
        remove(&dirs, &path, pid);
    }
}

/// The listening socket; dropping it removes the socket file and the registry entry.
pub struct Server {
    pub path: PathBuf,
    dirs: Dirs,
    /// The registry entry as last written (`sync` rewrites it when what `ui ls` shows changes).
    session: std::cell::RefCell<Session>,
}

impl Drop for Server {
    fn drop(&mut self) {
        let mine = LIVE.lock().ok().and_then(|mut g| {
            let i = g.iter().position(|(_, p, _)| *p == self.path)?;
            Some(g.remove(i))
        });
        if let Some((dirs, path, pid)) = mine {
            remove(&dirs, &path, pid);
        }
    }
}

/// Whether the bridge opens: not with `--no-agent`, not with `tui.agent: false`.
pub fn wanted(no_agent: bool, source: &crate::model::Source) -> bool {
    !no_agent && source.agent
}

impl Server {
    /// When this session started (kept across an in-place upgrade).
    pub fn started(&self) -> String {
        self.session.borrow().started.clone()
    }

    /// Keep the registry entry current: scope, tab, label, theme and whether an agent is attached (what
    /// `ui ls` shows). Writes only when one of them changed.
    pub fn sync(&self, app: &crate::app::App) {
        let mut s = self.session.borrow().clone();
        s.scope = app.scope_label(&app.ui.scope);
        s.tab = app.ui.tab.label().to_lowercase();
        s.label = app.label.clone();
        s.theme = app.ui.theme.name().to_string();
        s.agent = app.agent.connected;
        if s != *self.session.borrow() {
            let _ = wire::update(&self.dirs, &s);
            *self.session.borrow_mut() = s;
        }
    }

    /// Bind `tui-<pid>.sock` (0600) in the 0700 socket directory, register the session, and start the
    /// listener. `paused` says the user is in $EDITOR (the loop can't answer then); `sink` forwards a request
    /// to the loop and returns false once the loop is gone.
    pub fn start(
        dirs: Dirs,
        profile: &str,
        paused: Box<dyn Fn() -> bool + Send>,
        sink: Box<dyn Fn(AgentRequest) -> bool + Send>,
    ) -> std::io::Result<Server> {
        let pid = std::process::id();
        let path = dirs.socket_for(pid);
        if let Some(dir) = path.parent() {
            wire::private_dir(dir)?;
        }
        let _ = std::fs::remove_file(&path); // a stale file from a crashed run with the same pid
        let listener = UnixListener::bind(&path)?;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |d| d.as_secs() as i64);
        let session = Session {
            id: wire::session_id(pid),
            label: None,
            scope: "all".into(),
            tab: "overview".into(),
            theme: String::new(),
            agent: false,
            pid,
            socket: path.display().to_string(),
            // an upgraded session keeps the start time it had (the same pid, re-registered after exec)
            started: std::env::var(crate::upgrade::STARTED_ENV)
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| wire::utc_stamp(now)),
            cwd: std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
            profile: profile.to_string(),
            version: crate::app::build_version(),
        };
        wire::register(&dirs, &session)?;
        let dirs_keep = dirs.clone();
        if let Ok(mut g) = LIVE.lock() {
            g.push((dirs, path.clone(), pid));
        }
        // one thread per connection: a slow, silent or waiting client (`ui wait`) never holds up the others
        let paused = std::sync::Arc::new(std::sync::Mutex::new(paused));
        let sink = std::sync::Arc::new(std::sync::Mutex::new(sink));
        let gone = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                if gone.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let Ok(stream) = stream else { continue };
                let (paused, sink, gone) = (paused.clone(), sink.clone(), gone.clone());
                std::thread::spawn(move || {
                    let is_paused = || paused.lock().map(|f| f()).unwrap_or(false);
                    let send = |r: AgentRequest| sink.lock().map(|f| f(r)).unwrap_or(false);
                    if !serve(stream, &is_paused, &send) {
                        gone.store(true, std::sync::atomic::Ordering::SeqCst);
                    }
                });
            }
        });
        Ok(Server {
            path,
            dirs: dirs_keep,
            session: std::cell::RefCell::new(session),
        })
    }
}

/// One connection: read one request line, answer it, close. Returns false when the loop is gone.
fn serve(
    mut stream: UnixStream,
    paused: &dyn Fn() -> bool,
    sink: &dyn Fn(AgentRequest) -> bool,
) -> bool {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
    let mut line = String::new();
    let read = BufReader::new((&stream).take(MAX_REQUEST)).read_line(&mut line);
    let mut alive = true;
    let answer = match read {
        Err(e) => error_doc(
            "usage",
            2,
            format!("Could not read the request ({e})."),
            &[],
        ),
        Ok(_) => match precheck(&line) {
            Err(e) => e,
            Ok(doc)
                if paused()
                    && (doc["op"] == "send"
                        || doc["kind"] == "upgrade"
                        || doc["kind"] == "classify") =>
            {
                error_doc(
                    "user_busy",
                    4,
                    "The user is in an editor. Nothing was changed.",
                    &[
                        "Run `ownpurse ui state` again when they are back, and resend if it still makes sense.",
                    ],
                )
            }
            Ok(doc) => {
                let is_wait = doc["op"] == "wait";
                let wait_ms = doc["timeout_ms"]
                    .as_u64()
                    .unwrap_or(WAIT_DEFAULT_MS)
                    .min(WAIT_MAX_MS);
                let (tx, rx) = mpsc::channel();
                let conn = (doc["kind"] == "upgrade")
                    .then(|| stream.try_clone().ok())
                    .flatten();
                if sink(AgentRequest {
                    doc,
                    reply: tx,
                    conn,
                }) {
                    let limit = if is_wait {
                        Duration::from_millis(wait_ms.saturating_add(2_000))
                    } else {
                        ANSWER
                    };
                    rx.recv_timeout(limit).unwrap_or_else(|_| {
                        error_doc("bridge", 1, "The TUI did not answer in time.", &[])
                    })
                } else {
                    alive = false;
                    error_doc("closed", 1, "The TUI is closing.", &[])
                }
            }
        },
    };
    // core's canonical JSON (sorted keys, exact digits): the state struct's one wire form
    let mut out = ownpurse_core::canon::canonical(&answer);
    out.push('\n');
    let _ = stream.write_all(out.as_bytes());
    let _ = stream.flush();
    alive
}

/// The envelope rules, checked before the loop sees anything: JSON, `v: 1`, a known `kind` (only `view`
/// is implemented; `annotate` and `task` are reserved), a known `op`.
pub fn precheck(line: &str) -> Result<Value, Value> {
    let doc: Value = serde_json::from_str(line.trim())
        .map_err(|e| error_doc("usage", 2, format!("The request is not JSON ({e})."), &[]))?;
    if doc["v"] != wire::PROTOCOL {
        return Err(error_doc(
            "protocol",
            2,
            format!(
                "Protocol version {} is not supported (this TUI speaks v{}).",
                doc["v"],
                wire::PROTOCOL
            ),
            &[],
        ));
    }
    match doc["kind"].as_str().unwrap_or("view") {
        "view" => {}
        // agent writes to the classification layer (v0.11.0, spec classify §5a): its own ops
        "classify" => {
            let ops = ownpurse_core::classify::request::OPS;
            return match doc["op"].as_str() {
                Some(op) if ops.contains(&op) => Ok(doc),
                other => Err(error_doc(
                    "usage",
                    2,
                    format!(
                        "Unknown op `{}` for kind `classify` ({}).",
                        other.unwrap_or(""),
                        ops.join(", ")
                    ),
                    &[],
                )),
            };
        }
        // the one non-view action (v0.10.3): its own kind and op, nothing else
        "upgrade" => {
            return match doc["op"].as_str() {
                Some("upgrade") => Ok(doc),
                other => Err(error_doc(
                    "usage",
                    2,
                    format!(
                        "Unknown op `{}` for kind `upgrade` (upgrade).",
                        other.unwrap_or("")
                    ),
                    &[],
                )),
            };
        }
        k @ ("annotate" | "task") => {
            return Err(error_doc(
                "not_available",
                2,
                format!("`{k}` requests are not available in v0.10 (view only)."),
                &[],
            ));
        }
        k => {
            return Err(error_doc(
                "usage",
                2,
                format!("Unknown request kind `{k}` (view, upgrade, classify)."),
                &[],
            ));
        }
    }
    match doc["op"].as_str() {
        Some("state" | "screen" | "send" | "wait") => Ok(doc),
        other => Err(error_doc(
            "usage",
            2,
            format!(
                "Unknown op `{}` (state, screen, send, wait).",
                other.unwrap_or("")
            ),
            &[],
        )),
    }
}

fn ok(rev: u64) -> serde_json::Map<String, Value> {
    let mut m = serde_json::Map::new();
    m.insert("v".into(), json!(wire::PROTOCOL));
    m.insert("ok".into(), json!(true));
    m.insert("rev".into(), json!(rev));
    m
}

fn with_rev(mut e: Value, rev: u64) -> Value {
    e["rev"] = json!(rev);
    e
}

/// The verbs of a send: `verbs: [["tab", "2"], …]`, or `msg` (`{"verb": "tab", "args": ["2"]}` or a list of
/// those; `ui send --json`).
fn verbs_of(doc: &Value) -> Result<Vec<Vec<String>>, Value> {
    let strs = |v: &Value| -> Option<Vec<String>> {
        v.as_array()?
            .iter()
            .map(|w| match w {
                Value::String(s) => Some(s.clone()),
                Value::Number(n) => Some(n.to_string()),
                _ => None,
            })
            .collect()
    };
    let list: Vec<Vec<String>> = if let Some(v) = doc.get("verbs") {
        v.as_array()
            .ok_or_else(|| error_doc("usage", 2, "`verbs` must be a list of verbs.", &[]))?
            .iter()
            .map(|x| {
                strs(x).ok_or_else(|| error_doc("usage", 2, "A verb is a list of strings.", &[]))
            })
            .collect::<Result<_, _>>()?
    } else if let Some(m) = doc.get("msg") {
        let one =
            |x: &Value| -> Result<Vec<String>, Value> {
                let verb = x["verb"].as_str().ok_or_else(|| {
                    error_doc("usage", 2, "A message needs `verb` (and `args`).", &[])
                })?;
                let mut w = vec![verb.to_string()];
                if let Some(a) = x.get("args") {
                    w.extend(strs(a).ok_or_else(|| {
                        error_doc("usage", 2, "`args` is a list of strings.", &[])
                    })?);
                }
                Ok(w)
            };
        match m {
            Value::Array(a) => a.iter().map(one).collect::<Result<_, _>>()?,
            other => vec![one(other)?],
        }
    } else {
        return Err(error_doc("usage", 2, "A send needs `verbs`.", &[]));
    };
    if list.is_empty() {
        return Err(error_doc("usage", 2, "No verb given.", &[]));
    }
    for w in &list {
        wire::check_verb(w).map_err(|(code, msg)| error_doc(code, 2, msg, &[]))?;
    }
    Ok(list)
}

impl<B: Backend> Runtime<B> {
    /// The frame as drawn now: the same pure render at the TUI's size and theme (draw never mutates).
    pub fn current_frame(&self) -> Buffer {
        let (w, h) = self.app.ui.size;
        let area = Rect::new(0, 0, w, h);
        let mut buf = Buffer::empty(area);
        let _ = crate::ui::render(&self.app, &mut buf, area);
        buf
    }

    /// The frame's text and the state without its revision: what "the view changed" means.
    pub fn view_print(&self) -> (String, Value) {
        let mut st = self.app.state_json(0);
        if let Some(m) = st.as_object_mut() {
            m.remove("rev");
            m.remove("canonical"); // carries the revision and the agent's own log
        }
        (crate::snapshot::text(&self.current_frame()), st)
    }

    fn screen_doc(&self, format: &str) -> Result<Value, Value> {
        let buf = self.current_frame();
        let th = &self.app.ui.theme;
        let body = match format {
            "text" => json!(crate::snapshot::text(&buf)),
            "ansi" => json!(crate::render::ansi(&buf)),
            "html" => json!(crate::render::html(&buf, th, "ownpurse")),
            "svg" => json!(crate::render::svg(&buf, th)),
            "grid" => crate::render::grid(&buf, th),
            other => {
                return Err(error_doc(
                    "usage",
                    2,
                    format!("Unknown screen format `{other}` (text, ansi, html, svg, png)."),
                    &[],
                ));
            }
        };
        Ok(
            json!({"format": format, "rev": self.app.rev, "cols": buf.area.width, "rows": buf.area.height, "body": body}),
        )
    }

    /// Answer one agent request (from the loop). Draws pending changes first, so a screenshot is the frame
    /// the user sees for the revision it reports.
    pub fn agent(&mut self, doc: &Value) -> Value {
        let doc = match precheck(&doc.to_string()) {
            Ok(d) => d,
            Err(e) => return with_rev(e, self.app.rev),
        };
        // `rows` (all | a..b): the whole derived list or a slice, read-only: no mark, no log, no frame
        if doc["op"] == "state"
            && let Some(r) = doc.get("rows").filter(|r| !r.is_null())
        {
            let Some(which) = r.as_str().and_then(crate::state::Rows::parse) else {
                return with_rev(
                    error_doc(
                        "usage",
                        2,
                        format!("`rows` is `all` or `a..b` (1-based), got {r}."),
                        &[],
                    ),
                    self.app.rev,
                );
            };
            let mut m = ok(self.app.rev);
            m.insert(
                "state".into(),
                self.app.state_json_rows(self.app.rev, which),
            );
            return Value::Object(m);
        }
        self.app.agent_seen();
        // the one non-view action (v0.10.3): the loop hands over after an `upgrading` answer
        if doc["kind"] == "upgrade" {
            let from =
                crate::app::full_version(&self.app.version_label, self.app.build_label.as_deref());
            let ask = self.upgrade_ask();
            self.app.agent_log(&match &ask {
                crate::upgrade::Ask::UpToDate => "upgrade · up to date".to_string(),
                crate::upgrade::Ask::Deferred(to) => format!("upgrade to v{to} · deferred"),
                crate::upgrade::Ask::Go(to) => format!("upgrade to v{to}"),
            });
            let _ = self.frame();
            return crate::upgrade::ask_doc(&ask, &from, self.app.rev);
        }
        // an agent's classify write: through update (user_busy checked there), applied by the loop
        if doc["kind"] == "classify" {
            return self.agent_classify(&doc);
        }
        self.check_update(); // an agent request is interaction too
        let _ = self.frame();
        match doc["op"].as_str().unwrap_or("") {
            "state" => {
                self.app.agent_log("read state");
                let _ = self.frame();
                let mut m = ok(self.app.rev);
                m.insert("state".into(), self.app.state_json(self.app.rev));
                Value::Object(m)
            }
            "screen" => {
                let format = doc["format"].as_str().unwrap_or("text");
                let shot = match self.screen_doc(format) {
                    Ok(s) => s,
                    Err(e) => return with_rev(e, self.app.rev),
                };
                let shown = if format == "grid" { "png" } else { format };
                self.app.agent_log(&format!("screenshot {shown}"));
                self.app.agent_acted();
                let rev = self.app.rev;
                let _ = self.frame();
                let mut m = ok(rev);
                m.insert("screen".into(), shot);
                Value::Object(m)
            }
            _ => self.agent_send(&doc),
        }
    }

    /// Start a `ui wait`: answered at once when the condition holds (or the request is bad), else kept by the
    /// loop and answered when an event makes it hold, or at its deadline.
    pub fn wait_start(&mut self, doc: &Value, reply: mpsc::Sender<Value>) -> Option<Wait> {
        let until = doc["until"].as_str().unwrap_or("").to_string();
        let cond = match parse_until(&until) {
            Ok(c) => c,
            Err(m) => {
                let _ = reply.send(with_rev(
                    error_doc("usage", 2, m, &["Conditions: idle, running=none, rev>N, rev>=N, store_rev>N, store_rev>=N."]),
                    self.app.rev,
                ));
                return None;
            }
        };
        self.app.agent_seen();
        self.app.agent_log(&format!("wait until {until}"));
        let _ = self.frame();
        let ms = doc["timeout_ms"]
            .as_u64()
            .unwrap_or(WAIT_DEFAULT_MS)
            .min(WAIT_MAX_MS);
        let now = std::time::Instant::now();
        let w = Wait {
            until,
            cond,
            started: now,
            deadline: now + Duration::from_millis(ms),
            reply,
        };
        if self.wait_holds(&w.cond) {
            let _ = w.reply.send(self.wait_answer(&w, true));
            return None;
        }
        Some(w)
    }

    fn store_rev(&self) -> i64 {
        self.app.classify.as_ref().map_or(0, |d| d.provider.rev())
    }

    pub fn wait_holds(&self, c: &WaitCond) -> bool {
        let running = self.app.ui.run.is_some();
        match c {
            WaitCond::Idle => !running && !self.upgrade_due,
            WaitCond::RunningNone => !running,
            WaitCond::Rev(ge, n) => {
                if *ge {
                    self.app.rev >= *n
                } else {
                    self.app.rev > *n
                }
            }
            WaitCond::StoreRev(ge, n) => {
                let s = self.store_rev();
                if *ge { s >= *n } else { s > *n }
            }
            WaitCond::Record(ge, n) => {
                let r = self.app.model.record_version;
                if *ge { r >= *n } else { r > *n }
            }
        }
    }

    /// The answer to a wait: the summary when it held, else `timeout` (exit 4) with the current values.
    pub fn wait_answer(&self, w: &Wait, held: bool) -> Value {
        let waited = w.started.elapsed().as_millis() as u64;
        let actual = json!({"rev": self.app.rev, "store_rev": self.store_rev(),
                            "record": self.app.model.record_version,
                            "running": self.app.ui.run.is_some(), "idle": self.wait_holds(&WaitCond::Idle)});
        if held {
            let mut m = ok(self.app.rev);
            m.insert("until".into(), json!(w.until));
            m.insert("held".into(), json!(true));
            m.insert("waited_ms".into(), json!(waited));
            m.insert("actual".into(), actual);
            m.insert("summary".into(), self.app.state_summary(self.app.rev));
            Value::Object(m)
        } else {
            let mut d = error_doc(
                "timeout",
                4,
                format!("`{}` did not hold within {} ms.", w.until, waited),
                &[
                    "Read the values in `actual`; wait again with a longer --timeout or a different condition.",
                ],
            );
            d["actual"] = actual;
            with_rev(d, self.app.rev)
        }
    }

    /// While the terminal is below the minimum size the screen shows only the size warning: a change the
    /// user can't see announced is refused (`too_small`, exit 4); reads still answer.
    fn too_small(&mut self, what: &str) -> Option<Value> {
        let (w, h) = self.app.ui.size;
        if w >= crate::ui::MIN_W && h >= crate::ui::MIN_H {
            return None;
        }
        self.app
            .agent_log(&format!("{what} · refused, terminal too small"));
        let _ = self.frame();
        let mut e = error_doc(
            "too_small",
            4,
            format!(
                "The user's terminal is too small to show the change ({w}×{h}; it needs at least {}×{}). Nothing was changed.",
                crate::ui::MIN_W,
                crate::ui::MIN_H
            ),
            &["Ask the user to enlarge their terminal, then read `ownpurse ui state` and resend."],
        );
        e["actual"] = json!({"cols": w, "rows": h});
        e["expected"] = json!({"cols": crate::ui::MIN_W, "rows": crate::ui::MIN_H});
        Some(e)
    }

    fn agent_classify(&mut self, doc: &Value) -> Value {
        use ownpurse_core::classify::request::{ClassifyRequest, error_response};
        let rev = self.app.rev;
        let req = match ClassifyRequest::from_json(doc) {
            Ok(r) => r,
            Err(e) => return with_rev(error_response(&e), rev),
        };
        self.app.agent_client = Some(
            req.client
                .chars()
                .filter(|c| !c.is_control())
                .take(40)
                .collect(),
        );
        // a write the user can't see announced is refused like a view change (a config dry run still answers)
        let preview = matches!(
            req.op,
            ownpurse_core::classify::request::Op::Config { dry_run: true, .. }
        );
        if !preview && let Some(e) = self.too_small(&format!("classify {}", req.op.name())) {
            return with_rev(e, rev);
        }
        self.app.agent.classify_reply = None;
        self.send(Msg::AgentClassify(Box::new(req)));
        let _ = self.frame();
        let mut reply =
            self.app.agent.classify_reply.take().unwrap_or_else(|| {
                error_doc("internal", 1, "The TUI did not apply the request.", &[])
            });
        if reply["ok"] == true {
            reply["summary_view"] = self.app.state_summary(self.app.rev);
        }
        with_rev(reply, self.app.rev)
    }

    fn agent_send(&mut self, doc: &Value) -> Value {
        let rev = self.app.rev;
        let words = match verbs_of(doc) {
            Ok(w) => w,
            Err(e) => return with_rev(e, rev),
        };
        let what = words
            .iter()
            .map(|w| match w.len() {
                1 => w[0].clone(),
                _ => format!("{}={}", w[0], w[1..].join(",")),
            })
            .collect::<Vec<_>>()
            .join(" ");
        // the user's input wins: refused, never queued (cosmetic theme verbs excepted: they touch no input)
        let cosmetic = words.iter().all(|w| {
            w.first()
                .is_some_and(|v| wire::COSMETIC.contains(&v.as_str()))
        });
        if !cosmetic && let Some(e) = self.too_small(&format!("send {what}")) {
            return with_rev(e, rev);
        }
        if let Some(busy) = self.app.agent_busy().filter(|_| !cosmetic) {
            self.app.agent_skipped();
            self.app
                .agent_log(&format!("send {what} · refused, user busy"));
            let _ = self.frame();
            return with_rev(
                error_doc(
                    "user_busy",
                    4,
                    format!("The user is {busy}. Nothing was changed."),
                    &[
                        "Run `ownpurse ui state` again when they are done, and resend if it still makes sense.",
                    ],
                ),
                rev,
            );
        }
        let expect = doc
            .get("expect_rev")
            .and_then(Value::as_u64)
            .or_else(|| doc.get("expect_seq").and_then(Value::as_u64));
        if let Some(n) = expect.filter(|n| *n != rev) {
            self.app.agent_skipped();
            self.app
                .agent_log(&format!("send {what} · refused, view changed"));
            let _ = self.frame();
            return with_rev(
                error_doc(
                    "view_changed",
                    4,
                    format!(
                        "The user changed the view since you read it (rev {n} → {rev}). Nothing was changed."
                    ),
                    &["Run `ownpurse ui state` again and resend if it still makes sense."],
                ),
                rev,
            );
        }
        // intent guards: every `--expect` predicate must hold on the view as it is now (atomic with the send)
        let mut expects = Vec::new();
        for e in doc
            .get("expect")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            match wire::parse_expect(e.as_str().unwrap_or("")) {
                Ok(x) => expects.push(x),
                Err(m) => {
                    return with_rev(
                        error_doc(
                            "usage",
                            2,
                            format!("{m} Nothing was changed."),
                            &[
                                "Predicates are PATH=VALUE, PATH!=VALUE or PATH~TEXT over `ownpurse ui state --json` fields, e.g. --expect tab=ledger --expect selected.account_code=1010.",
                            ],
                        ),
                        rev,
                    );
                }
            }
        }
        if !expects.is_empty() {
            let now = self.app.state_json(rev);
            let failing = wire::check_expects(&now, &expects);
            if !failing.is_empty() {
                self.app.agent_skipped();
                self.app
                    .agent_log(&format!("send {what} · refused, view mismatch"));
                let _ = self.frame();
                let mut d = error_doc(
                    "view_mismatch",
                    4,
                    format!(
                        "The view does not match what you expected ({}). Nothing was changed.",
                        failing
                            .iter()
                            .map(|(e, _)| e.text.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                    &[
                        "Run `ownpurse ui state` and decide again; resend with --expect for the view you now intend.",
                    ],
                );
                d["expected"] = Value::Array(
                    failing
                        .iter()
                        .map(|(e, _)| json!({"path": e.path, "op": e.op, "value": e.value}))
                        .collect(),
                );
                d["actual"] = Value::Array(
                    failing
                        .iter()
                        .map(|(e, a)| json!({"path": e.path, "value": a, "text": wire::expect_text(a)}))
                        .collect(),
                );
                return with_rev(d, rev);
            }
        }
        let mut verbs = Vec::new();
        for w in &words {
            match self.app.parse_verb(w) {
                Ok(v) => verbs.push(v),
                Err((code, msg)) => {
                    return with_rev(
                        error_doc(&code, 2, format!("{msg} Nothing was changed."), &[]),
                        rev,
                    );
                }
            }
        }
        self.app.agent_client = doc
            .get("client")
            .and_then(Value::as_str)
            .map(|c| c.chars().filter(|c| !c.is_control()).take(40).collect());
        let before = ViewSnap::of(&self.app.ui);
        self.app.agent_no_history = None;
        // a reload claims a change only when the view shows one (same record → unchanged)
        let reloads = verbs
            .iter()
            .any(|v| matches!(v, crate::agent::Verb::Reload));
        let seen = reloads.then(|| self.view_print());
        self.app.layers_touched.clear();
        self.app.walk_saved = None;
        // one send = one update = one revision; atomic (any refusal applies nothing)
        self.send(Msg::AgentSend(verbs));
        let _ = self.frame();
        let mut changes: Vec<String> = Vec::new();
        let mut failed: Option<Value> = None;
        match self.app.agent.batch.take() {
            Some(Ok(c)) => changes = c,
            Some(Err((code, msg))) => {
                let exit = if [
                    "usage",
                    "not_applicable",
                    "bad_caption",
                    "bad_token",
                    "not_found",
                ]
                .contains(&code.as_str())
                {
                    2
                } else {
                    1
                };
                // say which verb failed and that the ones before it were rolled back (atomic send)
                let at = self.app.agent.failed.take();
                let i = at.as_ref().map_or(0, |(i, _)| *i);
                let name = |k: usize| words.get(k).map(|w| w.join(" ")).unwrap_or_default();
                let msg = if words.len() > 1 {
                    let mut m = format!("verb {} ({}) failed: {msg}", i + 1, name(i));
                    if i > 0 {
                        let before: Vec<String> =
                            (0..i).map(|k| format!("{} ({})", k + 1, name(k))).collect();
                        m.push_str(&format!(
                            " Verb{} {} not applied (the send is atomic). Nothing was changed.",
                            if i > 1 { "s" } else { "" },
                            before.join(", ")
                        ));
                    } else {
                        m.push_str(" Nothing was changed.");
                    }
                    m
                } else {
                    format!("{msg} Nothing was changed.")
                };
                let help: Vec<String> = at.and_then(|(_, h)| h).into_iter().collect();
                let help: Vec<&str> = help.iter().map(String::as_str).collect();
                failed = Some(error_doc(&code, exit, msg, &help));
            }
            None => {}
        }
        if seen.is_some_and(|b| b == self.view_print()) {
            changes.retain(|c| c != "reloaded the record");
        }
        if !changes.is_empty() {
            self.app.agent_announce(changes.clone(), before);
        }
        self.app.agent_log(&format!("send {what}"));
        let _ = self.frame();
        if let Some(e) = failed {
            return with_rev(e, self.app.rev);
        }
        let mut m = ok(self.app.rev);
        m.insert("changed".into(), json!(changes));
        if cosmetic {
            let w = self.app.contrast_warnings();
            if !w.is_empty() {
                m.insert("contrast_warnings".into(), json!(w));
            }
        }
        let no_history = self.app.agent_no_history.take();
        // per verb: what it did, or that it changed nothing (an agent never has to infer it)
        let outcomes = std::mem::take(&mut self.app.agent.effects);
        let effects: Vec<Value> = words
            .iter()
            .enumerate()
            .map(|(i, w)| {
                let verb = w.join(" ");
                match outcomes.get(i).cloned().flatten() {
                    Some(what) if !what.is_empty() => {
                        json!({"verb": verb, "changed": true, "what": what})
                    }
                    Some(_) => json!({"verb": verb, "changed": true, "what": "set"}),
                    None => {
                        let why = match (w.first().map(String::as_str), &no_history) {
                            (Some("back" | "forward"), Some(t)) => t.to_string(),
                            _ => "no change (already so, or nothing to act on here)".to_string(),
                        };
                        json!({"verb": verb, "changed": false, "what": why})
                    }
                }
            })
            .collect();
        m.insert("effects".into(), Value::Array(effects));
        // where each layer this send placed landed (resolved, off screen above/below, or not found and why)
        let touched = std::mem::take(&mut self.app.layers_touched);
        if !touched.is_empty() {
            let landed: Vec<Value> = touched
                .iter()
                .filter_map(|id| self.app.ui.layers.get(id))
                .map(|l| {
                    crate::overlay::landed(self.app.hits.plan.as_ref(), l, &self.app.hits.anchors)
                })
                .collect();
            m.insert("layers".into(), Value::Array(landed));
        }
        match self.app.walk_saved.take() {
            Some(Ok(p)) => {
                m.insert("saved".into(), json!(p.display().to_string()));
            }
            Some(Err(e)) => {
                m.insert("saved".into(), Value::Null);
                m.insert("save_error".into(), json!(e));
            }
            None => {}
        }
        m.insert("summary".into(), self.app.state_summary(self.app.rev));
        if let Some(text) = no_history {
            m.insert("note".into(), json!("no_history"));
            m.insert("message".into(), json!(text));
        }
        m.insert("state".into(), self.app.state_json(self.app.rev));
        if let Some(f) = doc.get("screen").and_then(Value::as_str) {
            match self.screen_doc(f) {
                Ok(s) => {
                    m.insert("screen".into(), s);
                }
                Err(e) => return with_rev(e, self.app.rev),
            }
        }
        Value::Object(m)
    }
}
