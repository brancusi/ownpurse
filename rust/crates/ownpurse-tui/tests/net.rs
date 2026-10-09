//! Sync and login from the app (spec P6 network §7) against the fake CLI (`tests/fixtures/fake-cli`):
//! no network; records are written only as temp copies (the fake's `ownpurse-fake-record`, the api crate's
//! Writer). The runtime starts the real child process (setsid, stdin null) and its events come back through
//! a channel, as in the terminal loop.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ownpurse_tui::app::{App, BandKind, Msg, Overlay, update};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::net::{self, RunEvent};
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::run::ResultLevel;
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

type Rt = Runtime<Quiet<Counting>>;

fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!(
        "ownpurse-tui-net-{}-{name}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

struct Fake {
    dir: PathBuf,
}

impl Fake {
    fn argv(&self) -> PathBuf {
        self.dir.join("argv")
    }
    fn marker(&self) -> PathBuf {
        self.dir.join("marker")
    }
}

/// A runtime over a counting backend whose spawner runs the fake CLI with `scenario`.
fn runtime(app: App, scenario: &str, out: Counting) -> (Rt, Receiver<RunEvent>, Fake) {
    runtime_env(app, scenario, out, vec![])
}

/// `runtime`, on a temp copy of the record that the fake appends the run's rows to (as the real CLI).
fn runtime_record(mut app: App, scenario: &str) -> (Rt, Receiver<RunEvent>, Fake) {
    let record = snapshot::record_copy(&mut app);
    let design = app.model.orgs.iter().find(|o| o.alias == "design").unwrap();
    let env = vec![
        ("OWNPURSE_FAKE_DELAY".to_string(), "0".to_string()),
        ("OWNPURSE_FAKE_LEFT".to_string(), design.left.to_string()),
        (
            "OWNPURSE_FAKE_RECORD".to_string(),
            record.display().to_string(),
        ),
        (
            "OWNPURSE_FAKE_WRITER".to_string(),
            snapshot::fake_writer().display().to_string(),
        ),
        ("OWNPURSE_FAKE_TENANTS".to_string(), design.tenant.clone()),
    ];
    runtime_env(app, scenario, Counting::default(), env)
}

fn runtime_env(
    app: App,
    scenario: &str,
    out: Counting,
    extra: Vec<(String, String)>,
) -> (Rt, Receiver<RunEvent>, Fake) {
    let fake = Fake { dir: tmp(scenario) };
    let backend = Quiet::fixed(out, true, 100, 30);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 30)),
        },
    )
    .unwrap();
    let mut rt = Runtime::new(terminal, app);
    let (tx, rx) = channel();
    let mut envs = extra;
    envs.extend([
        ("OWNPURSE_FAKE_SCENARIO".to_string(), scenario.to_string()),
        (
            "OWNPURSE_FAKE_ARGV".to_string(),
            fake.argv().display().to_string(),
        ),
        (
            "OWNPURSE_FAKE_MARKER".to_string(),
            fake.marker().display().to_string(),
        ),
    ]);
    rt.spawner = Some(Box::new(move |program, args| {
        let tx = std::sync::Mutex::new(tx.clone());
        net::spawn_with_env(program, args, &envs, move |ev| {
            if let Ok(t) = tx.lock() {
                let _ = t.send(ev);
            }
        })
    }));
    rt.frame().unwrap();
    (rt, rx, fake)
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

/// Feed child events into the runtime until `done` holds (or 10 s pass). Returns the events seen.
fn pump(rt: &mut Rt, rx: &Receiver<RunEvent>, done: impl Fn(&Rt) -> bool) -> Vec<RunEvent> {
    let t = Instant::now();
    let mut seen = Vec::new();
    while !done(rt) && t.elapsed() < Duration::from_secs(60) {
        if let Ok(ev) = rx.recv_timeout(Duration::from_millis(50)) {
            seen.push(ev.clone());
            rt.send(Msg::Run(ev));
            rt.frame().unwrap();
        }
    }
    assert!(done(rt), "timed out; events: {seen:?}");
    seen
}

