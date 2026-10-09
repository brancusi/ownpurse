//! Writes in the TUI (Phase 8; read-only display, the TUI never links the api crate): a record copy gets a
//! real write appended by the api crate's test helper (`ownpurse-fake-record … write`: seal, demo standing
//! approval, run through the production engine against the loopback fake Xero), then Activity lists each
//! attempt, System lists the package with its stage, the journal's L3 names its package, and the
//! external-change notice says what landed.

use ownpurse_tui::app::App;
use ownpurse_tui::snapshot::{self, demo_app, run_script, text};
use ownpurse_tui::theme;
use std::process::Command;

const ROAST: &str = "c0ffee00-0000-4000-8000-0000000000a1";

/// The demo app on a record copy, before and after a write appended to it.
fn written() -> (
    App,
    ownpurse_tui::external::DiskRevs,
    ownpurse_tui::external::DiskRevs,
) {
    let mut a = demo_app(theme::LEDGER_DARK, false);
    let rec = snapshot::record_copy(&mut a);
    let before = ownpurse_tui::external::read(&rec, None);
    let st = Command::new(snapshot::fake_writer())
        .args([
            rec.to_str().unwrap(),
            "write",
            ROAST,
            "2026-10-05T09:40:00Z",
        ])
        .status()
        .expect("fake writer");
    assert!(st.success());
    let after = ownpurse_tui::external::read(&rec, None);
    snapshot::reload(&mut a);
    (a, before, after)
}

fn screen(a: &mut App, script: &str) -> String {
    run_script(a, script, 100, 30);
    text(&snapshot::render(a, 100, 30))
}

#[test]
fn activity_lists_each_write_attempt() {
    let (mut a, ..) = written();
    let w: Vec<_> = a
        .model
        .activity
        .log
        .iter()
        .filter(|r| r.is_write())
        .collect();
    let states: Vec<&str> = w
        .iter()
        .map(|r| r.status_text())
        .map(|s| if s == "sending" { "sending" } else { "other" })
        .collect();
    assert_eq!(w.len(), 2, "sending + confirmed");
    assert!(states.contains(&"sending"));
    assert!(w.iter().any(|r| r.status_text() == "confirmed"
        && r.endpoint == "op-1 create_journal · Owner-paid cupping supplies"
        && r.path.contains("XA:fixture-write-01:op-1")));
    // (filtered to the attempts: the write's own HTTP calls carry real durations)
    let s = screen(&mut a, "4 / c r e a t e enter");
    assert!(s.contains("write op-1 create_journal · Owner-p"), "{s}");
    insta::assert_snapshot!("writes-activity-100x30", s);
}

#[test]
fn system_lists_the_package_with_its_stage() {
    let (mut a, ..) = written();
    let p = &a.model.system.packages;
    assert_eq!(p.len(), 1);
    assert_eq!(p[0].stage.as_str(), "landed");
    assert_eq!(p[0].approval.as_deref(), Some("demo standing approval"));
    assert_eq!((p[0].landed, p[0].ops), (1, 1));
    let s = screen(&mut a, "5 / O w n e r enter");
    assert!(
        s.contains("Writes") && s.contains("landed") && s.contains("demo standing approval"),
        "{s}"
    );
    insta::assert_snapshot!("writes-system-100x30", s);
    // no package: no Writes section at all
    let mut b = demo_app(theme::LEDGER_DARK, false);
    assert!(!screen(&mut b, "5").contains("Writes"));
}

#[test]
fn the_journal_names_its_package() {
    let (mut a, ..) = written();
    let bs = "backspace ".repeat(10);
    let s = screen(
        &mut a,
        &format!("2 ] d {bs}2 0 2 6 - 1 2 - 3 1 enter / 5 0 2 0 enter enter enter"),
    );
    assert!(s.contains("written by"), "{s}");
    assert!(s.contains("XA:fixture-write-01:op-1"), "{s}");
    insta::assert_snapshot!("writes-journal-l3-100x30", s);
}

#[test]
fn the_notice_says_what_landed() {
    let (a, before, after) = written();
    let note = ownpurse_tui::external::describe(&a.source.record, None, before, after);
    assert!(
        note.starts_with("write: 1 op landed · XA:fixture-write-01"),
        "{note}"
    );
    assert!(ownpurse_tui::external::is_note(&note));
}

