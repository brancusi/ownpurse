//! The rules engine: conditions are ANDed; rules run highest priority first; for each label type the first
//! (highest-priority) rule that sets it wins. Output per line: one suggestion per label type with its source
//! `rule:<id>`, the rule's confidence and the evidence (which condition matched which text).

use super::anchor::{Anchor, Line};
use super::config::{ClassifyConfig, Cond, Rule, SetValue, Sign};
use super::{Confidence, Evidence};
use rust_decimal::Decimal;

/// One label suggested by one rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleHit {
    pub rule: String,
    pub label_type: String,
    pub value: String,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
}

/// Everything the rules say about one line.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LineSuggestions {
    /// One per label type, in config label-type order.
    pub hits: Vec<RuleHit>,
    /// `(rule id, group kind)` for rules that propose a group.
    pub group_kinds: Vec<(String, String)>,
    /// Every rule that matched, highest priority first.
    pub fired: Vec<String>,
}

/// "Would match N lines" with a sample of up to 5 anchors (record order).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WouldMatch {
    pub count: usize,
    pub sample: Vec<Anchor>,
}

/// Whole-word, case-insensitive: `needle` (lowercase) occurs in `hay` (lowercase) with no letter or digit on
/// either side.
pub fn has_word(hay: &str, needle: &str) -> bool {
    if needle.is_empty() {
        return false;
    }
    let mut from = 0;
    while let Some(k) = hay[from..].find(needle) {
        let start = from + k;
        let end = start + needle.len();
        let before = hay[..start].chars().next_back();
        let after = hay[end..].chars().next();
        let edge = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
        if edge(before) && edge(after) {
            return true;
        }
        from = start + hay[start..].chars().next().map_or(1, char::len_utf8);
    }
    false
}

fn any_contains(hay: &str, needles: &[String]) -> Option<String> {
    let h = hay.to_lowercase();
    needles
        .iter()
        .find(|n| !n.is_empty() && h.contains(&n.to_lowercase()))
        .cloned()
}

fn ev(condition: &str, matched: impl Into<String>) -> Evidence {
    Evidence {
        condition: condition.into(),
        matched: matched.into(),
    }
}

/// Does `rule` match `line`? `Some(evidence, matched choice)` if every condition holds.
pub fn matches(
    config: &ClassifyConfig,
    rule: &Rule,
    line: &Line,
) -> Option<(Vec<Evidence>, Option<String>)> {
    let mut evidence = vec![];
    let mut matched = None;
    for c in &rule.when {
        match c {
            Cond::DescriptionHasAlias(ty) => {
                let t = config.label_type(ty)?;
                let d = line.description.to_lowercase();
                let hit = t.active().find_map(|ch| {
                    ch.aliases
                        .iter()
                        .find(|a| has_word(&d, &a.to_lowercase()))
                        .map(|a| (ch.id.clone(), a.clone()))
                })?;
                evidence.push(ev(c.key(), hit.1));
                matched = Some(hit.0);
            }
            Cond::Payee(p) => {
                if line.payee.to_lowercase() != p.to_lowercase() {
                    return None;
                }
                evidence.push(ev(c.key(), line.payee.clone()));
            }
            Cond::PayeeHas(v) => evidence.push(ev(c.key(), any_contains(&line.payee, v)?)),
            Cond::DescriptionHas(v) => {
                evidence.push(ev(c.key(), any_contains(&line.description, v)?));
            }
            Cond::AccountIn(v) => {
                if !v.contains(&line.account_code) {
                    return None;
                }
                evidence.push(ev(c.key(), line.account_code.clone()));
            }
            Cond::OrgIn(v) => {
                if !v.contains(&line.org) {
                    return None;
                }
                evidence.push(ev(c.key(), line.org.clone()));
            }
            Cond::AmountBetween(lo, hi) => {
                if line.amount < *lo || line.amount > *hi {
                    return None;
                }
                evidence.push(ev(c.key(), crate::money::money_str(line.amount)));
            }
            Cond::DateBetween(a, b) => {
                if line.date.as_str() < a.as_str() || line.date.as_str() > b.as_str() {
                    return None;
                }
                evidence.push(ev(c.key(), line.date.clone()));
            }
            Cond::Sign(s) => {
                let ok = match s {
                    Sign::In => line.amount > Decimal::ZERO,
                    Sign::Out => line.amount < Decimal::ZERO,
                };
                if !ok {
                    return None;
                }
                evidence.push(ev(c.key(), crate::money::money_str(line.amount)));
            }
            Cond::PayeeIsOrg(want) => {
                let org = line.counterparty_org.as_deref()?;
                if want != "any" && want != org {
                    return None;
                }
                evidence.push(ev(c.key(), format!("{} = {org}", line.payee)));
            }
            Cond::BankTransfer(want) => {
                if line.bank_transfer != *want {
                    return None;
                }
                evidence.push(ev(c.key(), line.bank_transfer.to_string()));
            }
        }
    }
    Some((evidence, matched))
}

