//! Glyph sets (spec §2.3, from the kit) and the one fold from Unicode UI glyphs to ASCII. Data from Xero
//! (names, contacts) is never folded; only the UI's own glyphs are.

pub use crate::theme_tokens::{ASCII, Glyphs, UNICODE};

pub fn set(ascii: bool) -> &'static Glyphs {
    if ascii { &ASCII } else { &UNICODE }
}

/// The UI glyphs and their ASCII forms (the kit's `ASCII` set plus the arrows and marks the copy uses).
const FOLD: &[(char, &str)] = &[
    ('▌', ">"),
    ('✓', "ok"),
    ('≠', "!="),
    ('⌁', "<-"),
    ('·', "-"),
    ('›', ">"),
    ('…', "..."),
    ('▾', "v"),
    ('→', "->"),
    ('←', "<-"),
    ('─', "-"),
    ('━', "="),
    ('│', "|"),
    ('▸', ">"),
    ('◂', "<"),
    ('↑', "^"),
    ('↓', "v"),
    ('⌃', "^"),
    ('⇧', "S-"),
    ('⌫', "BS"),
    ('×', "x"),
    ('⇅', "<>"),
    ('⇄', "<>"),
    ('↔', "<->"),
    ('●', "*"),
    ('≈', "~"),
    ('⬆', "^"),
    ('✗', "x"),
    ('↵', "Enter"),
    ('—', "-"),
    ('≥', ">="),
    ('▰', "#"),
    ('▱', "-"),
    // manifests: void, minus, delta, the manifest mark, the dashed hairline, the quiet dot, the accountant flag
    ('⊘', "V"),
    ('−', "-"),
    ('Δ', "D"),
    ('⎇', "@"),
    ('╌', "-"),
    ('◦', "o"),
    ('○', "o"),
    ('“', "\""),
    ('”', "\""),
    ('⚑', "!"),
    ('◆', "*"),
];

pub fn fold(s: &str, ascii: bool) -> String {
    if !ascii || s.is_ascii() {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    for (i, &ch) in chars.iter().enumerate() {
        // the quiet ✓ before a word reads as the word alone in plain text (`exact`, `matches Xero`); a
        // bare ✓ stays `ok` (design, v0.12.0)
        if ch == '✓'
            && chars.get(i + 1) == Some(&' ')
            && chars.get(i + 2).is_some_and(|c| c.is_alphabetic())
        {
            continue;
        }
        if ch == ' '
            && i > 0
            && chars[i - 1] == '✓'
            && chars.get(i + 1).is_some_and(|c| c.is_alphabetic())
        {
            continue;
        }
        match FOLD.iter().find(|(c, _)| *c == ch) {
            Some((_, a)) => out.push_str(a),
            None => out.push(ch),
        }
    }
    out
}

/// UTF-8 and not a CJK locale (where East-Asian-ambiguous glyphs may draw double-width).
pub fn utf8_locale(env: &dyn Fn(&str) -> Option<String>) -> bool {
    let locale = ["LC_ALL", "LC_CTYPE", "LANG"]
        .iter()
        .find_map(|k| env(k).filter(|v| !v.is_empty()))
        .unwrap_or_default();
    let lower = locale.to_ascii_lowercase();
    let utf8 = lower.contains("utf-8") || lower.contains("utf8");
    let cjk = ["ja", "zh", "ko"].iter().any(|p| lower.starts_with(p));
    utf8 && !cjk
}

#[cfg(test)]
mod tests {
    use super::*;
    use unicode_width::UnicodeWidthStr;

    #[test]
    fn every_unicode_glyph_is_single_width() {
        let g = &UNICODE;
        for s in [
            g.cursor,
            g.ok,
            g.flag,
            g.ne,
            g.cite,
            g.sep,
            g.crumb,
            g.ellipsis,
            g.scope,
            g.arrow,
            g.rule,
            g.rule_heavy,
            g.vsep,
        ] {
            assert_eq!(UnicodeWidthStr::width(s), 1, "{s}");
        }
        for (c, a) in FOLD {
            assert_eq!(UnicodeWidthStr::width(c.to_string().as_str()), 1, "{c}");
            assert!(a.is_ascii());
        }
    }

    #[test]
    fn fold_keeps_data() {
        assert_eq!(fold("org: one ▾ · 2h", true), "org: one v - 2h");
        assert_eq!(fold("CARD™ 大阪", true), "CARD™ 大阪");
        assert_eq!(fold("▾", false), "▾");
    }
}
