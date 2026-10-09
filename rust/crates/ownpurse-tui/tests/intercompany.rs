//! Intercompany (tab 3): screen equals the CLI goldens (`ic --json` on the CLI fixture, the demo's
//! ic-2025-12-31.json, the engine fixture's ic-2025-12-31.json), acceptance F1–F7 and G1–G6, L3 via the
//! Ledger widget with `· via <pair>` (E6), `a`/`b` via the Ledger, navigation, V6, V7 and V8.

use ownpurse_tui::app::{App, BandKind, Effect, Msg, Scope, Tab, update};
use ownpurse_tui::ic_nav::{IcRow, SORTS, dec, s};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::model::Model;
use ownpurse_tui::snapshot::{self, fixture_app, fixtures_dir, ic_app, row_text, run_script};
use ownpurse_tui::theme;
use serde_json::Value;

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

fn read(path: &str) -> Value {
    serde_json::from_str(&std::fs::read_to_string(fixtures_dir().join(path)).unwrap()).unwrap()
}

fn key(a: &mut App, k: &str) -> Vec<Effect> {
    let fx = update(a, Msg::Key(Key::parse(k).unwrap().event()));
    snapshot::render(a, 140, 40);
    fx
}

fn fmt_opt(v: &Value) -> String {
    dec(v).map_or("—".into(), ownpurse_core::fmt)
}

/// Every L1 row on screen carries the golden's difference, residual and status.
fn check_l1(a: &mut App, pairs: &[Value]) {
    let txt = screen(a, 140, 40);
    for p in pairs {
        let r = IcRow::of(p);
        let line = txt
            .lines()
            .find(|l| {
                l.contains(&format!("  {}  ", r.label))
                    || l.starts_with(&format!("▌ {}  ", r.label))
            })
            .unwrap_or_else(|| panic!("row {}:\n{txt}", r.label));
        assert!(
            line.contains(&fmt_opt(&p["difference"])),
            "{} difference: {line}",
            r.id
        );
        assert!(
            line.contains(&fmt_opt(&p["residual"])),
            "{} residual: {line}",
            r.id
        );
        assert!(line.contains(&r.status_text()), "{} status: {line}", r.id);
    }
}

#[test]
fn screen_equals_demo_golden_l1_and_bridges() {
    let golden = read("demo/ic-2025-12-31.json");
    let pairs = golden["result"]["pairs"].as_array().unwrap().clone();
    let mut a = app();
    run_script(&mut a, "3", 140, 40);
    check_l1(&mut a, &pairs);
    // every bridge (L2) equals the golden's bridge numbers
    let n = a.ic_rows().len();
    for i in 0..n {
        let mut b = app();
        run_script(&mut b, "3", 140, 40);
        for _ in 0..i {
            key(&mut b, "j");
        }
        let id = s(&b.ic_rows()[i]["id"]);
        let p = pairs.iter().find(|p| p["id"] == id.as_str()).unwrap();
        key(&mut b, "enter");
        let txt = screen(&mut b, 140, 40);
        for (label, k) in [
            ("Difference at year end", "difference"),
            ("= prior-year opening difference", "opening_difference"),
            ("movement difference", "movement_difference"),
            ("below", "movement_explained"),
            ("Unexplained residual", "residual"),
        ] {
            let line = txt
                .lines()
                .find(|l| l.contains(label))
                .unwrap_or_else(|| panic!("{id} {label}\n{txt}"));
            assert!(
                line.trim_end()
                    .trim_end_matches(" ✓")
                    .ends_with(&fmt_opt(&p[k])),
                "{id} {label}: {line}"
            );
        }
        for it in p["movement_items"].as_array().unwrap() {
            let line = txt
                .lines()
                .find(|l| l.contains(it["date"].as_str().unwrap()))
                .unwrap();
            assert!(line.contains(&fmt_opt(&it["amount"])), "{id} item: {line}");
        }
    }
}

