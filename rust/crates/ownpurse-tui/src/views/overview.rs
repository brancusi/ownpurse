//! Tab 1: Overview (spec §9, M1). One row per org: record age, **parity (year)** and **parity (history)**
//! (each names its basis), issues, Xero TB fetched, calls left; the intercompany line; Needs attention.
//! Enter on the org cell opens Ledger L1 for that org; Enter on a parity cell opens Ledger L1 with the `!`
//! filter on and that cell's basis (←/→ move between the cells). The model is prepared in `update`
//! (`books::OverviewModel`); draw only lays it out.

use crate::app::{App, OvCol, level_token};
use crate::books::{OverviewModel, Parity};
use crate::clock;
use crate::hit::{Hits, Target};
use crate::layout::{ColSpec, Kind};
use crate::model::Level;
use crate::theme_tokens::Token;
use crate::widgets::strip;
use crate::widgets::table::{self, Cell, Step, Table};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

pub const ORG: usize = 0;
pub const FETCHED: usize = 1;
pub const YEAR: usize = 2;
pub const HISTORY: usize = 3;
pub const ISSUES: usize = 4;
pub const TB_FETCHED: usize = 5;
pub const CALLS: usize = 6;

pub const COLS: [ColSpec; 7] = [
    ColSpec {
        title: "org",
        short_title: "",
        kind: Kind::Text { min: 4 },
        priority: 9,
    },
    ColSpec {
        title: "fetched",
        short_title: "",
        kind: Kind::Status,
        priority: 8,
    },
    ColSpec {
        title: "parity (year)",
        short_title: "year",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "parity (history)",
        short_title: "history",
        kind: Kind::Status,
        priority: 9,
    },
    ColSpec {
        title: "issues",
        short_title: "",
        kind: Kind::Status,
        priority: 5,
    },
    ColSpec {
        title: "TB fetched",
        short_title: "",
        kind: Kind::Id,
        priority: 4,
    },
    ColSpec {
        title: "calls left",
        short_title: "calls",
        kind: Kind::Amount,
        priority: 9,
    },
];

/// The declared degrade order (spec §4.3 Overview; connected/token arrive with the API crate):
/// TB fetched → issues shortens → parity (history) shortens → issues hides → then the compact forms.
pub const STEPS: [Step; 7] = [
    Step::Hide(TB_FETCHED),
    Step::Short(ISSUES),
    Step::Short(HISTORY),
    Step::Hide(ISSUES),
    Step::Short(FETCHED),
    Step::Short(YEAR),
    Step::Short(CALLS),
];

pub fn status(app: &App) -> String {
    // design pass 2: the footer carries no routine status; the bands and the focus line say it (with the
    // focus line off, what Enter does from a parity cell stays here)
    if app.source.view.focus_line {
        return String::new();
    }
    // What Enter does from a parity cell (M1).
    match app.ui.overview_col {
        OvCol::ParityYear => return "↵ Ledger · differs · year basis".into(),
        OvCol::ParityHistory => return "↵ Ledger · differs · full history".into(),
        OvCol::Org => {}
    }
    let synced = app
        .model
        .orgs
        .iter()
        .filter(|o| o.last_sync.is_some())
        .count();
    format!(
        "overview · {} orgs · {synced} synced · as of {}",
        app.model.orgs.len(),
        app.ui.as_of
    )
}

fn parity_cell(p: &Parity, quiet_ok: bool) -> Cell {
    let tok = match p {
        // a healthy ✓ stays grey (`tui.view.quiet_ok`, design pass 2 rule 1)
        Parity::Exact if quiet_ok => Token::Muted,
        Parity::Exact => Token::Ok,
        Parity::Differs(_) => Token::Warn,
        Parity::Unchecked => Token::Muted,
        Parity::Error(_) => Token::Error,
    };
    Cell::new(p.label(), tok).short(p.short())
}

