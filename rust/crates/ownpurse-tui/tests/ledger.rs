//! Ledger (tab 2): screen equals the CLI goldens (`tb` / `gl` on the CLI fixture), the
//! acceptance strings C1–C10, D1–D4, E1–E7 on the demo fixture, M1 landing states, navigation (§5),
//! V6 (pinned, recomputed basis strip) and V8 (amounts never cut) at every width.

use ownpurse_tui::app::{App, Effect, Msg, OvCol, Scope, Tab, update};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::model::Model;
use ownpurse_tui::snapshot::{self, fixture_app, fixtures_dir, row_text, run_script};
use ownpurse_tui::theme;
use serde_json::Value;

fn app() -> App {
    fixture_app(theme::LEDGER_DARK, false)
}

fn cli_app() -> App {
    let source = snapshot::cli_source();
    let now = snapshot::fixture_now();
    let model = Model::open(&source, now).unwrap();
    App::new(model, source, theme::LEDGER_DARK, false, now)
}

fn golden(name: &str) -> Value {
    let p = fixtures_dir().join("cli/goldens").join(name);
    let v: Value = serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap();
    v["json"][0].clone()
}

fn screen(a: &mut App, w: u16, h: u16) -> String {
    snapshot::text(&snapshot::render(a, w, h))
}

fn nocomma(s: &str) -> String {
    s.replace(',', "")
}

fn key(a: &mut App, k: &str) -> Vec<Effect> {
    let fx = update(a, Msg::Key(Key::parse(k).unwrap().event()));
    snapshot::render(a, 140, 40);
    fx
}

/// Move the L1 cursor to the row whose account name is `name`.
fn select_account(a: &mut App, name: &str) {
    let m = a.l1_model().unwrap().clone();
    let vis = a.l1_visible();
    let pos = vis
        .iter()
        .position(|&i| m.rows[i].name == name)
        .unwrap_or_else(|| panic!("{name}"));
    key(a, "home");
    for _ in 0..pos {
        key(a, "j");
    }
}

// ── screen equals CLI (tb, gl) ───────────────────────────────────────────────────────────────────

fn check_tb(a: &mut App, g: &Value) {
    let s = screen(a, 140, 40);
    for r in g["trial_balance"].as_array().unwrap() {
        // CLI account label is `Name (code)`; the screen shows code and name in their own columns
        let label = r["account"].as_str().unwrap();
        let name = label.rsplit_once(" (").map_or(label, |(n, _)| n);
        let line = s
            .lines()
            .find(|l| l.contains(&format!("  {name}  ")) || l.ends_with(name))
            .unwrap_or_else(|| panic!("row {name}:\n{s}"));
        let cells: Vec<String> = line
            .split("  ")
            .map(|c| nocomma(c.trim()))
            .filter(|c| !c.is_empty())
            .collect();
        for side in ["debit", "credit"] {
            if let Some(v) = r[side].as_str() {
                assert!(
                    cells.contains(&v.to_string()),
                    "{name} {side} {v} not in {line}"
                );
            }
        }
        assert_eq!(
            line.trim_start_matches('▌').trim_start().starts_with('!'),
            r["flag"] == "!",
            "{name} flag: {line}"
        );
    }
    let total = s.lines().find(|l| l.contains("Total")).unwrap();
    assert!(
        nocomma(total).contains(g["total_debit"].as_str().unwrap()),
        "{total}"
    );
    assert!(
        nocomma(total).contains(g["total_credit"].as_str().unwrap()),
        "{total}"
    );
    assert_eq!(total.contains("balanced ✓"), g["balanced"] == true);
}

#[test]
fn screen_equals_cli_tb() {
    let mut a = cli_app();
    run_script(&mut a, "2 ]", 140, 40);
    assert_eq!(a.ui.scope, Scope::Org(0));
    check_tb(&mut a, &golden("tb-one-2025.json"));
    run_script(&mut a, "h", 140, 40);
    check_tb(&mut a, &golden("tb-one-history.json"));
    run_script(&mut a, "h ]", 140, 40);
    check_tb(&mut a, &golden("tb-hq.json"));
}

