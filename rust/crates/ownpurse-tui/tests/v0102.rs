//! v0.10.2 features around the canonical state: navigation history, restore on return, captions (with the
//! escape fuzz), update detection on the minute tick, labels and sessions, accent per scope kind.

use ownpurse_tui::app::{App, Msg, Tab};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use serde_json::{Value, json};

type Rt = Runtime<Quiet<Counting>>;

fn rt_with(out: Counting, w: u16, h: u16) -> Rt {
    let terminal = Terminal::with_options(
        Quiet::fixed(out, true, w, h),
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, w, h)),
        },
    )
    .unwrap();
    let mut rt = Runtime::new(terminal, fixture_app(theme::LEDGER_DARK, false));
    rt.frame().unwrap();
    rt
}

fn rt() -> Rt {
    rt_with(Counting::default(), 100, 30)
}

fn send(rt: &mut Rt, verbs: &[&[&str]]) -> Value {
    rt.agent(&json!({"v": 1, "kind": "view", "op": "send", "verbs": verbs}))
}

fn key(rt: &mut Rt, k: &str) {
    rt.send(Msg::Key(Key::parse(k).unwrap().event()));
    rt.frame().unwrap();
}

fn keys(rt: &mut Rt, ks: &str) {
    for k in ks.split_whitespace() {
        key(rt, k);
    }
}

fn round_trip(a: &App) -> App {
    let text = a.view_state(a.rev).canonical();
    App::from_state_json(&text, a.source.clone(), false).unwrap()
}

// ── §6 navigation history ───────────────────────────────────────────────────────────────────────

#[test]
fn alt_arrows_walk_the_history_and_a_new_navigation_drops_the_forward_stack() {
    let mut rt = rt();
    keys(&mut rt, "2 ] ]"); // Ledger all → roast → hold
    assert_eq!(rt.app.ui.history.back.len(), 3);
    key(&mut rt, "j"); // a cursor move is not a navigation
    assert_eq!(rt.app.ui.history.back.len(), 3);
    key(&mut rt, "alt-left");
    assert_eq!(rt.app.scope_label(&rt.app.ui.scope), "roast");
    assert!(rt.app.ui.note.as_deref().unwrap().starts_with("back → "));
    key(&mut rt, "alt-left");
    assert_eq!(rt.app.scope_label(&rt.app.ui.scope), "all");
    assert_eq!(rt.app.ui.history.forward.len(), 2);
    // the history round-trips, forward stack included
    let b = round_trip(&rt.app);
    assert_eq!(b.ui.history, rt.app.ui.history);
    key(&mut rt, "alt-right");
    assert_eq!(rt.app.scope_label(&rt.app.ui.scope), "roast");
    key(&mut rt, "1"); // a new navigation truncates forward
    assert!(rt.app.ui.history.forward.is_empty());
    // the selection comes back with the entry
    let mut rt = rt_mut_select();
    key(&mut rt, "alt-left");
    assert_eq!(rt.app.ui.tab, Tab::Ledger);
    assert_eq!(rt.app.l1_cursor(), Some(2), "the row you left from");
}

fn rt_mut_select() -> Rt {
    let mut rt = rt();
    keys(&mut rt, "2 ] j j 1");
    rt
}

#[test]
fn the_history_is_bounded() {
    let mut rt = rt();
    for _ in 0..130 {
        key(&mut rt, "]");
    }
    assert_eq!(
        rt.app.ui.history.back.len(),
        ownpurse_tui::history::HISTORY_MAX
    );
}

#[test]
fn agent_back_and_forward_say_where_they_land() {
    let mut rt = rt();
    send(&mut rt, &[&["tab", "ledger"], &["scope", "design"]]);
    let r = send(&mut rt, &[&["back"]]);
    assert_eq!(
        r["state"]["tab"], "overview",
        "one send = one history entry: {r}"
    );
    assert_eq!(r["changed"][0], "back → Overview");
    assert!(
        r["state"]["notice"]
            .as_str()
            .unwrap()
            .starts_with("◆ agent: back → Overview")
    );
    let r = send(&mut rt, &[&["forward"]]);
    assert_eq!(r["state"]["scope"], "design");
    let r = send(&mut rt, &[&["forward"]]);
    assert_eq!(r["changed"], json!([]), "nothing forward: unchanged");
}

// ── §12 restore on return ───────────────────────────────────────────────────────────────────────

