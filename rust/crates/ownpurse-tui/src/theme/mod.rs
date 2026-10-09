//! Theme modes over the kit's tokens (spec §2): truecolor (`ledger-dark` / `ledger-light`), xterm-256,
//! ANSI-16 and `NO_COLOR` (modifiers only). Every style the UI paints comes from here.

use crate::glyphs::{self, Glyphs};
pub mod accent;
pub mod color;
pub mod def;
pub mod tokens;
pub mod user;

use ratatui::style::{Color, Modifier, Style};
use tokens::{BADGE_OFFLINE, BADGE_READ_ONLY, Token, styles};

#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Mode {
    Truecolor,
    Indexed,
    Ansi,
    NoColor,
}

#[derive(Clone, Copy, Debug, Eq, serde::Serialize, serde::Deserialize)]
pub struct Theme {
    pub mode: Mode,
    pub dark: bool,
    /// Paint the `bg` token behind everything (`[tui] paint_bg`, off by default).
    pub paint_bg: bool,
    /// The ASCII glyph set (non-UTF-8 or CJK locale, `glyphs = "ascii"`).
    pub ascii: bool,
    /// The resolved colours (derived from the theme, the knobs, the scope accent and the overlay by
    /// `App::repaint`; never stored). None: the built-in's colours with the cross accent.
    #[serde(skip)]
    pub pal: Option<def::Pal>,
}

/// Equality is the theme's inputs (mode, darkness, paint, glyphs): the resolved palette is derived state.
impl PartialEq for Theme {
    fn eq(&self, o: &Self) -> bool {
        (self.mode, self.dark, self.paint_bg, self.ascii) == (o.mode, o.dark, o.paint_bg, o.ascii)
    }
}

pub const LEDGER_DARK: Theme = Theme {
    mode: Mode::Truecolor,
    dark: true,
    paint_bg: false,
    ascii: false,
    pal: None,
};
pub const LEDGER_LIGHT: Theme = Theme {
    mode: Mode::Truecolor,
    dark: false,
    paint_bg: false,
    ascii: false,
    pal: None,
};
pub const ANSI: Theme = Theme {
    mode: Mode::Ansi,
    dark: true,
    paint_bg: false,
    ascii: false,
    pal: None,
};
pub const NO_COLOR_ASCII: Theme = Theme {
    mode: Mode::NoColor,
    dark: true,
    paint_bg: false,
    ascii: true,
    pal: None,
};

fn rgb(c: u32) -> Color {
    Color::Rgb((c >> 16) as u8, (c >> 8) as u8, c as u8)
}

/// The built-ins resolved with the cross accent and default knobs (computed once).
fn default_pal(dark: bool) -> &'static def::Pal {
    static D: std::sync::OnceLock<def::Pal> = std::sync::OnceLock::new();
    static L: std::sync::OnceLock<def::Pal> = std::sync::OnceLock::new();
    let cell = if dark { &D } else { &L };
    cell.get_or_init(|| {
        let name = if dark { "ledger-dark" } else { "ledger-light" };
        let t = def::builtin(name).unwrap_or_default();
        def::resolve(
            &t,
            def::Knobs::defaults(dark),
            "harbor",
            &[],
            &Default::default(),
        )
    })
}

impl Theme {
    /// `ledger-dark | ledger-light | ledger-dark-256 | ledger-light-256 | ansi | no-color`.
    pub fn named(name: &str) -> Option<Theme> {
        let t = match name {
            "ledger-dark" => LEDGER_DARK,
            "ledger-light" => LEDGER_LIGHT,
            "ledger-dark-256" => Theme {
                mode: Mode::Indexed,
                ..LEDGER_DARK
            },
            "ledger-light-256" => Theme {
                mode: Mode::Indexed,
                ..LEDGER_LIGHT
            },
            "ansi" => ANSI,
            "no-color" => Theme {
                ascii: false,
                ..NO_COLOR_ASCII
            },
            "no-color-ascii" => NO_COLOR_ASCII,
            _ => return None,
        };
        Some(t)
    }

