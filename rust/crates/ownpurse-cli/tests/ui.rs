//! `ownpurse ui …` against a stand-in TUI (a Unix socket answering canned protocol documents; the CLI
//! never links the TUI): the session rules, the refusal output (golden), exit codes, and the PNG.

use ownpurse_core::bridge::{self as wire, Dirs, Session};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixListener;
use std::path::PathBuf;
use std::process::Command;

fn dirs(name: &str) -> Dirs {
    let root = PathBuf::from(format!("/tmp/xac-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    Dirs {
        state: root.join("state/ownpurse"),
        sockets: root.join("state/ownpurse"),
    }
}

/// A stand-in TUI: answers each request with `answer(request)`, records the requests.
fn fake_tui(d: &Dirs, answer: fn(&Value) -> Value) -> std::sync::Arc<std::sync::Mutex<Vec<Value>>> {
    wire::private_dir(&d.sockets).unwrap();
    let path = d.socket_for(std::process::id());
    let _ = std::fs::remove_file(&path);
    let l = UnixListener::bind(&path).unwrap();
    wire::register(
        d,
        &Session {
            id: wire::session_id(std::process::id()),
            label: Some("books".into()),
            scope: "roast".into(),
            tab: "ledger".into(),
            theme: "ledger-dark".into(),
            agent: true,
            pid: std::process::id(),
            socket: path.display().to_string(),
            started: "2026-10-06T07:00:00Z".into(),
            cwd: "/w".into(),
            profile: "demo".into(),
            version: "0.10.0".into(),
        },
    )
    .unwrap();
    let seen = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    let s2 = seen.clone();
    std::thread::spawn(move || {
        for c in l.incoming() {
            let Ok(c) = c else { continue };
            let mut line = String::new();
            BufReader::new(&c).read_line(&mut line).unwrap();
            let req: Value = serde_json::from_str(&line).unwrap();
            let out = answer(&req);
            s2.lock().unwrap().push(req);
            let mut w = &c;
            w.write_all(format!("{out}\n").as_bytes()).unwrap();
        }
    });
    seen
}

fn ui(d: &Dirs, args: &[&str]) -> (i32, String) {
    let state_home = d.state.parent().unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(args)
        .env("XDG_STATE_HOME", state_home)
        .env_remove("XDG_RUNTIME_DIR")
        .output()
        .unwrap();
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap(),
    )
}

fn view_changed(_: &Value) -> Value {
    wire::error_doc(
        "view_changed",
        4,
        "The user changed the view since you read it (rev 41 → 43). Nothing was changed.",
        &["Run `ownpurse ui state` again and resend if it still makes sense."],
    )
}

#[test]
fn the_refusal_reads_exactly_as_design_wrote_it() {
    let d = dirs("stale");
    let seen = fake_tui(&d, view_changed);
    let (code, out) = ui(&d, &["ui", "send", "--expect-rev", "41", "tab", "2"]);
    assert_eq!(code, 4);
    assert_eq!(
        out,
        "error: view_changed\n\
         message: The user changed the view since you read it (rev 41 → 43). Nothing was changed.\n\
         help[1]:\n  Run `ownpurse ui state` again and resend if it still makes sense.\n"
    );
    let req = seen.lock().unwrap()[0].clone();
    assert_eq!(
        req,
        json!({"v": 1, "kind": "view", "op": "send", "verbs": [["tab", "2"]], "expect_rev": 41})
    );
    // --expect-seq is the same flag
    let (code, _) = ui(&d, &["ui", "send", "--expect-seq", "41", "tab", "2"]);
    assert_eq!(code, 4);
    assert_eq!(seen.lock().unwrap()[1]["expect_rev"], 41);
    // user_busy is exit 4 too
    let d2 = dirs("busy");
    fake_tui(&d2, |_| {
        wire::error_doc(
            "user_busy",
            4,
            "The user is typing in the filter. Nothing was changed.",
            &[],
        )
    });
    let (code, out) = ui(&d2, &["ui", "send", "drill"]);
    assert_eq!(code, 4);
    assert!(out.starts_with("error: user_busy\nmessage: The user is typing in the filter."));
}

