//! Explainer panels (walkthroughs design §4): one renderer per kind over core's `viz::Model`. Pure over the
//! model and the theme: box drawing in the colour themes, pure ASCII in the plain theme (`ascii`), and a
//! table when the area is too small for the drawing. Every element registers its anchor (`viz:<id>`) in the
//! returned [`Panel`]; elements scrolled out of sight are recorded as off screen, above or below.

pub mod before_after;
pub mod bridge;
pub mod checklist;
pub mod flow;
pub mod t_account;
pub mod timeline;

use crate::anchors::VizPanel;
use crate::layout::fit_str;
pub use crate::layout::width;
use crate::theme::Theme;
use crate::theme_tokens::Token;
use caretline_layers::Off;
use ownpurse_core::viz::Model;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::Span;

/// What a panel drew: each element's rect (`viz:<id>` without the prefix), the elements scrolled out of
/// sight, and how many body rows it has (for scrolling).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Panel {
    pub anchors: Vec<(String, Rect)>,
    pub off: Vec<(String, Off)>,
    pub rows: u16,
    /// The body's visible height.
    pub visible: u16,
    /// Where explanatory text was drawn (each wrapped line's cells): never cut (tests check).
    pub prose: Vec<Rect>,
}

impl VizPanel for Panel {
    fn anchors(&self) -> Vec<(String, Rect)> {
        self.anchors.clone()
    }
}

impl Panel {
    fn put(&mut self, id: &str, rect: Rect) {
        if let Some(e) = self.anchors.iter_mut().find(|(i, _)| i == id) {
            e.1 = e.1.union(rect);
        } else {
            self.anchors.push((id.to_string(), rect));
        }
    }
    fn put_off(&mut self, id: &str, off: Off) {
        if !self.anchors.iter().any(|(i, _)| i == id) && !self.off.iter().any(|(i, _)| i == id) {
            self.off.push((id.to_string(), off));
        }
    }
}

/// How the panel looks up an org's accent token (the viz's members, in the scope accent's order).
pub type OrgToken<'a> = &'a dyn Fn(&str) -> Token;

/// Draw a model into `area` (the content area). `scroll` is the body's first row.
/// The widest an explainer panel draws (centred in wider content areas).
pub const PANEL_W: u16 = 120;
/// The widest a T-account draws.
pub const T_ACCOUNT_W: u16 = 100;

pub fn draw(
    model: &Model,
    th: &Theme,
    org: OrgToken,
    buf: &mut Buffer,
    area: Rect,
    scroll: u16,
) -> Panel {
    let lead = model.lead();
    let lead = lead.as_str();
    // a readable width, centred: the T-account's two sides stay close, the others stay a paragraph wide
    let cap = match model {
        Model::TAccount(_) => T_ACCOUNT_W,
        _ => PANEL_W,
    };
    let area = if area.width > cap {
        Rect::new(area.x + (area.width - cap) / 2, area.y, cap, area.height)
    } else {
        area
    };
    match model {
        Model::Bridge(m) => bridge::draw(m, lead, th, org, buf, area, scroll),
        Model::Flow(m) => flow::draw(m, lead, th, org, buf, area, scroll),
        Model::TAccount(m) => t_account::draw(m, lead, th, org, buf, area, scroll),
        Model::Timeline(m) => timeline::draw(m, lead, th, org, buf, area, scroll),
        Model::BeforeAfter(m) => before_after::draw(m, lead, th, org, buf, area, scroll),
        Model::Checklist(m) => checklist::draw(m, lead, th, org, buf, area, scroll),
    }
}

/// An amount: thousands separators, two decimals, a minus sign when negative (`−1,234.00`; `-` in the
/// plain theme through the fold).
pub fn money(v: rust_decimal::Decimal) -> String {
    let s = ownpurse_core::fmt(v.abs());
    if v < rust_decimal::Decimal::ZERO && s != "0.00" {
        format!("−{s}")
    } else {
        s
    }
}

