//! The five tabs (spec §15): Overview · Ledger · Intercompany · Activity · System. In the shell, Overview
//! shows one row per org from the record; the others say they're coming in Phase 4.

pub mod activity;
pub mod catalog;
pub mod classify;
pub mod compare;
pub mod diff;
pub mod groups;
pub mod intercompany;
pub mod ledger;
pub mod manifest;
pub mod overview;
pub mod system;
pub mod viz;

use crate::app::{App, Tab};
use crate::hit::Hits;
use crate::theme_tokens::Token;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

/// Breadcrumb segments for the current view (left of row 3).
pub fn crumbs(app: &App) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(o) = app.ui.chain.last().filter(|o| o.to == app.ui.tab) {
        v.push(o.from.label().to_string());
    }
    match app.ui.tab {
        Tab::Overview => v.push("Overview".into()),
        Tab::Ledger if app.cmp_active() => v.extend(compare::crumbs(app)),
        Tab::Ledger => v.extend(ledger::crumbs(app)),
        Tab::Intercompany => v.extend(intercompany::crumbs(app)),
        Tab::Classify => v.extend(classify::crumbs(app)),
        Tab::Manifest => v.extend(manifest::crumbs(app)),
        t => {
            v.push(app.scope_label(&app.ui.scope));
            v.push(t.label().into());
        }
    }
    if app.ui.viz.is_some() {
        return viz::crumbs(app, v);
    }
    v
}

/// Cells the footer keeps for the status before dropping keys: all of an Enter hint (what the next key
/// does), the org and count under the differs filter (`design · 15 differ`), else 12.
pub fn status_keep(app: &App, status: &str) -> usize {
    // a history move says where it landed: all of it (keys drop first)
    if status.starts_with("back → ") || status.starts_with("forward → ") {
        return crate::layout::width(status);
    }
    // an upgrade note (deferred, failed, updated) or `Nothing to go back to`: all of it (keys drop first)
    if app.ui.note.as_deref().is_some_and(|n| {
        // `status` is already in the theme's glyphs (ASCII folds `·` to `-`)
        (app.ui.theme.text(n) == status
            || app.ui.theme.text(&crate::upgrade::narrow_note(n)) == status)
            && (crate::upgrade::is_note(n)
                || n.starts_with("Nothing to go ")
                // a Classify write's outcome (`accepted 3 labels · u undo`, `Not written: …`)
                || n.ends_with("· u undo")
                || n.starts_with("Not written: ")
                // what other processes changed (record, store, writes)
                || crate::external::is_note(n))
    }) {
        return crate::layout::width(status);
    }
    // the agent's notice: all of it while it shows (keys drop first; the palette hint stays)
    if let Some((t, _)) = app.agent_notice_text() {
        return crate::layout::width(&app.ui.theme.text(&t));
    }
    if let Some(r) = app.ui.run.as_ref().filter(|r| !r.band_visible) {
        return crate::layout::width(&app.ui.theme.text(&r.strip_text())) + 3 + 12;
    }
    if status.starts_with('↵') || status.starts_with("Enter") {
        40
    } else if app.ui.tab == Tab::Ledger && app.ui.differs_only && app.ledger_level() == 1 {
        status
            .split(" · ")
            .take(2)
            .map(crate::layout::width)
            .sum::<usize>()
            + 3
    } else {
        12
    }
}

/// The footer's left-hand line for the current view.
pub fn status(app: &App) -> String {
    if app.ui.viz.is_some() {
        return viz::status(app);
    }
    match app.ui.tab {
        Tab::Overview => overview::status(app),
        Tab::Ledger if app.cmp_active() => compare::status(app),
        Tab::Ledger => ledger::status(app),
        Tab::Intercompany => intercompany::status(app),
        Tab::Activity => activity::status(app),
        Tab::System => system::status(app),
        Tab::Classify => classify::status(app),
        Tab::Manifest => manifest::status(app),
    }
}

/// A one-line placeholder in the content area.
pub fn placeholder(app: &App, buf: &mut Buffer, area: Rect, text: &str) {
    if area.height == 0 {
        return;
    }
    let th = &app.ui.theme;
    let t = crate::layout::fit_str(
        &th.text(text),
        area.width.saturating_sub(2) as usize,
        th.glyphs().ellipsis,
    );
    buf.set_span(
        area.x + 1,
        area.y + 1.min(area.height - 1),
        &Span::styled(t, th.fg(Token::Muted)),
        area.width.saturating_sub(2),
    );
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    if app.ui.viz.is_some() {
        return viz::draw(app, buf, area, hits);
    }
    match app.ui.tab {
        Tab::Overview => overview::draw(app, buf, area, hits),
        Tab::Ledger => ledger::draw(app, buf, area, hits),
        Tab::Intercompany => intercompany::draw(app, buf, area, hits),
        Tab::Activity => activity::draw(app, buf, area, hits),
        Tab::System => system::draw(app, buf, area, hits),
        Tab::Classify => classify::draw(app, buf, area, hits),
        Tab::Manifest => manifest::draw(app, buf, area, hits),
    }
}

/// The short state at the breadcrumb's right (design pass 2: the provenance sentence's place when all is
/// well; `i` shows the sentence itself): `✓ matches Xero` on a healthy trial balance.
pub fn crumb_state(app: &App) -> Option<(String, Token)> {
    if app.ui.details || app.ui.viz.is_some() {
        return None;
    }
    match app.ui.tab {
        Tab::Ledger if !app.cmp_active() && app.ledger_level() == 1 => app
            .l1_model()
            .filter(|m| m.basis.ok())
            .map(|_| ("✓ matches Xero".to_string(), Token::Muted)),
        Tab::Intercompany if app.ic_level() == 2 => app.ic_pair().map(|p| {
            if intercompany::explained(p) {
                ("✓ fully explained".to_string(), Token::Muted)
            } else {
                (
                    format!("{} unexplained", intercompany::residual_text(p)),
                    Token::Error,
                )
            }
        }),
        _ => None,
    }
}

impl App {
    /// Blank rows between sections (`tui.view.density`: comfortable 1, compact 0).
    pub fn section_gap(&self) -> u16 {
        u16::from(self.source.view.density != "compact")
    }
}