#[test]
fn unknown_and_forbidden_verbs_exit_2_before_anything_is_sent() {
    let d = dirs("verbs");
    let seen = fake_tui(&d, |_| json!({"v": 1, "ok": true, "rev": 1}));
    for (args, code_word) in [
        (vec!["ui", "send", "sync", "design"], "forbidden"),
        (vec!["ui", "send", "quit"], "forbidden"),
        (vec!["ui", "send", "tab", "2", "confirm"], "forbidden"),
        (vec!["ui", "send", "dance"], "unknown_verb"),
        (vec!["ui", "send", "tab"], "usage"),
    ] {
        let (code, out) = ui(&d, &args);
        assert_eq!(code, 2, "{args:?}: {out}");
        assert!(
            out.starts_with(&format!("error: {code_word}\n")),
            "{args:?}: {out}"
        );
    }
    assert!(seen.lock().unwrap().is_empty(), "nothing reached the TUI");
    // the message form goes to the TUI as is (the TUI checks it too)
    let (code, _) = ui(
        &d,
        &["ui", "send", "--json", r#"{"verb": "tab", "args": ["2"]}"#],
    );
    assert_eq!(code, 0);
    assert_eq!(
        seen.lock().unwrap()[0]["msg"],
        json!({"verb": "tab", "args": ["2"]})
    );
}

#[test]
fn sessions_none_one_several() {
    let d = dirs("sessions");
    let (code, out) = ui(&d, &["ui", "state"]);
    assert_eq!(code, 1);
    assert!(out.starts_with("error: no_session\n"), "{out}");
    let (code, out) = ui(&d, &["ui", "ls"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("sessions[0]: none"), "{out}");
    fake_tui(
        &d,
        |_| json!({"v": 1, "ok": true, "rev": 7, "state": {"rev": 7, "tab": "overview", "rows": []}}),
    );
    let (code, out) = ui(&d, &["ui", "ls"]);
    assert_eq!(code, 0);
    let me = std::process::id();
    assert!(
        out.contains(&format!(
            "{},books,{me},roast,ledger,ledger-dark,true,demo,0.10.0",
            wire::session_id(me)
        )),
        "{out}"
    );
    let (code, out) = ui(&d, &["--json", "ui", "state"]);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["rev"], 7);
    // a second (live-looking) session: --session is required and the message lists them
    let parent = std::process::id(); // any live pid with a socket file will do
    let other = Session {
        id: wire::session_id(1),
        label: Some("fees".into()),
        scope: "roast".into(),
        tab: "overview".into(),
        theme: "ledger-dark".into(),
        agent: false,
        pid: 1,
        socket: d.socket_for(parent).display().to_string(),
        started: "x".into(),
        cwd: "/".into(),
        profile: "p".into(),
        version: "v".into(),
    };
    let mut all = wire::sessions(&d);
    all.push(other);
    for s in &all {
        wire::register(&d, s).unwrap();
    }
    let (code, out) = ui(&d, &["ui", "state"]);
    assert_eq!(code, 2, "{out}");
    assert!(
        out.contains("pass --session <id|label|scope:alias>"),
        "{out}"
    );
    assert!(
        out.contains(&format!("pid {parent} (demo, “books”, scope roast)")),
        "{out}"
    );
    for key in [parent.to_string(), wire::session_id(parent), "books".into()] {
        let (code, out) = ui(&d, &["ui", "state", "--session", &key]);
        assert_eq!(code, 0, "{key}: {out}");
    }
    // `scope:roast` matches both: ambiguous, exit 2, the matches listed
    let (code, out) = ui(&d, &["ui", "state", "--session", "scope:roast"]);
    assert_eq!(code, 2, "{out}");
    assert!(
        out.contains("matches 2 sessions") && out.contains("“fees”"),
        "{out}"
    );
    let (code, out) = ui(&d, &["ui", "state", "--session", "nobody"]);
    assert_eq!(code, 2, "{out}");
}

#[test]
fn a_png_screen_decodes_at_the_grid_size() {
    let d = dirs("png");
    fake_tui(&d, |req| {
        assert_eq!(
            req["format"], "grid",
            "png is rasterised by the CLI from the grid"
        );
        json!({"v": 1, "ok": true, "rev": 3, "screen": {"format": "grid", "rev": 3, "cols": 4, "rows": 2, "body": {
            "cols": 4, "rows": 2, "bg": "#141a1c", "fg": "#e2e8ea",
            "lines": [[{"x": 0, "w": 4, "t": "ok ✓", "s": ["o","k"," ","✓"], "sw": [1,1,1,1], "fg": "#a3d9a5", "bg": null, "b": false, "d": false, "i": false, "u": false}], []]
        }}})
    });
    let out_png = d.state.join("f.png");
    let (code, out) = ui(
        &d,
        &[
            "ui",
            "screen",
            "--format",
            "png",
            "--out",
            out_png.to_str().unwrap(),
        ],
    );
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("pixels: 40x40") && out.contains("rev: 3"),
        "{out}"
    );
    let dec = png::Decoder::new(std::io::Cursor::new(std::fs::read(&out_png).unwrap()));
    let r = dec.read_info().unwrap();
    assert_eq!((r.info().width, r.info().height), (40, 40));
    let (code, out) = ui(&d, &["ui", "screen", "--format", "png"]);
    assert_eq!(code, 2, "a PNG needs --out: {out}");
}

