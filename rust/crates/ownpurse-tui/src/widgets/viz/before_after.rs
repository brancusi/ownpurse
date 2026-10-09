//! What a manifest changes in one org: the trial-balance lines that move, then the balance-sheet and P&L
//! totals, each before → after with the change (Δ) in the proposed colour.

use super::money as fmt;
use super::{Body, OrgToken, Panel, fit, head, rtext, t, text};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ownpurse_core::viz::{BaLine, BeforeAfter};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

const AMT: u16 = 14;

pub fn draw(
    m: &BeforeAfter,
    lead: &str,
    th: &Theme,
    _org: OrgToken,
    buf: &mut Buffer,
    area: Rect,
    scroll: u16,
) -> Panel {
    let sub = [format!(
        "at {} · {} entr{} · {} conflict{} · trial balance as net debit (credits in brackets)",
        m.as_of,
        m.entries,
        if m.entries == 1 { "y" } else { "ies" },
        m.conflicts,
        if m.conflicts == 1 { "" } else { "s" }
    )];
    let (body, intro) = head(
        th,
        buf,
        area,
        &m.title,
        &[&[lead.to_string()][..], &sub[..]].concat(),
    );
    let inner = Rect::new(
        body.x + 2,
        body.y,
        body.width.saturating_sub(4),
        body.height,
    );
    let bs: Vec<&BaLine> = m.totals.iter().filter(|l| l.section == "bs").collect();
    let pl: Vec<&BaLine> = m.totals.iter().filter(|l| l.section == "pl").collect();
    let groups: Vec<(&str, Vec<&BaLine>)> = vec![
        ("Trial balance lines that move", m.lines.iter().collect()),
        ("Balance sheet", bs),
        ("Profit and loss", pl),
    ];
    let total: u16 = groups
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(_, v)| v.len() as u16 + 2)
        .sum::<u16>()
        + 1;
    let mut b = Body::new(inner, scroll, total.max(2) + intro.len() as u16);
    b.prose(buf, th, inner.x, &intro);
    // three amount columns at 100+, two (after, Δ) when narrower
    let three = inner.width >= 3 * (AMT + 1) + 30;
    let cols = if three { 3 } else { 2 };
    // the labels a readable width, the amounts right after them (wide screens leave the rest blank)
    let lw = inner.width.saturating_sub(cols * (AMT + 1)).min(40);
    let right = inner.x + lw + cols * (AMT + 1);
    if let Ok(y) = b.row() {
        let hd = th.fg(Token::Muted);
        let mut x = right;
        rtext(buf, x, y, &t(th, "Δ change"), hd);
        x -= AMT + 1;
        rtext(buf, x, y, "after", hd);
        if three {
            x -= AMT + 1;
            rtext(buf, x, y, "before", hd);
        }
    }
    if m.lines.is_empty() && m.totals.is_empty() {
        if let Ok(y) = b.row() {
            text(
                buf,
                inner.x,
                y,
                &fit(th, "Nothing moves in this org.", inner.width as usize),
                inner.width,
                th.fg(Token::Muted),
            );
        }
        return b.done();
    }
    for (title, lines) in groups {
        if lines.is_empty() {
            continue;
        }
        if let Ok(y) = b.row() {
            text(
                buf,
                inner.x,
                y,
                &fit(th, title, lw as usize),
                lw,
                th.heading(),
            );
        }
        for l in lines {
            let at = b.row();
            if let Ok(y) = at {
                let label = if l.code.is_empty() {
                    l.label.clone()
                } else {
                    format!("{} {}", l.code, l.label)
                };
                text(
                    buf,
                    inner.x + 2,
                    y,
                    &fit(th, &label, lw.saturating_sub(2) as usize),
                    lw.saturating_sub(2),
                    th.fg(Token::Text),
                );
                let mut x = right;
                let d = super::signed(l.delta);
                rtext(buf, x, y, &t(th, &d), th.fg(Token::Proposed));
                x -= AMT + 1;
                rtext(buf, x, y, &t(th, &fmt(l.after)), th.fg(Token::Text));
                if three {
                    x -= AMT + 1;
                    rtext(buf, x, y, &t(th, &fmt(l.before)), th.fg(Token::Text2));
                }
            }
            b.anchor(&l.id, at.map(|y| Rect::new(inner.x, y, inner.width, 1)));
        }
        let _ = b.row();
    }
    b.done()
}
