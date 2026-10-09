//! Compare (Ledger `v`): screen equals the CLI `tb --compare` goldens (map rows compared as sets, as the
//! CLI does), acceptance H1–H9 and I1–I3, V9 (no name merging, code-less never merges, no difference
//! column, no cross-org total), `m` adds no pairing, V8 at 60–220, and the Esc chain back to `v`.

use ownpurse_tui::app::{App, BandKind, Effect, Msg, Overlay, Scope, Tab, update};
use ownpurse_tui::ic_nav::{dec, s};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::model::Model;
use ownpurse_tui::snapshot::{self, fixture_app, fixtures_dir, run_script};
use ownpurse_tui::theme;
use serde_json::Value;
use std::collections::BTreeMap;

fn app() -> App {
    fixture_app(theme::LEDGER_DARK, false)
}

fn cli_app() -> App {
    let source = snapshot::cli_source();
    let now = snapshot::fixture_now();
    App::new(
        Model::open(&source, now).unwrap(),
        source,
        theme::LEDGER_DARK,
        false,
        now,
    )
}

fn screen(a: &mut App, w: u16, h: u16) -> String {
    snapshot::text(&snapshot::render(a, w, h))
}

fn key(a: &mut App, k: &str) -> Vec<Effect> {
    let fx = update(a, Msg::Key(Key::parse(k).unwrap().event()));
    snapshot::render(a, 140, 50);
    fx
}

fn golden(name: &str) -> Value {
    let p = fixtures_dir().join("cli/goldens").join(name);
    let v: Value = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    v["json"][0].clone()
}

fn rows_by_key(rows: &[Value]) -> BTreeMap<String, Value> {
    rows.iter().map(|r| (s(&r["key"]), r.clone())).collect()
}

/// Every golden row exists in the TUI model with the same per-org cells (the screen's numbers come from
/// these cells), and the set of rows is the same.
fn check_rows(a: &App, g: &Value) {
    let n = a.ui.cmp.as_ref().unwrap().orgs.len();
    let m = a.cmp_model(n).expect("model");
    let got = rows_by_key(&m.rows);
    let want = rows_by_key(g["rows"].as_array().unwrap());
    assert_eq!(
        got.keys().collect::<Vec<_>>(),
        want.keys().collect::<Vec<_>>(),
        "row sets"
    );
    for (k, w) in &want {
        let r = &got[k];
        for (org, cell) in w["cells"].as_object().unwrap() {
            let c = &r["cells"][org.as_str()];
            assert_eq!(c["exists"], cell["exists"], "{k} {org} exists");
            for side in ["debit", "credit", "net"] {
                assert_eq!(dec(&c[side]), dec(&cell[side]), "{k} {org} {side}");
            }
        }
        assert_eq!(r["names_differ"], w["names_differ"], "{k} ≠");
    }
}

fn pair_scope(a: &App) -> Scope {
    Scope::Group(
        a.model
            .groups
            .iter()
            .position(|g| g.name == "pair")
            .unwrap(),
    )
}

#[test]
fn screen_equals_cli_tb_compare_code_map_and_history() {
    let mut a = cli_app();
    a.ui.scope = pair_scope(&a);
    run_script(&mut a, "2 v", 140, 50);
    assert!(a.cmp_active());
    check_rows(&a, &golden("compare-code.json"));
    // the screen: each org's net under its column for the aligned row
    let txt = screen(&mut a, 140, 50);
    let line = txt
        .lines()
        .find(|l| l.contains("Retained Earnings") && l.contains('│'))
        .unwrap();
    assert!(
        line.contains("5,021.36") && line.contains("1,013.47"),
        "{line}"
    );
    run_script(&mut a, "a", 140, 50);
    check_rows(&a, &golden("compare-map.json"));
    // three orgs, full history, 2025-06-30
    let mut b = cli_app();
    b.ui.as_of = "2025-06-30".into();
    run_script(&mut b, "2 v h", 140, 50);
    assert_eq!(b.ui.cmp.as_ref().unwrap().orgs.len(), 3);
    check_rows(&b, &golden("compare-three-code-history.json"));
}

