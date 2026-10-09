//! Theme definitions and resolution (theme spec v2 §1, §2.6, §2.7, §3).
//!
//! Three layers, each referring only to the one below: primitives (`[palette]`, hex), semantic (`bg`,
//! `text.1`, … as expressions over the palette) and component (`header.band.bg`, … as expressions over the
//! semantic layer and the intensity knobs). Expressions are `#rrggbb`, `@palette.name`, `$token.name` and
//! `mix(a, b, t)` with `t` a number in 0..1 or `knob.<name>`; nothing else, so there is no scripting.
//!
//! Resolution order: built-in theme → user theme file (`extends` one built-in) → `tui.accent_intensity`
//! knobs → the scope accent (family of the scope, plus the members' families) → the overlay (session-only
//! tokens from the canonical state) → a `Pal`, the resolved colour of every token. Colours are never stored:
//! the Pal is recomputed from these inputs whenever one changes, so an imported state draws the same frame.

use super::color::{self, Rgb};
use super::tokens::Token;
use serde_json::Value;
use std::collections::BTreeMap;

/// Per mille as a decimal: 100 → `0.10`, 45 → `0.045`.
pub fn per_mille_text(v: u16) -> String {
    if v.is_multiple_of(10) {
        format!("{}.{:02}", v / 1000, (v % 1000) / 10)
    } else {
        format!("{}.{:03}", v / 1000, v % 1000)
    }
}

/// The intensity knobs (§2.6), in per mille of the accent mixed into a surface (0 = off).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Knobs {
    pub header_band: u16,
    pub section_band: u16,
    pub breadcrumb: u16,
    pub selection: u16,
    pub compare_columns: u16,
    pub status_warn: u16,
    pub status_err: u16,
    /// The header band fades from `header_band` at the left edge to 0 at the right.
    pub header_gradient: bool,
    /// Pair views: the header's left half in org A's accent, the right half in org B's.
    pub pair_split: bool,
    /// Manifests: the proposed-row tint (`proposed.tint`).
    pub proposed_rows: u16,
}

/// The knob names, as `tui.accent_intensity` and overlays (`knob.<name>`) spell them.
pub const KNOB_NAMES: [&str; 10] = [
    "header_band",
    "header_gradient",
    "section_band",
    "pair_band",
    "breadcrumb",
    "selection",
    "compare_columns",
    "status_warn",
    "status_err",
    "proposed_rows",
];

impl Knobs {
    /// The design defaults (dark / light differ: light surfaces tolerate less).
    pub fn defaults(dark: bool) -> Knobs {
        Knobs {
            header_band: if dark { 100 } else { 60 },
            selection: 40,
            proposed_rows: 60,
            ..Knobs::default()
        }
    }

    /// The ceilings (§2.6), COMPUTED: for each knob, the largest value (in steps of 0.01) where every text
    /// token that can sit on that surface (`text.1`–`text.3`, the accent, ok, warn, err, agent, key) stays
    /// ≥ 4.5:1 and the accent as a stroke ≥ 3:1, for every accent family, in the built-in theme. Computed once.
    pub fn ceilings(dark: bool) -> Knobs {
        static D: std::sync::OnceLock<Knobs> = std::sync::OnceLock::new();
        static L: std::sync::OnceLock<Knobs> = std::sync::OnceLock::new();
        *(if dark { &D } else { &L }).get_or_init(|| compute_ceilings(dark))
    }

    pub fn get(&self, name: &str) -> Option<u16> {
        Some(match name {
            "header_band" => self.header_band,
            "section_band" => self.section_band,
            "breadcrumb" => self.breadcrumb,
            "selection" => self.selection,
            "compare_columns" => self.compare_columns,
            "status_warn" => self.status_warn,
            "status_err" => self.status_err,
            "proposed_rows" => self.proposed_rows,
            "header_gradient" => u16::from(self.header_gradient),
            "pair_band" => u16::from(self.pair_split),
            _ => return None,
        })
    }

    /// One knob as state data: a decimal string (`"0.10"`), a bool, or `neutral` / `split`.
    pub fn text(&self, name: &str) -> Value {
        match name {
            "header_gradient" => Value::Bool(self.header_gradient),
            "pair_band" => Value::String(if self.pair_split { "split" } else { "neutral" }.into()),
            n => self
                .get(n)
                .map_or(Value::Null, |v| Value::String(per_mille_text(v))),
        }
    }

