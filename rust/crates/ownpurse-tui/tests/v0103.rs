//! v0.10.3 polish and the update-ready footer (§2): money twins, `back`/`forward` with no
//! history, the `⬆ vX ready · ⌃P Upgrade` copy at every width.

use ownpurse_tui::agent::{display_value, is_money};
use ownpurse_tui::app::{App, Msg};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::quiet::{Counting, Quiet};
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot::{self, fixture_app};
use ownpurse_tui::theme;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use serde_json::{Value, json};

type Rt = Runtime<Quiet<Counting>>;

fn rt() -> Rt {
    let backend = Quiet::fixed(Counting::default(), true, 100, 30);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, 100, 30)),
        },
    )
    .unwrap();
    let mut rt = Runtime::new(terminal, fixture_app(theme::LEDGER_DARK, false));
    rt.frame().unwrap();
    rt
}

// ── §7.2: a `_value` twin for every displayed amount ──────────────────────────────────────────────

/// Every screen with money columns: Ledger L1 (all and differs), L2, Compare (wide and narrow),
/// Intercompany L1 and items.
const MONEY_SCREENS: [&str; 7] = [
    "2 ] ] j enter",
    "2 ]",
    "2 ] d",
    "2 ] j enter",
    "2 ] v down down space enter",
    "3",
    "3 j enter",
];

#[test]
fn every_money_column_has_a_value_twin_equal_to_its_display() {
    use ownpurse_tui::views::{intercompany as ic, ledger};
    let money_titles: Vec<&str> = [
        &ledger::L1_COLS[..],
        &ledger::L1_DIFF_COLS[..],
        &ledger::L2_COLS[..],
        &ic::L1_COLS[..],
        &ic::ITEM_COLS[..],
    ]
    .concat()
    .iter()
    .filter(|c| is_money(c))
    .map(|c| c.title)
    .collect();
    assert!(money_titles.contains(&"debit") && money_titles.contains(&"residual"));
    let mut checked = 0;
    for (w, h) in [(140u16, 40u16), (80, 24)] {
        for script in MONEY_SCREENS {
            let mut a = fixture_app(theme::LEDGER_DARK, false);
            snapshot::run_script(&mut a, script, w, h);
            let st = a.state_json_rows(1, ownpurse_tui::state::Rows::All);
            let rows = st["rows"].as_array().unwrap();
            assert!(!rows.is_empty(), "{script}");
            for r in rows {
                let f = r["fields"].as_object().unwrap();
                for (k, v) in f {
                    let Some(text) = v.as_str() else { continue };
                    let money = display_value(text).is_some()
                        && (text.contains('.') && !k.ends_with("_value"));
                    let twin = f.get(&format!("{k}_value"));
                    if money {
                        let twin = twin.unwrap_or_else(|| {
                            panic!("{script}: `{k}` ({text}) has no twin: {f:?}")
                        });
                        assert_eq!(
                            twin.to_string(),
                            display_value(text).unwrap().to_string(),
                            "{script}: {k} = {text}"
                        );
                        checked += 1;
                    }
                    if let Some(t) = twin {
                        // a blank cell's twin is null; a shown amount's twin is an exact number
                        assert!(
                            t.is_null() == display_value(text).is_none(),
                            "{script}: {k}={text} → {t}"
                        );
                    }
                }
            }
        }
    }
    assert!(checked > 100, "{checked} twins checked");
}

#[test]
fn display_strings_parse_with_their_signs() {
    let v = |s: &str| display_value(s).map(|d| d.to_string());
    assert_eq!(v("7,036.41").as_deref(), Some("7036.41"));
    assert_eq!(v("(7,036.41)").as_deref(), Some("-7036.41"));
    assert_eq!(v("−12.50").as_deref(), Some("-12.50"));
    assert_eq!(v("Dr 1,234.56").as_deref(), Some("1234.56"));
    assert_eq!(v("roast Cr 5,318.47").as_deref(), Some("-5318.47"));
    assert_eq!(v("0.00").as_deref(), Some("0.00"));
    for none in ["", "—", "967/1,000", "12", "2025-12-31", "1,000.5"] {
        assert_eq!(v(none), None, "{none}");
    }
}

// ── §7.3: back / forward with nothing to go to ───────────────────────────────────────────────────