fn check_gl(a: &App, g: &Value) {
    let m = a.l2_model().unwrap();
    let d = |s: &str| ownpurse_core::money::parse(s).unwrap();
    assert_eq!(
        m.opening,
        d(g["opening_balance"].as_str().unwrap()),
        "opening"
    );
    assert_eq!(
        m.closing,
        d(g["closing_balance"].as_str().unwrap()),
        "closing"
    );
    assert_eq!(m.debits, d(g["debits"].as_str().unwrap()), "debits");
    assert_eq!(m.credits, d(g["credits"].as_str().unwrap()), "credits");
    assert_eq!(m.since, g["from"].as_str().unwrap());
    let ps = g["postings"].as_array().unwrap();
    assert_eq!(m.rows.len(), ps.len());
    for (r, p) in m.rows.iter().zip(ps) {
        assert_eq!(r.date, p["date"].as_str().unwrap());
        assert_eq!(r.source_id, p["source_id"].as_str().unwrap());
        assert_eq!(
            r.balance,
            d(p["balance"].as_str().unwrap()),
            "{} balance",
            r.source_id
        );
        let amt = match (p["debit"].as_str(), p["credit"].as_str()) {
            (Some(x), _) => d(x),
            (_, Some(c)) => -d(c),
            _ => unreachable!(),
        };
        assert_eq!(r.amount, amt);
    }
}

#[test]
fn screen_equals_cli_gl_and_range() {
    let mut a = cli_app();
    run_script(&mut a, "2 ]", 140, 40);
    select_account(&mut a, "Pewter Lane Shop Supplies");
    key(&mut a, "enter");
    let g = golden("gl-one-account.json");
    check_gl(&a, &g);
    // the screen: strip and the first page of rows
    let s = screen(&mut a, 140, 40);
    assert!(
        s.contains(
            "opening 0.00 from Xero TB 2024-12-31 · debits 86,560.76 · credits 21.68 · closing 86,539.08"
        ),
        "{s}"
    );
    let header = s
        .lines()
        .position(|l| l.contains("date") && l.contains("balance"))
        .unwrap();
    for (line, p) in s
        .lines()
        .skip(header + 1)
        .zip(g["postings"].as_array().unwrap().iter().take(5))
    {
        assert!(line.contains(p["date"].as_str().unwrap()), "{line}");
        assert!(
            nocomma(line).contains(p["balance"].as_str().unwrap()),
            "{line}"
        );
    }
    // range: Checking from 2025-03-01 to 2025-04-30 (gl-one-account-range): as of 2025-04-30, `f` 2025-03-01
    key(&mut a, "esc");
    run_script(
        &mut a,
        "d backspace backspace backspace backspace backspace 0 4 - 3 0 enter",
        140,
        40,
    );
    select_account(&mut a, "Pewter Lane Checking");
    key(&mut a, "enter");
    run_script(
        &mut a,
        "f backspace backspace backspace backspace backspace 0 3 - 0 1 enter",
        140,
        40,
    );
    check_gl(&a, &golden("gl-one-account-range.json"));
    assert!(screen(&mut a, 140, 40).contains("postings from 2025-03-01 to 2025-04-30"));
}

// ── acceptance (demo fixture) ────────────────────────────────────────────────────────────────────

#[test]
fn c1_c2_c7_c10_year_basis() {
    let mut a = app();
    run_script(&mut a, "2 ] ] ]", 80, 24);
    let s = screen(&mut a, 80, 24);
    assert!(
        s.lines()
            .nth(3)
            .unwrap()
            .contains("design › Trial balance at 2025-12-31 (accrual)"),
        "C1 {s}"
    );
    assert!(
        s.contains("✓ matches Xero"),
        "C2: a healthy basis is the breadcrumb's short state (design pass 2)"
    );
    // the balance mark sits after the totals, or wraps to the next line when long names take the room
    let lines: Vec<&str> = s.lines().collect();
    let t = lines.iter().position(|l| l.contains("Total")).unwrap();
    assert!(
        lines[t].contains("balanced ✓") || lines[t + 1].trim() == "balanced ✓",
        "C7 {s}"
    );
    assert!(s.contains("credit"), "C10 credit header at S");
}

