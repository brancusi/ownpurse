//! The confirm band (row H-2, spec §3, §9, V3): a docked band on `raised`, not a box. Ask: the question
//! with `[Cancel] [<action>]`, focus on Cancel. Refusal (budget, offline, the pre-API placeholder):
//! error colour, persistent, one focused `Close`. The focused button is REVERSED and carries the cursor
//! glyph, so focus reads without colour (V12).

use crate::app::{Band, BandKind};
use crate::hit::{Hits, Target};
use crate::layout::{width, wrap};
use crate::theme::Theme;
use crate::theme_tokens::Token;
use crate::widgets::fill;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

fn button_text(th: &Theme, label: &str, focused: bool) -> String {
    let g = th.glyphs();
    if focused {
        format!("{}{label} ", g.cursor)
    } else {
        format!(" {label} ")
    }
}

fn buttons_width(th: &Theme, band: &Band) -> usize {
    band.buttons()
        .iter()
        .enumerate()
        .map(|(i, b)| width(&button_text(th, b, i == band.focus)) + 1)
        .sum()
}

fn lines(th: &Theme, band: &Band, cols: u16) -> Vec<String> {
    let text_w = (cols as usize)
        .saturating_sub(2 + buttons_width(th, band) + 2)
        .max(10);
    wrap(&th.text(&band.text), text_w)
}

/// Rows the band takes at this width (it wraps; never cut), at most 3.
pub fn height(th: &Theme, band: &Band, cols: u16) -> u16 {
    lines(th, band, cols).len().clamp(1, 3) as u16
}

pub fn draw(th: &Theme, band: &Band, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let raised = th.raised();
    fill(buf, area, raised);
    let tok = match band.kind {
        BandKind::Refusal => Token::Error,
        BandKind::Ask { .. } => Token::Text,
    };
    let ls = lines(th, band, area.width);
    for (i, l) in ls.iter().take(area.height as usize).enumerate() {
        buf.set_span(
            area.x + 1,
            area.y + i as u16,
            &Span::styled(l.clone(), th.fg(tok).patch(raised)),
            area.width.saturating_sub(2),
        );
    }
    let y = area.bottom() - 1;
    let mut x = area
        .right()
        .saturating_sub(1 + buttons_width(th, band) as u16);
    for (i, b) in band.buttons().iter().enumerate() {
        let focused = i == band.focus;
        let t = button_text(th, b, focused);
        let w = width(&t) as u16;
        let style = if focused {
            th.focused_button()
        } else {
            th.fg(Token::Muted).patch(raised)
        };
        buf.set_span(x, y, &Span::styled(t, style), w);
        hits.add(Rect::new(x, y, w, 1), Target::Button(i));
        x += w + 1;
    }
}
