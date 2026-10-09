//! The in-place upgrade (v0.10.3, brief §6). The handover runs exactly as the terminal loop runs it
//! (`upgrade::handover`), with a fake `Handover` for the terminal side: its `exec` plays the NEW process
//! (reads the `--resume` file and rehydrates with `upgrade::resume`, as the binary does) and then fails,
//! or just fails. The pre-flight runs a real binary: this crate's `ownpurse-tui` as the new build, or a
//! fake one that refuses `--check-state`. Nothing here uses the installed ~/.local/bin.

use ownpurse_tui::app::{App, Msg};
use ownpurse_tui::canonical::Update;
use ownpurse_tui::keymap::Key;
use ownpurse_tui::net::{self, RunEvent};
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme;
use ownpurse_tui::upgrade::{self, Ask, Handover, Outcome, Plan};
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

type Rt = Runtime<Quiet<Counting>>;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "xab-up-{}-{name}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

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

fn ready(a: &mut App) {
    a.update_available = Some(Update {
        version: "0.10.4".into(),
        action: upgrade::ACTION.into(),
    });
}

fn keys(rt: &mut Rt, ks: &str) {
    for k in ks.split_whitespace() {
        rt.send(Msg::Key(Key::parse(k).unwrap().event()));
        rt.frame().unwrap();
    }
}

/// The new build: this crate's binary, with the demo config the fixture app reads.
fn new_build() -> (PathBuf, Vec<String>) {
    let cfg = snapshot::fixtures_dir().join("demo/ownpurse.json");
    (
        PathBuf::from(env!("CARGO_BIN_EXE_ownpurse-tui")),
        vec![
            "--config".into(),
            cfg.display().to_string(),
            "--profile".into(),
            "demo".into(),
        ],
    )
}

fn plan(program: PathBuf, args: Vec<String>, dir: &Path) -> Plan {
    Plan {
        program,
        args,
        dir: dir.to_path_buf(),
        reply_fd: None,
        env: vec![],
        preflight_timeout: SLOW_MACHINE,
    }
}

/// Tests run beside other builds: they assert reasons, never timing, so a pre-flight gets a minute.
const SLOW_MACHINE: std::time::Duration = std::time::Duration::from_secs(60);

fn preflight(program: &Path, args: &[String], f: &Path) -> Result<(), String> {
    upgrade::preflight_within(program, args, f, SLOW_MACHINE)
}

/// The terminal side, faked. `exec` plays the new process when `resume` is set.
#[derive(Default)]
struct Fake {
    calls: Vec<String>,
    exec_args: Vec<String>,
    resume: bool,
    resumed: Option<(App, String)>,
    resume_file_gone: bool,
}

impl Handover for Fake {
    fn release(&mut self) -> std::io::Result<()> {
        self.calls.push("release".into());
        Ok(())
    }
    fn exec(
        &mut self,
        _program: &Path,
        args: &[String],
        _env: &[(String, String)],
    ) -> std::io::Error {
        self.calls.push("exec".into());
        self.exec_args = args.to_vec();
        if self.resume {
            // what `ownpurse-tui … --resume FILE` does: read, delete, rehydrate
            let i = args.iter().position(|a| a == "--resume").unwrap();
            let file = PathBuf::from(&args[i + 1]);
            let text = std::fs::read_to_string(&file).unwrap();
            std::fs::remove_file(&file).unwrap();
            self.resume_file_gone = !file.exists();
            let fresh = fixture_app(theme::LEDGER_DARK, false);
            let (app, from) = upgrade::resume(&text, fresh.source.clone(), &fresh)
                .expect("the new build resumes");
            self.resumed = Some((app, from));
        }
        std::io::Error::other("simulated: exec replaced by the test")
    }
    fn reenter(&mut self) -> std::io::Result<()> {
        self.calls.push("reenter".into());
        Ok(())
    }
}

fn files_in(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .collect()
}

/// Every row but the footer (it carries the note and, after the upgrade, the new version).
fn body(buf: &ratatui::buffer::Buffer) -> Vec<String> {
    let h = buf.area.height;
    (0..h - 1).map(|y| snapshot::row_text(buf, y)).collect()
}