#[test]
fn c3_c4_differs_and_not_compared() {
    let mut a = cli_app();
    run_script(&mut a, "2 ]", 140, 40);
    let s = screen(&mut a, 140, 40);
    assert!(
        s.contains("· differs from Xero at 2025-12-31: 1 account"),
        "C3 {s}"
    );
    let mut a = app();
    a.ui.as_of = "2023-06-30".into();
    run_script(&mut a, "2 ]", 140, 40);
    let s = screen(&mut a, 140, 40);
    assert!(
        s.contains("Local full-history balances — no cached Xero TB to compare"),
        "C4 (no opening TB → history): {s}"
    );
}

#[test]
fn c5_c6_c9_full_history_and_differs_filter() {
    let mut a = app();
    run_script(&mut a, "2 ] ] ]", 140, 40);
    assert!(
        screen(&mut a, 140, 40)
            .lines()
            .last()
            .unwrap()
            .contains("h Full history"),
        "C6"
    );
    run_script(&mut a, "h", 140, 40);
    let s = screen(&mut a, 140, 40);
    assert!(joined(&s).contains("Local full-history balances: 15 accounts differ from Xero (marked !; usually pre-2025 setup balances). ⌃P → Refetch TB to open from Xero."), "C5 {s}");
    assert!(
        s.lines().last().unwrap().contains("h Year basis"),
        "C6 after h"
    );
    assert!(
        s.lines()
            .filter(|l| l.starts_with("  !") || l.starts_with("▌ !"))
            .count()
            >= 10,
        "! gutter"
    );
    run_script(&mut a, "!", 140, 40);
    let s = screen(&mut a, 140, 40);
    assert!(s.contains("showing: differs from Xero (15)"), "C9");
    assert_eq!(a.l1_visible().len(), 15);
}

#[test]
fn c8_all_scope_and_never_synced() {
    let mut a = app();
    run_script(&mut a, "2", 80, 24);
    assert!(
        screen(&mut a, 80, 24)
            .contains("Ledger needs one org: press g or ] to choose (roast, hold, design, lab).")
    );
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "never");
    run_script(&mut a, "2 ] ] ] ]", 80, 24);
    assert!(screen(&mut a, 80, 24).contains("No local record for lab yet. ⌃P → Sync lab."));
}

#[test]
fn d1_d2_d3_d4_postings() {
    let mut a = app();
    run_script(&mut a, "2 ] enter", 80, 24);
    let s = screen(&mut a, 80, 24);
    assert!(
        s.lines()
            .nth(3)
            .unwrap()
            .contains("roast › … › Tallowmere Roastery Operating Account (1010)"),
        "D1 middle shortens at S: {s}"
    );
    let s140 = screen(&mut a, 140, 40);
    assert!(
        s140.lines()
            .nth(3)
            .unwrap()
            .contains("roast › Trial balance at 2025-12-31 (accrual) › Tallowmere Roastery Operating Account (1010)")
    );
    assert!(
        s.contains("opening 11,934.69 from Xero TB 2024-12-31 · debits 38,829.25 ·"),
        "D2 wraps, never cut"
    );
    assert!(s.contains("closing 41,671.82"), "D2 second line");
    let labels = [
        "Spend",
        "Receive",
        "Transfer",
        "Journal",
        "Invoice",
        "Bill",
        "Credit note",
        "Payment",
        "Overpayment",
        "Prepayment",
    ];
    for r in &a.l2_model().unwrap().rows {
        assert!(labels.contains(&r.kind.as_str()), "D3 {}", r.kind);
    }
    assert!(s.contains("balance"), "D4 header");
}

