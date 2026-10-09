//! One balance over time: a row per month with a bar for the month-end balance (debit balances grow right
//! of the zero line, credit balances left of it), the balance, the month's movement, and a mark where a
//! manual journal landed. Narrow areas drop the bars (the table stays).

use super::{Body, OrgToken, Panel, fit, g, head, rtext, run, scale, t, text};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ownpurse_core::dr_cr;
use ownpurse_core::viz::Timeline;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use rust_decimal::Decimal;

const BAL: u16 = 15;
const MOV: u16 = 13;
const MARK: u16 = 11;

pub fn draw(
    m: &Timeline,
    lead: &str,
    th: &Theme,
    org: OrgToken,
    buf: &mut Buffer,
    area: Rect,
    scroll: u16,
) -> Panel {
    let journals: usize = m.months.iter().map(|x| x.journals).sum();
    let sub = [format!(
        "Opening {} on {} · bars: debit balance right of the line, credit left · ◆ a manual journal landed ({journals} in all)",
        dr_cr(m.opening),
        m.from
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
    let gl = g(th);
    let total = 1 + m.months.len() as u16;
    let mut b = Body::new(inner, scroll, total + intro.len() as u16);
    b.prose(buf, th, inner.x, &intro);
    let label_w = 8u16;
    let fixed = label_w + 1 + BAL + 1 + MOV + 1 + MARK;
    let bars = inner.width >= fixed + 20;
    // the bars stay a readable length, the columns follow them (wide screens leave the rest blank)
    let track_w = inner.width.saturating_sub(fixed + 1).min(26);
    let track_x = inner.x + label_w + 1;
    let cols_x = if bars {
        track_x + track_w + 1
    } else {
        inner.right().saturating_sub(BAL + 1 + MOV + 1 + MARK)
    };
    let mut vals: Vec<Decimal> = m.months.iter().map(|x| x.balance).collect();
    vals.push(Decimal::ZERO);
    let lo = vals.iter().copied().min().unwrap_or_default();
    let hi = vals.iter().copied().max().unwrap_or_default();
    let zero = scale(Decimal::ZERO, lo, hi, track_w.saturating_sub(1));
    if let Ok(y) = b.row() {
        let hd = th.fg(Token::Muted);
        text(buf, inner.x, y, "month", label_w, hd);
        rtext(buf, cols_x + BAL, y, "balance", hd);
        rtext(buf, cols_x + BAL + 1 + MOV, y, "movement", hd);
        text(
            buf,
            cols_x + BAL + 1 + MOV + 1,
            y,
            &t(th, "entries"),
            MARK,
            hd,
        );
    }
    let acc = th.fg(org(&m.org));
    for mo in &m.months {
        let at = b.row();
        if let Ok(y) = at {
            text(buf, inner.x, y, &mo.month, label_w, th.fg(Token::Text));
            if bars && track_w > 2 {
                let c = scale(mo.balance, lo, hi, track_w - 1);
                text(buf, track_x + zero, y, gl.tick, 1, th.fg(Token::Line));
                let (a, z) = (c.min(zero), c.max(zero));
                if a != z {
                    let tok = if mo.balance >= Decimal::ZERO {
                        acc
                    } else {
                        th.fg(Token::Text2)
                    };
                    run(buf, track_x + a, y, z - a + 1, gl.bar, tok);
                }
            }
            rtext(
                buf,
                cols_x + BAL,
                y,
                &t(th, &dr_cr(mo.balance)),
                th.fg(Token::Text),
            );
            let mv = if mo.postings == 0 {
                "—".to_string()
            } else {
                super::signed(mo.movement)
            };
            rtext(
                buf,
                cols_x + BAL + 1 + MOV,
                y,
                &t(th, &mv),
                th.fg(Token::Text2),
            );
            let mut mark = if mo.postings > 0 {
                format!("{}", mo.postings)
            } else {
                String::new()
            };
            if mo.journals > 0 {
                mark.push_str(if mo.journals == 1 {
                    " ◆ journal"
                } else {
                    " ◆ journals"
                });
            }
            text(
                buf,
                cols_x + BAL + 1 + MOV + 1,
                y,
                &fit(th, &mark, MARK as usize),
                MARK,
                th.fg(if mo.journals > 0 {
                    Token::Accent
                } else {
                    Token::Muted
                }),
            );
        }
        b.anchor(&mo.id, at.map(|y| Rect::new(inner.x, y, inner.width, 1)));
    }
    b.done()
}