    /// Set one knob from its text form: a fraction (`0.04`), a bool for `header_gradient`, or `neutral` /
    /// `split` for `pair_band`.
    pub fn set(&mut self, name: &str, value: &str) -> Result<(), String> {
        let v = value.trim();
        match name {
            "header_gradient" => {
                self.header_gradient = parse_bool(v).ok_or(format!("{name}: true or false"))?;
            }
            "pair_band" => {
                self.pair_split = match v {
                    "split" => true,
                    "neutral" => false,
                    _ => return Err(format!("{name}: neutral or split")),
                }
            }
            _ => {
                let p = per_mille(v).ok_or(format!("{name}: a number from 0 to 1"))?;
                let slot = match name {
                    "header_band" => &mut self.header_band,
                    "section_band" => &mut self.section_band,
                    "breadcrumb" => &mut self.breadcrumb,
                    "selection" => &mut self.selection,
                    "compare_columns" => &mut self.compare_columns,
                    "status_warn" => &mut self.status_warn,
                    "status_err" => &mut self.status_err,
                    "proposed_rows" => &mut self.proposed_rows,
                    _ => return Err(format!("unknown knob {name}")),
                };
                *slot = p;
            }
        }
        Ok(())
    }

    /// Knobs above their ceiling (applied anyway; a warning in System › Settings and `--check-theme`).
    pub fn over_ceiling(&self, dark: bool) -> Vec<&'static str> {
        let c = Knobs::ceilings(dark);
        KNOB_NAMES
            .iter()
            .filter(|n| !matches!(**n, "header_gradient" | "pair_band"))
            .filter(|n| self.get(n) > c.get(n))
            .copied()
            .collect()
    }
}

fn parse_bool(v: &str) -> Option<bool> {
    match v {
        "true" | "on" | "1" => Some(true),
        "false" | "off" | "0" => Some(false),
        _ => None,
    }
}

/// `0.04` → 40 (per mille), exact decimal parsing (no floats); `1` → 1000.
pub fn per_mille(v: &str) -> Option<u16> {
    let v = v.trim();
    let (int, frac) = v.split_once('.').unwrap_or((v, ""));
    if int.is_empty() && frac.is_empty() {
        return None;
    }
    let int: u32 = if int.is_empty() { 0 } else { int.parse().ok()? };
    if !frac.bytes().all(|b| b.is_ascii_digit()) || frac.len() > 3 {
        return None;
    }
    let mut f = frac.to_string();
    while f.len() < 3 {
        f.push('0');
    }
    let f: u32 = f.parse().ok()?;
    let p = int * 1000 + f;
    (p <= 1000).then_some(p as u16)
}

/// The texts that can sit on a tinted surface (§4) and the knob → surface it tints.
const TEXTS: [Token; 9] = [
    Token::Text,
    Token::Text2,
    Token::Muted,
    Token::Accent,
    Token::Ok,
    Token::Warn,
    Token::Error,
    Token::Info,
    Token::Key,
];

#[allow(clippy::float_arithmetic)]
fn passes(p: &Pal, surface: Token, texts: &[Token]) -> bool {
    texts
        .iter()
        .all(|t| color::contrast(p.get(*t), p.get(surface)) >= 4.5)
        && color::contrast(p.get(Token::Text4), p.get(surface)) >= 3.0
}