#[test]
fn returning_to_a_filtered_tb_restores_the_filter_and_the_row() {
    for how in ["up", "esc", "back"] {
        let mut rt = rt();
        send(
            &mut rt,
            &[
                &["tab", "ledger"],
                &["scope", "roast"],
                &["filter", "Tallowmere "],
                &["select", "1060"],
            ],
        );
        send(&mut rt, &[&["drill"]]);
        assert_eq!(rt.app.ui.filter, "", "postings start unfiltered");
        match how {
            "up" => {
                send(&mut rt, &[&["up"]]);
            }
            "esc" => {
                key(&mut rt, "esc"); // the notice
                key(&mut rt, "esc");
            }
            _ => {
                send(&mut rt, &[&["back"]]);
            }
        }
        assert_eq!(
            rt.app.ui.filter, "Tallowmere ",
            "{how}: the filter comes back"
        );
        assert_eq!(rt.app.ledger_level(), 1, "{how}");
        let st = rt.app.view_state(rt.app.rev);
        let sel = st.selected.unwrap();
        assert_eq!(sel["account_code"], "1060", "{how}: the row drilled from");
        // and it round-trips
        assert_eq!(round_trip(&rt.app).ui.filter, "Tallowmere ");
    }
}

// ── §11 captions ────────────────────────────────────────────────────────────────────────────────

#[test]
fn captions_replace_the_notice_and_live_as_long_as_the_view() {
    let mut rt = rt();
    let cap = "Owner trial balance review, 2025 · 18 lines · 97,450.00";
    let r = rt.agent(
        &json!({"v": 1, "kind": "view", "op": "send", "client": "claude",
        "verbs": [["tab", "ledger"], ["scope", "roast"], ["caption", cap]]}),
    );
    let rev = r["rev"].as_u64().unwrap();
    assert_eq!(
        r["state"]["caption"],
        json!({"text": cap, "by": "claude", "rev": rev})
    );
    let foot = snapshot::row_text(&rt.current_frame(), 29);
    assert!(foot.starts_with(&format!(" ◆ {cap} · ` back")), "{foot}");
    assert!(
        !foot.contains("agent:"),
        "the caption replaces the auto notice: {foot}"
    );
    // kept through selection moves and help
    keys(&mut rt, "j ? esc");
    assert!(rt.app.ui.caption.is_some());
    // cleared by a navigation; state shows null
    key(&mut rt, "1");
    assert!(rt.app.ui.caption.is_none());
    assert_eq!(rt.app.state_json(rt.app.rev)["caption"], Value::Null);
    // a caption alone: no back hint; Esc dismisses only it
    let r = send(&mut rt, &[&["caption", "Overview of four orgs"]]);
    assert_eq!(r["changed"], json!(["caption “Overview of four orgs”"]));
    let foot = snapshot::row_text(&rt.current_frame(), 29);
    assert!(foot.starts_with(" ◆ Overview of four orgs   "), "{foot}");
    assert!(!foot.contains("back"));
    let tab = rt.app.ui.tab;
    key(&mut rt, "esc");
    assert!(rt.app.ui.caption.is_none());
    assert_eq!(rt.app.ui.tab, tab);
    // caption clear
    send(&mut rt, &[&["caption", "x"]]);
    send(&mut rt, &[&["caption", "clear"]]);
    assert!(rt.app.ui.caption.is_none());
}

#[test]
fn a_caption_send_is_atomic() {
    let mut rt = rt();
    let ui = rt.app.ui.clone();
    let rev = rt.app.rev;
    let r = send(
        &mut rt,
        &[
            &["tab", "ledger"],
            &["scope", "roast"],
            &["select", "no such row"],
            &["caption", "x"],
        ],
    );
    assert_eq!(r["error"], "not_found");
    assert!(
        r["message"]
            .as_str()
            .unwrap()
            .ends_with("Nothing was changed."),
        "{r}"
    );
    assert_eq!(
        (
            rt.app.ui.tab,
            rt.app.ui.scope.clone(),
            rt.app.ui.caption.clone(),
            rt.app.ui.history.clone()
        ),
        (
            ui.tab,
            ui.scope.clone(),
            ui.caption.clone(),
            ui.history.clone()
        ),
        "nothing applied, caption included"
    );
    assert_eq!(rt.app.rev, rev);
    // and a good send is one revision
    let r = send(
        &mut rt,
        &[&["tab", "ledger"], &["scope", "roast"], &["caption", "x"]],
    );
    assert_eq!(r["rev"].as_u64().unwrap(), rev + 1);
}