#[test]
fn back_and_forward_with_no_history_say_so_without_a_new_revision() {
    for (verb, text) in [
        ("back", "nothing to go back to"),
        ("forward", "nothing to go forward to"),
    ] {
        let mut rt = rt();
        let rev = rt.app.rev;
        let r = rt.agent(&json!({"v": 1, "kind": "view", "op": "send", "verbs": [[verb]]}));
        assert_eq!(r["ok"], true, "{r}");
        assert_eq!(r["changed"], json!([]));
        assert_eq!(r["note"], "no_history");
        assert_eq!(r["message"], text);
        assert_eq!(r["rev"], json!(rev), "no rev bump");
        // the footer says it too
        let foot = snapshot::row_text(&rt.current_frame(), 29);
        let shown = if verb == "back" {
            "Nothing to go back to"
        } else {
            "Nothing to go forward to"
        };
        assert!(foot.contains(shown), "{foot}");
        // the key: the same one-line notice, no revision
        let mut rt = self::rt();
        let rev = rt.app.rev;
        let k = if verb == "back" {
            "alt-left"
        } else {
            "alt-right"
        };
        rt.send(Msg::Key(Key::parse(k).unwrap().event()));
        rt.frame().unwrap();
        assert_eq!(rt.app.rev, rev);
        assert_eq!(rt.app.ui.note.as_deref(), Some(shown));
    }
    // with history, back moves and says where (no `note`)
    let mut rt = rt();
    rt.agent(&json!({"v": 1, "kind": "view", "op": "send", "verbs": [["tab", "ledger"]]}));
    let r = rt.agent(&json!({"v": 1, "kind": "view", "op": "send", "verbs": [["back"]]}));
    assert!(r.get("note").is_none(), "{r}");
    assert_eq!(r["changed"].as_array().unwrap().len(), 1);
}

// ── §2: the update-ready footer ───────────────────────────────────────────────────────────────────

#[test]
fn the_ready_footer_keeps_the_running_version_at_every_width() {
    for (w, h, want, ascii) in [
        (
            100,
            30,
            "v0.0.0-test  ⬆ v0.10.4 ready · ⌃P Upgrade",
            "v0.0.0-test  ^ v0.10.4 ready - ^P Upgrade",
        ),
        (
            80,
            24,
            "v0.0.0-test  ⬆ v0.10.4 · ⌃P Upgrade",
            "v0.0.0-test  ^ v0.10.4 - ^P Upgrade",
        ),
        (60, 20, "v0.0.0-test ⬆ 0.10.4", "v0.0.0-test ^ 0.10.4"),
    ] {
        for (th, want) in [(theme::LEDGER_DARK, want), (theme::NO_COLOR_ASCII, ascii)] {
            let mut a: App = fixture_app(th, false);
            a.update_available = Some(ownpurse_tui::canonical::Update {
                version: "0.10.4".into(),
                action: ownpurse_tui::upgrade::ACTION.into(),
            });
            let buf = snapshot::render(&mut a, w, h);
            let foot = snapshot::row_text(&buf, h - 1);
            assert!(foot.trim_end().ends_with(want), "{w}: {foot}");
            let st: Value = a.state_json(1);
            assert_eq!(
                st["update_available"],
                json!({"version": "0.10.4", "action": "upgrade_in_place"})
            );
        }
    }
}

// ── design fix: a status message outranks optional key hints, in every theme ────────────────────────