#[test]
fn h1_h3_header_and_columns() {
    let mut a = app();
    run_script(&mut a, "2 v", 140, 50);
    let txt = screen(&mut a, 140, 50);
    assert!(
        txt.lines()
            .next()
            .unwrap()
            .contains("compare: roast · hold · design · lab ▾"),
        "H1 L"
    );
    let s80 = screen(&mut a, 80, 24);
    assert!(s80.lines().next().unwrap().contains("compare"), "H1 S");
    assert!(txt.contains("TB+25 ✓"), "H3");
    assert!(
        txt.contains("debit") && txt.contains("credit"),
        "H9 L: debit/credit with 4 orgs"
    );
    let s100 = screen(&mut a, 100, 30);
    assert!(
        s100.contains("net") && !s100.contains("credit"),
        "H9 M: net"
    );
    assert!(
        s80.contains("widen the terminal to compare more orgs (showing 2 of 4)"),
        "H9 S"
    );
}

#[test]
fn h2_h5_h7_h8_strip_names_sections_dashes() {
    let mut a = app();
    a.ui.details = true; // the full basis and the align hint are details (design pass 2)
    run_script(&mut a, "2 v", 140, 50);
    let txt = screen(&mut a, 140, 50);
    assert!(txt.contains("align: code (a toggles: code ⇄ map)"), "H2");
    assert!(
        txt.contains("basis: all ✓ TB 2024-12-31 + 2025 · record 2h"),
        "H2 per org"
    );
    // the 1060 row (the group chart's card code): names differ (≠) and the strip names each org's account (H5)
    let pos = a
        .cmp_model(4)
        .unwrap()
        .rows
        .iter()
        .position(|r| r["key"] == "code:1060")
        .unwrap();
    for _ in 0..pos {
        key(&mut a, "down");
    }
    let txt = screen(&mut a, 140, 50);
    let line = txt
        .lines()
        .find(|l| l.contains("Tallowmere Green-Bean Car") && l.contains('│'))
        .unwrap_or_else(|| panic!("{txt}"));
    assert!(line.contains("1060 ≠"), "H5 ≠: {line}");
    assert!(
        txt.contains(
            "1060 · roast Tallowmere Green-Bean Card · design Quillfeather Site-Visit Card · lab SALTMARSH SENSORY PANEL CARD™"
        ),
        "H5 strip {txt}"
    );
    assert!(txt.contains("── only in roast ──"), "H7");
    // H8: — where the org has no such code; blank where it exists with zero
    assert!(line.ends_with('—') || line.contains("—"), "H8 dash: {line}");
}

#[test]
fn h6_map_with_no_account_map() {
    let mut a = app();
    a.source.account_map = serde_json::json!({});
    a.invalidate();
    run_script(&mut a, "2 v a", 140, 50);
    let txt = screen(&mut a, 140, 50);
    assert!(txt.contains(
        "No account_map configured: nothing merges; every account sits in its org's section."
    ));
    assert!(
        a.cmp_model(4)
            .unwrap()
            .rows
            .iter()
            .all(|r| r["section"] != "aligned"),
        "nothing merges"
    );
}

#[test]
fn v9_no_name_merging_no_difference_no_cross_org_total() {
    for align in ["code", "map"] {
        let mut a = app();
        run_script(
            &mut a,
            if align == "code" { "2 v" } else { "2 v a" },
            140,
            50,
        );
        let m = a.cmp_model(4).unwrap().clone();
        for r in m.rows.iter().filter(|r| r["section"] == "aligned") {
            let key = s(&r["key"]);
            if align == "code" {
                // rows merge only on an identical, non-empty code
                let code = key.strip_prefix("code:").expect("code key");
                assert!(!code.is_empty());
                for (org, labels) in r["members"].as_object().unwrap() {
                    for l in labels.as_array().unwrap() {
                        assert!(
                            s(l).ends_with(&format!("({code})")),
                            "{org} {l} merged under {code}"
                        );
                    }
                }
            } else {
                assert!(key.starts_with("map:"), "{key}");
            }
        }
        // code-less accounts never merge in code mode
        if align == "code" {
            for r in m.rows.iter().filter(|r| s(&r["code"]).is_empty()) {
                assert_ne!(r["section"], "aligned", "{}", r["label"]);
            }
        }
        let txt = screen(&mut a, 140, 50);
        assert!(
            !txt.to_lowercase().contains("difference"),
            "no difference column"
        );
        let totals: Vec<&str> = txt.lines().filter(|l| l.contains("Total")).collect();
        assert_eq!(totals.len(), 1);
        assert!(
            totals[0].contains("Total (each org's own)"),
            "{}",
            totals[0]
        );
        assert_eq!(
            totals[0].matches('│').count(),
            4,
            "one total per org: {}",
            totals[0]
        );
    }
    // identical names under different codes don't merge (CLI fixture: "Retained Earnings" is 3200 in hq and co,
    // but "Checking"-like names with different codes stay apart)
    let mut a = cli_app();
    run_script(&mut a, "2 v", 140, 50);
    let m = a.cmp_model(3).unwrap();
    let mut names: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for r in &m.rows {
        names.entry(s(&r["label"])).or_default().push(s(&r["key"]));
    }
    for (name, keys) in names {
        let codes: Vec<&String> = keys.iter().filter(|k| k.starts_with("code:")).collect();
        assert!(codes.len() <= 1, "{name} merged across codes: {keys:?}");
    }
}