#[test]
fn e1_to_e7_document() {
    let mut a = app();
    run_script(&mut a, "2 ] enter enter", 140, 40);
    let s = screen(&mut a, 140, 40);
    // design pass 2: the document leads with what happened, the rest on one quiet line
    for l in [
        "9,104.57  received from Larkspur Grocers Cooperative",
        "2025-02-03",
        "Receive money",
        "Authorised",
        "no tax",
        "reconciled ✓",
    ] {
        assert!(s.contains(l), "E1 {l}");
    }
    assert!(
        s.lines()
            .nth(3)
            .unwrap()
            .trim_end()
            .ends_with("Receive 2025-02-03")
            || s.lines().nth(3).unwrap().contains("› Receive 2025-02-03"),
        "E2"
    );
    a.ui.details = true;
    assert!(
        joined(&screen(&mut a, 140, 40)).contains("o opens it in Xero"),
        "E3 (details)"
    );
    a.ui.details = false;
    let s100 = screen(&mut a, 100, 40); // stacked: each note on one line
    assert!(
        s100.contains("Not fetched yet. ⌃P → Fetch attachments or Fetch history · 1 call each"),
        "E4 {s100}"
    );
    for h in ["description", "account", "amount"] {
        assert!(s.contains(h), "E5 {h}");
    }
    // E7: at S the panes stack (attachments below the fields and lines)
    let s80 = screen(&mut a, 80, 24);
    let fields_y = s80
        .lines()
        .position(|l| l.contains("received from"))
        .unwrap();
    let att_y = s80.lines().position(|l| l.contains("Attachments")).unwrap();
    assert!(att_y > fields_y + 4, "E7 stacked");
    let att_line = s.lines().find(|l| l.contains("Attachments")).unwrap();
    assert!(
        att_line.find("Attachments").unwrap() > 60,
        "side by side at L: {att_line}"
    );
}

#[test]
fn o_and_y_on_a_document_and_fetch_placeholders_need_a_document() {
    let mut a = app();
    run_script(&mut a, "2 ] enter", 140, 40);
    assert!(
        a.palette_entries()
            .iter()
            .all(|e| !e.label.starts_with("Fetch")),
        "no document yet"
    );
    key(&mut a, "enter");
    let fx = key(&mut a, "o");
    assert_eq!(
        fx,
        vec![Effect::Open(
            // through the org switch (the org's ShortCode is in the record), so it opens in the right org
            "https://go.xero.com/organisationlogin/default.aspx?shortcode=%21TmR4q&redirecturl=%2FBank%2FViewTransaction.aspx%3FbankTransactionID%3Dbt-102".into()
        )]
    );
    let fx = key(&mut a, "y");
    assert_eq!(fx, vec![Effect::Copy("bt-102".into())]);
    assert!(
        a.palette_entries()
            .iter()
            .any(|e| e.label == "Fetch history…")
    );
    run_script(&mut a, "ctrl-p h i s t o r y enter", 140, 40);
    assert_eq!(
        a.ui.band.as_ref().unwrap().text,
        "Not available yet (coming in v0.9.x)."
    );
}

// ── M1 landing and navigation (§5) ───────────────────────────────────────────────────────────────

#[test]
fn m1_landing_states_are_real() {
    // B10: roast parity (history) → Ledger L1, strip says full history and the filter count
    let mut a = app();
    run_script(&mut a, "right right enter", 140, 40);
    let s = screen(&mut a, 140, 40);
    assert!(s.contains("Local full-history balances"), "{s}");
    assert!(s.contains("showing: differs from Xero (2)"), "{s}");
    assert_eq!(a.l1_visible().len(), 2);
    // B11: design parity (year) → year basis strip, differs filter on (nothing differs on the year basis)
    let mut a = app();
    run_script(&mut a, "j j right enter", 140, 40);
    let s = screen(&mut a, 140, 40);
    assert!(s.contains("✓ matches Xero"), "{s}");
    assert!(s.contains("showing: differs from Xero (0)"), "{s}");
    // B12: Esc → back on Overview, same row and cell
    run_script(&mut a, "esc", 140, 40);
    assert_eq!(
        (a.ui.tab, a.ui.overview_col),
        (Tab::Overview, OvCol::ParityYear)
    );
    assert_eq!(a.ui.overview_sel.as_deref(), Some("design"));
}

