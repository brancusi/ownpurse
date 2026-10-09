//! Frame pieces (spec §3): header, tabs, footer, strip, table, palette, help, confirm band, picker.
//! Each draws into the buffer from `&App` and registers its hit targets; none mutates the app.

pub mod confirm;
pub mod footer;
pub mod header;
pub mod help;
pub mod palette;
pub mod picker;
pub mod runband;
pub mod strip;
pub mod table;
pub mod tabs;
pub mod viz;

use crate::layout;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};

/// Draw spans at (x, y), fitted to `max` cells with the theme's ellipsis. Returns the x after.
pub fn put(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    spans: Vec<Span<'static>>,
    max: u16,
    ellipsis: &str,
) -> u16 {
    if max == 0 || y >= buf.area.bottom() || x >= buf.area.right() {
        return x;
    }
    let max = max.min(buf.area.right() - x);
    let fitted = layout::fit(spans, max as usize, ellipsis);
    let w = layout::spans_width(&fitted) as u16;
    buf.set_line(x, y, &Line::from(fitted), max);
    x + w
}

/// Paint a rectangle's style (background bars, overlays).
pub fn fill(buf: &mut Buffer, rect: Rect, style: Style) {
    let rect = rect.intersection(buf.area);
    for y in rect.top()..rect.bottom() {
        for x in rect.left()..rect.right() {
            if let Some(c) = buf.cell_mut((x, y)) {
                // reset first: `set_style` only adds modifiers, so BOLD under an overlay would leak through
                c.reset();
                c.set_symbol(" ");
                c.set_style(style);
            }
        }
    }
}

/// A box for overlays: `raised` paint, a `line`-token border and a title (spec §3: boxes only for
/// overlays). Returns the inner rect.
pub fn overlay_box(buf: &mut Buffer, rect: Rect, title: &str, th: &crate::theme::Theme) -> Rect {
    use crate::theme_tokens::Token;
    let rect = rect.intersection(buf.area);
    // One clear column on each side, so the border never touches the view underneath.
    for y in rect.top()..rect.bottom() {
        for x in [rect.left().checked_sub(1), Some(rect.right())]
            .into_iter()
            .flatten()
        {
            if x < buf.area.right() {
                buf[(x, y)].set_symbol(" ").set_style(th.base());
            }
        }
    }
    fill(buf, rect, th.raised());
    if rect.width < 2 || rect.height < 2 {
        return Rect::default();
    }
    let g = th.glyphs();
    let line = th.fg(Token::Line).patch(th.raised());
    let (h, v) = (g.rule, g.vsep);
    let (tl, tr, bl, br) = if th.ascii {
        ("+", "+", "+", "+")
    } else {
        ("╭", "╮", "╰", "╯")
    };
    for x in rect.left()..rect.right() {
        buf[(x, rect.top())].set_symbol(h).set_style(line);
        buf[(x, rect.bottom() - 1)].set_symbol(h).set_style(line);
    }
    for y in rect.top()..rect.bottom() {
        buf[(rect.left(), y)].set_symbol(v).set_style(line);
        buf[(rect.right() - 1, y)].set_symbol(v).set_style(line);
    }
    buf[(rect.left(), rect.top())].set_symbol(tl);
    buf[(rect.right() - 1, rect.top())].set_symbol(tr);
    buf[(rect.left(), rect.bottom() - 1)].set_symbol(bl);
    buf[(rect.right() - 1, rect.bottom() - 1)].set_symbol(br);
    let t = th.text(&format!(" {title} "));
    put(
        buf,
        rect.left() + 2,
        rect.top(),
        vec![Span::styled(t, th.heading().patch(th.raised()))],
        rect.width.saturating_sub(4),
        th.glyphs().ellipsis,
    );
    Rect::new(
        rect.left() + 1,
        rect.top() + 1,
        rect.width - 2,
        rect.height - 2,
    )
}

/// `overlay_box`, also clearing any sliver of 1–2 columns left between its margin and the screen edge, so
/// no stray glyph of the view behind shows at the edge (design, 2026-10-06; every overlay from v0.12.0, the
/// shell's palette, picker and help included: the render epoch let their frames change).
pub fn overlay_box_clear(
    buf: &mut Buffer,
    rect: Rect,
    title: &str,
    th: &crate::theme::Theme,
) -> Rect {
    let r = rect.intersection(buf.area);
    let left_gap = r.left().saturating_sub(1).saturating_sub(buf.area.left());
    let right_gap = buf.area.right().saturating_sub(r.right() + 1);
    for y in r.top()..r.bottom() {
        if left_gap <= 2 {
            for x in buf.area.left()..r.left().saturating_sub(1) {
                buf[(x, y)].set_symbol(" ").set_style(th.base());
            }
        }
        if right_gap <= 2 {
            for x in (r.right() + 1).min(buf.area.right())..buf.area.right() {
                buf[(x, y)].set_symbol(" ").set_style(th.base());
            }
        }
    }
    overlay_box(buf, rect, title, th)
}