// ── §6: mid-typing, overlays, an active filter, L2/L3 drills ────────────────────────────────────────

#[test]
fn an_upgrade_restores_the_exact_view_mid_typing_included() {
    for script in [
        "2 ] / A c m",                           // typing a filter
        "2 ] / A c m e left left",               // …with the caret moved
        "ctrl-p s y n",                          // the palette open, mid-query
        "g d e s",                               // the scope picker, mid-query
        "2 ] / T a l l o w m e r e space enter", // a filter active
        "2 ] j enter",                           // Ledger L2
        "2 ] j enter enter",                     // Ledger L3
        "3 enter",                               // Intercompany drill
        "?",                                     // help open
    ] {
        let mut a = fixture_app(theme::LEDGER_DARK, false);
        ready(&mut a);
        let mut rt = rt_with(a, Counting::default());
        keys(&mut rt, script);
        let live_ui = rt.app.ui.clone();
        let live = rt.current_frame();
        assert_eq!(rt.upgrade_ask(), Ask::Go("0.10.4".into()), "{script}");
        let dir = tmp("restore");
        let (program, args) = new_build();
        let mut fake = Fake {
            resume: true,
            ..Fake::default()
        };
        let o = upgrade::handover(&mut rt, &plan(program, args.clone(), &dir), &mut fake);
        assert!(matches!(o, Outcome::ExecFailed(_)), "{script}: {o:?}");
        assert_eq!(fake.calls, ["release", "exec", "reenter"], "{script}");
        // exec got the start-up args plus --resume FILE
        assert_eq!(&fake.exec_args[..args.len()], &args[..]);
        assert_eq!(fake.exec_args[args.len()], "--resume");
        assert!(
            fake.resume_file_gone,
            "the new build deletes the state file"
        );
        let (mut b, from) = fake.resumed.take().unwrap();
        assert_eq!(from, snapshot::TEST_VERSION);
        // exactly where they were: the whole Ui, but the one-line `updated to vX` note
        let mut want = live_ui.clone();
        want.note = Some(upgrade::updated_note(&from, &b.version_label));
        assert_eq!(b.ui, want, "{script}");
        assert_eq!(b.update_available, None, "the new build is the update");
        let got = snapshot::render(&mut b, 100, 30);
        assert_eq!(body(&got), body(&live), "{script}: the same frame");
        assert!(files_in(&dir).is_empty(), "{script}: no state file left");
    }
}

#[test]
fn the_state_file_is_private_and_carries_the_live_frame() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    ready(&mut a);
    let rt = rt_with(a, Counting::default());
    let dir = tmp("private");
    let f = upgrade::export(&rt.app, &rt.current_frame(), &dir).unwrap();
    use std::os::unix::fs::PermissionsExt;
    assert_eq!(
        std::fs::metadata(&f).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert_eq!(
        std::fs::metadata(&dir).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let v: Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
    assert!(v["frame"].as_str().unwrap().contains("read-only"));
    assert!(v["state"]["canonical"].is_object());
    // the pre-flight with the real new build passes on it
    let (program, args) = new_build();
    preflight(&program, &args, &f).unwrap();
}

// ── §4c: a failed pre-flight changes nothing ───────────────────────────────────────────────────────

fn fake_build(dir: &Path, body: &str) -> PathBuf {
    let p = dir.join("ownpurse-tui-new");
    std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    p
}

#[test]
fn a_failed_preflight_keeps_the_old_process_unchanged() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    ready(&mut a);
    let mut rt = rt_with(a, Counting::default());
    keys(&mut rt, "2 ] / A c m");
    let ui = rt.app.ui.clone();
    let rev = rt.app.rev;
    let bin = tmp("fakebin");
    let program = fake_build(
        &bin,
        "echo 'error: preflight_failed'; echo 'message: record_changed: the record moved'; exit 1",
    );
    let dir = tmp("pf");
    let mut fake = Fake::default();
    let o = upgrade::handover(&mut rt, &plan(program, vec![], &dir), &mut fake);
    assert_eq!(
        o,
        Outcome::PreflightFailed("record_changed: the record moved".into())
    );
    assert!(fake.calls.is_empty(), "the terminal was never touched");
    assert!(files_in(&dir).is_empty(), "the state file is deleted");
    let mut want = ui.clone();
    want.note = Some("Upgrade not started: record changed · r reload, then retry".into());
    assert_eq!(rt.app.ui, want, "only the note says why");
    assert_eq!(rt.app.rev, rev);
    // the agent's answer names it
    let d = o.doc("0.10.2", "0.10.4");
    assert_eq!(
        (d["ok"].clone(), d["error"].clone(), d["exit"].clone()),
        (json!(false), json!("preflight_failed"), json!(1))
    );
    assert_eq!(d["reason"], "record_changed: the record moved");
    // a build that doesn't know --check-state, or hangs, fails it too
    let o = upgrade::handover(
        &mut rt,
        &plan(
            fake_build(&bin, "echo 'unknown argument --check-state' >&2; exit 2"),
            vec![],
            &dir,
        ),
        &mut fake,
    );
    assert_eq!(
        o,
        Outcome::PreflightFailed("unknown argument --check-state".into())
    );
    assert!(fake.calls.is_empty());
}