#[test]
fn n2_drill_chain_and_esc_back_to_origin() {
    let mut a = app();
    run_script(&mut a, "enter enter enter", 140, 40);
    assert_eq!((a.ui.tab, a.ledger_level()), (Tab::Ledger, 3));
    run_script(&mut a, "esc", 140, 40);
    assert_eq!(a.ledger_level(), 2);
    run_script(&mut a, "esc", 140, 40);
    assert_eq!(a.ledger_level(), 1);
    run_script(&mut a, "esc", 140, 40);
    assert_eq!(a.ui.tab, Tab::Overview);
    assert_eq!(a.ui.overview_sel.as_deref(), Some("roast"));
}

#[test]
fn n5_cursor_kept_by_id_across_scope_as_of_and_reload() {
    let mut a = app();
    run_script(&mut a, "2 ] j j j", 140, 40);
    let id = a.nav().l1_sel.clone().unwrap();
    run_script(&mut a, "] [", 140, 40);
    assert_eq!(a.nav().l1_sel.as_ref(), Some(&id), "scope round trip");
    let name = |a: &App| {
        let m = a.l1_model().unwrap();
        m.rows[a.l1_visible()[a.l1_cursor().unwrap()]]
            .account_id
            .clone()
    };
    assert_eq!(name(&a), id);
    run_script(
        &mut a,
        "d backspace backspace backspace backspace backspace 0 6 - 3 0 enter",
        140,
        40,
    );
    assert_eq!(a.ui.as_of, "2025-06-30");
    assert_eq!(name(&a), id, "as-of change keeps the account");
    // per-scope L2 state: design keeps its own level
    run_script(&mut a, "enter ] ] [ [", 140, 40);
    assert_eq!(a.ledger_level(), 2);
    let mut rt_app = app();
    run_script(&mut rt_app, "2 ] j j", 140, 40);
    let before = rt_app.nav().l1_sel.clone();
    let source = rt_app.source.clone();
    let model = Model::open(&source, rt_app.now).unwrap();
    let books = ownpurse_tui::books::Books::open(&source).ok();
    update(&mut rt_app, Msg::Reloaded(Box::new((model, books))));
    snapshot::render(&mut rt_app, 140, 40);
    assert_eq!(rt_app.nav().l1_sel, before, "reload keeps the cursor id");
    assert!(
        rt_app.l1_model().is_some(),
        "models prepared again after reload"
    );
}

#[test]
fn n1_arriving_takes_no_cursor_and_unbound_letters_do_nothing() {
    let mut a = app();
    run_script(&mut a, "2 ]", 100, 30);
    a.dirty = false;
    update(&mut a, Msg::Key(Key::parse("a").unwrap().event()));
    assert!(a.ui.prompt.is_none() && !a.dirty);
}

// ── V6 and V8 at every width ─────────────────────────────────────────────────────────────────────

fn joined(s: &str) -> String {
    s.lines().map(str::trim).collect::<Vec<_>>().join(" ")
}

