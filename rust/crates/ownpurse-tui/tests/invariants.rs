//! Invariants V1, V2, V3, V5, V12, V13 (spec §1), the M10 placeholder rules, and the shell's navigation
//! and mouse paths (acceptance N), against the fixture record with the clock pinned.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ownpurse_tui::app::{
    App, BandKind, Effect, Msg, NetAction, OFFLINE_REFUSAL, Overlay, Scope, Tab, update,
};
use ownpurse_tui::hit::Target;
use ownpurse_tui::keymap::{self, BINDINGS, Ctx, Key};
use ownpurse_tui::layout::{self, HeaderFacts};
use ownpurse_tui::model::Level;
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{
    self, fixture_app, header_problems, row_text, run_script, shell_states,
};
use ownpurse_tui::theme::{self, Theme};
use ratatui::Terminal;
use ratatui::TerminalOptions;
use ratatui::Viewport;
use ratatui::layout::Rect;
use ratatui::style::Color;

fn app() -> App {
    fixture_app(theme::LEDGER_DARK, false)
}

fn key(app: &mut App, k: &str) -> Vec<Effect> {
    let fx = update(app, Msg::Key(Key::parse(k).expect(k).event()));
    snapshot::render(app, 80, 24);
    fx
}

fn screen(app: &mut App, w: u16, h: u16) -> String {
    snapshot::text(&snapshot::render(app, w, h))
}

// ── V1: no write path ────────────────────────────────────────────────────────────────────────────

#[test]
fn v1_depends_only_on_core_and_terminal_crates() {
    let toml = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let deps = toml
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("\n[")
        .next()
        .unwrap();
    let allowed = [
        "ownpurse-core",
        "ratatui",
        "crossterm",
        "unicode-width",
        "unicode-segmentation",
        "rust_decimal",
        "serde_json",
        // the view state struct (`state::ViewState`, v0.10.1) derives Serialize; already in the tree via core
        "serde",
        "libc",
        // v0.11.0: OS file events for classify.toml hand edits (local only; no network, no timer)
        "notify",
        // v0.12.0: the user config (keys.toml, themes): spanned TOML and its revision hash (both in core)
        "toml_edit",
        "sha2",
        // manifests: word highlights in the generic diff view (already in the tree through insta)
        "similar",
        // overlays and walkthroughs: placement over our own drawing (pure: serde only, no I/O)
        "caretline-layers",
        "caretline-tour",
    ];
    for line in deps.lines().map(str::trim) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line.split(['=', '.', ' ']).next().unwrap().trim();
        assert!(allowed.contains(&name), "unexpected dependency {name}");
        for bad in ["write", "api", "http", "reqwest", "ureq", "hyper"] {
            assert!(!name.contains(bad), "{name}");
        }
    }
}

#[test]
fn v1_no_action_maps_to_a_write() {
    let forbidden = ["write", "post", "void", "journal", "delete", "approve"];
    let ids = BINDINGS
        .iter()
        .map(|b| b.action)
        .chain(keymap::ACTIONS.iter().map(|a| a.id));
    for id in ids {
        for f in forbidden {
            assert!(!id.contains(f), "action {id} looks like a write");
        }
    }
    for a in keymap::ACTIONS.iter().filter(|a| a.network) {
        if let Some(cli) = a.cli {
            assert!(!cli.contains("post") && !cli.contains("write"), "{cli}");
        }
    }
    // every palette entry, run, produces no effect at all but navigation or a refusal
    let base = app();
    for (i, e) in base.palette_entries().into_iter().enumerate() {
        let mut a = app();
        update(&mut a, Msg::Action("palette.open"));
        if let Some(Overlay::Palette { sel, .. }) = &mut a.ui.overlay {
            *sel = i;
        }
        let fx = update(&mut a, Msg::Action("palette.run"));
        assert!(
            fx.iter()
                .all(|f| !matches!(f, Effect::Network(_) | Effect::Spawn { .. })),
            "{} produced {fx:?}",
            e.label
        );
        if e.network {
            // choosing a network entry only opens the confirm bar (or a refusal); nothing runs yet
            assert!(
                matches!(
                    a.ui.band.as_ref().map(|b| &b.kind),
                    Some(BandKind::Refusal | BandKind::Ask { .. })
                ),
                "{}",
                e.label
            );
        }
    }
}

// ── V2: zero calls ───────────────────────────────────────────────────────────────────────────────