#[test]
fn the_preflight_reloads_data_drift_and_refuses_config_drift() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    ready(&mut a);
    let mut rt = rt_with(a, Counting::default());
    keys(&mut rt, "2 ]");
    let dir = tmp("drift");
    let (program, args) = new_build();
    let f = upgrade::export(&rt.app, &rt.current_frame(), &dir).unwrap();
    let text = std::fs::read_to_string(&f).unwrap();
    // data drift (another process wrote between export and check): reloaded, never a failure. The record
    // version and the store revision are the data's; the frame (which shows the newer data) isn't compared.
    let mut v: Value = serde_json::from_str(&text).unwrap();
    v["state"]["canonical"]["record"]["chain_seq"] = json!(1);
    v["frame"] = json!("the frame of older data");
    std::fs::write(&f, v.to_string()).unwrap();
    preflight(&program, &args, &f).expect("record drift is not a reason to fail a live upgrade");
    let mut v: Value = serde_json::from_str(&text).unwrap();
    if v["state"]["canonical"]["record"]["classify_rev"].is_number() {
        v["state"]["canonical"]["record"]["classify_rev"] = json!(1);
        std::fs::write(&f, v.to_string()).unwrap();
        preflight(&program, &args, &f).expect("store drift is not a reason to fail a live upgrade");
    }
    // ...but a lost view field still is, data drift or not
    let mut v: Value = serde_json::from_str(&text).unwrap();
    v["state"]["canonical"]["record"]["chain_seq"] = json!(1);
    v["state"]["canonical"]["ui"]["a_field_this_build_drops"] = json!(true);
    std::fs::write(&f, v.to_string()).unwrap();
    let e = preflight(&program, &args, &f).unwrap_err();
    assert!(e.contains("would not survive"), "{e}");
    // config drift: an org the config doesn't have
    let mut v: Value = serde_json::from_str(&text).unwrap();
    v["state"]["canonical"]["config"]["orgs"]
        .as_array_mut()
        .unwrap()
        .push(json!(["gone", "00000000-0000-0000-0000-000000000000"]));
    v["state"]["canonical"]["config"]["hash"] = json!("0");
    std::fs::write(&f, v.to_string()).unwrap();
    let e = preflight(&program, &args, &f).unwrap_err();
    assert!(e.starts_with("config_changed: "), "{e}");
    // a schema this build can't migrate
    let mut v: Value = serde_json::from_str(&text).unwrap();
    v["state"]["canonical"]["v"] = json!(99);
    std::fs::write(&f, v.to_string()).unwrap();
    let e = preflight(&program, &args, &f).unwrap_err();
    assert!(e.contains("schema v99"), "{e}");
    // a frame that doesn't match
    let mut v: Value = serde_json::from_str(&text).unwrap();
    v["frame"] = json!("something else");
    std::fs::write(&f, v.to_string()).unwrap();
    let e = preflight(&program, &args, &f).unwrap_err();
    assert!(e.starts_with("the restored frame differs"), "{e}");
    // and the untouched file passes
    std::fs::write(&f, &text).unwrap();
    preflight(&program, &args, &f).unwrap();
}