#[test]
fn screen_equals_cli_ic_json() {
    // ic-all (summary text) and ic-pair (the bridge envelope) on the CLI fixture
    let all = read("cli/goldens/ic-all.json");
    let summary = all["text"]
        .as_str()
        .unwrap()
        .lines()
        .find(|l| l.starts_with("summary: "))
        .unwrap()
        .to_string();
    let mut a = cli_app();
    run_script(&mut a, "3", 140, 40);
    let txt = screen(&mut a, 140, 40);
    for kv in summary.trim_start_matches("summary: ").split(", ") {
        let (k, v) = kv.split_once('=').unwrap();
        assert!(txt.contains(&format!("{v} {k}")), "{kv}:\n{txt}");
    }
    let pair = read("cli/goldens/ic-pair.json")["json"][0]["data"]["pairs"][0].clone();
    check_l1(&mut a, std::slice::from_ref(&pair));
    let pos = a.ic_rows().iter().position(|p| p["id"] == "hq-co").unwrap();
    for _ in 0..pos {
        key(&mut a, "j");
    }
    key(&mut a, "enter");
    let txt = screen(&mut a, 140, 40);
    assert!(
        txt.lines()
            .find(|l| l.contains("Unexplained residual"))
            .unwrap()
            .trim_end_matches(" ✓")
            .ends_with(&fmt_opt(&pair["residual"]))
    );
    // the known items of hq-co, both not found (G5)
    for k in pair["known_items"].as_array().unwrap() {
        assert!(
            txt.contains(&format!("   ! not found in the record · {}", s(&k["text"]))),
            "{txt}"
        );
    }
}

#[test]
fn screen_equals_engine_fixture_all_statuses() {
    let golden = read("ic-2025-12-31.json");
    let pairs = golden["result"]["pairs"].as_array().unwrap().clone();
    let mut a = ic_app(theme::LEDGER_DARK, false);
    run_script(&mut a, "3", 140, 40);
    check_l1(&mut a, &pairs);
}

#[test]
fn f1_f2_f3_l1() {
    let mut a = app();
    run_script(&mut a, "3", 140, 40);
    let txt = screen(&mut a, 140, 40);
    assert!(
        txt.lines().nth(3).unwrap().contains("Intercompany › pairs"),
        "F1 crumb"
    );
    assert!(txt.contains("Intercompany at 2025-12-31 · balances: cached Xero TBs (oldest fetched 07:46 UTC) · items: documents"), "F1 strip");
    assert!(
        txt.contains("4 pairs · 1 matches · 2 bridged · 1 unexplained"),
        "F2"
    );
    for c in [
        "pair",
        "side A",
        "side B (party)",
        "difference",
        "residual",
        "status",
    ] {
        assert!(txt.contains(c), "F3 {c}");
    }
}

#[test]
fn f2_unexplained_in_error_f5_statuses_f6_default_sort() {
    let mut a = ic_app(theme::LEDGER_DARK, false);
    run_script(&mut a, "3", 140, 40);
    let buf = snapshot::render(&mut a, 140, 40);
    let txt = snapshot::text(&buf);
    let (y, line) = txt
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains("3 unexplained"))
        .unwrap();
    let x = line[..line.find("3 unexplained").unwrap()].chars().count() as u16;
    assert_eq!(
        buf[(x, y as u16)].fg,
        ownpurse_tui::theme::LEDGER_DARK
            .fg(ownpurse_tui::theme_tokens::Token::Error)
            .fg
            .unwrap(),
        "F2 error"
    );
    assert!(
        txt.contains("not supported") && !txt.contains("not supported: "),
        "F5: status only"
    );
    // F6: unexplained first, then abs(difference) desc
    let rows: Vec<IcRow> = a.ic_rows().iter().map(|p| IcRow::of(p)).collect();
    let first_other = rows.iter().position(|r| r.status != "unexplained").unwrap();
    assert!(
        rows[..first_other]
            .iter()
            .all(|r| r.status == "unexplained")
    );
    let un: Vec<_> = rows[..first_other]
        .iter()
        .map(|r| r.difference.unwrap().abs())
        .collect();
    assert!(un.windows(2).all(|w| w[0] >= w[1]), "{un:?}");
    // F5 not compared at 2025-06-30 (demo)
    let mut d = app();
    d.ui.as_of = "2025-06-30".into();
    d.invalidate();
    run_script(&mut d, "3", 140, 40);
    let txt = screen(&mut d, 140, 40);
    assert!(txt.contains("— not compared"), "{txt}");
    assert!(txt.contains("not compared for design, hold, lab, roast: no cached Xero TB at 2025-06-30 (⌃P → Refetch)"), "{txt}");
}

#[test]
fn f7_no_pairs() {
    let mut a = app();
    a.source.ic_config = serde_json::json!({});
    a.invalidate();
    run_script(&mut a, "3", 100, 30);
    assert!(screen(&mut a, 100, 30).contains(
        "No intercompany pairs configured. Add \"intercompany.pairs\" to ownpurse.json (docs/ROADMAP.md §2)."
    ));
}