fn record_fingerprint() -> (u64, std::time::SystemTime, i64) {
    let path = snapshot::fixture_source().record;
    let meta = std::fs::metadata(&path).unwrap();
    let rec = ownpurse_core::Record::open(&path).unwrap();
    (
        meta.len(),
        meta.modified().unwrap(),
        rec.http_calls().unwrap(),
    )
}

#[test]
fn v2_random_sessions_make_no_network_effects() {
    let mut pool: Vec<&str> = BINDINGS
        .iter()
        .flat_map(|b| b.keys.iter().copied())
        .collect();
    pool.extend([
        "a",
        "o",
        "w",
        "n",
        "e",
        "s",
        "y",
        "x",
        "z",
        "space",
        "backspace",
        "enter",
        "tab",
    ]);
    pool.sort_unstable();
    pool.dedup();
    let before = record_fingerprint();
    for seed in 1..=5u64 {
        let mut a = app();
        snapshot::render(&mut a, 100, 30);
        let mut s = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
        let mut effects = Vec::new();
        for _ in 0..200 {
            s ^= s << 13;
            s ^= s >> 7;
            s ^= s << 17;
            let k = pool[(s % pool.len() as u64) as usize];
            let fx = update(&mut a, Msg::Key(Key::parse(k).unwrap().event()));
            effects.extend(fx.into_iter().filter(|f| matches!(f, Effect::Network(_))));
            snapshot::render(&mut a, 100, 30);
        }
        assert!(effects.is_empty(), "seed {seed}: {effects:?}");
    }
    assert_eq!(record_fingerprint(), before, "the record was touched");
}

#[test]
fn v2_runtime_counts_zero_network_effects_over_a_session() {
    let mut rt = runtime(false);
    for k in [
        "ctrl-p", "s", "y", "n", "c", "enter", "enter", "2", "]", "g", "enter", "?", "esc", "r",
    ] {
        rt.send(Msg::Key(Key::parse(k).unwrap().event()));
        rt.frame().unwrap();
    }
    assert_eq!(rt.network_effects, 0);
}

// ── V3: network actions: palette only, confirm with focus on Cancel, no single-letter confirm ─────

#[test]
fn v3_network_actions_have_no_keys_and_confirm_is_sealed() {
    for a in keymap::ACTIONS.iter().filter(|a| a.network) {
        assert!(
            !BINDINGS.iter().any(|b| b.action == a.id),
            "{} has a key",
            a.id
        );
    }
    let allowed = ["tab", "shift-tab", "left", "right", "enter", "esc"];
    for b in BINDINGS.iter().filter(|b| b.ctx == Ctx::Confirm) {
        for k in b.keys {
            assert!(allowed.contains(k), "confirm binds {k}");
        }
    }
    assert!(keymap::conflicts().is_empty(), "{:?}", keymap::conflicts());
}

#[test]
fn v3_enter_on_the_band_cancels_and_tab_enter_never_reaches_xero() {
    let mut a = app();
    a.open_confirm(NetAction::SyncOrg(1), None);
    assert_eq!(
        a.ui.band.as_ref().unwrap().focus,
        0,
        "focus starts on Cancel"
    );
    assert!(key(&mut a, "enter").is_empty());
    assert!(a.ui.band.is_none(), "Enter on Cancel closes");

    a.open_confirm(NetAction::SyncOrg(1), None);
    key(&mut a, "tab");
    assert_eq!(a.ui.band.as_ref().unwrap().focus, 1);
    // Tab+Enter runs the installed CLI (P6 network): one Spawn of the resolved binary, never a
    // Network effect from the TUI itself.
    let fx = key(&mut a, "enter");
    assert!(
        fx.iter().all(|f| !matches!(f, Effect::Network(_))),
        "{fx:?}"
    );
    assert!(
        matches!(fx.as_slice(), [Effect::Spawn { args, .. }] if args[..2] == ["sync", "hold"]),
        "{fx:?}"
    );
    assert!(a.ui.band.is_none());
    assert!(a.ui.run.is_some());
}

#[test]
fn v3_a_stray_letter_in_the_band_does_nothing() {
    for (setup, name) in [
        (
            Box::new(|a: &mut App| a.open_confirm(NetAction::SyncAll, None))
                as Box<dyn Fn(&mut App)>,
            "ask",
        ),
        (
            Box::new(|a: &mut App| a.request_network(NetAction::SyncAll)),
            "refusal",
        ),
    ] {
        let mut a = app();
        setup(&mut a);
        let band = a.ui.band.clone();
        for c in "yYnqgs12345[]/?:dhrv!".chars() {
            let fx = update(
                &mut a,
                Msg::Key(KeyEvent::new(KeyCode::Char(c), KeyModifiers::NONE)),
            );
            assert!(fx.is_empty(), "{name}: {c} → {fx:?}");
            assert_eq!(a.ui.band, band, "{name}: {c} changed the band");
            assert_eq!(a.ui.tab, Tab::Overview);
        }
    }
}