// ── §4e: exec failure ──────────────────────────────────────────────────────────────────────────────

#[test]
fn an_exec_failure_takes_the_terminal_back_and_reports_it() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    ready(&mut a);
    let mut rt = rt_with(a, Counting::default());
    keys(&mut rt, "2 ] j enter");
    let ui = rt.app.ui.clone();
    let dir = tmp("exec");
    let (program, args) = new_build();
    let mut fake = Fake::default();
    let o = upgrade::handover(&mut rt, &plan(program, args, &dir), &mut fake);
    assert_eq!(
        o,
        Outcome::ExecFailed("simulated: exec replaced by the test".into())
    );
    assert_eq!(fake.calls, ["release", "exec", "reenter"]);
    assert!(files_in(&dir).is_empty());
    let mut want = ui;
    want.note = Some(format!(
        "Upgrade failed · still v{} · simulated: exec replaced by the test",
        rt.app.version_label
    ));
    assert_eq!(rt.app.ui, want);
    assert_eq!(o.doc("0.10.2", "0.10.4")["error"], "exec_failed");
    // the old build still offers the upgrade
    assert_eq!(rt.upgradable(), Some("0.10.4".into()));
}

// ── §4a: deferred while a sync runs (the fake CLI), applied after it ───────────────────────────────

fn runtime_fake_cli(app: App, scenario: &str) -> (Rt, Receiver<RunEvent>) {
    let mut rt = rt_with(app, Counting::default());
    let fake = tmp(scenario);
    let (tx, rx) = channel();
    let envs = vec![
        ("OWNPURSE_FAKE_SCENARIO".to_string(), scenario.to_string()),
        (
            "OWNPURSE_FAKE_ARGV".to_string(),
            fake.join("argv").display().to_string(),
        ),
        (
            "OWNPURSE_FAKE_MARKER".to_string(),
            fake.join("marker").display().to_string(),
        ),
    ];
    rt.spawner = Some(Box::new(move |program, args| {
        let tx = std::sync::Mutex::new(tx.clone());
        net::spawn_with_env(program, args, &envs, move |ev| {
            if let Ok(t) = tx.lock() {
                let _ = t.send(ev);
            }
        })
    }));
    (rt, rx)
}

fn pump(rt: &mut Rt, rx: &Receiver<RunEvent>, done: impl Fn(&Rt) -> bool) {
    let t = Instant::now();
    while !done(rt) && t.elapsed() < Duration::from_secs(60) {
        if let Ok(ev) = rx.recv_timeout(Duration::from_millis(50)) {
            rt.send(Msg::Run(ev));
            rt.frame().unwrap();
        }
    }
    assert!(done(rt), "timed out");
}

#[test]
fn an_upgrade_during_a_sync_is_deferred_then_applies_after_the_reload() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    ready(&mut a);
    let (mut rt, rx) = runtime_fake_cli(a, "slow");
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    assert!(rt.app.ui.run.is_some(), "the sync runs");
    // the agent asks (the palette asks the same way)
    let r = rt.agent(&json!({"v": 1, "kind": "upgrade", "op": "upgrade"}));
    assert_eq!(r["ok"], true, "{r}");
    assert_eq!(r["status"], "deferred");
    assert_eq!(r["to"], "0.10.4");
    assert!(!rt.upgrade_due);
    assert_eq!(
        rt.app.ui.note.as_deref(),
        Some("Upgrade will run when sync design finishes")
    );
    let reloads = rt.reloads;
    pump(&mut rt, &rx, |rt| rt.app.ui.run.is_none());
    assert_eq!(rt.reloads, reloads + 1, "the run's reload came first");
    assert!(rt.upgrade_due, "due by itself once the sync finished");
    // a second request while one is due doesn't stack anything odd
    assert!(!rt.upgrade_deferred);
}

