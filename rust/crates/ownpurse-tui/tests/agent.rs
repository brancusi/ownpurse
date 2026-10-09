//! The agent bridge (v0.10, view-only): safety items 1–8 and acceptance items 1–7. Requests go
//! through `Runtime::agent` exactly as the loop answers them (the socket tests add the real listener).

use ownpurse_core::bridge::{self as wire, Dirs};
use ownpurse_tui::app::{App, Msg, Tab};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme;
use ratatui::backend::TestBackend;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use serde_json::{Value, json};
use std::path::PathBuf;

type Rt = Runtime<Quiet<Counting>>;

fn rt_with(app: App, out: Counting) -> Rt {
    let backend = Quiet::fixed(out, true, 100, 30);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 30)),
        },
    )
    .unwrap();
    let mut rt = Runtime::new(terminal, app);
    rt.frame().unwrap();
    rt
}

fn rt() -> Rt {
    rt_with(fixture_app(theme::LEDGER_DARK, false), Counting::default())
}

fn req(rt: &mut Rt, doc: Value) -> Value {
    rt.agent(&doc)
}

fn send(rt: &mut Rt, verbs: &[&[&str]]) -> Value {
    req(
        rt,
        json!({"v": 1, "kind": "view", "op": "send", "verbs": verbs}),
    )
}

fn state(rt: &mut Rt) -> Value {
    req(rt, json!({"v": 1, "kind": "view", "op": "state"}))
}

fn key(rt: &mut Rt, k: &str) {
    rt.send(Msg::Key(Key::parse(k).unwrap().event()));
    rt.frame().unwrap();
}

fn notice(rt: &Rt) -> Option<String> {
    rt.app.agent_notice_text().map(|(t, _)| t)
}

// ── verbs go through update ──────────────────────────────────────────────────────────────────────

#[test]
fn view_verbs_apply_through_update_and_answer_the_new_state() {
    let mut rt = rt();
    let r0 = state(&mut rt)["rev"].as_u64().unwrap();
    let r = send(&mut rt, &[&["tab", "ledger"], &["scope", "design"]]);
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["changed"], json!(["opened Ledger", "scope design"]));
    assert!(r["rev"].as_u64().unwrap() > r0);
    assert_eq!(r["state"]["tab"], "ledger");
    assert_eq!(r["state"]["scope"], "design");
    assert_eq!(r["state"]["rev"], r["rev"]);
    assert_eq!(rt.app.ui.tab, Tab::Ledger);
    // visible rows only, each with an anchor; the selected row is one of them
    let rows = r["state"]["rows"].as_array().unwrap();
    assert!(!rows.is_empty() && rows.len() < 30, "{}", rows.len());
    assert!(rows.iter().all(|x| x["anchor"]["account_code"].is_string()));
    assert_eq!(
        rows.iter().filter(|x| x["selected"] == true).count(),
        1,
        "{rows:?}"
    );
    // select by account code, drill to postings (anchors carry doc_type / doc_id / line_id), back
    let r = send(
        &mut rt,
        &[&["scope", "roast"], &["select", "#1"], &["drill"]],
    );
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["state"]["mode"], "postings");
    let first = &r["state"]["rows"][0]["anchor"];
    assert!(
        first["doc_id"].is_string() && first["line_id"].is_number(),
        "{first}"
    );
    let doc_id = first["doc_id"].as_str().unwrap().to_string();
    let r = send(
        &mut rt,
        &[
            &["select", &json!({"doc_id": doc_id}).to_string()],
            &["drill"],
        ],
    );
    assert_eq!(r["state"]["mode"], "document", "{r}");
    assert_eq!(r["state"]["selected"]["doc_id"], doc_id.as_str());
    let r = send(&mut rt, &[&["up"], &["up"]]);
    assert_eq!(r["state"]["mode"], "trial balance");
    // the strip facts and the size
    let st = state(&mut rt)["state"].clone();
    assert_eq!(st["strip"]["calls_left_estimated"], false);
    assert_eq!(st["size"], json!({"cols": 100, "rows": 30}));
    assert_eq!(rt.spawns, 0);
    assert_eq!(rt.network_effects, 0);
}