#[test]
fn v6_strip_pinned_whole_and_recomputed() {
    for (script, want) in [
        (
            "2 ] ] ] h",
            "Local full-history balances: 15 accounts differ from Xero (marked !; usually pre-2025 setup balances). ⌃P → Refetch TB to open from Xero.",
        ),
        (
            "2 ]",
            "Opening from Xero TB 2024-12-31 + 2025 activity · matches Xero ✓",
        ),
        (
            "2 ] enter",
            "opening 11,934.69 from Xero TB 2024-12-31 · debits 38,829.25 · credits 9,092.12 · closing 41,671.82",
        ),
    ] {
        let mut a = app();
        // the strip shows whole when it shows (a healthy basis is under `i`, design pass 2)
        a.ui.details = true;
        run_script(&mut a, script, 100, 30);
        for w in 60..=220u16 {
            let s = screen(&mut a, w, 30);
            // the strip starts on row 4, right under the breadcrumb, and wraps without cutting
            let strip: String = s.lines().skip(4).take(3).collect::<Vec<_>>().join("\n");
            assert!(joined(&strip).contains(want), "{script} at {w}:\n{s}");
        }
    }
    // recomputed every draw: toggling the basis changes the strip in the next frame
    let mut a = app();
    a.ui.details = true;
    run_script(&mut a, "2 ]", 100, 30);
    let before = screen(&mut a, 100, 30);
    run_script(&mut a, "h", 100, 30);
    let after = screen(&mut a, 100, 30);
    assert_ne!(before.lines().nth(4), after.lines().nth(4));
}

#[test]
fn v8_amounts_never_cut_on_l1_l2_l3() {
    for script in [
        "2 ] ] ] h",
        "2 ] ] ] h !",
        "right right enter",
        "2 ] enter",
        "2 ] enter enter",
        "2 ] ] ] ]",
    ] {
        for th in [theme::LEDGER_DARK, theme::NO_COLOR_ASCII] {
            let mut a = fixture_app(th, false);
            run_script(&mut a, script, 100, 30);
            for w in 60..=220u16 {
                let buf = snapshot::render(&mut a, w, 40);
                let rows: Vec<String> = (0..40).map(|y| row_text(&buf, y)).collect();
                let s = rows.join("\n");
                let amounts: Vec<String> = match a.ledger_level() {
                    1 => {
                        let m = a.l1_model().unwrap();
                        a.l1_visible()
                            .iter()
                            .map(|&i| m.rows[i].balance)
                            .filter(|v| !v.is_zero())
                            .map(|v| ownpurse_core::fmt(v.abs()))
                            .chain([ownpurse_core::fmt(m.debit)])
                            .collect()
                    }
                    2 => a
                        .l2_model()
                        .unwrap()
                        .rows
                        .iter()
                        .flat_map(|r| {
                            [
                                ownpurse_core::fmt(r.amount.abs()),
                                ownpurse_core::fmt(r.balance),
                            ]
                        })
                        .collect(),
                    _ => a
                        .doc_model()
                        .unwrap()
                        .lines
                        .iter()
                        .map(|l| l.amount.clone())
                        .collect(),
                };
                for v in amounts {
                    assert!(s.contains(&v), "{script} at {w}: {v} missing\n{s}");
                }
            }
        }
    }
}

// ── C11: the differs filter shows Xero and the difference (what Parity showed) ──────────────────────

#[test]
fn c11_xero_and_difference_equal_cli_parity() {
    for (script, golden_name) in [
        ("2 ] h !", "parity-one.json"),
        ("2 ] !", "parity-one-since.json"),
    ] {
        let mut a = cli_app();
        run_script(&mut a, script, 140, 40);
        let s = screen(&mut a, 140, 40);
        assert!(s.contains("xero") && s.contains("difference"), "{s}");
        for d in golden(golden_name)["differences"].as_array().unwrap() {
            let dec = |k: &str| ownpurse_core::money::parse(d[k].as_str().unwrap()).unwrap();
            let line = s
                .lines()
                .find(|l| l.starts_with("▌ ! ") || l.starts_with("  !  4"))
                .unwrap_or_else(|| panic!("{s}"));
            assert!(
                line.contains(&ownpurse_core::dr_cr(dec("xero"))),
                "{golden_name} xero: {line}"
            );
            assert!(
                line.contains(&ownpurse_core::dr_cr(dec("difference"))),
                "{golden_name} difference: {line}"
            );
            let m = a.l1_model().unwrap();
            let r = m
                .rows
                .iter()
                .find(|r| r.account_id == d["account_id"].as_str().unwrap())
                .unwrap();
            assert_eq!(
                (r.xero, r.difference),
                (Some(dec("xero")), Some(dec("difference")))
            );
        }
    }
}

