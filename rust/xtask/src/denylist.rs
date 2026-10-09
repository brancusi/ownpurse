//! The PRIVATE denylist: terms that must never appear in this repository (names, amounts, identifiers from
//! private books). The list itself is never committed, hashed or summarised here: a hashed list of short
//! tokens can be reversed by brute force. It is read from `$OWNPURSE_DENYLIST` (a path), else
//! `~/.config/ownpurse/denylist` (usually a symlink to the private file).
//!
//! Format: one term per line; `#` starts a comment line; `!phrase` allows a phrase: a hit lying wholly inside an
//! occurrence of an allowed phrase is not reported (so a generic word can be allowed in its harmless context). Name terms match case-insensitively on word boundaries (no letter or digit right before or after:
//! `zqbolt` matches `Zqbolt Supplies`, never `thunderzqbolt`), except amounts: a term shaped like money (`-?[0-9][0-9,]*\.[0-9]{2}`) matches a whole amount token in either form
//! (`3,083.19` or `3083.19`), never part of a longer number (`100.00` does not match `2100.000`). One
//! Aho-Corasick automaton holds the text terms and a hash set the normalised amounts, so thousands of terms
//! scan the tree in about a second. Text and binary files (SQLite fixtures) are scanned as bytes.
//!
//! A hit prints `file:line` (text) or `file@offset` (binary) and the term's LINE NUMBER in the private list,
//! never the term: neither a CI log nor a terminal ever shows the list.
//!
//! When the list is missing: `CI=true` → `denylist: skipped (CI, no private list)` (exit 0, public CI has no
//! list); `OWNPURSE_DENYLIST=skip` → a loud warning naming why (exit 0); otherwise exit 1 with where to put
//! the list. check.sh runs this, so the pre-push hook and a local gate refuse to pass without it.

use crate::files::{self, Item};
use aho_corasick::{AhoCorasick, AhoCorasickBuilder, MatchKind};
use regex::bytes::Regex;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Where the private list comes from.
#[derive(Debug, PartialEq, Eq)]
pub enum Source {
    Path(PathBuf),
    /// `OWNPURSE_DENYLIST=skip`: the explicit, loud bypass.
    Skip,
    /// No list: skipped in CI, refused everywhere else.
    Missing {
        ci: bool,
        looked: Vec<PathBuf>,
    },
}

/// Find the list from an environment lookup and a home folder.
pub fn locate(env: &dyn Fn(&str) -> Option<String>, home: Option<&Path>) -> Source {
    let ci = env("CI").is_some_and(|v| v == "true" || v == "1");
    let mut looked = Vec::new();
    if let Some(v) = env("OWNPURSE_DENYLIST").filter(|v| !v.is_empty()) {
        if v == "skip" {
            return Source::Skip;
        }
        let p = PathBuf::from(v);
        if p.is_file() {
            return Source::Path(p);
        }
        looked.push(p);
    } else if let Some(h) = home {
        let p = h.join(".config/ownpurse/denylist");
        if p.is_file() {
            return Source::Path(p);
        }
        looked.push(p);
    }
    Source::Missing { ci, looked }
}