#[test]
fn every_verb_maps_and_bad_arguments_change_nothing() {
    let mut rt = rt();
    let r = send(
        &mut rt,
        &[
            &["asof", "2025-06-30"],
            &["filter", "fees"],
            &["tab", "ledger"],
            &["scope", "design"],
            &["basis", "history"],
            &["clear-filter"],
            &["tab", "3"],
            &["tab", "overview"],
        ],
    );
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(rt.app.ui.as_of, "2025-06-30");
    assert!(rt.app.ui.full_history);
    assert!(rt.app.ui.filter.is_empty());
    assert!(rt.app.ui.prompt.is_none(), "no editor left open");
    let r = send(&mut rt, &[&["compare", "roast,design"]]);
    assert_eq!(
        r["state"]["compare"]["orgs"],
        json!(["roast", "design"]),
        "{r}"
    );
    let before = rt.app.ui.clone();
    let rev = rt.app.rev;
    for bad in [
        vec!["tab", "9"],
        vec!["asof", "yesterday"],
        vec!["scope", "nobody"],
        vec!["basis", "sometimes"],
        vec!["compare", "roast"],
    ] {
        let r = send(&mut rt, &[&bad]);
        assert_eq!(
            (r["ok"].clone(), r["exit"].clone()),
            (json!(false), json!(2)),
            "{bad:?}: {r}"
        );
        assert_eq!(rt.app.ui, before, "{bad:?}");
        assert_eq!(rt.app.rev, rev);
    }
    let r = send(&mut rt, &[&["select", "no such row anywhere"]]);
    assert_eq!(r["error"], "not_found");
    assert_eq!(r["exit"], 2);
}

// ── safety 1 / acceptance 7: the protocol rejects anything beyond view state ────────────────────

#[test]
fn forbidden_and_unknown_requests_are_refused_and_change_nothing() {
    let mut rt = rt();
    let before = rt.app.ui.clone();
    let rev = rt.app.rev;
    let mut words: Vec<&str> = wire::FORBIDDEN.to_vec();
    // v0.10.3: `upgrade` is its own request kind, never a view verb
    words.extend(["dance", "Enter", "ctrl-p", "upgrade"]);
    for w in words {
        for doc in [
            json!({"v": 1, "kind": "view", "op": "send", "verbs": [[w, "design"]]}),
            json!({"v": 1, "kind": "view", "op": "send", "verbs": [[w]]}),
            json!({"v": 1, "kind": "view", "op": "send", "msg": {"verb": w}}),
        ] {
            let r = req(&mut rt, doc.clone());
            assert_eq!(r["ok"], false, "{doc}");
            assert_eq!(r["exit"], 2, "{doc}: {r}");
            let code = r["error"].as_str().unwrap();
            assert!(["forbidden", "unknown_verb"].contains(&code), "{doc}: {r}");
            if wire::FORBIDDEN.contains(&w) || w == "upgrade" {
                assert_eq!(code, "forbidden", "{doc}");
            }
        }
    }
    for doc in [
        json!({"v": 1, "kind": "annotate", "op": "send", "verbs": [["note", "x"]]}),
        json!({"v": 1, "kind": "task", "op": "state"}),
        json!({"v": 1, "kind": "write", "op": "state"}),
        json!({"v": 2, "kind": "view", "op": "state"}),
        json!({"v": 1, "kind": "view", "op": "quit"}),
        json!({"v": 1, "kind": "view", "op": "send", "verbs": [["tab", "2"], ["confirm"]]}),
        // the upgrade kind takes its one op only
        json!({"v": 1, "kind": "upgrade", "op": "state"}),
        json!({"v": 1, "kind": "upgrade", "op": "send", "verbs": [["tab", "2"]]}),
        json!({"v": 1, "kind": "upgrade"}),
        json!({"v": 1, "kind": "view", "op": "upgrade"}),
    ] {
        let r = req(&mut rt, doc.clone());
        assert_eq!(
            (r["ok"].clone(), r["exit"].clone()),
            (json!(false), json!(2)),
            "{doc}: {r}"
        );
    }
    assert_eq!(
        req(&mut rt, json!({"v": 1, "kind": "annotate", "op": "send"}))["error"],
        "not_available"
    );
    assert_eq!(rt.app.ui, before, "nothing moved");
    assert_eq!(rt.app.rev, rev);
    assert!(!rt.quit);
    assert_eq!((rt.spawns, rt.network_effects), (0, 0));
    assert!(rt.app.ui.band.is_none(), "no confirm was opened");
}