/// A change: `+1,234.00` / `−1,234.00` / `0.00`.
pub fn signed(v: rust_decimal::Decimal) -> String {
    if v > rust_decimal::Decimal::ZERO {
        format!("+{}", money(v))
    } else {
        money(v)
    }
}

/// The box-drawing and bar glyphs, Unicode or ASCII.
pub struct G {
    pub h: &'static str,
    pub hh: &'static str,
    pub v: &'static str,
    pub cross: &'static str,
    pub tl: &'static str,
    pub tr: &'static str,
    pub bl: &'static str,
    pub br: &'static str,
    pub tee_down: &'static str,
    pub tee_up: &'static str,
    pub bar: &'static str,
    pub tick: &'static str,
    pub arrow_r: &'static str,
    pub arrow_l: &'static str,
    pub dot: &'static str,
}

pub const UNI: G = G {
    h: "─",
    hh: "═",
    v: "│",
    cross: "┼",
    tl: "┌",
    tr: "┐",
    bl: "└",
    br: "┘",
    tee_down: "┬",
    tee_up: "┴",
    bar: "█",
    tick: "▏",
    arrow_r: "▶",
    arrow_l: "◀",
    dot: "·",
};

pub const ASC: G = G {
    h: "-",
    hh: "=",
    v: "|",
    cross: "+",
    tl: "+",
    tr: "+",
    bl: "+",
    br: "+",
    tee_down: "+",
    tee_up: "+",
    bar: "#",
    tick: "|",
    arrow_r: ">",
    arrow_l: "<",
    dot: "-",
};

pub fn g(th: &Theme) -> &'static G {
    if th.ascii { &ASC } else { &UNI }
}

/// Text for the panel: the UI glyphs folded in the plain theme, and anything else non-ASCII (names from
/// the books) shown as `?` there, so the plain theme draws pure ASCII.
pub fn t(th: &Theme, s: &str) -> String {
    let f = th.text(s);
    if th.ascii {
        let mut out = String::with_capacity(f.len());
        for c in f.chars() {
            if c.is_ascii() {
                out.push(c);
            } else {
                out.push_str(latin_fold(c).unwrap_or("?"));
            }
        }
        out
    } else {
        f
    }
}

/// Latin letters with marks, ligatures and the like as plain ASCII (`é` → `e`, `ß` → `ss`, `ø` → `o`); None
/// for anything else (shown as `?`).
pub fn latin_fold(c: char) -> Option<&'static str> {
    const TABLE: &[(&str, &str)] = &[
        ("àáâãäåāăą", "a"),
        ("ÀÁÂÃÄÅĀĂĄ", "A"),
        ("çćĉċč", "c"),
        ("ÇĆĈĊČ", "C"),
        ("ďđð", "d"),
        ("ĎĐÐ", "D"),
        ("èéêëēĕėęě", "e"),
        ("ÈÉÊËĒĔĖĘĚ", "E"),
        ("ĝğġģ", "g"),
        ("ĜĞĠĢ", "G"),
        ("ĥħ", "h"),
        ("ĤĦ", "H"),
        ("ìíîïĩīĭįı", "i"),
        ("ÌÍÎÏĨĪĬĮİ", "I"),
        ("ĵ", "j"),
        ("Ĵ", "J"),
        ("ķ", "k"),
        ("Ķ", "K"),
        ("ĺļľŀł", "l"),
        ("ĹĻĽĿŁ", "L"),
        ("ñńņňŉ", "n"),
        ("ÑŃŅŇ", "N"),
        ("òóôõöøōŏő", "o"),
        ("ÒÓÔÕÖØŌŎŐ", "O"),
        ("ŕŗř", "r"),
        ("ŔŖŘ", "R"),
        ("śŝşšș", "s"),
        ("ŚŜŞŠȘ", "S"),
        ("ţťŧț", "t"),
        ("ŢŤŦȚ", "T"),
        ("ùúûüũūŭůűų", "u"),
        ("ÙÚÛÜŨŪŬŮŰŲ", "U"),
        ("ŵ", "w"),
        ("Ŵ", "W"),
        ("ýÿŷ", "y"),
        ("ÝŸŶ", "Y"),
        ("źżž", "z"),
        ("ŹŻŽ", "Z"),
        ("ß", "ss"),
        ("æ", "ae"),
        ("Æ", "AE"),
        ("œ", "oe"),
        ("Œ", "OE"),
        ("þ", "th"),
        ("Þ", "TH"),
        ("‘’‚′", "'"),
        ("“”„″", "\""),
        ("–", "-"),
        ("\u{a0}", " "),
    ];
    TABLE
        .iter()
        .find(|(from, _)| from.contains(c))
        .map(|(_, to)| *to)
}