/// A centred rect of at most `w`×`h` inside `area`, `top` rows from the top when given.
pub fn centred(area: Rect, w: u16, h: u16, top: Option<u16>) -> Rect {
    let w = w.min(area.width.saturating_sub(4));
    let h = h.min(area.height.saturating_sub(2));
    let x = area.x + (area.width - w) / 2;
    let y = top.map_or(area.y + (area.height - h) / 2, |t| {
        area.y + t.min(area.height - h)
    });
    Rect::new(x, y, w, h)
}

/// The text cursor after drawn text `t` at (x, y): a reversed space after it, or (caret inside the text)
/// the character at the caret reversed. `w` is the drawn width of `t`.
pub fn caret(
    buf: &mut Buffer,
    x: u16,
    y: u16,
    t: &str,
    caret: Option<usize>,
    style: ratatui::style::Style,
    w: u16,
) {
    use ratatui::style::Modifier;
    match caret.filter(|&c| c < t.chars().count()) {
        None => buf.set_span(
            x + w,
            y,
            &Span::styled(" ", style.add_modifier(Modifier::REVERSED)),
            1,
        ),
        Some(c) => {
            let before: String = t.chars().take(c).collect();
            let cx = x + crate::layout::width(&before) as u16;
            let ch: String = t.chars().nth(c).map(String::from).unwrap_or_default();
            let cw = crate::layout::width(&ch).max(1) as u16;
            buf.set_span(
                cx,
                y,
                &Span::styled(ch, style.add_modifier(Modifier::REVERSED)),
                cw,
            )
        }
    };
}

/// A section band (design pass 2, "bands, not boxes"): the row on `section.band.bg` (tinted by the knob
/// `section_band`), the label in `text.2` at the 2-cell inset, an optional note at the right, and the
/// hairline under it as a `rule`-coloured underline (truecolor and 256 colours). In ANSI and NO_COLOR the
/// label is bold instead (no paint to separate it).
pub fn band(
    th: &crate::theme::Theme,
    buf: &mut Buffer,
    row: Rect,
    label: &str,
    right: Option<(String, crate::theme_tokens::Token)>,
) {
    band_to(th, buf, row, label, right, None);
}

/// `band` with the note ending at column `note_end` (a table's right edge) instead of the row's.
pub fn band_to(
    th: &crate::theme::Theme,
    buf: &mut Buffer,
    row: Rect,
    label: &str,
    right: Option<(String, crate::theme_tokens::Token)>,
    note_end: Option<u16>,
) {
    use crate::theme_tokens::Token;
    let row = row.intersection(buf.area);
    if row.height == 0 || row.width < 4 {
        return;
    }
    let row = Rect { height: 1, ..row };
    let painted = matches!(
        th.mode,
        crate::theme::Mode::Truecolor | crate::theme::Mode::Indexed
    );
    let bg = th.bg(Token::SectionBand);
    let hair = th.hairline();
    fill(buf, row, bg.patch(hair));
    let label_style = if painted {
        th.fg(Token::Text2)
    } else {
        th.heading()
    };
    let ell = th.glyphs().ellipsis;
    let right_w = right
        .as_ref()
        .map_or(0, |(t, _)| layout::width(&th.text(t)) as u16);
    let left_max = row
        .width
        .saturating_sub(4 + if right_w > 0 { right_w + 2 } else { 0 });
    put(
        buf,
        row.x + 2,
        row.y,
        vec![Span::styled(
            th.text(label),
            bg.patch(hair).patch(label_style),
        )],
        left_max,
        ell,
    );
    if let Some((t, tok)) = right
        && right_w + 4 < row.width
    {
        let end = note_end.map_or(row.right() - 2, |e| e.min(row.right() - 2));
        buf.set_span(
            end.saturating_sub(right_w).max(row.x + 2),
            row.y,
            &Span::styled(th.text(&t), bg.patch(hair).patch(th.fg(tok))),
            right_w,
        );
    }
}