#[test]
fn c11_xero_only_accounts_total_label_and_class_hides_first() {
    let mut a = app();
    run_script(&mut a, "2 ] ] ] h !", 80, 24);
    let s = screen(&mut a, 80, 24);
    for (name, xero) in [
        ("Large-Format Pl…", "Dr 17,914.26"),
        ("Sabbatical Rese…", "Dr 8,209.43"),
        ("Cartwright Yard…", "Dr 2,531.79"),
    ] {
        // long names end-truncate at 80 columns; the amounts never do
        let line = s
            .lines()
            .find(|l| l.contains(&format!("  {name}  ")))
            .unwrap_or_else(|| panic!("{name}\n{s}"));
        assert!(line.contains(xero), "{name}: {line}");
        assert!(
            line.contains(&xero.replace("Dr", "Cr")),
            "{name} difference: {line}"
        );
    }
    let header = s
        .lines()
        .find(|l| l.contains("account") && l.contains("difference"))
        .unwrap();
    assert!(
        !header.contains("class") && header.contains("code"),
        "class hides first at 80: {header}"
    );
    assert!(s.contains("Total (all acco…  11,336.41  11,336.41"), "{s}");
    assert!(
        s.lines().last().unwrap().starts_with(" design · 15 differ"),
        "count leads the status"
    );
    run_script(&mut a, "!", 80, 24);
    assert!(
        screen(&mut a, 80, 24).contains("Total  ")
            && !screen(&mut a, 80, 24).contains("(all accounts)")
    );
}

#[test]
fn c11_v8_xero_and_difference_never_cut() {
    for th in [theme::LEDGER_DARK, theme::NO_COLOR_ASCII] {
        let mut a = fixture_app(th, false);
        run_script(&mut a, "2 ] ] ] h !", 100, 30);
        let want: Vec<String> = a
            .l1_model()
            .unwrap()
            .rows
            .iter()
            .filter(|r| r.differs)
            .flat_map(|r| {
                [
                    ownpurse_core::dr_cr(r.xero.unwrap()),
                    ownpurse_core::dr_cr(r.difference.unwrap()),
                ]
            })
            .collect();
        for w in 60..=220u16 {
            let s = screen(&mut a, w, 40);
            for v in &want {
                assert!(s.contains(v.as_str()), "{w}: {v}\n{s}");
            }
        }
    }
}

#[test]
fn c4_year_basis_not_compared() {
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "noclose");
    run_script(&mut a, "2 ]", 140, 40);
    let s = screen(&mut a, 140, 40);
    assert!(
        s.contains("Opening from Xero TB 2024-12-31 + 2025 activity · not compared: no cached Xero TB at 2025-12-31 (⌃P → Refetch)"),
        "{s}"
    );
}

#[test]
fn never_synced_footer_shows_only_working_keys() {
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "never");
    run_script(&mut a, "2 ] ] ] ]", 140, 40);
    let last = screen(&mut a, 140, 40).lines().last().unwrap().to_string();
    assert!(
        !last.contains("Full history") && !last.contains("Compare") && !last.contains("Differs"),
        "{last}"
    );
}

#[test]
fn no_render_shows_a_sync_after_the_pinned_clock() {
    let mut a = app();
    run_script(&mut a, "2 ] enter enter", 140, 40);
    a.ui.details = true;
    let s = screen(&mut a, 140, 40);
    assert!(s.contains("local v1 · synced 2026-10-05 07:46 UTC"), "{s}");
}

// ── C12 and the ANSI cursor band ───────────────────────────────────────────

fn cell_at(buf: &ratatui::buffer::Buffer, y: u16, needle: &str) -> (u16, u16) {
    let line = row_text(buf, y);
    let x = line[..line
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} in {line}"))]
        .chars()
        .count() as u16;
    (x, y)
}