#[test]
fn up_to_date_and_the_palette_entry() {
    let mut rt = rt_with(fixture_app(theme::LEDGER_DARK, false), Counting::default());
    let r = rt.agent(&json!({"v": 1, "kind": "upgrade", "op": "upgrade"}));
    assert_eq!(
        (r["ok"].clone(), r["status"].clone()),
        (json!(true), json!("up_to_date"))
    );
    assert!(!rt.upgrade_due);
    assert!(
        !rt.app
            .palette_entries()
            .iter()
            .any(|e| e.label.starts_with("Upgrade")),
        "no entry without an update"
    );
    // the restart state (a build without in-place upgrade) has no entry either
    rt.app.update_available = Some(Update {
        version: "0.10.4".into(),
        action: "restart".into(),
    });
    assert!(
        !rt.app
            .palette_entries()
            .iter()
            .any(|e| e.label.starts_with("Upgrade"))
    );
    assert_eq!(rt.upgrade_ask(), Ask::UpToDate);
    ready(&mut rt.app);
    let e = rt
        .app
        .palette_entries()
        .into_iter()
        .find(|e| e.label == "Upgrade to v0.10.4")
        .expect("the palette entry");
    assert!(!e.network && !e.disabled);
    // choosing it makes the handover due (the loop performs it)
    keys(&mut rt, "ctrl-p u p g r a d e enter");
    assert!(rt.upgrade_due);
}

// ── idle after the upgrade ─────────────────────────────────────────────────────────────────────────

#[test]
fn idle_writes_nothing_after_the_upgrade() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    ready(&mut a);
    let mut rt = rt_with(a, Counting::default());
    keys(&mut rt, "2 ] / A c");
    let dir = tmp("idle");
    let (program, args) = new_build();
    let mut fake = Fake {
        resume: true,
        ..Fake::default()
    };
    upgrade::handover(&mut rt, &plan(program, args, &dir), &mut fake);
    let (b, _) = fake.resumed.take().unwrap();
    let out = Counting::default();
    let mut rt = rt_with(b, out.clone());
    rt.frame().unwrap();
    let bytes = out.bytes();
    assert!(bytes > 0, "the first frame drew");
    let minute = rt.app.now.div_euclid(60) * 60;
    for s in [5, 20, 40] {
        rt.send(Msg::Tick(minute + s));
        rt.frame().unwrap();
    }
    assert_eq!(out.bytes(), bytes, "idle: zero bytes");
    // and the half-typed filter is still being typed
    assert_eq!(
        rt.app.ui.prompt,
        Some(ownpurse_tui::app::Prompt::Filter("Ac".into()))
    );
}

#[test]
fn base_args_drop_the_one_shot_flags() {
    let a = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
    assert_eq!(
        upgrade::base_args(a(
            "--config c.json --resume /x/up.json --resume-reply 7 --profile demo"
        )),
        a("--config c.json --profile demo")
    );
    assert_eq!(
        upgrade::base_args(a("--check-state f --now 2026-01-01T00:00Z")),
        a("--now 2026-01-01T00:00Z")
    );
}

#[test]
fn schema_versions_migrate_or_refuse_never_reset() {
    use ownpurse_tui::canonical::{STATE_V, migrate};
    let ok = migrate(json!({"v": STATE_V, "x": 1})).unwrap();
    assert_eq!(ok, json!({"v": STATE_V, "x": 1}));
    for (doc, says) in [
        (json!({"v": 0}), "can't be migrated"),
        (json!({"v": STATE_V + 1}), "newer build"),
        (json!({"x": 1}), "no schema version"),
    ] {
        let e = migrate(doc.clone()).unwrap_err();
        assert_eq!(e.exit(), 2, "{doc}");
        assert!(e.message().contains(says), "{doc}: {}", e.message());
    }
}

// ── v0.11.1: the pre-flight across versions that change the UI ───────────────────────────────────────

/// What a v0.10.4 TUI hands over: its state (the frozen v0.10.4 fixture) and its live frame, rendered by the
/// real v0.10.4 binary (`tests/fixtures/upgrade/v0.10.4/*.ans`).
fn v0104_handover(name: &str) -> Value {
    let dir = snapshot::fixtures_dir();
    let state: Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join(format!("state/v0.10.4/{name}.json"))).unwrap(),
    )
    .unwrap();
    let frame = std::fs::read_to_string(dir.join(format!("upgrade/v0.10.4/{name}.ans"))).unwrap();
    json!({"state": state, "frame": frame})
}