    pub fn name(&self) -> &'static str {
        match (self.mode, self.dark) {
            (Mode::Truecolor, true) => "ledger-dark",
            (Mode::Truecolor, false) => "ledger-light",
            (Mode::Indexed, true) => "ledger-dark-256",
            (Mode::Indexed, false) => "ledger-light-256",
            (Mode::Ansi, _) => "ansi",
            (Mode::NoColor, _) => "no-color",
        }
    }

    pub fn glyphs(&self) -> &'static Glyphs {
        glyphs::set(self.ascii)
    }

    /// Fold display text to the ASCII glyph set when it's active (one place; measure after folding).
    pub fn text(&self, s: &str) -> String {
        glyphs::fold(s, self.ascii)
    }

    /// The resolved colour of a token (truecolor value).
    pub fn rgb(&self, t: Token) -> u32 {
        match &self.pal {
            Some(p) if p.dark == self.dark => p.get(t),
            _ => default_pal(self.dark).get(t),
        }
    }

    /// The resolved palette (the built-in's with the cross accent when none was set).
    pub fn palette(&self) -> def::Pal {
        self.pal
            .filter(|p| p.dark == self.dark)
            .unwrap_or(*default_pal(self.dark))
    }

    fn color(&self, t: Token) -> Option<Color> {
        match self.mode {
            Mode::Truecolor => Some(rgb(self.rgb(t))),
            Mode::Indexed => Some(Color::Indexed(color::xterm256(self.rgb(t)))),
            Mode::Ansi => t.ansi().0,
            Mode::NoColor => None,
        }
    }

    /// ANSI or NO_COLOR: no palette (overlays mark rings by reversing).
    pub fn palette_free(&self) -> bool {
        self.ansi_like()
    }

    fn ansi_like(&self) -> bool {
        matches!(self.mode, Mode::Ansi | Mode::NoColor)
    }

    /// Foreground style for a text token.
    pub fn fg(&self, t: Token) -> Style {
        let mut s = Style::default();
        if let Some(c) = self.color(t) {
            s = s.fg(c);
        }
        if self.ansi_like() {
            s = s.add_modifier(t.ansi().1);
        }
        if matches!(
            t,
            Token::Accent | Token::Member1 | Token::Member2 | Token::Member3 | Token::Member4
        ) && !self.ansi_like()
        {
            s = s.add_modifier(Modifier::BOLD);
        }
        s
    }

    /// Background paint for a surface token (truecolor / 256 only; ANSI paints nothing).
    pub fn bg(&self, t: Token) -> Style {
        if self.ansi_like() {
            return Style::default();
        }
        match self.color(t) {
            Some(c) => Style::default().bg(c),
            None => Style::default(),
        }
    }

    /// The frame's base: `bg` only with `paint_bg`.
    pub fn base(&self) -> Style {
        if self.paint_bg {
            self.bg(Token::Bg)
        } else {
            Style::default()
        }
    }

    /// Plain top surface (bars without a component band).
    pub fn bar(&self) -> Style {
        self.bg(Token::Surface)
    }

    /// Background paint for a computed colour (the header gradient, pair split): truecolor or 256 only.
    pub fn bg_rgb(&self, c: u32) -> Style {
        match self.mode {
            Mode::Truecolor => Style::default().bg(rgb(c)),
            Mode::Indexed => Style::default().bg(Color::Indexed(color::xterm256(c))),
            _ => Style::default(),
        }
    }

    /// A spotlight's dimmed text: the text colour taken about 62% of the way to the background (clearly
    /// quieter than any text token; ANSI and NO_COLOR use DIM, and the overlay marks the holes instead).
    pub fn spotlight_dim(&self) -> Style {
        let c = color::mix(self.rgb(Token::Text), self.rgb(Token::Bg), 620);
        match self.mode {
            Mode::Truecolor => Style::default().fg(rgb(c)),
            Mode::Indexed => Style::default().fg(Color::Indexed(color::xterm256(c))),
            _ => Style::default().add_modifier(Modifier::DIM),
        }
    }

    /// The header band (`header.band.bg`: the top surface tinted by the scope accent, knob `header_band`).
    pub fn header_band(&self) -> Style {
        self.bg(Token::HeaderBand)
    }

    /// The footer band (`footer.bg`).
    pub fn footer_band(&self) -> Style {
        self.bg(Token::FooterBand)
    }

    /// The hairline under a band: an underline in `rule` (truecolor and 256 colours; nothing in ANSI and
    /// NO_COLOR). The cell's foreground is set to `rule` on blank cells by the caller's fill, so a terminal
    /// without underline colours still draws it quietly.
    pub fn hairline(&self) -> Style {
        match self.color(Token::Line) {
            Some(c) if !self.ansi_like() => Style::default()
                .fg(c)
                .underline_color(c)
                .add_modifier(Modifier::UNDERLINED),
            _ => Style::default(),
        }
    }

    /// Overlays and the confirm band.
    pub fn raised(&self) -> Style {
        self.bg(Token::Raised)
    }

    /// The focused pane's cursor row: `selection` paint, or REVERSED with the fg stripped in ANSI.
    pub fn selection(&self) -> Style {
        if self.ansi_like() {
            // no DIM inside the band: DIM on a reversed cell dims its background and splits the row
            Style::default()
                .add_modifier(Modifier::REVERSED)
                .remove_modifier(Modifier::DIM)
        } else {
            self.bg(Token::RowSelected)
        }
    }

    /// The cursor row as one reversed band in ANSI and NO_COLOR (tc handoff §1.3.3): every cell's fg and bg
    /// stripped after the row is drawn, so coloured spans don't invert into solid blocks. Modifiers stay.
    pub fn band_row(&self, buf: &mut ratatui::buffer::Buffer, row: ratatui::layout::Rect) {
        if !self.ansi_like() {
            return;
        }
        let row = row.intersection(buf.area);
        for y in row.top()..row.bottom() {
            for x in row.left()..row.right() {
                let c = &mut buf[(x, y)];
                c.fg = Color::Reset;
                c.bg = Color::Reset;
                // one continuous reverse span: DIM on a reversed cell dims its background in many terminals
                c.modifier.remove(Modifier::DIM);
            }
        }
    }

    /// Inverse badges: `READ-ONLY` (ok) and `OFFLINE` (warn).
    pub fn badge(&self, t: Token) -> Style {
        match self.mode {
            Mode::Truecolor | Mode::Indexed => Style::default()
                .fg(self.color(Token::Bg).unwrap_or(Color::Reset))
                .bg(self.color(t).unwrap_or(Color::Reset))
                .add_modifier(Modifier::BOLD),
            Mode::Ansi => Style::default()
                .fg(self.color(t).unwrap_or(Color::Reset))
                .add_modifier(Modifier::REVERSED | Modifier::BOLD),
            Mode::NoColor => Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD),
        }
    }

    pub fn badge_read_only(&self) -> Style {
        self.badge(BADGE_READ_ONLY)
    }

    pub fn badge_offline(&self) -> Style {
        self.badge(BADGE_OFFLINE)
    }

    /// The confirm band's focused button.
    pub fn focused_button(&self) -> Style {
        self.fg(Token::Text).add_modifier(styles::FOCUSED_BUTTON)
    }

    pub fn heading(&self) -> Style {
        self.fg(Token::Text).add_modifier(styles::HEADING)
    }

    /// Next theme for the palette's "Change theme" (keeps the glyph set).
    pub fn cycle(&self) -> Theme {
        let next = match self.name() {
            "ledger-dark" => LEDGER_LIGHT,
            "ledger-light" => ANSI,
            _ => LEDGER_DARK,
        };
        Theme {
            ascii: self.ascii,
            paint_bg: self.paint_bg,
            ..next
        }
    }

    /// `[tui] theme = "auto"` (spec §2.2): `NO_COLOR` → no colour; truecolor by `COLORTERM`; 256 by
    /// `TERM`; else ANSI. Dark or light from `COLORFGBG` (the OSC 11 query lands with Phase 4).
    pub fn detect(env: &dyn Fn(&str) -> Option<String>) -> Theme {
        let ascii = !glyphs::utf8_locale(env);
        if env("NO_COLOR").is_some_and(|v| !v.is_empty()) {
            return Theme {
                ascii,
                ..NO_COLOR_ASCII
            };
        }
        let dark = env("COLORFGBG")
            .and_then(|v| v.rsplit(';').next().and_then(|b| b.parse::<u8>().ok()))
            .is_none_or(|bg| !matches!(bg, 7 | 15));
        let colorterm = env("COLORTERM").unwrap_or_default();
        let mode = if colorterm == "truecolor" || colorterm == "24bit" {
            Mode::Truecolor
        } else if env("TERM").is_some_and(|t| t.contains("256color")) {
            Mode::Indexed
        } else {
            Mode::Ansi
        };
        Theme {
            mode,
            dark,
            paint_bg: false,
            ascii,
            pal: None,
        }
    }
}

