//! `ownpurse ui ls|state|screen|send|upgrade`: the agent bridge's client (view only, plus the one non-view
//! action, `upgrade`, v0.10.3). A running
//! `ownpurse tui` is the server on a local Unix socket (0600; never TCP). Reads return the view state as
//! data and the current frame; sends change the view through the TUI's own `update` path, refused when
//! the user has moved since the agent last read (`--expect-rev`) or is typing in an overlay.
//! Exit codes: 0 ok, 1 runtime (no session, no match), 2 usage (unknown or forbidden verb), 4 refused
//! (`view_changed`, `user_busy`).

use crate::app::{CliError, Global, Res};
use crate::obj;
use crate::out::Out;
use ownpurse_core::bridge::{self as wire, Dirs, Session};
use ownpurse_core::raster;
use serde_json::{Map, Value, json};
use std::path::PathBuf;
use std::time::Duration;

pub(crate) const TIMEOUT: Duration = Duration::from_secs(15);

pub const FORMATS: [&str; 5] = ["text", "ansi", "html", "svg", "png"];

fn dirs() -> Dirs {
    Dirs::process()
}

fn listing(s: &[Session]) -> String {
    s.iter()
        .map(|x| {
            format!(
                "{} pid {} ({}{}, scope {})",
                x.id,
                x.pid,
                x.profile,
                x.label
                    .as_deref()
                    .map(|l| format!(", “{l}”"))
                    .unwrap_or_default(),
                x.scope
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

/// The session to talk to: `--session <id|pid|label|scope:alias>`, else the only live one.
fn pick(session: Option<&str>) -> Res<Session> {
    let all = wire::sessions(&dirs());
    match session {
        Some(key) => wire::find_session(&all, key).cloned().map_err(|hits| {
            if hits.is_empty() {
                CliError::usage(format!("No live TUI session matches `{key}`.")).hint(if all.is_empty() {
                    "No sessions: start `ownpurse tui`.".to_string()
                } else {
                    format!("Live sessions: {}.", listing(&all))
                })
            } else {
                let owned: Vec<Session> = hits.into_iter().cloned().collect();
                CliError::usage(format!(
                    "`{key}` matches {} sessions: pass an id or a label.",
                    owned.len()
                ))
                .hint(format!("Matches: {}.", listing(&owned)))
            }
        }),
        None => match all.len() {
            0 => Err(CliError::runtime(
                "no_session",
                "No running ownpurse TUI with the agent bridge.",
            )
            .hint("Start `ownpurse tui` (the bridge is on unless `--no-agent` or `tui.agent: false`).")),
            1 => Ok(all[0].clone()),
            _ => Err(CliError::usage(format!(
                "{} TUI sessions are running: pass --session <id|label|scope:alias>.",
                all.len()
            ))
            .hint(format!("Live sessions: {}.", listing(&all)))),
        },
    }
}

/// Send one request; a refusal comes back as the CLI's error (its code, message, help and exit code).
fn call(s: &Session, op: &str, fields: Map<String, Value>) -> Res<Value> {
    call_within(s, op, fields, TIMEOUT)
}

fn call_within(s: &Session, op: &str, fields: Map<String, Value>, limit: Duration) -> Res<Value> {
    let req = wire::request_doc("view", op, fields);
    let resp = wire::request(std::path::Path::new(&s.socket), &req, limit).map_err(|e| {
        CliError::runtime("bridge", format!("Could not reach TUI {}: {e}", s.pid))
            .hint("Check `ownpurse ui ls`; the TUI may have exited.")
    })?;
    if resp["ok"] == true {
        return Ok(resp);
    }
    let mut e = CliError::new(
        resp["error"].as_str().unwrap_or("error"),
        resp["exit"].as_i64().map_or(1, |x| x as i32),
        resp["message"]
            .as_str()
            .unwrap_or("The TUI refused the request."),
    );
    for h in resp["help"].as_array().into_iter().flatten() {
        if let Some(h) = h.as_str() {
            e = e.hint(h);
        }
    }
    if let Some(r) = resp.get("rev").filter(|r| !r.is_null()) {
        e.detail.push(("rev".into(), r.clone()));
    }
    for k in ["expected", "actual", "skipped", "anchors"] {
        if let Some(v) = resp.get(k).filter(|v| !v.is_null()) {
            e.detail.push((k.into(), v.clone()));
        }
    }
    Err(e)
}

/// `ui upgrade`: ask the TUI to re-exec into the newer installed build (request kind `upgrade`). The answer
/// comes from the old build (`up_to_date`, `deferred`, `preflight_failed`, `exec_failed`) or, after exec,
/// from the new one on the same connection (`upgraded {from, to}`).
pub fn upgrade(g: &Global, session: Option<&str>) -> Res<()> {
    let s = pick(session)?;
    let req = wire::request_doc("upgrade", "upgrade", Map::new());
    let resp = match wire::request(std::path::Path::new(&s.socket), &req, UPGRADE_TIMEOUT) {
        Ok(r) => r,
        Err(e) => {
            // the connection went with the old process: the registry says whether the new one took over
            let start = std::time::Instant::now();
            loop {
                if let Some(now) = wire::sessions(&dirs())
                    .into_iter()
                    .find(|x| x.pid == s.pid && x.version != s.version)
                {
                    break json!({"ok": true, "status": "upgraded", "from": s.version, "to": now.version});
                }
                if start.elapsed() > Duration::from_secs(5) {
                    return Err(CliError::runtime(
                        "bridge",
                        format!("Could not reach TUI {}: {e}", s.pid),
                    )
                    .hint("Check `ownpurse ui ls`; the TUI may have exited."));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    };
    if resp["ok"] != true {
        let mut e = CliError::new(
            resp["error"].as_str().unwrap_or("error"),
            resp["exit"].as_i64().map_or(1, |x| x as i32),
            resp["message"]
                .as_str()
                .unwrap_or("The TUI refused the request."),
        );
        for k in ["reason", "from", "to"] {
            if let Some(v) = resp.get(k).filter(|v| !v.is_null()) {
                e.detail.push((k.into(), v.clone()));
            }
        }
        for h in resp["help"].as_array().into_iter().flatten() {
            if let Some(h) = h.as_str() {
                e = e.hint(h);
            }
        }
        return Err(e);
    }
    let mut out = Out::new(g.json);
    out.field("status", resp["status"].clone());
    for k in [
        "from",
        "to",
        "version",
        "render_epoch",
        "render_epoch_from",
        "render_epoch_to",
        "message",
    ] {
        if let Some(v) = resp.get(k).filter(|v| !v.is_null()) {
            out.field(k, v.clone());
        }
    }
    out.hint(match resp["status"].as_str() {
        Some("upgraded") => "Run `ownpurse ui state` to read the restored view.",
        Some("deferred") => {
            "It applies by itself when the sync finishes; `ownpurse ui ls` shows the version."
        }
        _ => "Nothing to do: the installed ownpurse-tui is not newer than the running one.",
    });
    out.emit();
    Ok(())
}

/// The upgrade waits for the pre-flight (up to 8 s) and the new build's start.
const UPGRADE_TIMEOUT: Duration = Duration::from_secs(30);

pub fn ls(g: &Global) -> Res<()> {
    let all = wire::sessions(&dirs());
    let rows: Vec<Map<String, Value>> = all
        .iter()
        .map(|s| {
            obj![
                ("id", s.id.clone()),
                ("label", s.label.clone()),
                ("pid", s.pid),
                ("scope", s.scope.clone()),
                ("tab", s.tab.clone()),
                ("theme", s.theme.clone()),
                ("agent", s.agent),
                ("profile", s.profile.clone()),
                ("version", s.version.clone()),
                ("started", s.started.clone()),
                ("cwd", s.cwd.clone()),
                ("socket", s.socket.clone()),
            ]
        })
        .collect();
    let mut out = Out::new(g.json);
    out.table(
        "sessions",
        &[
            "id", "label", "pid", "scope", "tab", "theme", "agent", "profile", "version", "started",
        ],
        rows,
        None,
    );
    out.hint(if all.len() > 1 {
        "Several sessions: pass `--session <id|label|scope:alias>` to `ownpurse ui state|screen|send`."
    } else {
        "Run `ownpurse ui state` to read the view, `ownpurse ui send …` to change it."
    });
    out.emit();
    Ok(())
}

/// The state's text form: scalar facts as fields, the visible rows as a table.
fn state_out(out: &mut Out, st: &Value) {
    for k in [
        "rev",
        "profile",
        "tab",
        "level",
        "mode",
        "scope",
        "as_of",
        "basis",
        "filter",
        "filter_applies",
        "filter_matches",
        "filter_total",
        "differs_only",
        "sort",
        "breadcrumb",
        "compare",
        "selected",
        "view",
        "truncated",
        "strip",
        "size",
        "theme",
        "overlay",
        "prompt",
        "band",
        "running",
        "busy",
    ] {
        if let Some(v) = st.get(k) {
            out.field(k, v.clone());
        }
    }
    let (cols, rows) = flat_rows(st);
    out.table_cols("rows", cols, rows, None);
}

/// The rows as a native TOON table (v0.10.3): the anchor's keys as `anchor.<key>` columns and every field
/// as its own column, in order of first appearance; no JSON-in-a-string cells.
fn flat_rows(st: &Value) -> (Vec<String>, Vec<Map<String, Value>>) {
    let mut cols: Vec<String> = vec!["row".into(), "selected".into()];
    let mut rows = Vec::new();
    for r in st["rows"].as_array().into_iter().flatten() {
        let mut m = obj![
            ("row", r["row"].clone()),
            ("selected", r["selected"].clone())
        ];
        let anchor = r["anchor"].as_object().cloned().unwrap_or_default();
        let fields = r["fields"].as_object().cloned().unwrap_or_default();
        for (k, v) in anchor
            .into_iter()
            .map(|(k, v)| (format!("anchor.{k}"), v))
            .chain(fields)
        {
            if !cols.contains(&k) {
                cols.push(k.clone());
            }
            m.insert(k, v);
        }
        rows.push(m);
    }
    (cols, rows)
}

/// `ui send`'s default text (v0.10.3): what changed and where the view is now, not the whole state.
fn send_summary(out: &mut Out, resp: &Value) {
    let st = &resp["state"];
    out.field("changed", resp["changed"].clone());
    for k in ["layers", "saved", "save_error"] {
        if let Some(v) = resp.get(k) {
            out.field(k, v.clone());
        }
    }
    if let Some(n) = resp.get("note").filter(|n| !n.is_null()) {
        out.field("note", n.clone());
        out.field("message", resp["message"].clone());
    }
    out.field("rev", st["rev"].clone());
    let crumbs: Vec<String> = st["breadcrumb"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|c| c.as_str().map(str::to_string))
        .collect();
    out.field("breadcrumb", crumbs.join(" › "));
    if let Some(q) = st["filter"].as_str().filter(|q| !q.is_empty()) {
        out.field(
            "filter",
            format!(
                "“{q}” {} of {}",
                st["filter_matches"].as_u64().unwrap_or(0),
                st["filter_total"].as_u64().unwrap_or(0)
            ),
        );
    }
    if !st["selected"].is_null() {
        out.field("selected", st["selected_label"].clone());
        out.field("anchor", st["selected"].clone());
    }
    if let Some(c) = st["caption"].get("text").filter(|c| !c.is_null()) {
        out.field("caption", c.clone());
    }
    if let Some(n) = st.get("notice").filter(|n| !n.is_null()) {
        out.field("notice", n.clone());
    }
}

pub fn state(
    g: &Global,
    session: Option<&str>,
    rows: Option<String>,
    only: Option<&str>,
) -> Res<()> {
    let s = pick(session)?;
    let mut fields = Map::new();
    if let Some(r) = rows {
        fields.insert("rows".into(), json!(r));
    }
    let resp = call(&s, "state", fields)?;
    // `--fields tab,breadcrumb,selected.account_code`: a lean projection (dotted paths kept as keys)
    if let Some(list) = only {
        let mut m = Map::new();
        for path in list.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            m.insert(
                path.to_string(),
                wire::at_path(&resp["state"], path).clone(),
            );
        }
        if g.json {
            print_json(&Value::Object(m));
        } else {
            let mut out = Out::new(false);
            for (k, v) in m {
                out.field(&k, v);
            }
            out.emit();
        }
        return Ok(());
    }
    if g.json {
        print_json(&resp["state"]);
        return Ok(());
    }
    let mut out = Out::new(false);
    state_out(&mut out, &resp["state"]);
    out.hint("`ownpurse ui send --expect-rev <rev> <verb …>` changes the view; `ownpurse ui screen` shows it.");
    out.emit();
    Ok(())
}

fn print_json(v: &Value) {
    println!(
        "{}",
        serde_json::to_string_pretty(v).unwrap_or_else(|_| v.to_string())
    );
}

/// A screen response's body in `format`, as bytes (PNG rasterised here from the TUI's cell grid).
fn screen_bytes(screen: &Value, format: &str) -> Res<Vec<u8>> {
    if format == "png" {
        return raster::png(&screen["body"]).map_err(|e| CliError::runtime("png", e));
    }
    Ok(screen["body"].as_str().unwrap_or("").as_bytes().to_vec())
}

fn wire_format(format: &str) -> &str {
    if format == "png" { "grid" } else { format }
}

/// Write a frame to `out` (or stdout for text formats); returns the summary fields.
fn deliver(
    screen: &Value,
    format: &str,
    out: Option<&PathBuf>,
    json_out: bool,
) -> Res<Map<String, Value>> {
    let bytes = screen_bytes(screen, format)?;
    let mut m = obj![
        ("rev", screen["rev"].clone()),
        ("format", format),
        ("cols", screen["cols"].clone()),
        ("rows", screen["rows"].clone()),
    ];
    match out {
        Some(p) => {
            std::fs::write(p, &bytes).map_err(|e| {
                CliError::runtime("write", format!("Could not write {}: {e}", p.display()))
            })?;
            m.insert("out".into(), json!(p.display().to_string()));
            m.insert("bytes".into(), json!(bytes.len()));
            if format == "png" {
                let (cols, rows) = (
                    screen["cols"].as_u64().unwrap_or(0),
                    screen["rows"].as_u64().unwrap_or(0),
                );
                m.insert(
                    "pixels".into(),
                    json!(format!(
                        "{}x{}",
                        cols * u64::from(raster::CELL_W),
                        rows * u64::from(raster::CELL_H)
                    )),
                );
            }
        }
        None if format == "png" => {
            return Err(CliError::usage("A PNG needs a file: pass --out FILE."));
        }
        None if json_out => {
            m.insert(
                "body".into(),
                json!(String::from_utf8_lossy(&bytes).to_string()),
            );
        }
        None => {
            use std::io::Write;
            let mut so = std::io::stdout().lock();
            let _ = so.write_all(&bytes);
            let _ = so.flush();
            m.insert("out".into(), json!("stdout"));
        }
    }
    Ok(m)
}

/// `ui screen --from-state FILE`: the frame of a saved state, rendered by `ownpurse-tui --state` (the
/// sibling binary, else PATH). No live session; the record must match unless drift is allowed.
pub fn screen_from_state(
    g: &Global,
    format: &str,
    out: Option<PathBuf>,
    file: &std::path::Path,
    allow_drift: bool,
    allow_config_drift: bool,
    allow_classify_drift: bool,
) -> Res<()> {
    let tui = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("ownpurse-tui")))
        .filter(|p| p.exists())
        .unwrap_or_else(|| PathBuf::from("ownpurse-tui"));
    let mut cmd = std::process::Command::new(&tui);
    cmd.arg("--state")
        .arg(file)
        .arg("--format")
        .arg(if format == "text" { "txt" } else { format });
    if let Some(p) = &g.profile {
        cmd.arg("--profile").arg(p);
    }
    if allow_drift {
        cmd.arg("--allow-record-drift");
    }
    if allow_config_drift {
        cmd.arg("--allow-config-drift");
    }
    if allow_classify_drift {
        cmd.arg("--allow-classify-drift");
    }
    if let Some(o) = &out {
        cmd.arg("--out").arg(o);
    }
    let res = cmd.output().map_err(|e| {
        CliError::runtime(
            "tui_missing",
            format!("Could not run {}: {e}", tui.display()),
        )
        .hint("Install both binaries with ./install.sh.")
    })?;
    if !res.status.success() {
        let text = String::from_utf8_lossy(&res.stdout).to_string();
        let field = |k: &str| {
            text.lines()
                .find_map(|l| l.strip_prefix(&format!("{k}: ")))
                .map(str::to_string)
        };
        let code = field("error").unwrap_or_else(|| "render".into());
        let msg = field("message")
            .unwrap_or_else(|| String::from_utf8_lossy(&res.stderr).trim().to_string());
        let mut e = CliError::new(&code, res.status.code().unwrap_or(1), msg);
        if code == "record_changed" {
            e = e.hint("Pass --allow-record-drift to render against the current record (marked as drifted).");
        }
        if code == "config_changed" {
            e = e.hint(
                "Pass --allow-config-drift to render under the current config (marked as drifted).",
            );
        }
        if code == "classify_changed" {
            e = e.hint(
                "Pass --allow-classify-drift to render with the current labels (marked as drifted).",
            );
        }
        return Err(e);
    }
    if out.is_none() && g.json {
        // the same document as a live `ui screen --json`: rev, format, size and the body
        let saved: Value = std::fs::read_to_string(file)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or(Value::Null);
        print_json(
            &json!({"rev": saved["rev"], "format": format, "cols": saved["size"]["cols"],
                           "rows": saved["size"]["rows"], "from_state": file.display().to_string(),
                           "body": String::from_utf8_lossy(&res.stdout)}),
        );
    } else if out.is_none() {
        use std::io::Write;
        let _ = std::io::stdout().write_all(&res.stdout);
    } else if g.json {
        print_json(&json!({"format": format, "out": out.map(|p| p.display().to_string())}));
    }
    Ok(())
}

pub fn screen(g: &Global, format: &str, out: Option<PathBuf>, session: Option<&str>) -> Res<()> {
    let s = pick(session)?;
    let resp = call(&s, "screen", obj![("format", wire_format(format))])?;
    let to_stdout = out.is_none() && format != "png";
    let m = deliver(&resp["screen"], format, out.as_ref(), g.json)?;
    if g.json {
        print_json(&Value::Object(m));
    } else if !to_stdout {
        let mut o = Out::new(false);
        for (k, v) in m {
            o.field(&k, v);
        }
        o.emit();
    }
    Ok(())
}

pub struct SendArgs {
    pub verbs: Vec<String>,
    pub expect_rev: Option<u64>,
    /// `--expect PATH=VALUE` predicates (AND-ed; refused with view_mismatch when one fails).
    pub expect: Vec<String>,
    pub screen: Option<String>,
    pub json_out: bool,
    pub out: Option<PathBuf>,
    pub caption: Option<String>,
    pub client: Option<String>,
    pub session: Option<String>,
    /// `--state`: the whole view state in text (default: the compact summary).
    pub full_state: bool,
}

pub fn send(g: &Global, a: &SendArgs) -> Res<()> {
    // `ui send --json '<Msg>'`: the global --json flag followed by one JSON argument is the message form
    let msg: Option<Value> = match a.verbs.as_slice() {
        [one]
            if g.json
                && (one.trim_start().starts_with('{') || one.trim_start().starts_with('[')) =>
        {
            Some(serde_json::from_str(one).map_err(|e| {
                CliError::usage(format!("--json: not a message ({e})."))
                    .hint(r#"Example: ownpurse ui send --json '{"verb": "tab", "args": ["2"]}'"#)
            })?)
        }
        _ => None,
    };
    let json_out = a.json_out || (g.json && msg.is_none());
    let mut fields = Map::new();
    match &msg {
        Some(m) => {
            fields.insert("msg".into(), m.clone());
        }
        None => {
            let mut verbs = wire::split_verbs(&a.verbs).map_err(|(code, message)| {
                CliError::new(code, 2, message).hint(
                    "View verbs: tab, scope, asof, filter, clear-filter, select, drill, up, back, forward, basis, differs, compare, reload, open, caption, label, manifest, mode; cosmetic: theme, accent, overlay set|clear, hint, highlight, spotlight, clear-hints, narrate; walkthrough play|step|next|prev|stop|record (overlay, hint, spotlight, clear-hints, narrate and walkthrough take the rest of the list: send them last).",
                )
            })?;
            // --caption rides in the same send (one update, atomic); `overlay` takes the rest, so it stays last
            if let Some(c) = &a.caption {
                let at = verbs.len() - usize::from(verbs.last().is_some_and(|v| v[0] == "overlay"));
                verbs.insert(at, vec!["caption".into(), c.clone()]);
            }
            fields.insert("verbs".into(), json!(verbs));
        }
    }
    if let Some(c) = &a.client {
        fields.insert("client".into(), json!(c));
    }
    if let Some(n) = a.expect_rev {
        fields.insert("expect_rev".into(), json!(n));
    }
    if !a.expect.is_empty() {
        for e in &a.expect {
            wire::parse_expect(e).map_err(|m| {
                CliError::usage(m).hint(
                    "Predicates are PATH=VALUE, PATH!=VALUE or PATH~TEXT over `ownpurse ui state --json` fields, e.g. --expect tab=ledger --expect level=2.",
                )
            })?;
        }
        fields.insert("expect".into(), json!(a.expect));
    }
    if let Some(f) = &a.screen {
        fields.insert("screen".into(), json!(wire_format(f)));
    }
    let s = pick(a.session.as_deref())?;
    let resp = call(&s, "send", fields)?;
    contrast_warnings(&resp);
    let shot = match (&a.screen, resp.get("screen")) {
        (Some(f), Some(sc)) => {
            let to_stdout = a.out.is_none() && f != "png";
            if to_stdout && !json_out {
                None // printed after the state, below
            } else {
                Some(deliver(sc, f, a.out.as_ref(), json_out)?)
            }
        }
        _ => None,
    };
    if json_out {
        let mut doc = obj![
            ("rev", resp["rev"].clone()),
            ("changed", resp["changed"].clone()),
            ("effects", resp["effects"].clone()),
            ("summary", resp["summary"].clone()),
            ("state", resp["state"].clone()),
        ];
        if let Some(n) = resp.get("note").filter(|n| !n.is_null()) {
            doc.insert("note".into(), n.clone());
            doc.insert("message".into(), resp["message"].clone());
        }
        // overlays: where each layer landed; a recording: where it was saved
        for k in ["layers", "saved", "save_error"] {
            if let Some(v) = resp.get(k) {
                doc.insert(k.into(), v.clone());
            }
        }
        if let Some(m) = shot {
            doc.insert("screen".into(), Value::Object(m));
        }
        print_json(&Value::Object(doc));
        return Ok(());
    }
    let mut out = Out::new(false);
    if a.full_state {
        out.field("changed", resp["changed"].clone());
        if let Some(n) = resp.get("note").filter(|n| !n.is_null()) {
            out.field("note", n.clone());
            out.field("message", resp["message"].clone());
        }
        state_out(&mut out, &resp["state"]);
    } else {
        send_summary(&mut out, &resp);
        out.hint("`--state` for the whole view state, `--json` for it as JSON.");
    }
    if let Some(m) = &shot {
        out.field("screen", Value::Object(m.clone()));
    }
    out.emit();
    if let (Some(f), None, Some(sc)) = (&a.screen, &shot, resp.get("screen")) {
        deliver(sc, f, None, false)?;
    }
    Ok(())
}

/// Overlays are exempt from the contrast rules but warned (theme spec v2 §2.7): each failure on stderr,
/// exit 0.
fn contrast_warnings(resp: &Value) {
    for w in resp["contrast_warnings"].as_array().into_iter().flatten() {
        if let Some(w) = w.as_str() {
            eprintln!("warning: contrast: {w}");
        }
    }
}

/// `ui flash` arguments.
pub struct FlashArgs {
    pub session: Option<String>,
    pub times: u32,
    pub on: String,
    pub off: String,
    pub tokens: Vec<String>,
    pub accent: Option<String>,
}

/// `200ms`, `0.3s`, `300` (ms) → milliseconds.
fn millis(s: &str, what: &str) -> Res<u64> {
    let t = s.trim();
    let bad = || CliError::usage(format!("{what}: `{s}` is not a duration (200ms, 0.3s)."));
    if let Some(ms) = t.strip_suffix("ms") {
        return ms.trim().parse().map_err(|_| bad());
    }
    if let Some(sec) = t.strip_suffix('s') {
        let d = rust_decimal::Decimal::from_str_exact(sec.trim()).map_err(|_| bad())?;
        let ms = d * rust_decimal::Decimal::from(1000);
        return u64::try_from(ms.round()).map_err(|_| bad());
    }
    t.parse().map_err(|_| bad())
}

/// The photosensitivity limits (WCAG 2.3.1, at most three flashes in any second): `too_fast`, exit 2.
pub fn flash_limits(times: u32, on_ms: u64, off_ms: u64) -> Res<()> {
    if on_ms + off_ms < 334 {
        return Err(CliError::new(
            "too_fast",
            2,
            format!(
                "--on + --off is {} ms: a flash cycle must take at least 334 ms (at most three flashes a second).",
                on_ms + off_ms
            ),
        ));
    }
    if !(1..=5).contains(&times) {
        return Err(CliError::new(
            "too_fast",
            2,
            format!("--times {times}: from 1 to 5 flashes."),
        ));
    }
    Ok(())
}

static INTERRUPTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

extern "C" fn on_interrupt(_: libc::c_int) {
    INTERRUPTED.store(true, std::sync::atomic::Ordering::SeqCst);
}

/// Sleep in short steps; false when Ctrl-C arrived.
fn wait(ms: u64) -> bool {
    let end = std::time::Instant::now() + Duration::from_millis(ms);
    while std::time::Instant::now() < end {
        if INTERRUPTED.load(std::sync::atomic::Ordering::SeqCst) {
            return false;
        }
        std::thread::sleep(Duration::from_millis(10).min(end - std::time::Instant::now()));
    }
    !INTERRUPTED.load(std::sync::atomic::Ordering::SeqCst)
}

/// `ui flash`: the attention effect, done by the CLI as a sequence of overlay states (the TUI never
/// animates; theme spec v2 §2.8). Reads the session's overlay, sets the flash tokens, waits `--on`,
/// restores, waits `--off`, `--times` times, and always ends by restoring the overlay it read, Ctrl-C
/// included.
pub fn flash(g: &Global, a: &FlashArgs) -> Res<()> {
    let (on_ms, off_ms) = (millis(&a.on, "--on")?, millis(&a.off, "--off")?);
    flash_limits(a.times, on_ms, off_ms)?;
    let s = pick(a.session.as_deref())?;
    let st = call(&s, "state", Map::new())?;
    let prior: Map<String, Value> = st["state"]["theme"]["overlay"]["tokens"]
        .as_object()
        .cloned()
        .unwrap_or_default();
    // the flash tokens: --token pairs, --accent, else the header band at 3× its tint (capped at 0.30)
    let mut set: Vec<String> = a.tokens.clone();
    if let Some(acc) = &a.accent {
        set.push(format!("accent={acc}"));
    }
    if set.is_empty() {
        let now = st["state"]["theme"]["knobs"]["header_band"]
            .as_str()
            .and_then(|v| rust_decimal::Decimal::from_str_exact(v).ok())
            .unwrap_or(rust_decimal::Decimal::new(10, 2));
        let cap = rust_decimal::Decimal::new(30, 2);
        let v = (now * rust_decimal::Decimal::from(3)).min(cap).round_dp(2);
        set.push(format!("knob.header_band={v:.2}"));
    }
    let names: Vec<String> = set
        .iter()
        .map(|p| p.split_once('=').map_or(p.as_str(), |(k, _)| k).to_string())
        .collect();
    // the state to come back to after each flash: the prior values of the flashed tokens, or cleared
    let restore = |s: &Session| -> Res<Value> {
        let back: Vec<String> = names
            .iter()
            .filter_map(|n| {
                prior
                    .get(n)
                    .and_then(Value::as_str)
                    .map(|v| format!("{n}={v}"))
            })
            .collect();
        let clear: Vec<String> = names
            .iter()
            .filter(|n| !prior.contains_key(*n))
            .cloned()
            .collect();
        let mut verbs = Vec::new();
        if !clear.is_empty() {
            let mut v = vec!["overlay".to_string(), "clear".to_string()];
            v.extend(clear);
            verbs.push(v);
        }
        let mut last = None;
        for v in verbs {
            last = Some(send_verbs(s, vec![v])?);
        }
        if !back.is_empty() {
            let mut v = vec!["overlay".to_string(), "set".to_string()];
            v.extend(back);
            last = Some(send_verbs(s, vec![v])?);
        }
        Ok(last.unwrap_or(Value::Null))
    };
    INTERRUPTED.store(false, std::sync::atomic::Ordering::SeqCst);
    // SAFETY: the handler only stores to an atomic (async-signal-safe).
    let old = unsafe {
        libc::signal(
            libc::SIGINT,
            on_interrupt as *const () as libc::sighandler_t,
        )
    };
    let mut result: Res<()> = Ok(());
    let mut flashes = 0;
    for _ in 0..a.times {
        let mut v = vec!["overlay".to_string(), "set".to_string()];
        v.extend(set.clone());
        if let Err(e) = send_verbs(&s, vec![v]) {
            result = Err(e);
            break;
        }
        flashes += 1;
        let held = wait(on_ms);
        if let Err(e) = restore(&s) {
            result = Err(e);
            break;
        }
        if !held || !wait(off_ms) {
            break;
        }
    }
    // always end on the overlay that was there before (also after Ctrl-C or a failed send)
    let end = restore(&s);
    // SAFETY: puts back the handler that was installed before.
    unsafe {
        libc::signal(libc::SIGINT, old);
    }
    result?;
    end?;
    let interrupted = INTERRUPTED.load(std::sync::atomic::Ordering::SeqCst);
    let mut out = Out::new(g.json);
    out.field("flashed", json!(flashes));
    out.field("tokens", json!(set));
    out.field("restored", json!(true));
    if interrupted {
        out.field("interrupted", json!(true));
    }
    out.emit();
    if interrupted {
        return Err(CliError::new(
            "interrupted",
            130,
            "Interrupted; the prior overlay was restored.",
        ));
    }
    Ok(())
}

/// One `send` of these verbs (no expect_rev: cosmetic verbs never conflict with the user's view).
fn send_verbs(s: &Session, verbs: Vec<Vec<String>>) -> Res<Value> {
    for v in &verbs {
        wire::check_verb(v).map_err(|(code, message)| CliError::new(code, 2, message))?;
    }
    let mut f = Map::new();
    f.insert("verbs".into(), json!(verbs));
    f.insert("client".into(), json!("flash"));
    let resp = call(s, "send", f)?;
    Ok(resp)
}

/// `ui wait --until COND [--timeout S]`: the TUI answers when the condition holds (event-driven, no polling),
/// or `timeout` (exit 4) with the current values.
pub fn wait_until(g: &Global, session: Option<&str>, until: &str, timeout_s: u64) -> Res<()> {
    let s = pick(session)?;
    let ms = timeout_s.saturating_mul(1000).min(600_000);
    let resp = call_within(
        &s,
        "wait",
        obj![("until", until), ("timeout_ms", ms)],
        Duration::from_millis(ms + 5_000),
    )?;
    if g.json {
        let mut m = resp.as_object().cloned().unwrap_or_default();
        m.remove("v");
        m.remove("ok");
        print_json(&Value::Object(m));
        return Ok(());
    }
    let mut out = Out::new(false);
    for k in ["until", "held", "waited_ms", "rev"] {
        out.field(k, resp[k].clone());
    }
    out.field("actual", resp["actual"].clone());
    out.emit();
    Ok(())
}

#[cfg(test)]
mod flash_tests {
    use super::*;

    #[test]
    fn limits() {
        assert!(flash_limits(3, 200, 200).is_ok());
        assert!(flash_limits(5, 167, 167).is_ok());
        let e = flash_limits(3, 100, 200).unwrap_err();
        assert_eq!((e.code.as_str(), e.exit), ("too_fast", 2));
        let e = flash_limits(6, 200, 200).unwrap_err();
        assert_eq!((e.code.as_str(), e.exit), ("too_fast", 2));
        assert_eq!(millis("200ms", "x").unwrap(), 200);
        assert_eq!(millis("0.3s", "x").unwrap(), 300);
        assert_eq!(millis("250", "x").unwrap(), 250);
        assert!(millis("fast", "x").is_err());
    }
}
