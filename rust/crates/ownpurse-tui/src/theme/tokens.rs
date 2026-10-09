//! The theme's token names (theme spec v2 §1). Values live in the theme definitions (`theme::def`); this file
//! only names what a widget may ask for. Semantic tokens say what a colour means; component tokens say where
//! it is used (their defaults are expressions over the semantic layer); member tokens carry the per-org
//! accents of the scope's members (pair chips, Compare columns).

use ratatui::style::{Color, Modifier};

macro_rules! tokens {
    ($( $v:ident = $name:literal, $ansi:expr, $amod:expr; )*) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Token { $( $v ),* }
        impl Token {
            pub const ALL: &'static [Token] = &[ $( Token::$v ),* ];
            pub const COUNT: usize = [ $( $name ),* ].len();
            /// The token's name in theme files, overlays and the state (`text.1`, `header.band.bg`).
            pub fn name(self) -> &'static str {
                match self { $( Token::$v => $name ),* }
            }
            pub fn from_name(n: &str) -> Option<Token> {
                match n { $( $name => Some(Token::$v), )* _ => None }
            }
            /// ANSI-16 foreground (None = terminal default) and the modifiers that stand in for colour in
            /// ANSI and NO_COLOR.
            pub fn ansi(self) -> (Option<Color>, Modifier) {
                match self { $( Token::$v => ($ansi, $amod) ),* }
            }
        }
    };
}

const NONE: Modifier = Modifier::empty();

tokens! {
    // semantic (§1.1)
    Bg = "bg", None, NONE;
    Surface = "surface.top", None, NONE;
    SurfaceTabs = "surface.tabs", None, NONE;
    SurfaceSection = "surface.section", None, NONE;
    Raised = "surface.raised", None, NONE;
    SurfaceFoot = "surface.foot", None, NONE;
    Selection = "selection", None, Modifier::REVERSED;
    Line = "rule", None, Modifier::DIM;
    Text = "text.1", Some(Color::Reset), NONE;
    Text2 = "text.2", Some(Color::Reset), NONE;
    Muted = "text.3", None, Modifier::DIM;
    Text4 = "text.4", None, Modifier::DIM;
    Accent = "accent", Some(Color::Reset), Modifier::BOLD;
    AccentTint = "accent.tint", None, Modifier::UNDERLINED;
    Ok = "ok", Some(Color::Green), NONE;
    Warn = "warn", Some(Color::Yellow), NONE;
    Error = "err", Some(Color::Red), NONE;
    Info = "agent", Some(Color::LightBlue), NONE;
    Key = "key", Some(Color::Reset), Modifier::BOLD;
    OkTint = "ok.tint", None, NONE;
    WarnTint = "warn.tint", None, NONE;
    ErrorTint = "err.tint", None, NONE;
    // component (§1.2; knob-driven tints §2.6)
    HeaderBand = "header.band.bg", None, NONE;
    TabsBand = "tabs.bg", None, NONE;
    SectionBand = "section.band.bg", None, NONE;
    BreadcrumbBand = "breadcrumb.bg", None, NONE;
    RowSelected = "row.selected.bg", None, Modifier::REVERSED;
    StatusWarnRow = "status.warn.row", None, NONE;
    StatusErrRow = "status.err.row", None, NONE;
    FooterBand = "footer.bg", None, NONE;
    // manifests: the proposed cue (chip, glyphs, dashed hairline, Δ) and the proposed-row tint
    Proposed = "proposed", Some(Color::Magenta), Modifier::BOLD;
    ProposedTint = "proposed.tint", None, NONE;
    // the scope's members' own accents (pair chips, Compare columns) and their column tints
    Member1 = "member.1", Some(Color::Reset), Modifier::BOLD;
    Member2 = "member.2", Some(Color::Reset), Modifier::BOLD;
    Member3 = "member.3", Some(Color::Reset), Modifier::BOLD;
    Member4 = "member.4", Some(Color::Reset), Modifier::BOLD;
    MemberCol1 = "member.1.column", None, NONE;
    MemberCol2 = "member.2.column", None, NONE;
    MemberCol3 = "member.3.column", None, NONE;
    MemberCol4 = "member.4.column", None, NONE;
}

