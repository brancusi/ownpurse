//! System tab (spec §9 row 5, M3; acceptance L1–L5) on the demo fixture, plus its snapshot grid
//! (S/M/L × 4 theme modes). Effects ($EDITOR, open, copy) are asserted, never run.

use ownpurse_tui::app::{App, Effect, update};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::snapshot::{
    self, demo_app, demo_app_variant, header_problems, row_text, run_script, styled, tab_state_app,
    tab_states, text, themes,
};
use ownpurse_tui::theme;
use ownpurse_tui::views::system::{self, Section};

fn at(script: &str, w: u16, h: u16) -> (App, String) {
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, script, w, h);
    let t = text(&snapshot::render(&mut app, w, h));
    (app, t)
}

/// Press one key and return the effects it produced.
fn press(app: &mut App, k: &str) -> Vec<Effect> {
    let fx = update(
        app,
        ownpurse_tui::app::Msg::Key(Key::parse(k).expect("key").event()),
    );
    snapshot::render(app, 100, 30);
    fx
}

#[test]
fn l1_sections_and_connection() {
    let (app, t) = at("5", 140, 40);
    for h in ["Connection", "Settings"] {
        assert!(t.lines().any(|l| l.trim() == h), "{h}\n{t}");
    }
    assert!(
        t.contains("token: managed by the CLI (`ownpurse login`) · token file tokens/demo.json")
    );
    assert!(t.contains("scopes: 15 read · 3 write configured (never requested by this TUI)"));
    for org in ["roast", "hold", "design", "lab"] {
        assert!(t.contains(org));
    }
    assert!(t.contains("Tallowmere Coffee Roasting Co.") && t.contains("STANDARD"));
    let lay = system::layout(&app);
    let items = &lay.0;
    let sections: Vec<Section> = items.iter().map(|i| i.section).collect();
    assert!(sections.contains(&Section::Links));
    // record counts and last sync come from the record
    let roast = &app.model.system.orgs[0];
    assert_eq!((roast.plan.as_str(), roast.records), ("STANDARD", 28));
    assert_eq!(roast.last_sync.as_deref(), Some("2026-10-05T07:46:00"));
    let (_, t) = at("5 end", 140, 40);
    assert!(t.lines().any(|l| l.trim().starts_with("Links")), "{t}");
}

#[test]
fn l1_never_synced_org() {
    let mut app = demo_app_variant(theme::LEDGER_DARK, false, "never");
    run_script(&mut app, "5", 140, 40);
    let t = text(&snapshot::render(&mut app, 140, 40));
    assert!(t.contains("never synced"), "{t}");
}

#[test]
fn l2_settings_with_sources() {
    let app = demo_app(theme::LEDGER_DARK, false);
    let s = &app.source.sys.settings;
    let get = |k: &str| s.iter().find(|(key, _, _)| key == k).cloned();
    assert_eq!(get("tui.log_rows").map(|r| r.2), Some("config".into()));
    assert_eq!(
        get("tui.refresh_seconds").map(|r| r.2),
        Some("defaults".into())
    );
    assert_eq!(
        get("groups.all-co.orgs").map(|r| r.1),
        Some("roast, hold, design, lab".into())
    );
    assert_eq!(
        get("profile.client_id").map(|r| r.2),
        Some("profile:demo".into())
    );
    assert!(get("profile.orgs.lab").is_some());
    // the profile's extra link reaches Links
    assert!(
        app.model
            .system
            .links
            .iter()
            .any(|l| l.name == "Demo runbook")
    );
}

#[test]
fn l3_as_of_default_shown() {
    let (_, t) = at("5 / a s _ o f enter", 140, 40);
    assert!(t.contains("tui.as_of"), "{t}");
    assert!(t.contains("(default: last FY end → 2025-12-31)"), "{t}");
}

#[test]
fn l4_links_per_org_and_bank_account() {
    let app = demo_app(theme::LEDGER_DARK, false);
    let names: Vec<&str> = app
        .model
        .system
        .links
        .iter()
        .map(|l| l.name.as_str())
        .collect();
    for want in [
        "Open Tallowmere Coffee Roasting Co. in Xero",
        "Tallowmere Coffee Roasting Co. · Tallowmere Roastery Operating Account: reconcile",
        "Saltmarsh Cupping Lab LLC · SALTMARSH SENSORY PANEL CARD™: statement lines",
        "OAuth callback (redirect URI)",
    ] {
        assert!(names.contains(&want), "{want} in {names:?}");
    }
    let open = app
        .model
        .system
        .links
        .iter()
        .find(|l| l.name == "Open Tallowmere Coffee Roasting Co. in Xero")
        .expect("org link");
    assert_eq!(
        open.url,
        "https://go.xero.com/OrganisationLogin/?shortCode=!TmR4q"
    );
    // per-document templates are not listed
    assert!(!names.iter().any(|n| n.contains("bank transaction in Xero")));
}