#[test]
fn i1_i2_i3_postings_and_m_adds_no_pairing() {
    let mut a = app();
    run_script(&mut a, "2 v enter", 140, 50);
    let txt = screen(&mut a, 140, 50);
    assert!(
        txt.contains("postings side by side · no pairing inferred"),
        "I2"
    );
    assert!(
        txt.contains(
            "roast · Tallowmere Roastery Operating Account (1010)   opening Dr 11,934.69   closing Dr 41,671.82   8 postings"
        ),
        "I1 {txt}"
    );
    let before = a.cmp_l2().unwrap()["spine"].clone();
    run_script(&mut a, "m", 140, 50);
    let after = a.cmp_l2().unwrap()["spine"].clone();
    assert_eq!(before, after, "m changes no row and no pairing");
    let txt = screen(&mut a, 140, 50);
    assert!(
        txt.contains("highlighting existing Intercompany matches"),
        "I3"
    );
    // the marks are exactly the engine's matched postings that are shown (hold-roast: bt-202, bt-106,
    // bt-203, bt-107)
    assert_eq!(a.cmp_matched_shown(), 4);
    assert_eq!(
        txt.matches("≡ ").count(),
        5,
        "four marks plus the strip's ≡: {txt}"
    );
    let golden: Value = serde_json::from_str(
        &std::fs::read_to_string(fixtures_dir().join("demo/ic-2025-12-31.json")).unwrap(),
    )
    .unwrap();
    let engine: Vec<String> = golden["result"]["pairs"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| p["matched"].as_array().cloned().unwrap_or_default())
        .flat_map(|m| [s(&m["a"]["source_id"]), s(&m["b"]["source_id"])])
        .collect();
    for (_, id) in &a.cmp_view.matched {
        assert!(engine.contains(id), "{id} is not an engine match");
    }
}

#[test]
fn f_sets_the_range_for_every_column() {
    let mut a = app();
    run_script(
        &mut a,
        "2 v enter f backspace backspace backspace backspace backspace 0 7 - 0 1 enter",
        140,
        50,
    );
    let sp = a.cmp_l2().unwrap();
    assert_eq!(sp["since"], "2025-07-01");
    for line in sp["spine"].as_array().unwrap() {
        for p in line["cells"]
            .as_object()
            .unwrap()
            .values()
            .filter(|p| !p.is_null())
        {
            assert!(s(&p["date"]).as_str() >= "2025-07-01");
        }
    }
}

#[test]
fn entry_rules_picker_and_refusal() {
    // org scope → the multi-select picker, current org preselected; Enter disabled below 2
    let mut a = app();
    run_script(&mut a, "2 ] v", 100, 30);
    assert!(
        matches!(a.ui.overlay, Some(Overlay::Picker { multi: true, ref chosen, .. }) if chosen == &vec![0])
    );
    run_script(&mut a, "enter", 100, 30);
    assert!(a.ui.overlay.is_some(), "1 selected: Enter disabled");
    assert!(
        a.ui.note
            .as_deref()
            .unwrap_or("")
            .contains("2–4 orgs (1 selected)")
    );
    // a group toggles its members
    run_script(&mut a, "a l l", 100, 30);
    run_script(&mut a, "space", 100, 30);
    match &a.ui.overlay {
        Some(Overlay::Picker { chosen, .. }) => assert_eq!(chosen.len(), 4),
        o => panic!("{o:?}"),
    }
    run_script(&mut a, "enter", 100, 30);
    assert!(a.cmp_active());
    assert_eq!(a.ui.cmp.as_ref().unwrap().orgs.len(), 4);
    // a currency mismatch is refused, naming the orgs
    let mut w = snapshot::cli_app_profile(theme::LEDGER_DARK, "wide");
    let hq = w.model.orgs.iter().position(|o| o.alias == "hq").unwrap();
    let eur = w.model.orgs.iter().position(|o| o.alias == "eur").unwrap();
    w.compare_start(vec![hq, eur]);
    let b = w.ui.band.clone().unwrap();
    assert_eq!(b.kind, BandKind::Refusal);
    assert_eq!(
        b.text,
        "Cannot compare different base currencies: hq USD, eur EUR. The compare view does not open."
    );
    assert!(w.ui.cmp.is_none());
    // the palette entry opens it too
    let mut p = app();
    run_script(&mut p, "ctrl-p c o m p a r e enter", 100, 30);
    assert!(p.cmp_active());
}

