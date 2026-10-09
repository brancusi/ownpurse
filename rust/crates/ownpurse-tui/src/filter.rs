//! The `/` filter (and the agent bridge's `filter` verb; one `update` path, one meaning), v0.10.1:
//! - default: case-insensitive SUBSTRING over each screen's searched text (Ledger TB: code + account
//!   name; GL: date, source, reference, contact, description; Overview: alias + name; IC: pair id,
//!   label, orgs; Activity: time, org, method, path, status, problem; System: the row's cells; Compare:
//!   code + label). Spaces and punctuation are literal; nothing is fuzzy or tokenised.
//! - `"quoted text"`: the same phrase match with the quotes removed (the default doesn't tokenise, so it
//!   only matters for leading or trailing spaces).
//! - `code:2410` or `code:1450..1480`: account codes (exact, or an inclusive numeric range) on screens with
//!   codes (Ledger TB, Compare); elsewhere it matches as literal text.
//!
//! Where a filter means nothing (Ledger L3, IC L2/L3, Compare postings, the Ledger without one org) it
//! is refused: the key shows a note, the agent gets `not_applicable`.

/// A parsed filter.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Filter {
    None,
    /// Lower-cased phrase.
    Text(String),
    /// Inclusive code range (numeric when both ends parse; else exact text match on `lo`).
    Code {
        lo: String,
        hi: String,
    },
}

impl Filter {
    pub fn parse(q: &str) -> Filter {
        if q.is_empty() {
            return Filter::None;
        }
        if let Some(c) = q.trim().strip_prefix("code:") {
            let c = c.trim();
            if !c.is_empty() {
                let (lo, hi) = c.split_once("..").unwrap_or((c, c));
                return Filter::Code {
                    lo: lo.trim().to_string(),
                    hi: hi.trim().to_string(),
                };
            }
        }
        let t = q
            .strip_prefix('"')
            .and_then(|r| r.strip_suffix('"'))
            .unwrap_or(q);
        Filter::Text(t.to_lowercase())
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Filter::None)
    }

    /// A row with searched text `hay` (and its account code, when the screen has codes).
    pub fn matches(&self, hay: &str, code: Option<&str>) -> bool {
        match self {
            Filter::None => true,
            Filter::Text(t) => hay.to_lowercase().contains(t.as_str()),
            Filter::Code { lo, hi } => match code {
                Some(c) => code_in(c, lo, hi),
                // no codes on this screen: `code:…` is literal text
                None => hay.to_lowercase().contains(
                    &format!(
                        "code:{}",
                        if lo == hi {
                            lo.clone()
                        } else {
                            format!("{lo}..{hi}")
                        }
                    )
                    .to_lowercase(),
                ),
            },
        }
    }
}

fn code_in(c: &str, lo: &str, hi: &str) -> bool {
    match (c.parse::<u64>(), lo.parse::<u64>(), hi.parse::<u64>()) {
        (Ok(c), Ok(l), Ok(h)) => l <= c && c <= h,
        _ => c.eq_ignore_ascii_case(lo) || c.eq_ignore_ascii_case(hi),
    }
}

/// The strip chip, the query always quoted (spaces show): `filter: "Tallowmere -" · 3 of 11`.
pub fn chip(q: &str, shown: usize, total: usize) -> String {
    format!("filter: \"{q}\" · {shown} of {total}")
}

impl crate::app::App {
    /// Whether a filter means something on the current screen: Ok, or the screen's name for the refusal.
    pub fn filter_applies(&self) -> Result<(), &'static str> {
        use crate::app::Tab;
        match self.ui.tab {
            Tab::Overview | Tab::Activity | Tab::System => Ok(()),
            Tab::Classify if self.cl_queue() => Ok(()),
            Tab::Classify => Err("this Classify mode"),
            Tab::Manifest => Err("tab 7 Manifest"),
            Tab::Ledger if self.cmp_active() => match self.ui.cmp.as_ref().map(|c| c.level) {
                Some(2) => Err("Compare postings"),
                _ => Ok(()),
            },
            Tab::Ledger if self.ledger_org().is_none() => Err("the Ledger without one org"),
            Tab::Ledger => match self.ledger_level() {
                3 => Err("a Ledger document"),
                _ => Ok(()),
            },
            Tab::Intercompany => match self.ic_level() {
                1 => Ok(()),
                2 => Err("the Intercompany bridge"),
                _ => Err("an Intercompany document"),
            },
        }
    }

    /// (rows shown, rows before the filter) on the current screen; None where a filter doesn't apply.
    pub fn filter_counts(&self) -> Option<(usize, usize)> {
        use crate::app::Tab;
        self.filter_applies().ok()?;
        Some(match self.ui.tab {
            Tab::Overview => (self.overview_rows().len(), self.model.orgs.len()),
            Tab::Activity => (
                crate::views::activity::shown_len(self),
                crate::views::activity::unfiltered_len(self),
            ),
            Tab::System => (
                crate::views::system::layout(self).0.len(),
                crate::views::system::item_total(self),
            ),
            Tab::Ledger if self.cmp_active() => {
                let m = self.cmp_full_pub()?;
                (self.cmp_visible(m).len(), m.rows.len())
            }
            Tab::Ledger if self.ledger_level() == 2 => (
                self.l2_visible().len(),
                self.l2_model().map_or(0, |m| m.rows.len()),
            ),
            Tab::Ledger => (self.l1_visible().len(), self.l1_unfiltered()),
            Tab::Intercompany => (self.ic_rows().len(), self.ic_scoped().len()),
            Tab::Classify => {
                let r = self.cl_rows();
                (r.rows.len(), r.total)
            }
            Tab::Manifest => (0, 0),
        })
    }

    /// The strip chip for an active filter on this screen: `filter: "Tallowmere -" · 3 of 11`.
    pub fn filter_chip(&self) -> Option<String> {
        if self.ui.filter.is_empty() {
            return None;
        }
        let (n, total) = self.filter_counts()?;
        Some(chip(&self.ui.filter, n, total))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn substring_quotes_and_code_ranges() {
        let f = Filter::parse("Kiln Room -");
        assert!(f.matches("6140 Kiln Room - Afterburner", Some("6140")));
        assert!(!f.matches("1010 Kiln - Operating", Some("1010")));
        assert!(
            !f.matches("3010 Kiln Room Capital", Some("3010")),
            "punctuation is literal"
        );
        assert!(Filter::parse("kiln room").matches("Kiln Room Capital", None));
        assert!(Filter::parse("\" loan\"").matches("Brackenridge Term loan", None));
        assert!(!Filter::parse("\" loan\"").matches("loans", None));
        let r = Filter::parse("code:1450..1480");
        assert!(r.matches("x", Some("1450")) && r.matches("x", Some("1480")));
        assert!(!r.matches("x", Some("1481")) && !r.matches("x", Some("145")));
        assert!(Filter::parse("code:2410").matches("", Some("2410")));
        assert!(!Filter::parse("code:2410").matches("", Some("24100")));
        assert!(
            !Filter::parse("code:2410").matches("2410 Loans", None),
            "literal without codes"
        );
        assert!(Filter::parse("").is_none());
        assert_eq!(
            chip("Kiln Room -", 9, 43),
            "filter: \"Kiln Room -\" · 9 of 43"
        );
    }
}