/// `text` fitted to `w` cells (the theme's ellipsis).
pub fn fit(th: &Theme, s: &str, w: usize) -> String {
    fit_str(&t(th, s), w, th.glyphs().ellipsis)
}

/// Write `s` at (x, y), at most `w` cells, in `style`; returns the cells used.
pub fn text(buf: &mut Buffer, x: u16, y: u16, s: &str, w: u16, style: Style) -> u16 {
    if w == 0 || y >= buf.area.bottom() || x >= buf.area.right() {
        return 0;
    }
    let w = w.min(buf.area.right() - x);
    let s = fit_str(s, w as usize, "");
    let n = width(&s) as u16;
    buf.set_span(x, y, &Span::styled(s, style), w);
    n
}

/// Right-aligned text ending at `right` (exclusive).
pub fn rtext(buf: &mut Buffer, right: u16, y: u16, s: &str, style: Style) -> Rect {
    let n = width(s) as u16;
    let x = right.saturating_sub(n);
    text(buf, x, y, s, n, style);
    Rect::new(x, y, n, 1)
}

/// Explanatory text, wrapped to `w` cells (never cut with an ellipsis): `(line, token)` per row.
pub fn wrapped(th: &Theme, s: &str, w: u16, tok: Token) -> Vec<(String, Token)> {
    crate::layout::wrap(&t(th, s), w.max(1) as usize)
        .into_iter()
        .map(|l| (l, tok))
        .collect()
}

/// The fixed head is the title alone (heading style; a title may be cut). The sentences under it (the lead
/// in `text.2`, the rest muted) are returned wrapped, for the body to show as its first rows: they scroll
/// with it and are never cut. Returns the body's area and those rows (with a blank row after them when the
/// area has room).
pub fn head(
    th: &Theme,
    buf: &mut Buffer,
    area: Rect,
    title: &str,
    sub: &[String],
) -> (Rect, Vec<(String, Token)>) {
    let w = area.width.saturating_sub(4);
    if area.height == 0 {
        return (area, vec![]);
    }
    text(
        buf,
        area.x + 2,
        area.y,
        &fit(th, title, w as usize),
        w,
        th.heading(),
    );
    let mut intro = vec![];
    for (i, s) in sub.iter().enumerate() {
        let tok = if i == 0 { Token::Text2 } else { Token::Muted };
        intro.extend(wrapped(th, s, w, tok));
    }
    if area.height > 12 && !intro.is_empty() {
        intro.push((String::new(), Token::Muted));
    }
    let y = area.y + 1;
    (
        Rect::new(area.x, y, area.width, area.bottom().saturating_sub(y)),
        intro,
    )
}

/// A scrolling list of body rows: `row()` gives the screen row for the next virtual row, or None when it
/// is scrolled out of sight (the caller still records its anchors as off screen).
pub struct Body {
    pub area: Rect,
    pub scroll: u16,
    pub n: u16,
    pub panel: Panel,
}

impl Body {
    pub fn new(area: Rect, scroll: u16, total: u16) -> Body {
        // the scroll never leaves a blank page below the last row
        let max = total.saturating_sub(area.height);
        let scroll = scroll.min(max);
        Body {
            area,
            scroll,
            n: 0,
            panel: Panel {
                visible: area.height,
                rows: total,
                ..Default::default()
            },
        }
    }

