//! User themes (theme spec v2 §3): `<config dir>/themes/<name>.toml`, one level of `extends` over a
//! built-in, with `@palette` / `$token` / `mix()` / `#hex` expressions. Every problem is a `ConfigIssue`
//! with file:line, so the loader keeps the previous theme and the shared error band names the spot.
//!
//! ```toml
//! [meta]
//! name    = "warm"
//! extends = "ledger-dark"
//! [palette]
//! slate.950 = { hex = "#161412" }      # or slate.950 = "#161412"
//! [semantic]
//! bg = "@slate.950"
//! [component]
//! "section.band.bg" = "mix($surface.section, $accent, 0.04)"
//! [accents.ember]
//! dark  = { hex = "#F08A55" }
//! light = { hex = "#A8461A" }
//! ```

use super::color::{self, Rgb};
use super::def::{self, ThemeDef};
use crate::user_config::{ConfigIssue, ThemeFile, line_of};
use std::collections::BTreeMap;

/// The loader's hook (`user_config::ThemeValidator`): the file parses into a valid theme.
pub fn validate(f: &ThemeFile, _all: &BTreeMap<String, ThemeFile>) -> Result<(), ConfigIssue> {
    parse(&f.file, &f.name, &f.text).map(|_| ())
}

/// The resolved theme for a file's text (its `extends` built-in with the file's entries over it).
pub fn parse(file: &str, name: &str, text: &str) -> Result<ThemeDef, ConfigIssue> {
    let issue = |line: usize, path: &str, message: String| ConfigIssue {
        file: file.to_string(),
        line,
        path: path.to_string(),
        message,
    };
    let doc = toml_edit::Document::parse(text.to_string()).map_err(|e| {
        issue(
            e.span().map_or(0, |s| line_of(text, s.start)),
            "(syntax)",
            e.message().to_string(),
        )
    })?;
    let at = |i: &toml_edit::Item| i.span().map_or(0, |s| line_of(text, s.start));
    let at_v = |v: &toml_edit::Value| v.span().map_or(0, |s| line_of(text, s.start));
    let table = |key: &str| -> Result<Option<&toml_edit::Table>, ConfigIssue> {
        match doc.get(key) {
            None => Ok(None),
            Some(toml_edit::Item::Table(t)) => Ok(Some(t)),
            Some(other) => Err(issue(at(other), key, format!("[{key}] must be a table"))),
        }
    };
    let str_of = |v: &toml_edit::Value, path: &str| -> Result<String, ConfigIssue> {
        v.as_str()
            .map(str::to_string)
            .ok_or_else(|| issue(at_v(v), path, "expected a string".into()))
    };
    // a colour: "#rrggbb" or { hex = "#rrggbb" }
    let hex_of = |v: &toml_edit::Value, path: &str| -> Result<Rgb, ConfigIssue> {
        let s = match v {
            toml_edit::Value::InlineTable(t) => t
                .get("hex")
                .and_then(|h| h.as_str())
                .ok_or_else(|| issue(at_v(v), path, "expected { hex = \"#rrggbb\" }".into()))?
                .to_string(),
            other => str_of(other, path)?,
        };
        color::parse_hex(&s).ok_or_else(|| issue(at_v(v), path, format!("{s:?} is not #rrggbb")))
    };

    // [meta]
    let meta = table("meta")?;
    let get_meta = |k: &str| meta.and_then(|m| m.get(k));
    if let Some(i) = get_meta("name") {
        let v = i
            .as_value()
            .ok_or_else(|| issue(at(i), "meta.name", "expected a string".into()))?;
        let n = str_of(v, "meta.name")?;
        if n != name {
            return Err(issue(
                at(i),
                "meta.name",
                format!("{n:?} must match the file name {name:?}"),
            ));
        }
    }
    let base_name = match get_meta("extends") {
        None => "ledger-dark".to_string(),
        Some(i) => {
            let v = i
                .as_value()
                .ok_or_else(|| issue(at(i), "meta.extends", "expected a string".into()))?;
            let b = str_of(v, "meta.extends")?;
            if def::builtin(&b).is_none() {
                return Err(issue(
                    at(i),
                    "meta.extends",
                    format!(
                        "{b:?}: a theme extends a built-in only (one level): {}",
                        def::BUILTINS.join(", ")
                    ),
                ));
            }
            b
        }
    };
    let base = def::builtin(&base_name).unwrap_or_default();
    let mut over = ThemeDef {
        dark: base.dark,
        ..ThemeDef::default()
    };
    // remember each entry's line so an expression error points at it
    let mut lines: BTreeMap<String, usize> = BTreeMap::new();

    if let Some(t) = table("palette")? {
        let mut entries = Vec::new();
        walk(t, "", &mut entries);
        for (k, i) in entries {
            let path = format!("palette.{k}");
            let v = i
                .as_value()
                .ok_or_else(|| issue(at(i), &path, "expected a colour".into()))?;
            over.palette.insert(k.clone(), hex_of(v, &path)?);
        }
    }
    for layer in ["semantic", "component"] {
        let Some(t) = table(layer)? else { continue };
        let mut entries = Vec::new();
        walk(t, "", &mut entries);
        for (k, i) in entries {
            let k = k.as_str();
            let path = format!("{layer}.{k}");
            let v = i
                .as_value()
                .ok_or_else(|| issue(at(i), &path, "expected an expression".into()))?;
            if v.as_float().is_some() || v.as_integer().is_some() {
                return Err(issue(
                    at_v(v),
                    &path,
                    "intensities are set in ownpurse.json tui.accent_intensity, not in a theme"
                        .into(),
                ));
            }
            let e = str_of(v, &path)?;
            if crate::theme_tokens::Token::from_name(k).is_none() {
                return Err(issue(
                    at(i),
                    &path,
                    format!(
                        "unknown token{}",
                        def::suggest(k, crate::theme_tokens::Token::ALL.iter().map(|t| t.name()))
                    ),
                ));
            }
            lines.insert(path.clone(), at_v(v));
            if layer == "semantic" {
                over.semantic.insert(k.to_string(), e);
            } else {
                over.component.insert(k.to_string(), e);
            }
        }
    }
    if let Some(t) = table("accents")? {
        for (fam, i) in t.iter() {
            let path = format!("accents.{fam}");
            if !super::accent::is_family(fam) || fam == "graphite" {
                return Err(issue(
                    at(i),
                    &path,
                    format!(
                        "unknown accent family{} (families: harbor, ember, iris, rose, sea)",
                        def::suggest(fam, super::accent::FAMILIES.iter().copied())
                    ),
                ));
            }
            let slot = if base.dark { "dark" } else { "light" };
            let entry = match i {
                toml_edit::Item::Table(tt) => tt.get(slot).and_then(|x| x.as_value()),
                toml_edit::Item::Value(toml_edit::Value::InlineTable(it)) => it.get(slot),
                other => {
                    return Err(issue(
                        at(other),
                        &path,
                        "expected dark = {…} and light = {…}".into(),
                    ));
                }
            };
            if let Some(v) = entry {
                over.accents
                    .insert(fam.to_string(), hex_of(v, &format!("{path}.{slot}"))?);
            }
        }
    }
    // [type] and [glyphs] are accepted (spec §3.1); styles and glyph sets come from the mode today.
    for k in doc.iter().map(|(k, _)| k) {
        if ![
            "meta",
            "palette",
            "semantic",
            "component",
            "accents",
            "type",
            "glyphs",
        ]
        .contains(&k)
        {
            let i = doc.get(k).expect("listed");
            return Err(issue(
                at(i),
                k,
                format!(
                    "unknown section{} (meta, palette, semantic, component, accents, type, glyphs)",
                    def::suggest(
                        k,
                        [
                            "meta",
                            "palette",
                            "semantic",
                            "component",
                            "accents",
                            "type",
                            "glyphs"
                        ]
                        .into_iter()
                    )
                ),
            ));
        }
    }
    let t = ThemeDef::extend(&base, name, &over);
    if let Some(e) = t.errors().into_iter().next() {
        // `semantic.bg = "@slate.95": unknown palette entry …`: point at that entry
        let path = e.split([' ', ':']).next().unwrap_or("").to_string();
        let line = lines.get(&path).copied().unwrap_or(0);
        let msg = e
            .strip_prefix(&path)
            .map(|r| r.trim_start_matches([' ', '=']).trim_start())
            .unwrap_or(&e)
            .to_string();
        return Err(issue(line, &path, msg));
    }
    Ok(t)
}