fn compute_ceilings(dark: bool) -> Knobs {
    let def = ledger(dark);
    let fams = ["harbor", "ember", "iris", "rose", "sea"];
    let mut out = Knobs {
        header_gradient: true,
        pair_split: true,
        ..Knobs::default()
    };
    let members: Vec<String> = fams[1..].iter().map(|s| s.to_string()).collect();
    for name in [
        "header_band",
        "section_band",
        "breadcrumb",
        "selection",
        "compare_columns",
        "status_warn",
        "status_err",
        "proposed_rows",
    ] {
        let (surfaces, texts): (Vec<Token>, &[Token]) = match name {
            "header_band" => (vec![Token::HeaderBand], &TEXTS),
            "section_band" => (vec![Token::SectionBand], &TEXTS),
            "breadcrumb" => (vec![Token::BreadcrumbBand], &TEXTS),
            "selection" => (vec![Token::RowSelected], &TEXTS),
            // a Compare column holds amounts and labels, never a status word
            "compare_columns" => (
                (0..4).map(Token::member_column).collect(),
                &[Token::Text, Token::Text2, Token::Muted],
            ),
            "status_warn" => (
                vec![Token::StatusWarnRow],
                &[Token::Text, Token::Muted, Token::Warn],
            ),
            // a proposed row: its text, labels, the proposed glyph, and any status word in it
            "proposed_rows" => (
                vec![Token::ProposedTint],
                &[
                    Token::Text,
                    Token::Muted,
                    Token::Proposed,
                    Token::Warn,
                    Token::Error,
                ],
            ),
            _ => (
                vec![Token::StatusErrRow],
                &[Token::Text, Token::Muted, Token::Error],
            ),
        };
        let mut best = 0;
        let mut t = 0;
        while t <= 400 {
            let mut k = Knobs::defaults(dark);
            let _ = k.set(name, &format!("0.{t:03}"));
            let ok = fams.iter().all(|f| {
                let p = resolve(&def, k, f, &members, &BTreeMap::new());
                surfaces.iter().all(|s| passes(&p, *s, texts))
            });
            if !ok {
                break;
            }
            best = t;
            t += 10;
        }
        let _ = out.set(name, &format!("0.{best:03}"));
    }
    out
}

/// A theme: its palette, semantic and component expressions, and its accent families.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ThemeDef {
    pub name: String,
    pub dark: bool,
    pub palette: BTreeMap<String, Rgb>,
    pub semantic: BTreeMap<String, String>,
    pub component: BTreeMap<String, String>,
    pub accents: BTreeMap<String, Rgb>,
}

/// The built-in theme names.
pub const BUILTINS: [&str; 2] = ["ledger-dark", "ledger-light"];

pub fn builtin(name: &str) -> Option<ThemeDef> {
    match name {
        "ledger-dark" => Some(ledger(true)),
        "ledger-light" => Some(ledger(false)),
        _ => None,
    }
}

fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

fn hexmap(pairs: &[(&str, Rgb)]) -> BTreeMap<String, Rgb> {
    pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
}

/// The component defaults (§1.2), the same in both built-ins.
fn components() -> BTreeMap<String, String> {
    map(&[
        (
            "header.band.bg",
            "mix($surface.top, $accent, knob.header_band)",
        ),
        ("tabs.bg", "$surface.tabs"),
        (
            "section.band.bg",
            "mix($surface.section, $accent, knob.section_band)",
        ),
        ("breadcrumb.bg", "mix($bg, $accent, knob.breadcrumb)"),
        (
            "row.selected.bg",
            "mix($selection, $accent, knob.selection)",
        ),
        ("status.warn.row", "mix($bg, $warn, knob.status_warn)"),
        ("status.err.row", "mix($bg, $err, knob.status_err)"),
        ("footer.bg", "$surface.foot"),
        ("accent.tint", "mix($bg, $accent, 0.16)"),
        ("ok.tint", "mix($bg, $ok, 0.14)"),
        ("warn.tint", "mix($bg, $warn, 0.14)"),
        ("err.tint", "mix($bg, $err, 0.14)"),
        ("proposed.tint", "mix($bg, $proposed, knob.proposed_rows)"),
        (
            "member.1.column",
            "mix($bg, $member.1, knob.compare_columns)",
        ),
        (
            "member.2.column",
            "mix($bg, $member.2, knob.compare_columns)",
        ),
        (
            "member.3.column",
            "mix($bg, $member.3, knob.compare_columns)",
        ),
        (
            "member.4.column",
            "mix($bg, $member.4, knob.compare_columns)",
        ),
    ])
}

