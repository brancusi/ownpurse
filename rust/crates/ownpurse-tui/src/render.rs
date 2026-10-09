//! Styled review renders of a frame (tc guide-renders pattern): an HTML `<pre>` with one inline-styled
//! `<span>` per style run, and an ANSI (SGR) text file, so design can review colour outside a terminal.
//! Plain text stays `snapshot::text`. Pure functions over a `Buffer`; nothing here touches a terminal.

use crate::theme::Theme;
use ratatui::buffer::{Buffer, Cell};
use ratatui::style::{Color, Modifier};
use unicode_width::UnicodeWidthStr;

/// The xterm palette for the 16 named colours (ANSI themes render on a generic dark terminal).
const NAMED: [(u8, u8, u8); 16] = [
    (0x00, 0x00, 0x00),
    (0xcd, 0x00, 0x00),
    (0x00, 0xcd, 0x00),
    (0xcd, 0xcd, 0x00),
    (0x00, 0x00, 0xee),
    (0xcd, 0x00, 0xcd),
    (0x00, 0xcd, 0xcd),
    (0xe5, 0xe5, 0xe5),
    (0x7f, 0x7f, 0x7f),
    (0xff, 0x00, 0x00),
    (0x00, 0xff, 0x00),
    (0xff, 0xff, 0x00),
    (0x5c, 0x5c, 0xff),
    (0xff, 0x00, 0xff),
    (0x00, 0xff, 0xff),
    (0xff, 0xff, 0xff),
];

fn named_index(c: Color) -> Option<u8> {
    Some(match c {
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::Gray => 7,
        Color::DarkGray => 8,
        Color::LightRed => 9,
        Color::LightGreen => 10,
        Color::LightYellow => 11,
        Color::LightBlue => 12,
        Color::LightMagenta => 13,
        Color::LightCyan => 14,
        Color::White => 15,
        _ => return None,
    })
}

/// An xterm-256 index as RGB.
pub fn indexed_rgb(i: u8) -> (u8, u8, u8) {
    match i {
        0..=15 => NAMED[i as usize],
        16..=231 => {
            let n = i - 16;
            let level = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            (level(n / 36), level((n / 6) % 6), level(n % 6))
        }
        _ => {
            let g = 8 + (i - 232) * 10;
            (g, g, g)
        }
    }
}

/// A colour as RGB (`None` = the terminal default).
pub fn rgb(c: Color) -> Option<(u8, u8, u8)> {
    match c {
        Color::Reset => None,
        Color::Rgb(r, g, b) => Some((r, g, b)),
        Color::Indexed(i) => Some(indexed_rgb(i)),
        other => named_index(other).map(|i| NAMED[i as usize]),
    }
}

/// The page's default background and foreground for a theme (ANSI and NO_COLOR: a generic dark terminal).
pub fn page_colors(th: &Theme) -> ((u8, u8, u8), (u8, u8, u8)) {
    match th.name() {
        "ledger-dark" | "ledger-dark-256" => ((0x14, 0x19, 0x1c), (0xe2, 0xe8, 0xea)),
        "ledger-light" | "ledger-light-256" => ((0xf5, 0xf7, 0xf5), (0x1c, 0x24, 0x27)),
        _ => ((0x1d, 0x1f, 0x21), (0xc5, 0xc8, 0xc6)),
    }
}

fn hex((r, g, b): (u8, u8, u8)) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// The runs of one row: (symbols, cell style source), wide clusters drawn once.
fn runs(buf: &Buffer, y: u16) -> Vec<(String, Cell)> {
    let mut out: Vec<(String, Cell)> = Vec::new();
    let mut skip = 0;
    for x in buf.area.left()..buf.area.right() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let cell = &buf[(x, y)];
        let sym = cell.symbol();
        skip = UnicodeWidthStr::width(sym).saturating_sub(1);
        let same = out.last().is_some_and(|(_, c)| {
            c.fg == cell.fg
                && c.bg == cell.bg
                && c.modifier == cell.modifier
                && c.underline_color == cell.underline_color
        });
        if same {
            if let Some((s, _)) = out.last_mut() {
                s.push_str(sym);
            }
        } else {
            out.push((sym.to_string(), cell.clone()));
        }
    }
    out
}