#[test]
fn enter_on_a_package_lists_its_ops_with_their_xero_links() {
    use ownpurse_tui::app::{Effect, Msg, update};
    use ownpurse_tui::keymap::Key;
    let (mut a, ..) = written();
    let p = &a.model.system.packages[0];
    let url = p.op_urls[0].clone().expect("the created journal's link");
    assert!(
        url.starts_with("https://go.xero.com/organisationlogin/default.aspx?shortcode=%21TmR4q&redirecturl=%2FJournal%2FView.aspx%3FinvoiceID%3D"),
        "{url}"
    );
    assert_eq!(p.op_states[0], "confirmed");
    let s = screen(&mut a, "5 / O w n e r enter");
    assert!(!s.contains("› op-1"), "folded: {s}");
    fn key(a: &mut App, k: &str) -> Vec<Effect> {
        let fx = update(a, Msg::Key(Key::parse(k).unwrap().event()));
        snapshot::render(a, 100, 30);
        fx
    }
    key(&mut a, "enter");
    key(&mut a, "j");
    let fx = key(&mut a, "o");
    assert_eq!(fx, vec![Effect::Open(url.clone())]);
    let fx = key(&mut a, "enter");
    assert_eq!(fx, vec![Effect::Open(url)]);
    let s = text(&snapshot::render(&mut a, 100, 30));
    assert!(
        s.contains("› op-1 create_journal") && s.contains("open in Xero"),
        "{s}"
    );
}

/// The agent bridge on System › Writes (v0.13.1 bug): `select` by the row's anchor (JSON or `key=value`)
/// finds the package row, and `drill` opens its operations (as Enter does) and says so; on a row where
/// drill means nothing it is refused, never a claimed change.
#[test]
fn agent_select_by_anchor_and_drill_open_a_packages_ops() {
    use ownpurse_tui::quiet::{Counting, Quiet};
    use ownpurse_tui::runtime::Runtime;
    use ratatui::layout::Rect;
    use ratatui::{Terminal, TerminalOptions, Viewport};
    use serde_json::{Value, json};
    let (a, ..) = written();
    let hash = a.model.system.packages[0].hash.clone();
    let backend = Quiet::fixed(Counting::default(), true, 100, 30);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 30)),
        },
    )
    .unwrap();
    let mut rt = Runtime::new(terminal, a);
    rt.frame().unwrap();
    let mut send = |verbs: Value| -> Value {
        rt.agent(&json!({"v": 1, "kind": "view", "op": "send", "verbs": verbs}))
    };
    let r = send(json!([
        ["tab", "system"],
        ["select", format!("system_item=pkg:{hash}")]
    ]));
    assert_eq!(r["ok"], true, "key=value anchor: {r}");
    let r = send(json!([[
        "select",
        json!({"system_item": format!("pkg:{hash}")}).to_string()
    ]]));
    assert_eq!(r["ok"], true, "JSON anchor: {r}");
    let r = send(json!([["drill"]]));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(
        r["changed"],
        json!(["listed the 1 operation of Owner-paid cupping supplies"]),
        "says what opened: {r}"
    );
    assert_eq!(r["summary"]["level"], 2, "{r}");
    assert!(
        r["summary"]["selected"]["system_item"]
            .as_str()
            .is_some_and(|k| k.starts_with("pkgop:")),
        "the first operation is selected (visible): {r}"
    );
    // drill on an operation is refused (it is a link: `open` opens it)
    let r = send(json!([["drill"]]));
    assert_eq!(r["error"], "not_applicable", "{r}");
    let r = send(json!([["open"]]));
    assert_eq!(r["ok"], true, "{r}");
    assert!(
        r["changed"][0]
            .as_str()
            .unwrap()
            .starts_with("opened https://go.xero.com/"),
        "{r}"
    );
    // up folds them, back on the package; up again has nothing to fold
    let r = send(json!([["up"]]));
    assert_eq!(r["summary"]["level"], 1, "{r}");
    assert_eq!(
        r["summary"]["selected"]["system_item"],
        format!("pkg:{hash}")
    );
    let r = send(json!([["up"]]));
    assert_eq!(r["error"], "not_applicable", "{r}");
    // drill on a settings row: refused, never a claimed change
    let r = send(json!([["select", "system_item=set:tui.editor"], ["drill"]]));
    assert_eq!(r["error"], "not_applicable", "{r}");
}