#[test]
fn sort_cycles_and_never_drops_rows_and_filter_and_scope() {
    let mut a = ic_app(theme::LEDGER_DARK, false);
    run_script(&mut a, "3", 120, 30);
    let n = a.ic_rows().len();
    for mode in 1..=3 {
        run_script(&mut a, "s", 120, 30);
        assert_eq!(a.ic_rows().len(), n, "sort {}", SORTS[mode % 3]);
    }
    assert_eq!(a.ui.ic_sort, 0, "status → difference → pair → status");
    run_script(&mut a, "/ r e s i d", 120, 30);
    assert_eq!(a.ic_rows().len(), 1);
    run_script(&mut a, "enter esc", 120, 30);
    assert_eq!(a.ic_rows().len(), n);
    // scope: one = either side; group pair (hq+co) = both sides in the group
    a.ui.scope = Scope::Org(0); // one
    assert!(
        a.ic_rows()
            .iter()
            .all(|p| p["a_org"] == "one" || p["b_org"] == "one")
    );
    assert_eq!(a.ic_rows().len(), 1);
    a.ui.scope = Scope::Group(0);
    assert!(a.ic_rows().iter().all(|p| {
        ["hq", "co"].contains(&p["a_org"].as_str().unwrap())
            && ["hq", "co"].contains(&p["b_org"].as_str().unwrap())
    }));
}

#[test]
fn not_supported_and_not_compared_refuse_to_open() {
    let mut a = ic_app(theme::LEDGER_DARK, false);
    run_script(&mut a, "3 s s j j j j j enter", 120, 30);
    let band = a.ui.band.clone().expect("refusal");
    assert_eq!(band.kind, BandKind::Refusal);
    assert_eq!(
        band.text,
        "Nobody ↔ Co: org `nobody` isn't connected or configured. Check the pair in ownpurse.json."
    );
    assert_eq!(a.ic_level(), 1);
}

#[test]
fn g1_g2_g3_g4_g6_bridge() {
    let mut a = app();
    a.ui.details = true; // the provenance line is a detail (design pass 2)
    run_script(&mut a, "3 j", 140, 40);
    key(&mut a, "enter");
    let buf = snapshot::render(&mut a, 140, 40);
    let txt = snapshot::text(&buf);
    assert!(
        txt.contains(
            "residual 0.00 ✓ · balances: Xero TB hold 07:46 · design 07:46 · items from documents"
        ),
        "G1 {txt}"
    );
    let order = [
        "Difference at year end",
        "= prior-year opening difference",
        "+ 2025 movement difference",
        "explained by 1 item below",
        "Unexplained residual",
    ];
    let ys: Vec<usize> = order
        .iter()
        .map(|l| {
            txt.lines()
                .position(|x| x.contains(l))
                .unwrap_or_else(|| panic!("G2 {l}\n{txt}"))
        })
        .collect();
    assert!(ys.windows(2).all(|w| w[0] < w[1]), "G2 order");
    let ry = ys[4] as u16;
    let rx = row_text(&buf, ry).find("Unexplained").unwrap() as u16;
    assert!(
        buf[(rx, ry)]
            .modifier
            .contains(ratatui::style::Modifier::BOLD),
        "G3 bold"
    );
    assert!(row_text(&buf, ry - 1).contains('─'), "G3 rule above");
    assert!(txt.contains("Items on one side only"), "G4");
    for c in ["side", "date", "account", "amount", "why"] {
        assert!(txt.contains(c), "G4 {c}");
    }
    assert!(
        txt.contains(
            "why: no party contact; no matching entry in hold  ·  BankTransactions bt-307"
        ),
        "G4 detail"
    );
    assert!(
        txt.contains("known items: none") && txt.contains("context: none"),
        "G5 none"
    );
    let last = txt.lines().last().unwrap();
    assert!(last.contains("a A") && last.contains("b B"), "G6 {last}");
}

#[test]
fn g1_g3_unexplained_residual_in_error() {
    let mut a = ic_app(theme::LEDGER_DARK, false);
    a.ui.details = true;
    run_script(&mut a, "3 enter", 140, 40);
    let buf = snapshot::render(&mut a, 140, 40);
    let txt = snapshot::text(&buf);
    assert!(
        txt.contains("residual (194.42) · balances") && !txt.contains("(194.42) ✓"),
        "G1 {txt}"
    );
    let y = txt
        .lines()
        .position(|l| l.contains("Unexplained residual"))
        .unwrap() as u16;
    let x = row_text(&buf, y).find("Unexplained").unwrap() as u16;
    assert_eq!(
        buf[(x, y)].fg,
        ownpurse_tui::theme::LEDGER_DARK
            .fg(ownpurse_tui::theme_tokens::Token::Error)
            .fg
            .unwrap()
    );
}