/// A standalone HTML page: one `<pre>`, inline colours per style run.
pub fn html(buf: &Buffer, th: &Theme, title: &str) -> String {
    let (page_bg, page_fg) = page_colors(th);
    let mut body = String::new();
    for y in buf.area.top()..buf.area.bottom() {
        for (text, c) in runs(buf, y) {
            let mut fg = rgb(c.fg).unwrap_or(page_fg);
            let mut bg = rgb(c.bg);
            if c.modifier.contains(Modifier::REVERSED) {
                let b = bg.unwrap_or(page_bg);
                bg = Some(fg);
                fg = b;
            }
            if c.modifier.contains(Modifier::DIM) {
                // dim the text only (opacity would lighten a reversed or painted background too)
                let b = bg.unwrap_or(page_bg);
                let m = |x: u8, y: u8| ((u16::from(x) * 6 + u16::from(y) * 4) / 10) as u8;
                fg = (m(fg.0, b.0), m(fg.1, b.1), m(fg.2, b.2));
            }
            let mut css = Vec::new();
            if fg != page_fg {
                css.push(format!("color:{}", hex(fg)));
            }
            if let Some(b) = bg.filter(|b| *b != page_bg) {
                css.push(format!("background:{}", hex(b)));
            }
            if c.modifier.contains(Modifier::BOLD) {
                css.push("font-weight:bold".into());
            }
            if c.modifier.contains(Modifier::ITALIC) {
                css.push("font-style:italic".into());
            }
            if c.modifier.contains(Modifier::UNDERLINED) {
                css.push("text-decoration:underline".into());
                if let Some(u) = rgb(c.underline_color) {
                    css.push(format!("text-decoration-color:{}", hex(u)));
                }
            }
            if css.is_empty() {
                body.push_str(&escape(&text));
            } else {
                body.push_str(&format!(
                    "<span style=\"{}\">{}</span>",
                    css.join(";"),
                    escape(&text)
                ));
            }
        }
        body.push('\n');
    }
    format!(
        "<!doctype html>\n<html><head><meta charset=\"utf-8\"><title>{}</title></head>\n\
         <body style=\"margin:0;padding:12px;background:#808080\">\n<pre style=\"display:inline-block;margin:0;padding:0;background:{bg};color:{fg};\
         font:13px/1.25 'JetBrains Mono','SF Mono',Menlo,monospace\">\n{body}</pre>\n</body></html>\n",
        escape(title),
        bg = hex(page_bg),
        fg = hex(page_fg),
    )
}

/// One style run with its first column and width in cells (wide clusters count 2).
pub struct Run {
    pub x: u16,
    pub cells: u16,
    pub text: String,
    /// Each cell cluster and its width (1, or 2 for wide characters).
    pub syms: Vec<(String, u16)>,
    /// Resolved colours (REVERSED applied): foreground, and background when it isn't the page's.
    pub fg: (u8, u8, u8),
    pub bg: Option<(u8, u8, u8)>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    pub underline: bool,
}

/// The runs of row `y` with positions and resolved colours (for SVG and the PNG grid).
pub fn placed_runs(buf: &Buffer, th: &Theme, y: u16) -> Vec<Run> {
    let (page_bg, page_fg) = page_colors(th);
    let mut out: Vec<Run> = Vec::new();
    let mut skip = 0;
    for x in buf.area.left()..buf.area.right() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        let c = &buf[(x, y)];
        let sym = c.symbol();
        let w = UnicodeWidthStr::width(sym).max(1) as u16;
        skip = w.saturating_sub(1);
        let mut fg = rgb(c.fg).unwrap_or(page_fg);
        let mut bg = rgb(c.bg);
        if c.modifier.contains(Modifier::REVERSED) {
            let b = bg.unwrap_or(page_bg);
            bg = Some(fg);
            fg = b;
        }
        let bg = bg.filter(|b| *b != page_bg);
        let (bold, dim, italic, underline) = (
            c.modifier.contains(Modifier::BOLD),
            c.modifier.contains(Modifier::DIM),
            c.modifier.contains(Modifier::ITALIC),
            c.modifier.contains(Modifier::UNDERLINED),
        );
        match out.last_mut() {
            Some(r)
                if r.fg == fg
                    && r.bg == bg
                    && (r.bold, r.dim, r.italic, r.underline) == (bold, dim, italic, underline) =>
            {
                r.text.push_str(sym);
                r.syms.push((sym.to_string(), w));
                r.cells += w;
            }
            _ => out.push(Run {
                x: x - buf.area.left(),
                cells: w,
                text: sym.to_string(),
                syms: vec![(sym.to_string(), w)],
                fg,
                bg,
                bold,
                dim,
                italic,
                underline,
            }),
        }
    }
    out
}

