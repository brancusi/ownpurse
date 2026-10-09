//! Walkthroughs: the file format is caretline-tour's (`caretline_tour::Tour`, read strictly by its
//! `parse_json` and linted by its `check`; walkthroughs design §1, §3). One parser: this module re-exports
//! the tour types and keeps only ownpurse's own parts:
//!
//! - a step's `host` (opaque to caretline): `{view, viz, checks, why}` ([`StepHost`]): `view` is a partial
//!   canonical view state (`{"state": {…}}`, the view keys) and/or view verbs (`{"verbs": [["tab", "2"], …]}`),
//!   applied with view verbs only; `viz` a visualisation spec `{kind, …params}`; `checks` assertions such as
//!   `amount(roast, 6140, 2025-12-31) = 898.06`, evaluated against the record at play time; `why` the lay
//!   mode's "why this matters" line;
//! - where the files live (`walkthroughs.dir`, default `walkthroughs/` beside ownpurse.json), listing,
//!   reading and writing them;
//! - the checks' parser and evaluator.
//!
//! ```json
//! {"id": "correction", "version": 1, "title": "How a correction gets made", "kind": "hint",
//!  "step": [{"id": "s1", "narration": {"title": "The entry", "text": "This [manual journal] …"},
//!            "anchor": {"host": {"kind": "row", "key": "code=6140"}},
//!            "data": {"title": "Here", "text": "The line that moves."}, "place": {"arrow": true},
//!            "host": {"view": {"verbs": [["scope", "roast"], ["tab", "2"]]}, "checks": []}}]}
//! ```
//!
//! Nothing in a walkthrough can approve, apply or write: steps use view verbs only.

pub use caretline_tour::{
    Level, Narration, Place, Problem, Step, StepAnchor, StepLayer, Tour as Walkthrough, check,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The format version this build writes.
pub const VERSION: u32 = 1;

/// A step's `host`, as ownpurse reads it (unknown keys are kept by caretline and ignored here).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StepHost {
    /// `{"state": {view keys}}` and/or `{"verbs": [[verb, args…], …]}`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view: Option<Value>,
    /// A visualisation spec `{kind, …params}` (computed from the ledger at play time).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub viz: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<String>,
    /// "Why this matters" (lay mode).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub why: Option<String>,
}

impl StepHost {
    /// A step's host part (an absent or malformed `host` reads as empty; `validate` says so).
    pub fn of(step: &Step) -> StepHost {
        step.host
            .as_ref()
            .and_then(|h| serde_json::from_value(h.clone()).ok())
            .unwrap_or_default()
    }

    pub fn to_value(&self) -> Option<Value> {
        let v = serde_json::to_value(self).ok()?;
        (v.as_object().is_some_and(|m| !m.is_empty())).then_some(v)
    }
}

/// The walkthrough's `meta` (caretline-tour carries it opaquely): who wrote it, when, against which
/// record, for whom.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// The record (its label or chain head) the walkthrough was written against.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub record: Option<String>,
    /// `lay` or `informed`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,
}

impl Meta {
    pub fn of(w: &Walkthrough) -> Meta {
        w.meta
            .as_ref()
            .and_then(|m| serde_json::from_value(m.clone()).ok())
            .unwrap_or_default()
    }

    pub fn to_value(&self) -> Option<Value> {
        let v = serde_json::to_value(self).ok()?;
        (v.as_object().is_some_and(|m| !m.is_empty())).then_some(v)
    }
}

/// A step's visualisation spec, if any.
pub fn viz(step: &Step) -> Option<Value> {
    StepHost::of(step).viz
}

/// Read a walkthrough: caretline-tour's strict parser, then its lint (errors refuse; warnings pass).
pub fn parse(text: &str) -> Result<Walkthrough, String> {
    let w = caretline_tour::parse_json(text)?;
    if let Some(p) = check(&w).into_iter().find(|p| p.level == Level::Error) {
        return Err(format!(
            "{}{}: {}",
            p.code,
            p.step
                .as_ref()
                .map(|s| format!(" (step {s})"))
                .unwrap_or_default(),
            p.detail
        ));
    }
    for s in &w.steps {
        if let Some(h) = &s.host {
            serde_json::from_value::<StepHost>(h.clone())
                .map_err(|e| format!("step {}: host: {e}", s.id))?;
        }
    }
    Ok(w)
}

/// Pretty JSON with a trailing newline (the file as written).
pub fn to_text(w: &Walkthrough) -> String {
    let mut s = serde_json::to_string_pretty(w).unwrap_or_default();
    s.push('\n');
    s
}

