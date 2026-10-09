//! The snapshot harness (spec §10, §13): the fixture app with a pinned clock, key scripts
//! (`OWNPURSE_TUI_KEYS`-style: `2 ] ] enter`, `<click:x,y>`, `<dclick:x,y>`, `<wheel:down:3>`), the shell
//! states, and buffer → text (plain for guide renders, styled for review). Used by the snapshot tests,
//! the V5 property test, the benches and `scripts/tui-snapshots.sh`.

use crate::app::{App, Msg, NetAction, Scope};
use crate::hit::Hits;
use crate::keymap::Key;
use crate::model::{Model, Source};
use crate::theme::{self, Theme};
use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier};
use std::path::{Path, PathBuf};
use unicode_width::UnicodeWidthStr;

/// The pinned clock for the fixture: the record's syncs are at 07:46 UTC, so ages read `2h`.
pub const FIXTURE_NOW: &str = "2026-10-05T09:46Z";

pub fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

/// The CLI fixture config (`tests/fixtures/cli/ownpurse.json`, profile `fixture`): orgs one, hq, co.
pub fn cli_config_path() -> PathBuf {
    fixtures_dir().join("cli/ownpurse.json")
}

/// The CLI fixture `Source` (core's loader, empty environment).
pub fn cli_source() -> Source {
    let path = cli_config_path();
    let env = ownpurse_core::config::Env {
        vars: Default::default(),
        cwd: fixtures_dir().join("cli"),
        home: PathBuf::from("/nonexistent"),
    };
    let cfg = ownpurse_core::config::load(&env, Some("fixture"), Some(&path))
        .expect("cli fixture config");
    let mut s = Source::from_config(&cfg);
    // no manifest store in the fixture folders (tests that need one use the demo copy)
    s.manifests = None;
    s
}

/// The CLI fixture under another profile (`wide` adds eur, a different base currency, and june).
pub fn cli_app_profile(theme: Theme, profile: &str) -> App {
    let path = cli_config_path();
    let env = ownpurse_core::config::Env {
        vars: Default::default(),
        cwd: fixtures_dir().join("cli"),
        home: PathBuf::from("/nonexistent"),
    };
    let cfg =
        ownpurse_core::config::load(&env, Some(profile), Some(&path)).expect("cli fixture config");
    let mut source = Source::from_config(&cfg);
    source.manifests = None;
    let now = fixture_now();
    let model = Model::open(&source, now).expect("cli record");
    pinned(App::new(model, source, theme, false, now))
}

/// The demo's intercompany variant (`demo/ownpurse-ic.json` + `record-ic.sqlite`): the four demo pairs plus
/// `design-roast-proofs` (unexplained residual 1,412.57) and `roast-rhein` (not supported: different base currencies).
pub fn demo_ic_app(theme: Theme, offline: bool) -> App {
    let dir = demo_dir();
    let env = ownpurse_core::config::Env {
        vars: Default::default(),
        cwd: dir.clone(),
        home: PathBuf::from("/nonexistent"),
    };
    let cfg = ownpurse_core::config::load(&env, Some("demo"), Some(&dir.join("ownpurse-ic.json")))
        .expect("demo ic config");
    let mut source = Source::from_config(&cfg);
    source.manifests = None;
    let now = fixture_now();
    let model = Model::open(&source, now).expect("demo ic record");
    pinned(App::new(model, source, theme, offline, now))
}

/// The snapshot and test fixture: the demo record (roast, hold, design, lab; groups all-co and ops).
pub fn fixture_source() -> Source {
    demo_source("fresh")
}

/// Paths shown relative to `rust/` with no home, so snapshots don't depend on where the repo is checked out.
/// The version label snapshot tests render, so a version bump changes no snapshot.
pub const TEST_VERSION: &str = "0.0.0-test";

/// The fake CLI the tests run (`tests/fixtures/fake-cli/ownpurse`; no network; it writes only to record
/// copies under the temp folder, through `ownpurse-fake-record`).
pub fn fake_cli() -> PathBuf {
    fixtures_dir().join("fake-cli").join("ownpurse")
}

pub fn pinned(mut app: App) -> App {
    app.version_label = TEST_VERSION.into();
    app.build_label = None;
    // never whatever `ownpurse` happens to be on this machine's PATH
    app.cli = Some(fake_cli());
    app.cli_version = Some("0.9.0".into());
    app.cli_shown = Some("~/.local/bin/ownpurse".into());
    app.paths = crate::system::PathBase {
        cwd: fixtures_dir().join("..").join(".."),
        home: None,
    };
    app
}

pub fn fixture_now() -> i64 {
    crate::clock::parse_utc(FIXTURE_NOW).expect("pinned clock")
}

pub fn fixture_app(theme: Theme, offline: bool) -> App {
    let source = fixture_source();
    let now = fixture_now();
    let model =
        Model::open(&source, now).expect("fixture record (rust/tests/fixtures/record.sqlite)");
    pinned(App::new(model, source, theme, offline, now))
}