fn finished(rt: &Rt) -> bool {
    rt.app.ui.run.is_none() && rt.app.ui.result.is_some()
}

fn run_script_on(a: &mut App, keys: &str) {
    snapshot::run_script(a, keys, 140, 40);
}

fn app() -> App {
    fixture_app(theme::LEDGER_DARK, false)
}

fn wait_for(path: &Path) -> String {
    let t = Instant::now();
    while t.elapsed() < Duration::from_secs(60) {
        if let Ok(s) = std::fs::read_to_string(path)
            && !s.is_empty()
        {
            return s;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("{} never appeared", path.display());
}

// ── 1. V1 at compile time ───────────────────────────────────────────────────────────────────────

#[test]
fn a1_cargo_tree_has_no_api_crate() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let out = std::process::Command::new(cargo)
        .args([
            "tree",
            "--offline",
            "-p",
            "ownpurse-tui",
            "-e",
            "normal,build",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo tree");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let tree = String::from_utf8_lossy(&out.stdout);
    assert!(tree.contains("ownpurse-core"), "{tree}");
    assert!(
        !tree.contains("ownpurse-xero"),
        "the TUI must never link the api crate:\n{tree}"
    );
    assert!(
        !tree.contains("ureq") && !tree.contains("reqwest"),
        "no HTTP client in the TUI:\n{tree}"
    );
}

// ── 2. exact argv ───────────────────────────────────────────────────────────────────────────────

fn argv_lines(f: &Fake) -> Vec<Vec<String>> {
    std::fs::read_to_string(f.argv())
        .unwrap_or_default()
        .split("\n\n")
        .filter(|b| !b.trim().is_empty())
        .map(|b| b.lines().map(str::to_string).collect())
        .collect()
}

#[test]
fn a2_exact_argv_per_action() {
    let mut a = app();
    a.source.config_arg = Some(PathBuf::from("/w/ownpurse.json"));
    let (mut rt, rx, fake) = runtime(a, "ok", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, finished);
    key(&mut rt, "esc");
    // sync all (the fake replays the 4-org scenario regardless; only argv matters here)
    keys(&mut rt, "ctrl-p s y n c space a l l enter tab enter");
    pump(&mut rt, &rx, finished);
    key(&mut rt, "esc");
    // a group scope offers `Sync group:all-co…`
    keys(
        &mut rt,
        "] ] ] ] ] ctrl-p s y n c space g r o u p enter tab enter",
    );
    pump(&mut rt, &rx, finished);
    key(&mut rt, "esc");
    let tail = [
        "--profile",
        "demo",
        "--config",
        "/w/ownpurse.json",
        "--json",
        "--progress",
        "jsonl",
    ];
    let want = |head: &[&str]| -> Vec<String> {
        head.iter()
            .chain(tail.iter())
            .map(|s| s.to_string())
            .collect()
    };
    assert_eq!(
        argv_lines(&fake),
        vec![
            want(&["sync", "design"]),
            want(&["sync", "all"]),
            want(&["sync", "group:all-co"])
        ]
    );
    // login has no target, and no --config when the TUI found its config itself
    let (mut rt, rx, fake) = runtime(app(), "login", Counting::default());
    keys(&mut rt, "ctrl-p l o g enter tab enter");
    pump(&mut rt, &rx, finished);
    assert_eq!(
        argv_lines(&fake),
        vec![
            [
                "login",
                "--profile",
                "demo",
                "--json",
                "--progress",
                "jsonl"
            ]
            .map(String::from)
            .to_vec()
        ]
    );
    assert_eq!(
        rt.app.ui.result.as_ref().unwrap().lines,
        ["✓ Logged in · 4 orgs connected"]
    );
    assert_eq!(rt.reloads, 0, "login doesn't reload the record");
}

// ── 3. Esc never kills a run ────────────────────────────────────────────────────────────────────

#[test]
fn a3_esc_hides_the_band_and_the_run_completes() {
    let (mut rt, rx, fake) = runtime(app(), "slow", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    assert!(rt.app.ui.run.as_ref().unwrap().band_visible);
    pump(&mut rt, &rx, |rt| {
        rt.app.ui.run.as_ref().is_some_and(|r| r.calls > 0)
    });
    key(&mut rt, "esc");
    let r = rt.app.ui.run.as_ref().expect("still running");
    assert!(!r.band_visible, "Esc hides the band");
    let screen = snapshot::text(&snapshot::render(&mut rt.app, 100, 30));
    assert!(
        screen.lines().last().unwrap().contains("⇅ sync design"),
        "progress in the strip:\n{screen}"
    );
    // navigation keeps working mid-run
    keys(&mut rt, "2 ] enter");
    assert_eq!(rt.app.ledger_level(), 2);
    pump(&mut rt, &rx, finished);
    assert_eq!(
        wait_for(&fake.marker()).trim(),
        "done slow",
        "the child ran to completion"
    );
    let res = rt.app.ui.result.as_ref().unwrap();
    assert_eq!(res.level, ResultLevel::Ok);
    assert!(
        res.lines[0].starts_with("✓ Synced design · 15 calls · 25 records, 3 new versions · 0:0"),
        "{:?}",
        res.lines
    );
    assert!(res.lines[0].ends_with(" · record reloaded"));
}

// ── 4. frames per event, idle writes nothing ────────────────────────────────────────────────────

#[test]
fn a4_one_frame_per_event_and_zero_bytes_idle_after() {
    let out = Counting::default();
    let (mut rt, rx, _fake) = runtime(app(), "ok", out.clone());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    let t = Instant::now();
    let mut events = 0;
    while !finished(&rt) && t.elapsed() < Duration::from_secs(60) {
        if let Ok(ev) = rx.recv_timeout(Duration::from_millis(50)) {
            let draws = rt.draws;
            rt.send(Msg::Run(ev));
            rt.frame().unwrap();
            assert!(!rt.frame().unwrap(), "no second frame for one event");
            assert_eq!(
                rt.draws,
                draws + 1,
                "each progress event redraws exactly once"
            );
            events += 1;
        }
    }
    assert!(finished(&rt));
    assert_eq!(events, 18, "start, 15 entity calls, done, exit");
    // 10 s of idle after the result: ticks inside one minute write nothing
    let bytes = out.bytes();
    let minute = rt.app.now.div_euclid(60) * 60;
    rt.app.now = minute;
    for i in 0..20 {
        rt.send(Msg::Tick(minute + i / 2));
        assert!(!rt.frame().unwrap());
    }
    assert_eq!(out.bytes(), bytes, "idle after the result writes 0 bytes");
}

// ── 5. a refusal spawns nothing ─────────────────────────────────────────────────────────────────

#[test]
fn a5_refusals_spawn_nothing() {
    let mut a = app();
    a.model.orgs[2].left = 4;
    let (mut rt, _rx, fake) = runtime(a, "ok", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter");
    let b = rt.app.ui.band.clone().unwrap();
    assert_eq!(b.kind, BandKind::Refusal);
    assert_eq!(
        b.text,
        "Not enough calls left for design (needs ≈ 15, 4 left). Xero resets each org's daily limit at its own time."
    );
    assert_eq!(b.buttons(), vec!["Close".to_string()]);
    keys(&mut rt, "tab enter");
    assert_eq!(rt.spawns, 0);
    // a group is refused when any member would exceed its budget, naming that org
    keys(&mut rt, "] ] ] ] ] ctrl-p s y n c space g r o u p enter");
    assert!(
        rt.app
            .ui
            .band
            .as_ref()
            .unwrap()
            .text
            .contains("for design (needs ≈ 15, 4 left)")
    );
    key(&mut rt, "enter");
    // no CLI: the exact copy, Close only, nothing runs
    let mut a = app();
    a.cli = None;
    let (mut rt2, _rx2, _f2) = runtime(a, "ok", Counting::default());
    keys(&mut rt2, "ctrl-p s y n c space d e s i g n enter");
    assert_eq!(
        rt2.app.ui.band.as_ref().unwrap().text,
        "ownpurse isn't installed next to the app or on PATH. Run ./install.sh."
    );
    keys(&mut rt2, "tab enter");
    assert_eq!(rt2.spawns, 0);
    // Cancel (the default focus) spawns nothing either
    let (mut rt3, _rx3, _f3) = runtime(app(), "ok", Counting::default());
    keys(&mut rt3, "ctrl-p s y n c space d e s i g n enter enter");
    assert_eq!(rt3.spawns, 0);
    assert!(rt3.app.ui.band.is_none());
    assert_eq!(
        std::fs::read_to_string(fake.argv()).unwrap_or_default(),
        "",
        "the fake never ran"
    );
}

// ── 6. reload once, after completion ────────────────────────────────────────────────────────────

#[test]
fn a6_reload_exactly_once_after_the_run() {
    let (mut rt, rx, _fake) = runtime(app(), "slow", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, |rt| {
        rt.app.ui.run.as_ref().is_some_and(|r| r.calls >= 9)
    });
    assert_eq!(rt.reloads, 0, "never mid-run");
    pump(&mut rt, &rx, finished);
    assert_eq!(rt.reloads, 1, "once, after completion");
    // a failure reloads too (partial fetches are kept); a failed start doesn't
    let (mut rt, rx, _fake) = runtime(app(), "limit", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, finished);
    assert_eq!(rt.reloads, 1);
    let res = rt.app.ui.result.clone().unwrap();
    assert_eq!(res.level, ResultLevel::Warn);
    assert_eq!(
        res.lines,
        [
            "✗ error: daily_limit · Xero daily call limit reached for this organisation. · record reloaded",
            "Use cached data (`--offline`).",
            "Xero resets each org's daily limit at its own time; `ownpurse usage` shows what's left."
        ]
    );
    let mut a = app();
    a.cli = Some(PathBuf::from("/nonexistent/ownpurse"));
    let (mut rt, _rx, _fake) = runtime(a, "ok", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    let res = rt.app.ui.result.clone().unwrap();
    assert!(
        res.lines[0].starts_with("✗ Couldn't start ownpurse: "),
        "{:?}",
        res.lines
    );
    assert_eq!(rt.reloads, 0);
}

// ── 7. two runs can't overlap ───────────────────────────────────────────────────────────────────

#[test]
fn a7_one_run_at_a_time() {
    let (mut rt, rx, _fake) = runtime(app(), "slow", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    assert_eq!(rt.spawns, 1);
    let net: Vec<_> = rt
        .app
        .palette_entries()
        .into_iter()
        .filter(|e| e.network)
        .collect();
    assert!(!net.is_empty());
    for e in &net {
        assert_eq!(
            e.suffix.as_deref(),
            Some("· running: sync design"),
            "{}",
            e.label
        );
    }
    keys(&mut rt, "ctrl-p s y n c space r o a s t enter");
    assert!(rt.app.ui.band.is_none(), "no confirm while running");
    assert_eq!(rt.app.ui.note.as_deref(), Some("running: sync design"));
    assert_eq!(rt.spawns, 1);
    assert!(!matches!(rt.app.ui.overlay, Some(Overlay::Palette { .. })));
    pump(&mut rt, &rx, finished);
}

// ── 8. quitting leaves the child running to completion ──────────────────────────────────────────

#[test]
fn a8_quit_while_running_lets_the_child_finish() {
    let (mut rt, rx, fake) = runtime(app(), "slow", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, |rt| {
        rt.app.ui.run.as_ref().is_some_and(|r| r.calls > 0)
    });
    key(&mut rt, "q");
    let b = rt.app.ui.band.clone().unwrap();
    assert_eq!(
        b.text,
        "Sync design is still running. Quit the app and let it finish in the background?"
    );
    assert_eq!(b.focus, 0, "focus on Cancel");
    assert_eq!(
        b.buttons(),
        vec!["Cancel".to_string(), "Quit, let it finish".to_string()]
    );
    key(&mut rt, "enter");
    assert!(!rt.quit, "Enter on Cancel stays");
    keys(&mut rt, "q tab enter");
    assert!(rt.quit, "quit");
    drop(rt);
    drop(rx); // the app is gone: nobody reads the child's output any more
    assert!(!fake.marker().exists() || wait_for(&fake.marker()).contains("slow"));
    assert_eq!(
        wait_for(&fake.marker()).trim(),
        "done slow",
        "the child completed on its own"
    );
}

// ── 9. NO_COLOR / ASCII glyphs ──────────────────────────────────────────────────────────────────

#[test]
fn a9_ascii_glyphs() {
    for (state, want, gone) in [
        ("net-running-band", "<> Syncing design", "⇅"),
        ("net-done-ok", "Synced design", "✓"),
        ("net-done-error", "x error: needs_login", "✗"),
        ("net-confirm-sync", "~ 15 calls", "≈"),
    ] {
        let st = snapshot::shell_states()
            .into_iter()
            .find(|s| s.name == state)
            .unwrap();
        let mut a = snapshot::state_app(&st, theme::NO_COLOR_ASCII, 100, 30);
        let t = snapshot::text(&snapshot::render(&mut a, 100, 30));
        assert!(t.contains(want), "{state}: {want}\n{t}");
        assert!(!t.contains(gone), "{state}: {gone}\n{t}");
    }
    let st = snapshot::shell_states()
        .into_iter()
        .find(|s| s.name == "net-running-band")
        .unwrap();
    let mut a = snapshot::state_app(&st, theme::NO_COLOR_ASCII, 100, 30);
    let row0 = snapshot::row_text(&snapshot::render(&mut a, 100, 30), 0);
    assert!(row0.contains("<> 929/1,000 left"), "header ⇅ → <>: {row0}");
}

// ── and the PM's additions ──────────────────────────────────────────────────────────────────────

#[cfg(unix)]
fn script(dir: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let p = dir.join("ownpurse");
    std::fs::write(&p, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    p
}

#[cfg(unix)]
#[test]
fn sibling_beats_path_and_none_spawns_nothing() {
    let root = tmp("resolve");
    let app_dir = root.join("app");
    let path_dir = root.join("bin");
    std::fs::create_dir_all(&app_dir).unwrap();
    std::fs::create_dir_all(&path_dir).unwrap();
    let on_path = script(&path_dir, "echo ownpurse 0.0.1");
    let path = std::env::join_paths([&path_dir]).unwrap();
    assert_eq!(
        net::resolve_cli(Some(&app_dir), Some(&path)),
        Some(on_path),
        "PATH when no sibling"
    );
    let sibling = script(&app_dir, "echo ownpurse 0.9.0");
    assert_eq!(
        net::resolve_cli(Some(&app_dir), Some(&path)),
        Some(sibling.clone()),
        "a sibling wins over an earlier PATH entry"
    );
    assert_eq!(net::cli_version(&sibling).as_deref(), Some("0.9.0"));
    let empty = root.join("empty");
    std::fs::create_dir_all(&empty).unwrap();
    let empty_path = std::env::join_paths([&empty]).unwrap();
    assert_eq!(net::resolve_cli(Some(&empty), Some(&empty_path)), None);
}

#[test]
fn system_shows_the_resolved_cli_and_reads_its_version_once() {
    let mut a = app();
    a.cli_version = None;
    a.cli_shown = None; // the real resolved path (snapshots pin ~/.local/bin/ownpurse)
    let (mut rt, _rx, _f) = runtime(a, "ok", Counting::default());
    assert_eq!(rt.app.cli_version, None, "not read at start");
    key(&mut rt, "5");
    assert_eq!(
        rt.app.cli_version.as_deref(),
        Some("0.9.0"),
        "read once, when System is shown"
    );
    let t = snapshot::text(&snapshot::render(&mut rt.app, 140, 40));
    assert!(
        t.contains("cli: tests/fixtures/fake-cli/ownpurse (0.9.0)"),
        "{t}"
    );
    rt.app.cli_version = Some("cached".into());
    keys(&mut rt, "1 5");
    assert_eq!(
        rt.app.cli_version.as_deref(),
        Some("cached"),
        "never again on navigation"
    );
}

#[test]
fn result_band_dismissal_rules() {
    let (mut rt, rx, _f) = runtime(app(), "ok", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, finished);
    // Esc only dismisses
    key(&mut rt, "esc");
    assert!(rt.app.ui.result.is_none());
    assert_eq!(rt.app.ui.tab, ownpurse_tui::app::Tab::Overview);
    // the next key dismisses and also does its normal action
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, finished);
    let before = rt.app.overview_cursor();
    key(&mut rt, "j");
    assert!(rt.app.ui.result.is_none());
    assert_ne!(rt.app.overview_cursor(), before, "j moved the cursor too");
    // y copies the band text
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, finished);
    let copy = rt.app.ui.result.as_ref().unwrap().copy.clone();
    update(
        &mut rt.app,
        Msg::Key(KeyEvent::new(KeyCode::Char('y'), KeyModifiers::NONE)),
    );
    assert!(rt.app.ui.result.is_none());
    assert!(copy.starts_with("✓ Synced design"));
}

// ── post-run figures come from the reload ──────────────────────────────────────────────────

fn header_left(rt: &mut Rt) -> (String, ratatui::style::Color) {
    let buf = snapshot::render(&mut rt.app, 100, 30);
    let row = snapshot::row_text(&buf, 0);
    let x = row.find(" left").map_or(0, |i| {
        row[..i]
            .rfind(' ')
            .map_or(0, |j| row[..j + 1].chars().count())
    });
    (row, buf[(x as u16, 0)].fg)
}

#[test]
fn b1_calls_left_after_a_run_come_from_the_reloaded_record() {
    // scope design: the header shows design's own figure, as Xero last reported it
    let (mut rt, rx, _f) = runtime_record(app(), "ok");
    // the header shows the budget and the record age always here (`tui.view`, v0.12.0)
    rt.app.source.view.budget_in_header = "always";
    rt.app.source.view.record_age_in_header = "always";
    keys(&mut rt, "] ] ]");
    let before = rt.app.header_facts().left;
    assert_eq!(before, 938);
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, finished);
    assert_eq!(rt.reloads, 1);
    let f = rt.app.header_facts();
    assert_eq!(f.left, before - 15, "a 15-call run: the header drops by 15");
    assert!(!f.left_estimated, "Xero reported it (X-DayLimit-Remaining)");
    let (row, _) = header_left(&mut rt);
    assert!(
        row.contains("record 09:46 UTC · just now") && row.contains("923/1,000 left"),
        "{row}"
    );

    // the daily limit: the header shows what Xero reported (0), in the low-budget colour
    let (mut rt, rx, _f) = runtime_record(app(), "limit");
    keys(&mut rt, "] ] ]");
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, finished);
    let f = rt.app.header_facts();
    assert_eq!(f.left, 0);
    assert_eq!(f.quota_level, ownpurse_tui::model::Level::Err);
    let (row, fg) = header_left(&mut rt);
    assert!(row.contains("0/1,000 left"), "{row}");
    assert_eq!(
        Some(fg),
        theme::LEDGER_DARK
            .fg(ownpurse_tui::theme_tokens::Token::Error)
            .fg,
        "low-budget colour: {row}"
    );
}

// ── PM: a reported figure vs the local estimate ─────────────────────────────────────────────────

#[test]
fn calls_left_reported_vs_estimated_in_the_header_and_overview() {
    // lab never synced today: no X-DayLimit-Remaining, so 1,000 is the local estimate
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "never");
    let lab = a.model.orgs.iter().position(|o| o.alias == "lab").unwrap();
    let design = a
        .model
        .orgs
        .iter()
        .position(|o| o.alias == "design")
        .unwrap();
    assert!(!a.model.orgs[lab].left_known);
    assert!(a.model.orgs[design].left_known);
    a.source.view.budget_in_header = "always";
    a.ui.scope = ownpurse_tui::app::Scope::Org(lab);
    let row0 = snapshot::row_text(&snapshot::render(&mut a, 140, 40), 0);
    assert!(row0.contains("≈ 1,000 left (estimated)"), "{row0}");
    let row0 = snapshot::row_text(&snapshot::render(&mut a, 60, 20), 0);
    assert!(
        row0.contains("≈ 1,000"),
        "the short form keeps the mark: {row0}"
    );
    a.ui.scope = ownpurse_tui::app::Scope::Org(design);
    let row0 = snapshot::row_text(&snapshot::render(&mut a, 140, 40), 0);
    assert!(row0.contains("938/1,000 left"), "{row0}");
    assert!(!row0.contains("estimated") && !row0.contains('≈'), "{row0}");
    // the scope picker's detail and Activity's quota strip: the same rule
    a.ui.scope = ownpurse_tui::app::Scope::All;
    let picker = a
        .picker_entries("")
        .into_iter()
        .find(|e| e.label.contains("lab"))
        .unwrap();
    assert!(
        picker.detail.contains("≈ 1,000 left (estimated)"),
        "{}",
        picker.detail
    );
    let owner_pick = a
        .picker_entries("")
        .into_iter()
        .find(|e| e.label.contains("design"))
        .unwrap();
    assert!(
        owner_pick.detail.contains("938/1,000 left"),
        "{}",
        owner_pick.detail
    );
    a.ui.details = true; // the per-org counts are the details layer
    run_script_on(&mut a, "4");
    let t = snapshot::text(&snapshot::render(&mut a, 140, 40));
    let strip = t
        .lines()
        .find(|l| l.contains("lab") && l.contains("left of"))
        .unwrap_or_else(|| panic!("{t}"));
    assert!(strip.contains("≈ 1,000 (est.) left of 1,000"), "{strip}");
    let strip = t
        .lines()
        .find(|l| l.contains("design") && l.contains("left of"))
        .unwrap();
    assert!(
        strip.contains("938 left of 1,000") && !strip.contains("est."),
        "{strip}"
    );
    run_script_on(&mut a, "1");
    // Overview: the same marking per org
    a.ui.scope = ownpurse_tui::app::Scope::All;
    let t = snapshot::text(&snapshot::render(&mut a, 140, 40));
    let line = |alias: &str| {
        t.lines()
            .find(|l| l.trim_start_matches(['▌', ' ']).starts_with(alias))
            .unwrap_or_else(|| panic!("{alias}:\n{t}"))
            .to_string()
    };
    assert!(line("lab").contains("≈ 1,000 (estimated)"), "{t}");
    assert!(line("design").contains("938/1,000"), "{t}");
    assert!(!line("design").contains("estimated"), "{t}");
    // the refusal names the estimate when it is one
    a.model.orgs[lab].left = 4;
    a.request_network(ownpurse_tui::app::NetAction::SyncOrg(lab));
    assert_eq!(
        a.ui.band.as_ref().unwrap().text,
        "Not enough calls left for lab (needs ≈ 15, ≈ 4 left, estimated). Xero resets each org's daily limit at its own time."
    );
}

// ── disabled rows, run segments ───────────────────────────────────────────────────────────────────────────

#[test]
fn p1_disabled_palette_rows_are_muted_whole() {
    let st = snapshot::shell_states()
        .into_iter()
        .find(|s| s.name == "net-running-palette")
        .unwrap();
    let mut a = snapshot::state_app(&st, theme::LEDGER_DARK, 100, 30);
    let buf = snapshot::render(&mut a, 100, 30);
    let muted = theme::LEDGER_DARK
        .fg(ownpurse_tui::theme_tokens::Token::Muted)
        .fg;
    let mut checked = 0;
    for y in 0..30u16 {
        let row = snapshot::row_text(&buf, y);
        let Some(i) = row.find("⇅ Sync roast…") else {
            continue;
        };
        let x0 = row[..i].chars().count() as u16;
        for x in x0..x0 + 12 {
            assert_eq!(Some(buf[(x, y)].fg), muted, "{row}");
            checked += 1;
        }
    }
    assert!(checked > 0, "found the row");
}

#[test]
fn f1_never_two_figures_for_one_org() {
    let one_figure = |row0: &str, live: i64| {
        let want = format!("⇅ {}/1,000 left", ownpurse_tui::clock::int(live));
        assert!(row0.contains(&want), "{want}: {row0}");
        assert_eq!(row0.matches('⇅').count(), 1, "{row0}");
        assert!(!row0.contains("⇅ design"), "{row0}");
        assert!(
            !row0.contains("938/1,000"),
            "the stale figure is gone: {row0}"
        );
    };
    // scope all while design, the lowest org, syncs: design's live figure in place, with ⇅
    let (mut rt, rx, _f) = runtime(app(), "slow", Counting::default());
    keys(&mut rt, "ctrl-p s y n c space d e s i g n enter tab enter");
    pump(&mut rt, &rx, |rt| {
        rt.app.ui.run.as_ref().is_some_and(|r| r.calls >= 2)
    });
    let live = rt.app.ui.run.as_ref().unwrap().day_remaining.unwrap();
    one_figure(
        &snapshot::row_text(&snapshot::render(&mut rt.app, 140, 40), 0),
        live,
    );
    pump(&mut rt, &rx, finished);
    // scope design, syncing design: the same
    let (mut rt, rx, _f) = runtime(app(), "slow", Counting::default());
    keys(
        &mut rt,
        "] ] ] ctrl-p s y n c space d e s i g n enter tab enter",
    );
    pump(&mut rt, &rx, |rt| {
        rt.app.ui.run.as_ref().is_some_and(|r| r.calls >= 2)
    });
    let live = rt.app.ui.run.as_ref().unwrap().day_remaining.unwrap();
    one_figure(
        &snapshot::row_text(&snapshot::render(&mut rt.app, 140, 40), 0),
        live,
    );
    pump(&mut rt, &rx, finished);
    // scope all while roast syncs (967 left; design's 938 is the figure shown): roast gets its own segment
    let (mut rt, rx, _f) = runtime_env(
        app(),
        "slow",
        Counting::default(),
        vec![("OWNPURSE_FAKE_LEFT".into(), "967".into())],
    );
    rt.app.source.view.budget_in_header = "always";
    keys(&mut rt, "ctrl-p s y n c space r o a s t enter tab enter");
    pump(&mut rt, &rx, |rt| {
        rt.app.ui.run.as_ref().is_some_and(|r| r.calls >= 2)
    });
    let live = rt.app.ui.run.as_ref().unwrap().day_remaining.unwrap();
    let row0 = snapshot::row_text(&snapshot::render(&mut rt.app, 140, 40), 0);
    assert!(
        row0.contains(&format!(
            "938/1,000 left    ⇅ roast · {}/1,000 left",
            ownpurse_tui::clock::int(live)
        )),
        "{row0}"
    );
    pump(&mut rt, &rx, finished);
}