impl Token {
    /// The member accent token for the i-th member (0-based), and its column tint.
    pub fn member(i: usize) -> Token {
        [
            Token::Member1,
            Token::Member2,
            Token::Member3,
            Token::Member4,
        ][i.min(3)]
    }
    pub fn member_column(i: usize) -> Token {
        [
            Token::MemberCol1,
            Token::MemberCol2,
            Token::MemberCol3,
            Token::MemberCol4,
        ][i.min(3)]
    }
    /// A background token (painted, never text).
    pub fn is_surface(self) -> bool {
        matches!(
            self,
            Token::Bg
                | Token::Surface
                | Token::SurfaceTabs
                | Token::SurfaceSection
                | Token::Raised
                | Token::SurfaceFoot
                | Token::Selection
                | Token::AccentTint
                | Token::OkTint
                | Token::WarnTint
                | Token::ErrorTint
                | Token::HeaderBand
                | Token::TabsBand
                | Token::SectionBand
                | Token::BreadcrumbBand
                | Token::RowSelected
                | Token::StatusWarnRow
                | Token::StatusErrRow
                | Token::FooterBand
                | Token::ProposedTint
                | Token::MemberCol1
                | Token::MemberCol2
                | Token::MemberCol3
                | Token::MemberCol4
        )
    }
}

/// Badges (inverse). Truecolor/256: fg = Bg token, bg = the badge token.
/// ANSI-16: REVERSED + the colour. NO_COLOR: REVERSED + BOLD.
pub const BADGE_READ_ONLY: Token = Token::Ok; // "READ-ONLY"
pub const BADGE_OFFLINE: Token = Token::Warn; // "OFFLINE"

/// Text-level styles (not colours). Applied in every mode.
pub mod styles {
    use ratatui::style::Modifier;
    pub const HEADING: Modifier = Modifier::BOLD; // section headings, active tab, total rows
    pub const TOTAL_ROW: Modifier = Modifier::BOLD; // plus a `line` rule above, drawn by the table
    pub const MATCH: Modifier = Modifier::BOLD; // picker matched letters (+ accent fg; ANSI: + UNDERLINED)
    pub const FOCUSED_BUTTON: Modifier = Modifier::REVERSED; // confirm band: the focused button (Cancel by default)
}

/// Glyphs (spec §2.3). Layout code must measure the chosen string.
pub struct Glyphs {
    pub cursor: &'static str,
    pub ok: &'static str,
    pub flag: &'static str,
    pub ne: &'static str,
    pub cite: &'static str,
    pub sep: &'static str,
    pub crumb: &'static str,
    pub ellipsis: &'static str,
    pub scope: &'static str,
    pub arrow: &'static str,
    pub rule: &'static str,
    pub rule_heavy: &'static str,
    pub vsep: &'static str,
}

#[rustfmt::skip]
pub const UNICODE: Glyphs = Glyphs { cursor: "▌", ok: "✓", flag: "!", ne: "≠", cite: "⌁", sep: "·", crumb: "›",
    ellipsis: "…", scope: "▾", arrow: "→", rule: "─", rule_heavy: "━", vsep: "│" };
#[rustfmt::skip]
pub const ASCII: Glyphs = Glyphs { cursor: ">", ok: "ok", flag: "!", ne: "!=", cite: "<-", sep: "-", crumb: ">",
    ellipsis: "...", scope: "v", arrow: "->", rule: "-", rule_heavy: "=", vsep: "|" };

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_round_trip() {
        for t in Token::ALL {
            assert_eq!(Token::from_name(t.name()), Some(*t));
        }
        assert_eq!(Token::ALL.len(), Token::COUNT);
    }

    #[test]
    fn ansi_never_blue_or_light_yellow() {
        for t in Token::ALL {
            assert!(
                !matches!(t.ansi().0, Some(Color::Blue) | Some(Color::LightYellow)),
                "{t:?}"
            );
        }
    }
}
