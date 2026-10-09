//! The scope picker (`g`, spec §9): ranked list (alias > prefix > short code > name word > fuzzy), each
//! org with its freshness and calls left, `● current` on the current scope. The multi-select (Compare)
//! mode arrives with Compare in Phase 4.

use crate::app::{App, Overlay};
use crate::hit::{Hits, Target};
use crate::layout::{fit_str, width};
use crate::theme_tokens::Token;
use crate::widgets::centred;
use crate::widgets::palette::draw_query;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Span;

pub fn draw(app: &App, buf: &mut Buffer, area: Rect, hits: &mut Hits) {
    let Some(Overlay::Picker {
        query,
        sel,
        multi,
        chosen,
    }) = &app.ui.overlay
    else {
        return;
    };
    let th = &app.ui.theme;
    let g = th.glyphs();
    let entries = app.picker_entries(query);
    let box_w = area.width.saturating_sub(4).min(96);
    let box_h = (entries.len().max(1) + 4) as u16;
    let rect = centred(
        area,
        box_w,
        box_h.min(area.height.saturating_sub(6)),
        Some(3),
    );
    let title = if *multi {
        "Compare orgs: space toggles, Enter opens with 2–4"
    } else {
        "Switch scope"
    };
    let inner = crate::widgets::overlay_box_clear(buf, rect, title, th);
    hits.overlay = Some(rect);
    if inner.height < 3 {
        return;
    }
    let raised = th.raised();
    draw_query(
        th,
        buf,
        inner,
        query,
        "type to filter (alias, name, short code)",
        app.ui.caret,
    );
    let list = Rect::new(inner.x, inner.y + 2, inner.width, inner.height - 2);
    if entries.is_empty() {
        buf.set_span(
            list.x + 2,
            list.y,
            &Span::styled("No matching scope.", th.fg(Token::Muted).patch(raised)),
            list.width,
        );
        return;
    }
    let label_w = entries
        .iter()
        .map(|e| width(&e.label))
        .max()
        .unwrap_or(0)
        .min(24);
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
        let current = if e.current {
            th.text("● current")
        } else {
            String::new()
        };
        let cur_w = width(&current) as u16;
        let mut x = list.x + 2;
        if *multi {
            // [x] chosen · [-] some of a group's members · [ ] none (a glyph, not only colour; V12)
            let members = app.members(&e.scope);
            let n = members.iter().filter(|m| chosen.contains(m)).count();
            let mark = if n == members.len() && n > 0 {
                "[x] "
            } else if n > 0 {
                "[-] "
            } else {
                "[ ] "
            };
            buf.set_span(
                x,
                y,
                &Span::styled(mark, th.fg(Token::Accent).patch(base)),
                4,
            );
            x += 4;
        }
        let label = fit_str(&e.label, label_w, g.ellipsis);
        buf.set_span(
            x,
            y,
            &Span::styled(label, th.fg(Token::Text).patch(base)),
            label_w as u16,
        );
        let dx = x + label_w as u16 + 2;
        let room = list.right().saturating_sub(dx + cur_w + 2) as usize;
        buf.set_span(
            dx,
            y,
            &Span::styled(
                fit_str(&th.text(&e.detail), room, g.ellipsis),
                th.fg(Token::Muted).patch(base),
            ),
            room as u16,
        );
        if cur_w > 0 {
            buf.set_span(
                list.right() - 1 - cur_w,
                y,
                &Span::styled(current, th.fg(Token::Accent).patch(base)),
                cur_w,
            );
        }
        if selected {
            th.band_row(buf, Rect::new(list.x, y, list.width, 1));
        }
        hits.add(Rect::new(list.x, y, list.width, 1), Target::PickerRow(i));
    }
}
