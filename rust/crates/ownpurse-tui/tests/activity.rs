//! Activity tab (spec §9 row 4, M2; acceptance K1–K5) on the demo fixture, plus its snapshot grid
//! (S/M/L × 4 theme modes). Snapshots are reviewed with `cargo insta review`.

use ownpurse_tui::activity::StatusClass;
use ownpurse_tui::app::{App, Tab, update};
use ownpurse_tui::snapshot::{
    self, demo_app, demo_app_variant, header_problems, row_text, run_script, styled, tab_state_app,
    tab_states, text, themes,
};
use ownpurse_tui::theme::{self, Theme};
use ownpurse_tui::theme_tokens::Token;
use ownpurse_tui::views::activity;

fn screen(app: &mut App, w: u16, h: u16) -> String {
    text(&snapshot::render(app, w, h))
}

fn at(script: &str, w: u16, h: u16) -> (App, String) {
    let mut app = demo_app(theme::LEDGER_DARK, false);
    // these checks read the details layer (today's counts, the per-call columns; design pass 2)
    app.ui.details = true;
    run_script(&mut app, script, w, h);
    let t = screen(&mut app, w, h);
    (app, t)
}

#[test]
fn k1_budget_summary() {
    for (w, h) in snapshot::SML.iter().copied() {
        let (_, t) = at("4", w, h);
        assert!(t.contains("Today (UTC): 139 calls"), "{w}\n{t}");
        assert!(
            t.contains("Per minute: 0 of 55 (soft limit; Xero allows 60)"),
            "{w}\n{t}"
        );
        for (org, n, left) in [
            ("roast", 33, 967),
            ("hold", 22, 978),
            ("design", 62, 938),
            ("lab", 22, 978),
        ] {
            // padded to the longest alias in scope, so the columns line up
            let want = format!("{org:<6}  {n} calls today · {left} left of 1,000");
            assert!(t.contains(&want), "{w}: {want}\n{t}");
        }
        // last sync per org, at every size (full at L, short at S)
        assert!(t.contains("synced 2h ago") || t.contains("last sync 07:46 UTC · 2h ago"));
    }
    // the summary is 3–5 lines for four orgs
    let app = demo_app(theme::LEDGER_DARK, false);
    for w in [58, 78, 98, 138] {
        let n = activity::summary(&app, w).len();
        assert!((3..=5).contains(&n), "{w}: {n} lines");
    }
}

#[test]
fn k2_log_columns_and_status_never_hidden() {
    let (_, t) = at("4", 140, 40);
    let head = t.lines().find(|l| l.contains("endpoint")).expect("header");
    for col in [
        "time", "org", "endpoint", "result", "day left", "method", "ms",
    ] {
        assert!(head.contains(col), "{col} in {head}");
    }
    for w in 60..=220u16 {
        let (_, t) = at("4 ] ] ]", w, 30);
        let head = t.lines().find(|l| l.contains("result")).unwrap_or("");
        assert!(
            head.contains("result") && head.contains("day left"),
            "{w}: {head}"
        );
        assert!(t.contains("429"), "{w}: 429 visible");
        if w < 70 {
            assert!(!head.contains("method"), "{w}: method hides first");
        }
    }
}

fn status_cell_token(app: &mut App, code: &str, w: u16, h: u16) -> (Theme, ratatui::style::Style) {
    let buf = snapshot::render(app, w, h);
    let head_y = (0..h)
        .find(|&y| row_text(&buf, y).contains("result") && row_text(&buf, y).contains("day left"))
        .expect("log header");
    let col = row_text(&buf, head_y).find("result").expect("result col") as u16;
    for y in head_y + 1..h {
        let row = row_text(&buf, y);
        if row.get(col as usize..).is_some_and(|r| r.starts_with(code)) {
            return (app.ui.theme, buf[(col, y)].style());
        }
    }
    panic!("no {code} row");
}