/// The table rows (org indexes in `app.overview_rows()` order).
pub fn rows(app: &App, ov: &OverviewModel) -> Vec<Vec<Cell>> {
    app.overview_rows()
        .into_iter()
        .map(|i| {
            let o = &app.model.orgs[i];
            let f = app.model.freshness(&[i], app.now);
            let (_, ql) = app.model.quota(&[i]);
            let fresh_tok = match f.level {
                Level::Ok => Token::Text,
                l => level_token(l),
            };
            let r = ov.rows.iter().find(|r| r.org == i);
            let (year, history, issues, tb) = match r {
                Some(r) => (
                    parity_cell(&r.year, app.source.view.quiet_ok),
                    parity_cell(&r.history, app.source.view.quiet_ok),
                    Cell::new(
                        r.issues(),
                        if r.unbalanced + r.uncoded > 0 {
                            Token::Warn
                        } else {
                            Token::Muted
                        },
                    )
                    .short(r.issues_short()),
                    Cell::new(
                        r.tb_fetched.clone().unwrap_or_else(|| "—".into()),
                        Token::Muted,
                    ),
                ),
                None => (
                    parity_cell(&Parity::Unchecked, true),
                    parity_cell(&Parity::Unchecked, true),
                    Cell::new("—", Token::Muted),
                    Cell::new("—", Token::Muted),
                ),
            };
            vec![
                Cell::new(o.alias.clone(), Token::Text),
                Cell::new(
                    f.full().trim_start_matches("record ").to_string(),
                    fresh_tok,
                )
                .short(f.compact()),
                year,
                history,
                issues,
                tb,
                if o.left_known {
                    Cell::new(
                        format!("{}/{}", clock::int(o.left), clock::int(app.model.limit)),
                        level_token(ql),
                    )
                    .short(clock::int(o.left))
                } else {
                    // no day-remaining header from Xero today: the local estimate, marked as one
                    Cell::new(format!("{} (estimated)", o.left_fig()), level_token(ql))
                        .short(format!("{} (est.)", o.left_fig()))
                },
            ]
        })
        .collect()
}

/// The quiet second line under the selected org (design pass 2 tier 2): where Enter goes, and the org's
/// calls left (the budget's home on Overview now that its column is a detail).
pub fn focus_line(app: &App, ov: &OverviewModel) -> Option<String> {
    if !app.source.view.focus_line {
        return None;
    }
    let i = *app.overview_rows().get(app.overview_cursor()?)?;
    let o = &app.model.orgs[i];
    let left = if o.left_known {
        format!("{} calls left", clock::int(o.left))
    } else {
        format!("{} calls left (estimated)", o.left_fig())
    };
    let r = ov.rows.iter().find(|r| r.org == i);
    let (p, basis) = match app.ui.overview_col {
        OvCol::ParityYear => (r.map(|r| &r.year), "year basis"),
        OvCol::ParityHistory => (r.map(|r| &r.history), "full history"),
        OvCol::Org => (None, ""),
    };
    let go = match p {
        Some(Parity::Differs(1)) => {
            format!("Enter → Ledger, only the account that differs ({basis})")
        }
        Some(Parity::Differs(n)) => {
            format!("Enter → Ledger, only the {n} accounts that differ ({basis})")
        }
        Some(_) => format!("Enter → Ledger ({basis})"),
        None => format!("Enter → {} trial balance", o.alias),
    };
    Some(format!("{go} · {left}"))
}

