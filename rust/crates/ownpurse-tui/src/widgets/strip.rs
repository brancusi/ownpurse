//! The strip under the breadcrumb (spec §3, V6): 1–3 rows that wrap and are never cut. It carries the basis
//! sentence when it needs a look (a healthy one is the breadcrumb's `✓ matches Xero`, design pass 2) and the
//! filter chip.

use crate::layout::wrap;
use crate::theme::Theme;
use crate::theme_tokens::Token;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::{Line, Span};

/// The content inset (design pass 2: two cells).
const INSET: u16 = 2;

/// Rows the strip needs at this width (at most 3).
pub fn height(text: &str, width: u16) -> u16 {
    if text.is_empty() {
        0
    } else if text.contains('\t') {
        1
    } else {
        wrap(text, width.saturating_sub(2 * INSET) as usize)
            .len()
            .min(3) as u16
    }
}

/// The filter chip's pieces: `filter:` and the count in `text.3`, the quoted query in `text.1` (the v0.10.1
/// review: no lavender, no new hue).
fn chip_spans(th: &Theme, line: &str) -> Option<Vec<Span<'static>>> {
    let rest = line.strip_prefix("filter: \"")?;
    let end = rest.rfind("\" ")?;
    let (q, tail) = rest.split_at(end);
    Some(vec![
        Span::styled("filter: ", th.fg(Token::Muted)),
        Span::styled(format!("\"{q}\""), th.fg(Token::Text)),
        Span::styled(tail[1..].to_string(), th.fg(Token::Muted)),
    ])
}

pub fn draw(th: &Theme, buf: &mut Buffer, area: Rect, text: &str, token: Token) {
    // `left\tright`: one line, the right part ending at the right inset (a short state)
    if let Some((l, r)) = text.split_once('\t') {
        if area.height == 0 {
            return;
        }
        let (l, r) = (th.text(l), th.text(r));
        let rw = crate::layout::width(&r) as u16;
        let room = area.width.saturating_sub(2 * INSET + rw + 2);
        let l = crate::layout::fit_str(&l, room as usize, th.glyphs().ellipsis);
        buf.set_span(area.x + INSET, area.y, &Span::styled(l, th.fg(token)), room);
        if rw + 2 * INSET < area.width {
            buf.set_span(
                area.right() - INSET - rw,
                area.y,
                &Span::styled(r, th.fg(token)),
                rw,
            );
        }
        return;
    }
    let text = th.text(text);
    for (i, line) in wrap(&text, area.width.saturating_sub(2 * INSET) as usize)
        .into_iter()
        .take(area.height as usize)
        .enumerate()
    {
        let spans = chip_spans(th, &line).unwrap_or_else(|| vec![Span::styled(line, th.fg(token))]);
        buf.set_line(
            area.x + INSET,
            area.y + i as u16,
            &Line::from(spans),
            area.width.saturating_sub(INSET),
        );
    }
}
