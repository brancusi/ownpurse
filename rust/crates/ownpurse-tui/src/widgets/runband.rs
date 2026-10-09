//! The run's bands in the confirm slot (spec P6 §4–§5): one row while a child runs (`Esc hide` on the
//! right), then the result (ok / warn / error, the first line plus up to 3 help rows). Neither takes
//! focus; the screen underneath keeps its keys.

use crate::layout::{fit_str, width};
use crate::run::{ResultBand, ResultLevel, RunState};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use crate::widgets::fill;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

pub fn draw_run(th: &Theme, r: &RunState, buf: &mut Buffer, area: Rect) {
    if area.height == 0 {
        return;
    }
    let raised = th.raised();
    fill(buf, area, raised);
    let hint = "Esc hide";
    let hw = width(hint) as u16;
    let room = area.width.saturating_sub(hw + 6) as usize;
    let text = fit_str(&th.text(&r.band_text()), room, th.glyphs().ellipsis);
    buf.set_span(
        area.x + 1,
        area.y,
        &Span::styled(text, th.fg(Token::Text).patch(raised)),
        room as u16,
    );
    buf.set_span(
        area.right().saturating_sub(hw + 1),
        area.y,
        &Span::styled(hint, th.fg(Token::Muted).patch(raised)),
        hw,
    );
}

fn token(l: ResultLevel) -> Token {
    match l {
        ResultLevel::Ok => Token::Ok,
        ResultLevel::Warn => Token::Warn,
        ResultLevel::Error => Token::Error,
    }
}

/// Wrap at ` · ` boundaries only, into at most `max` rows: whole segments are packed onto a
/// row while they fit; a later row starts with its segment; whatever is left joins the last row (cut there).
pub fn wrap_dots(text: &str, sep: &str, w: usize, max: usize) -> Vec<String> {
    let mut rows: Vec<String> = Vec::new();
    for seg in text.split(sep) {
        let full = rows.len() >= max;
        match rows.last_mut() {
            Some(row) if full || width(row) + width(sep) + width(seg) <= w => {
                row.push_str(sep);
                row.push_str(seg);
            }
            _ => rows.push(seg.to_string()),
        }
    }
    rows
}

/// The result's rows: the first line wrapped at ` · ` (at most 2 rows), then each help row cut to one
/// row (`y` copies the full text).
fn rows(th: &Theme, r: &ResultBand, cols: u16) -> Vec<(String, bool)> {
    let first_w = cols.saturating_sub(12) as usize;
    let mut out: Vec<(String, bool)> =
        wrap_dots(&th.text(&r.lines[0]), &th.text(" · "), first_w.max(10), 2)
            .into_iter()
            .map(|l| (l, true))
            .collect();
    out.extend(
        r.lines
            .iter()
            .skip(1)
            .take(3)
            .map(|l| (format!("  {}", th.text(l)), false)),
    );
    out
}

pub fn result_height(th: &Theme, r: &ResultBand, cols: u16) -> u16 {
    rows(th, r, cols).len().clamp(1, 5) as u16
}

pub fn draw_result(th: &Theme, r: &ResultBand, buf: &mut Buffer, area: Rect) {
    let raised = th.raised();
    fill(buf, area, raised);
    let hint = "y copy";
    let hw = width(hint) as u16;
    for (i, (text, first)) in rows(th, r, area.width)
        .into_iter()
        .take(area.height as usize)
        .enumerate()
    {
        let y = area.y + i as u16;
        let room = area.width.saturating_sub(if i == 0 { hw + 4 } else { 4 }) as usize;
        let tok = if first { token(r.level) } else { Token::Text };
        buf.set_span(
            area.x + 1,
            y,
            &Span::styled(
                fit_str(&text, room, th.glyphs().ellipsis),
                th.fg(tok).patch(raised),
            ),
            room as u16,
        );
    }
    buf.set_span(
        area.right().saturating_sub(hw + 1),
        area.y,
        &Span::styled(hint, th.fg(Token::Muted).patch(raised)),
        hw,
    );
}

#[cfg(test)]
mod tests {
    use super::wrap_dots;

    #[test]
    fn result_lines_wrap_only_between_segments() {
        let t = "✗ error: daily_limit · Xero daily call limit reached for this organisation. · record reloaded";
        assert_eq!(
            wrap_dots(t, " · ", 80, 2),
            [
                "✗ error: daily_limit · Xero daily call limit reached for this organisation.",
                "record reloaded"
            ]
        );
        // the second row takes whatever is left (the band cuts it there)
        assert_eq!(
            wrap_dots(t, " · ", 60, 2),
            [
                "✗ error: daily_limit",
                "Xero daily call limit reached for this organisation. · record reloaded"
            ]
        );
        assert_eq!(wrap_dots(t, " · ", 200, 2), [t]);
        assert_eq!(wrap_dots("a · b · c · d", " · ", 1, 2), ["a", "b · c · d"]);
    }
}
