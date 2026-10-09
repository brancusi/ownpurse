//! The close worklist: groups (Deadlines, Waiting on <owner>, Agent work, Postings by state), each item a
//! line with a status glyph and the status word from the file, verbatim. Glyph and word always go together
//! (never colour alone).

use super::{Body, OrgToken, Panel, fit, head, text, wrapped};
use crate::layout::width;
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ownpurse_core::viz::{Checklist, Tone};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

fn mark(th: &Theme, tone: Tone) -> (&'static str, Token) {
    match (tone, th.ascii) {
        (Tone::Done, false) => ("✓", Token::Ok),
        (Tone::Done, true) => ("[x]", Token::Ok),
        (Tone::Open, false) => ("○", Token::Text2),
        (Tone::Open, true) => ("[ ]", Token::Text2),
        (Tone::Blocked, false) => ("■", Token::Error),
        (Tone::Blocked, true) => ("[#]", Token::Error),
        (Tone::Late, false) => ("!", Token::Error),
        (Tone::Late, true) => ("[!]", Token::Error),
        (Tone::Soon, false) => ("◔", Token::Warn),
        (Tone::Soon, true) => ("[~]", Token::Warn),
        (Tone::Unknown, false) => ("?", Token::Muted),
        (Tone::Unknown, true) => ("[?]", Token::Muted),
    }
}

pub fn draw(
    m: &Checklist,
    lead: &str,
    th: &Theme,
    _org: OrgToken,
    buf: &mut Buffer,
    area: Rect,
    scroll: u16,
) -> Panel {
    let counts = m
        .counts
        .iter()
        .map(|(k, v)| format!("{v} {k}"))
        .collect::<Vec<_>>()
        .join(" · ");
    let mut sub = vec![format!("{} · today {} · {counts}", m.file, m.today)];
    if !m.also.is_empty() {
        sub.push(format!("also in the file: {}", m.also.join(", ")));
    }
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
    let status_w = m
        .groups
        .iter()
        .flat_map(|g| g.items.iter())
        .map(|i| width(&i.status))
        .max()
        .unwrap_or(0)
        .min(18) as u16;
    let key_w = m
        .groups
        .iter()
        .flat_map(|g| g.items.iter())
        .map(|i| width(&i.key))
        .max()
        .unwrap_or(0)
        .min(10) as u16;
    let glyph_w = if th.ascii { 3 } else { 1 };
    let tx = inner.x + 2 + glyph_w + 1 + status_w + 2 + key_w + 2;
    let tw = inner.right().saturating_sub(tx).max(10);
    // each item's text (and its detail, muted) wrapped in the text column, never cut
    let lines: Vec<Vec<Vec<(String, Token)>>> = m
        .groups
        .iter()
        .map(|g| {
            g.items
                .iter()
                .map(|i| {
                    let mut v = wrapped(th, &i.text, tw, Token::Text);
                    if let Some(d) = &i.detail {
                        v.extend(wrapped(th, d, tw, Token::Muted));
                    }
                    v
                })
                .collect()
        })
        .collect();
    let total: usize = lines
        .iter()
        .map(|g| g.iter().map(Vec::len).sum::<usize>() + 2)
        .sum();
    let mut b = Body::new(inner, scroll, (total + intro.len()) as u16);
    b.prose(buf, th, inner.x, &intro);
    for (gi, g) in m.groups.iter().enumerate() {
        let at = b.row();
        if let Ok(y) = at {
            let s = format!("{} ({})", g.title, g.items.len());
            text(
                buf,
                inner.x,
                y,
                &fit(th, &s, inner.width as usize),
                inner.width,
                th.heading(),
            );
        }
        b.anchor(&g.id, at.map(|y| Rect::new(inner.x, y, inner.width, 1)));
        for (ii, i) in g.items.iter().enumerate() {
            let ls = &lines[gi][ii];
            let first = b.prose(buf, th, tx, &ls[..1.min(ls.len())]);
            if let Some(Ok(r)) = first.first() {
                let y = r.y;
                let (glyph, tok) = mark(th, i.tone);
                let mut x = inner.x + 2;
                text(buf, x, y, glyph, glyph_w, th.fg(tok));
                x += glyph_w + 1;
                let st = fit(th, &i.status, status_w as usize);
                text(buf, x, y, &st, status_w, th.fg(tok));
                x += status_w + 2;
                text(
                    buf,
                    x,
                    y,
                    &fit(th, &i.key, key_w as usize),
                    key_w,
                    th.fg(Token::Muted),
                );
            }
            for r in first {
                b.anchor(&i.id, r.map(|r| Rect::new(inner.x, r.y, inner.width, 1)));
            }
            for r in b.prose(buf, th, tx, &ls[1.min(ls.len())..]) {
                b.anchor(&i.id, r);
            }
        }
        let _ = b.row();
    }
    b.done()
}
