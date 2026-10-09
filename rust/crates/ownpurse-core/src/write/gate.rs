//! The AZ freeze gate (design, PM addition 1; CLAUDE.md 4 Sep 2026), shared with book branches: an operation
//! needs an `accountant_decision` reference when
//! - its org is in `branches.accountant_gate` (`[{org, through}]` in the PROJECT ownpurse.json) and the dated
//!   effect falls on or before `through`; or
//! - it is prior-period: dated before the start of the financial year that contains the latest synced
//!   trial balance (`prior_before`), for any org.
//!
//! "Dated effect": the journal date for a create; the target document's date for a recode or a void (a void
//! of a 2025 journal changes 2025 just as much as a 2025 create does).

/// Why an operation needs an accountant's reference (None when it does not).
pub fn needs_accountant(
    org: &str,
    date: &str,
    gates: &[(String, String)],
    prior_before: Option<&str>,
) -> Option<String> {
    for (o, through) in gates {
        if o == org && date <= through.as_str() {
            return Some(format!(
                "{org} is frozen through {through} (the accountant gate): a {date} change needs the accountant's decision"
            ));
        }
    }
    if let Some(p) = prior_before
        && date < p
    {
        return Some(format!(
            "{date} is a prior period (before {p}): the accountant decides prior-period changes"
        ));
    }
    None
}

/// A usable reference: a short identifier, never a placeholder.
pub fn reference_ok(r: &str) -> bool {
    let t = r.trim();
    t.len() >= 3
        && t.len() <= 200
        && !matches!(
            t.to_ascii_lowercase().as_str(),
            "tbd" | "todo" | "none" | "n/a" | "na" | "pending" | "xxx"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_and_prior_period() {
        let g = vec![("az".to_string(), "2025-12-31".to_string())];
        assert!(needs_accountant("az", "2025-06-30", &g, None).is_some());
        assert!(needs_accountant("az", "2025-12-31", &g, None).is_some());
        assert!(needs_accountant("az", "2026-01-01", &g, None).is_none());
        assert!(needs_accountant("fa", "2025-06-30", &g, None).is_none());
        assert!(needs_accountant("fa", "2024-12-31", &g, Some("2025-01-01")).is_some());
        assert!(reference_ok("AJE-3 (email 2026-10-01)"));
        assert!(!reference_ok("TBD"));
        assert!(!reference_ok(" "));
    }
}