#[test]
fn bad_captions_are_rejected_and_nothing_raw_reaches_the_terminal() {
    let out = Counting::default();
    let mut rt = rt_with(out.clone(), 100, 30);
    let cases = [
        "\x1b[2J\x1b[Hgotcha",
        "\x1b]52;c;ZXZpbA==\x07",
        "\x1bP+q\x1b\\",
        "\u{9b}31mred",
        "line\nbreak",
        "carriage\rreturn",
        "bell\x07",
        "nul\u{0}",
        "del\x7f",
        "zero\u{200b}width",
        "rtl\u{202e}override",
        "isolate\u{2067}x\u{2069}",
        &"x".repeat(121),
    ];
    for c in cases {
        let r = send(&mut rt, &[&["caption", c]]);
        assert_eq!(
            (r["error"].clone(), r["exit"].clone()),
            (json!("bad_caption"), json!(2)),
            "{c:?}: {r}"
        );
        assert!(rt.app.ui.caption.is_none());
        let shown = snapshot::text(&rt.current_frame());
        assert!(!shown.contains('\x1b'), "{c:?}");
    }
    let bytes = out.data();
    // the terminal stream's escapes are the renderer's own (CSI cursor/colour); never OSC, DCS or a
    // caption's text after an ESC
    let s = String::from_utf8_lossy(&bytes);
    assert!(!s.contains("\x1b]52") && !s.contains("\x1bP+q") && !s.contains("gotcha"));
    // unicode is fine
    let r = send(&mut rt, &[&["caption", "東京 · ünï · 🙂"]]);
    assert_eq!(r["ok"], true, "{r}");
}

#[test]
fn captions_truncate_by_segment_and_keys_drop_first() {
    let cap = "Owner trial balance review, 2025 · 18 lines · 97,450.00";
    for (w, h, want) in [
        (140u16, 40u16, format!(" ◆ {cap} · ` back")),
        (100, 30, format!(" ◆ {cap} · ` back")),
        (
            80,
            24,
            " ◆ Owner trial balance review,… · … · 97,450.00 · `".to_string(),
        ),
        (60, 20, " ◆ Owner trial b… · … · 97,450.00".to_string()),
    ] {
        let mut a = fixture_app(theme::LEDGER_DARK, false);
        snapshot::agent_send(
            &mut a,
            &[&["tab", "ledger"], &["scope", "roast"], &["caption", cap]],
            w,
            h,
        );
        let foot = snapshot::row_text(&snapshot::render(&mut a, w, h), h - 1);
        assert!(foot.starts_with(&want), "{w}: {foot}");
        assert!(
            foot.trim_end()
                .ends_with(&format!("v{}", snapshot::TEST_VERSION)),
            "{w}: version protected: {foot}"
        );
    }
}

// ── §7 update detection ─────────────────────────────────────────────────────────────────────────