#[test]
fn l5_filter_spans_all_sections() {
    let (app, t) = at("5 / r o a s t enter", 140, 40);
    let lay = system::layout(&app);
    let items = &lay.0;
    let secs: std::collections::BTreeSet<String> =
        items.iter().map(|i| format!("{:?}", i.section)).collect();
    assert_eq!(secs.len(), 3, "{secs:?}");
    assert!(
        t.contains(&format!("filter: \"roast\" · {} of ", items.len())),
        "{t}"
    );
    let (_, t) = at("5 / z z z q enter", 100, 30);
    assert!(t.contains("Nothing matches the filter."));
}

#[test]
fn effects_open_copy_edit_are_produced_not_run() {
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, "5 / o p e n space t a l l o w enter", 100, 30);
    let url = "https://go.xero.com/OrganisationLogin/?shortCode=!TmR4q".to_string();
    assert_eq!(press(&mut app, "o"), vec![Effect::Open(url.clone())]);
    assert_eq!(press(&mut app, "enter"), vec![Effect::Open(url.clone())]);
    assert_eq!(press(&mut app, "y"), vec![Effect::Copy(url)]);
    // e only in Settings: on a link it does nothing; on a setting it opens the config file
    assert!(press(&mut app, "e").is_empty());
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, "5 j j j j", 100, 30);
    assert_eq!(
        system::selected(&app).map(|i| i.section),
        Some(Section::Settings)
    );
    match press(&mut app, "e").as_slice() {
        [Effect::Edit(p)] => assert!(p.ends_with("demo/ownpurse.json"), "{p:?}"),
        other => panic!("{other:?}"),
    }
    // o / y on a setting do nothing (keys belong to links)
    assert!(press(&mut app, "o").is_empty());
    // the footer names the section's keys
    let t = text(&snapshot::render(&mut app, 140, 40));
    let footer = t.lines().last().unwrap_or("");
    assert!(footer.contains("e Edit config"), "{footer}");
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, "5 end", 140, 40);
    let t = text(&snapshot::render(&mut app, 140, 40));
    let footer = t.lines().last().unwrap_or("");
    assert!(
        footer.contains("o Open") && footer.contains("y Copy"),
        "{footer}"
    );
}

#[test]
fn cursor_kept_by_key_and_clicks() {
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, "5 j j", 100, 30);
    assert_eq!(app.ui.system.sel.as_deref(), Some("org:design"));
    // a filter that keeps the row keeps the cursor on it
    run_script(&mut app, "/ d e s i g n enter", 100, 30);
    assert_eq!(
        system::selected(&app).map(|i| i.key),
        Some("org:design".into())
    );
    // clicking a row selects it; double-click on a link opens it
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, "5", 100, 30);
    let t = text(&snapshot::render(&mut app, 100, 30));
    let y = t
        .lines()
        .position(|l| l.contains("Saltmarsh Cupping Lab LLC"))
        .expect("row") as u16;
    run_script(&mut app, &format!("<click:5,{y}>"), 100, 30);
    assert_eq!(app.ui.system.sel.as_deref(), Some("org:lab"));
    // the wheel scrolls the list
    run_script(&mut app, "<wheel:down:3>", 100, 30);
    assert_eq!(app.ui.system.top, 3);
}

#[test]
fn system_snapshots() {
    let mut problems = Vec::new();
    let mut n = 0;
    for st in tab_states().iter().filter(|s| s.name.starts_with("system")) {
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
    assert_eq!(n, 6 * 4 * 3);
}

/// Spec §14: arriving on System reuses the formatted list and column fits (built once per load,
/// filter, width and minute).
#[test]
fn arrival_reuses_cached_models() {
    let mut app = demo_app(theme::LEDGER_DARK, false);
    run_script(&mut app, "5 1", 140, 40);
    let before = system::builds();
    run_script(&mut app, "5 1 5 1 5 j j k", 140, 40);
    assert_eq!(system::builds(), before, "tab arrival rebuilt a model");
    run_script(&mut app, "/ x enter", 140, 40);
    assert!(system::builds() > before, "a filter change rebuilds");
}

/// The scopes note is never cut: the long form from 100 columns, the short one below.
#[test]
fn scopes_line_never_cut() {
    for w in 60..=220u16 {
        let (_, t) = at("5", w, 30);
        let want = if w >= system::SCOPES_WIDE {
            "scopes: 15 read · 3 write configured (never requested by this TUI)"
        } else {
            "scopes: 15 read · 3 write (unused)"
        };
        let line = t.lines().find(|l| l.contains("scopes:")).unwrap_or("");
        assert_eq!(line.trim(), want, "{w}");
    }
}