/// A new, empty walkthrough.
pub fn new(id: &str, title: &str) -> Walkthrough {
    Walkthrough {
        id: id.into(),
        version: VERSION,
        title: title.into(),
        kind: caretline_tour::DEFAULT_KIND.into(),
        reoffer: false,
        steps: vec![],
        meta: None,
    }
}

/// The step's title (its narration's, else its id).
pub fn step_title(s: &Step) -> String {
    s.narration
        .as_ref()
        .and_then(|n| n.title.clone())
        .unwrap_or_else(|| s.id.clone())
}

/// The step's body text.
pub fn step_text(s: &Step) -> String {
    s.narration
        .as_ref()
        .map(|n| n.text.clone())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_tour_shape_strictly() {
        let text = r#"{"id": "w", "title": "T", "step": [
            {"id": "s1", "anchor": {"host": {"kind": "row", "key": "code=6140"}},
             "data": {"title": "A", "text": "B"}, "place": {"connector": true},
             "narration": {"title": "Page", "text": "Body"},
             "host": {"viz": {"kind": "bridge"}, "checks": ["count(roast, 6140) = 3"], "why": "w"}},
            {"id": "s2", "layers": [{"anchor": {"host": {"kind": "chip", "key": "stage"}}, "place": {"ring": true}},
                                    {"id": "x", "anchor": {"screen": "center"}, "data": {"text": "t"}}]}]}"#;
        let w = parse(text).unwrap();
        assert_eq!(w.version, 1);
        assert_eq!(viz(&w.steps[0]).unwrap()["kind"], "bridge");
        assert_eq!(StepHost::of(&w.steps[0]).why.as_deref(), Some("w"));
        assert_eq!(w.steps[0].layers[0].id, "s1/0");
        assert!(w.steps[0].layers[0].place.arrow);
        assert_eq!(w.steps[1].layers[1].id, "x");
        assert_eq!(parse(&to_text(&w)).unwrap(), w);
        let both = r#"{"id": "w", "title": "T", "step": [{"id": "s", "anchor": {"caret": true},
            "layers": [{"anchor": {"caret": true}}]}]}"#;
        assert!(parse(both).unwrap_err().contains("not both"));
        assert!(parse(r#"{"id": "w", "title": "T", "author": "x"}"#).is_err());
    }
}
/// The walkthroughs folder: config `walkthroughs.dir` (relative to the config's folder), default
/// `walkthroughs` beside ownpurse.json.
pub fn dir(cfg: &crate::config::Config) -> std::path::PathBuf {
    let d = cfg
        .settings
        .get_str("walkthroughs.dir")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("walkthroughs");
    let p = std::path::Path::new(d);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        cfg.root.join(p)
    }
}

/// A walkthrough id is a file name: letters, digits, `-` and `_`.
pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Every walkthrough in `dir` (by id; unreadable files are listed with their error).
pub fn list(dir: &std::path::Path) -> Vec<(String, Result<Walkthrough, String>)> {
    let mut out = Vec::new();
    let Ok(rd) = std::fs::read_dir(dir) else {
        return out;
    };
    let mut paths: Vec<std::path::PathBuf> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    paths.sort();
    for p in paths {
        let id = p
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let w = std::fs::read_to_string(&p)
            .map_err(|e| e.to_string())
            .and_then(|t| parse(&t));
        out.push((id, w));
    }
    out
}

/// Read `dir/<id>.json`.
pub fn read(dir: &std::path::Path, id: &str) -> Result<Walkthrough, String> {
    if !valid_id(id) {
        return Err(format!(
            "`{id}` is not a walkthrough id (letters, digits, - and _)."
        ));
    }
    let p = dir.join(format!("{id}.json"));
    let t = std::fs::read_to_string(&p).map_err(|e| format!("{}: {e}", p.display()))?;
    parse(&t).map_err(|e| format!("{}: {e}", p.display()))
}

/// Write `dir/<id>.json` atomically (a temp file, then a rename), creating the folder.
pub fn write(dir: &std::path::Path, w: &Walkthrough) -> Result<std::path::PathBuf, String> {
    if !valid_id(&w.id) {
        return Err(format!(
            "`{}` is not a walkthrough id (letters, digits, - and _).",
            w.id
        ));
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let p = dir.join(format!("{}.json", w.id));
    let tmp = dir.join(format!(".{}.json.tmp", w.id));
    std::fs::write(&tmp, to_text(w)).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, &p).map_err(|e| format!("{}: {e}", p.display()))?;
    Ok(p)
}