#[test]
fn from_ledger_l2_opens_that_accounts_postings() {
    let mut a = app();
    run_script(&mut a, "2 ] enter v down down space enter", 140, 50);
    let c = a.ui.cmp.as_ref().unwrap();
    assert_eq!(c.level, 2, "opens on the row holding roast's Checking");
    assert_eq!(c.l2_row.as_deref(), Some("code:1010"));
}

#[test]
fn esc_chain_back_to_where_v_was_pressed_and_cursors_kept() {
    let mut a = app();
    run_script(&mut a, "2 ] enter", 140, 50);
    let ledger_nav = a.nav();
    run_script(&mut a, "[", 140, 50); // all
    run_script(&mut a, "v", 140, 50);
    assert!(a.cmp_active());
    run_script(&mut a, "down", 140, 50);
    let row = a.ui.cmp.as_ref().unwrap().row.clone();
    // amount → that org's ledger (via compare), Esc returns with the cursor kept
    run_script(&mut a, "right enter", 140, 50);
    assert!(a.ui.via.is_some() && !a.cmp_active());
    assert_eq!(a.ledger_level(), 2);
    assert!(
        screen(&mut a, 140, 50)
            .lines()
            .next()
            .unwrap()
            .contains("via compare")
    );
    run_script(&mut a, "esc", 140, 50);
    assert!(a.cmp_active());
    assert_eq!(a.ui.cmp.as_ref().unwrap().row, row, "row kept");
    // account → L2 → posting → L3, then back out
    run_script(&mut a, "left enter enter", 140, 50);
    assert_eq!(a.ledger_level(), 3, "a posting opens that org's document");
    run_script(&mut a, "esc", 140, 50);
    assert_eq!(a.ui.cmp.as_ref().unwrap().level, 2);
    run_script(&mut a, "esc esc", 140, 50);
    assert!(a.ui.cmp.is_none(), "back to the Ledger");
    assert_eq!(a.ui.tab, Tab::Ledger);
    run_script(&mut a, "]", 140, 50);
    assert_eq!(a.nav(), ledger_nav, "the Ledger's own state untouched");
}

#[test]
fn columns_reorder_and_drop_but_never_below_two() {
    let mut a = app();
    run_script(&mut a, "2 v right >", 140, 50);
    assert_eq!(a.cmp_aliases(), vec!["hold", "roast", "design", "lab"]);
    run_script(&mut a, "x x x", 140, 50);
    assert_eq!(a.cmp_aliases().len(), 2);
    assert!(
        screen(&mut a, 140, 50)
            .lines()
            .next()
            .unwrap()
            .contains(" ↔ "),
        "two compared orgs are a pair chip (v0.12.0)"
    );
}

#[test]
fn v8_amounts_never_truncate() {
    for th in [theme::LEDGER_DARK, theme::NO_COLOR_ASCII] {
        for script in ["2 v", "2 v a", "2 v h"] {
            let mut a = fixture_app(th, false);
            run_script(&mut a, script, 140, 60);
            for w in 60..=220u16 {
                let txt = screen(&mut a, w, 60);
                let k = a.cmp_k();
                let m = a.cmp_model(k).unwrap();
                for r in &m.rows {
                    for (_, c) in r["cells"].as_object().unwrap() {
                        if let Some(v) = dec(&c["net"]).filter(|v| !v.is_zero()) {
                            let f = ownpurse_core::fmt(v);
                            let abs = ownpurse_core::fmt(v.abs());
                            assert!(
                                txt.contains(&f) || txt.contains(&abs),
                                "{script} {w}: {f}\n{txt}"
                            );
                        }
                    }
                }
                assert!(!txt.contains("~ ") || th.ascii, "no cut amount");
            }
        }
    }
}

