//! The palette (`⌃P` / `:`, spec §6, §9): every `palette: true` action with its current keys. Network
//! actions are marked `⇅`; while a run is going they are disabled with `· running: …`;
//! under `--offline` they're hidden (M4).

use crate::app::{App, Overlay};
use crate::hit::{Hits, Target};
use crate::layout::{fit_str, width};
use crate::theme_tokens::Token;
use crate::widgets::centred;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let Some(Overlay::Palette { query, sel }) = &app.ui.overlay else {
        return;
    };
    let th = &app.ui.theme;
    let g = th.glyphs();
    let entries = app.palette_filtered(query);
    let box_w = area.width.saturating_sub(4).min(76);
    let box_h = (entries.len().max(1) + 4) as u16;
    let rect = centred(
        area,
        box_w,
        box_h.min(area.height.saturating_sub(6)),
        Some(3),
    );
    let inner = crate::widgets::overlay_box_clear(buf, rect, "Commands", th);
    hits.overlay = Some(rect);
    if inner.height < 3 {
        return;
    }
    let raised = th.raised();
    draw_query(th, buf, inner, query, "type to filter", app.ui.caret);
    let list = Rect::new(inner.x, inner.y + 2, inner.width, inner.height - 2);
    if entries.is_empty() {
        buf.set_span(
            list.x + 2,
            list.y,
            &Span::styled("No matching commands.", th.fg(Token::Muted).patch(raised)),
            list.width,
        );
        return;
    }
    let top = sel.saturating_sub(list.height as usize - 1);
    for (row, (i, e)) in entries
        .iter()
        .enumerate()
        .skip(top)
        .take(list.height as usize)
        .enumerate()
    {
        let y = list.y + row as u16;
        let selected = i == *sel;
        let base = if selected { th.selection() } else { raised };
        for x in list.left()..list.right() {
            buf[(x, y)].set_style(base);
        }
        if selected {
            buf.set_span(
                list.x,
                y,
                &Span::styled(g.cursor, th.fg(Token::Accent).patch(base)),
                1,
            );
        }
        let mark = if e.network {
            th.text("⇅ ")
        } else {
            "  ".into()
        };
        let keys = e.keys.clone().map(|k| th.text(&k)).unwrap_or_default();
        let keys_w = width(&keys) as u16;
        // a disabled entry (a run is going) is muted whole: mark, label and suffix
        let (mark_tok, label_tok) = if e.disabled {
            (Token::Muted, Token::Muted)
        } else {
            (Token::Info, Token::Text)
        };
        let mut x = list.x + 1;
        buf.set_span(
            x,
            y,
            &Span::styled(mark.clone(), th.fg(mark_tok).patch(base)),
            2,
        );
        x += width(&mark) as u16;
        let avail =
            list.right()
                .saturating_sub(x + if keys_w > 0 { keys_w + 3 } else { 1 }) as usize;
        let label = th.text(&e.label);
        let mut suffix = if let Some(sfx) = &e.suffix {
            th.text(&format!(" {sfx}"))
        } else {
            String::new()
        };
        // The suffix stays while the label keeps at least 12 cells; the label shortens first.
        if width(&label).min(12) + width(&suffix) > avail {
            suffix.clear();
        }
        let label = fit_str(&label, avail.saturating_sub(width(&suffix)), g.ellipsis);
        let w = width(&label) as u16;
        buf.set_span(x, y, &Span::styled(label, th.fg(label_tok).patch(base)), w);
        x += w;
        if !suffix.is_empty() {
            let sw = width(&suffix) as u16;
            buf.set_span(
                x,
                y,
                &Span::styled(suffix, th.fg(Token::Muted).patch(base)),
                sw,
            );
        }
        if keys_w > 0 {
            buf.set_span(
                list.right() - 1 - keys_w,
                y,
                &Span::styled(keys, th.fg(Token::Muted).patch(base)),
                keys_w,
            );
        }
        if selected {
            th.band_row(buf, Rect::new(list.x, y, list.width, 1));
        }
        hits.add(Rect::new(list.x, y, list.width, 1), Target::PaletteRow(i));
    }
}

/// The filter line of a finder: `› query` with a caret, or the placeholder; then a rule.
pub fn draw_query(
    th: &crate::theme::Theme,
    buf: &mut Buffer,
    inner: Rect,
    query: &str,
    placeholder: &str,
    caret: Option<usize>,
) {
    let raised = th.raised();
    let g = th.glyphs();
    let prompt = format!("{} ", th.text("›"));
    buf.set_span(
        inner.x + 1,
        inner.y,
        &Span::styled(prompt.clone(), th.fg(Token::Muted).patch(raised)),
        2,
    );
    let x = inner.x + 1 + width(&prompt) as u16;
    let room = inner.right().saturating_sub(x + 1);
    if query.is_empty() {
        let caret = Span::styled(" ", raised.add_modifier(ratatui::style::Modifier::REVERSED));
        buf.set_span(x, inner.y, &caret, 1);
        buf.set_span(
            x + 1,
            inner.y,
            &Span::styled(
                fit_str(placeholder, room.saturating_sub(1) as usize, g.ellipsis),
                th.fg(Token::Muted).patch(raised),
            ),
            room.saturating_sub(1),
        );
    } else {
        let q = fit_str(query, room.saturating_sub(1) as usize, g.ellipsis);
        let w = width(&q) as u16;
        buf.set_span(
            x,
            inner.y,
            &Span::styled(q.clone(), th.fg(Token::Text).patch(raised)),
            w,
        );
        crate::widgets::caret(buf, x, inner.y, &q, caret, raised, w);
    }
    let line = th.fg(Token::Line).patch(raised);
    for xx in inner.left()..inner.right() {
        buf[(xx, inner.y + 1)].set_symbol(g.rule).set_style(line);
    }
}