// ── M10: the pre-API placeholder ─────────────────────────────────────────────────────────────────

#[test]
fn m10_placeholder_texts_are_exact() {
    let mut a = app();
    a.ui.scope = Scope::Org(1);
    // sync and login now confirm and run the CLI (tests/net.rs); the rest keep their placeholders
    let cases = [
        (
            NetAction::RefetchTb,
            "Not available yet (coming in v0.9.x).",
        ),
        (
            NetAction::FetchHistory,
            "Not available yet (coming in v0.9.x).",
        ),
        (
            NetAction::FetchAttach,
            "Not available yet (coming in v0.9.x).",
        ),
    ];
    for (n, text) in cases {
        a.request_network(n);
        let b = a.ui.band.clone().unwrap();
        assert_eq!(b.text, text);
        assert_eq!(b.kind, BandKind::Refusal);
        assert_eq!(b.buttons(), vec!["Close"]);
        assert_eq!(b.focus, 0, "Close has focus");
        // Esc closes; no action button
        update(&mut a, Msg::Key(Key::parse("esc").unwrap().event()));
        assert!(a.ui.band.is_none());
    }
}

#[test]
fn m10_band_is_error_coloured_and_persistent() {
    let mut a = app();
    run_script(&mut a, "ctrl-p r e f e t c h enter", 80, 24);
    let buf = snapshot::render(&mut a, 80, 24);
    let row = (0..24)
        .find(|&y| row_text(&buf, y).contains("Not available yet (coming in v0.9.x)"))
        .unwrap();
    let x = row_text(&buf, row).find("Not available").unwrap() as u16;
    assert_eq!(
        buf[(x, row)].fg,
        ownpurse_tui::theme::LEDGER_DARK
            .fg(ownpurse_tui::theme_tokens::Token::Error)
            .fg
            .unwrap(),
        "error colour"
    );
    assert!(
        (row..24).any(|y| row_text(&buf, y).contains("Close")),
        "Close sits on the band (the long refetch copy may wrap)"
    );
    // persistent: a minute passing and a resize keep it
    let later = a.now + 600;
    update(&mut a, Msg::Tick(later));
    update(&mut a, Msg::Resize(100, 30));
    assert!(a.ui.band.is_some());
}

#[test]
fn m10_offline_hides_network_entries_and_offline_refusal_wins() {
    let mut a = fixture_app(theme::LEDGER_DARK, true);
    assert!(a.palette_entries().iter().all(|e| !e.network));
    run_script(&mut a, "ctrl-p", 140, 40);
    let text = screen(&mut a, 140, 40);
    for word in ["Sync", "Log in", "Refetch", "Fetch", "⇅"] {
        assert!(
            !text.contains(word),
            "offline palette shows {word}:\n{text}"
        );
    }
    a.request_network(NetAction::SyncOrg(0));
    assert_eq!(a.ui.band.as_ref().unwrap().text, OFFLINE_REFUSAL);
    // online, no network entry carries a suffix any more
    let mut b = app();
    run_script(&mut b, "ctrl-p", 140, 40);
    let text = screen(&mut b, 140, 40);
    for e in b.palette_entries().iter().filter(|e| e.network) {
        let line = text.lines().find(|l| l.contains(&e.label)).unwrap();
        assert!(!line.contains("for now") && !line.contains("CLI"), "{line}");
    }
}

// ── V5: the header is always whole ───────────────────────────────────────────────────────────────

#[test]
fn v5_header_whole_at_every_width_on_every_screen_and_scope() {
    let mut problems = Vec::new();
    for offline in [false, true] {
        for st in shell_states().into_iter().filter(|s| s.name != "too-small") {
            let mut base = snapshot::state_app(&st, theme::LEDGER_DARK, 100, 30);
            base.offline = offline || st.offline;
            for scope in base.scopes() {
                base.ui.scope = scope.clone();
                for w in 60..=220u16 {
                    let buf = snapshot::render(&mut base, w, 20);
                    for p in header_problems(&base, &row_text(&buf, 0)) {
                        problems.push(format!(
                            "{} {scope:?} offline={offline} w={w}: {p}",
                            st.name
                        ));
                    }
                }
            }
        }
    }
    assert!(
        problems.is_empty(),
        "{}",
        problems[..problems.len().min(20)].join("\n")
    );
}

