//! The bridge: a horizontal waterfall. The first bar is the difference between the two sides' books; each
//! reconciling item takes its part away (a step bar from what was left before it to what is left after);
//! the last bar is what is left unexplained (0.00 when fully explained). Labels left, amounts right.

use super::money as fmt;
use super::{Body, OrgToken, Panel, fit, g, head, rtext, run, scale, t, text, wrapped};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ownpurse_core::viz::{Bridge, Step};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use rust_decimal::Decimal;

const AMT: u16 = 14;
/// The longest bar track.
const TRACK_MAX: u16 = 30;

pub fn draw(
    m: &Bridge,
    lead: &str,
    th: &Theme,
    org: OrgToken,
    buf: &mut Buffer,
    area: Rect,
    scroll: u16,
) -> Panel {
    let sub = [m.a_says.clone(), m.b_says.clone()];
    let (body, intro) = head(
        th,
        buf,
        area,
        &m.title,
        &[&[lead.to_string()][..], &sub[..]].concat(),
    );
    // the two sides in their own accents, at the head's right
    let tag = format!("{} ↔ {}", m.a_org, m.b_org);
    if area.width > 60 {
        let tw = super::width(&t(th, &tag)) as u16;
        // at the right edge, or (wide) a few cells after the title: never a wide blank between them
        let title_end = area.x + 2 + super::width(&t(th, &m.title)) as u16;
        let x = area.right().saturating_sub(tw + 2).min(title_end + 4);
        let a = t(th, &m.a_org);
        let n = text(buf, x, area.y, &a, tw, th.fg(org(&m.a_org)));
        let mid = t(th, " ↔ ");
        let n2 = text(buf, x + n, area.y, &mid, tw, th.fg(Token::Muted));
        text(
            buf,
            x + n + n2,
            area.y,
            &t(th, &m.b_org),
            tw,
            th.fg(org(&m.b_org)),
        );
    }
    let inner = Rect::new(
        body.x + 2,
        body.y,
        body.width.saturating_sub(4),
        body.height,
    );
    if let Some(r) = &m.reason {
        let why = wrapped(th, &format!("Not compared: {r}"), inner.width, Token::Warn);
        let mut b = Body::new(inner, scroll, (intro.len() + why.len()) as u16);
        b.prose(buf, th, inner.x, &intro);
        let rows = b.prose(buf, th, inner.x, &why);
        for r in rows {
            b.anchor("residual", r);
        }
        return b.done();
    }
    let bars = inner.width >= 90;
    let detail_w = inner.width.saturating_sub(AMT + 3);
    let details: Vec<Vec<(String, Token)>> = m
        .items
        .iter()
        .map(|it| wrapped(th, &it.detail, detail_w, Token::Muted))
        .collect();
    let ok = m.residual == Decimal::ZERO;
    let end_tok = if ok { Token::Ok } else { Token::Error };
    let verdict = if ok {
        format!(
            "✓ fully explained: {} = {}",
            fmt(m.start.amount),
            if m.items.is_empty() {
                "0.00".to_string()
            } else {
                m.items
                    .iter()
                    .map(|i| fmt(i.amount))
                    .collect::<Vec<_>>()
                    .join(" + ")
            }
        )
    } else {
        format!("{} left unexplained", fmt(m.residual))
    };
    let verdict = wrapped(th, &verdict, inner.width, end_tok);
    // rows: the intro, start, each item and its detail, the rule, the end, a blank, the verdict
    let total = intro.len()
        + 1
        + m.items.len()
        + details.iter().map(Vec::len).sum::<usize>()
        + 3
        + verdict.len();
    let mut b = Body::new(inner, scroll, total as u16);
    b.prose(buf, th, inner.x, &intro);
    let lw = if bars {
        (inner.width * 2 / 5).clamp(24, 44)
    } else {
        inner.width.saturating_sub(AMT + 1)
    };
    let track_x = inner.x + lw + 1;
    // the bars stay a readable length (the amounts follow them; wide screens leave the rest blank)
    let track_w = inner.width.saturating_sub(lw + 1 + AMT + 2).min(TRACK_MAX);
    let amt_right = if bars {
        (track_x + track_w + 2 + AMT).min(inner.right())
    } else {
        inner.right()
    };
    let gl = g(th);
    // the scale: every bar's ends
    let mut vals = vec![Decimal::ZERO, m.start.amount, m.residual];
    vals.extend(m.items.iter().map(|i| i.running));
    let lo = vals.iter().copied().min().unwrap_or_default();
    let hi = vals.iter().copied().max().unwrap_or_default();
    let mut prev = m.start.amount;
    let bar = |b: &mut Body,
               buf: &mut Buffer,
               step: &Step,
               from: Decimal,
               to: Decimal,
               label: String,
               ltok: Token,
               btok: Token,
               atok: Token| {
        let at = b.row();
        let rect = at.map(|y| Rect::new(inner.x, y, inner.width, 1));
        if let Ok(y) = at {
            text(
                buf,
                inner.x,
                y,
                &fit(th, &label, lw as usize),
                lw,
                th.fg(ltok),
            );
            if bars && track_w > 2 {
                let (c0, c1) = {
                    let a = scale(from.min(to), lo, hi, track_w - 1);
                    let z = scale(from.max(to), lo, hi, track_w - 1);
                    (a, z)
                };
                if c0 == c1 {
                    run(buf, track_x + c0, y, 1, gl.tick, th.fg(btok));
                } else {
                    run(buf, track_x + c0, y, c1 - c0 + 1, gl.bar, th.fg(btok));
                }
            }
            let a = if step.id == "residual" && step.amount == Decimal::ZERO {
                t(th, "✓ 0.00")
            } else {
                t(th, &fmt(step.amount))
            };
            rtext(buf, amt_right, y, &a, th.fg(atok));
        }
        b.anchor(&step.id, rect);
    };
    bar(
        &mut b,
        buf,
        &m.start,
        Decimal::ZERO,
        m.start.amount,
        m.start.label.clone(),
        Token::Text,
        Token::Text2,
        Token::Text,
    );
    for (i, it) in m.items.iter().enumerate() {
        bar(
            &mut b,
            buf,
            it,
            prev,
            it.running,
            format!("− {}", it.label),
            Token::Text,
            Token::Accent,
            Token::Text,
        );
        prev = it.running;
        for r in b.prose(buf, th, inner.x + 2, &details[i]) {
            b.anchor(&it.id, r);
        }
    }
    // the rule over the last amount (a sum line)
    if let Ok(y) = b.row() {
        run(
            buf,
            amt_right.saturating_sub(AMT),
            y,
            AMT,
            gl.h,
            th.fg(Token::Line),
        );
    }
    bar(
        &mut b,
        buf,
        &m.end,
        Decimal::ZERO,
        m.residual,
        m.end.label.clone(),
        Token::Text,
        end_tok,
        end_tok,
    );
    let _ = b.row();
    b.prose(buf, th, inner.x, &verdict);
    b.done()
}