impl Token {
    pub fn all() -> impl Iterator<Item = Token> {
        Token::ALL.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detection() {
        let env = |pairs: &'static [(&'static str, &'static str)]| {
            move |k: &str| {
                pairs
                    .iter()
                    .find(|(a, _)| *a == k)
                    .map(|(_, v)| v.to_string())
            }
        };
        let t = Theme::detect(&env(&[("COLORTERM", "truecolor"), ("LANG", "en_US.UTF-8")]));
        assert_eq!(t, LEDGER_DARK);
        let t = Theme::detect(&env(&[("NO_COLOR", "1"), ("LANG", "C")]));
        assert_eq!(t.mode, Mode::NoColor);
        assert!(t.ascii);
        let t = Theme::detect(&env(&[
            ("TERM", "xterm-256color"),
            ("LC_ALL", "ja_JP.UTF-8"),
        ]));
        assert_eq!(t.mode, Mode::Indexed);
        assert!(t.ascii, "CJK locale gets the ASCII set");
        let t = Theme::detect(&env(&[
            ("COLORTERM", "24bit"),
            ("COLORFGBG", "0;15"),
            ("LANG", "en_US.UTF-8"),
        ]));
        assert_eq!(t, LEDGER_LIGHT);
    }

    #[test]
    fn no_color_paints_nothing() {
        for t in Token::all() {
            let s = NO_COLOR_ASCII.fg(t);
            assert!(s.fg.is_none() && s.bg.is_none(), "{t:?}");
        }
        assert!(NO_COLOR_ASCII.bar().bg.is_none());
        assert!(ANSI.raised().bg.is_none());
    }
}