// ── acceptance 1–2: the user's input wins ───────────────────────────────────────────────────────────

#[test]
fn a_send_while_the_user_types_is_refused_user_busy_with_no_frame() {
    let mut rt = rt();
    state(&mut rt); // connected (that draws once)
    for (keys, what) in [
        ("/", "typing in the filter"),
        ("ctrl-p", "in the command palette"),
        ("g", "in the scope picker"),
        ("?", "reading help"),
    ] {
        for k in keys.split_whitespace() {
            key(&mut rt, k);
        }
        let ui = rt.app.ui.clone();
        let draws = rt.draws;
        let r = send(&mut rt, &[&["tab", "ledger"]]);
        assert_eq!(r["error"], "user_busy", "{r}");
        assert_eq!(r["exit"], 4);
        assert_eq!(
            r["message"],
            format!("The user is {what}. Nothing was changed.")
        );
        assert_eq!(rt.app.ui, ui);
        assert_eq!(rt.draws, draws, "no frame while he is busy");
        assert!(notice(&rt).is_none(), "nothing shown in the overlay");
        key(&mut rt, "esc");
        assert_eq!(
            notice(&rt).as_deref(),
            Some("◇ agent: change skipped, you moved first"),
            "shown once he closes it"
        );
        key(&mut rt, "esc");
        assert!(notice(&rt).is_none());
    }
    // the confirm band
    rt.app
        .open_confirm(ownpurse_tui::app::NetAction::SyncOrg(2), None);
    let r = send(&mut rt, &[&["tab", "ledger"]]);
    assert_eq!(
        r["message"],
        "The user is looking at a confirm band. Nothing was changed."
    );
}

#[test]
fn a_stale_rev_is_refused_view_changed_and_the_view_stays() {
    let mut rt = rt();
    let r0 = state(&mut rt)["rev"].as_u64().unwrap();
    key(&mut rt, "2"); // the user moves
    key(&mut rt, "]");
    let now = rt.app.rev;
    assert!(now > r0);
    let ui = rt.app.ui.clone();
    let r = req(
        &mut rt,
        json!({"v": 1, "kind": "view", "op": "send", "verbs": [["tab", "1"]], "expect_rev": r0}),
    );
    assert_eq!(r["error"], "view_changed");
    assert_eq!(r["exit"], 4);
    assert_eq!(
        r["message"],
        format!(
            "The user changed the view since you read it (rev {r0} → {now}). Nothing was changed."
        )
    );
    assert_eq!(
        r["help"],
        json!(["Run `ownpurse ui state` again and resend if it still makes sense."])
    );
    assert_eq!(rt.app.ui, ui, "unchanged");
    assert_eq!(
        notice(&rt).as_deref(),
        Some("◇ agent: change skipped, you moved first")
    );
    // `expect_seq` is the same guard; the current rev passes
    let r = req(
        &mut rt,
        json!({"v": 1, "kind": "view", "op": "send", "verbs": [["tab", "1"]], "expect_seq": now}),
    );
    assert_eq!(r["ok"], true, "{r}");
}

// ── acceptance 3–4: the notice and going back ───────────────────────────────────────────────────

#[test]
fn the_notice_clears_on_the_next_key_esc_only_dismisses() {
    let mut rt = rt();
    send(
        &mut rt,
        &[&["tab", "ledger"], &["scope", "design"], &["differs", "on"]],
    );
    assert_eq!(
        notice(&rt).as_deref(),
        Some("◆ agent: opened Ledger · scope design · only accounts that differ · ` back")
    );
    assert_eq!(rt.app.agent_mark(), Some(true), "◆ agent in the header");
    key(&mut rt, "esc");
    assert!(notice(&rt).is_none());
    assert_eq!(rt.app.agent_mark(), Some(false), "back to ◇ (connected)");
    assert_eq!(rt.app.ui.tab, Tab::Ledger, "Esc only dismissed");
    assert!(rt.app.ui.differs_only);
    send(&mut rt, &[&["scope", "roast"]]);
    key(&mut rt, "1");
    assert!(notice(&rt).is_none());
    assert_eq!(
        rt.app.ui.tab,
        Tab::Overview,
        "any other key dismisses and acts"
    );
}