/// SVG cell size in user units (a 16-unit monospace font on a 10 × 20 grid).
pub const SVG_CELL: (u16, u16) = (10, 20);

/// The frame as SVG: one background rect per coloured run, one `<text>` per run stretched to its cells
/// (`textLength`), so the grid holds whatever monospace font renders it. Bold, dim, italic and underline
/// carry over; colours are the theme's resolved RGB (as the HTML render).
pub fn svg(buf: &Buffer, th: &Theme) -> String {
    let (cw, ch) = SVG_CELL;
    let (page_bg, page_fg) = page_colors(th);
    let (w, h) = (buf.area.width * cw, buf.area.height * ch);
    let mut out = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" \
         font-family=\"Hack, 'DejaVu Sans Mono', Menlo, monospace\" font-size=\"16\">\n\
         <rect width=\"{w}\" height=\"{h}\" fill=\"{}\"/>\n",
        hex(page_bg)
    );
    for y in buf.area.top()..buf.area.bottom() {
        let runs = placed_runs(buf, th, y);
        let ry = (y - buf.area.top()) * ch;
        for r in &runs {
            if let Some(b) = r.bg {
                out.push_str(&format!(
                    "<rect x=\"{}\" y=\"{ry}\" width=\"{}\" height=\"{ch}\" fill=\"{}\"/>\n",
                    r.x * cw,
                    r.cells * cw,
                    hex(b)
                ));
            }
        }
        for r in &runs {
            if r.text.trim().is_empty() {
                continue;
            }
            let mut attrs = format!("fill=\"{}\"", hex(r.fg));
            if r.fg == page_fg {
                attrs = format!("fill=\"{}\"", hex(page_fg));
            }
            if r.bold {
                attrs.push_str(" font-weight=\"bold\"");
            }
            if r.dim {
                attrs.push_str(" fill-opacity=\"0.6\"");
            }
            if r.italic {
                attrs.push_str(" font-style=\"italic\"");
            }
            if r.underline {
                attrs.push_str(" text-decoration=\"underline\"");
            }
            out.push_str(&format!(
                "<text x=\"{}\" y=\"{}\" textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\" xml:space=\"preserve\" {attrs}>{}</text>\n",
                r.x * cw,
                ry + ch - 5,
                r.cells * cw,
                escape(&r.text)
            ));
        }
    }
    out.push_str("</svg>\n");
    out
}

/// The frame as a cell grid for the PNG rasteriser in the CLI (which never links this crate): size, page
/// colours, and per row the placed runs with resolved colours.
pub fn grid(buf: &Buffer, th: &Theme) -> serde_json::Value {
    let (page_bg, page_fg) = page_colors(th);
    let rows: Vec<serde_json::Value> = (buf.area.top()..buf.area.bottom())
        .map(|y| {
            serde_json::Value::Array(
                placed_runs(buf, th, y)
                    .into_iter()
                    .map(|r| {
                        serde_json::json!({
                            "x": r.x,
                            "w": r.cells,
                            "t": r.text,
                            "s": r.syms.iter().map(|(t, _)| t.clone()).collect::<Vec<_>>(),
                            "sw": r.syms.iter().map(|(_, w)| *w).collect::<Vec<_>>(),
                            "fg": hex(r.fg),
                            "bg": r.bg.map(hex),
                            "b": r.bold,
                            "d": r.dim,
                            "i": r.italic,
                            "u": r.underline,
                        })
                    })
                    .collect(),
            )
        })
        .collect();
    serde_json::json!({
        "cols": buf.area.width,
        "rows": buf.area.height,
        "bg": hex(page_bg),
        "fg": hex(page_fg),
        "lines": rows,
    })
}