/// The Organisations band's note: `all synced 2h ago`, or the oldest record's age when one is stale.
fn synced_note(app: &App) -> (String, Token) {
    let members = app.members(&app.ui.scope);
    let f = app.model.freshness(&members, app.now);
    let n = app.model.orgs.len();
    let synced = app
        .model
        .orgs
        .iter()
        .filter(|o| o.last_sync.is_some())
        .count();
    match f.level {
        Level::Ok if synced == n => (format!("all synced {} ago", f.age_token()), Token::Muted),
        Level::Ok => (format!("{synced} of {n} synced"), Token::Muted),
        l => (f.full(), level_token(l)),
    }
}

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let th = &app.ui.theme;
    let Some(ov) = app.overview.as_deref() else {
        let msg = app
            .books_error
            .clone()
            .unwrap_or_else(|| "Reading the record…".into());
        super::placeholder(app, buf, area, &msg);
        return;
    };
    let mut area = area;
    if let Some(line) = app.filter_chip() {
        let h = strip::height(&line, area.width).min(area.height);
        strip::draw(th, buf, Rect { height: h, ..area }, &line, Token::Muted);
        area.y += h;
        area.height -= h;
    }
    if area.height < 3 {
        return;
    }
    // the Organisations band, then the table under it
    crate::widgets::band(
        th,
        buf,
        Rect { height: 1, ..area },
        "Organisations",
        Some(synced_note(app)),
    );
    area.y += 1;
    area.height -= 1;
    let rows = rows(app, ov);
    // Lines below the table wrap (status words are never cut); the IC line breaks between its parts.
    let line_w = area.width.saturating_sub(4) as usize;
    // (spans, band label): a band row, or a content line at the 2-cell inset
    let mut below: Vec<(Vec<Span<'static>>, Option<&'static str>)> = Vec::new();
    if app.section_gap() > 0 {
        below.push((vec![], None));
    }
    if let Some(ic) = &ov.ic {
        below.push((vec![], Some("Intercompany")));
        let sep = "   ".to_string();
        let mut spans = Vec::new();
        let mut used = 0;
        for (i, part) in ic.parts.iter().enumerate() {
            let (tok, mark) = if part.ends_with("unexplained") && ic.unexplained > 0 {
                (Token::Error, "")
            } else if part.ends_with("unexplained") {
                (Token::Muted, " ✓")
            } else if part.ends_with("not compared") || part.ends_with("not supported") {
                (Token::Warn, "")
            } else {
                (Token::Text, "")
            };
            let text = th.text(&format!("{part}{mark}"));
            let pw = crate::layout::width(&text);
            if i > 0 {
                if used + sep.len() + pw > line_w {
                    below.push((std::mem::take(&mut spans), None));
                    used = 0;
                } else {
                    spans.push(Span::raw(sep.clone()));
                    used += sep.len();
                }
            }
            spans.push(Span::styled(text, th.fg(tok)));
            used += pw;
        }
        below.push((spans, None));
        if app.section_gap() > 0 {
            below.push((vec![], None));
        }
    }
    let mut warn_rows: Vec<usize> = Vec::new();
    below.push((vec![], Some("Needs attention")));
    if ov.attention.is_empty() {
        below.push((
            vec![Span::styled(
                "Nothing needs attention.",
                th.fg(Token::Muted),
            )],
            None,
        ));
    }
    for a in &ov.attention {
        for (k, l) in crate::layout::wrap(&th.text(a), line_w.saturating_sub(2))
            .into_iter()
            .enumerate()
        {
            let l = if k == 0 { l } else { format!("  {l}") };
            warn_rows.push(below.len());
            below.push((vec![Span::styled(l, th.fg(Token::Warn))], None));
        }
    }
    let focus = focus_line(app, ov);
    let below_h = below.len() as u16;
    let table_h = (rows.len() as u16 + 1 + u16::from(focus.is_some()))
        .min(area.height.saturating_sub(below_h).max(2))
        .min(area.height);
    let t_area = Rect {
        height: table_h,
        ..area
    };
    hits.content = Some(t_area);
    // Only a parity cell is marked (C12); on the org cell the row's ▌ gutter says it all.
    let cursor_col = match app.ui.overview_col {
        OvCol::Org => None,
        OvCol::ParityYear => Some(YEAR),
        OvCol::ParityHistory => Some(HISTORY),
    };
    // the details layer (`i`) adds the record times and the calls left
    let hidden: &[usize] = if app.ui.details {
        &[]
    } else {
        &[FETCHED, TB_FETCHED, CALLS]
    };
    let cursor = app.overview_cursor();
    let pos = table::draw(
        th,
        buf,
        t_area,
        &Table {
            cols: &COLS,
            rows: &rows,
            steps: &STEPS,
            top: app.ui.overview_top,
            cursor,
            cursor_col,
            footer: None,
            hidden,
            focus: focus.clone(),
            groups: &[],
        },
        hits,
    );
    let body = table_h.saturating_sub(1) as usize;
    let visible = rows.len().saturating_sub(app.ui.overview_top).min(body);
    let sel_drawn = cursor.and_then(|c| c.checked_sub(app.ui.overview_top));
    for r in 0..visible {
        // rows under the focus line sit one lower
        let shift = u16::from(focus.is_some() && sel_drawn.is_some_and(|s| r > s));
        let y = t_area.y + 1 + r as u16 + shift;
        if y >= t_area.bottom() {
            break;
        }
        for col in [ORG, YEAR, HISTORY] {
            if let Some((x, w)) = pos[col] {
                hits.add(Rect::new(x, y, w, 1), Target::Cell(r, col));
            }
        }
    }
    if rows.is_empty() {
        buf.set_span(
            area.x + 2,
            area.y + 1,
            &Span::styled("No org matches the filter.", th.fg(Token::Muted)),
            area.width.saturating_sub(2),
        );
    }
    let ell = th.glyphs().ellipsis;
    let warn_tint = (th.palette().knobs.status_warn > 0).then(|| th.bg(Token::StatusWarnRow));
    for (k, (y, (spans, band))) in (area.y + table_h..area.bottom()).zip(below).enumerate() {
        if let Some(bg) = warn_tint.filter(|_| warn_rows.contains(&k)) {
            crate::widgets::fill(buf, Rect::new(area.x, y, area.width, 1), bg);
        }
        match band {
            Some(label) => {
                crate::widgets::band(th, buf, Rect::new(area.x, y, area.width, 1), label, None)
            }
            None => {
                crate::widgets::put(buf, area.x + 2, y, spans, area.width.saturating_sub(4), ell);
            }
        }
    }
}