fn v0104_names() -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(snapshot::fixtures_dir().join("upgrade/v0.10.4"))
        .unwrap()
        .flatten()
        .filter_map(|e| {
            e.file_name()
                .to_str()?
                .strip_suffix(".ans")
                .map(String::from)
        })
        .collect();
    v.sort();
    v
}

#[test]
fn a_v0104_state_upgrades_to_this_build_which_appends_a_tab() {
    let names = v0104_names();
    assert!(names.len() >= 6, "{names:?}");
    for name in &names {
        let doc = v0104_handover(name);
        let text = doc.to_string();
        upgrade::check(&text, snapshot::fixture_source()).unwrap_or_else(|e| panic!("{name}: {e}"));
        // the old tab bar really is shorter: the allowance is what lets it through
        let old_tabs =
            snapshot::strip_sgr(doc["frame"].as_str().unwrap().split('\n').nth(1).unwrap());
        assert!(!old_tabs.contains("6 "), "{name}: {old_tabs}");
    }
    // and through the real binary, as the old build runs it (the fake-new-binary harness stays below)
    let dir = tmp("v0104");
    let f = dir.join("handover.json");
    std::fs::write(&f, v0104_handover("filter-half-typed").to_string()).unwrap();
    let (program, args) = new_build();
    preflight(&program, &args, &f).unwrap();
}

#[test]
fn a_lost_field_fails_the_preflight() {
    // a field the new build doesn't carry (as if it dropped one): the state would not survive
    let mut doc = v0104_handover("filter-half-typed");
    doc["state"]["canonical"]["ui"]["zz_dropped_field"] = json!("kept by the old build");
    let e = upgrade::check(&doc.to_string(), snapshot::fixture_source()).unwrap_err();
    assert_eq!(
        e,
        "the view state would not survive the upgrade (canonical.ui.zz_dropped_field is lost or changed)"
    );
    // a value the import would change is lost too
    assert_eq!(
        upgrade::lost(
            &json!({"a": {"b": [1, 2]}}),
            &json!({"a": {"b": [1, 3], "new": 0}}),
            "canonical"
        ),
        Some("canonical.a.b[1]".into())
    );
    assert_eq!(
        upgrade::lost(
            &json!({"a": 1}),
            &json!({"a": 1, "added": true}),
            "canonical"
        ),
        None,
        "a newer build may add fields"
    );
}

/// What THIS build hands over (same render epoch): its exported state and live frame.
fn this_build_handover(script: &str) -> Value {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    ready(&mut a);
    let mut rt = rt_with(a, Counting::default());
    keys(&mut rt, script);
    let frame = rt.current_frame();
    let dir = tmp("same-epoch");
    let f = upgrade::export(&rt.app, &frame, &dir).unwrap();
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
    let _ = std::fs::remove_file(&f);
    doc
}

#[test]
fn a_frame_difference_beyond_appended_tabs_fails_strictly() {
    // within one render epoch the frame rule is unchanged (v0.12.0 render epoch, same-epoch case)
    let doc = this_build_handover("2 ] / T a l l o w m e r e space enter");
    assert_eq!(
        upgrade::check(&doc.to_string(), snapshot::fixture_source()),
        Ok(None)
    );
    let frame = doc["frame"].as_str().unwrap().to_string();
    let mut rows: Vec<String> = frame.split('\n').map(String::from).collect();
    // a body row differs (as if a newer build moved something): strict, the upgrade aborts
    rows[5] = format!("{}x", rows[5]);
    let mut changed = doc.clone();
    changed["frame"] = json!(rows.join("\n"));
    let e = upgrade::check(&changed.to_string(), snapshot::fixture_source()).unwrap_err();
    assert_eq!(
        e,
        "the restored frame differs from the live one (row 6: the row differs)"
    );
    // the tab bar may only append: a renamed old tab fails
    let mut rows: Vec<String> = frame.split('\n').map(String::from).collect();
    rows[1] = rows[1].replace("Ledger", "Ledgr ");
    let mut renamed = doc.clone();
    renamed["frame"] = json!(rows.join("\n"));
    let e = upgrade::check(&renamed.to_string(), snapshot::fixture_source()).unwrap_err();
    assert_eq!(
        e,
        "the restored frame differs from the live one (row 2: the tab bar changed beyond appended tabs)"
    );
}

