//! Keywords: namespaced names (`obs/amount`, `xero.invoice/Reference`) for attributes and enumerated values.

use std::fmt;
use std::sync::Arc;

/// A namespaced name, `namespace/name`. Both parts are non-empty and use letters, digits, `.`, `-` and `_`;
/// the namespace may be dotted (`xero.invoice`). Ordered and compared as text.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Kw(Arc<str>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KwError(pub String);

impl fmt::Display for KwError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not a keyword (want namespace/name): {:?}", self.0)
    }
}

impl std::error::Error for KwError {}

fn part_ok(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
        && !s.starts_with('.')
        && !s.ends_with('.')
}

impl Kw {
    pub fn new(text: &str) -> Result<Kw, KwError> {
        match text.split_once('/') {
            Some((ns, name)) if part_ok(ns) && part_ok(name) => Ok(Kw(Arc::from(text))),
            _ => Err(KwError(text.to_string())),
        }
    }

    /// A keyword from a literal known to be valid (built-in vocabulary). Panics on a bad literal, which is a
    /// programming error caught by the first test that touches it.
    pub fn lit(text: &'static str) -> Kw {
        match Kw::new(text) {
            Ok(k) => k,
            Err(e) => panic!("{e}"),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn ns(&self) -> &str {
        self.0.split_once('/').map_or("", |(ns, _)| ns)
    }

    pub fn name(&self) -> &str {
        self.0.split_once('/').map_or("", |(_, n)| n)
    }
}

impl fmt::Display for Kw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl fmt::Debug for Kw {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, ":{}", self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keywords_need_a_namespace_and_a_name() {
        assert_eq!(Kw::new("obs/amount").unwrap().ns(), "obs");
        assert_eq!(
            Kw::new("xero.invoice/LineItems").unwrap().name(),
            "LineItems"
        );
        for bad in [
            "amount", "/amount", "obs/", "obs/a/b", "obs/a b", ".x/y", "x./y", "",
        ] {
            assert!(Kw::new(bad).is_err(), "{bad}");
        }
    }
}