/// ledger-dark / ledger-light: the semantic table of §1.1 (with the text.3 / text.4 fix) and the accent
/// families of §2.1.
fn ledger(dark: bool) -> ThemeDef {
    let (palette, accents) = if dark {
        (
            hexmap(&[
                ("slate.950", 0x14191C),
                ("slate.925", 0x171D20),
                ("slate.915", 0x1A2125),
                ("slate.900", 0x1B2226),
                ("slate.880", 0x1C2428),
                ("slate.800", 0x22303A),
                ("slate.700", 0x2D383E),
                ("slate.400", 0x717E85),
                ("slate.300", 0x93A0A7),
                ("slate.200", 0xB9C4C9),
                ("slate.50", 0xE6ECEE),
                ("white", 0xFFFFFF),
                ("green.300", 0x8FCB92),
                ("amber.300", 0xE6BC5E),
                ("coral.300", 0xF0907B),
                ("sky.300", 0x88B1EC),
                ("violet.300", 0xD49AD0),
            ]),
            hexmap(&[
                ("harbor", 0x72B6D4),
                ("ember", 0xE8834F),
                ("iris", 0x9D8FF2),
                ("rose", 0xE8769F),
                ("sea", 0x4DBFA9),
            ]),
        )
    } else {
        (
            hexmap(&[
                ("paper.50", 0xF7F8F8),
                ("paper.75", 0xF1F3F4),
                ("paper.100", 0xEDF1F2),
                ("paper.110", 0xECEFF0),
                ("paper.120", 0xEEF1F2),
                ("paper.200", 0xDFE9EE),
                ("paper.300", 0xC9D1D4),
                ("ink.500", 0x768186),
                ("ink.600", 0x566369),
                ("ink.700", 0x3D4A50),
                ("ink.900", 0x162024),
                ("green.700", 0x2E6B39),
                ("amber.700", 0x805800),
                ("coral.700", 0xB23A24),
                ("sky.700", 0x2D5FA3),
                ("violet.700", 0x874083),
            ]),
            hexmap(&[
                ("harbor", 0x1D6A88),
                ("ember", 0xA8461A),
                ("iris", 0x5340B0),
                ("rose", 0xA33462),
                ("sea", 0x17705F),
            ]),
        )
    };
    let semantic = if dark {
        map(&[
            ("bg", "@slate.950"),
            ("surface.top", "@slate.900"),
            ("surface.tabs", "@slate.925"),
            ("surface.section", "@slate.915"),
            ("surface.raised", "@slate.880"),
            ("surface.foot", "@slate.900"),
            ("selection", "@slate.800"),
            ("rule", "@slate.700"),
            ("text.1", "@slate.50"),
            ("text.2", "@slate.200"),
            ("text.3", "@slate.300"),
            ("text.4", "@slate.400"),
            ("ok", "@green.300"),
            ("warn", "@amber.300"),
            ("err", "@coral.300"),
            ("agent", "@sky.300"),
            ("proposed", "@violet.300"),
            ("key", "@white"),
        ])
    } else {
        map(&[
            ("bg", "@paper.50"),
            ("surface.top", "@paper.110"),
            ("surface.tabs", "@paper.75"),
            ("surface.section", "@paper.100"),
            ("surface.raised", "@paper.120"),
            ("surface.foot", "@paper.110"),
            ("selection", "@paper.200"),
            ("rule", "@paper.300"),
            ("text.1", "@ink.900"),
            ("text.2", "@ink.700"),
            ("text.3", "@ink.600"),
            ("text.4", "@ink.500"),
            ("ok", "@green.700"),
            ("warn", "@amber.700"),
            ("err", "@coral.700"),
            ("agent", "@sky.700"),
            ("proposed", "@violet.700"),
            ("key", "@ink.900"),
        ])
    };
    ThemeDef {
        name: if dark { "ledger-dark" } else { "ledger-light" }.into(),
        dark,
        palette,
        semantic,
        component: components(),
        accents,
    }
}

impl ThemeDef {
    /// A user theme over a built-in (`extends`, one level): palette, semantic, component and accent entries
    /// replace the built-in's.
    pub fn extend(base: &ThemeDef, name: &str, over: &ThemeDef) -> ThemeDef {
        let mut t = base.clone();
        t.name = name.to_string();
        t.palette.extend(over.palette.clone());
        t.semantic.extend(over.semantic.clone());
        t.component.extend(over.component.clone());
        t.accents.extend(over.accents.clone());
        t
    }

    /// Every expression's error (`section.key = "expr": why`), checked by resolving with each family. Empty:
    /// the theme is valid.
    pub fn errors(&self) -> Vec<String> {
        let mut out = Vec::new();
        for (layer, m) in [("semantic", &self.semantic), ("component", &self.component)] {
            for (k, e) in m {
                if Token::from_name(k).is_none() {
                    out.push(format!(
                        "{layer}.{k}: unknown token{}",
                        suggest(k, Token::ALL.iter().map(|t| t.name()))
                    ));
                    continue;
                }
                let none = BTreeMap::new();
                let ctx = Ctx::new(self, Knobs::defaults(self.dark), "harbor", &[], &none);
                match ctx.eval(e, 0) {
                    // an error in a token this one refers to is reported once, at that token
                    Err(why) if !why.starts_with("via $") => {
                        out.push(format!("{layer}.{k} = \"{e}\": {why}"))
                    }
                    _ => {}
                }
            }
        }
        out
    }
}