/// What a missing or skipped list means: (exit code, message).
pub fn verdict(src: &Source) -> Option<(i32, String)> {
    match src {
        Source::Path(_) => None,
        Source::Skip => Some((
            0,
            "denylist: !!! SKIPPED: OWNPURSE_DENYLIST=skip !!!\n\
             denylist: !!! nothing was checked for private terms; never push or tag like this without a reason !!!"
                .into(),
        )),
        Source::Missing { ci: true, .. } => {
            Some((0, "denylist: skipped (CI, no private list)".into()))
        }
        Source::Missing { ci: false, looked } => Some((
            1,
            format!(
                "denylist: no private list (looked at {}).\n\
                 denylist: put it at ~/.config/ownpurse/denylist (or a symlink there), or set OWNPURSE_DENYLIST=<path>.\n\
                 denylist: to bypass deliberately: OWNPURSE_DENYLIST=skip (prints a warning).",
                looked
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        )),
    }
}

/// A compiled list: text terms in one automaton, amounts by normalised value.
pub struct List {
    ac: Option<AhoCorasick>,
    /// pattern index → list line number
    text_lines: Vec<usize>,
    /// normalised amount (no separators, no sign) → list line number
    amounts: HashMap<String, usize>,
    amount_token: Regex,
    /// `!phrase` lines: hits inside an occurrence of one are not reported
    allow: Option<AhoCorasick>,
    pub terms: usize,
}

fn money_shaped(t: &str) -> bool {
    let re = regex::Regex::new(r"^-?[0-9][0-9,]*\.[0-9]{2}$").expect("money regex");
    re.is_match(t)
}

fn norm_amount(t: &str) -> String {
    t.trim_start_matches('-').replace(',', "")
}

impl List {
    pub fn parse(text: &str) -> List {
        let mut pats = Vec::new();
        let mut text_lines = Vec::new();
        let mut amounts = HashMap::new();
        let mut allow = Vec::new();
        for (i, raw) in text.lines().enumerate() {
            let t = raw.trim();
            if t.is_empty() || t.starts_with('#') {
                continue;
            }
            if let Some(a) = t.strip_prefix('!') {
                if !a.trim().is_empty() {
                    allow.push(a.trim().to_string());
                }
                continue;
            }
            if money_shaped(t) {
                amounts.entry(norm_amount(t)).or_insert(i + 1);
            } else {
                pats.push(t.to_string());
                text_lines.push(i + 1);
            }
        }
        let terms = pats.len() + amounts.len();
        let ac = (!pats.is_empty()).then(|| {
            AhoCorasickBuilder::new()
                .ascii_case_insensitive(true)
                .match_kind(MatchKind::Standard)
                .build(&pats)
                .expect("denylist automaton")
        });
        List {
            ac,
            text_lines,
            amounts,
            amount_token: Regex::new(r"[0-9][0-9,]*\.[0-9]{2}").expect("amount regex"),
            allow: (!allow.is_empty()).then(|| {
                AhoCorasickBuilder::new()
                    .ascii_case_insensitive(true)
                    .match_kind(MatchKind::Standard)
                    .build(&allow)
                    .expect("allow automaton")
            }),
            terms,
        }
    }

    /// Every hit in one file: (offset, list line).
    pub fn hits(&self, bytes: &[u8]) -> Vec<(usize, usize)> {
        // (start, end, list line)
        let mut found: Vec<(usize, usize, usize)> = Vec::new();
        if let Some(ac) = &self.ac {
            // a letter, a digit or any non-ASCII byte (part of a UTF-8 letter) on either side: inside a word
            let wordy = |b: u8| b.is_ascii_alphanumeric() || b >= 0x80;
            for m in ac.find_overlapping_iter(bytes) {
                let before = m.start().checked_sub(1).map(|i| bytes[i]);
                let after = bytes.get(m.end()).copied();
                let first = bytes[m.start()];
                let last = bytes[m.end() - 1];
                // a boundary is only required where the term itself starts or ends with a word character
                if (wordy(first) && before.is_some_and(wordy))
                    || (wordy(last) && after.is_some_and(wordy))
                {
                    continue;
                }
                found.push((m.start(), m.end(), self.text_lines[m.pattern().as_usize()]));
            }
        }
        if !self.amounts.is_empty() {
            for m in self.amount_token.find_iter(bytes) {
                let before = m.start().checked_sub(1).map(|i| bytes[i]);
                let after = bytes.get(m.end()).copied();
                if before.is_some_and(|b| b.is_ascii_digit() || b == b'.' || b == b',')
                    || after.is_some_and(|b| b.is_ascii_digit())
                {
                    continue; // part of a longer number
                }
                let tok = String::from_utf8_lossy(m.as_bytes());
                if let Some(line) = self.amounts.get(&norm_amount(&tok)) {
                    found.push((m.start(), m.end(), *line));
                }
            }
        }
        if let Some(allow) = &self.allow {
            let spans: Vec<(usize, usize)> = allow
                .find_overlapping_iter(bytes)
                .map(|m| (m.start(), m.end()))
                .collect();
            found.retain(|(s, e, _)| !spans.iter().any(|(a, b)| a <= s && e <= b));
        }
        let mut out: Vec<(usize, usize)> = found.into_iter().map(|(s, _, l)| (s, l)).collect();
        out.sort_unstable();
        out
    }
}

/// Scan files; print each hit (never the term). Returns the hit count.
pub fn scan(list: &List, items: &[Item]) -> usize {
    let mut n = 0;
    for it in items {
        for (off, line) in list.hits(&it.bytes) {
            println!(
                "denylist: {}: private term (list line {line})",
                files::place(it, off)
            );
            n += 1;
        }
    }
    n
}

pub fn run(args: &[String]) -> i32 {
    let home = std::env::var("HOME").ok().map(PathBuf::from);
    let src = locate(&|k| std::env::var(k).ok(), home.as_deref());
    if let Some((code, msg)) = verdict(&src) {
        println!("{msg}");
        return code;
    }
    let Source::Path(p) = src else { return 1 };
    let text = match std::fs::read_to_string(&p) {
        Ok(t) => t,
        Err(e) => {
            println!("denylist: could not read the private list ({e})");
            return 1;
        }
    };
    let list = List::parse(&text);
    let root = files::root();
    let mut items = Vec::new();
    if !args.iter().any(|a| a == "--no-tracked") {
        items.extend(files::tracked(&root));
    }
    for d in args.iter().filter(|a| !a.starts_with("--")) {
        items.extend(files::under(Path::new(d)));
    }
    let started = std::time::Instant::now();
    let hits = scan(&list, &items);
    if hits == 0 {
        println!(
            "denylist: OK ({} terms, {} files, {} ms)",
            list.terms,
            items.len(),
            started.elapsed().as_millis()
        );
        0
    } else {
        println!("denylist: {hits} hit(s)");
        1
    }
}

/// A temp list with planted synthetic terms: every planted file is caught, clean files pass, the output never
/// carries a term, and the three discovery paths behave.
pub fn self_test() -> i32 {
    let mut ok = true;
    let mut fail = |m: String| {
        println!("denylist self-test: {m}");
        ok = false;
    };
    let term = ["Zq", "Planted", "Vendor"].concat();
    let pair = ["zq ", "planted pair"].concat();
    let amount = "4,321.98";
    let list_text =
        format!("# a comment line\n{term}\n\n{pair}\n{amount}\n100.00\n!{term} are fine\nzqbolt\n");
    let list = List::parse(&list_text);
    let item = |name: &str, bytes: Vec<u8>| Item {
        name: name.into(),
        bytes,
    };
    let caught = [
        (
            "a.md",
            format!("x {} y", term.to_uppercase()).into_bytes(),
            2,
        ),
        (
            "b.md",
            format!("x {} y", pair.to_uppercase()).into_bytes(),
            4,
        ),
        ("c.md", b"total 4,321.98 here".to_vec(), 5),
        ("d.md", b"total 4321.98 here".to_vec(), 5),
        ("e.md", b"(4,321.98)".to_vec(), 5),
        ("f.md", b"x 100.00 y".to_vec(), 6),
        ("h.md", b"paid by Zqbolt Supplies.".to_vec(), 8),
        ("i.md", b"(ZQBOLT)".to_vec(), 8),
        (
            "g.sqlite",
            [b"\0\x01".to_vec(), term.as_bytes().to_vec(), b"\0".to_vec()].concat(),
            2,
        ),
    ];
    for (name, bytes, line) in &caught {
        let h = list.hits(bytes);
        if !h.iter().any(|(_, l)| l == line) {
            fail(format!(
                "{name}: NOT caught (want list line {line}), got {h:?}"
            ));
        }
    }
    for (name, bytes) in [
        (
            "clean.md",
            b"roast hold design lab 3,083.19 2100.000 14,321.98 4321.985".to_vec(),
        ),
        ("clean2.md", b"zq plantedvendo".to_vec()),
        ("allowed.md", format!("x {term} are fine y").into_bytes()),
        (
            "word.md",
            b"a thunderzqbolt order; zqbolted; zqplantedvendorx".to_vec(),
        ),
    ] {
        let h = list.hits(&bytes);
        if !h.is_empty() {
            fail(format!("{name}: clean text flagged {h:?}"));
        }
    }
    // the report names file:line and the list line, never the term
    let it = item("docs/x.md", format!("a\nb {term}\n").into_bytes());
    let place = files::place(&it, list.hits(&it.bytes)[0].0);
    if place != "docs/x.md:2" {
        fail(format!("place {place}"));
    }
    if list.terms != 5 {
        fail(format!("{} terms", list.terms));
    }
    // discovery: env path, home default, missing (local: refused; CI: skipped), skip (loud)
    let d = files::temp("denylist-self");
    let lp = d.join("list");
    std::fs::write(&lp, &list_text).expect("write list");
    let home = d.join("home");
    std::fs::create_dir_all(home.join(".config/ownpurse")).expect("home");
    let env_with = |pairs: Vec<(&'static str, String)>| {
        move |k: &str| pairs.iter().find(|(a, _)| *a == k).map(|(_, v)| v.clone())
    };
    let s = locate(
        &env_with(vec![("OWNPURSE_DENYLIST", lp.display().to_string())]),
        None,
    );
    if s != Source::Path(lp.clone()) {
        fail(format!("env path: {s:?}"));
    }
    let s = locate(&env_with(vec![]), Some(&home));
    if !matches!(s, Source::Missing { ci: false, .. }) || verdict(&s).map(|v| v.0) != Some(1) {
        fail(format!("missing locally must fail: {s:?}"));
    }
    let s = locate(&env_with(vec![("CI", "true".into())]), Some(&home));
    if verdict(&s) != Some((0, "denylist: skipped (CI, no private list)".into())) {
        fail(format!("missing in CI must skip: {s:?}"));
    }
    let s = locate(
        &env_with(vec![("OWNPURSE_DENYLIST", "skip".into())]),
        Some(&home),
    );
    if s != Source::Skip || !verdict(&s).is_some_and(|v| v.0 == 0 && v.1.contains("SKIPPED")) {
        fail(format!("skip: {s:?}"));
    }
    std::fs::copy(&lp, home.join(".config/ownpurse/denylist")).expect("copy list");
    let s = locate(&env_with(vec![]), Some(&home));
    if s != Source::Path(home.join(".config/ownpurse/denylist")) {
        fail(format!("home default: {s:?}"));
    }
    let _ = std::fs::remove_dir_all(&d);
    println!("denylist self-test: {}", if ok { "OK" } else { "FAILED" });
    if ok { 0 } else { 1 }
}

#[cfg(test)]
mod tests {
    #[test]
    fn self_test_passes() {
        assert_eq!(super::self_test(), 0);
    }
}