#[test]
fn g5_known_items_found_and_not_found_and_context() {
    let mut a = ic_app(theme::LEDGER_DARK, false);
    run_script(&mut a, "3 s s j j j enter", 140, 40);
    let txt = screen(&mut a, 140, 40);
    let lines: Vec<&str> = txt.lines().collect();
    let k = lines
        .iter()
        .position(|l| l.trim() == "known items:")
        .expect("heading");
    assert_eq!(
        lines[k + 1].trim_end(),
        "    ✓ found 2024-05-01 hq Lease Recharge Receivable (1405) 1,013.47 · 2024 lease"
    );
    assert!(
        txt.contains("    ! not found in the record · inexact float"),
        "{txt}"
    );
    let c = lines
        .iter()
        .position(|l| l.trim() == "context:")
        .expect("context heading");
    assert_eq!(
        lines[c + 1].trim_end(),
        "    hq Co Settlement Clearing (2405) Cr 311.83"
    );
}

#[test]
fn e6_l3_in_tab_via_pair_and_esc_back_with_cursor() {
    let mut a = app();
    run_script(&mut a, "3 j enter enter", 140, 40);
    assert_eq!((a.ui.tab, a.ic_level()), (Tab::Intercompany, 3));
    let txt = screen(&mut a, 140, 40);
    assert!(
        txt.lines().next().unwrap().contains("● hold ↔ ● design"),
        "E6 header: the pair chip (v0.12.0)"
    );
    assert!(txt.lines().nth(1).unwrap().contains("3 Intercompany"));
    assert!(
        txt.contains("received from") || txt.contains("paid to") || txt.contains("Lines"),
        "Ledger L3 widget"
    );
    assert_eq!(a.ui.scope, Scope::All, "scope unchanged");
    // N4: leave to the Ledger and come back: still the same document
    run_script(&mut a, "2 3", 140, 40);
    assert_eq!(a.ic_level(), 3);
    let item = a.ic_nav().item.clone();
    run_script(&mut a, "esc", 140, 40);
    assert_eq!(a.ic_level(), 2);
    assert_eq!(a.ic_nav().item, item, "item cursor kept");
    run_script(&mut a, "esc", 140, 40);
    assert_eq!(a.ic_level(), 1);
    assert_eq!(
        a.ic_nav().pair.as_deref(),
        Some("hold-design"),
        "pair cursor kept"
    );
}

#[test]
fn a_and_b_open_the_sides_in_the_ledger_and_esc_returns_to_the_bridge() {
    let mut a = app();
    run_script(&mut a, "3 j enter a", 140, 40);
    assert_eq!(a.ui.tab, Tab::Ledger);
    assert_eq!(a.ledger_level(), 2, "one account: straight to its postings");
    let txt = screen(&mut a, 140, 40);
    assert!(
        txt.lines().next().unwrap().contains("● hold ↔ ● design"),
        "{txt}"
    );
    assert!(
        txt.lines().nth(3).unwrap().contains(
            "Brackenridge ↔ Quillfeather (management fees) › … › Fees Receivable from Quillfeather (1470)"
        ),
        "{txt}"
    );
    run_script(&mut a, "esc", 140, 40);
    assert_eq!(
        (a.ui.tab, a.ic_level()),
        (Tab::Intercompany, 2),
        "Esc returns to the bridge"
    );
    assert!(a.ui.via.is_none());
    // two accounts (engine fixture hq side): L1 limited to them
    let mut b = ic_app(theme::LEDGER_DARK, false);
    run_script(&mut b, "3 s s j j enter a", 140, 40);
    assert_eq!((b.ui.tab, b.ledger_level()), (Tab::Ledger, 1));
    assert_eq!(b.l1_visible().len(), 2, "only the side's accounts");
    run_script(&mut b, "esc", 140, 40);
    assert_eq!(b.ui.tab, Tab::Intercompany);
}

#[test]
fn o_and_y_on_an_item_and_a_document() {
    let mut a = app();
    run_script(&mut a, "3 j enter", 140, 40);
    let fx = key(&mut a, "y");
    assert_eq!(fx, vec![Effect::Copy("bt-307".into())]);
    let fx = key(&mut a, "o");
    assert_eq!(
        fx,
        vec![Effect::Open(
            "https://go.xero.com/Bank/ViewTransaction.aspx?bankTransactionID=bt-307".into()
        )]
    );
}

#[test]
fn per_scope_state() {
    let mut a = app();
    run_script(&mut a, "3 enter", 140, 40);
    run_script(&mut a, "]", 140, 40);
    assert_eq!(a.ic_level(), 1, "roast has its own state");
    run_script(&mut a, "[", 140, 40);
    assert_eq!(a.ic_level(), 2, "back on all: the bridge is still open");
}