/// The resolved colour of every token, plus the knobs and how many members the scope has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pal {
    /// Resolved for a dark theme (a Pal for the other mode is ignored until the next repaint).
    pub dark: bool,
    pub c: [Rgb; Token::COUNT],
    pub knobs: Knobs,
    pub members: u8,
}

impl Default for Pal {
    fn default() -> Pal {
        resolve(
            &ledger(true),
            Knobs::defaults(true),
            "harbor",
            &[],
            &BTreeMap::new(),
        )
    }
}

impl Pal {
    pub fn get(&self, t: Token) -> Rgb {
        self.c[t as usize]
    }
}

/// Resolve every token for these inputs. Invalid overlay values are skipped (they're refused when set).
pub fn resolve(
    def: &ThemeDef,
    knobs: Knobs,
    accent: &str,
    members: &[String],
    overlay: &BTreeMap<String, String>,
) -> Pal {
    let mut knobs = knobs;
    for (k, v) in overlay {
        if let Some(n) = k.strip_prefix("knob.") {
            let _ = knobs.set(n, v);
        }
    }
    let ctx = Ctx::new(def, knobs, accent, members, overlay);
    let mut c = [0; Token::COUNT];
    for t in Token::ALL {
        c[*t as usize] = ctx.token(t.name(), 0).unwrap_or(0);
    }
    Pal {
        dark: def.dark,
        c,
        knobs,
        members: members.len().min(4) as u8,
    }
}

struct Ctx<'a> {
    def: &'a ThemeDef,
    knobs: Knobs,
    accent: &'a str,
    members: &'a [String],
    overlay: &'a BTreeMap<String, String>,
}

impl<'a> Ctx<'a> {
    fn new(
        def: &'a ThemeDef,
        knobs: Knobs,
        accent: &'a str,
        members: &'a [String],
        overlay: &'a BTreeMap<String, String>,
    ) -> Ctx<'a> {
        Ctx {
            def,
            knobs,
            accent,
            members,
            overlay,
        }
    }

    /// An accent family's colour in this theme (`graphite` = `text.1`).
    fn family(&self, f: &str, depth: u8) -> Result<Rgb, String> {
        if f == "graphite" {
            return self.token("text.1", depth + 1);
        }
        self.def.accents.get(f).copied().ok_or_else(|| {
            format!(
                "unknown accent {f:?}{}",
                suggest(f, self.def.accents.keys().map(String::as_str))
            )
        })
    }

    fn token(&self, name: &str, depth: u8) -> Result<Rgb, String> {
        if depth > 24 {
            return Err(format!("{name}: expressions refer to each other in a loop"));
        }
        if let Some(v) = self.overlay.get(name) {
            // an accent overlay may name a family
            if (name == "accent" || name.starts_with("member.")) && super::accent::is_family(v) {
                return self.family(v, depth);
            }
            if let Ok(c) = self.eval(v, depth + 1) {
                return Ok(c);
            }
        }
        match name {
            "accent" => return self.family(self.accent, depth),
            n if n.starts_with("member.") && !n.ends_with(".column") => {
                let i: usize = n[7..].parse().map_err(|_| format!("unknown token {n}"))?;
                return match self.members.get(i.saturating_sub(1)) {
                    Some(f) => self.family(f, depth),
                    None => self.family(self.accent, depth),
                };
            }
            _ => {}
        }
        let e = self
            .def
            .semantic
            .get(name)
            .or_else(|| self.def.component.get(name))
            .ok_or_else(|| {
                format!(
                    "unknown token {name:?}{}",
                    suggest(name, Token::ALL.iter().map(|t| t.name()))
                )
            })?;
        self.eval(e, depth + 1)
    }

    fn eval(&self, e: &str, depth: u8) -> Result<Rgb, String> {
        let e = e.trim();
        if let Some(h) = e.strip_prefix('#') {
            return color::parse_hex(h).ok_or_else(|| format!("{e:?} is not #rrggbb"));
        }
        if let Some(p) = e.strip_prefix('@') {
            if let Some(f) = p.strip_prefix("accent.") {
                return self.family(f, depth);
            }
            return self.def.palette.get(p).copied().ok_or_else(|| {
                format!(
                    "unknown palette entry{}",
                    suggest(p, self.def.palette.keys().map(String::as_str))
                )
            });
        }
        if let Some(t) = e.strip_prefix('$') {
            return self.token(t, depth).map_err(|why| {
                if why.starts_with("via $") || why.starts_with("unknown token") {
                    why
                } else {
                    format!("via ${t}: {why}")
                }
            });
        }
        if let Some(args) = e.strip_prefix("mix(").and_then(|r| r.strip_suffix(')')) {
            let parts = split_args(args);
            let [a, b, t] = parts.as_slice() else {
                return Err("mix takes three arguments: mix(a, b, t)".into());
            };
            let t = match t.trim().strip_prefix("knob.") {
                Some(k) => self.knobs.get(k).ok_or(format!("unknown knob {k:?}"))?,
                None => per_mille(t).ok_or(format!("{t:?} is not a number from 0 to 1"))?,
            };
            return Ok(color::mix(
                self.eval(a, depth + 1)?,
                self.eval(b, depth + 1)?,
                t,
            ));
        }
        Err(format!(
            "{e:?}: expected #hex, @palette, $token or mix(a, b, t)"
        ))
    }
}