#[test]
fn k3_status_styles() {
    for (script, code, token) in [
        ("4 ] ]", "200", Token::Muted),
        ("4 ] ]", "404", Token::Warn),
        ("4 ] ] ]", "429", Token::Error),
    ] {
        let mut app = demo_app(theme::LEDGER_DARK, false);
        app.ui.details = true;
        run_script(&mut app, script, 140, 40);
        let (th, style) = status_cell_token(&mut app, code, 140, 40);
        assert_eq!(style.fg, th.fg(token).fg, "{code}");
    }
    // the classes behind them
    let app = demo_app(theme::LEDGER_DARK, false);
    let log = &app.model.activity.log;
    let class = |s: i64| log.iter().find(|r| r.status == Some(s)).map(|r| r.class());
    assert_eq!(class(200), Some(StatusClass::Ok));
    assert_eq!(class(404), Some(StatusClass::Client));
    assert_eq!(class(429), Some(StatusClass::Error));
    assert_eq!(class(503), Some(StatusClass::Error));
}

#[test]
fn k4_grouped_by_endpoint() {
    let (app, t) = at("4 G", 100, 30);
    assert!(app.ui.activity.grouped);
    let head = t
        .lines()
        .find(|l| l.contains("avg ms"))
        .expect("grouped header");
    for col in ["endpoint", "calls", "errors", "avg ms", "last"] {
        assert!(head.contains(col), "{col}");
    }
    assert!(t.contains("BankTransactions/{id}/Attachments"));
    let eps = activity::endpoints(&app);
    assert_eq!(eps.iter().map(|e| e.calls).sum::<i64>(), 142);
    assert_eq!(eps.iter().map(|e| e.errors).sum::<i64>(), 3);
    // G again returns to the log
    let (app, _) = at("4 G G", 100, 30);
    assert!(!app.ui.activity.grouped);
}

#[test]
fn k5_arriving_never_takes_the_cursor() {
    let (app, _) = at("/ a esc 4", 100, 30);
    assert_eq!(app.ui.tab, Tab::Activity);
    assert!(app.ui.prompt.is_none());
    let (app, _) = at("4", 100, 30);
    assert!(app.ui.prompt.is_none());
}

#[test]
fn filter_and_scope() {
    let (app, t) = at("4 / 4 0 4 enter", 100, 30);
    // the filter chip (v0.10.1): the query and "shown of total"
    assert!(t.contains("filter: \"404\" · 1 of "), "{t}");
    assert_eq!(activity::rows(&app).len(), 1);
    // scope: roast only (no identity rows, no other org)
    let (app, _) = at("4 ]", 100, 30);
    assert!(activity::rows(&app).iter().all(|r| r.org == "roast"));
    assert_eq!(activity::rows(&app).len(), 33);
    // group all-co = every org row; all adds the identity rows
    let (app, _) = at("4 ] ] ] ] ]", 100, 30);
    assert_eq!(activity::rows(&app).len(), 140);
    let (app, _) = at("4", 100, 30);
    assert_eq!(activity::rows(&app).len(), 142);
    let (_, t) = at("4 / z z z enter", 100, 30);
    assert!(t.contains("No call matches the filter."));
}

#[test]
fn detail_band_and_navigation_keep_ids() {
    let (mut app, t) = at("4 ] ] ] j j", 140, 40);
    assert!(t.contains("status 429 Rate limited (minute)"), "{t}");
    let id = app.ui.activity.sel.expect("selected id");
    // a reload keeps the cursor on the same row id
    let model = app.model.clone();
    update(
        &mut app,
        ownpurse_tui::app::Msg::Reloaded(Box::new((model, None))),
    );
    assert_eq!(app.ui.activity.sel, Some(id));
    assert_eq!(activity::cursor(&app), Some(2));
    // a click selects that row
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, "4", 100, 30);
    let r = app.hits.content.expect("table area");
    run_script(&mut app, &format!("<click:10,{}>", r.y + 4), 100, 30);
    assert_eq!(activity::cursor(&app), Some(3));
    // the wheel scrolls without moving the selection while it stays visible
    run_script(&mut app, "<wheel:down:2>", 100, 30);
    assert_eq!(activity::cursor(&app), Some(3));
    assert_eq!(app.ui.activity.top, 2);
}

#[test]
fn low_budget_org_is_named_not_only_coloured() {
    let mut app = demo_app_variant(theme::NO_COLOR_ASCII, false, "lowbudget");
    app.ui.details = true;
    run_script(&mut app, "4", 80, 24);
    let t = screen(&mut app, 80, 24);
    assert!(
        t.contains("design  62 calls today -  62 left of 1,000 - low"),
        "{t}"
    );
}

