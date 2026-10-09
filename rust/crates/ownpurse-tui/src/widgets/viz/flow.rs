//! Money between the orgs, drawn like a sequence diagram: one box per org along the top (in the org's
//! accent), a lifeline under each, and one arrow per edge from who pays (or owes) to whom, the amount on
//! the arrow and the sentence under it. Too narrow for the boxes: a table (from → to, amount, words).

use super::money as fmt;
use super::{Body, OrgToken, Panel, fit, g, head, note, rtext, run, t, text, wrapped};
use crate::layout::width;
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ownpurse_core::viz::{Flow, FlowBasis};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;

/// Per edge: the sentence sits on the arrow; where the lines under it start; those lines.
type EdgePlan = (bool, u16, Vec<(String, Token)>);

pub fn draw(
    m: &Flow,
    lead: &str,
    th: &Theme,
    org: OrgToken,
    buf: &mut Buffer,
    area: Rect,
    scroll: u16,
) -> Panel {
    let sub = [match m.basis {
        FlowBasis::Owed => {
            "Each arrow points from the org that owes to the org that is owed (as the first org of each pair records it)."
                .to_string()
        }
        FlowBasis::Paid => {
            "Each arrow points from the org that paid, lent or charged to the other, over the period."
                .to_string()
        }
    }];
    let (body, intro) = head(
        th,
        buf,
        area,
        &m.title,
        &[&[lead.to_string()][..], &sub[..]].concat(),
    );
    let inner0 = Rect::new(
        body.x + 2,
        body.y,
        body.width.saturating_sub(4),
        body.height,
    );
    // the sentences stay above the boxes, wrapped, never cut
    let mut prose = vec![];
    for (i, (l, tok)) in intro.iter().enumerate() {
        let y = inner0.y + i as u16;
        if y >= inner0.bottom() {
            break;
        }
        let n = text(buf, inner0.x, y, l, width(l) as u16, th.fg(*tok));
        if n > 0 {
            prose.push(Rect::new(inner0.x, y, n, 1));
        }
    }
    let used = (intro.len() as u16).min(inner0.height);
    let inner = Rect::new(
        inner0.x,
        inner0.y + used,
        inner0.width,
        inner0.height - used,
    );
    let n = m.nodes.len() as u16;
    if n == 0 {
        note(th, buf, inner, "No orgs to show.", Token::Muted);
        return Panel::default();
    }
    let col = inner.width / n.max(1);
    let drawing = col >= 14 && inner.width >= 56;
    if !drawing {
        let mut p = table(m, th, org, buf, inner, scroll);
        p.prose.extend(prose);
        return p;
    }
    let gl = g(th);
    let centre = |i: usize| inner.x + col * i as u16 + col / 2;
    let pos = |a: &str| m.nodes.iter().position(|x| x.org == a);
    let mut panel = Panel {
        visible: inner.height,
        prose,
        ..Default::default()
    };
    // the boxes (3 rows), fixed at the top of the body
    let bw = col.saturating_sub(2).min(22);
    for (i, nd) in m.nodes.iter().enumerate() {
        let x = centre(i).saturating_sub(bw / 2);
        let r = Rect::new(x, inner.y, bw, 3.min(inner.height));
        if r.height < 3 {
            break;
        }
        let tok = org(&nd.org);
        let st = th.fg(tok);
        text(buf, x, r.y, gl.tl, 1, st);
        run(buf, x + 1, r.y, bw - 2, gl.h, st);
        text(buf, x + bw - 1, r.y, gl.tr, 1, st);
        text(buf, x, r.y + 1, gl.v, 1, st);
        let name = fit(th, &nd.org, (bw - 2) as usize);
        let nw = width(&name) as u16;
        text(
            buf,
            x + 1 + (bw - 2 - nw) / 2,
            r.y + 1,
            &name,
            nw,
            th.fg(tok).add_modifier(ratatui::style::Modifier::BOLD),
        );
        text(buf, x + bw - 1, r.y + 1, gl.v, 1, st);
        text(buf, x, r.y + 2, gl.bl, 1, st);
        run(buf, x + 1, r.y + 2, (bw - 2) / 2, gl.h, st);
        text(buf, x + 1 + (bw - 2) / 2, r.y + 2, gl.tee_down, 1, st);
        run(
            buf,
            x + 2 + (bw - 2) / 2,
            r.y + 2,
            (bw - 2) - (bw - 2) / 2 - 1,
            gl.h,
            st,
        );
        text(buf, x + bw - 1, r.y + 2, gl.br, 1, st);
        panel.put(&nd.id, r);
    }
    let lanes = Rect::new(
        inner.x,
        inner.y + 3,
        inner.width,
        inner.height.saturating_sub(3),
    );
    // each edge: the arrow row, then its sentences wrapped under it (from just left of its first lifeline)
    let plan: Vec<EdgePlan> = m
        .edges
        .iter()
        .map(|e| {
            let (Some(fi), Some(ti)) = (pos(&e.from), pos(&e.to)) else {
                return (false, inner.x, vec![]);
            };
            let (l, r) = (centre(fi).min(centre(ti)), centre(fi).max(centre(ti)));
            let on_arrow = width(&format!(" {} ", t(th, &e.words))) as u16 + 4 <= r - l;
            let mut parts = vec![];
            if !on_arrow {
                parts.push(e.words.clone());
            }
            if let Some(n) = &e.note {
                parts.push(n.clone());
            }
            let x = l.saturating_sub(2).max(inner.x);
            let lines = if parts.is_empty() {
                vec![]
            } else {
                wrapped(
                    th,
                    &parts.join(" · "),
                    inner.right().saturating_sub(x),
                    Token::Muted,
                )
            };
            (on_arrow, x, lines)
        })
        .collect();
    let total = m.edges.len() as u16
        + plan.iter().map(|p| p.2.len() as u16).sum::<u16>()
        + if m.edges.is_empty() { 1 } else { 0 };
    let mut b = Body::new(lanes, scroll, total);
    b.panel.anchors = panel.anchors;
    b.panel.prose = panel.prose;
    // the lifelines first (the arrows draw over them), as long as the edges they carry plus one row: never
    // empty lines down to the bottom of a tall screen
    let lane_end = lanes.y + lanes.height.min(total.saturating_sub(scroll) + 1);
    for i in 0..m.nodes.len() {
        for y in lanes.y..lane_end {
            text(buf, centre(i), y, gl.v, 1, th.fg(Token::Line));
        }
    }
    if m.edges.is_empty()
        && let Ok(y) = b.row()
    {
        text(
            buf,
            inner.x,
            y,
            &fit(
                th,
                "No intercompany pair between these orgs.",
                inner.width as usize,
            ),
            inner.width,
            th.fg(Token::Muted),
        );
    }
    for (k, e) in m.edges.iter().enumerate() {
        let (Some(fi), Some(ti)) = (pos(&e.from), pos(&e.to)) else {
            continue;
        };
        let at = b.row();
        let (on_arrow, sx, lines) = &plan[k];
        let (x0, x1) = (centre(fi), centre(ti));
        let (l, r) = (x0.min(x1), x0.max(x1));
        let right = x1 > x0;
        let tok = match e.status.as_str() {
            "unexplained" => Token::Error,
            "not compared" | "not supported" => Token::Warn,
            _ => Token::Text2,
        };
        let mut rect = at.map(|y| Rect::new(l, y, r - l + 1, 1));
        if let Ok(y) = at {
            run(buf, l + 1, y, (r - l).saturating_sub(1), gl.h, th.fg(tok));
            if right {
                text(buf, r - 1, y, gl.arrow_r, 1, th.fg(tok));
            } else {
                text(buf, l + 1, y, gl.arrow_l, 1, th.fg(tok));
            }
            text(buf, x0, y, gl.dot, 1, th.fg(org(&e.from)));
            let amt = e.amount.map_or("—".to_string(), fmt);
            // the sentence on the arrow when it fits ("roast owes hold 1,234.00"), else the amount
            let words = format!(" {} ", t(th, &e.words));
            let label = if *on_arrow {
                words
            } else {
                format!(" {} ", t(th, &amt))
            };
            let lw = width(&label) as u16;
            if lw + 4 <= r - l {
                let x = l + (r - l - lw) / 2;
                text(buf, x, y, &label, lw, th.fg(Token::Text));
            } else {
                // the amount beside the arrow when it is too short to hold it
                let x = r + 2;
                text(buf, x, y, &label, lw, th.fg(Token::Text));
                rect = Ok(Rect::new(l, y, r - l + 2 + lw, 1));
            }
        }
        b.anchor(&e.id, rect);
        for r in b.prose(buf, th, *sx, lines) {
            b.anchor(&e.id, r);
        }
    }
    b.done()
}