#[test]
fn v6_strip_pinned_and_whole_at_every_width() {
    for script in ["3", "3 j enter", "3 j enter enter"] {
        let mut a = app();
        a.ui.details = true; // the strip shows whole when it shows (provenance is a detail)
        run_script(&mut a, script, 100, 30);
        let want = if script == "3" {
            "Intercompany at 2025-12-31 · balances: cached Xero TBs (oldest fetched 07:46 UTC) · items: documents"
        } else {
            "residual 0.00 ✓ · balances: Xero TB hold 07:46 · design 07:46 · items from documents"
        };
        for w in 60..=220u16 {
            let txt = screen(&mut a, w, 30);
            let strip: Vec<&str> = txt.lines().skip(4).take(3).map(str::trim).collect();
            assert!(strip.join(" ").contains(want), "{script} at {w}:\n{txt}");
        }
    }
}

#[test]
fn v7_v8_residual_and_amounts_never_hidden() {
    for (make, label) in [
        (app as fn() -> App, "demo"),
        (|| ic_app(theme::LEDGER_DARK, false), "engine"),
    ] {
        for th in [theme::LEDGER_DARK, theme::NO_COLOR_ASCII] {
            for sort in 0..3 {
                for filter in ["", "co"] {
                    let mut a = make();
                    a.ui.theme = th;
                    a.ui.ic_sort = sort;
                    a.ui.filter = filter.into();
                    run_script(&mut a, "3", 100, 30);
                    let rows: Vec<IcRow> = a.ic_rows().iter().map(|p| IcRow::of(p)).collect();
                    for w in 60..=220u16 {
                        let txt = screen(&mut a, w, 30);
                        let header = txt
                            .lines()
                            .find(|l| l.contains("difference") && l.contains("pair  "))
                            .unwrap_or_else(|| panic!("{w}\n{txt}"));
                        assert!(
                            header.contains("residual"),
                            "{label} sort={sort} filter={filter:?} {w}: residual column\n{txt}"
                        );
                        for r in &rows {
                            for v in [r.difference, r.residual].iter().flatten() {
                                let f = ownpurse_core::fmt(*v);
                                assert!(
                                    txt.contains(&f),
                                    "{label} sort={sort} filter={filter:?} w={w}: {f} missing\n{txt}"
                                );
                            }
                        }
                    }
                }
            }
        }
    }
    // the bridge's residual row and item amounts at every width
    let mut a = ic_app(theme::LEDGER_DARK, false);
    run_script(&mut a, "3 enter", 100, 30);
    for w in 60..=220u16 {
        let txt = screen(&mut a, w, 30);
        let line = txt
            .lines()
            .find(|l| l.contains("Unexplained residual"))
            .unwrap_or_else(|| panic!("{w}\n{txt}"));
        assert!(line.ends_with("(194.42)"), "{w}: {line}");
        assert!(txt.contains("204.63  ") || txt.contains(" 204.63"), "{w}");
    }
}

#[test]
fn demo_variant_unexplained_and_not_supported() {
    let golden = read("demo/ic-variant-2025-12-31.json");
    let pairs = golden["result"]["pairs"].as_array().unwrap().clone();
    let mut a = snapshot::demo_ic_app(theme::LEDGER_DARK, false);
    run_script(&mut a, "3", 140, 40);
    check_l1(&mut a, &pairs);
    let txt = screen(&mut a, 140, 40);
    assert!(
        txt.contains("6 pairs · 1 matches · 2 bridged · 2 unexplained · 1 not supported"),
        "{txt}"
    );
    run_script(&mut a, "down down", 140, 40);
    let txt = screen(&mut a, 140, 40);
    let detail = "Tallowmere ↔ Rösterei: different base currencies (roast USD, rhein EUR). Cross-org views refuse different currencies or fiscal years.";
    assert!(txt.contains(detail), "detail line: {txt}");
    run_script(&mut a, "enter", 140, 40);
    assert_eq!(a.ui.band.as_ref().unwrap().text, detail);
    run_script(&mut a, "esc up up enter", 140, 40);
    let txt = screen(&mut a, 140, 40);
    assert!(txt.contains("1,412.57 unexplained"), "{txt}");
}

#[test]
fn side_panel_without_accounts_says_so() {
    let mut a = ic_app(theme::LEDGER_DARK, false);
    run_script(&mut a, "3 enter", 80, 30);
    let txt = screen(&mut a, 80, 30);
    assert!(
        txt.contains("A one 0.00 · no paired accounts with postings"),
        "{txt}"
    );
}