#[test]
fn activity_snapshots() {
    let mut problems = Vec::new();
    let mut n = 0;
    for st in tab_states()
        .iter()
        .filter(|s| s.name.starts_with("activity"))
    {
        for (theme_name, theme) in themes() {
            for &(w, h) in st.sizes {
                let mut app = tab_state_app(st, theme, w, h);
                let buf = snapshot::render(&mut app, w, h);
                let name = format!("{}-{w}x{h}-{theme_name}", st.name);
                for p in header_problems(&app, &row_text(&buf, 0)) {
                    problems.push(format!("{name}: {p}"));
                }
                insta::assert_snapshot!(name, styled(&buf));
                n += 1;
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
    assert_eq!(n, 7 * 4 * 3);
}

/// Spec §14: arriving on Activity (and redrawing it) reuses the prepared log, cells and summary; only a
/// reload, a scope or filter change, a resize or the minute rebuilds them.
#[test]
fn arrival_reuses_cached_models() {
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, "4 1", 140, 40);
    let before = activity::builds();
    run_script(&mut app, "4 1 4 1 4 j j k", 140, 40);
    assert_eq!(activity::builds(), before, "tab arrival rebuilt a model");
    run_script(&mut app, "4 ]", 140, 40);
    assert!(activity::builds() > before, "a scope change rebuilds");
    let after_scope = activity::builds();
    let reloaded = ownpurse_tui::model::Model::open(&app.source, app.now).expect("reload");
    update(
        &mut app,
        ownpurse_tui::app::Msg::Reloaded(Box::new((reloaded, None))),
    );
    snapshot::render(&mut app, 140, 40);
    assert!(activity::builds() > after_scope, "a reload rebuilds");
}

/// An overlay paints fresh cells: with an empty query no palette row carries BOLD (no stale highlight or
/// modifier leaking from the screen underneath), in every theme and size.
#[test]
fn palette_empty_query_has_no_highlighted_spans() {
    use ratatui::style::Modifier;
    for (name, th) in themes() {
        for &(w, h) in snapshot::SML {
            let mut app = demo_app(th, false);
            run_script(&mut app, "ctrl-p", w, h);
            let buf = snapshot::render(&mut app, w, h);
            let r = app.hits.overlay.expect("palette box");
            for y in r.top() + 3..r.bottom() - 1 {
                let row = row_text(&buf, y);
                if row.contains('▌') || row.contains('>') && name.contains("ascii") {
                    continue; // the selected row's cursor gutter is accent + bold by design
                }
                for x in r.left() + 1..r.right() - 1 {
                    assert!(
                        !buf[(x, y)].modifier.contains(Modifier::BOLD),
                        "{name} {w}x{h}: bold at {x},{y} in {row:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn per_org_lines_align_and_log_rows_is_honoured() {
    use ownpurse_tui::snapshot::{fixture_app, run_script};
    let mut a = fixture_app(ownpurse_tui::theme::LEDGER_DARK, false);
    a.ui.details = true; // the per-call detail and today's counts are the details layer (design pass 2)
    run_script(&mut a, "4", 140, 40);
    let txt = ownpurse_tui::snapshot::text(&ownpurse_tui::snapshot::render(&mut a, 140, 40));
    let lines: Vec<&str> = txt
        .lines()
        .filter(|l| l.contains(" calls today · "))
        .collect();
    assert!(lines.len() >= 2, "{txt}");
    let col = |l: &str, pat: &str| l.find(pat).map(|i| l[..i].chars().count());
    let todays: Vec<_> = lines.iter().map(|l| col(l, " today")).collect();
    assert!(
        todays.windows(2).all(|w| w[0] == w[1]),
        "counts line up: {lines:#?}"
    );
    let lefts: Vec<_> = lines.iter().map(|l| col(l, " left of")).collect();
    assert!(
        lefts.windows(2).all(|w| w[0] == w[1]),
        "calls left line up: {lines:#?}"
    );
    // tui.log_rows
    let mut src = a.source.clone();
    src.log_rows = 3;
    let model = ownpurse_tui::model::Model::open(&src, a.now).unwrap();
    assert_eq!(model.activity.log.len(), 3);
}