/// One `checks` assertion, parsed: `amount(org, account, date) = value` (the account's net debit balance at
/// the date: credits are negative; the account by code, id or name) or `count(org, account[, from, to]) = n`
/// (its postings in the range, both ends included).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Check {
    Amount {
        org: String,
        account: String,
        date: String,
        value: rust_decimal::Decimal,
    },
    Count {
        org: String,
        account: String,
        from: Option<String>,
        to: Option<String>,
        n: usize,
    },
}

/// Parse a check's text.
pub fn parse_check(text: &str) -> Result<Check, String> {
    let bad = || {
        format!(
            "`{text}`: a check is `amount(org, account, YYYY-MM-DD) = value` or `count(org, account[, from, to]) = n`."
        )
    };
    let (lhs, rhs) = text.split_once('=').ok_or_else(bad)?;
    let (f, args) = lhs.trim().split_once('(').ok_or_else(bad)?;
    let args = args.trim().strip_suffix(')').ok_or_else(bad)?;
    let a: Vec<String> = args.split(',').map(|s| s.trim().to_string()).collect();
    let rhs = rhs.trim();
    let date_ok = |d: &str| crate::dates::parse_iso(d).is_some();
    match (f.trim(), a.len()) {
        ("amount", 3) if date_ok(&a[2]) => Ok(Check::Amount {
            org: a[0].clone(),
            account: a[1].clone(),
            date: a[2].clone(),
            value: crate::money::parse(rhs).map_err(|_| bad())?,
        }),
        ("count", 2) | ("count", 4) => {
            let (from, to) = if a.len() == 4 {
                if !date_ok(&a[2]) || !date_ok(&a[3]) {
                    return Err(bad());
                }
                (Some(a[2].clone()), Some(a[3].clone()))
            } else {
                (None, None)
            };
            Ok(Check::Count {
                org: a[0].clone(),
                account: a[1].clone(),
                from,
                to,
                n: rhs.parse().map_err(|_| bad())?,
            })
        }
        _ => Err(bad()),
    }
}

/// A check's result: `Ok(())` when it holds, else what the record says now.
pub fn eval_check(
    check: &Check,
    record: &crate::record::Record,
    orgs: &[(String, String)],
) -> Result<(), String> {
    let org = match check {
        Check::Amount { org, .. } | Check::Count { org, .. } => org,
    };
    let tid = orgs
        .iter()
        .find(|(a, _)| a == org)
        .map(|(_, t)| t.clone())
        .ok_or_else(|| format!("no org `{org}`"))?;
    let view = crate::view::View::main(record);
    let ledger = crate::ledger::Ledger::build(&view, &tid).map_err(|e| e.to_string())?;
    let account = match check {
        Check::Amount { account, .. } | Check::Count { account, .. } => account,
    };
    let acct = ledger
        .chart
        .by_code(account)
        .or_else(|| ledger.chart.by_id.get(account))
        .or_else(|| {
            ledger
                .chart
                .by_id
                .values()
                .find(|a| a.name.eq_ignore_ascii_case(account))
        })
        .map(|a| a.account_id.clone())
        .ok_or_else(|| format!("no account `{account}` in {org}"))?;
    match check {
        Check::Amount { date, value, .. } => {
            let fy = record.fy_end(&tid).map_err(|e| e.to_string())?;
            let b = crate::balances::resolve(
                record,
                &ledger,
                &tid,
                date,
                fy,
                crate::balances::Mode::Auto,
            )
            .map_err(|e| e.to_string())?;
            let now = b.balances.get(&acct).copied().unwrap_or_default();
            if now == *value {
                Ok(())
            } else {
                Err(format!("now {:.2}", now))
            }
        }
        Check::Count { from, to, n, .. } => {
            let k = ledger
                .postings
                .iter()
                .filter(|p| p.account_id == acct)
                .filter(|p| from.as_deref().is_none_or(|f| p.date.as_str() >= f))
                .filter(|p| to.as_deref().is_none_or(|t| p.date.as_str() <= t))
                .count();
            if k == *n {
                Ok(())
            } else {
                Err(format!("now {k}"))
            }
        }
    }
}

#[cfg(test)]
mod check_tests {
    use super::*;

    #[test]
    fn parses_checks() {
        assert!(matches!(
            parse_check("amount(roast, 6140, 2025-12-31) = -1240.00").unwrap(),
            Check::Amount { .. }
        ));
        assert!(matches!(
            parse_check("count(roast, 6140, 2025-01-01, 2025-12-31) = 3").unwrap(),
            Check::Count { n: 3, .. }
        ));
        assert!(parse_check("amount(roast, 6140) = 1").is_err());
        assert!(parse_check("sum(roast, 6140, 2025-01-01) = 1").is_err());
    }
}