/// The narrow form: one row per edge.
fn table(m: &Flow, th: &Theme, org: OrgToken, buf: &mut Buffer, inner: Rect, scroll: u16) -> Panel {
    let words: Vec<Vec<(String, Token)>> = m
        .edges
        .iter()
        .map(|e| {
            let mut s = e.words.clone();
            if let Some(n) = &e.note {
                s.push_str(" · ");
                s.push_str(n);
            }
            wrapped(th, &s, inner.width.saturating_sub(2), Token::Muted)
        })
        .collect();
    let total = m.nodes.len() as u16
        + 1
        + m.edges.len() as u16
        + words.iter().map(|w| w.len() as u16).sum::<u16>();
    let mut b = Body::new(inner, scroll, total);
    for nd in &m.nodes {
        let at = b.row();
        if let Ok(y) = at {
            let s = format!("[{}] {}", nd.org, nd.name);
            text(
                buf,
                inner.x,
                y,
                &fit(th, &s, inner.width as usize),
                inner.width,
                th.fg(org(&nd.org)),
            );
        }
        b.anchor(&nd.id, at.map(|y| Rect::new(inner.x, y, inner.width, 1)));
    }
    let _ = b.row();
    for (k, e) in m.edges.iter().enumerate() {
        let at = b.row();
        if let Ok(y) = at {
            let s = format!("{} → {}", e.from, e.to);
            text(
                buf,
                inner.x,
                y,
                &fit(th, &s, inner.width.saturating_sub(16) as usize),
                inner.width.saturating_sub(16),
                th.fg(Token::Text),
            );
            let amt = e.amount.map_or("—".to_string(), fmt);
            rtext(buf, inner.right(), y, &t(th, &amt), th.fg(Token::Text));
        }
        b.anchor(&e.id, at.map(|y| Rect::new(inner.x, y, inner.width, 1)));
        for r in b.prose(buf, th, inner.x + 2, &words[k]) {
            b.anchor(&e.id, r);
        }
    }
    b.done()
}