#[test]
fn an_upgrade_note_drops_key_hints_before_it_is_cut_in_every_theme() {
    use ownpurse_tui::upgrade::{self, Outcome};
    let notes = [
        upgrade::failed_note(
            &Outcome::PreflightFailed("record_changed: the record moved".into()),
            "0.0.0-test",
            "0.10.4",
        ),
        upgrade::failed_note(
            &Outcome::PreflightFailed("config_changed: orgs: a,b → a".into()),
            "0.0.0-test",
            "0.10.4",
        ),
        upgrade::deferred_note("0.10.4", "sync design"),
        upgrade::updated_note("0.10.1", "0.10.2"),
    ];
    let optional = ["Filter", "Org", "Scope", "Reload", "Help", "Quit"];
    for th in [
        theme::LEDGER_DARK,
        theme::LEDGER_LIGHT,
        theme::ANSI,
        theme::NO_COLOR_ASCII,
    ] {
        for note in &notes {
            for w in [60u16, 70, 80, 90, 100, 120, 140] {
                let mut a = fixture_app(th, false);
                a.update_available = Some(ownpurse_tui::canonical::Update {
                    version: "0.10.4".into(),
                    action: upgrade::ACTION.into(),
                });
                a.ui.note = Some(note.clone());
                let buf = snapshot::render(&mut a, w, 30);
                let foot = snapshot::row_text(&buf, 29);
                let full = a.ui.theme.text(note);
                let narrow = a.ui.theme.text(&upgrade::narrow_note(note));
                let short_whole = upgrade::short_note(&upgrade::narrow_note(note))
                    .into_iter()
                    .chain(upgrade::short_note(note))
                    .any(|s| foot.contains(&a.ui.theme.text(&s)));
                if !foot.contains(&full) && !foot.contains(&narrow) {
                    // shortened: every optional key hint went first (only `q` may outlast the reason)
                    for k in optional.iter().filter(|k| !(short_whole && **k == "Quit")) {
                        assert!(
                            !foot.contains(*k),
                            "{} {w}: `{k}` kept while the note is cut: {foot}",
                            th.name()
                        );
                    }
                    // and a preflight note keeps its remedy before anything else is cut
                    if let Some(short) = upgrade::short_note(note) {
                        let short = a.ui.theme.text(&short);
                        assert!(
                            foot.contains(&short) || w < 100,
                            "{} {w}: {foot}",
                            th.name()
                        );
                    }
                }
                assert!(
                    foot.trim_start().starts_with(&a.ui.theme.text(&note[..8])),
                    "{} {w}: {foot}",
                    th.name()
                );
            }
        }
    }
}

#[test]
fn preflight_reasons_have_remedies() {
    use ownpurse_tui::upgrade::preflight_remedy;
    assert_eq!(
        preflight_remedy("record_changed: x"),
        ("record changed", "r reload, then retry")
    );
    assert_eq!(
        preflight_remedy("config_changed: x"),
        ("config changed", "fix config, then retry")
    );
    assert_eq!(
        preflight_remedy("bad_state: State schema v9 …"),
        ("state not readable", "quit and restart to update")
    );
    assert_eq!(
        preflight_remedy("the restored frame differs from the live one (row 3)").1,
        "quit and restart to update"
    );
    assert_eq!(
        preflight_remedy("the new build did not answer within 8s").1,
        "reinstall, then retry"
    );
}

#[test]
fn with_the_update_notice_the_palette_hint_yields_and_narrow_remedies_shorten() {
    use ownpurse_tui::upgrade::{self, Outcome};
    let failed = upgrade::failed_note(
        &Outcome::PreflightFailed("record_changed: x".into()),
        "0.0.0-test",
        "0.10.4",
    );
    let config = upgrade::failed_note(
        &Outcome::PreflightFailed("config_changed: x".into()),
        "0.0.0-test",
        "0.10.4",
    );
    let other = upgrade::failed_note(
        &Outcome::PreflightFailed("the new build did not answer within 8s".into()),
        "0.0.0-test",
        "0.10.4",
    );
    let deferred = upgrade::deferred_note("0.10.4", "sync design");
    let foot = |note: &str, w: u16| {
        let mut a = fixture_app(theme::LEDGER_DARK, false);
        a.update_available = Some(ownpurse_tui::canonical::Update {
            version: "0.10.4".into(),
            action: upgrade::ACTION.into(),
        });
        a.ui.note = Some(note.to_string());
        // a real-length version: `v0.10.3`, not the 11-cell test label
        a.version_label = "0.10.3".into();
        let buf = snapshot::render(&mut a, w, 24);
        snapshot::row_text(&buf, 23)
    };
    // 80: whole, the palette hint gone (the notice says `⌃P Upgrade`)
    for (note, want) in [
        (&failed, "Upgrade not started · r reload, then retry"),
        (&deferred, "Upgrade will run when sync design finishes"),
    ] {
        let f = foot(note, 80);
        assert!(f.contains(want), "80: {f}");
        assert!(!f.contains("palette"), "80: {f}");
        assert!(f.trim_end().ends_with("⬆ v0.10.4 · ⌃P Upgrade"), "80: {f}");
    }
    // 60: the short remedies
    for (note, want) in [
        (&failed, "r reload, retry"),
        (&config, "fix config, retry"),
        (&other, "reinstall, retry"),
    ] {
        let f = foot(note, 60);
        assert!(f.contains(want), "60: {f}");
        assert!(!f.contains("then"), "60: {f}");
    }
    // the full form stays at 80 and up
    assert!(foot(&config, 100).contains("fix config, then retry"));
    // without a note, q stays while other hints go
    let f = foot("", 80);
    assert!(f.contains(" q "), "{f}");
}
