//! The glossary behind walkthroughs' `[term]` links (walkthroughs design §2): a shipped default
//! (`assets/glossary.toml`) plus a project's own `glossary.toml`, whose entries add to or replace the
//! defaults by term.

/// One entry.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub term: String,
    pub aliases: Vec<String>,
    pub text: String,
    pub examples: Vec<String>,
}

/// The default glossary's text.
pub const DEFAULT: &str = include_str!("../assets/glossary.toml");

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Glossary {
    pub entries: Vec<Entry>,
}

fn strings(item: Option<&toml_edit::Item>) -> Vec<String> {
    item.and_then(toml_edit::Item::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn norm(s: &str) -> String {
    s.trim().to_lowercase()
}

impl Glossary {
    /// Parse a glossary file: a table per term with `text`, optional `aliases` and `examples`.
    pub fn parse(text: &str) -> Result<Glossary, String> {
        let doc = toml_edit::Document::parse(text).map_err(|e| e.to_string())?;
        let mut entries = Vec::new();
        for (term, item) in doc.as_table().iter() {
            let Some(t) = item.as_table() else {
                return Err(format!("`{term}` is not a table"));
            };
            let Some(text) = t.get("text").and_then(toml_edit::Item::as_str) else {
                return Err(format!("`{term}` has no `text`"));
            };
            entries.push(Entry {
                term: term.to_string(),
                aliases: strings(t.get("aliases")),
                text: text.to_string(),
                examples: strings(t.get("examples")),
            });
        }
        Ok(Glossary { entries })
    }

    /// The shipped default.
    pub fn default_set() -> Glossary {
        Glossary::parse(DEFAULT).unwrap_or_default()
    }

    /// The default with a project's entries over it (same term: the project's wins).
    pub fn with(mut self, project: Glossary) -> Glossary {
        for e in project.entries {
            match self
                .entries
                .iter_mut()
                .find(|x| norm(&x.term) == norm(&e.term))
            {
                Some(x) => *x = e,
                None => self.entries.push(e),
            }
        }
        self
    }

    /// The default, plus `glossary.toml` in `dir` when there is one (an unreadable file is an error).
    pub fn load(dir: Option<&std::path::Path>) -> Result<Glossary, String> {
        let base = Glossary::default_set();
        let Some(dir) = dir else { return Ok(base) };
        let p = dir.join("glossary.toml");
        match std::fs::read_to_string(&p) {
            Ok(t) => {
                Ok(base.with(Glossary::parse(&t).map_err(|e| format!("{}: {e}", p.display()))?))
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(base),
            Err(e) => Err(format!("{}: {e}", p.display())),
        }
    }

    /// The entry for a term or one of its aliases (case-insensitive).
    pub fn get(&self, term: &str) -> Option<&Entry> {
        let t = norm(term);
        self.entries
            .iter()
            .find(|e| norm(&e.term) == t || e.aliases.iter().any(|a| norm(a) == t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_every_design_term_in_two_to_four_sentences() {
        let g = Glossary::default_set();
        for t in [
            "debit",
            "credit",
            "trial balance",
            "balance sheet",
            "P&L",
            "reconciled",
            "suspense",
            "clearing account",
            "intercompany",
            "due to",
            "due from",
            "reclass journal",
            "void",
            "manual journal",
            "accrual",
            "cash basis",
            "prior period",
            "opening balance",
            "S corporation",
            "K-1",
            "per diem",
            "owner contribution",
            "distribution",
            "loan from owner",
            "books",
            "asset",
            "liability",
            "equity",
            "expense",
            "revenue",
            "income",
            "owner draw",
            "reimbursement",
            "basis",
            "cost basis",
        ] {
            let e = g.get(t).unwrap_or_else(|| panic!("missing {t}"));
            let n = e.text.matches(". ").count() + 1;
            assert!((2..=4).contains(&n), "{t}: {n} sentences");
            assert!(!e.examples.is_empty(), "{t}: no example");
        }
        let p = Glossary::parse("[debit]\ntext = \"Mine.\"\n[x]\ntext = \"New.\"\n").unwrap();
        let g = g.with(p);
        assert_eq!(g.get("dr").map(|e| e.text.as_str()), None);
        assert_eq!(g.get("debit").unwrap().text, "Mine.");
        assert!(g.get("x").is_some());
        assert!(Glossary::parse("[a]\nb = 1\n").is_err());
    }
}