// ── v0.12.0: the render epoch ──────────────────────────────────────────────────────────────────────

/// The frozen v0.11.1 states (epoch 0) with their frames: what a v0.11.1 TUI hands over.
fn v0111_handovers() -> Vec<(String, Value)> {
    let dir = snapshot::fixtures_dir().join("state/v0.11.1");
    let mut v: Vec<(String, Value)> = std::fs::read_dir(&dir)
        .unwrap()
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            (p.extension()? == "json").then_some(p)
        })
        .map(|p| {
            let state: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
            let frame = std::fs::read_to_string(p.with_extension("txt")).unwrap();
            (
                p.file_stem().unwrap().to_string_lossy().to_string(),
                json!({"state": state, "frame": frame}),
            )
        })
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0));
    v
}

#[test]
fn an_epoch_0_state_upgrades_to_epoch_1_on_state_equivalence() {
    let all = v0111_handovers();
    assert!(all.len() >= 2, "frozen v0.11.1 states");
    for (name, doc) in &all {
        assert_eq!(upgrade::epoch_of(&doc["state"]["canonical"]), 0, "{name}");
        assert_eq!(
            upgrade::check(&doc.to_string(), snapshot::fixture_source()),
            Ok(Some((0, upgrade::RENDER_EPOCH))),
            "{name}"
        );
    }
    // the reply and the note say so
    let d = upgrade::upgraded_doc("0.11.1", "0.12.0", 7, Some((0, 1)));
    assert_eq!(d["render_epoch"], "changed (visual redesign)");
    assert_eq!(
        (d["render_epoch_from"].clone(), d["render_epoch_to"].clone()),
        (json!(0), json!(1))
    );
    assert!(
        upgrade::upgraded_doc("0.12.0", "0.12.1", 7, None)
            .get("render_epoch")
            .is_none()
    );
    assert_eq!(
        upgrade::updated_note_new_look("0.11.1", "0.12.0"),
        "updated v0.11.1 → v0.12.0 · new look · state kept"
    );
    // resume shows it
    let (_, doc) = &all[0];
    let fresh = fixture_app(theme::LEDGER_DARK, false);
    let (b, from) = upgrade::resume(&doc.to_string(), snapshot::fixture_source(), &fresh).unwrap();
    assert_eq!(
        b.ui.note.as_deref(),
        Some(upgrade::updated_note_new_look(&from, &b.version_label).as_str())
    );
}

#[test]
fn a_lost_field_still_fails_across_epochs() {
    let (_, mut doc) = v0111_handovers().remove(0);
    doc["state"]["canonical"]["ui"]["zz_dropped_field"] = json!("kept by the old build");
    let e = upgrade::check(&doc.to_string(), snapshot::fixture_source()).unwrap_err();
    assert!(
        e.starts_with("the view state would not survive the upgrade"),
        "{e}"
    );
}

#[test]
fn the_epoch_is_carried_and_derived() {
    assert_eq!(upgrade::epoch_of(&json!({"render_epoch": 1})), 1);
    assert_eq!(
        upgrade::epoch_of(&json!({"env": {"version_label": "0.11.1"}})),
        0
    );
    assert_eq!(
        upgrade::epoch_of(&json!({"env": {"version_label": "0.10.2"}})),
        0
    );
    assert_eq!(
        upgrade::epoch_of(&json!({"env": {"version_label": "0.12.0"}})),
        1
    );
    assert_eq!(upgrade::RENDER_EPOCH, 2);
    let a = fixture_app(theme::LEDGER_DARK, false);
    assert_eq!(
        a.canonical_value()["render_epoch"],
        json!(upgrade::RENDER_EPOCH)
    );
}

#[test]
fn a_hung_preflight_is_killed_at_its_timeout() {
    let bin = tmp("hung");
    let program = fake_build(&bin, "sleep 30");
    let dir = tmp("hung-state");
    let f = dir.join("state.json");
    std::fs::write(&f, "{}").unwrap();
    let e = upgrade::preflight_within(&program, &[], &f, std::time::Duration::from_millis(300))
        .unwrap_err();
    assert!(e.starts_with("the new build did not answer within"), "{e}");
}
