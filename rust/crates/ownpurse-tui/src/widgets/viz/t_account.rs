//! The classic T: debits on the left, credits on the right, the opening balance on the side it sits, the
//! largest postings each side (the rest summed as "N more"), the totals, and the closing balance under the
//! T. Narrow areas get the two sides stacked as one table.

use super::money as fmt;
use super::{Body, OrgToken, Panel, fit, g, head, rtext, run, t, text, wrapped};
use crate::layout::width;
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ownpurse_core::dr_cr;
use ownpurse_core::viz::{TAccount, TEntry};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Modifier;
use rust_decimal::Decimal;

const AMT: u16 = 13;

pub fn draw(
    m: &TAccount,
    lead: &str,
    th: &Theme,
    org: OrgToken,
    buf: &mut Buffer,
    area: Rect,
    scroll: u16,
) -> Panel {
    let sub = [format!(
        "{} to {} · opening {} · {} debit{} and {} credit{}",
        m.from,
        m.to,
        m.opening_source,
        m.debit_count,
        if m.debit_count == 1 { "" } else { "s" },
        m.credit_count,
        if m.credit_count == 1 { "" } else { "s" },
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
    if inner.width < 64 {
        return stacked(m, th, buf, inner, scroll, &intro);
    }
    let gl = g(th);
    let acc = th.fg(org(&m.org));
    let half = inner.width / 2;
    let lx = inner.x;
    let lw = half.saturating_sub(1);
    let mid = inner.x + half;
    let rx = mid + 2;
    let rw = inner.right().saturating_sub(rx);
    let side = |list: &Vec<TEntry>, more: &Option<TEntry>| list.len() + usize::from(more.is_some());
    let rows = 1 + side(&m.debits, &m.debits_more).max(side(&m.credits, &m.credits_more));
    // head row, rule, rows, rule, totals, blank, closing
    let total = 2 + rows as u16 + 4;
    let closing = wrapped(
        th,
        &format!(
            "Closing balance at {}: {}  ({})",
            m.to,
            dr_cr(m.closing),
            if m.closing > Decimal::ZERO {
                "the debit side is bigger"
            } else if m.closing < Decimal::ZERO {
                "the credit side is bigger"
            } else {
                "the two sides are equal"
            }
        ),
        inner.width,
        Token::Text,
    );
    let mut b = Body::new(
        inner,
        scroll,
        total + intro.len() as u16 + closing.len() as u16 - 1,
    );
    b.prose(buf, th, inner.x, &intro);
    // the T's top: the side names and the heavy bar
    if let Ok(y) = b.row() {
        let dl = t(th, "Debit");
        let cl = t(th, "Credit");
        // the side names meet at the stem of the T (`Debit │ Credit`), as accountants draw it
        let dw = (super::width(&dl) as u16).min(lw);
        text(
            buf,
            lx + lw - dw,
            y,
            &fit(th, &dl, lw as usize),
            lw,
            acc.add_modifier(Modifier::BOLD),
        );
        text(
            buf,
            rx,
            y,
            &fit(th, &cl, rw as usize),
            rw,
            acc.add_modifier(Modifier::BOLD),
        );
    }
    if let Ok(y) = b.row() {
        run(buf, lx, y, inner.width, gl.hh, acc);
        text(buf, mid, y, gl.tee_down, 1, acc);
    }
    let line = |buf: &mut Buffer, x: u16, w: u16, y: u16, label: &str, amt: &str, ltok: Token| {
        let lab_w = w.saturating_sub(AMT + 1);
        text(
            buf,
            x,
            y,
            &fit(th, label, lab_w as usize),
            lab_w,
            th.fg(ltok),
        );
        rtext(buf, x + w, y, &t(th, amt), th.fg(Token::Text));
    };
    let entry_label = |e: &TEntry| {
        if e.date.is_empty() {
            e.label.clone()
        } else {
            format!("{} {}", &e.date[5..], e.label)
        }
    };
    let mut left: Vec<(String, String, Token, String)> = vec![];
    let mut right: Vec<(String, String, Token, String)> = vec![];
    let opening = (
        "Opening balance".to_string(),
        fmt(m.opening.abs()),
        Token::Text2,
        "opening".to_string(),
    );
    if m.opening >= Decimal::ZERO {
        left.push(opening);
    } else {
        right.push(opening);
    }
    for e in m.debits.iter().chain(m.debits_more.iter()) {
        let tok = if e.count > 1 {
            Token::Muted
        } else {
            Token::Text
        };
        left.push((entry_label(e), fmt(e.amount), tok, e.id.clone()));
    }
    for e in m.credits.iter().chain(m.credits_more.iter()) {
        let tok = if e.count > 1 {
            Token::Muted
        } else {
            Token::Text
        };
        right.push((entry_label(e), fmt(e.amount), tok, e.id.clone()));
    }
    let n = left.len().max(right.len());
    for i in 0..n {
        let at = b.row();
        if let Ok(y) = at {
            text(buf, mid, y, gl.v, 1, acc);
            if let Some((l, a, tok, _)) = left.get(i) {
                line(buf, lx, lw, y, l, a, *tok);
            }
            if let Some((l, a, tok, _)) = right.get(i) {
                line(buf, rx, rw, y, l, a, *tok);
            }
        }
        if let Some((_, _, _, id)) = left.get(i) {
            b.anchor(id, at.map(|y| Rect::new(lx, y, lw, 1)));
        }
        if let Some((_, _, _, id)) = right.get(i) {
            b.anchor(id, at.map(|y| Rect::new(rx, y, rw, 1)));
        }
    }
    if let Ok(y) = b.row() {
        run(buf, lx, y, inner.width, gl.h, th.fg(Token::Line));
        text(buf, mid, y, gl.cross, 1, acc);
    }
    if let Ok(y) = b.row() {
        text(buf, mid, y, gl.v, 1, acc);
        let dt = m.debit_total + m.opening.max(Decimal::ZERO);
        let ct = m.credit_total + (-m.opening).max(Decimal::ZERO);
        line(buf, lx, lw, y, "Total", &fmt(dt), Token::Text2);
        line(buf, rx, rw, y, "Total", &fmt(ct), Token::Text2);
    }
    let _ = b.row();
    let cx = if closing.len() == 1 {
        let w = width(&closing[0].0) as u16;
        inner.x + inner.width.saturating_sub(w) / 2
    } else {
        inner.x
    };
    for r in b.prose(buf, th, cx, &closing) {
        b.anchor("closing", r);
    }
    b.done()
}

/// The narrow form: opening, debits, credits, closing as one list.
fn stacked(
    m: &TAccount,
    th: &Theme,
    buf: &mut Buffer,
    inner: Rect,
    scroll: u16,
    intro: &[(String, Token)],
) -> Panel {
    let total = 4
        + (m.debits.len() + m.credits.len()) as u16
        + u16::from(m.debits_more.is_some())
        + u16::from(m.credits_more.is_some())
        + 2;
    let mut b = Body::new(inner, scroll, total + intro.len() as u16);
    b.prose(buf, th, inner.x, intro);
    let lw = inner.width.saturating_sub(AMT + 1);
    let row =
        |b: &mut Body, buf: &mut Buffer, id: Option<&str>, label: &str, amt: &str, tok: Token| {
            let at = b.row();
            if let Ok(y) = at {
                text(
                    buf,
                    inner.x,
                    y,
                    &fit(th, label, lw as usize),
                    lw,
                    th.fg(tok),
                );
                rtext(buf, inner.right(), y, &t(th, amt), th.fg(Token::Text));
            }
            if let Some(id) = id {
                b.anchor(id, at.map(|y| Rect::new(inner.x, y, inner.width, 1)));
            }
        };
    row(
        &mut b,
        buf,
        Some("opening"),
        "Opening balance",
        &dr_cr(m.opening),
        Token::Text2,
    );
    row(
        &mut b,
        buf,
        None,
        &format!("Debits ({})", m.debit_count),
        &fmt(m.debit_total),
        Token::Text2,
    );
    for e in m.debits.iter().chain(m.debits_more.iter()) {
        row(
            &mut b,
            buf,
            Some(&e.id),
            &format!("  {} {}", e.date, e.label),
            &fmt(e.amount),
            Token::Text,
        );
    }
    row(
        &mut b,
        buf,
        None,
        &format!("Credits ({})", m.credit_count),
        &fmt(m.credit_total),
        Token::Text2,
    );
    for e in m.credits.iter().chain(m.credits_more.iter()) {
        row(
            &mut b,
            buf,
            Some(&e.id),
            &format!("  {} {}", e.date, e.label),
            &fmt(e.amount),
            Token::Text,
        );
    }
    let _ = b.row();
    row(
        &mut b,
        buf,
        Some("closing"),
        &format!("Closing balance at {}", m.to),
        &dr_cr(m.closing),
        Token::Text,
    );
    b.done()
}