#[test]
fn backtick_restores_the_view_before_the_agent_one_step() {
    let mut rt = rt();
    key(&mut rt, "j"); // the user's own cursor
    let before = rt.app.ui.clone();
    send(&mut rt, &[&["tab", "ledger"], &["scope", "design"]]);
    send(&mut rt, &[&["filter", "loan"], &["basis", "history"]]); // unseen: one step back covers both
    assert!(
        notice(&rt)
            .unwrap()
            .starts_with("◆ agent: filter “loan” on Ledger")
    );
    key(&mut rt, "`");
    assert!(notice(&rt).is_none());
    assert_eq!(rt.app.ui.tab, before.tab);
    assert_eq!(rt.app.ui.scope, before.scope);
    assert_eq!(rt.app.ui.filter, before.filter);
    assert_eq!(rt.app.ui.full_history, before.full_history);
    assert_eq!(rt.app.ui.overview_sel, before.overview_sel, "selection too");
    // only one step: a second ` is an ordinary key (nothing bound) and the view stays
    let ui = rt.app.ui.clone();
    key(&mut rt, "`");
    assert_eq!(rt.app.ui, ui);
}

// ── safety 5 / acceptance 5: idle and no-op requests write nothing ──────────────────────────────

#[test]
fn idle_and_no_op_requests_write_zero_bytes() {
    let out = Counting::default();
    let mut rt = rt_with(fixture_app(theme::LEDGER_DARK, false), out.clone());
    state(&mut rt); // first contact: ◇ agent appears (one frame)
    send(&mut rt, &[&["scope", "design"]]); // a notice
    let bytes = out.bytes();
    let draws = rt.draws;
    // 10 s of idle (ticks inside one minute)
    let minute = rt.app.now.div_euclid(60) * 60;
    rt.app.now = minute;
    for i in 0..20 {
        rt.send(Msg::Tick(minute + i / 2));
        assert!(!rt.frame().unwrap());
    }
    // a read that changes nothing on screen, and a send that changes nothing
    state(&mut rt);
    let r = send(&mut rt, &[&["scope", "design"]]);
    assert_eq!(r["changed"], json!([]));
    assert_eq!(out.bytes(), bytes, "idle and no-op requests write 0 bytes");
    assert_eq!(rt.draws, draws);
}

// ── acceptance 6: Activity has one row per read, screenshot and send ────────────────────────────