#[test]
fn a_newer_installed_build_shows_after_the_minute_tick_and_idle_writes_nothing() {
    let dir = std::env::temp_dir().join(format!("xab-upd-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bin = dir.join("ownpurse-tui");
    let write = |v: &str| {
        std::fs::write(&bin, format!("#!/bin/sh\necho ownpurse-tui {v}\n")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    };
    write(&ownpurse_tui::app::build_version());
    let out = Counting::default();
    let mut rt = rt_with(out.clone(), 100, 30);
    rt.update_probe = Some(bin.clone());
    rt.settle_update(std::time::Duration::from_secs(60));
    assert_eq!(rt.update_reads, 1);
    assert!(rt.app.update_available.is_none());
    // a minute with no change: one stat, no --version, 0 bytes
    let bytes = out.bytes();
    let minute = rt.app.now.div_euclid(60) * 60;
    for m in 1..=3 {
        rt.send(Msg::Tick(minute + 60 * m));
        rt.frame().unwrap();
    }
    assert_eq!(rt.update_reads, 1, "no --version without an mtime change");
    let ticks = out.bytes() - bytes;
    // (the minute ticks redraw ages anyway; the update check itself adds nothing)
    let _ = ticks;
    // reinstall a newer build: the next minute tick shows it
    std::thread::sleep(std::time::Duration::from_millis(1100));
    write("99.0.0");
    rt.send(Msg::Tick(minute + 60 * 4));
    // the read runs on its own thread; its answer comes with the next event
    rt.settle_update(std::time::Duration::from_secs(60));
    rt.frame().unwrap();
    assert_eq!(rt.update_reads, 2);
    assert_eq!(
        rt.app.update_available,
        Some(ownpurse_tui::canonical::Update {
            version: "99.0.0".into(),
            action: ownpurse_tui::upgrade::ACTION.into()
        })
    );
    let foot = snapshot::row_text(&rt.current_frame(), 29);
    assert!(
        foot.trim_end().ends_with(&format!(
            "v{}  ⬆ v99.0.0 ready · ⌃P Upgrade",
            rt.app.version_label
        )),
        "{foot}"
    );
    let st = rt.app.state_json(rt.app.rev);
    assert_eq!(
        st["update_available"],
        json!({"version": "99.0.0", "action": "upgrade_in_place"})
    );
    // a state render shows the exported value (no detection runs)
    let b = round_trip(&rt.app);
    assert_eq!(b.update_available, rt.app.update_available);
    // nothing changes for an idle runtime: no tick, no key, no bytes
    let bytes = out.bytes();
    for _ in 0..5 {
        assert!(!rt.frame().unwrap());
    }
    assert_eq!(out.bytes(), bytes);
}

#[test]
fn a_minute_tick_with_no_change_writes_zero_bytes_beyond_the_clock() {
    // same minute, many ticks: 0 bytes even with the update probe armed (V13)
    let out = Counting::default();
    let mut rt = rt_with(out.clone(), 100, 30);
    rt.update_probe = std::env::current_exe().ok();
    rt.settle_update(std::time::Duration::from_secs(60));
    rt.frame().unwrap();
    let bytes = out.bytes();
    let minute = rt.app.now.div_euclid(60) * 60;
    for s in 0..50 {
        rt.send(Msg::Tick(minute + s));
        assert!(!rt.frame().unwrap());
    }
    assert_eq!(out.bytes(), bytes);
}

// ── §8–10 theme, accent, keymap, label ──────────────────────────────────────────────────────────

#[test]
fn accent_follows_the_scope_kind_and_round_trips() {
    for (verbs, kind, members) in [
        (vec![vec!["scope", "all"]], "all", 4),
        (vec![vec!["scope", "design"]], "org", 1),
        (vec![vec!["scope", "group:all-co"]], "group", 4),
        (vec![vec!["compare", "roast,design"]], "pair", 2),
    ] {
        let mut a = fixture_app(theme::LEDGER_DARK, false);
        let v: Vec<&[&str]> = verbs.iter().map(|v| v.as_slice()).collect();
        snapshot::agent_send(&mut a, &v, 100, 30);
        let st = a.view_state(a.rev);
        assert_eq!(st.accent.scope_kind, kind);
        assert_eq!(st.accent.members.len(), members);
        // v0.12.0: the family the scope resolves to (an org's own; the cross accent otherwise)
        let want = if kind == "org" { "ember" } else { "harbor" };
        assert_eq!(st.accent.token, want);
        let b = round_trip(&a);
        assert_eq!(b.view_state(b.rev).accent, st.accent, "{kind}");
        let j = a.state_json(a.rev);
        assert_eq!(j["theme"]["overlay"], json!({"tokens": {}}));
        assert_eq!(j["theme"]["revision"], Value::Null);
        // v0.12.0 filled the reserved keymap block: the built-in table has no revision and no overrides
        assert_eq!(j["keymap"], json!({"revision": null, "overrides": 0}));
    }
}

#[test]
fn the_label_verb_sets_the_session_label() {
    let mut rt = rt();
    let r = send(&mut rt, &[&["label", "books"]]);
    assert_eq!(r["changed"], json!(["label “books”"]));
    assert_eq!(r["state"]["label"], "books");
    assert_eq!(round_trip(&rt.app).label.as_deref(), Some("books"));
}

#[test]
fn caret_editing_inserts_and_deletes_where_the_caret_is() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    snapshot::run_script(
        &mut a,
        "/ r o a s t left left x home backspace right backspace",
        100,
        30,
    );
    // roast ← ← (caret 3) x → roaxst (4); Home (0); ⌫ at 0 does nothing; → (1); ⌫ removes the r → oaxst (0)
    assert_eq!(a.ui.filter, "oaxst");
    let st = a.view_state(a.rev);
    let input = st.input.unwrap();
    assert_eq!((input.field, input.text.as_str()), ("filter", "oaxst"));
    assert_eq!(input.caret, 0);
    assert_eq!(round_trip(&a).ui.caret, a.ui.caret);
}

/// Design (v0.10.2 fix): the update notice keeps the running version (faint) and appends the update (accent).
#[test]
fn the_update_notice_keeps_the_running_version() {
    for (w, h, want) in [
        (
            100,
            30,
            "v0.0.0-test  ⬆ v0.10.4 installed · quit and restart to use it",
        ),
        (80, 24, "v0.0.0-test  ⬆ v0.10.4 · restart"),
        (60, 20, "v0.0.0-test ⬆ 0.10.4"),
    ] {
        let mut a = snapshot::fixture_app(ownpurse_tui::theme::LEDGER_DARK, false);
        a.update_available = Some(ownpurse_tui::canonical::Update {
            version: "0.10.4".into(),
            action: "restart".into(),
        });
        let buf = snapshot::render(&mut a, w, h);
        let foot = snapshot::row_text(&buf, h - 1);
        assert!(foot.trim_end().ends_with(want), "{w}: {foot}");
        // running version in the faint tone, the update in accent
        let chars: Vec<char> = foot.trim_end().chars().collect();
        let at = |c: usize| buf[(c as u16, h - 1)].fg;
        let start = chars.len() - want.chars().count();
        let arrow = start + want.chars().position(|c| c == '⬆').unwrap();
        assert_ne!(at(start), at(arrow), "{w}: two tones");
        assert_eq!(
            at(arrow),
            at(chars.len() - 1),
            "{w}: the update part is one tone"
        );
    }
}

#[test]
fn a_build_that_does_not_answer_is_asked_again_and_never_blocks_long() {
    let dir = std::env::temp_dir().join(format!("xab-upd-slow-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bin = dir.join("ownpurse-tui");
    let write = |body: &str| {
        let tmp = dir.join(".new");
        std::fs::write(&tmp, format!("#!/bin/sh\n{body}\n")).unwrap();
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&tmp, std::fs::Permissions::from_mode(0o755)).unwrap();
        std::fs::rename(&tmp, &bin).unwrap();
    };
    // no answer in time: bounded, nothing remembered
    write("sleep 30");
    let t = std::time::Instant::now();
    assert_eq!(
        ownpurse_tui::net::tui_version_within(&bin, std::time::Duration::from_millis(300)),
        None
    );
    assert!(t.elapsed() < std::time::Duration::from_secs(5));
    // garbage: not a version
    write("echo hello");
    assert_eq!(
        ownpurse_tui::net::tui_version_within(&bin, std::time::Duration::from_secs(3)),
        None
    );
    let mut rt = rt_with(Counting::default(), 100, 30);
    rt.update_probe = Some(bin.clone());
    rt.settle_update(std::time::Duration::from_secs(60));
    rt.settle_update(std::time::Duration::from_secs(60));
    assert_eq!(
        rt.update_reads, 2,
        "a failed read is retried at the next interaction"
    );
    assert!(rt.app.update_available.is_none());
    // a read never holds the loop: it runs on its own thread
    write("sleep 30");
    let t = std::time::Instant::now();
    rt.check_update();
    assert!(t.elapsed() < std::time::Duration::from_secs(2));
    assert!(rt.update_pending.is_some());
    // then it answers: noticed, and remembered
    write("echo ownpurse-tui 99.0.0");
    rt.update_pending = None; // (the slow read is abandoned, as after its timeout)
    rt.settle_update(std::time::Duration::from_secs(60));
    assert_eq!(
        rt.app.update_available.as_ref().map(|u| u.version.as_str()),
        Some("99.0.0")
    );
    let n = rt.update_reads;
    rt.settle_update(std::time::Duration::from_secs(60));
    assert_eq!(rt.update_reads, n, "a good read is remembered");
}

#[test]
fn an_identical_build_is_never_a_handover() {
    let dir = std::env::temp_dir().join(format!("xab-upd-same-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let bin = dir.join("ownpurse-tui");
    let mut rt = rt_with(Counting::default(), 100, 30);
    let mine =
        ownpurse_tui::app::full_version(&rt.app.version_label, rt.app.build_label.as_deref());
    std::fs::write(&bin, format!("#!/bin/sh\necho ownpurse-tui {mine}\n")).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();
    rt.update_probe = Some(bin);
    // even when something claims an update to this very build
    rt.app.update_available = Some(ownpurse_tui::canonical::Update {
        version: mine.clone(),
        action: ownpurse_tui::upgrade::ACTION.into(),
    });
    assert_eq!(rt.upgrade_ask(), ownpurse_tui::upgrade::Ask::UpToDate);
    assert!(!rt.upgrade_due);
    assert!(rt.app.update_available.is_none());
}