    /// The next row: Some(y) on screen, Err(off) above or below.
    pub fn row(&mut self) -> Result<u16, Off> {
        let i = self.n;
        self.n += 1;
        if i < self.scroll {
            Err(Off::Above { x: None })
        } else if i - self.scroll >= self.area.height {
            Err(Off::Below { x: None })
        } else {
            Ok(self.area.y + (i - self.scroll))
        }
    }

    /// Anchor `id` at `rect` (on screen) or as off screen.
    pub fn anchor(&mut self, id: &str, at: Result<Rect, Off>) {
        match at {
            Ok(r) => self.panel.put(id, r),
            Err(o) => self.panel.put_off(id, o),
        }
    }

    /// Wrapped explanatory lines as body rows at `x` (each line already fits).
    pub fn prose(
        &mut self,
        buf: &mut Buffer,
        th: &Theme,
        x: u16,
        lines: &[(String, Token)],
    ) -> Vec<Result<Rect, Off>> {
        let mut out = vec![];
        for (l, tok) in lines {
            let at = self.row();
            let r = at.map(|y| {
                let n = text(buf, x, y, l, width(l) as u16, th.fg(*tok));
                Rect::new(x, y, n, 1)
            });
            if let Ok(r) = r
                && r.width > 0
            {
                self.panel.prose.push(r);
            }
            out.push(r);
        }
        out
    }

    pub fn done(mut self) -> Panel {
        self.panel.rows = self.n;
        self.panel
    }
}

/// A one-line message in the body (an empty model, an error).
pub fn note(th: &Theme, buf: &mut Buffer, area: Rect, msg: &str, tok: Token) {
    if area.height == 0 {
        return;
    }
    let w = area.width.saturating_sub(4);
    text(
        buf,
        area.x + 2,
        area.y,
        &fit(th, msg, w as usize),
        w,
        th.fg(tok),
    );
}

/// A cell range of `n` cells of `glyph` at (x, y).
pub fn run(buf: &mut Buffer, x: u16, y: u16, n: u16, glyph: &str, style: Style) {
    for i in 0..n {
        if let Some(c) = buf.cell_mut((x + i, y)) {
            c.set_symbol(glyph);
            c.set_style(style);
        }
    }
}

/// Map `v` in [lo, hi] to a column in [0, w].
pub fn scale(
    v: rust_decimal::Decimal,
    lo: rust_decimal::Decimal,
    hi: rust_decimal::Decimal,
    w: u16,
) -> u16 {
    use rust_decimal::Decimal;
    use rust_decimal::prelude::ToPrimitive;
    if hi <= lo {
        return 0;
    }
    let f = ((v - lo) * Decimal::from(w) / (hi - lo)).round();
    f.to_u16().unwrap_or(0).min(w)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;

    #[test]
    fn plain_folds_latin_letters() {
        let th = crate::theme::NO_COLOR_ASCII;
        assert_eq!(t(&th, "Wren & Ashby Cafés"), "Wren & Ashby Cafes");
        assert_eq!(t(&th, "Rösterei Straße Øst"), "Rosterei Strasse Ost");
        assert_eq!(t(&th, "大阪"), "??");
        assert_eq!(t(&crate::theme::LEDGER_DARK, "Cafés"), "Cafés");
    }

    #[test]
    fn scale_ends() {
        let (lo, hi) = (Decimal::from(-10), Decimal::from(30));
        assert_eq!(scale(lo, lo, hi, 40), 0);
        assert_eq!(scale(hi, lo, hi, 40), 40);
        assert_eq!(scale(Decimal::ZERO, lo, hi, 40), 10);
    }

    #[test]
    fn body_scrolls_and_marks_off_screen() {
        let mut b = Body::new(Rect::new(0, 5, 10, 2), 1, 4);
        assert!(matches!(b.row(), Err(Off::Above { .. })));
        assert_eq!(b.row(), Ok(5));
        assert_eq!(b.row(), Ok(6));
        assert!(matches!(b.row(), Err(Off::Below { .. })));
    }
}