#[test]
fn c12_selected_parity_cell_in_every_mode() {
    use ratatui::style::{Color, Modifier};
    for (name, th) in snapshot::themes() {
        let mut a = fixture_app(th, false);
        run_script(&mut a, "right right", 100, 30);
        let buf = snapshot::render(&mut a, 100, 30);
        let y = (0..30)
            .find(|&y| row_text(&buf, y).contains(" roast "))
            .unwrap();
        let line = row_text(&buf, y);
        let marked = matches!(th.mode, theme::Mode::NoColor) || th.ascii;
        let value_at = if marked {
            let mark = if th.ascii {
                "> 2 accounts differ"
            } else {
                "▸ 2 accounts differ"
            };
            assert!(
                line.contains(mark),
                "{name}: marker before the value: {line}"
            );
            mark
        } else {
            "2 accounts differ"
        };
        let (x, y) = cell_at(&buf, y, value_at);
        let c = &buf[(x, y)];
        match th.mode {
            theme::Mode::Truecolor | theme::Mode::Indexed => {
                assert!(
                    c.modifier.contains(Modifier::REVERSED | Modifier::BOLD),
                    "{name}: {:?}",
                    c.modifier
                );
                assert!(
                    !c.modifier.contains(Modifier::UNDERLINED),
                    "{name}: no underline"
                );
            }
            _ => {
                assert!(
                    // v0.12.0: the row stays one reverse span; the cursor cell is underlined and bold
                    c.modifier
                        .contains(Modifier::BOLD | Modifier::UNDERLINED | Modifier::REVERSED),
                    "{name}"
                );
                let (ox, _) = cell_at(&buf, y, "roast");
                assert!(
                    buf[(ox, y)].modifier.contains(Modifier::REVERSED),
                    "{name}: the row is reversed"
                );
                // one reversed band: no colour on the cursor row (tc handoff §1.3.3)
                for xx in 0..100 {
                    assert_eq!(
                        (buf[(xx, y)].fg, buf[(xx, y)].bg),
                        (Color::Reset, Color::Reset),
                        "{name} x={xx}"
                    );
                }
            }
        }
    }
}

#[test]
fn ansi_cursor_rows_are_one_band_in_every_table() {
    use ratatui::style::Color;
    for script in ["2 ] ] ] h", "2 ] enter", "4", "5", "ctrl-p", "g"] {
        let mut a = fixture_app(theme::ANSI, false);
        run_script(&mut a, script, 100, 30);
        let buf = snapshot::render(&mut a, 100, 30);
        let mut found = false;
        for y in 4..29 {
            let line = row_text(&buf, y);
            if let Some(x) = line.find('▌') {
                let x0 = line[..x].chars().count() as u16;
                if buf[(x0 + 2, y)]
                    .modifier
                    .contains(ratatui::style::Modifier::REVERSED)
                {
                    found = true;
                    for xx in x0 + 1..100 {
                        let c = &buf[(xx, y)];
                        if c.modifier.contains(ratatui::style::Modifier::REVERSED) {
                            assert_eq!(
                                c.fg,
                                Color::Reset,
                                "{script}: coloured span at {xx},{y}: {line}"
                            );
                        }
                    }
                }
            }
        }
        assert!(found, "{script}: no cursor row");
    }
}

#[test]
fn differs_filter_names_the_sign() {
    let mut a = app();
    run_script(&mut a, "2 ] ] ] h !", 140, 40);
    let s = screen(&mut a, 140, 40);
    assert_eq!(s.matches("difference = local − Xero").count(), 1, "{s}");
    assert!(
        s.lines()
            .any(|l| l.contains("xero") && l.contains("local − xero")),
        "≥100 label"
    );
    let s80 = screen(&mut a, 80, 24);
    assert!(
        s80.lines()
            .any(|l| l.contains("xero") && l.contains("difference"))
            && !s80.contains("local − xero"),
        "{s80}"
    );
}