#[test]
fn a_filter_notice_prints_without_doubled_quotes() {
    let d = dirs("quotes");
    fake_tui(&d, |_| {
        json!({"v": 1, "ok": true, "rev": 5, "changed": ["filter “Kiln Room -” on Ledger"],
               "state": {"rev": 5, "tab": "ledger", "filter": "Kiln Room -", "filter_matches": 9,
                         "filter_total": 43, "rows": []}})
    });
    let (code, out) = ui(&d, &["ui", "send", "filter", "Kiln Room -"]);
    assert_eq!(code, 0, "{out}");
    let first = out.lines().next().unwrap();
    assert_eq!(first, "changed: filter “Kiln Room -” on Ledger");
    assert!(!out.contains("\"\""), "{out}");
    assert!(out.contains("filter: “Kiln Room -” 9 of 43"), "{out}");
    // the whole state with --state
    let (_, out) = ui(&d, &["ui", "send", "--state", "filter", "Kiln Room -"]);
    assert!(out.contains("filter_matches: 9\nfilter_total: 43"), "{out}");
    // JSON stays exact
    let (_, out) = ui(&d, &["ui", "send", "--json-out", "filter", "Kiln Room -"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["changed"][0], "filter “Kiln Room -” on Ledger");
}

// ── v0.10.3 §7: the compact `ui send` text (golden), native TOON rows, `back` with no history ────────

/// An exact JSON number (as the TUI writes the `_value` twins).
fn exact(n: &str) -> Value {
    serde_json::from_str(n).unwrap()
}

fn sent(_: &Value) -> Value {
    json!({"v": 1, "ok": true, "rev": 12, "changed": ["opened Ledger", "scope roast", "filter “card” on Ledger"],
           "state": {"rev": 12, "tab": "ledger", "breadcrumb": ["roast", "Trial balance at 2025-12-31 (accrual)"],
                     "filter": "card", "filter_matches": 1, "filter_total": 11,
                     "selected": {"org": "roast", "account_code": "1060", "account_id": "r-card"},
                     "selected_label": "1060 Tallowmere Green-Bean Card",
                     "caption": {"text": "Card balance check", "by": "agent", "rev": 12},
                     "notice": "◆ agent: opened Ledger · scope roast · filter “card” on Ledger · ` back",
                     "rows": [{"row": 1, "selected": true,
                               "anchor": {"org": "roast", "account_code": "1060"},
                               "fields": {"code": "1060", "account": "Tallowmere Green-Bean Card", "debit": "", "debit_value": null,
                                          "credit": "487.36", "credit_value": exact("487.36")}}]}})
}

#[test]
fn ui_send_prints_a_compact_summary_by_default() {
    let d = dirs("compact");
    fake_tui(&d, sent);
    let (code, out) = ui(
        &d,
        &[
            "ui", "send", "tab", "ledger", "scope", "roast", "filter", "card",
        ],
    );
    assert_eq!(code, 0, "{out}");
    assert_eq!(
        out,
        "changed: opened Ledger, scope roast, filter “card” on Ledger\n\
rev: 12\n\
breadcrumb: roast › Trial balance at 2025-12-31 (accrual)\n\
filter: “card” 1 of 11\n\
selected: 1060 Tallowmere Green-Bean Card\n\
anchor: org=roast, account_code=1060, account_id=r-card\n\
caption: Card balance check\n\
notice: ◆ agent: opened Ledger · scope roast · filter “card” on Ledger · ` back\n\
help[1]:\n  `--state` for the whole view state, `--json` for it as JSON.\n"
    );
    // --state: the whole state, rows as a native table (no JSON inside a cell)
    let (_, out) = ui(&d, &["ui", "send", "--state", "tab", "ledger"]);
    assert!(
        out.contains("rows[1]{row,selected,anchor.org,anchor.account_code,code,account,debit,debit_value,credit,credit_value}:\n  1,true,roast,1060,1060,Tallowmere Green-Bean Card,,,487.36,487.36\n"),
        "{out}"
    );
    assert!(!out.contains("{\""), "{out}");
    // --json: the same document as before
    let (_, out) = ui(&d, &["ui", "send", "--json-out", "tab", "ledger"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        v["state"]["rows"][0]["fields"]["credit_value"].to_string(),
        "487.36"
    );
}

fn no_history(_: &Value) -> Value {
    json!({"v": 1, "ok": true, "rev": 7, "changed": [], "note": "no_history", "message": "nothing to go back to",
           "state": {"rev": 7, "breadcrumb": ["all", "Overview"], "rows": []}})
}

#[test]
fn back_with_no_history_is_never_silent() {
    let d = dirs("nohist");
    fake_tui(&d, no_history);
    let (code, out) = ui(&d, &["ui", "send", "back"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.starts_with(
            "changed: (none)\nnote: no_history\nmessage: nothing to go back to\nrev: 7\n"
        ),
        "{out}"
    );
    let (_, out) = ui(&d, &["ui", "send", "--json-out", "back"]);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(
        (v["note"].clone(), v["changed"].clone()),
        (json!("no_history"), json!([]))
    );
}

// ── v0.10.3: `ui upgrade` ───────────────────────────────────────────────────────────────────────────

#[test]
fn ui_upgrade_reports_each_outcome() {
    for (name, answer, code, first) in [
        (
            "upg-ok",
            (|_: &Value| json!({"v": 1, "ok": true, "status": "upgraded", "from": "0.10.2", "to": "0.10.3", "rev": 9}))
                as fn(&Value) -> Value,
            0,
            "status: upgraded\nfrom: 0.10.2\nto: 0.10.3\n",
        ),
        (
            "upg-def",
            |_: &Value| {
                json!({"v": 1, "ok": true, "status": "deferred", "from": "0.10.2", "to": "0.10.3", "rev": 9,
                               "message": "Deferred until the sync finishes: the upgrade to v0.10.3 applies by itself after it."})
            },
            0,
            "status: deferred\nfrom: 0.10.2\nto: 0.10.3\nmessage: Deferred until the sync finishes",
        ),
        (
            "upg-none",
            |_: &Value| json!({"v": 1, "ok": true, "status": "up_to_date", "version": "0.10.3", "rev": 9}),
            0,
            "status: up_to_date\nversion: 0.10.3\n",
        ),
        (
            "upg-pf",
            |_: &Value| {
                let mut d = wire::error_doc(
                    "preflight_failed",
                    1,
                    "The upgrade to v0.10.3 was not started: the pre-flight failed (record_changed: …). The TUI is unchanged.",
                    &[],
                );
                d["reason"] = json!("record_changed: …");
                d
            },
            1,
            "error: preflight_failed\nmessage: The upgrade to v0.10.3 was not started",
        ),
    ] {
        let d = dirs(name);
        let seen = fake_tui(&d, answer);
        let (c, out) = ui(&d, &["ui", "upgrade"]);
        assert_eq!(c, code, "{name}: {out}");
        assert!(out.starts_with(first), "{name}: {out}");
        let req = seen.lock().unwrap()[0].clone();
        assert_eq!(
            (req["kind"].clone(), req["op"].clone()),
            (json!("upgrade"), json!("upgrade"))
        );
    }
}