/// Evaluate every active rule on one line.
pub fn evaluate(config: &ClassifyConfig, line: &Line) -> LineSuggestions {
    let mut out = LineSuggestions::default();
    let mut by_type: Vec<RuleHit> = vec![];
    for rule in config.rules_by_priority() {
        let Some((evidence, matched)) = matches(config, rule, line) else {
            continue;
        };
        out.fired.push(rule.id.clone());
        if let Some(g) = &rule.group_kind {
            out.group_kinds.push((rule.id.clone(), g.clone()));
        }
        for (ty, v) in &rule.set {
            if by_type.iter().any(|h| &h.label_type == ty) {
                continue;
            }
            let value = match v {
                SetValue::Choice(c) => c.clone(),
                SetValue::Matched => match &matched {
                    Some(m) => m.clone(),
                    None => continue,
                },
            };
            by_type.push(RuleHit {
                rule: rule.id.clone(),
                label_type: ty.clone(),
                value,
                confidence: rule.confidence,
                evidence: evidence.clone(),
            });
        }
    }
    for t in &config.label_types {
        if let Some(k) = by_type.iter().position(|h| h.label_type == t.id) {
            out.hits.push(by_type.swap_remove(k));
        }
    }
    out
}

/// How many of `lines` the rule matches, with a sample of 5.
pub fn would_match(config: &ClassifyConfig, rule: &Rule, lines: &[Line]) -> WouldMatch {
    let mut w = WouldMatch::default();
    for l in lines {
        if matches(config, rule, l).is_some() {
            w.count += 1;
            if w.sample.len() < 5 {
                w.sample.push(l.anchor.clone());
            }
        }
    }
    w
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::classify::anchor::{DocType, Fingerprint};
    use crate::classify::config::{parse, tests::SAMPLE};

    pub(crate) fn line(
        org: &str,
        id: &str,
        payee: &str,
        desc: &str,
        amount: &str,
        acct: &str,
        date: &str,
    ) -> Line {
        let amount: Decimal = amount.parse().unwrap();
        Line {
            anchor: Anchor::line(org, DocType::BankTransaction, id, "L1"),
            org: org.into(),
            tenant_id: "t".into(),
            doc_type: DocType::BankTransaction,
            doc_id: id.into(),
            line_id: "L1".into(),
            date: date.into(),
            payee: payee.into(),
            description: desc.into(),
            amount,
            account_code: acct.into(),
            account_name: String::new(),
            reconciled: Some(true),
            reference: String::new(),
            status: "AUTHORISED".into(),
            fingerprint: Fingerprint {
                org: org.into(),
                doc_id: id.into(),
                line_id: "L1".into(),
                account: acct.into(),
                amount: amount.to_string(),
                date: date.into(),
                description_hash: crate::classify::anchor::description_hash(desc),
            },
            counterparty_org: None,
            bank_transfer: false,
        }
    }

    fn cfg_with(rules: &str) -> ClassifyConfig {
        let base = SAMPLE.split("# the alias rule").next().unwrap();
        parse(&format!("{base}{rules}"), "c").unwrap()
    }

    fn one(when: &str) -> (ClassifyConfig, Rule) {
        let c = cfg_with(&format!(
            "[[rules]]\nid = \"r\"\nwhen = {{ {when} }}\nset = {{ bearer = \"hold\" }}\n"
        ));
        let r = c.rules[0].clone();
        (c, r)
    }

    #[test]
    fn whole_word() {
        assert!(has_word("hbl sample run", "hbl"));
        assert!(has_word("for the harvest.", "harvest"));
        assert!(!has_word("harvesting", "harvest"));
        assert!(!has_word("rehbl", "hbl"));
        assert!(has_word("x — harvest ☕", "harvest"));
    }

    #[test]
    fn each_condition() {
        let l = line(
            "roast",
            "bt-1",
            "Hollowpine Packaging Works",
            "Bag proofs for HBL",
            "-1148.63",
            "5310",
            "2025-04-17",
        );
        let cases = [
            ("payee = \"hollowpine packaging works\"", true),
            ("payee = \"Hollowpine\"", false),
            ("payee_has = [\"pine\"]", true),
            ("payee_has = \"oak\"", false),
            ("description_has = [\"proofs\"]", true),
            ("description_has = \"reprint\"", false),
            ("account_in = [\"5310\", \"5510\"]", true),
            ("account_in = [\"53\"]", false),
            ("org_in = [\"roast\"]", true),
            ("org_in = [\"lab\"]", false),
            ("amount_between = [-2000.17, -1000.03]", true),
            ("amount_between = [0.01, 99.99]", false),
            ("date_between = [\"2025-04-01\", \"2025-04-30\"]", true),
            ("date_between = [\"2025-05-01\", \"2025-05-31\"]", false),
            ("sign = \"out\"", true),
            ("sign = \"in\"", false),
            ("description_has_alias = \"project\"", true),
            ("payee_has = \"pine\", sign = \"in\"", false),
        ];
        for (when, want) in cases {
            let (c, r) = one(when);
            assert_eq!(matches(&c, &r, &l).is_some(), want, "{when}");
        }
        let (c, r) = one("payee_has = \"pine\", account_in = [\"5310\"]");
        let (ev, _) = matches(&c, &r, &l).unwrap();
        assert_eq!(
            ev[0],
            Evidence {
                condition: "payee_has".into(),
                matched: "pine".into()
            }
        );
        assert_eq!(ev[1].matched, "5310");
    }

    #[test]
    fn matched_and_priorities() {
        let c = parse(SAMPLE, "c").unwrap();
        let l = line(
            "roast",
            "bt-2",
            "Hollowpine Packaging Works",
            "Wayfinding signs",
            "-263.94",
            "5310",
            "2025-11-03",
        );
        let s = evaluate(&c, &l);
        assert_eq!(s.fired, vec!["r-alias", "r-hollowpine"]);
        // project from the higher-priority alias rule ($matched = refit), bearer from the payee rule
        assert_eq!(s.hits.len(), 2);
        assert_eq!(
            (
                s.hits[0].label_type.as_str(),
                s.hits[0].value.as_str(),
                s.hits[0].rule.as_str()
            ),
            ("project", "refit", "r-alias")
        );
        assert_eq!(s.hits[0].evidence[0].matched, "wayfinding");
        assert_eq!(
            (
                s.hits[1].value.as_str(),
                s.hits[1].confidence.to_string().as_str()
            ),
            ("roast", "0.85")
        );
        // no alias: the payee rule sets the project too
        let l2 = line(
            "roast",
            "bt-3",
            "Hollowpine Packaging Works",
            "Bag proofs",
            "-41.27",
            "5310",
            "2025-11-03",
        );
        let s2 = evaluate(&c, &l2);
        assert_eq!(s2.hits[0].value, "harvest");
        assert_eq!(s2.hits[0].rule, "r-hollowpine");
    }

    #[test]
    fn would_match_counts_and_samples_five() {
        let c = parse(SAMPLE, "c").unwrap();
        let lines: Vec<Line> = (0..8)
            .map(|i| {
                line(
                    "roast",
                    &format!("bt-{i}"),
                    if i % 2 == 0 {
                        "Hollowpine Packaging Works"
                    } else {
                        "Other"
                    },
                    "x",
                    "-1.13",
                    "5310",
                    "2025-01-02",
                )
            })
            .collect();
        let w = would_match(&c, c.rule("r-hollowpine").unwrap(), &lines);
        assert_eq!(w.count, 4);
        assert_eq!(w.sample.len(), 4);
        let many: Vec<Line> = (0..9)
            .map(|i| {
                line(
                    "roast",
                    &format!("bt-{i}"),
                    "Hollowpine Packaging Works",
                    "x",
                    "-1.13",
                    "5310",
                    "2025-01-02",
                )
            })
            .collect();
        let w = would_match(&c, c.rule("r-hollowpine").unwrap(), &many);
        assert_eq!((w.count, w.sample.len()), (9, 5));
    }
}
