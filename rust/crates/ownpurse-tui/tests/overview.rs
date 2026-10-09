//! Overview (tab 1): content parity with the CLI goldens (`parity` and `ic` on the same
//! fixture), the M1 key paths, the declared degrade order and V8 for its table.

use ownpurse_tui::app::{App, Msg, OvCol, Scope, Tab, update};
use ownpurse_tui::hit::Target;
use ownpurse_tui::keymap::Key;
use ownpurse_tui::snapshot::{self, fixture_app, fixtures_dir, row_text, run_script};
use ownpurse_tui::theme;
use serde_json::Value;

fn app() -> App {
    fixture_app(theme::LEDGER_DARK, false)
}

/// With the details layer on (`i`): the record times and calls-left columns (design pass 2).
fn app_d() -> App {
    let mut a = app();
    a.ui.details = true;
    a
}

fn demo_golden(name: &str) -> Value {
    let path = fixtures_dir().join("demo").join(name);
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

/// The screen as a grid of cells (one char per cell for the widths used here).
fn grid(app: &mut App, w: u16, h: u16) -> Vec<Vec<char>> {
    let buf = snapshot::render(app, w, h);
    (0..h)
        .map(|y| row_text(&buf, y).chars().collect())
        .collect()
}

fn find(rows: &[Vec<char>], needle: &str) -> Option<(usize, usize)> {
    let n: Vec<char> = needle.chars().collect();
    rows.iter().enumerate().find_map(|(y, r)| {
        (0..r.len().saturating_sub(n.len() - 1))
            .find(|&x| r[x..x + n.len()] == n[..])
            .map(|x| (y, x))
    })
}

/// The text of the cell in `row` under the column whose title starts at `x`, up to the next 2 spaces.
fn cell(rows: &[Vec<char>], y: usize, x: usize) -> String {
    let r = &rows[y];
    let mut out = String::new();
    let mut i = x;
    while i < r.len() && !(r[i] == ' ' && r.get(i + 1) == Some(&' ')) {
        out.push(r[i]);
        i += 1;
    }
    out.trim().to_string()
}

#[test]
fn screen_equals_the_parity_golden_per_basis() {
    // demo/parity-2025-12-31.json: the frozen basis results (auto = year, history) on the demo record
    let parity = demo_golden("parity-2025-12-31.json");
    let mut a = app_d();
    let g = grid(&mut a, 140, 40);
    let (hy, _) = find(&g, "parity (year)").unwrap();
    let col = |t: &str| find(&g[hy..=hy], t).unwrap().1;
    let (year_x, hist_x, tb_x) = (
        col("parity (year)"),
        col("parity (history)"),
        col("TB fetched"),
    );
    let label = |v: &Value| match v["matches_xero"].as_bool() {
        Some(true) => "✓ exact".to_string(),
        Some(false) => format!("{} accounts differ", v["differs"]),
        None => "— not checked".to_string(),
    };
    for (org, v) in parity.as_object().unwrap() {
        let y = (hy + 1..g.len())
            .find(|&y| cell(&g, y, 2) == *org)
            .unwrap_or_else(|| panic!("row {org}"));
        assert_eq!(cell(&g, y, year_x), label(&v["auto"]), "{org} year");
        assert_eq!(cell(&g, y, hist_x), label(&v["history"]), "{org} history");
        assert_eq!(cell(&g, y, tb_x), "2026-10-05 07:46", "{org} TB fetched");
    }
}

#[test]
fn acceptance_b1_b2_b5_b6() {
    let mut a = app_d();
    let screen = snapshot::text(&snapshot::render(&mut a, 140, 40));
    for h in [
        "org",
        "fetched",
        "parity (year)",
        "parity (history)",
        "calls left",
    ] {
        assert!(screen.contains(h), "B1 {h}");
    }
    let roast = screen.lines().find(|l| l.contains(" roast ")).unwrap();
    assert!(
        roast.contains("✓ exact") && roast.contains("2 accounts differ"),
        "B2 {roast}"
    );
    let design = screen.lines().find(|l| l.contains(" design ")).unwrap();
    assert!(
        design.contains("15 accounts differ") && design.contains("938/1,000"),
        "{design}"
    );
    assert!(
        screen.contains("Intercompany")
            && screen.contains("4 pairs   1 match   2 bridged   1 unexplained"),
        "B5"
    );
    assert!(screen.contains("Needs attention"), "B6");
    let s80 = snapshot::text(&snapshot::render(&mut a, 80, 24));
    assert!(
        s80.lines()
            .find(|l| l.contains(" roast "))
            .unwrap()
            .contains("differ"),
        "B2 at S: {s80}"
    );
}

#[test]
fn acceptance_b7_b8_b9_variants() {
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "stale");
    a.ui.details = true;
    let s = snapshot::text(&snapshot::render(&mut a, 140, 40));
    assert!(
        s.lines()
            .find(|l| l.contains(" lab "))
            .unwrap()
            .contains("stale"),
        "B7 row"
    );
    assert!(
        s.contains("! lab record is 2d old (TTL 1d). ⌃P → Sync lab"),
        "B7 attention:\n{s}"
    );
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "never");
    a.ui.details = true;
    let s = snapshot::text(&snapshot::render(&mut a, 140, 40));
    assert!(
        s.lines()
            .find(|l| l.contains(" lab "))
            .unwrap()
            .contains("no local record"),
        "B8 row"
    );
    assert!(
        // the confirm's figure (core `estimate_or_floor`): one number everywhere
        s.contains("! No local record for lab yet. ⌃P → Sync lab · ≈ 15+ calls, first full sync."),
        "B8:\n{s}"
    );
    let mut a = app();
    a.ui.as_of = "2025-06-30".into();
    a.invalidate();
    let s = snapshot::text(&snapshot::render(&mut a, 140, 40));
    assert!(
        s.contains(
            "! 4 pairs not compared: no cached Xero TB at 2025-06-30. ⌃P → Refetch (≈ 4 calls)"
        ),
        "B9:\n{s}"
    );
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "lowbudget");
    a.ui.details = true;
    let buf = snapshot::render(&mut a, 140, 40);
    let s = snapshot::text(&buf);
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(" design "))
        .unwrap();
    let x = line.find("62/1,000").expect("design 62 left") as u16;
    let x = line[..x as usize].chars().count() as u16;
    assert_eq!(
        buf[(x, y as u16)].fg,
        ownpurse_tui::theme::LEDGER_DARK
            .fg(ownpurse_tui::theme_tokens::Token::Error)
            .fg
            .unwrap(),
        "low budget in error"
    );
}