#[test]
fn strip_at_most_three_rows_at_80_never_cut() {
    for (script, map_off) in [
        ("2 v", false),
        ("2 v a", false),
        ("2 v a", true),
        ("2 v down down down", false),
        ("2 v h", false),
    ] {
        let mut a = app();
        if map_off {
            a.source.account_map = serde_json::json!({});
            a.invalidate();
        }
        run_script(&mut a, script, 80, 24);
        let txt = screen(&mut a, 80, 24);
        let lines: Vec<&str> = txt.lines().collect();
        // the strip sits between the breadcrumb (row 3) and the org header (the first row with │)
        let header = lines.iter().position(|l| l.contains('│')).unwrap();
        assert!(
            header - 4 <= 3,
            "{script} map_off={map_off}: {} strip rows\n{txt}",
            header - 4
        );
        // only the names line (the last strip row) may end with … (the column headers carry the names)
        assert!(
            !lines[4..header.saturating_sub(1)]
                .iter()
                .any(|l| l.contains('…')),
            "never cut:\n{txt}"
        );
        assert!(
            !txt.contains("Enter: amount"),
            "the hint line is dropped below 100"
        );
    }
}

#[test]
fn picker_footer_says_compare_and_when_it_is_disabled() {
    let mut a = app();
    run_script(&mut a, "2 ] v", 120, 30);
    assert!(
        screen(&mut a, 120, 30)
            .lines()
            .last()
            .unwrap()
            .contains("Enter compare (choose 2–4)")
    );
    run_script(&mut a, "down down space", 120, 30);
    let last = screen(&mut a, 120, 30).lines().last().unwrap().to_string();
    assert!(
        last.contains("Enter compare") && !last.contains("choose 2–4"),
        "{last}"
    );
}

#[test]
fn four_org_labels_at_100_show_the_first_name_and_ne() {
    let mut a = app();
    run_script(&mut a, "2 v", 100, 30);
    let txt = screen(&mut a, 100, 30);
    assert!(txt.contains("  Tallowmere Paid-In Capital ≠  "), "{txt}");
    assert!(
        !txt.lines()
            .any(|l| l.contains("Tallowmere Paid-In Capital / Brackenridge")),
        "no truncated join"
    );
}

#[test]
fn compare_default_align_is_honoured() {
    let mut a = app();
    a.source.compare_align = "map".into();
    run_script(&mut a, "2 v", 140, 50);
    assert_eq!(a.ui.cmp.as_ref().unwrap().align, "map");
    let cfg = ownpurse_core::config::Settings::build(
        Some(
            serde_json::json!({"tui": {"compare_default_align": "map", "log_rows": 7}})
                .as_object()
                .unwrap(),
        ),
        None,
        None,
    );
    assert_eq!(cfg.get_str("tui.compare_default_align"), Some("map"));
}

#[test]
fn strip_collapses_at_every_width() {
    for script in ["2 v", "2 ] v down down space enter"] {
        // design pass 2: ONE strip line (`align: code` left, the basis state right); the full basis is a detail
        let mut a = app();
        run_script(&mut a, script, 140, 40);
        for w in 60..=220u16 {
            let txt = screen(&mut a, w, 40);
            let lines: Vec<&str> = txt.lines().collect();
            let header = lines.iter().position(|l| l.contains('│')).unwrap();
            let rows = header - 4;
            let limit = if w >= 100 { 1 } else { 2 };
            assert!(rows <= limit, "{script} at {w}: {rows} strip rows\n{txt}");
            let strip = lines[4..header].join(" ");
            assert!(strip.contains("align: code"), "{script} at {w}: {strip}");
            assert!(
                !lines[4..header - 1].iter().any(|l| l.contains('…')),
                "{script} at {w}: cut\n{txt}"
            );
        }
    }
}

#[test]
fn differing_bases_are_listed_per_org() {
    let mut a = app();
    a.ui.details = true;
    a.ui.as_of = "2023-06-30".into(); // no opening TB anywhere: every org full history, not compared
    run_script(&mut a, "2 v h", 140, 40);
    let txt = screen(&mut a, 140, 40);
    assert!(
        txt.contains("basis: all not compared full history · record 2h"),
        "{txt}"
    );
}