/// The four theme modes of the snapshot grid.
pub fn themes() -> [(&'static str, Theme); 4] {
    [
        ("ledger-dark", theme::LEDGER_DARK),
        ("ledger-light", theme::LEDGER_LIGHT),
        ("ansi", theme::ANSI),
        ("nocolor-ascii", theme::NO_COLOR_ASCII),
    ]
}

/// Render the app at `w`×`h` into a fresh buffer and keep the frame's hit targets.
pub fn render(app: &mut App, w: u16, h: u16) -> Buffer {
    app.ui.size = (w, h);
    app.repaint();
    let area = Rect::new(0, 0, w, h);
    let mut buf = Buffer::empty(area);
    let hits: Hits = crate::ui::render(app, &mut buf, area);
    app.hits = hits;
    app.dirty = false;
    buf
}

fn mouse(kind: MouseEventKind, x: u16, y: u16) -> MouseEvent {
    MouseEvent {
        kind,
        column: x,
        row: y,
        modifiers: KeyModifiers::NONE,
    }
}

/// One script token → messages. Unknown tokens are an error (scripts are test inputs).
pub fn token_msgs(t: &str) -> Result<Vec<Msg>, String> {
    if let Some(inner) = t.strip_prefix('<').and_then(|r| r.strip_suffix('>')) {
        let (name, arg) = inner.split_once(':').ok_or(format!("bad token {t}"))?;
        let xy = |s: &str| -> Result<(u16, u16), String> {
            let (x, y) = s.split_once(',').ok_or(format!("bad xy {s}"))?;
            Ok((
                x.trim().parse().map_err(|_| s.to_string())?,
                y.trim().parse().map_err(|_| s.to_string())?,
            ))
        };
        return match name {
            "click" | "dclick" => {
                let (x, y) = xy(arg)?;
                Ok(vec![Msg::Mouse {
                    ev: mouse(MouseEventKind::Down(MouseButton::Left), x, y),
                    clicks: if name == "dclick" { 2 } else { 1 },
                }])
            }
            "wheel" => {
                let (dir, n) = arg.split_once(':').unwrap_or((arg, "1"));
                let kind = if dir == "up" {
                    MouseEventKind::ScrollUp
                } else {
                    MouseEventKind::ScrollDown
                };
                let n: usize = n.parse().map_err(|_| t.to_string())?;
                Ok((0..n)
                    .map(|_| Msg::Mouse {
                        ev: mouse(kind, 1, 6),
                        clicks: 0,
                    })
                    .collect())
            }
            "resize" => {
                let (w, h) = xy(arg)?;
                Ok(vec![Msg::Resize(w, h)])
            }
            _ => Err(format!("unknown token {t}")),
        };
    }
    let k = Key::parse(t).ok_or(format!("unknown key {t}"))?;
    Ok(vec![Msg::Key(k.event())])
}

/// Run a key script, rendering after every step so clicks hit the current frame's targets.
pub fn run_script(app: &mut App, script: &str, w: u16, h: u16) {
    render(app, w, h);
    for t in script.split_whitespace() {
        for m in token_msgs(t).unwrap_or_else(|e| panic!("{e}")) {
            // classification writes run (in memory for the demo, as the loop runs them); others don't
            let fx = crate::app::update(app, m);
            crate::classify::run_effects(app, fx);
            render(app, w, h);
        }
    }
}

/// A state's (theme name, size) filter.
pub type RendersIn = fn(&str, (u16, u16)) -> bool;

pub struct ShellState {
    pub name: &'static str,
    pub script: &'static str,
    pub offline: bool,
    pub setup: Option<fn(&mut App)>,
    pub sizes: &'static [(u16, u16)],
    /// Which (theme, size) pairs render (None = every theme at every size).
    pub only: Option<RendersIn>,
}

impl ShellState {
    /// Whether this state renders in `theme` at `size`.
    pub fn renders(&self, theme: &str, size: (u16, u16)) -> bool {
        self.only.is_none_or(|f| f(theme, size))
    }
}

/// The agent states: 100x30 in ledger-dark, ledger-light and nocolor-ascii;
/// 80x24 dark.
fn agent_grid(theme: &str, size: (u16, u16)) -> bool {
    match size {
        (100, 30) => theme != "ansi",
        (80, 24) => theme == "ledger-dark",
        _ => false,
    }
}

/// The version-footer state: 100x30 in ledger-dark, ledger-light and nocolor-ascii; 80x24 dark;
/// 60x20 in both ledger themes.
fn version_grid(theme: &str, size: (u16, u16)) -> bool {
    match size {
        (100, 30) => theme != "ansi",
        (80, 24) => theme == "ledger-dark",
        (60, 20) => theme == "ledger-dark" || theme == "ledger-light",
        _ => false,
    }
}

/// The caption states: 100x30 in ledger-dark, ledger-light and nocolor-ascii; 80x24
/// dark.
fn caption_grid(theme: &str, size: (u16, u16)) -> bool {
    match size {
        (100, 30) => theme != "ansi",
        (80, 24) => theme == "ledger-dark",
        _ => false,
    }
}

fn dark_80(theme: &str, size: (u16, u16)) -> bool {
    size == (80, 24) && theme == "ledger-dark"
}

/// A realistic agent caption on the demo world: the roaster's card spend (3 lines in 2025, 1,990.90 in all).
const CAPTION: &str = "Green-bean card spend, 2025 · 3 lines · 1,990.90";

/// The filtered-TB states (v0.10.1 item 5): 100x30 in ledger-dark, ledger-light and nocolor-ascii.
fn filter_grid(theme: &str, size: (u16, u16)) -> bool {
    size == (100, 30) && theme != "ansi"
}

/// An agent send without a runtime (the snapshot harness): parse, apply each verb through `update`, draw
/// between verbs (fresh row targets), announce and log, as `Runtime::agent` does.
pub fn agent_send(a: &mut App, verbs: &[&[&str]], w: u16, h: u16) {
    let _ = agent_send_result(a, verbs, w, h);
}

/// `agent_send`, returning what changed (or the refusal), as `Runtime::agent` answers a send.
pub fn agent_send_result(
    a: &mut App,
    verbs: &[&[&str]],
    w: u16,
    h: u16,
) -> Result<Vec<String>, (String, String)> {
    a.agent_seen();
    render(a, w, h);
    let before = crate::agent::ViewSnap::of(&a.ui);
    let mut parsed = Vec::new();
    for v in verbs {
        let words: Vec<String> = v.iter().map(|s| (*s).to_string()).collect();
        parsed.push(a.parse_verb(&words)?);
    }
    let _ = crate::app::update(a, Msg::AgentSend(parsed));
    render(a, w, h);
    let res = a.agent.batch.take().unwrap_or(Ok(vec![]));
    if let Ok(changes) = &res
        && !changes.is_empty()
    {
        a.agent_announce(changes.clone(), before);
    }
    let what: Vec<String> = verbs.iter().map(|v| v.join("=")).collect();
    a.agent_log(&format!("send {}", what.join(" ")));
    res
}

pub const GRID: &[(u16, u16)] = &[(60, 20), (80, 24), (100, 30), (140, 40)];
pub const SL: &[(u16, u16)] = &[(80, 24), (140, 40)];
/// The scope-accent states: the design review sizes.
pub const ACC: &[(u16, u16)] = &[(80, 24), (100, 30)];
/// Spec P6 §6: 100x30 (both themes) and 80x24 (dark, nocolor-ascii); the grid renders every mode.
pub const NET: &[(u16, u16)] = &[(80, 24), (100, 30)];

/// Feed a running child's stderr lines (the same JSON the fake CLI prints).
pub fn feed(a: &mut App, lines: &[&str]) {
    for l in lines {
        let _ = crate::app::update(a, Msg::Run(crate::net::RunEvent::Line((*l).to_string())));
    }
}

/// The child's exit.
pub fn exit(a: &mut App, code: i32, stdout: &str, elapsed_secs: u64) {
    let _ = crate::app::update(
        a,
        Msg::Run(crate::net::RunEvent::Exit {
            code: Some(code),
            stdout: stdout.to_string(),
            elapsed_secs,
        }),
    );
}

/// Point the app at a fresh copy of its record (and WAL) under the temp folder: the fake CLI appends a
/// run's rows there, never to a committed fixture.
pub fn record_copy(a: &mut App) -> PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static N: AtomicUsize = AtomicUsize::new(0);
    let dir = std::env::temp_dir().join(format!(
        "ownpurse-tui-record-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp record folder");
    let src = a.source.record.clone();
    for ext in ["", "-wal", "-shm"] {
        let mut from = src.clone().into_os_string();
        from.push(ext);
        let from = PathBuf::from(from);
        if from.exists() {
            std::fs::copy(&from, dir.join(format!("record.sqlite{ext}"))).expect("copy record");
        }
    }
    a.source.record = dir.join("record.sqlite");
    a.source.record.clone()
}

/// `ownpurse-fake-record` (ownpurse-xero's test helper; the TUI never links that crate): `$OWNPURSE_FAKE_WRITER`,
/// else next to this executable in cargo's target folder, else built once with cargo.
pub fn fake_writer() -> PathBuf {
    static W: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();
    W.get_or_init(|| {
        if let Some(p) = std::env::var_os("OWNPURSE_FAKE_WRITER") {
            return PathBuf::from(p);
        }
        let name = "ownpurse-fake-record";
        let exe = std::env::current_exe().unwrap_or_default();
        let find = || {
            exe.ancestors()
                .skip(1)
                .take(3)
                .map(|d| d.join(name))
                .find(|p| p.is_file())
        };
        if let Some(p) = find() {
            return p;
        }
        let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
        let mut cmd = std::process::Command::new(cargo);
        cmd.args(["build", "--quiet", "-p", "ownpurse-xero", "--bin", name])
            .current_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."));
        if exe.components().any(|c| c.as_os_str() == "release") {
            cmd.arg("--release");
        }
        let ok = cmd.status().is_ok_and(|s| s.success());
        find().filter(|_| ok).unwrap_or_else(|| {
            panic!(
                "{name} not found next to {}; run `cargo build -p ownpurse-xero --bin {name}`",
                exe.display()
            )
        })
    })
    .clone()
}

/// The runtime's reload (`Effect::Reload`): the model and books from the record, fresh.
pub fn reload(a: &mut App) {
    let m = Model::open(&a.source, a.now).expect("reload the record");
    let books = crate::books::Books::open(&a.source).ok();
    let _ = crate::app::update(a, Msg::Reloaded(Box::new((m, books))));
}

/// Confirm and run `sync design` with the fake CLI for real (`scenario`), on a copy of the record: its
/// progress events and exit go through `update`, and the record reloads when the app asks to (once).
pub fn net_run_fake(a: &mut App, scenario: &str) {
    let record = record_copy(a);
    // always the fake (renders build the app from the real binary's resolution; never run a real CLI here)
    a.cli = Some(fake_cli());
    run_script(a, "ctrl-p s y n c space d e s i g n enter tab", 100, 30);
    let fx = crate::app::update(a, Msg::Key(Key::parse("enter").expect("key").event()));
    let Some((program, args)) = fx.into_iter().find_map(|f| match f {
        crate::app::Effect::Spawn { program, args } => Some((program, args)),
        _ => None,
    }) else {
        panic!("Enter on Sync spawned nothing");
    };
    assert_eq!(program, fake_cli(), "snapshot runs use the fake CLI only");
    let design = a
        .model
        .orgs
        .iter()
        .find(|o| o.alias == "design")
        .expect("design");
    let out = std::process::Command::new(program)
        .args(args)
        .envs([
            ("OWNPURSE_FAKE_SCENARIO", scenario.to_string()),
            ("OWNPURSE_FAKE_DELAY", "0".into()),
            ("OWNPURSE_FAKE_LEFT", design.left.to_string()),
            ("OWNPURSE_FAKE_RECORD", record.display().to_string()),
            ("OWNPURSE_FAKE_WRITER", fake_writer().display().to_string()),
            ("OWNPURSE_FAKE_TENANTS", design.tenant.clone()),
            ("OWNPURSE_FAKE_AT", FIXTURE_NOW.replace('Z', ":00Z")),
        ])
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run the fake CLI");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    feed(a, &stderr.lines().collect::<Vec<_>>());
    let fx = crate::app::update(
        a,
        Msg::Run(crate::net::RunEvent::Exit {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            elapsed_secs: 21, // pinned: the real run's wall time would vary
        }),
    );
    if fx.contains(&crate::app::Effect::Reload) {
        reload(a);
    }
}

/// Confirm `Sync design…`, choose Sync, and feed the first events (optionally a minute-limit wait).
pub fn net_running(a: &mut App, wait: bool) {
    run_script(
        a,
        "ctrl-p s y n c space d e s i g n enter tab enter",
        100,
        30,
    );
    feed(
        a,
        &[
            r#"{"ev":"start","org":"design","entities":14,"estimate":15,"estimate_label":"≈ 15 calls"}"#,
            r#"{"ev":"entity","org":"design","entity":"BankTransactions","page":2,"calls":9,"day_remaining":929}"#,
        ],
    );
    if wait {
        feed(
            a,
            &[r#"{"ev":"wait","org":"design","reason":"minute_limit","seconds":4}"#],
        );
    }
}

/// Every shell state (spec §13 rows that exist in the shell; M10 placeholder bands).
pub fn shell_states() -> Vec<ShellState> {
    let agent = |name, setup| ShellState {
        name,
        script: "",
        offline: false,
        setup: Some(setup),
        sizes: NET,
        only: Some(agent_grid),
    };
    let s = |name, script, offline, setup, sizes| ShellState {
        name,
        script,
        offline,
        setup,
        sizes,
        only: None,
    };
    vec![
        s("overview", "", false, None, GRID),
        s("ledger", "2", false, None, GRID),
        s("ledger-roast", "2 ]", false, None, GRID),
        s("ledger-history", "2 ] ] ] h", false, None, SML),
        s("ledger-differs", "2 ] ] ] h !", false, None, SML),
        s(
            "ledger-not-compared",
            "2 ]",
            false,
            Some(|a: &mut App| a.ui.as_of = "2023-06-30".into()),
            SML,
        ),
        s(
            "ledger-c4-not-compared-year",
            "2 ]",
            false,
            Some(|a: &mut App| *a = demo_app_variant(a.ui.theme, a.offline, "noclose")),
            SML,
        ),
        s("ledger-l2", "2 ] enter", false, None, SML),
        s(
            "ledger-l2-range",
            "2 ] enter f backspace backspace backspace backspace backspace 0 6 - 0 1 enter",
            false,
            None,
            SML,
        ),
        s("ledger-l2-history", "2 ] h enter", false, None, SML),
        s("ledger-l3", "2 ] enter enter", false, None, SML),
        s(
            "ledger-never",
            "2 ] ] ] ]",
            false,
            Some(|a: &mut App| *a = demo_app_variant(a.ui.theme, a.offline, "never")),
            SML,
        ),
        s("ledger-m1-history", "right right enter", false, None, SML),
        s("ledger-m1-year", "j j right enter", false, None, SML),
        s("intercompany", "3", false, None, GRID),
        s("ic-sort-difference", "3 s", false, None, SML),
        s("ic-sort-pair", "3 s s", false, None, SML),
        s("ic-filter", "3 / s a l t", false, None, SML),
        s("ic-scope-org", "3 ] ] ]", false, None, SML),
        s(
            "ic-not-compared",
            "3",
            false,
            Some(|a: &mut App| {
                a.ui.as_of = "2025-06-30".into();
                a.invalidate();
            }),
            SML,
        ),
        s("ic-l2-bridged", "3 enter", false, None, SML),
        s("ic-l2-items", "3 enter j", false, None, SML),
        s("ic-l3", "3 enter enter", false, None, SML),
        s("ic-via-ledger", "3 enter a", false, None, SML),
        s(
            "ic-unexplained",
            "3",
            false,
            Some(|a: &mut App| *a = ic_app(a.ui.theme, a.offline)),
            SML,
        ),
        s(
            "ic-l2-unexplained",
            "3 enter",
            false,
            Some(|a: &mut App| *a = ic_app(a.ui.theme, a.offline)),
            SML,
        ),
        s(
            "ic-l2-known-items",
            "3 s s j j j j j enter",
            false,
            Some(|a: &mut App| *a = ic_app(a.ui.theme, a.offline)),
            SML,
        ),
        s(
            "ic-not-supported-refusal",
            "3 s s j enter",
            false,
            Some(|a: &mut App| *a = ic_app(a.ui.theme, a.offline)),
            SML,
        ),
        s(
            "ic-demo-unexplained",
            "3",
            false,
            Some(|a: &mut App| *a = demo_ic_app(a.ui.theme, a.offline)),
            SML,
        ),
        s(
            "ic-demo-bridge-unexplained",
            "3 enter",
            false,
            Some(|a: &mut App| *a = demo_ic_app(a.ui.theme, a.offline)),
            SML,
        ),
        s(
            "ic-demo-not-supported",
            "3 down",
            false,
            Some(|a: &mut App| *a = demo_ic_app(a.ui.theme, a.offline)),
            SML,
        ),
        s(
            "ic-demo-refusal",
            "3 down enter",
            false,
            Some(|a: &mut App| *a = demo_ic_app(a.ui.theme, a.offline)),
            SML,
        ),
        s(
            "ic-no-pairs",
            "3",
            false,
            Some(|a: &mut App| {
                let mut src = a.source.clone();
                src.ic_config = serde_json::json!({});
                let model = Model::open(&src, a.now).expect("record");
                *a = pinned(App::new(model, src, a.ui.theme, a.offline, a.now));
            }),
            SML,
        ),
        s("compare-4-code", "2 v", false, None, SML),
        s(
            "compare-2-code",
            "2 ] v down down space enter",
            false,
            None,
            SML,
        ),
        s("compare-picker", "2 ] v down down space", false, None, SML),
        s("compare-ne-row", "2 v down down down", false, None, SML),
        s(
            "compare-amount-col",
            "2 v down down down right",
            false,
            None,
            SML,
        ),
        s("compare-map", "2 v a", false, None, SML),
        s(
            "compare-map-no-account-map",
            "2 v a",
            false,
            Some(|a: &mut App| {
                a.source.account_map = serde_json::json!({});
                a.invalidate();
            }),
            SML,
        ),
        s("compare-history", "2 v h", false, None, SML),
        s("compare-l2", "2 v down down down enter", false, None, SML),
        s("compare-l2-marks", "2 v enter m", false, None, SML),
        s(
            "compare-refused",
            "2 v",
            false,
            Some(|a: &mut App| {
                let th = a.ui.theme;
                *a = cli_app_profile(th, "wide");
                a.compare_start(vec![1, 3]);
            }),
            SL,
        ),
        // v0.12.0 scope accents (theme spec v2 §2): the four orgs, a pair, compare (3), a group, all
        s("accent-org-1", "2 ] j", false, None, ACC),
        s("accent-org-2", "2 ] ] j", false, None, ACC),
        s("accent-org-3", "2 ] ] ] j", false, None, ACC),
        s("accent-org-4", "2 ] ] ] ] j", false, None, ACC),
        s("accent-pair", "3 enter", false, None, ACC),
        s("accent-compare-3", "2 ] ] ] ] ] ] v", false, None, ACC),
        s("accent-group", "] ] ] ] ] ]", false, None, ACC),
        s("accent-all", "", false, None, ACC),
        // v0.12.0 the focused view: the details layer (`i`)
        s("view-details-on", "2 ] enter enter i", false, None, ACC),
        // v0.12.0 the accent intensity knobs (theme spec v2 §6), each on alone, then all at their ceilings
        s(
            "knob-section-band-on",
            "2 ]",
            false,
            Some(|a: &mut App| knob(a, "section_band", "0.04")),
            ACC,
        ),
        s(
            "knob-pair-split",
            "3 enter",
            false,
            Some(|a: &mut App| knob(a, "pair_band", "split")),
            ACC,
        ),
        s(
            "knob-breadcrumb",
            "2 ] enter enter",
            false,
            Some(|a: &mut App| knob(a, "breadcrumb", "0.08")),
            ACC,
        ),
        s(
            "knob-compare-columns",
            "2 v",
            false,
            Some(|a: &mut App| knob(a, "compare_columns", "0.08")),
            ACC,
        ),
        s(
            "knob-status-rows",
            "",
            false,
            Some(|a: &mut App| {
                knob(a, "status_warn", "0.10");
                knob(a, "status_err", "0.10")
            }),
            ACC,
        ),
        s(
            "knob-header-gradient",
            "2 ]",
            false,
            Some(|a: &mut App| knob(a, "header_gradient", "true")),
            ACC,
        ),
        s(
            "knob-ceiling-all",
            "2 ]",
            false,
            Some(knobs_at_ceiling),
            ACC,
        ),
        s(
            "view-density-compact",
            "",
            false,
            Some(|a: &mut App| a.source.view.density = "compact"),
            ACC,
        ),
        s("ic-l2-explained", "3 j enter", false, None, ACC),
        // v0.12.0 the cosmetic overlay and user themes (theme spec v2 §6)
        s(
            "overlay-header",
            "",
            false,
            Some(|a: &mut App| overlay(a, &[("header.band.bg", "#3a2a20")])),
            ACC,
        ),
        s(
            "overlay-accent",
            "2 ]",
            false,
            Some(|a: &mut App| overlay(a, &[("accent", "iris")])),
            ACC,
        ),
        s(
            "overlay-cleared",
            "2 ]",
            false,
            Some(|a: &mut App| {
                overlay(a, &[("accent", "iris")]);
                a.ui.theme_overlay.clear();
                a.ui.theme_revision += 1;
            }),
            ACC,
        ),
        s(
            "theme-user-extends",
            "2 ]",
            false,
            Some(|a: &mut App| {
                a.ui.theme_user = Some(("warm".into(), DEMO_USER_THEME.into()));
                a.ui.theme_revision += 1;
            }),
            ACC,
        ),
        s("theme-invalid", "", false, Some(theme_invalid), ACC),
        s("overview-details", "i", false, None, ACC),
        s("ledger-details", "2 ] i", false, None, ACC),
        s("activity", "4", false, None, GRID),
        s("system", "5", false, None, GRID),
        s("overview-group", "] ] ] ] ]", false, None, GRID),
        s("overview-offline", "", true, None, GRID),
        s("overview-cell-history", "right right", false, None, GRID),
        s(
            "overview-stale",
            "",
            false,
            Some(|a: &mut App| *a = demo_app_variant(a.ui.theme, a.offline, "stale")),
            GRID,
        ),
        s(
            "overview-never-synced",
            "",
            false,
            Some(|a: &mut App| *a = demo_app_variant(a.ui.theme, a.offline, "never")),
            GRID,
        ),
        s(
            "overview-low-budget",
            "",
            false,
            Some(|a: &mut App| *a = demo_app_variant(a.ui.theme, a.offline, "lowbudget")),
            GRID,
        ),
        s(
            "overview-not-compared",
            "",
            false,
            Some(|a: &mut App| {
                a.ui.as_of = "2025-06-30".into();
                a.invalidate();
            }),
            GRID,
        ),
        s("help", "?", false, None, GRID),
        s("palette", "ctrl-p", false, None, GRID),
        s("palette-offline", "ctrl-p", true, None, GRID),
        s(
            "confirm-cancel",
            "",
            false,
            Some(|a: &mut App| a.open_confirm(NetAction::SyncOrg(2), None)),
            GRID,
        ),
        s(
            "confirm-action",
            "tab",
            false,
            Some(|a: &mut App| a.open_confirm(NetAction::SyncOrg(2), None)),
            GRID,
        ),
        s("picker", "g", false, None, GRID),
        s("picker-filter", "g l a b", false, None, SL),
        s("filter", "/ d e s", false, None, SL),
        s(
            "placeholder-sync-org",
            "ctrl-p s y n c space d e s i g n enter",
            false,
            None,
            SML,
        ),
        s("placeholder-login", "ctrl-p l o g enter", false, None, SML),
        // Fetch history is palette-gated on an open document (Phase 4 L3); the band itself, directly:
        s(
            "placeholder-fetch-history",
            "",
            false,
            Some(|a: &mut App| a.request_network(NetAction::FetchHistory)),
            SML,
        ),
        s(
            "placeholder-refetch",
            "2 ] ctrl-p r e f e t c h enter",
            false,
            None,
            SML,
        ),
        s(
            "refusal-offline",
            "",
            true,
            Some(|a: &mut App| a.request_network(NetAction::SyncOrg(0))),
            SL,
        ),
        // sync and login from the app (spec P6 §6)
        s(
            "net-confirm-sync",
            "ctrl-p s y n c space d e s i g n enter",
            false,
            None,
            NET,
        ),
        s(
            "net-confirm-sync-first",
            "",
            false,
            Some(|a: &mut App| {
                *a = demo_app_variant(a.ui.theme, a.offline, "never");
                run_script(a, "ctrl-p s y n c space l a b enter", 100, 30);
            }),
            NET,
        ),
        s(
            "net-confirm-sync-unknown",
            "",
            false,
            Some(|a: &mut App| {
                a.model.orgs[2].sync_estimate = None; // the estimator returned none
                run_script(a, "ctrl-p s y n c space d e s i g n enter", 100, 30);
            }),
            NET,
        ),
        s(
            "net-confirm-sync-group",
            "] ] ] ] ] ctrl-p s y n c space g r o u p enter",
            false,
            None,
            NET,
        ),
        s("net-confirm-login", "ctrl-p l o g enter", false, None, NET),
        s(
            "net-refused-budget",
            "",
            false,
            Some(|a: &mut App| {
                a.model.orgs[2].left = 4;
                run_script(a, "ctrl-p s y n c space d e s i g n enter", 100, 30);
            }),
            NET,
        ),
        s(
            "net-running-band",
            "",
            false,
            Some(|a: &mut App| net_running(a, false)),
            NET,
        ),
        s(
            "net-running-wait",
            "",
            false,
            Some(|a: &mut App| net_running(a, true)),
            NET,
        ),
        s(
            "net-running-hidden",
            "esc 2 ] enter",
            false,
            Some(|a: &mut App| net_running(a, false)),
            NET,
        ),
        s(
            "net-running-palette",
            "ctrl-p",
            false,
            Some(|a: &mut App| net_running(a, false)),
            NET,
        ),
        s(
            "net-login-waiting",
            "",
            false,
            Some(|a: &mut App| {
                run_script(a, "ctrl-p l o g enter tab enter", 100, 30);
                feed(a, &[r#"{"ev":"browser","url_shown":false,"timeout":900}"#]);
            }),
            NET,
        ),
        s(
            "net-done-ok",
            "",
            false,
            Some(|a: &mut App| {
                run_script(a, "] ] ]", 100, 30); // scope design: the header shows the synced org
                net_run_fake(a, "ok");
            }),
            NET,
        ),
        s(
            "net-done-limit",
            "",
            false,
            Some(|a: &mut App| {
                run_script(a, "] ] ]", 100, 30); // scope design: the header shows the synced org
                net_run_fake(a, "limit");
            }),
            NET,
        ),
        s(
            "net-done-error",
            "",
            false,
            Some(|a: &mut App| {
                run_script(a, "] ] ]", 100, 30); // scope design: the header shows the synced org
                net_run_fake(a, "error");
            }),
            NET,
        ),
        s(
            "net-no-cli",
            "ctrl-p s y n c space d e s i g n enter",
            false,
            Some(|a: &mut App| a.cli = None),
            NET,
        ),
        s(
            "net-quit-running",
            "q",
            false,
            Some(|a: &mut App| net_running(a, false)),
            NET,
        ),
        // agent captions
        ShellState {
            name: "footer-caption",
            script: "",
            offline: false,
            setup: Some(|a: &mut App| {
                let (w, h) = a.ui.size;
                agent_send(
                    a,
                    &[
                        &["tab", "ledger"],
                        &["scope", "roast"],
                        &["caption", CAPTION],
                    ],
                    w,
                    h,
                );
            }),
            sizes: NET,
            only: Some(caption_grid),
        },
        ShellState {
            name: "footer-caption-truncated",
            script: "",
            offline: false,
            setup: Some(|a: &mut App| {
                let (w, h) = a.ui.size;
                agent_send(
                    a,
                    &[
                        &["tab", "ledger"],
                        &["scope", "roast"],
                        &["caption", CAPTION],
                    ],
                    w,
                    h,
                );
            }),
            sizes: NET,
            only: Some(dark_80),
        },
        ShellState {
            name: "footer-caption-only",
            script: "",
            offline: false,
            setup: Some(|a: &mut App| {
                let (w, h) = a.ui.size;
                agent_send(
                    a,
                    &[&["caption", "Four orgs · all synced at 07:46 UTC"]],
                    w,
                    h,
                );
            }),
            sizes: NET,
            only: Some(caption_grid),
        },
        // a newer build installed
        ShellState {
            name: "footer-update-restart",
            script: "",
            offline: false,
            setup: Some(|a: &mut App| {
                a.update_available = Some(crate::canonical::Update {
                    version: "0.99.0".into(),
                    action: "restart".into(),
                });
            }),
            sizes: GRID,
            only: Some(version_grid),
        },
        // a newer build that can take over in place (v0.10.3: `⬆ vX ready · ⌃P Upgrade`)
        ShellState {
            name: "footer-update-ready",
            script: "",
            offline: false,
            setup: Some(ready),
            sizes: GRID,
            only: Some(version_grid),
        },
        // the palette's `Upgrade to vX` entry
        ShellState {
            name: "upgrade-palette",
            script: "ctrl-p u p g",
            offline: false,
            setup: Some(ready),
            sizes: GRID,
            only: Some(caption_grid),
        },
        // asked while a sync runs: deferred, applies by itself after it
        ShellState {
            name: "upgrade-deferred",
            script: "",
            offline: false,
            setup: Some(|a: &mut App| {
                net_running(a, false);
                ready(a);
                a.ui.note = Some(crate::upgrade::deferred_note("0.99.0", "sync design"));
            }),
            sizes: GRID,
            only: Some(caption_grid),
        },
        // the pre-flight refused (the old build carries on unchanged)
        ShellState {
            name: "upgrade-preflight-failed",
            script: "",
            offline: false,
            setup: Some(|a: &mut App| {
                ready(a);
                let o = crate::upgrade::Outcome::PreflightFailed(
                    "record_changed: the record moved since the state was saved".into(),
                );
                a.ui.note = Some(crate::upgrade::failed_note(&o, &a.version_label, "0.99.0"));
            }),
            sizes: GRID,
            only: Some(caption_grid),
        },
        // the new build, just resumed
        ShellState {
            name: "upgrade-updated",
            script: "",
            offline: false,
            setup: Some(|a: &mut App| {
                a.ui.note = Some(crate::upgrade::updated_note("0.10.1", &a.version_label));
            }),
            sizes: GRID,
            only: Some(caption_grid),
        },
        // navigation history: Alt+← back to the trial balance from another org's
        ShellState {
            name: "history-back",
            script: "2 ] ] alt-left",
            offline: false,
            setup: None,
            sizes: NET,
            only: Some(caption_grid),
        },
        // a state rendered against another record version (--allow-record-drift)
        ShellState {
            name: "state-drift",
            script: "2 ]",
            offline: false,
            setup: Some(|a: &mut App| {
                a.drift = Some(crate::canonical::Drift {
                    from: a.model.record_version - 7,
                    to: a.model.record_version,
                    config: None,
                    classify: None,
                    manifest: None,
                });
            }),
            sizes: NET,
            only: Some(caption_grid),
        },
        // a state rendered under another config (--allow-config-drift)
        ShellState {
            name: "config-drift",
            script: "2 ]",
            offline: false,
            setup: Some(|a: &mut App| {
                a.drift = Some(crate::canonical::Drift {
                    from: a.model.record_version,
                    to: a.model.record_version,
                    config: Some("orgs: a,b,c,d → a,c,d".into()),
                    classify: None,
                    manifest: None,
                });
            }),
            sizes: NET,
            only: Some(caption_grid),
        },
        // the running version: the footer's last item
        ShellState {
            name: "footer-version",
            script: "",
            offline: false,
            setup: None,
            sizes: GRID,
            only: Some(version_grid),
        },
        // the `/` filter on the Ledger TB: chip with its count, only the matches, the whole TB's totals
        ShellState {
            name: "ledger-filter",
            script: "2 ] / T a l l o w m e r e space enter",
            offline: false,
            setup: None,
            sizes: NET,
            only: Some(filter_grid),
        },
        ShellState {
            name: "ledger-filter-none",
            script: "2 ] / K i l n space R o o m space - enter",
            offline: false,
            setup: None,
            sizes: NET,
            only: Some(filter_grid),
        },
        // the agent bridge
        agent("agent-connected", |a: &mut App| a.agent_seen()),
        agent("agent-notice", |a: &mut App| {
            let size = a.ui.size;
            agent_send(
                a,
                &[&["tab", "ledger"], &["scope", "design"], &["differs", "on"]],
                size.0,
                size.1,
            );
        }),
        agent("agent-skipped", |a: &mut App| {
            a.agent_seen();
            a.agent_skipped();
        }),
        s("too-small", "", false, None, &[(52, 18)]),
    ]
}

/// A newer build installed that can take over in place.
fn ready(a: &mut App) {
    a.update_available = Some(crate::canonical::Update {
        version: "0.99.0".into(),
        action: crate::upgrade::ACTION.into(),
    });
}

/// Build the app for a state at a size.
pub fn state_app(st: &ShellState, theme: Theme, w: u16, h: u16) -> App {
    state_app_with(st, &fixture_app, theme, w, h)
}

/// Build a state's app from any base (the binary's `--dump-states` uses its own `--record`).
pub fn state_app_with(
    st: &ShellState,
    make: &dyn Fn(Theme, bool) -> App,
    theme: Theme,
    w: u16,
    h: u16,
) -> App {
    let mut app = make(theme, st.offline);
    if let Some(f) = st.setup {
        f(&mut app);
    }
    run_script(&mut app, st.script, w, h);
    app
}

/// Plain text, one line per row, trailing spaces trimmed (wide clusters drawn once).
pub fn text(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in buf.area.top()..buf.area.bottom() {
        out.push_str(row_text(buf, y).trim_end());
        out.push('\n');
    }
    out
}

pub fn row_text(buf: &Buffer, y: u16) -> String {
    let mut s = String::new();
    let mut skip = 0;
    for x in buf.area.left()..buf.area.right() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let sym = buf[(x, y)].symbol();
        skip = UnicodeWidthStr::width(sym).saturating_sub(1);
        s.push_str(sym);
    }
    s
}

fn color(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02X}{g:02X}{b:02X}"),
        Color::Indexed(i) => format!("i{i}"),
        other => format!("{other:?}").to_lowercase(),
    }
}

fn style_key(cell: &ratatui::buffer::Cell) -> String {
    let mut parts = Vec::new();
    if cell.fg != Color::Reset {
        parts.push(format!("fg={}", color(cell.fg)));
    }
    if cell.bg != Color::Reset {
        parts.push(format!("bg={}", color(cell.bg)));
    }
    let m = cell.modifier;
    let mut flags = String::new();
    for (f, c) in [
        (Modifier::BOLD, 'B'),
        (Modifier::DIM, 'D'),
        (Modifier::ITALIC, 'I'),
        (Modifier::UNDERLINED, 'U'),
        (Modifier::REVERSED, 'R'),
    ] {
        if m.contains(f) {
            flags.push(c);
        }
    }
    if !flags.is_empty() {
        parts.push(format!("+{flags}"));
    }
    if m.contains(Modifier::UNDERLINED) && cell.underline_color != Color::Reset {
        parts.push(format!("ul={}", color(cell.underline_color)));
    }
    parts.join(" ")
}

/// Text, then the style runs per row (`r<y> <x0>-<x1> fg=… bg=… +BDR`), default runs omitted, so
/// theme snapshots differ where their paint does.
pub fn styled(buf: &Buffer) -> String {
    let mut out = text(buf);
    out.push_str("--- styles\n");
    for y in buf.area.top()..buf.area.bottom() {
        let mut runs: Vec<(u16, u16, String)> = Vec::new();
        for x in buf.area.left()..buf.area.right() {
            let k = style_key(&buf[(x, y)]);
            match runs.last_mut() {
                Some((_, end, key)) if *key == k && *end + 1 == x => *end = x,
                _ => runs.push((x, x, k)),
            }
        }
        for (a, b, k) in runs.into_iter().filter(|(_, _, k)| !k.is_empty()) {
            out.push_str(&format!("r{y} {a}-{b} {k}\n"));
        }
    }
    out
}

/// A buffer cropped to its first `rows` rows (the XS header-only snapshots).
pub fn crop(buf: &Buffer, rows: u16) -> Buffer {
    let area = Rect::new(0, 0, buf.area.width, rows.min(buf.area.height));
    let mut out = Buffer::empty(area);
    for y in 0..area.height {
        for x in 0..area.width {
            out[(x, y)] = buf[(x, y)].clone();
        }
    }
    out
}

/// V5 on a rendered header row: READ-ONLY, OFFLINE (when set), the scope token, the as-of date, the
/// record age and the calls-left number, each whole.
pub fn header_problems(app: &App, row0: &str) -> Vec<String> {
    let f = app.header_facts();
    let fold = |s: &str| crate::glyphs::fold(s, app.ui.theme.ascii);
    let mut missing = Vec::new();
    let mut need = |label: &str, ok: bool| {
        if !ok {
            missing.push(format!("{label} missing in {row0:?}"));
        }
    };
    // v0.12.0 (design pass 2): the quiet read-only mark ends the row; the exceptions show when they apply
    need(
        "read-only at the right",
        row0.trim_end().ends_with("read-only"),
    );
    if app.offline {
        need("offline", row0.contains("offline"));
    }
    need(
        "scope",
        row0.contains(&fold(&f.scope_full))
            || row0.contains(&fold(&f.scope_compact))
            || (!f.scope_min.is_empty() && row0.contains(&fold(&f.scope_min))),
    );
    need("as-of", row0.contains(&f.as_of));
    let members = app.members(&app.ui.scope);
    let age = app.model.freshness(&members, app.now).age_token();
    if f.show_age {
        need("record age", row0.contains(&age));
    }
    if f.show_quota || f.busy {
        need("calls left", row0.contains(&crate::clock::int(f.left)));
    }
    need("no ellipsis", app.ui.theme.ascii || !row0.contains('…'));
    missing
}

/// The scopes of the fixture (for property tests): all, each org, the group.
pub fn fixture_scopes(app: &App) -> Vec<Scope> {
    app.scopes()
}

// ── the demo fixture (rust/tests/fixtures/demo; invented orgs roast, hold, design, lab) ────────────────

/// The demo fixture's directory.
pub fn demo_dir() -> PathBuf {
    fixtures_dir().join("demo")
}

/// The demo record variants (`fresh` = `record.sqlite`; see tests/fixtures/demo/README.md).
pub const DEMO_VARIANTS: [&str; 5] = ["fresh", "stale", "never", "lowbudget", "noclose"];

/// The demo source for a variant: `demo/ownpurse.json` (profile `demo`, groups `all-co` and `ops`, 4 pairs, an
/// account_map) through core's loader with an empty environment, the record swapped for the variant.
pub fn demo_source(variant: &str) -> Source {
    let dir = demo_dir();
    let env = ownpurse_core::config::Env {
        vars: Default::default(),
        cwd: dir.clone(),
        home: PathBuf::from("/nonexistent"),
    };
    let cfg = ownpurse_core::config::load(&env, Some("demo"), Some(&dir.join("ownpurse.json")))
        .expect("demo config");
    let mut source = Source::from_config(&cfg);
    let file = if variant == "fresh" {
        "record.sqlite".to_string()
    } else {
        format!("record-{variant}.sqlite")
    };
    source.record = dir.join(file);
    // the Classify tab: a private copy of the demo classification (classify.toml + the store), so writes in
    // tests and renders never touch the fixtures; the store's clock is pinned like the app's
    source.classify = Some(classify_copy(&dir));
    // manifests: a private store beside the classify copy (never the fixture folder)
    if let Some(c) = &source.classify
        && let Some(db) = c.store.parent()
    {
        source.manifests = Some(ownpurse_core::manifest::Paths::beside(
            db.join("manifests.sqlite"),
        ));
    }
    ownpurse_core::classify::store::pin_clock(Some("2026-10-05T09:46:00Z"));
    source
}

/// The cross-version frame rule, shared by the frozen-state test and the upgrade pre-flight (one
/// implementation): `got` and `want` have the same rows (one per line) and every row is byte-identical, except
/// row 1, the tab bar, which may only APPEND tab labels after the old ones (`tabs_row_extends`, compared as text,
/// so ANSI frames work too). Returns the first row (0-based) that breaks the rule, with the reason.
pub fn frame_difference(got: &str, want: &str) -> Option<(usize, &'static str)> {
    let g: Vec<&str> = got.split('\n').collect();
    let w: Vec<&str> = want.split('\n').collect();
    for (i, (a, b)) in g.iter().zip(&w).enumerate() {
        if a == b {
            continue;
        }
        if i == 1 && tabs_row_extends(&strip_sgr(b), &strip_sgr(a)) {
            continue;
        }
        return Some((
            i,
            if i == 1 {
                "the tab bar changed beyond appended tabs"
            } else {
                "the row differs"
            },
        ));
    }
    (g.len() != w.len()).then(|| {
        (
            g.len().min(w.len()),
            "the frame has a different number of rows",
        )
    })
}

/// A row without its SGR escapes (`ESC [ … m`).
pub fn strip_sgr(row: &str) -> String {
    let mut out = String::with_capacity(row.len());
    let mut it = row.chars().peekable();
    while let Some(c) = it.next() {
        if c == '\x1b' && it.peek() == Some(&'[') {
            for d in it.by_ref() {
                if d == 'm' {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}

/// The frozen-state rule for row 1 (the tab bar), PM 2026-10-06: the old row's text (its trailing padding
/// aside) must be a byte-identical prefix of the new row, and the only content allowed after it is new tab
/// labels, each `  <n> <Label>`, then spaces. A changed, removed or reordered old tab fails.
pub fn tabs_row_extends(old: &str, new: &str) -> bool {
    let old = old.trim_end_matches(' ');
    let Some(rest) = new.strip_prefix(old) else {
        return false;
    };
    let mut rest = rest.trim_end_matches(' ');
    if rest.is_empty() {
        return new.len() == old.len() || new[old.len()..].bytes().all(|b| b == b' ');
    }
    while !rest.is_empty() {
        // the gap before an appended tab: at least two spaces (the tab row's gap is 2 or 3 by width)
        let r = rest.trim_start_matches(' ');
        if rest.len() - r.len() < 2 {
            return false;
        }
        let digits = r.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 {
            return false;
        }
        let Some(r) = r[digits..].strip_prefix(' ') else {
            return false;
        };
        let word = r.bytes().take_while(|b| b.is_ascii_alphabetic()).count();
        if word == 0 || !r.as_bytes()[0].is_ascii_uppercase() {
            return false;
        }
        rest = &r[word..];
    }
    true
}

/// A fresh copy of the demo classification (`classify.toml`, `db/classify.sqlite`) in a temp folder.
pub fn classify_copy(demo: &Path) -> ownpurse_core::classify::ClassifyPaths {
    use std::sync::atomic::{AtomicU64, Ordering};
    static N: AtomicU64 = AtomicU64::new(0);
    static SWEPT: std::sync::Once = std::sync::Once::new();
    // copies left by earlier runs whose process is gone are removed once per run
    SWEPT.call_once(|| {
        let Ok(dir) = std::fs::read_dir(std::env::temp_dir()) else {
            return;
        };
        for e in dir.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let Some(pid) = name
                .strip_prefix("ownpurse-classify-")
                .and_then(|r| r.split('-').next())
                .and_then(|p| p.parse::<i32>().ok())
            else {
                continue;
            };
            // SAFETY: kill(pid, 0) only asks whether the process exists; it sends nothing
            let alive = unsafe { libc::kill(pid, 0) } == 0;
            if !alive {
                let _ = std::fs::remove_dir_all(e.path());
            }
        }
    });
    let d = std::env::temp_dir().join(format!(
        "ownpurse-classify-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("db")).expect("temp classify folder");
    std::fs::copy(demo.join("classify.toml"), d.join("classify.toml")).expect("classify.toml");
    std::fs::copy(
        demo.join("db/classify.sqlite"),
        d.join("db/classify.sqlite"),
    )
    .expect("classify store");
    ownpurse_core::classify::ClassifyPaths {
        config: d.join("classify.toml"),
        store: d.join("db/classify.sqlite"),
    }
}

/// The intercompany engine fixture (`tests/fixtures/record.sqlite` + the pairs in `ic-2025-12-31.json`):
/// orgs one, hq, co; seven pairs with every status (bridged, unexplained, not supported, not compared at
/// other dates) and known items found / not found. Group `pair` = hq + co.
pub fn ic_source() -> Source {
    let dir = fixtures_dir();
    let golden: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("ic-2025-12-31.json")).expect("ic golden"),
    )
    .expect("json");
    let orgs: Vec<(String, String)> = golden["tenants"]
        .as_object()
        .expect("tenants")
        .iter()
        .map(|(a, t)| (a.clone(), t.as_str().unwrap_or("").to_string()))
        .collect();
    let mut src = Source::bare(
        dir.join("record.sqlite"),
        orgs,
        vec![("pair".into(), vec!["hq".into(), "co".into()])],
    );
    src.ic_config = golden["config"].clone();
    src.profile = "ic".into();
    src
}

pub fn ic_app(theme: Theme, offline: bool) -> App {
    let source = ic_source();
    let now = fixture_now();
    let model = Model::open(&source, now).expect("ic fixture record");
    pinned(App::new(model, source, theme, offline, now))
}

/// The demo app for a variant, clock pinned to `FIXTURE_NOW`.
pub fn demo_app_variant(theme: Theme, offline: bool, variant: &str) -> App {
    let source = demo_source(variant);
    let now = fixture_now();
    let model = Model::open(&source, now).expect("demo record (rust/tests/fixtures/demo)");
    pinned(App::new(model, source, theme, offline, now))
}

/// The main demo app (`record.sqlite`).
pub fn demo_app(theme: Theme, offline: bool) -> App {
    demo_app_variant(theme, offline, "fresh")
}

/// The agent caption the frozen states carry (a realistic one on the demo world: Brackenridge's fee
/// receivable from Quillfeather in 2025, three lines, closing 4,670.51).
pub const FROZEN_CAPTION: &str = "Brackenridge fee receivable, 2025 · 3 lines · 4,670.51";

/// The states a release freezes for the next one to read (`tests/fixtures/state/<version>/`; the v21 gate
/// test imports every one and draws its frame). `release` picks the set: "0.10.2" froze the first four,
/// "0.10.3" adds what an upgrade must carry over (mid-typing, overlays, deep drills, the update notice).
pub fn frozen_state_cases(release: &str) -> Vec<(&'static str, App)> {
    let script_app = |theme: Theme, script: &str, w: u16, h: u16| {
        let mut a = fixture_app(theme, false);
        run_script(&mut a, script, w, h);
        a
    };
    let mut cases: Vec<(&'static str, App)> = vec![
        (
            "ledger-filtered",
            script_app(
                theme::LEDGER_DARK,
                "2 ] / T a l l o w m e r e space enter",
                100,
                30,
            ),
        ),
        (
            "palette-typing",
            script_app(theme::NO_COLOR_ASCII, "ctrl-p s y n left", 80, 24),
        ),
        (
            "compare-two",
            script_app(theme::LEDGER_DARK, "2 ] v down down space enter", 100, 30),
        ),
    ];
    if release != "0.10.2" {
        cases.extend([
            (
                "filter-half-typed",
                script_app(theme::LEDGER_DARK, "2 ] / T a l l left", 100, 30),
            ),
            (
                "picker-typing",
                script_app(theme::LEDGER_LIGHT, "g d e s", 80, 24),
            ),
            (
                "ledger-l3",
                script_app(theme::LEDGER_DARK, "2 ] j enter enter", 100, 30),
            ),
            ("update-ready", {
                let mut a = fixture_app(theme::LEDGER_DARK, false);
                a.update_available = Some(crate::canonical::Update {
                    version: "0.99.0".into(),
                    action: crate::upgrade::ACTION.into(),
                });
                run_script(&mut a, "2 ]", 100, 30);
                a
            }),
        ]);
    }
    let mut a = fixture_app(theme::LEDGER_LIGHT, false);
    agent_send(
        &mut a,
        &[
            &["tab", "ledger"],
            &["scope", "hold"],
            &["caption", FROZEN_CAPTION],
        ],
        100,
        30,
    );
    cases.push(("agent-caption", a));
    // v0.12.0 (render epoch 1): the theme overlay, the details layer and a user theme ride in the state
    if !["0.10.2", "0.10.3", "0.10.4", "0.11.0", "0.11.1"].contains(&release) {
        let mut a = script_app(theme::LEDGER_DARK, "2 ] i", 100, 30);
        overlay(&mut a, &[("accent", "iris"), ("knob.section_band", "0.04")]);
        cases.push(("overlay-details", a));
        let mut a = script_app(theme::LEDGER_DARK, "3 enter", 100, 30);
        a.ui.theme_user = Some(("warm".into(), DEMO_USER_THEME.into()));
        a.ui.theme_revision += 1;
        cases.push(("user-theme-pair", a));
    }
    // v0.14.0: overlay layers (an agent's hint, attributed) and the narration panel ride in the state
    if ![
        "0.10.2", "0.10.3", "0.10.4", "0.11.0", "0.11.1", "0.12.0", "0.12.1", "0.13.0",
    ]
    .contains(&release)
    {
        let mut a = fixture_app(theme::LEDGER_DARK, false);
        agent_send(&mut a, &[&["tab", "ledger"], &["scope", "hold"]], 100, 30);
        agent_send(
            &mut a,
            &[&[
                "narrate",
                "The holding company",
                "Its [trial balance] at the year end.",
            ]],
            100,
            30,
        );
        agent_send(
            &mut a,
            &[&["hint", "#2", "This row.", "title=Here"]],
            100,
            30,
        );
        cases.push(("hint-narrate", a));
    }
    cases
}

/// S, M and L (the tab snapshot sizes, spec §13).
pub const SML: &[(u16, u16)] = &[(80, 24), (100, 30), (140, 40)];

/// A tab state on the demo fixture: a key script on a record variant.
pub struct TabState {
    pub name: &'static str,
    pub script: &'static str,
    pub variant: &'static str,
    pub sizes: &'static [(u16, u16)],
    /// Run after the script (e.g. a hand edit of classify.toml and the watcher's event).
    pub setup: Option<fn(&mut App)>,
}

/// Activity and System states (spec §13 rows Activity, System) on the demo fixture.
pub fn tab_states() -> Vec<TabState> {
    let s = |name, script, variant| TabState {
        name,
        script,
        variant,
        sizes: SML,
        setup: None,
    };
    vec![
        s("activity-log", "4", "fresh"),
        s("activity-log-hold-404", "4 ] ]", "fresh"),
        s("activity-log-design-429", "4 ] ] ]", "fresh"),
        s("activity-filter", "4 / t r i a l enter", "fresh"),
        s("activity-grouped", "4 G", "fresh"),
        s("activity-low-budget", "4", "lowbudget"),
        s("activity-stale-never", "4", "never"),
        s("system", "5", "fresh"),
        s("system-settings", "5 j j j j", "fresh"),
        s("system-as-of-default", "5 / a s _ o f enter", "fresh"),
        s("system-links", "5 end", "fresh"),
        s(
            "system-filter-reconcile",
            "5 / r e c o n c i l e enter",
            "fresh",
        ),
        s("system-never-synced", "5", "never"),
    ]
    .into_iter()
    .chain(classify_states())
    .collect()
}

/// The Classify grid (spec classify §8): 100×30 in ledger-dark, ledger-light and nocolor-ascii, plus 80×24
/// (dark); the tests pick the themes, the renders take all four.
pub const CLASSIFY_SIZES: &[(u16, u16)] = &[(100, 30), (80, 24)];

/// The Classify states (the Queue slice of spec classify §8), on the demo fixture with the demo labels.
pub fn classify_states() -> Vec<TabState> {
    let s = |name, script| TabState {
        name,
        script,
        variant: "fresh",
        sizes: CLASSIFY_SIZES,
        setup: None,
    };
    vec![
        s("classify-queue", "6"),
        s(
            "classify-queue-empty",
            "6 / h a r v e s t space z z z enter",
        ),
        s(
            "classify-queue-filtered",
            "6 / o r g : r o a s t space c o n f < 0 . 9 enter",
        ),
        s("classify-queue-stale", "6 / s t a l e enter"),
        s("classify-queue-bulk-preview", "6 A - -"),
        s("classify-queue-jev-disabled", "6 ctrl-p j e v"),
        s("classify-queue-picker-bearer", "6 j b"),
        s(
            "classify-queue-note",
            "6 j j n P r o o f s space f o r space t h e space H a r v e s t space B l e n d space b a g s",
        ),
        s("classify-queue-project-keys", "6 j p"),
        s("classify-queue-group-picker", "6 j G"),
        s("classify-queue-accepted", "6 j enter"),
        s("classify-queue-make-rule", "6 j R"),
        // Catalog (spec classify §3.D)
        s("classify-catalog-projects", "6 ,"),
        s(
            "classify-catalog-project-edit",
            "6 , enter tab , space H B 2 5",
        ),
        s("classify-catalog-archive-blocked", "6 , D"),
        s("classify-catalog-rules", "6 , right"),
        s("classify-catalog-rule-edit-preview", "6 , right enter"),
        s(
            "classify-catalog-diff",
            "6 , N R o a s t e r y space r e f i t enter right enter shift-tab backspace 0 enter w",
        ),
        // the classify.toml watcher's bands (shown over the Tree mode, as the spec's tree states)
        // Groups (spec classify §3.E)
        s("classify-groups", "6 ."),
        TabState {
            setup: Some(group_one_to_one),
            ..s("classify-groups-one-to-one-ok", "6 .")
        },
        TabState {
            setup: Some(group_gap),
            ..s("classify-groups-gap", "6 .")
        },
        TabState {
            setup: Some(group_aggregate),
            ..s("classify-groups-aggregate", "6 .")
        },
        TabState {
            setup: Some(group_running),
            ..s("classify-groups-running-balance", "6 .")
        },
        TabState {
            setup: Some(group_gap_add_member),
            ..s("classify-groups-add-member", "6 .")
        },
        // agent writes (spec classify §5a): the notice, A in the agent colour, U
        TabState {
            setup: Some(agent_writes_labels),
            ..s(
                "classify-agent-write-labels",
                "6 / o r g : d e s i g n space s t a t u s : a l l enter",
            )
        },
        TabState {
            setup: Some(agent_writes_config),
            ..s("classify-agent-write-config", "6 ,")
        },
        TabState {
            setup: Some(agent_writes_then_undo),
            ..s(
                "classify-agent-write-undone",
                "6 / o r g : d e s i g n space s t a t u s : a l l enter",
            )
        },
        TabState {
            setup: Some(hand_edit_valid),
            ..s("classify-tree-hand-edit-reloaded", "6 , ,")
        },
        TabState {
            setup: Some(hand_edit_invalid),
            ..s("classify-tree-hand-edit-error", "6 , ,")
        },
    ]
}

/// An agent's classify request as the bridge delivers it (through `update`, applied by the loop's effect
/// runner): the answer document. `doc` gets `v`, `kind`, `cv`, `client` and `expect_store_rev` filled in
/// when missing.
pub fn agent_classify(app: &mut App, mut doc: serde_json::Value) -> serde_json::Value {
    let rev = app.classify.as_ref().map_or(0, |d| d.provider.rev());
    for (k, v) in [
        ("v", serde_json::json!(1)),
        ("kind", serde_json::json!("classify")),
        ("cv", serde_json::json!(1)),
        ("client", serde_json::json!("claude")),
        ("expect_store_rev", serde_json::json!(rev)),
    ] {
        if doc.get(k).is_none() {
            doc[k] = v;
        }
    }
    app.agent_seen();
    let req = match ownpurse_core::classify::request::ClassifyRequest::from_json(&doc) {
        Ok(r) => r,
        Err(e) => return ownpurse_core::classify::request::error_response(&e),
    };
    app.agent.classify_reply = None;
    let fx = crate::app::update(app, Msg::AgentClassify(Box::new(req)));
    crate::classify::run_effects(app, fx);
    let (w, h) = app.ui.size;
    render(app, w, h);
    app.agent
        .classify_reply
        .take()
        .unwrap_or(serde_json::Value::Null)
}

/// The agent sets every label on the design org's first three listed lines (`agent-write-labels`).
/// The queue's rows largest amount first (`Sort::AmountDesc`'s order: ties by date, then anchor), whatever
/// the queue's own order: fixtures that pick "the largest lines" keep picking the same ones.
pub fn rows_by_amount(app: &mut App) -> Vec<ownpurse_core::classify::QueueRow> {
    let mut rows: Vec<_> = app.cl_rows().rows.iter().cloned().collect();
    rows.sort_by(|a, b| {
        b.amount
            .abs()
            .cmp(&a.amount.abs())
            .then_with(|| a.date.cmp(&b.date))
            .then_with(|| a.anchor.cmp(&b.anchor))
    });
    rows
}

pub fn agent_writes_labels(app: &mut App) {
    let anchors: Vec<String> = rows_by_amount(app)
        .iter()
        .take(3)
        .map(|r| r.anchor.to_string())
        .collect();
    agent_classify(
        app,
        serde_json::json!({"op": "set", "anchors": anchors,
            "labels": {"project": "refit", "bearer": "design", "treatment": "owner-paid"}}),
    );
}

/// The agent adds a project to classify.toml (`agent-write-config`).
pub fn agent_writes_config(app: &mut App) {
    agent_classify(
        app,
        serde_json::json!({"op": "config", "confirm": true, "edits": [
            {"edit": "add_choice", "label_type": "project", "id": "tasting-tour", "name": "Tasting tour"}
        ]}),
    );
}

/// Labels, then the user's `U` (`agent-write-undone`).
pub fn agent_writes_then_undo(app: &mut App) {
    agent_writes_labels(app);
    let (w, h) = app.ui.size;
    run_script(app, "U", w, h);
}

/// The anchor of the first line of `org` with this signed amount.
pub fn line_anchor(app: &App, org: &str, amount: &str) -> String {
    let want: rust_decimal::Decimal = amount.parse().expect("amount");
    let d = app.classify.as_ref().expect("classification");
    d.provider
        .queue(
            &ownpurse_core::classify::Filter::parse("status:all").expect("filter"),
            ownpurse_core::classify::Sort::DateAsc,
        )
        .into_iter()
        .find(|r| r.org == org && r.amount == want)
        .unwrap_or_else(|| panic!("no {org} line of {amount}"))
        .anchor
        .to_string()
}

/// A group made through the agent path (the demo store has two; the chain-check states need more):
/// `members` are `(org, signed amount, role)`. Returns its id.
pub fn make_group(
    app: &mut App,
    kind: &str,
    title: &str,
    members: &[(&str, &str, &str)],
) -> String {
    let made = agent_classify(
        app,
        serde_json::json!({"op": "group_create", "group_kind": kind, "title": title}),
    );
    let id = made["group"].as_str().expect("group id").to_string();
    for (org, amount, role) in members {
        let a = line_anchor(app, org, amount);
        agent_classify(
            app,
            serde_json::json!({"op": "group_add", "group": id, "anchors": [a], "role": role}),
        );
    }
    // made through the agent path for brevity; the states show the user's own view
    app.agent.notice = None;
    app.agent.acted = false;
    app.agent.connected = false;
    app.ui.classify.groups.sel = Some(id.clone());
    let (w, h) = app.ui.size;
    render(app, w, h);
    id
}

/// One origin = one mirror: the term-loan drawdown on both sides.
pub fn group_one_to_one(app: &mut App) {
    make_group(
        app,
        "intercompany_chain",
        "Term loan drawdown",
        &[
            ("hold", "-18437.62", "origin"),
            ("roast", "18437.62", "mirror"),
        ],
    );
}

/// A mirror missing.
pub fn group_gap(app: &mut App) {
    make_group(
        app,
        "intercompany_chain",
        "Equipment advance",
        &[("hold", "-6912.48", "origin")],
    );
}

/// The gap group, then `→ m`: its suggested member (the same amount on the same day in the lab).
pub fn group_gap_add_member(app: &mut App) {
    group_gap(app);
    let (w, h) = app.ui.size;
    run_script(app, "right m", w, h);
}

/// Two origins against one mirror: the share explained, never a line-level match.
pub fn group_aggregate(app: &mut App) {
    make_group(
        app,
        "intercompany_chain",
        "Term loan repayments 2025",
        &[
            ("roast", "-3281.19", "origin"),
            ("roast", "-412.37", "origin"),
            ("hold", "-18437.62", "mirror"),
        ],
    );
}

/// A settlement group with its pair rule (Brackenridge 1460 = Tallowmere 2410).
pub fn group_running(app: &mut App) {
    make_group(
        app,
        "settlement",
        "Term loan and its first instalment",
        &[
            ("hold", "-18437.62", "origin"),
            ("roast", "18437.62", "mirror"),
            ("roast", "-3281.19", "settlement"),
            ("hold", "3281.19", "settlement"),
        ],
    );
}

/// A hand edit of the app's (temp) classify.toml, then the watcher's event.
fn hand_edit(app: &mut App, from: &str, to: &str) {
    if let Some(p) = app.source.classify.as_ref() {
        let text = std::fs::read_to_string(&p.config).expect("classify.toml");
        std::fs::write(&p.config, text.replacen(from, to, 1)).expect("hand edit");
    }
    let _ = crate::app::update(app, Msg::ClassifyConfigChanged);
}

/// A valid hand edit: a project renamed.
pub fn hand_edit_valid(app: &mut App) {
    hand_edit(app, "\"Wayfinding refit\"", "\"Wayfinding refit, phase 2\"");
}

/// An invalid hand edit: a rule sets an unknown bearer.
pub fn hand_edit_invalid(app: &mut App) {
    hand_edit(
        app,
        "bearer = \"roast\", treatment",
        "bearer = \"roast2\", treatment",
    );
}

/// Build a tab state's app at a size.
pub fn tab_state_app(st: &TabState, theme: Theme, w: u16, h: u16) -> App {
    let mut app = demo_app_variant(theme, false, st.variant);
    run_script(&mut app, st.script, w, h);
    if let Some(f) = st.setup {
        f(&mut app);
        render(&mut app, w, h);
    }
    app
}

/// Set one accent intensity knob (dark and light) for a knob snapshot state.
pub fn knob(a: &mut App, name: &str, value: &str) {
    for k in a.source.knobs.iter_mut() {
        let _ = k.set(name, value);
    }
    a.dirty = true;
}

/// Every knob at its computed ceiling (dark and light), the gradient on.
pub fn knobs_at_ceiling(a: &mut App) {
    for (i, k) in a.source.knobs.iter_mut().enumerate() {
        let mut c = crate::theme::def::Knobs::ceilings(i == 0);
        c.header_gradient = true;
        *k = c;
    }
    a.dirty = true;
}

/// Set overlay tokens (the `overlay set` verb's effect) for a snapshot state.
pub fn overlay(a: &mut App, pairs: &[(&str, &str)]) {
    for (k, v) in pairs {
        a.ui.theme_overlay.insert(k.to_string(), v.to_string());
    }
    a.ui.theme_revision += 1;
    a.dirty = true;
}

/// The demo user theme (`themes/warm.toml`): warmer surfaces, a faint accent in the section bands.
pub const DEMO_USER_THEME: &str = "[meta]\nname = \"warm\"\nextends = \"ledger-dark\"\n\n[palette]\nslate.950 = { hex = \"#161412\" }\nslate.900 = { hex = \"#1E1A17\" }\n\n[semantic]\nbg = \"@slate.950\"\n\"surface.top\" = \"@slate.900\"\n\n[component]\n\"section.band.bg\" = \"mix($surface.section, $accent, 0.04)\"\n";

/// An invalid theme file save: the previous theme stays, the error band names file:line.
pub fn theme_invalid(a: &mut App) {
    a.user_config_event(crate::user_config::ConfigEvent::Invalid(
        crate::user_config::ConfigIssue {
            file: "themes/warm.toml".into(),
            line: 9,
            path: "semantic.bg".into(),
            message: "\"@slate.95\": unknown palette entry (did you mean \"slate.950\"?)".into(),
        },
    ));
}

/// The demo app on a given record copy and manifest store (manifest tests and renders: the record copy may
/// carry approvals, write attempts and read-backs; the store, `manifests.current` beside it, picks the shown
/// manifest).
pub fn demo_app_at(theme: Theme, record: &Path, manifests: &Path) -> App {
    let mut source = demo_source("fresh");
    source.record = record.to_path_buf();
    source.manifests = Some(ownpurse_core::manifest::Paths::beside(
        manifests.to_path_buf(),
    ));
    let now = fixture_now();
    let model = Model::open(&source, now).expect("demo record copy");
    let mut app = pinned(App::new(model, source, theme, false, now));
    app.prepare();
    app
}