fn facts(
    scope_full: &str,
    scope_compact: &str,
    offline: bool,
    fresh: (&str, &str, &str),
    left: i64,
) -> HeaderFacts {
    HeaderFacts {
        product: "ownpurse · default · v0.9.0-dev".into(),
        offline,
        scope_full: scope_full.into(),
        scope_compact: scope_compact.into(),
        scope_min: String::new(),
        as_of: "2025-12-31".into(),
        fresh_full: fresh.0.into(),
        fresh_compact: fresh.1.into(),
        fresh_min: fresh.2.into(),
        fresh_level: Level::Warn,
        left,
        limit: 1000,
        quota_level: Level::Ok,
        left_estimated: false,
        busy: false,
        run: None,
        agent: None,
        plain: false,
        drift: None,
        classify: None,
        show_age: true,
        show_quota: true,
        writes: false,
        manifest: None,
    }
}

#[test]
fn v5_worst_case_headers_fit_from_60_columns() {
    let scopes = [
        ("org: all ▾", "all ▾"),
        ("org: lab ▾", "lab ▾"),
        ("org: group:all-co ▾ (4)", "all-co (4) ▾"),
        (
            "org: compare: roast · hold · design · lab ▾",
            "compare (4) ▾",
        ),
    ];
    let freshes = [
        (
            "record 07:46 UTC · 12d ago stale",
            "07:46Z · 12d stale",
            "12d stale",
        ),
        ("no local record", "no local record", "no record"),
    ];
    for ascii in [false, true] {
        for (full, compact) in scopes {
            for fresh in freshes {
                for offline in [false, true] {
                    for (left, estimated) in [(1000, false), (977, false), (5, false), (1000, true)]
                    {
                        let mut f = facts(full, compact, offline, fresh, left);
                        // the local fallback figure carries its `≈` mark at every width
                        f.left_estimated = estimated;
                        // the agent mark never breaks the fit
                        f.agent = (left == 977)
                            .then_some(true)
                            .or((left == 5).then_some(false));
                        for w in 60..=220usize {
                            let parts = layout::header(&f, w, ascii);
                            let text: String = parts.iter().map(|(s, _)| s.as_str()).collect();
                            assert!(layout::width(&text) <= w, "{w}: {text}");
                            let fold = |s: &str| ownpurse_tui::glyphs::fold(s, ascii);
                            // v0.12.0: the quiet read-only mark ends the row; offline is an exception
                            assert!(text.trim_end().ends_with("read-only"), "{w}: {text}");
                            if offline {
                                assert!(text.contains("offline"), "{w}: {text}");
                            }
                            assert!(
                                text.contains(&fold(full)) || text.contains(&fold(compact)),
                                "{w}: {text}"
                            );
                            assert!(text.contains("2025-12-31"));
                            assert!(
                                text.contains(fresh.2.split(' ').next().unwrap())
                                    || text.contains("record"),
                                "{w}: {text}"
                            );
                            assert!(
                                text.contains(&ownpurse_tui::clock::int(left)),
                                "{w}: {text}"
                            );
                            if estimated {
                                assert!(text.contains(&fold("≈")), "{w}: {text}");
                            }
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn v5_v080_regressions() {
    // compare-2 at 140 keeps the whole header; compare-4 at 100 keeps "left"
    let f2 = facts(
        "org: compare: roast · design ▾",
        "compare (2) ▾",
        false,
        ("record 07:46 UTC · 2h ago", "07:46Z · 2h", "2h"),
        938,
    );
    let t: String = layout::header(&f2, 140, false)
        .into_iter()
        .map(|(s, _)| s)
        .collect();
    assert!(
        t.contains("read-only") && t.contains("compare") && t.contains("938/1,000 left"),
        "{t}"
    );
    let f4 = facts(
        "org: compare: roast · hold · design · lab ▾",
        "compare (4) ▾",
        false,
        ("record 07:46 UTC · 2h ago", "07:46Z · 2h", "2h"),
        938,
    );
    let t: String = layout::header(&f4, 100, false)
        .into_iter()
        .map(|(s, _)| s)
        .collect();
    assert!(t.contains("938") && t.contains("left"), "{t}");
}

// ── V12: never colour alone ──────────────────────────────────────────────────────────────────────

#[test]
fn v12_no_color_states_read_by_glyphs_and_words() {
    let nc = theme::NO_COLOR_ASCII;
    let mut a = fixture_app(nc, false);
    a.model.orgs[0].last_sync = None; // one never synced
    a.invalidate();
    let buf = snapshot::render(&mut a, 80, 24);
    for y in 0..24 {
        for x in 0..80 {
            let c = &buf[(x, y)];
            assert!(
                c.fg == Color::Reset && c.bg == Color::Reset,
                "colour at {x},{y}"
            );
        }
    }
    let t = snapshot::text(&buf);
    assert!(
        row_text(&buf, 2).starts_with("--=========="),
        "active tab marked by '=': {}",
        row_text(&buf, 2)
    );
    assert!(t.contains("> roast"), "cursor row gutter");
    assert!(t.contains("no local record"), "never synced is a word");
    assert!(
        t.contains("! No local record for roast yet."),
        "attention carries '!'"
    );
    assert!(t.contains("^P palette"));
    let mut a = fixture_app(nc, true);
    a.open_confirm(NetAction::SyncOrg(1), None);
    let t = screen(&mut a, 80, 24);
    assert!(
        t.contains(">Cancel") && t.contains(" Sync "),
        "focused button carries the cursor glyph:\n{t}"
    );
    let row0 = row_text(&snapshot::render(&mut a, 80, 24), 0);
    assert!(
        row0.contains("offline") && row0.trim_end().ends_with("read-only"),
        "{row0}"
    );
    let mut a = fixture_app(nc, false);
    a.ui.scope = Scope::Org(1);
    a.model.ttl = 60; // make hq stale
    let t = screen(&mut a, 100, 30);
    assert!(t.contains("stale"), "stale is a word:\n{t}");
}

// ── V13: idle writes nothing; only a state change redraws ────────────────────────────────────────

fn runtime(sync: bool) -> Runtime<Quiet<Counting>> {
    runtime_with(sync, Counting::default())
}

fn runtime_with(sync: bool, out: Counting) -> Runtime<Quiet<Counting>> {
    let backend = Quiet::fixed(out, sync, 80, 24);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 80, 24)),
        },
    )
    .unwrap();
    Runtime::new(terminal, app())
}

#[test]
fn v13_idle_writes_zero_bytes_and_redraws_only_on_change() {
    let out = Counting::default();
    let mut rt = runtime_with(true, out.clone());
    assert!(rt.frame().unwrap());
    let first = out.bytes();
    assert!(first > 0);
    let data = String::from_utf8_lossy(&out.data()).into_owned();
    assert!(
        data.contains("\x1b[?2026h") && data.contains("\x1b[?2026l"),
        "synchronized output"
    );
    // 5 s of simulated idle: ticks every 250 ms within the same minute
    let t0 = rt.app.now;
    for i in 1..=20 {
        rt.send(Msg::Tick(t0 + i / 4));
        assert!(!rt.frame().unwrap(), "redraw while idle");
    }
    assert_eq!(out.bytes(), first, "idle wrote bytes");
    // an unbound key changes nothing: no redraw
    rt.send(Msg::Key(KeyEvent::new(
        KeyCode::Char('a'),
        KeyModifiers::NONE,
    )));
    assert!(!rt.frame().unwrap());
    assert_eq!(out.bytes(), first);
    // the minute turns: one redraw, but nothing visible changed, so nothing is written
    rt.send(Msg::Tick(t0 + 61));
    assert!(rt.frame().unwrap());
    assert_eq!(out.bytes(), first, "an unchanged frame wrote bytes");
    // a state change redraws once and writes
    let draws = rt.draws;
    rt.send(Msg::Key(Key::parse("2").unwrap().event()));
    assert!(rt.frame().unwrap());
    assert!(!rt.frame().unwrap());
    assert_eq!(rt.draws, draws + 1);
    assert!(out.bytes() > first);
}

#[test]
fn v13_without_sync_support_frames_are_plain() {
    let out = Counting::default();
    let mut rt = runtime_with(false, out.clone());
    rt.frame().unwrap();
    let data = String::from_utf8_lossy(&out.data()).into_owned();
    assert!(!data.contains("\x1b[?2026h"));
}

// ── footer (§3.2, §6) ────────────────────────────────────────────────────────────────────────────

#[test]
fn footer_ends_with_palette_and_globals_go_key_only_when_narrow() {
    for (w, h) in [(60, 20), (80, 24), (100, 30), (140, 40)] {
        for st in shell_states()
            .into_iter()
            .filter(|s| ["overview", "ledger", "ledger-roast", "system"].contains(&s.name))
        {
            for th in [theme::LEDGER_DARK, theme::NO_COLOR_ASCII] {
                let mut a = snapshot::state_app(&st, th, w, h);
                let buf = snapshot::render(&mut a, w, h);
                let last = row_text(&buf, h - 1);
                let want = if th.ascii {
                    "^P palette"
                } else {
                    "⌃P palette"
                };
                // then the version, the protected last item
                // v0.12.0: the keys start at the left inset; the palette hint is their last, the version ends the row
                assert!(
                    last.contains(want)
                        && last
                            .trim_end()
                            .ends_with(&format!("v{}", snapshot::TEST_VERSION)),
                    "{} {w}: {last}",
                    st.name
                );
                if w < 100 {
                    assert!(
                        !last.contains("Filter") && !last.contains("Scope"),
                        "{w}: {last}"
                    );
                } else {
                    // at most five keys (design pass 2): the screen's first, then the global ones
                    assert!(last.contains("/ Filter"), "{w}: {last}");
                }
            }
        }
    }
}

#[test]
fn footer_label_of_h_names_the_target_basis() {
    let mut a = app();
    run_script(&mut a, "2 ] ]", 140, 40);
    assert!(
        screen(&mut a, 140, 40)
            .lines()
            .last()
            .unwrap()
            .contains("h Full history")
    );
    run_script(&mut a, "h", 140, 40);
    assert!(
        screen(&mut a, 140, 40)
            .lines()
            .last()
            .unwrap()
            .contains("h Year basis")
    );
}

// ── navigation and mouse (acceptance N) ──────────────────────────────────────────────────────────

#[test]
fn n1_arriving_never_takes_the_cursor() {
    let mut a = app();
    key(&mut a, "2");
    a.dirty = false;
    key(&mut a, "a");
    assert!(a.ui.prompt.is_none() && a.ui.overlay.is_none());
    assert!(!a.dirty, "an unbound key is not a state change");
    // a prompt never survives a tab switch by click
    key(&mut a, "1");
    key(&mut a, "/");
    let r = a.hits.find(Target::Tab(Tab::System)).unwrap();
    update(&mut a, Msg::Action("tab.system"));
    assert!(a.ui.prompt.is_none(), "{r:?}");
}

#[test]
fn n2_n3_drill_from_overview_and_esc_returns_to_origin() {
    let mut a = app();
    key(&mut a, "j"); // hq
    key(&mut a, "enter");
    assert_eq!(a.ui.tab, Tab::Ledger);
    assert_eq!(a.ui.scope, Scope::Org(1));
    assert!(
        screen(&mut a, 80, 24)
            .lines()
            .nth(3)
            .unwrap()
            .contains("Overview › hold › Trial balance at 2025-12-31 (accrual)")
    );
    key(&mut a, "esc");
    assert_eq!(a.ui.tab, Tab::Overview);
    assert_eq!(a.ui.scope, Scope::All);
    assert_eq!(a.ui.overview_sel.as_deref(), Some("hold"), "same row");
}

#[test]
fn n5_scope_round_trip_keeps_the_cursor_id() {
    let mut a = app();
    key(&mut a, "j");
    key(&mut a, "]");
    key(&mut a, "[");
    assert_eq!(a.ui.overview_sel.as_deref(), Some("hold"));
    assert_eq!(a.ui.scope, Scope::All);
}

#[test]
fn n6_n7_n8_clicks_run_the_same_actions_as_keys() {
    let mut a = app();
    snapshot::render(&mut a, 100, 30);
    let r = a.hits.find(Target::Tab(Tab::Intercompany)).unwrap();
    run_script(&mut a, &format!("<click:{},{}>", r.x + 1, r.y), 100, 30);
    assert_eq!(a.ui.tab, Tab::Intercompany);
    // a row click selects; the next key does what it would have done
    let mut a = app();
    snapshot::render(&mut a, 100, 30);
    let row = a.hits.find(Target::Row(2)).unwrap();
    run_script(&mut a, &format!("<click:5,{}>", row.y), 100, 30);
    assert_eq!(a.ui.overview_sel.as_deref(), Some("design"));
    assert_eq!(a.ui.tab, Tab::Overview, "a click never drills");
    run_script(&mut a, "k", 100, 30);
    assert_eq!(a.ui.overview_sel.as_deref(), Some("hold"));
    // double-click drills
    run_script(&mut a, &format!("<dclick:5,{}>", row.y), 100, 30);
    assert_eq!((a.ui.tab, a.ui.scope.clone()), (Tab::Ledger, Scope::Org(2)));
}

#[test]
fn n9_clicking_a_footer_key_runs_its_action() {
    let mut a = app();
    snapshot::render(&mut a, 120, 30);
    let r = a.hits.find(Target::Action("scope.picker")).unwrap();
    // the header scope is registered first; the footer's g is the second target with that action
    let footer = a
        .hits
        .targets
        .iter()
        .find(|(rect, t)| *t == Target::Action("scope.picker") && rect.y == 29)
        .map(|(rect, _)| *rect)
        .unwrap();
    assert_ne!(r, footer);
    run_script(
        &mut a,
        &format!("<click:{},{}>", footer.x, footer.y),
        120,
        30,
    );
    assert!(matches!(a.ui.overlay, Some(Overlay::Picker { .. })));
}

#[test]
fn n11_click_outside_an_overlay_closes_it_and_nothing_else() {
    let mut a = app();
    run_script(&mut a, "ctrl-p", 100, 30);
    let tab = a.hits.find(Target::Tab(Tab::System)).unwrap();
    run_script(&mut a, &format!("<click:{},{}>", tab.x, tab.y), 100, 30);
    assert!(a.ui.overlay.is_none());
    assert_eq!(a.ui.tab, Tab::Overview, "the click did nothing underneath");
}

#[test]
fn picker_ranks_alias_prefix_first_and_switches() {
    let mut a = app();
    run_script(&mut a, "g l a b", 80, 24);
    let entries = match &a.ui.overlay {
        Some(Overlay::Picker { query, .. }) => a.picker_entries(query),
        _ => panic!("picker"),
    };
    assert_eq!(entries[0].label, "lab");
    run_script(&mut a, "enter", 80, 24);
    assert_eq!(a.ui.scope, Scope::Org(3));
    assert!(row_text(&snapshot::render(&mut a, 80, 24), 0).contains("lab ▾"));
}

#[test]
fn as_of_prompt_validates_and_applies() {
    let mut a = app();
    run_script(&mut a, "2 d backspace backspace 3 0 enter", 100, 30);
    assert_eq!(a.ui.as_of, "2025-12-30");
    run_script(&mut a, "d x enter", 100, 30);
    assert_eq!(a.ui.as_of, "2025-12-30", "invalid date keeps the prompt");
    assert!(a.ui.prompt.is_some());
    run_script(&mut a, "esc", 100, 30);
    assert!(a.ui.prompt.is_none());
}

#[test]
fn filter_narrows_overview_rows_incrementally() {
    let mut a = app();
    run_script(&mut a, "/ l a b", 100, 30);
    assert_eq!(a.overview_rows().len(), 1);
    run_script(&mut a, "enter", 100, 30);
    assert!(a.ui.prompt.is_none());
    assert_eq!(a.ui.filter, "lab");
    run_script(&mut a, "esc", 100, 30);
    assert!(a.ui.filter.is_empty(), "Esc on the view clears the filter");
}

#[test]
fn reload_is_an_effect_and_zero_calls() {
    let mut rt = runtime(false);
    rt.frame().unwrap();
    rt.send(Msg::Key(Key::parse("r").unwrap().event()));
    assert_eq!(rt.app.model.orgs.len(), 4);
    assert!(rt.frame().unwrap(), "reload redraws");
    assert_eq!(rt.network_effects, 0);
}

#[test]
fn quit_is_an_effect() {
    let mut a = app();
    assert_eq!(key(&mut a, "q"), vec![Effect::Quit]);
}

#[test]
fn too_small_says_so() {
    let mut a = app();
    let t = screen(&mut a, 52, 18);
    assert_eq!(t.trim(), "ownpurse needs at least 60×20 · now 52×18");
}

#[test]
fn theme_cycle_from_the_palette() {
    let mut a = app();
    run_script(&mut a, "ctrl-p t h e m e enter", 80, 24);
    assert_eq!(
        a.ui.theme,
        Theme {
            ..theme::LEDGER_LIGHT
        }
    );
}

#[test]
fn keys_flag_prints_the_keymap_markdown() {
    // an empty user config folder: the built-in table (never the real ~/.config/ownpurse)
    let empty = std::env::temp_dir().join(format!("xa-keys-empty-{}", std::process::id()));
    std::fs::create_dir_all(&empty).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_ownpurse-tui"))
        .arg("--keys")
        .env("OWNPURSE_CONFIG_DIR", &empty)
        .output()
        .unwrap();
    assert!(out.status.success());
    assert_eq!(
        String::from_utf8(out.stdout).unwrap(),
        keymap::to_markdown()
    );
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_ownpurse-tui"))
        .args(["--keys", "conflicts"])
        .env("OWNPURSE_CONFIG_DIR", &empty)
        .output()
        .unwrap();
    assert!(out.status.success() && out.stdout.is_empty());
}

#[test]
fn band_cells_carry_only_the_bands_own_style() {
    use ownpurse_tui::theme_tokens::Token;
    for (name, th) in snapshot::themes() {
        for refusal in [false, true] {
            let mut a = fixture_app(th, false);
            run_script(&mut a, "2 ] j j", 80, 24); // a selected row on the Ledger
            if refusal {
                a.request_network(NetAction::SyncOrg(0));
            } else {
                a.open_confirm(NetAction::SyncOrg(0), Some(15));
            }
            let buf = snapshot::render(&mut a, 80, 24);
            let raised = th.raised();
            let allowed: Vec<ratatui::style::Style> = [
                raised,
                th.fg(Token::Text).patch(raised),
                th.fg(Token::Error).patch(raised),
                th.fg(Token::Muted).patch(raised),
                th.focused_button(),
            ]
            .iter()
            .map(|s| raised.patch(*s)) // spans paint over the band's own fill
            .collect();
            let band_rows: Vec<u16> = (0..23)
                .filter(|&y| {
                    let t = row_text(&buf, y);
                    t.contains("Sync") && (t.contains("Close") || t.contains("Cancel"))
                        || t.contains("from Xero?")
                })
                .collect();
            assert!(!band_rows.is_empty(), "{name}: no band");
            for y in band_rows {
                for x in 0..80 {
                    let c = &buf[(x, y)];
                    let st = ratatui::style::Style::default()
                        .fg(c.fg)
                        .bg(c.bg)
                        .add_modifier(c.modifier);
                    let ok = allowed.iter().any(|s| {
                        s.fg.unwrap_or(ratatui::style::Color::Reset) == c.fg
                            && s.bg.unwrap_or(ratatui::style::Color::Reset) == c.bg
                            && s.add_modifier == c.modifier
                    });
                    assert!(
                        ok,
                        "{name} refusal={refusal}: cell {x},{y} {st:?} not a band style"
                    );
                }
            }
        }
    }
}

// ── spec §9: calls-left and reset wording ────────────────────────────────────────────────────────

/// Xero's daily window resets per tenant at its own time: none of the banned phrases below (a calls-left
/// figure qualified by the day, a fixed midnight-UTC reset, a rolling window) in any Rust crate's src,
/// tests or snapshots, or the fake CLI. ("calls today" as a count since 00:00 UTC is fine.)
#[test]
fn s9_no_left_today_or_fixed_reset_wording() {
    let banned: Vec<String> = [
        ["left", " today"],
        ["resets 00:00", " utc"],
        ["rolling", " 24"],
    ]
    .iter()
    .map(|p| p.concat())
    .collect();
    let rust = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut roots = vec![rust.join("tests/fixtures/fake-cli")];
    for c in std::fs::read_dir(rust.join("crates")).unwrap().flatten() {
        roots.push(c.path().join("src"));
        roots.push(c.path().join("tests"));
    }
    let mut files = Vec::new();
    while let Some(p) = roots.pop() {
        if p.is_dir() {
            roots.extend(std::fs::read_dir(&p).unwrap().flatten().map(|e| e.path()));
        } else if p.is_file() {
            files.push(p);
        }
    }
    let mut hits = Vec::new();
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else {
            continue; // binary fixtures (records)
        };
        for (n, line) in text.lines().enumerate() {
            let l = line.to_lowercase();
            for b in &banned {
                if l.contains(b.as_str()) {
                    hits.push(format!("{}:{}: {}", f.display(), n + 1, line.trim()));
                }
            }
        }
    }
    assert!(files.len() > 100, "scanned {} files", files.len());
    assert!(hits.is_empty(), "{}", hits.join("\n"));
}

#[test]
fn r1_no_screen_sends_anyone_to_a_retired_tool() {
    // no capture may name a tool that no longer exists, or promise what isn't there
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots");
    let banned: Vec<String> = [
        ["ownpurse", "-py"].concat(),
        ["Py", "thon"].concat(),
        "for now".into(),
        ["Not in the ", "Rust CLI"].concat(),
    ]
    .to_vec();
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        let text = std::fs::read_to_string(&p).unwrap_or_default();
        for bad in &banned {
            assert!(!text.contains(bad), "{} mentions {bad:?}", p.display());
        }
    }
}