/// Split `a, mix(b, c, 0.1), 0.2` at top-level commas.
fn split_args(s: &str) -> Vec<String> {
    let (mut out, mut cur, mut depth) = (Vec::new(), String::new(), 0i32);
    for ch in s.chars() {
        match ch {
            '(' => {
                depth += 1;
                cur.push(ch);
            }
            ')' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => out.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    out.push(cur);
    out
}

/// ` (did you mean "slate.950"?)` for the closest name sharing a prefix, else nothing.
pub fn suggest<'a>(got: &str, names: impl Iterator<Item = &'a str>) -> String {
    let best = names
        .filter(|n| n.starts_with(got) || got.starts_with(*n) || common_prefix(n, got) >= 3)
        .max_by_key(|n| common_prefix(n, got));
    match best {
        Some(n) => format!(" (did you mean \"{n}\"?)"),
        None => String::new(),
    }
}

fn common_prefix(a: &str, b: &str) -> usize {
    a.chars().zip(b.chars()).take_while(|(x, y)| x == y).count()
}

/// Validate one overlay entry (`token=value`): the token name and the value's form. `bad_token` otherwise.
pub fn check_overlay(def: &ThemeDef, key: &str, value: &str) -> Result<(), String> {
    if let Some(k) = key.strip_prefix("knob.") {
        return Knobs::default().set(k, value).map_err(|e| {
            if e.starts_with("unknown knob") {
                format!("unknown knob {k:?}; knobs: {}", KNOB_NAMES.join(", "))
            } else {
                e
            }
        });
    }
    let Some(t) = Token::from_name(key) else {
        let prefix = key.split('.').next().unwrap_or(key);
        let valid: Vec<&str> = Token::ALL
            .iter()
            .map(|t| t.name())
            .filter(|n| n.starts_with(prefix))
            .collect();
        return Err(format!(
            "unknown token {key:?}; valid: {}",
            if valid.is_empty() {
                Token::ALL
                    .iter()
                    .map(|t| t.name())
                    .collect::<Vec<_>>()
                    .join(", ")
            } else {
                valid.join(", ")
            }
        ));
    };
    if (t == Token::Accent || t.name().starts_with("member.")) && super::accent::is_family(value) {
        return Ok(());
    }
    if (t == Token::Accent || t.name().starts_with("member."))
        && value.chars().all(|c| c.is_ascii_alphabetic())
    {
        return Err(format!(
            "unknown accent {value:?}{}; families: {}",
            suggest(value, super::accent::FAMILIES.iter().copied()),
            super::accent::FAMILIES.join(", ")
        ));
    }
    let mut o = BTreeMap::new();
    o.insert(key.to_string(), value.to_string());
    Ctx::new(def, Knobs::defaults(def.dark), "harbor", &[], &o)
        .eval(value, 0)
        .map(|_| ())
}

#[cfg(test)]
#[allow(clippy::float_arithmetic)]
mod tests {
    use super::*;
    use crate::theme::color::contrast;

    fn pal(dark: bool, accent: &str, knobs: Knobs) -> Pal {
        resolve(&ledger(dark), knobs, accent, &[], &BTreeMap::new())
    }

    #[test]
    fn builtins_are_valid_and_resolve_the_spec_values() {
        for d in [true, false] {
            assert!(ledger(d).errors().is_empty(), "{:?}", ledger(d).errors());
        }
        let p = pal(true, "harbor", Knobs::defaults(true));
        assert_eq!(p.get(Token::Bg), 0x14191C);
        assert_eq!(p.get(Token::Muted), 0x93A0A7);
        assert_eq!(p.get(Token::Accent), 0x72B6D4);
        assert_eq!(
            p.get(Token::HeaderBand),
            color::mix(0x1B2226, 0x72B6D4, 100)
        );
        let p = pal(false, "ember", Knobs::defaults(false));
        assert_eq!(p.get(Token::Accent), 0xA8461A);
        assert_eq!(p.get(Token::HeaderBand), color::mix(0xECEFF0, 0xA8461A, 60));
    }

    #[test]
    fn per_mille_parses_exactly() {
        assert_eq!(per_mille("0.04"), Some(40));
        assert_eq!(per_mille("0.1"), Some(100));
        assert_eq!(per_mille("1"), Some(1000));
        assert_eq!(per_mille(".5"), Some(500));
        assert_eq!(per_mille("1.5"), None);
        assert_eq!(per_mille("0.0001"), None);
        assert_eq!(per_mille("x"), None);
    }

    #[test]
    fn extends_and_expressions() {
        let mut over = ThemeDef::default();
        over.palette.insert("slate.950".into(), 0x161412);
        over.semantic.insert("bg".into(), "@slate.950".into());
        over.component.insert(
            "section.band.bg".into(),
            "mix($surface.section, $accent, 0.04)".into(),
        );
        let t = ThemeDef::extend(&ledger(true), "warm", &over);
        assert!(t.errors().is_empty());
        let p = resolve(&t, Knobs::defaults(true), "ember", &[], &BTreeMap::new());
        assert_eq!(p.get(Token::Bg), 0x161412);
        assert_eq!(
            p.get(Token::SectionBand),
            color::mix(0x1A2125, 0xE8834F, 40)
        );
        // errors name the key, the expression and a suggestion
        let mut bad = ThemeDef::default();
        bad.semantic.insert("bg".into(), "@slate.95".into());
        let t = ThemeDef::extend(&ledger(true), "bad", &bad);
        assert_eq!(
            t.errors(),
            vec![
                "semantic.bg = \"@slate.95\": unknown palette entry (did you mean \"slate.950\"?)"
            ]
        );
    }

    #[test]
    fn overlay_wins_and_accent_takes_a_family() {
        let mut o = BTreeMap::new();
        o.insert("accent".into(), "iris".into());
        o.insert("header.band.bg".into(), "#3a2a20".into());
        o.insert("knob.selection".into(), "0".into());
        let p = resolve(&ledger(true), Knobs::defaults(true), "ember", &[], &o);
        assert_eq!(p.get(Token::Accent), 0x9D8FF2);
        assert_eq!(p.get(Token::HeaderBand), 0x3A2A20);
        assert_eq!(p.get(Token::RowSelected), 0x22303A);
        assert!(check_overlay(&ledger(true), "accent", "iris").is_ok());
        assert!(check_overlay(&ledger(true), "header.band.bg", "mix($bg, $accent, 0.3)").is_ok());
        assert!(
            check_overlay(&ledger(true), "header.bnad.bg", "#000000")
                .unwrap_err()
                .contains("header.band.bg")
        );
        assert!(check_overlay(&ledger(true), "knob.selection", "2").is_err());
        assert!(check_overlay(&ledger(true), "text.1", "#12").is_err());
    }

    /// The text surfaces of §4 at the given knobs.
    fn surfaces(p: &Pal) -> Vec<(&'static str, Rgb)> {
        vec![
            ("bg", p.get(Token::Bg)),
            ("surface.top", p.get(Token::Surface)),
            ("header.band", p.get(Token::HeaderBand)),
            ("surface.section", p.get(Token::SectionBand)),
            ("selection", p.get(Token::RowSelected)),
            ("surface.raised", p.get(Token::Raised)),
            ("surface.foot", p.get(Token::SurfaceFoot)),
            ("breadcrumb", p.get(Token::BreadcrumbBand)),
        ]
    }

    /// The contrast table (§4, V22): every built-in × every accent family × every surface, at the default
    /// knobs and at every knob's ceiling.
    pub fn contrast_rows(knobs_at: &str) -> Vec<(String, f64)> {
        let mut rows = Vec::new();
        for dark in [true, false] {
            let knobs = if knobs_at == "ceiling" {
                Knobs::ceilings(dark)
            } else {
                Knobs::defaults(dark)
            };
            for fam in ["harbor", "ember", "iris", "rose", "sea"] {
                let p = pal(dark, fam, knobs);
                for (sn, s) in surfaces(&p) {
                    for (tn, t) in [
                        ("text.1", Token::Text),
                        ("text.2", Token::Text2),
                        ("text.3", Token::Muted),
                        ("accent", Token::Accent),
                        ("ok", Token::Ok),
                        ("warn", Token::Warn),
                        ("err", Token::Error),
                        ("agent", Token::Info),
                        ("key", Token::Key),
                    ] {
                        rows.push((
                            format!("{} {fam} {tn} on {sn}", if dark { "dark" } else { "light" }),
                            contrast(p.get(t), s),
                        ));
                    }
                }
            }
        }
        rows
    }

    #[test]
    fn v22_contrast_at_defaults_and_ceilings() {
        let mut bad = Vec::new();
        for at in ["default", "ceiling"] {
            for (what, r) in contrast_rows(at) {
                if r < 4.5 {
                    bad.push(format!("{at}: {what} = {r:.2}:1"));
                }
            }
        }
        assert!(bad.is_empty(), "{}", bad.join("\n"));
        // text.4 is exempt by name but stays ≥ 3:1; the gutter (accent as a stroke) ≥ 3:1
        for dark in [true, false] {
            for at in [Knobs::defaults(dark), Knobs::ceilings(dark)] {
                let p = pal(dark, "harbor", at);
                for (sn, s) in surfaces(&p) {
                    assert!(contrast(p.get(Token::Text4), s) >= 3.0, "text.4 on {sn}");
                }
            }
        }
    }

    #[test]
    fn status_rows_keep_their_text_at_the_ceiling() {
        for dark in [true, false] {
            let p = pal(dark, "harbor", Knobs::ceilings(dark));
            for (row, t) in [
                (Token::StatusWarnRow, Token::Warn),
                (Token::StatusErrRow, Token::Error),
            ] {
                assert!(contrast(p.get(t), p.get(row)) >= 4.5, "{dark} {row:?}");
                assert!(
                    contrast(p.get(Token::Text), p.get(row)) >= 4.5,
                    "{dark} {row:?}"
                );
                assert!(
                    contrast(p.get(Token::Muted), p.get(row)) >= 4.5,
                    "{dark} {row:?}"
                );
            }
        }
    }
}

#[cfg(test)]
mod ceiling_table {
    /// Prints the computed ceilings next to design's (`cargo test -p ownpurse-tui ceiling_table -- --nocapture`).
    #[test]
    fn print_ceilings() {
        for dark in [true, false] {
            let c = super::Knobs::ceilings(dark);
            println!("{} {c:?}", if dark { "dark" } else { "light" });
        }
    }

    /// The contrast table for the hand-back: per theme, knobs (defaults / ceilings) and text token, the
    /// lowest ratio over every accent family and surface, and where it occurs
    /// (`cargo test -p ownpurse-tui contrast_table -- --ignored --nocapture`).
    #[test]
    #[ignore]
    #[allow(clippy::float_arithmetic)]
    fn contrast_table() {
        for at in ["default", "ceiling"] {
            let rows = super::tests::contrast_rows(at);
            let mut worst: std::collections::BTreeMap<String, (f64, String)> = Default::default();
            for (what, r) in rows {
                let parts: Vec<&str> = what.split(' ').collect();
                let key = format!("{} {}", parts[0], parts[2]);
                let e = worst.entry(key).or_insert((99.0, String::new()));
                if r < e.0 {
                    *e = (r, format!("{} on {}", parts[1], parts[4]));
                }
            }
            for (k, (r, w)) in worst {
                println!("| {at} | {k} | {r:.2}:1 | {w} |");
            }
        }
    }
}