#[test]
fn screen_equals_the_intercompany_golden_summary() {
    let sm = demo_golden("ic-2025-12-31.json")["result"]["summary"].clone();
    let mut a = app();
    let screen = snapshot::text(&snapshot::render(&mut a, 140, 40));
    let line = screen.lines().find(|l| l.contains(" pairs ")).unwrap();
    // the same counts as the golden; nouns go through the one plural helper (1 pair / 1 match)
    for (k, one) in [
        ("pairs", "pair"),
        ("matches", "match"),
        ("bridged", "bridged"),
        ("unexplained", "unexplained"),
    ] {
        let n = sm[k].as_i64().unwrap();
        let word = if n == 1 { one } else { k };
        assert!(line.contains(&format!("{n} {word}")), "{k} not in {line}");
    }
}

fn key(a: &mut App, k: &str) {
    update(a, Msg::Key(Key::parse(k).unwrap().event()));
}

#[test]
fn m1_overview_parity_year_to_ledger_l1_differs_year() {
    let mut a = app();
    key(&mut a, "right");
    assert_eq!(a.ui.overview_col, OvCol::ParityYear);
    key(&mut a, "enter");
    assert_eq!((a.ui.tab, a.ui.scope.clone()), (Tab::Ledger, Scope::Org(0)));
    assert!(a.ui.differs_only, "! filter on");
    assert!(!a.ui.full_history, "year basis");
    key(&mut a, "esc");
    assert_eq!(a.ui.tab, Tab::Overview);
    assert_eq!(a.ui.overview_col, OvCol::ParityYear, "same cell");
    assert_eq!(a.ui.overview_sel.as_deref(), Some("roast"), "same row");
    assert!(!a.ui.differs_only && !a.ui.full_history);
}

#[test]
fn m1_overview_parity_history_to_ledger_l1_differs_history() {
    let mut a = app();
    run_script(&mut a, "j j right right enter", 100, 30);
    assert_eq!((a.ui.tab, a.ui.scope.clone()), (Tab::Ledger, Scope::Org(2)));
    assert!(a.ui.differs_only && a.ui.full_history);
    run_script(&mut a, "esc", 100, 30);
    assert_eq!(
        (a.ui.tab, a.ui.overview_col),
        (Tab::Overview, OvCol::ParityHistory)
    );
    assert_eq!(a.ui.overview_sel.as_deref(), Some("design"));
}

#[test]
fn m1_click_and_double_click_on_a_parity_cell() {
    let mut a = app();
    snapshot::render(&mut a, 140, 40);
    let r = a
        .hits
        .find(Target::Cell(2, 3))
        .expect("history cell of design");
    run_script(&mut a, &format!("<click:{},{}>", r.x + 1, r.y), 140, 40);
    assert_eq!(a.ui.overview_sel.as_deref(), Some("design"));
    assert_eq!(a.ui.overview_col, OvCol::ParityHistory);
    assert_eq!(a.ui.tab, Tab::Overview, "a click never drills");
    run_script(&mut a, &format!("<dclick:{},{}>", r.x + 1, r.y), 140, 40);
    assert_eq!(a.ui.tab, Tab::Ledger);
    assert!(a.ui.differs_only && a.ui.full_history);
}

