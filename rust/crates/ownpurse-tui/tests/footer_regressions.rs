//! Footer regressions: a caption plus a long update notice (a dev build's `0.99.1+g6dc723f.dirty`) once
//! emptied the key hints down to the palette hint and then indexed below zero (a panic in the live TUI).

use ownpurse_tui::history::Caption;
use ownpurse_tui::snapshot::{self, demo_app, run_script};
use ownpurse_tui::theme;

#[test]
fn a_caption_and_a_long_update_notice_fit_without_a_panic() {
    for w in [60u16, 80, 100, 120, 140] {
        let mut a = demo_app(theme::LEDGER_DARK, false);
        run_script(&mut a, "2 ]", w, 30);
        a.update_available = Some(ownpurse_tui::canonical::Update {
            version: "0.99.1+g6dc723f.dirty".into(),
            action: ownpurse_tui::upgrade::ACTION.into(),
        });
        a.ui.caption = Some(Caption {
            text: "Card spend before the upgrade, with a long caption that fills the footer".into(),
            by: "agent".into(),
            rev: a.rev,
            fresh: false,
        });
        let b = snapshot::render(&mut a, w, 30);
        let last = snapshot::text(&b).lines().last().unwrap().to_string();
        assert!(last.contains("0.99.1"), "{w}: {last}");
    }
}
