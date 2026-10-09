//! The demo fixture (rust/tests/fixtures/demo) holds the facts the acceptance checks rely on.

use ownpurse_tui::model::Level;
use ownpurse_tui::snapshot::{DEMO_VARIANTS, demo_app_variant};
use ownpurse_tui::theme;

fn left(variant: &str) -> Vec<(String, i64, Option<String>)> {
    let app = demo_app_variant(theme::LEDGER_DARK, false, variant);
    app.model
        .orgs
        .iter()
        .map(|o| (o.alias.clone(), o.left, o.last_sync.clone()))
        .collect()
}

#[test]
fn fresh_calls_left_and_syncs() {
    let rows = left("fresh");
    let got: Vec<(&str, i64)> = rows.iter().map(|(a, l, _)| (a.as_str(), *l)).collect();
    assert_eq!(
        got,
        [("roast", 967), ("hold", 978), ("design", 938), ("lab", 978)]
    );
    assert!(
        rows.iter()
            .all(|(_, _, s)| s.as_deref() == Some("2026-10-05T07:46:00"))
    );
    let app = demo_app_variant(theme::LEDGER_DARK, false, "fresh");
    assert_eq!(app.header_facts().left, 938);
    assert_eq!(app.model.groups[0].members.len(), 4);
}

#[test]
fn variants_hold_their_state() {
    for v in DEMO_VARIANTS {
        let app = demo_app_variant(theme::LEDGER_DARK, false, v);
        let lab = app.model.freshness(&[3], app.now);
        let (owner_left, owner_level) = app.model.quota(&[2]);
        match v {
            "fresh" => assert_eq!(lab.level, Level::Ok),
            "stale" => assert_eq!(lab.level, Level::Warn),
            "never" => assert_eq!(lab.at, None),
            "lowbudget" => assert_eq!((owner_left, owner_level), (62, Level::Err)),
            "noclose" => assert_eq!(lab.level, Level::Ok),
            _ => unreachable!(),
        }
    }
}