#[test]
fn activity_has_one_row_per_read_screenshot_and_send() {
    let mut rt = rt();
    state(&mut rt);
    req(
        &mut rt,
        json!({"v": 1, "kind": "view", "op": "screen", "format": "png"}).clone(),
    );
    req(
        &mut rt,
        json!({"v": 1, "kind": "view", "op": "screen", "format": "grid"}),
    );
    send(&mut rt, &[&["tab", "2"], &["scope", "design"]]);
    let rows: Vec<String> = rt
        .app
        .model
        .activity
        .log
        .iter()
        .filter(|r| r.org == "agent")
        .map(|r| r.endpoint.clone())
        .collect();
    assert_eq!(
        rows,
        ["send tab=2 scope=design", "screenshot png", "read state"],
        "newest first (the unknown `png` wire format was refused and not logged)"
    );
    // they show on Activity and survive a record reload
    send(&mut rt, &[&["reload"]]);
    let r = send(&mut rt, &[&["tab", "activity"]]);
    let shown: Vec<&Value> = r["state"]["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|x| x["fields"]["org"] == "agent")
        .collect();
    assert!(shown.len() >= 4, "{shown:?}");
    assert!(rt.app.agent.acted, "a screenshot marks ◆ agent");
}

// ── safety 4: ui screen is the frame the TUI drew ───────────────────────────────────────────────

#[test]
fn the_screen_is_byte_identical_to_the_drawn_frame() {
    let terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
    let mut rt = Runtime::new(terminal, fixture_app(theme::LEDGER_DARK, false));
    rt.frame().unwrap();
    rt.agent(
        &json!({"v": 1, "kind": "view", "op": "send", "verbs": [["tab", "2"], ["scope", "design"]]}),
    );
    for format in ["text", "ansi", "html", "svg", "grid"] {
        let r = rt.agent(&json!({"v": 1, "kind": "view", "op": "screen", "format": format}));
        assert_eq!(r["ok"], true, "{r}");
        // the answer is the frame drawn for the rev it names (the screenshot's own ◆ mark comes after)
        assert_eq!(r["rev"], r["screen"]["rev"]);
        if format == "text" {
            let drawn = snapshot::text(rt.terminal.backend().buffer());
            let shot = r["screen"]["body"].as_str().unwrap();
            // the frame drawn after the screenshot differs only by ◇ → ◆ in the header
            assert_eq!(
                shot.replace('◇', "◆"),
                drawn,
                "screen text == the TestBackend frame"
            );
        }
    }
    // and with nothing pending, the off-screen render equals the backend buffer cell for cell
    rt.frame().unwrap();
    assert_eq!(&rt.current_frame(), rt.terminal.backend().buffer());
}

// ── safety 3: never a token ─────────────────────────────────────────────────────────────────────

#[test]
fn state_and_screens_never_carry_the_token_file() {
    let dir = PathBuf::from(format!("/tmp/xab-{}-canary", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let canary = "CANARY-7f3a-refresh-token-do-not-leak";
    let tok = dir.join("tokens.json");
    std::fs::write(
        &tok,
        json!({"access_token": canary, "refresh_token": canary}).to_string(),
    )
    .unwrap();
    let demo = snapshot::fixtures_dir().join("demo");
    let mut cfg: Value =
        serde_json::from_str(&std::fs::read_to_string(demo.join("ownpurse.json")).unwrap())
            .unwrap();
    cfg["profiles"]["demo"]["token_file"] = json!(tok.display().to_string());
    cfg["profiles"]["demo"]["store"] = json!(demo.join("record.sqlite").display().to_string());
    let path = dir.join("ownpurse.json");
    std::fs::write(&path, cfg.to_string()).unwrap();
    let env = ownpurse_core::config::Env {
        vars: Default::default(),
        cwd: dir.clone(),
        home: PathBuf::from("/nonexistent"),
    };
    let c = ownpurse_core::config::load(&env, Some("demo"), Some(&path)).unwrap();
    let source = ownpurse_tui::model::Source::from_config(&c);
    let now = snapshot::fixture_now();
    let model = ownpurse_tui::model::Model::open(&source, now).unwrap();
    let app = snapshot::pinned(App::new(model, source, theme::LEDGER_DARK, false, now));
    let mut rt = rt_with(app, Counting::default());
    let mut seen = String::new();
    for t in ["1", "2", "3", "4", "5"] {
        let r = send(&mut rt, &[&["tab", t]]);
        seen.push_str(&r.to_string());
        for f in ["text", "html", "svg", "grid"] {
            seen.push_str(
                &req(
                    &mut rt,
                    json!({"v": 1, "kind": "view", "op": "screen", "format": f}),
                )
                .to_string(),
            );
        }
    }
    assert!(seen.contains("System") && seen.len() > 10_000);
    assert!(
        !seen.contains(canary),
        "the token file's contents never leave the TUI"
    );
    assert!(!seen.contains("refresh_token"));
}

// ── safety 2, 7, 8: the socket, the registry, the switches ──────────────────────────────────────

fn short_dirs(name: &str) -> Dirs {
    // short paths: a Unix socket address holds about 100 bytes
    let root = PathBuf::from(format!("/tmp/xab-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    Dirs {
        state: root.join("s"),
        sockets: root.join("r"),
    }
}

#[test]
fn the_socket_is_private_registered_and_removed_on_exit() {
    use std::os::unix::fs::PermissionsExt;
    let dirs = short_dirs("sock");
    let (tx, rx) = std::sync::mpsc::channel::<ownpurse_tui::bridge::AgentRequest>();
    let server = ownpurse_tui::bridge::Server::start(
        dirs.clone(),
        "demo",
        Box::new(|| false),
        Box::new(move |r| tx.send(r).is_ok()),
    )
    .unwrap();
    let meta = std::fs::symlink_metadata(&server.path).unwrap();
    assert_eq!(meta.permissions().mode() & 0o777, 0o600, "socket 0600");
    let dmode = std::fs::metadata(&dirs.sockets)
        .unwrap()
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(dmode, 0o700, "socket directory 0700");
    // A connect from another uid is refused by these bits (the kernel checks write permission on the
    // socket file and search permission on the directory). Testing it needs a second user (root to
    // setuid), which the gate doesn't have; the bits are what the kernel enforces.
    let live = wire::sessions(&dirs);
    assert_eq!(live.len(), 1);
    assert_eq!(live[0].pid, std::process::id());
    assert_eq!(live[0].profile, "demo");
    // a real request over the socket, answered by the loop (here: this thread)
    let sock = server.path.clone();
    let client = std::thread::spawn(move || {
        wire::request(
            &sock,
            &wire::request_doc("view", "state", Default::default()),
            std::time::Duration::from_secs(60),
        )
    });
    let mut rt = rt();
    let r = rx.recv_timeout(std::time::Duration::from_secs(60)).unwrap();
    let answer = rt.agent(&r.doc);
    r.reply.send(answer).unwrap();
    let got = client.join().unwrap().unwrap();
    assert_eq!(got["ok"], true);
    assert_eq!(got["state"]["tab"], "overview");
    // envelope errors are answered by the listener itself (no loop round trip)
    let bad = std::os::unix::net::UnixStream::connect(&server.path).unwrap();
    {
        use std::io::{BufRead, BufReader, Write};
        let mut w = &bad;
        w.write_all(b"not json\n").unwrap();
        let mut line = String::new();
        BufReader::new(&bad).read_line(&mut line).unwrap();
        let v: Value = serde_json::from_str(&line).unwrap();
        assert_eq!(
            (v["error"].clone(), v["exit"].clone()),
            (json!("usage"), json!(2))
        );
    }
    let path = server.path.clone();
    drop(server);
    assert!(!path.exists(), "socket removed on exit");
    assert!(
        wire::sessions(&dirs).is_empty(),
        "registry entry removed on exit"
    );
}

#[test]
fn an_editor_makes_the_listener_answer_user_busy_itself() {
    let dirs = short_dirs("edit");
    let server = ownpurse_tui::bridge::Server::start(
        dirs,
        "demo",
        Box::new(|| true),
        Box::new(|_| panic!("the loop is in $EDITOR; nothing reaches it")),
    )
    .unwrap();
    let r = wire::request(
        &server.path,
        &wire::request_doc(
            "view",
            "send",
            serde_json::from_value(json!({"verbs": [["tab", "2"]]})).unwrap(),
        ),
        std::time::Duration::from_secs(60),
    )
    .unwrap();
    assert_eq!(r["error"], "user_busy");
    assert_eq!(r["exit"], 4);
}

#[test]
fn no_agent_and_tui_agent_false_open_no_socket() {
    let demo = snapshot::fixtures_dir().join("demo");
    let mut cfg: Value =
        serde_json::from_str(&std::fs::read_to_string(demo.join("ownpurse.json")).unwrap())
            .unwrap();
    let load = |cfg: &Value| {
        let dir = PathBuf::from(format!("/tmp/xab-{}-off", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("ownpurse.json");
        std::fs::write(&p, cfg.to_string()).unwrap();
        let env = ownpurse_core::config::Env {
            vars: Default::default(),
            cwd: demo.clone(),
            home: PathBuf::from("/nonexistent"),
        };
        ownpurse_tui::model::Source::from_config(
            &ownpurse_core::config::load(&env, Some("demo"), Some(&p)).unwrap(),
        )
    };
    let on = load(&cfg);
    assert!(on.agent, "on by default");
    assert!(ownpurse_tui::bridge::wanted(false, &on));
    assert!(!ownpurse_tui::bridge::wanted(true, &on), "--no-agent");
    cfg["tui"]["agent"] = json!(false);
    let off = load(&cfg);
    assert!(!off.agent);
    assert!(
        !ownpurse_tui::bridge::wanted(false, &off),
        "tui.agent: false"
    );
    // main.rs opens the socket only through `wanted`
    let main =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs")).unwrap();
    assert_eq!(main.matches("bridge::Server::start(").count(), 1);
    assert!(
        main.contains("let agent_on = ownpurse_tui::bridge::wanted(a.no_agent, &rt.app.source);")
    );
    assert!(main.contains("let mut bridge = if agent_on {"));
    // the only other start: re-opening it after a failed upgrade exec, when it was open before
    assert!(main.contains("if self.had_bridge && self.bridge.is_none() {"));
}

// ── the notice keeps its back hint; ASCII leads ───────────────────────────────────────────

#[test]
fn the_notice_keeps_the_back_hint_at_every_width() {
    for (w, h) in [(60u16, 20u16), (80, 24), (100, 30)] {
        let mut a = fixture_app(theme::LEDGER_DARK, false);
        snapshot::agent_send(
            &mut a,
            &[&["tab", "ledger"], &["scope", "design"], &["differs", "on"]],
            w,
            h,
        );
        let buf = snapshot::render(&mut a, w, h);
        let strip = snapshot::row_text(&buf, h - 1);
        assert!(strip.starts_with(" ◆ agent: opened"), "{w}: {strip}");
        assert!(strip.contains("· ` back"), "{w}: the hint stays: {strip}");
        if w < 100 {
            assert!(strip.contains('…'), "{w}: the middle is cut: {strip}");
        }
        assert!(
            strip.contains("differ · ` back"),
            "{w}: the end of the change stays: {strip}"
        );
    }
}

#[test]
fn ascii_and_no_color_leads_are_star_and_dash() {
    for th in [
        theme::NO_COLOR_ASCII,
        theme::Theme::named("no-color").unwrap(),
    ] {
        let mut a = fixture_app(th, false);
        a.agent_seen();
        a.agent_skipped();
        let strip = snapshot::row_text(&snapshot::render(&mut a, 100, 30), 29);
        assert!(
            strip.starts_with(" - agent: change skipped, you moved first"),
            "{strip}"
        );
        let mut a = fixture_app(th, false);
        snapshot::agent_send(&mut a, &[&["scope", "design"]], 100, 30);
        let buf = snapshot::render(&mut a, 100, 30);
        let strip = snapshot::row_text(&buf, 29);
        let want = ownpurse_tui::glyphs::fold(" * agent: scope design · ` back", th.ascii);
        assert!(strip.starts_with(&want), "{strip}");
        assert!(snapshot::row_text(&buf, 0).contains("[agent*]"));
        // Activity's rows carry words, no glyph
        snapshot::run_script(&mut a, "4", 100, 30);
        let t = snapshot::text(&snapshot::render(&mut a, 100, 30));
        assert!(
            t.contains("agent") && !t.contains('◆') && !t.contains('◇'),
            "{t}"
        );
    }
}

// ── "Open in Xero": the `open` verb (a link in the browser, never an API call) ──────────────────

#[test]
fn the_open_verb_opens_the_document_on_screen_and_is_refused_elsewhere() {
    let mut rt = rt();
    let r = send(&mut rt, &[&["tab", "ledger"], &["open"]]);
    assert_eq!(
        (r["ok"].clone(), r["error"].clone()),
        (json!(false), json!("not_applicable")),
        "{r}"
    );
    assert!(rt.external.is_empty(), "a refused batch opens nothing");
    key(&mut rt, "2");
    key(&mut rt, "]");
    key(&mut rt, "enter");
    key(&mut rt, "enter");
    rt.external.clear();
    let r = send(&mut rt, &[&["open"]]);
    assert_eq!(r["ok"], true, "{r}");
    let url = "https://go.xero.com/organisationlogin/default.aspx?shortcode=%21TmR4q&redirecturl=%2FBank%2FViewTransaction.aspx%3FbankTransactionID%3Dbt-102";
    assert_eq!(
        r["changed"],
        json!([format!("opened {url} in the browser")]),
        "{r}"
    );
    assert_eq!(
        rt.external,
        vec![ownpurse_tui::app::Effect::Open(url.into())]
    );
    // `open` takes no argument
    let r = send(&mut rt, &[&["open", "x"]]);
    assert_eq!(r["ok"], false, "{r}");
}