#[test]
fn b3_v8_parity_words_and_values_whole_at_every_width() {
    for th in [theme::LEDGER_DARK, theme::NO_COLOR_ASCII] {
        let mut a = fixture_app(th, false);
        a.ui.details = true;
        for w in 60..=220u16 {
            let g = grid(&mut a, w, 30);
            let (hy, _) = find(&g, "  org  ").unwrap();
            let header: String = g[hy].iter().collect();
            assert!(
                header.contains("year") && header.contains("history"),
                "{w}: {header}"
            );
            assert!(header.contains("calls"), "{w}: {header}");
            for r in g[hy + 1..hy + 6]
                .iter()
                .filter(|r| !r.iter().collect::<String>().contains("Enter"))
                .take(4)
            {
                let row: String = r.iter().collect();
                assert!(
                    !row.contains('…') && !row.contains('~'),
                    "{w}: cut cell in {row}"
                );
                assert!(
                    ["967", "978", "938"].iter().any(|n| row.contains(n)),
                    "{w}: calls left in {row}"
                );
                let parity = ["exact", "differ", "not checked"];
                assert!(parity.iter().any(|p| row.contains(p)), "{w}: {row}");
            }
        }
    }
}

#[test]
fn declared_degrade_order() {
    let mut a = app_d();
    let header = |a: &mut App, w| {
        let g = grid(a, w, 30);
        let (hy, _) = find(&g, "  org  ").unwrap();
        g[hy].iter().collect::<String>()
    };
    let h140 = header(&mut a, 140);
    assert!(
        h140.contains("TB fetched") && h140.contains("issues") && h140.contains("parity (history)")
    );
    let h80 = header(&mut a, 80);
    assert!(!h80.contains("TB fetched"), "{h80}");
    let screen80 = snapshot::text(&snapshot::render(&mut a, 80, 24));
    assert!(
        screen80.contains("differ"),
        "history stays whole: {screen80}"
    );
}

#[test]
fn overview_is_prepared_once_per_as_of() {
    let mut a = app();
    let first = a.overview.clone().unwrap();
    run_script(&mut a, "2 1 ] [ j k", 100, 30);
    assert!(
        std::sync::Arc::ptr_eq(&first, a.overview.as_ref().unwrap()),
        "tab/scope/cursor reuse the model"
    );
    a.ui.as_of = "2025-06-30".into();
    update(&mut a, Msg::Action("tab.ledger"));
    update(&mut a, Msg::Action("tab.overview"));
    assert_eq!(a.overview.as_ref().unwrap().as_of, "2025-06-30");
}

#[test]
fn b7_estimate_and_never_synced_dashes() {
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "stale");
    a.ui.details = true;
    let s = snapshot::text(&snapshot::render(&mut a, 140, 40));
    // ≈ 15 = core `estimate`, equal to the frozen estimate (demo/estimate-2026-10-05.json)
    assert!(
        s.contains("! lab record is 2d old (TTL 1d). ⌃P → Sync lab (≈ 15 calls)."),
        "{s}"
    );
    let mut a = snapshot::demo_app_variant(theme::LEDGER_DARK, false, "never");
    a.ui.details = true;
    let s = snapshot::text(&snapshot::render(&mut a, 140, 40));
    let row = s.lines().find(|l| l.contains(" lab ")).unwrap();
    // unknown issues read —; calls left is the local estimate, marked as one
    assert!(!row.contains("none"), "unknown issues read —: {row}");
    assert!(row.contains("≈ 1,000 (estimated)"), "{row}");
    assert!(!row.contains("1,000/1,000"), "{row}");
    assert_eq!(
        row.matches("—").count(),
        4,
        "2 × not checked, issues, TB fetched: {row}"
    );
}

#[test]
fn selected_parity_cell_reads_without_colour_and_footer_says_what_enter_does() {
    let mut a = fixture_app(theme::NO_COLOR_ASCII, false);
    run_script(&mut a, "right right", 100, 30);
    let s = snapshot::text(&snapshot::render(&mut a, 100, 30));
    let roast = s.lines().find(|l| l.contains(" roast ")).unwrap();
    assert!(
        roast.contains("> 2 accounts differ"),
        "mark before the value: {roast}"
    );
    assert!(
        s.contains("Enter -> Ledger, only the 2 accounts that differ (full history)"),
        "{s}"
    );
    let mut a = app();
    run_script(&mut a, "right", 100, 30);
    let buf = snapshot::render(&mut a, 100, 30);
    let s = snapshot::text(&buf);
    assert!(s.contains("Enter → Ledger (year basis)"), "{s}");
    let (y, line) = s
        .lines()
        .enumerate()
        .find(|(_, l)| l.contains(" roast "))
        .unwrap();
    let x = line[..line.find("✓ exact").unwrap()].chars().count() as u16;
    assert!(
        buf[(x, y as u16)]
            .modifier
            .contains(ratatui::style::Modifier::REVERSED)
    );
}