fn sgr_color(c: Color, fg: bool) -> Option<String> {
    let base = if fg { 30 } else { 40 };
    match c {
        Color::Reset => None,
        Color::Rgb(r, g, b) => Some(format!("{};2;{r};{g};{b}", base + 8)),
        Color::Indexed(i) => Some(format!("{};5;{i}", base + 8)),
        other => named_index(other).map(|i| {
            if i < 8 {
                format!("{}", base + i as u16)
            } else {
                format!("{}", base + 60 + (i - 8) as u16)
            }
        }),
    }
}

/// The frame as text with SGR escapes (`cat` it in a terminal); every row ends with a reset.
pub fn ansi(buf: &Buffer) -> String {
    let mut out = String::new();
    for y in buf.area.top()..buf.area.bottom() {
        for (text, c) in runs(buf, y) {
            let mut codes: Vec<String> = Vec::new();
            for (m, code) in [
                (Modifier::BOLD, "1"),
                (Modifier::DIM, "2"),
                (Modifier::ITALIC, "3"),
                (Modifier::UNDERLINED, "4"),
                (Modifier::REVERSED, "7"),
            ] {
                if c.modifier.contains(m) {
                    codes.push(code.into());
                }
            }
            codes.extend(sgr_color(c.fg, true));
            codes.extend(sgr_color(c.bg, false));
            if c.modifier.contains(Modifier::UNDERLINED)
                && let Color::Rgb(r, g, b) = c.underline_color
            {
                codes.push(format!("58;2;{r};{g};{b}"));
            }
            if codes.is_empty() {
                out.push_str(&text);
            } else {
                out.push_str(&format!("\x1b[{}m{text}\x1b[0m", codes.join(";")));
            }
        }
        out.push_str("\x1b[0m\n");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::layout::Rect;
    use ratatui::style::Style;

    fn buf() -> Buffer {
        let mut b = Buffer::empty(Rect::new(0, 0, 6, 1));
        b.set_string(0, 0, "a<", Style::default().fg(Color::Rgb(255, 0, 0)));
        b.set_string(2, 0, "東", Style::default().add_modifier(Modifier::BOLD));
        b.set_string(
            4,
            0,
            "z",
            Style::default().fg(Color::Red).bg(Color::Indexed(16)),
        );
        b
    }

    #[test]
    fn html_runs_escape_and_colour() {
        let h = html(&buf(), &crate::theme::LEDGER_DARK, "t");
        assert!(
            h.contains("<span style=\"color:#ff0000\">a&lt;</span>"),
            "{h}"
        );
        assert!(
            h.contains("<span style=\"font-weight:bold\">東</span>"),
            "{h}"
        );
        assert!(h.contains("color:#cd0000;background:#000000"), "{h}");
    }

    #[test]
    fn ansi_sgr_codes() {
        let a = ansi(&buf());
        assert!(a.contains("\x1b[38;2;255;0;0ma<\x1b[0m"), "{a:?}");
        assert!(a.contains("\x1b[1m東\x1b[0m"), "{a:?}");
        assert!(a.contains("\x1b[31;48;5;16mz\x1b[0m"), "{a:?}");
    }

    #[test]
    fn svg_and_grid_place_runs_on_the_cell_grid() {
        let th = crate::theme::LEDGER_DARK;
        let v = svg(&buf(), &th);
        assert!(
            v.starts_with("<svg") && v.trim_end().ends_with("</svg>"),
            "{v}"
        );
        assert!(v.contains("width=\"60\" height=\"20\""), "{v}");
        assert!(v.contains(">a&lt;</text>"), "{v}");
        // 東 is two cells wide: the next run starts at column 4 (x = 40)
        assert!(
            v.contains("x=\"40\" y=\"0\" width=\"10\" height=\"20\" fill=\"#000000\""),
            "{v}"
        );
        let g = grid(&buf(), &th);
        assert_eq!(g["cols"], 6);
        let runs = g["lines"][0].as_array().unwrap();
        assert_eq!(runs[1]["t"], "東");
        assert_eq!(runs[1]["w"], 2);
        assert_eq!(runs[1]["b"], true);
    }

    #[test]
    fn palette() {
        assert_eq!(indexed_rgb(196), (255, 0, 0));
        assert_eq!(indexed_rgb(232), (8, 8, 8));
        assert_eq!(indexed_rgb(9), (255, 0, 0));
    }
}