/// Dotted keys (`slate.950 = …`, `text.3 = …`) arrive as nested tables: flatten them to `a.b` names (an
/// inline `{ hex = … }` is a value).
fn walk<'a>(
    t: &'a dyn toml_edit::TableLike,
    prefix: &str,
    out: &mut Vec<(String, &'a toml_edit::Item)>,
) {
    for (k, i) in t.iter() {
        let key = if prefix.is_empty() {
            k.to_string()
        } else {
            format!("{prefix}.{k}")
        };
        match i {
            toml_edit::Item::Table(sub) => walk(sub, &key, out),
            toml_edit::Item::Value(toml_edit::Value::InlineTable(it))
                if it.get("hex").is_none() =>
            {
                walk(it, &key, out)
            }
            other => out.push((key, other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extends_overrides_and_resolves() {
        let t = parse(
            "themes/warm.toml",
            "warm",
            "[meta]\nname = \"warm\"\nextends = \"ledger-dark\"\n[palette]\nslate.950 = { hex = \"#161412\" }\n\
             [semantic]\nbg = \"@slate.950\"\n[component]\n\"section.band.bg\" = \"mix($surface.section, $accent, 0.04)\"\n\
             [accents.ember]\ndark = { hex = \"#F08A55\" }\nlight = { hex = \"#A8461A\" }\n",
        )
        .unwrap();
        assert!(t.dark);
        assert_eq!(t.palette["slate.950"], 0x161412);
        assert_eq!(t.accents["ember"], 0xF08A55);
        let p = def::resolve(
            &t,
            def::Knobs::defaults(true),
            "ember",
            &[],
            &BTreeMap::new(),
        );
        assert_eq!(p.get(crate::theme_tokens::Token::Bg), 0x161412);
    }

    #[test]
    fn errors_name_the_line() {
        let e = parse(
            "themes/warm.toml",
            "warm",
            "[meta]\nname = \"warm\"\n[palette]\nslate.950 = \"#161412\"\n[semantic]\nbg = \"@slate.95\"\n",
        )
        .unwrap_err();
        assert_eq!((e.line, e.path.as_str()), (6, "semantic.bg"));
        assert!(e.message.contains("unknown palette entry"), "{e}");
        assert!(e.message.contains("slate.950"), "{e}");

        let e = parse("themes/x.toml", "x", "[meta]\nextends = \"x\"\n").unwrap_err();
        assert_eq!((e.line, e.path.as_str()), (2, "meta.extends"));
        assert!(e.message.contains("built-in only"), "{e}");

        let e = parse("themes/x.toml", "x", "[semantic]\ntext.9 = \"#000000\"\n").unwrap_err();
        assert_eq!(e.line, 2);
        assert!(e.message.contains("unknown token"), "{e}");

        let e = parse(
            "themes/x.toml",
            "x",
            "[component]\n\"header.band.bg\" = \"mix($bg, 1)\"\n",
        )
        .unwrap_err();
        assert_eq!((e.line, e.path.as_str()), (2, "component.header.band.bg"));
        assert!(e.message.contains("three arguments"), "{e}");

        let e = parse("themes/x.toml", "x", "[meta]\nname = \"y\"\n").unwrap_err();
        assert!(e.message.contains("must match"), "{e}");

        let e = parse("themes/x.toml", "x", "[semantic\n").unwrap_err();
        assert_eq!(e.path, "(syntax)");
    }

    #[test]
    fn a_light_base_takes_the_light_accent() {
        let t = parse(
            "themes/l.toml",
            "l",
            "[meta]\nextends = \"ledger-light\"\n[accents.iris]\ndark = \"#000001\"\nlight = \"#000002\"\n",
        )
        .unwrap();
        assert!(!t.dark);
        assert_eq!(t.accents["iris"], 0x000002);
    }
}
