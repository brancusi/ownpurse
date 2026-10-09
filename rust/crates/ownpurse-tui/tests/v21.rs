//! V21, the round-trip law (v0.10.2): the canonical view state is complete.
//! (a) export → import → export is byte-identical;
//! (b) render(import(export(live))) == the live frame, cell for cell (TestBackend buffer);
//! (c) a property test: random keys, clicks, resizes, ticks and agent verbs, all tabs and levels, 4 widths
//!     and 4 themes; after EVERY step a fresh app built from the exported state alone (plus the same record)
//!     draws exactly the live frame. A fixed seed set runs in the gate; `OWNPURSE_V21_SOAK=<seeds>` runs
//!     more. A failure prints the seed and a minimal step sequence;
//! (d) the text, html, svg and png renders of the imported state equal the live ones.

use crossterm::event::{KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use ownpurse_tui::app::{App, Msg, update};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme::{self, Theme};
use ratatui::buffer::Buffer;

fn render(a: &mut App) -> Buffer {
    let (w, h) = a.ui.size;
    snapshot::render(a, w, h)
}

/// The full `ui state` document, canonical JSON.
fn export(a: &App) -> String {
    a.view_state(a.rev).canonical()
}

fn import(a: &App, allow_drift: bool) -> App {
    App::from_state_json(&export(a), a.source.clone(), allow_drift).expect("import")
}

fn script_app(theme: Theme, script: &str, w: u16, h: u16) -> App {
    let mut a = fixture_app(theme, false);
    snapshot::run_script(&mut a, script, w, h);
    a
}

/// Scripted states covering every layer: views and drills, overlays mid-query, prompts mid-typing with the
/// caret moved, the confirm band with focus moved, agent notices, captions, history with a forward stack.
fn scripted() -> Vec<(&'static str, App)> {
    let mut out: Vec<(&'static str, App)> = Vec::new();
    for script in [
        "",
        "2 ]",
        "2 ] j enter",
        "2 ] j enter enter",
        "3 enter",
        "4",
        "4 G",
        "5 j j",
        "2 ] v",
        "g",
        "g d e s",
        "ctrl-p s y",
        "/ r o a s t left left",
        "/ r o a s t home right",
        "d 2 0 2 5",
        "2 ] / l o a n enter j enter esc",
        "2 ] ] alt-left alt-left alt-right",
        "ctrl-p s y n c space d e s i g n enter tab",
        "?",
        // Classify (v0.11.0): the Queue, its editors mid-typing, the bulk preview, `p` pending, other modes
        "6",
        "6 j j",
        "6 j b d e",
        "6 j t down",
        "6 j s",
        "6 n N o t e left",
        "6 n a ctrl-e b enter c",
        "6 A + tab",
        "6 p",
        "6 / o r g : r o a s t enter",
        "6 / c o n f < x",
        "6 . .",
        "6 ,",
        "6 , right",
        "6 , right enter tab a b left",
        "6 , N R o a",
        "6 , D",
        "6 , N x enter w tab",
        "6 j R",
        "6 . enter",
        "6 . enter enter",
        "6 . right m",
        "6 . S",
    ] {
        let a = script_app(theme::LEDGER_DARK, script, 100, 30);
        out.push((script, a));
    }
    let mut a = fixture_app(theme::LEDGER_LIGHT, false);
    snapshot::agent_send(
        &mut a,
        &[
            &["tab", "ledger"],
            &["scope", "design"],
            &["caption", snapshot::FROZEN_CAPTION],
        ],
        100,
        30,
    );
    out.push(("agent caption", a));
    let mut a = fixture_app(theme::NO_COLOR_ASCII, false);
    snapshot::agent_send(
        &mut a,
        &[
            &["tab", "intercompany"],
            &["select", "hold-design"],
            &["drill"],
        ],
        80,
        24,
    );
    out.push(("agent drill (ascii)", a));
    out
}

#[test]
fn v21_a_export_import_export_is_byte_identical() {
    for (name, mut a) in scripted() {
        render(&mut a);
        let one = export(&a);
        let b = import(&a, false);
        let two = export(&b);
        assert_eq!(one, two, "{name}: export → import → export");
        assert!(one.contains("\"v\":1"), "{name}: schema version");
    }
}

#[test]
fn v21_b_and_d_imported_renders_equal_the_live_frame() {
    for (name, mut a) in scripted() {
        let live = render(&mut a);
        let mut b = import(&a, false);
        let back = render(&mut b);
        assert_eq!(
            live,
            back,
            "{name}: buffer\n{}\n---\n{}",
            snapshot::text(&live),
            snapshot::text(&back)
        );
        let th = a.ui.theme;
        assert_eq!(snapshot::text(&live), snapshot::text(&back), "{name}: text");
        assert_eq!(
            ownpurse_tui::render::html(&live, &th, "x"),
            ownpurse_tui::render::html(&back, &th, "x"),
            "{name}: html"
        );
        assert_eq!(
            ownpurse_tui::render::svg(&live, &th),
            ownpurse_tui::render::svg(&back, &th),
            "{name}: svg"
        );
        let png =
            |b: &Buffer| ownpurse_core::raster::png(&ownpurse_tui::render::grid(b, &th)).unwrap();
        assert_eq!(png(&live), png(&back), "{name}: png");
    }
}

// ── (c) the property test ───────────────────────────────────────────────────────────────────────

/// A small deterministic PRNG (xorshift64*): no new dependency.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
}

const KEYS: &[&str] = &[
    "1",
    "2",
    "3",
    "4",
    "5",
    "tab",
    "shift-tab",
    "]",
    "[",
    "g",
    "j",
    "k",
    "down",
    "up",
    "pagedown",
    "end",
    "home",
    "enter",
    "esc",
    "left",
    "right",
    "/",
    "a",
    "c",
    "m",
    "e",
    "o",
    "n",
    "backspace",
    "space",
    "d",
    "h",
    "!",
    "v",
    "G",
    "s",
    "ctrl-p",
    "?",
    "r",
    "`",
    "alt-left",
    "alt-right",
    "q",
    "x",
    "y",
    "f",
    // Classify (v0.11.0)
    "6",
    ",",
    ".",
    "p",
    "b",
    "t",
    "u",
    "A",
    "+",
    "-",
    "1",
    "ctrl-e",
    "ctrl-s",
    "N",
    "D",
    "w",
    "R",
    "G",
    "S",
    "m",
];

const VERBS: &[&[&str]] = &[
    &["tab", "ledger"],
    &["tab", "overview"],
    &["tab", "intercompany"],
    &["tab", "activity"],
    &["scope", "roast"],
    &["scope", "design"],
    &["scope", "all"],
    &["scope", "group:all-co"],
    &["filter", "Tallowmere"],
    &["filter", "code:2410..2440"],
    &["clear-filter"],
    &["select", "#2"],
    &["select", "1010"],
    &["drill"],
    &["up"],
    &["back"],
    &["forward"],
    &["asof", "2025-06-30"],
    &["basis", "history"],
    &["differs", "on"],
    &["compare", "roast,design"],
    &[
        "caption",
        "Green-bean card spend, 2025 · 3 lines · 1,990.90",
    ],
    &["caption", "clear"],
    &["label", "books"],
    &["tab", "classify"],
    &["filter", "org:roast conf<0.9"],
    &["filter", "status:stale"],
];

const SIZES: &[(u16, u16)] = &[(60, 20), (80, 24), (100, 30), (140, 40)];

#[derive(Clone, Debug)]
enum Step {
    Key(&'static str),
    Click(u16, u16),
    Resize(u16, u16),
    Tick(i64),
    Agent(&'static [&'static str]),
}

fn gen_step(r: &mut Rng, size: (u16, u16)) -> Step {
    match r.below(20) {
        0..=12 => Step::Key(r.pick(KEYS)),
        13 | 14 => Step::Click(
            r.below(size.0 as usize) as u16,
            r.below(size.1 as usize) as u16,
        ),
        15 => {
            let s = *r.pick(SIZES);
            Step::Resize(s.0, s.1)
        }
        16 => Step::Tick(r.below(400) as i64),
        _ => Step::Agent(r.pick(VERBS)),
    }
}

fn apply(a: &mut App, s: &Step) {
    let (w, h) = a.ui.size;
    match s {
        Step::Key(k) => {
            let _ = update(a, Msg::Key(Key::parse(k).expect("key").event()));
        }
        Step::Click(x, y) => {
            let ev = MouseEvent {
                kind: MouseEventKind::Down(MouseButton::Left),
                column: *x,
                row: *y,
                modifiers: KeyModifiers::NONE,
            };
            let _ = update(a, Msg::Mouse { ev, clicks: 1 });
        }
        Step::Resize(w, h) => {
            let _ = update(a, Msg::Resize(*w, *h));
        }
        Step::Tick(d) => {
            let t = a.now + d;
            let _ = update(a, Msg::Tick(t));
        }
        Step::Agent(v) => {
            if a.agent_busy().is_none() {
                let _ = snapshot::agent_send_result(a, &[*v], w, h);
            }
        }
    }
    // the runtime would quit or spawn here; the state is what matters
    render(a);
}

/// Run `steps`; the index of the first step after which the imported frame differs, with both frames.
fn first_divergence(theme: Theme, size: (u16, u16), steps: &[Step]) -> Option<(usize, String)> {
    let mut a = fixture_app(theme, false);
    let _ = update(&mut a, Msg::Resize(size.0, size.1));
    render(&mut a);
    for (i, s) in steps.iter().enumerate() {
        apply(&mut a, s);
        let live = render(&mut a);
        let mut b = match App::from_state_json(&export(&a), a.source.clone(), false) {
            Ok(b) => b,
            Err(e) => return Some((i, format!("import failed: {}", e.message()))),
        };
        let back = render(&mut b);
        if live != back {
            return Some((
                i,
                format!(
                    "live:\n{}\nimported:\n{}",
                    snapshot::text(&live),
                    snapshot::text(&back)
                ),
            ));
        }
        let (one, two) = (export(&a), export(&b));
        if one != two {
            return Some((i, "export → import → export differs".into()));
        }
    }
    None
}

/// The smallest step sequence (greedy one-step removal) that still diverges.
fn shrink(theme: Theme, size: (u16, u16), mut steps: Vec<Step>) -> Vec<Step> {
    if let Some((i, _)) = first_divergence(theme, size, &steps) {
        steps.truncate(i + 1);
    }
    let mut j = 0;
    while j < steps.len() {
        let mut fewer = steps.clone();
        fewer.remove(j);
        if first_divergence(theme, size, &fewer).is_some() {
            steps = fewer;
        } else {
            j += 1;
        }
    }
    steps
}

fn run_seed(seed: u64, steps_n: usize) -> Result<(), String> {
    let mut r = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
    let themes = [
        theme::LEDGER_DARK,
        theme::LEDGER_LIGHT,
        theme::ANSI,
        theme::NO_COLOR_ASCII,
    ];
    let theme = themes[(seed % 4) as usize];
    let size = SIZES[((seed / 4) % 4) as usize];
    let mut steps = Vec::new();
    let mut cur = size;
    for _ in 0..steps_n {
        let s = gen_step(&mut r, cur);
        if let Step::Resize(w, h) = s {
            cur = (w, h);
        }
        steps.push(s);
    }
    match first_divergence(theme, size, &steps) {
        None => Ok(()),
        Some((_, frames)) => {
            let min = shrink(theme, size, steps);
            Err(format!(
                "V21 failed: seed {seed} ({} at {}x{}); minimal steps ({}): {min:?}\n{frames}",
                theme.name(),
                size.0,
                size.1,
                min.len()
            ))
        }
    }
}

#[test]
fn v21_c_property_every_step_round_trips() {
    let soak: u64 = std::env::var("OWNPURSE_V21_SOAK")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    let seeds = if soak > 0 { soak } else { 16 };
    let steps = 40;
    let t = std::time::Instant::now();
    let mut failures = Vec::new();
    for seed in 1..=seeds {
        if let Err(e) = run_seed(seed, steps) {
            failures.push(e);
            break;
        }
    }
    eprintln!(
        "V21 property: {seeds} seeds × {steps} steps in {:.1?}",
        t.elapsed()
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

// ── record version ──────────────────────────────────────────────────────────────────────────────

#[test]
fn another_record_version_is_refused_unless_drift_is_allowed() {
    let mut a = script_app(theme::LEDGER_DARK, "2 ]", 100, 30);
    render(&mut a);
    let mut state: serde_json::Value = serde_json::from_str(&export(&a)).unwrap();
    let real = state["canonical"]["record"]["chain_seq"].as_i64().unwrap();
    state["canonical"]["record"]["chain_seq"] = serde_json::json!(real - 5);
    let text = state.to_string();
    let e = App::from_state_json(&text, a.source.clone(), false)
        .err()
        .unwrap();
    assert_eq!((e.code(), e.exit()), ("record_changed", 4));
    assert_eq!(
        e.message(),
        format!(
            "The state was exported against record version {}; the record is now at {real}. Nothing was rendered.",
            real - 5
        )
    );
    let mut b = App::from_state_json(&text, a.source.clone(), true).unwrap();
    let st = b.view_state(b.rev);
    assert_eq!(
        st.drift,
        Some(ownpurse_tui::canonical::Drift {
            from: real - 5,
            to: real,
            config: None,
            classify: None,
            manifest: None,
        })
    );
    let row0 = snapshot::row_text(&render(&mut b), 0);
    assert!(row0.contains("≠ record changed"), "the drift chip: {row0}");
    // a drifted state round-trips as drifted (it still names the version it came from)
    let c = App::from_state_json(&export(&b), b.source.clone(), true).unwrap();
    assert_eq!(c.drift, b.drift);
    let e = App::from_state_json(&export(&b), b.source.clone(), false)
        .err()
        .unwrap();
    assert_eq!(e.code(), "record_changed");
    // ASCII form of the chip
    let mut a = script_app(theme::NO_COLOR_ASCII, "", 100, 30);
    render(&mut a);
    let mut s: serde_json::Value = serde_json::from_str(&export(&a)).unwrap();
    s["canonical"]["record"]["chain_seq"] = serde_json::json!(1);
    let mut d = App::from_state_json(&s.to_string(), a.source.clone(), true).unwrap();
    assert!(snapshot::row_text(&render(&mut d), 0).contains("!= record changed"));
}

// ── config ──────────────────────────────────────────────────────────────────────────────────────────────

fn aliases(s: &ownpurse_tui::model::Source) -> Vec<String> {
    s.orgs.iter().map(|(a, _)| a.clone()).collect()
}

#[test]
fn scopes_are_stored_by_alias() {
    let mut a = script_app(theme::LEDGER_DARK, "2 ] ]", 100, 30);
    render(&mut a);
    let v: serde_json::Value = serde_json::from_str(&export(&a)).unwrap();
    let scope = &v["canonical"]["ui"]["scope"];
    assert!(scope["Org"].is_string(), "scope by alias: {scope}");
    let c = &v["canonical"]["config"];
    assert_eq!(c["orgs"].as_array().unwrap().len(), a.source.orgs.len());
    assert!(c["account_map"].is_string() && c["hash"].is_string());
    // never secrets, never paths
    let text = c.to_string();
    assert!(!text.contains('/') && !text.contains("token"), "{text}");
}

#[test]
fn a_reordered_config_resolves_to_the_same_frame() {
    for script in [
        "2 ] ]",
        "2 ] ] ]",
        "2 ] v down down space enter",
        "2 ] ] enter",
        "3 enter",
    ] {
        let mut a = script_app(theme::LEDGER_DARK, script, 100, 30);
        let live = render(&mut a);
        let mut source = a.source.clone();
        source.orgs.reverse();
        source.groups.reverse();
        assert_ne!(aliases(&source), aliases(&a.source));
        let mut b = App::from_state_json(&export(&a), source, false)
            .unwrap_or_else(|e| panic!("{script}: {}", e.message()));
        assert_eq!(b.drift, None, "{script}");
        assert_eq!(render(&mut b), live, "{script}: the same frame");
        assert_eq!(b.scope_label(&b.ui.scope), a.scope_label(&a.ui.scope));
    }
    // the scope picker lists scopes in config order: its picks and its cursor follow the aliases
    let mut a = script_app(theme::LEDGER_DARK, "2 ] v down down space", 100, 30);
    render(&mut a);
    let mut source = a.source.clone();
    source.orgs.reverse();
    let mut b = App::from_state_json(&export(&a), source, false).unwrap();
    render(&mut b);
    let picked = |x: &App| {
        let Some(ownpurse_tui::app::Overlay::Picker {
            query, sel, chosen, ..
        }) = &x.ui.overlay
        else {
            panic!("picker open")
        };
        let mut c: Vec<String> = chosen.iter().map(|i| x.source.orgs[*i].0.clone()).collect();
        c.sort();
        let at = x.scope_label(&x.picker_entries(query)[*sel].scope);
        (c, at)
    };
    let (chosen, at) = picked(&a);
    assert_eq!(chosen.len(), 2);
    assert_eq!(picked(&b), (chosen, at));
    assert_eq!(
        export(&App::from_state_json(&export(&b), b.source.clone(), false).unwrap()),
        export(&b)
    );
}

#[test]
fn a_removed_or_renamed_org_is_refused_unless_config_drift_is_allowed() {
    let mut a = script_app(theme::LEDGER_DARK, "2 ] ]", 100, 30);
    render(&mut a);
    let was = aliases(&a.source).join(",");
    // removed
    let mut removed = a.source.clone();
    let gone = removed.orgs.remove(1).0;
    for (_, m) in removed.groups.iter_mut() {
        m.retain(|x| *x != gone);
    }
    let e = App::from_state_json(&export(&a), removed.clone(), true)
        .err()
        .expect("refused even with record drift allowed");
    assert_eq!((e.code(), e.exit()), ("config_changed", 4));
    let now = aliases(&removed).join(",");
    assert!(
        e.message().contains(&format!("orgs: {was} → {now}")),
        "{}",
        e.message()
    );
    // renamed
    let mut renamed = a.source.clone();
    renamed.orgs[0].0 = "renamed".into();
    let e = App::from_state_json(&export(&a), renamed, false)
        .err()
        .unwrap();
    assert_eq!(e.code(), "config_changed");
    assert!(e.message().contains("renamed"), "{}", e.message());
    // another tenant under the same alias
    let mut moved = a.source.clone();
    moved.orgs[0].1 = "00000000-0000-0000-0000-000000000000".into();
    let e = App::from_state_json(&export(&a), moved, false)
        .err()
        .unwrap();
    assert!(e.message().contains("other tenant for"), "{}", e.message());
    // allowed: renders, drifted, the chip shows; an alias still present resolves, a missing one is `all`
    let allow = ownpurse_tui::canonical::AllowDrift {
        record: false,
        config: true,
        classify: false,
        manifest: false,
        live: false,
    };
    let mut b = App::from_state_json_with(&export(&a), removed.clone(), allow).unwrap();
    let d = b.drift.clone().expect("drifted");
    assert_eq!(d.from, d.to);
    assert!(d.config.as_deref().unwrap().contains("orgs: "));
    assert_eq!(b.view_state(b.rev).drift, Some(d));
    let row0 = snapshot::row_text(&render(&mut b), 0);
    assert!(row0.contains("≠ config changed"), "{row0}");
    let scope = b.scope_label(&b.ui.scope);
    let want = a.scope_label(&a.ui.scope);
    assert!(
        scope == want || (want == gone && scope == "all"),
        "{scope} vs {want}"
    );
    // a drifted state stays drifted, and needs the allowance again
    let c = App::from_state_json_with(&export(&b), removed.clone(), allow).unwrap();
    assert_eq!(c.drift, b.drift);
    let e = App::from_state_json(&export(&b), removed, false)
        .err()
        .unwrap();
    assert_eq!(e.code(), "config_changed");
}

#[test]
fn a_removed_scope_org_falls_back_to_all_and_compare_closes() {
    let mut a = script_app(theme::LEDGER_DARK, "2 ] v down down space enter", 100, 30);
    render(&mut a);
    let cmp = a.ui.cmp.clone().expect("compare open");
    let mut source = a.source.clone();
    let keep = cmp.orgs[0];
    let keep_alias = a.source.orgs[keep].0.clone();
    source.orgs.retain(|(x, _)| *x == keep_alias);
    source.groups.clear();
    let allow = ownpurse_tui::canonical::AllowDrift {
        record: false,
        config: true,
        classify: false,
        manifest: false,
        live: false,
    };
    let mut b = App::from_state_json_with(&export(&a), source, allow).unwrap();
    assert!(b.ui.cmp.is_none(), "a one-org compare closes");
    render(&mut b);
}

/// The frozen state fixtures, oldest first: `(version, dir)`.
fn frozen_dirs() -> Vec<(String, std::path::PathBuf)> {
    let root = snapshot::fixtures_dir().join("state");
    let mut v: Vec<(String, std::path::PathBuf)> = std::fs::read_dir(&root)
        .unwrap()
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let name = e.file_name().to_string_lossy().to_string();
            Some((name.strip_prefix('v')?.to_string(), e.path()))
        })
        .collect();
    v.sort_by(|a, b| {
        if ownpurse_tui::runtime::newer(&a.0, &b.0) {
            std::cmp::Ordering::Greater
        } else if a.0 == b.0 {
            std::cmp::Ordering::Equal
        } else {
            std::cmp::Ordering::Less
        }
    });
    v
}

#[test]
fn the_tab_row_allowance_is_exact() {
    let old = " 1 Overview  2 Ledger  3 Intercompany  4 Activity  5 System                ";
    let ok = " 1 Overview  2 Ledger  3 Intercompany  4 Activity  5 System  6 Classify    ";
    assert!(snapshot::tabs_row_extends(old, ok));
    assert!(snapshot::tabs_row_extends(old, old), "unchanged");
    for bad in [
        // an old tab label changed
        " 1 Overview  2 Ledgers 3 Intercompany  4 Activity  5 System  6 Classify    ",
        // tabs reordered
        " 1 Overview  2 Ledger  3 Intercompany  5 System  4 Activity  6 Classify    ",
        // a new tab inserted before the old ones end
        " 1 Overview  6 Classify  2 Ledger  3 Intercompany  4 Activity  5 System    ",
        // other content after the old tabs
        " 1 Overview  2 Ledger  3 Intercompany  4 Activity  5 System  ! changed     ",
        " 1 Overview  2 Ledger  3 Intercompany  4 Activity  5 System  6 Classify  x ",
        // an old tab dropped
        " 1 Overview  2 Ledger  3 Intercompany  4 Activity                          ",
    ] {
        assert!(!snapshot::tabs_row_extends(old, bad), "{bad:?}");
    }
    // only row 1 is forgiven: a change on any other row still fails (the same rule the upgrade pre-flight uses)
    let want = "header\n 1 Overview  2 Ledger\nbody\n";
    let got = "header\n 1 Overview  2 Ledger  6 Classify\nbody\n";
    assert_eq!(snapshot::frame_difference(got, want), None);
    let got2 = "header!\n 1 Overview  2 Ledger  6 Classify\nbody\n";
    assert_eq!(snapshot::frame_difference(got2, want).map(|d| d.0), Some(0));
    let got3 = "header\n 1 Overview  2 Ledgerx 6 Classify\nbody\n";
    assert_eq!(snapshot::frame_difference(got3, want).map(|d| d.0), Some(1));
    // ANSI frames: row 1 is compared as text, every other row byte for byte (styles included)
    let want = "h\n\x1b[1m 1 Overview\x1b[0m  2 Ledger   \x1b[0m\nb\n";
    let got = "h\n\x1b[1m 1 Overview\x1b[0m  2 Ledger  6 Classify\x1b[0m\nb\n";
    assert_eq!(snapshot::frame_difference(got, want), None);
    let restyled = "h\n\x1b[1m 1 Overview\x1b[0m  2 Ledger  6 Classify\x1b[0m\n\x1b[2mb\x1b[0m\n";
    assert_eq!(
        snapshot::frame_difference(restyled, want).map(|d| d.0),
        Some(2)
    );
}

/// Gate rule (permanent, v0.10.3 brief §5): every release reads the previous releases' states. Every
/// frozen fixture still imports (migrated when its schema is older) and draws its frozen frame; a release
/// freezes its own states before it ships (the newest fixture is never older than this build), so the
/// next release has them to read.
#[test]
fn every_frozen_state_still_imports() {
    let dirs = frozen_dirs();
    assert!(
        dirs.iter().any(|(v, _)| v == "0.10.2") && dirs.iter().any(|(v, _)| v == "0.10.3"),
        "{dirs:?}"
    );
    let newest = &dirs.last().unwrap().0;
    assert!(
        !ownpurse_tui::runtime::newer(ownpurse_tui::app::VERSION, newest),
        "v{} ships without its own frozen states (newest: v{newest}); run write_frozen_states",
        ownpurse_tui::app::VERSION
    );
    for (version, dir) in dirs {
        let mut n = 0;
        for e in std::fs::read_dir(&dir).unwrap().flatten() {
            let p = e.path();
            if p.extension().is_none_or(|x| x != "json") {
                continue;
            }
            let text = std::fs::read_to_string(&p).unwrap();
            let source = snapshot::fixture_source();
            let mut a = App::from_state_json(&text, source.clone(), false)
                .unwrap_or_else(|e| panic!("{}: {}", p.display(), e.message()));
            // an older render epoch (a visual redesign since): state only. Import, migrate, nothing lost, and
            // this build renders it self-consistently at every width (V21) without a panic.
            let doc: serde_json::Value = serde_json::from_str(&text).unwrap();
            let handed = doc.get("canonical").unwrap_or(&doc);
            if ownpurse_tui::upgrade::epoch_of(handed) != ownpurse_tui::upgrade::RENDER_EPOCH {
                assert_eq!(
                    ownpurse_tui::upgrade::lost(handed, &a.canonical_value(), "canonical"),
                    None,
                    "{}",
                    p.display()
                );
                ownpurse_tui::upgrade::self_consistent(&text, &source)
                    .unwrap_or_else(|e| panic!("{}: {e}", p.display()));
                n += 1;
                continue;
            }
            let frame = snapshot::text(&render(&mut a));
            let want = std::fs::read_to_string(p.with_extension("txt")).unwrap();
            assert_eq!(
                snapshot::frame_difference(&frame, &want),
                None,
                "{}",
                p.display()
            );
            n += 1;
        }
        assert!(n >= 2, "v{version}: {n} frozen states");
    }
}

/// The release whose states `write_frozen_states` freezes (the next release reads them).
const FREEZE: &str = "v0.16.0";

/// Writes tests/fixtures/state/<FREEZE> (run once per release with `--ignored`; the files are then frozen
/// and never regenerated: the next release must read them as they are).
#[test]
#[ignore]
fn write_frozen_states() {
    let dir = snapshot::fixtures_dir().join("state").join(FREEZE);
    assert!(
        !dir.exists(),
        "{} is frozen; bump FREEZE for a new release",
        dir.display()
    );
    std::fs::create_dir_all(&dir).unwrap();
    let cases = snapshot::frozen_state_cases(FREEZE.trim_start_matches('v'));
    for (name, mut a) in cases {
        render(&mut a);
        let mut v: serde_json::Value = serde_json::from_str(&export(&a)).unwrap();
        // no machine paths in a tracked fixture (the frames shown don't use them)
        v["canonical"]["env"]["cli"] = serde_json::json!("/usr/local/bin/ownpurse");
        v["canonical"]["env"]["paths"] = serde_json::json!({"cwd": "/w", "home": null});
        let text = ownpurse_core::canon::canonical(&v);
        let mut b = App::from_state_json(&text, a.source.clone(), false).unwrap();
        std::fs::write(dir.join(format!("{name}.json")), format!("{text}\n")).unwrap();
        std::fs::write(
            dir.join(format!("{name}.txt")),
            snapshot::text(&render(&mut b)),
        )
        .unwrap();
    }
}

/// `ownpurse-tui --state FILE` renders the frame from the state file alone (and refuses another record).
#[test]
fn the_binary_renders_a_state_file_alone() {
    let mut a = script_app(
        theme::LEDGER_DARK,
        "2 ] / T a l l o w m e r e space enter",
        100,
        30,
    );
    let live = snapshot::text(&render(&mut a));
    let dir = std::env::temp_dir().join(format!("xab-state-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let f = dir.join("state.json");
    std::fs::write(&f, a.view_state(a.rev).canonical()).unwrap();
    let cfg = snapshot::fixtures_dir().join("demo/ownpurse.json");
    let run = |extra: &[&str], file: &std::path::Path| {
        std::process::Command::new(env!("CARGO_BIN_EXE_ownpurse-tui"))
            .args([
                "--config",
                cfg.to_str().unwrap(),
                "--profile",
                "demo",
                "--state",
                file.to_str().unwrap(),
            ])
            .args(extra)
            .env_remove("XDG_RUNTIME_DIR")
            .output()
            .unwrap()
    };
    let out = run(&["--snapshot", "100x30", "--format", "txt"], &f);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(String::from_utf8(out.stdout).unwrap(), live);
    let png = dir.join("s.png");
    let out = run(&["--format", "png", "--out", png.to_str().unwrap()], &f);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(std::fs::read(&png).unwrap().starts_with(b"\x89PNG"));
    // another record version: refused (exit 4) unless drift is allowed
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
    v["canonical"]["record"]["chain_seq"] = serde_json::json!(3);
    let g = dir.join("old.json");
    std::fs::write(&g, v.to_string()).unwrap();
    let out = run(&["--format", "txt"], &g);
    assert_eq!(out.status.code(), Some(4));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(
        text.starts_with(
            "error: record_changed\nmessage: The state was exported against record version 3;"
        ),
        "{text}"
    );
    let out = run(&["--format", "txt", "--allow-record-drift"], &g);
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .contains("≠ record changed")
    );
    // another config (an org gone from the fingerprint): refused (exit 4) unless config drift is allowed
    let mut v: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
    v["canonical"]["config"]["orgs"]
        .as_array_mut()
        .unwrap()
        .push(serde_json::json!([
            "gone",
            "00000000-0000-0000-0000-000000000000"
        ]));
    v["canonical"]["config"]["hash"] = serde_json::json!("0");
    let h = dir.join("config.json");
    std::fs::write(&h, v.to_string()).unwrap();
    let out = run(&["--format", "txt", "--allow-record-drift"], &h);
    assert_eq!(out.status.code(), Some(4));
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.starts_with("error: config_changed\n"), "{text}");
    assert!(text.contains(",gone → "), "{text}");
    assert!(text.contains("--allow-config-drift"), "{text}");
    let out = run(&["--format", "txt", "--allow-config-drift"], &h);
    assert!(out.status.success());
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .lines()
            .next()
            .unwrap()
            .contains("≠ config changed")
    );
}

/// Live finding (agent-drive): the clock moves on (minute ticks) after the Overview model was built; a TUI
/// built from the state used the newer clock for its freshness lines and drew different "needs attention"
/// lines. The Overview now rebuilds when the minute changes, so live and imported frames agree.
#[test]
fn the_overview_follows_the_clock_so_an_imported_state_draws_the_same_lines() {
    let mut a = fixture_app(theme::LEDGER_DARK, false);
    snapshot::run_script(&mut a, "", 100, 30);
    let before = snapshot::text(&snapshot::render(&mut a, 100, 30));
    // two days later: roast's record is past its TTL
    let later = a.now + 2 * 86_400;
    ownpurse_tui::app::update(&mut a, ownpurse_tui::app::Msg::Tick(later));
    a.prepare();
    let live = snapshot::text(&snapshot::render(&mut a, 100, 30));
    assert_ne!(live, before, "the attention lines follow the clock");
    let mut b = import(&a, false);
    b.prepare();
    let imported = snapshot::text(&snapshot::render(&mut b, 100, 30));
    assert_eq!(live, imported);
}
