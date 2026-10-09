//! CHECK (design §5): every issue a manifest has, computed locally (no Xero call). Errors block planning;
//! warnings are acknowledged in review (an undoable event with the actor), then planning may go ahead.
//!
//! Each issue sits on a FIELD (a patch path such as `/JournalLines/n2/AccountCode`, or `""` for the whole
//! document), so the diff view can show it under the field and the Issues list can jump to it.

use super::effects::Diff;
use super::normalise::{dec, normalise};
use super::rules::Matched;
use super::{Ack, Manifest, Result};
use crate::record::Record;
use crate::view::{Applied, Ctx, View};
use rust_decimal::Decimal;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Error,
    Warning,
}

impl Severity {
    pub fn glyph(self) -> &'static str {
        match self {
            Severity::Error => "✗",
            Severity::Warning => "!",
        }
    }
    pub fn word(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issue {
    pub severity: Severity,
    /// `stale`, `not_writable`, `unbalanced`, `account`, `locked`, `accountant`, `duplicate`, `overlap`,
    /// `cash_basis`, `narration`, `lines`, `line_id`, `file` (errors); `reconciled`, `attachment`, `outlier`,
    /// `suspense`, `bridge`, `split` (warnings).
    pub code: &'static str,
    /// The entry (0: the manifest as a whole, e.g. a bridge it breaks).
    pub seq: i64,
    pub org: String,
    /// The patch path of the field it concerns (`""`: the document).
    pub path: String,
    /// The human label of that field.
    pub field: String,
    pub message: String,
    pub hint: String,
    /// For a warning: who acknowledged it, when.
    pub ack: Option<Ack>,
}

impl Issue {
    pub fn blocking(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// The issues, errors first (by entry), then warnings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Report {
    pub issues: Vec<Issue>,
}

impl Report {
    pub fn errors(&self) -> usize {
        self.issues.iter().filter(|i| i.blocking()).count()
    }
    pub fn warnings(&self) -> usize {
        self.issues.iter().filter(|i| !i.blocking()).count()
    }
    pub fn acknowledged(&self) -> usize {
        self.issues
            .iter()
            .filter(|i| !i.blocking() && i.ack.is_some())
            .count()
    }
    /// Warnings not yet acknowledged.
    pub fn unacked(&self) -> usize {
        self.warnings() - self.acknowledged()
    }
    pub fn for_entry(&self, seq: i64) -> impl Iterator<Item = &Issue> {
        self.issues.iter().filter(move |i| i.seq == seq)
    }
    /// `✗1 !2`, or `✓` when the entry is clean.
    pub fn entry_badge(&self, seq: i64) -> String {
        let e = self.for_entry(seq).filter(|i| i.blocking()).count();
        let w = self
            .for_entry(seq)
            .filter(|i| !i.blocking() && i.ack.is_none())
            .count();
        match (e, w) {
            (0, 0) => "✓".into(),
            (0, w) => format!("!{w}"),
            (e, 0) => format!("✗{e}"),
            (e, w) => format!("✗{e} !{w}"),
        }
    }
    /// The Issues mode's top line.
    pub fn summary(&self) -> String {
        if self.issues.is_empty() {
            return "✓ no issues · ready to plan".into();
        }
        let mut parts = vec![];
        let e = self.errors();
        if e > 0 {
            parts.push(format!(
                "{e} error{} block{} planning",
                if e == 1 { "" } else { "s" },
                if e == 1 { "s" } else { "" }
            ));
        }
        let w = self.warnings();
        if w > 0 {
            let a = self.acknowledged();
            parts.push(format!(
                "{w} warning{}, {a} acknowledged",
                if w == 1 { "" } else { "s" }
            ));
        }
        if e == 0 && self.unacked() == 0 {
            parts.push("ready to plan".into());
        }
        parts.join(" · ")
    }
}

/// What a check needs beyond the record and the manifest.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// The as-of date for effects (bridges, suspense).
    pub as_of: String,
    pub ic_config: Option<Value>,
    pub zero_accounts: std::collections::BTreeMap<String, Vec<String>>,
    /// Xero's per-journal line limit, once measured.
    pub line_limit: Option<usize>,
}

/// The human label of a patch path (a small per-entity table; unknown paths show raw).
pub fn label(path: &str) -> String {
    if let Some(pair) = path.strip_prefix("bridge:") {
        return format!("bridge {pair}");
    }
    let s: Vec<&str> = path.trim_start_matches('/').split('/').collect();
    let field = |f: &str| -> String {
        match f {
            "Date" => "Date".into(),
            "Narration" => "Narration".into(),
            "Status" => "Status".into(),
            "Reference" => "Reference".into(),
            "AccountCode" => "account".into(),
            "Description" => "description".into(),
            "LineAmount" => "amount".into(),
            "TaxAmount" => "tax".into(),
            "ShowOnCashBasisReports" => "Cash basis".into(),
            "LineAmountTypes" => "Amounts".into(),
            "Contact" => "Contact".into(),
            "Total" => "Total".into(),
            "IsReconciled" => "Reconciled".into(),
            other => format!("/{other}"),
        }
    };
    match s.as_slice() {
        [""] => "document".into(),
        ["JournalLines" | "LineItems"] => "lines".into(),
        ["JournalLines" | "LineItems", k] => format!("line {k}"),
        ["JournalLines" | "LineItems", k, f, ..] => format!("line {k} {}", field(f)),
        ["Attachments", n, ..] => format!("attachment {n}"),
        ["Attachments"] => "attachments".into(),
        [f] => field(f),
        _ => path.to_string(),
    }
}

/// The closest account codes to an unknown one (same prefix, or one edit away).
fn did_you_mean(code: &str, codes: &[String]) -> Option<String> {
    let close = |a: &str, b: &str| {
        if a.len() != b.len() {
            return false;
        }
        a.chars().zip(b.chars()).filter(|(x, y)| x != y).count() == 1
    };
    codes
        .iter()
        .find(|c| close(code, c))
        .or_else(|| {
            codes
                .iter()
                .find(|c| c.starts_with(&code[..code.len().min(3)]))
        })
        .cloned()
}

struct Ctx2<'a> {
    m: &'a Manifest,
    out: Vec<Issue>,
}

impl Ctx2<'_> {
    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        severity: Severity,
        code: &'static str,
        seq: i64,
        org: &str,
        path: &str,
        message: String,
        hint: impl Into<String>,
    ) {
        let ack = if severity == Severity::Warning {
            self.m.acked(seq, code, path).cloned()
        } else {
            None
        };
        self.out.push(Issue {
            severity,
            code,
            seq,
            org: org.into(),
            path: path.into(),
            field: label(path),
            message,
            hint: hint.into(),
            ack,
        });
    }
}

fn lines(after: &Value) -> Vec<(String, Value)> {
    for k in ["JournalLines", "LineItems"] {
        if let Some(o) = after.get(k).and_then(Value::as_object) {
            return o.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        }
    }
    vec![]
}

/// Check a manifest. `diff` (the effects) is optional: without it, bridge and suspense warnings are skipped.
pub fn check(
    record: &Record,
    ctx: &Ctx,
    m: &Manifest,
    opts: &Options,
    diff: Option<&Diff>,
) -> Result<Report> {
    let view = View::open(record, ctx, Some(m))?;
    let ov = view.overlay().expect("a manifest view");
    let mut c = Ctx2 { m, out: vec![] };
    for conf in &ov.conflicts {
        let e = m.entry(conf.seq);
        let why = conf
            .reasons
            .iter()
            .map(|r| format!("{} {} → {}", r.what, r.old, r.new))
            .collect::<Vec<_>>()
            .join("; ");
        c.push(
            Severity::Error,
            "stale",
            conf.seq,
            e.map_or("", |e| e.org.as_str()),
            "",
            format!("main moved since this entry was made: {why}"),
            "review the document as it is now, then edit the entry (e) or undo it (u)",
        );
    }
    // one entry per document
    let mut seen: Vec<(&str, &str, &str, i64)> = vec![];
    for e in m.active() {
        if e.id.starts_with("new:") {
            continue;
        }
        if let Some((.., first)) = seen
            .iter()
            .find(|(o, en, id, _)| *o == e.org && *en == e.entity && *id == e.id)
        {
            c.push(
                Severity::Error,
                "overlap",
                e.seq,
                &e.org,
                "",
                format!("entry {first} already changes this document"),
                "combine them: edit the first entry, then undo this one",
            );
        } else {
            seen.push((&e.org, &e.entity, &e.id, e.seq));
        }
    }
    for a in &ov.applied {
        check_entry(record, &view, ctx, m, a, opts, &mut c)?;
    }
    if let Some(d) = diff {
        effects(d, m, &mut c);
    }
    let mut issues = c.out;
    issues.sort_by_key(|i| (i.severity, i.seq));
    Ok(Report { issues })
}

#[allow(clippy::too_many_arguments)]
fn check_entry(
    record: &Record,
    view: &View<'_>,
    _ctx: &Ctx,
    m: &Manifest,
    a: &Applied,
    opts: &Options,
    c: &mut Ctx2<'_>,
) -> Result<()> {
    let seq = a.seq;
    let org = a.org.as_str();
    if let Matched::Unresolved { message, hint } = &a.matched {
        let path = m
            .entry(seq)
            .and_then(|e| e.paths().into_iter().next())
            .unwrap_or_default();
        c.push(
            Severity::Error,
            "reconciled",
            seq,
            org,
            &path,
            message.clone(),
            hint.clone(),
        );
    }
    if let Matched::NotWritable { message, hint } = &a.matched {
        let path = m
            .entry(seq)
            .and_then(|e| e.paths().into_iter().next())
            .unwrap_or_default();
        c.push(
            Severity::Error,
            "not_writable",
            seq,
            org,
            &path,
            message.clone(),
            hint.clone(),
        );
    }
    let chart: Vec<Value> = view.main_payloads_of(&a.tenant, "Accounts")?;
    let codes: Vec<String> = chart
        .iter()
        .filter_map(|x| x.get("Code").and_then(Value::as_str).map(str::to_string))
        .filter(|s| !s.is_empty())
        .collect();
    let entity = a.doc.0.as_str();
    let line_arr = if entity == "ManualJournals" {
        "JournalLines"
    } else {
        "LineItems"
    };
    let ls = lines(&a.after);
    let changed: Vec<String> = m.entry(seq).map(|e| e.paths()).unwrap_or_default();
    let line_touched = |k: &str| {
        a.before.is_none()
            || changed
                .iter()
                .any(|p| p.starts_with(&format!("/{line_arr}/{k}")))
    };
    // accounts: every line the entry adds or changes names one active, non-bank account
    for (k, l) in &ls {
        if !line_touched(k) || a.kind == "void" {
            continue;
        }
        let code = l.get("AccountCode").and_then(Value::as_str).unwrap_or("");
        let path = format!("/{line_arr}/{k}/AccountCode");
        let acct = chart
            .iter()
            .find(|x| x.get("Code").and_then(Value::as_str) == Some(code));
        match acct {
            None => {
                let dym = did_you_mean(code, &codes)
                    .map(|d| format!(" (did you mean {d}?)"))
                    .unwrap_or_default();
                c.push(
                    Severity::Error,
                    "account",
                    seq,
                    org,
                    &path,
                    format!("unknown account {code}{dym}"),
                    "use an account from the org's chart",
                );
            }
            Some(x) => {
                let status = x.get("Status").and_then(Value::as_str).unwrap_or("ACTIVE");
                if status != "ACTIVE" {
                    c.push(
                        Severity::Error,
                        "account",
                        seq,
                        org,
                        &path,
                        format!("account {code} is {}", status.to_lowercase()),
                        "use an active account",
                    );
                } else if x.get("Type").and_then(Value::as_str) == Some("BANK") {
                    c.push(
                        Severity::Error,
                        "account",
                        seq,
                        org,
                        &path,
                        format!("{code} is a bank account; a journal or recode can't post to it"),
                        "bank lines come from bank transactions",
                    );
                }
            }
        }
    }
    // journals: balanced, explicit cash basis, a narration, two lines
    if entity == "ManualJournals" && a.kind != "void" {
        let net: Decimal = ls.iter().map(|(_, l)| dec(l, "LineAmount")).sum();
        if !net.is_zero() {
            c.push(
                Severity::Error,
                "unbalanced",
                seq,
                org,
                "/JournalLines",
                format!("unbalanced: Δ {}", crate::money::money_str(net)),
                "debits must equal credits",
            );
        }
        if a.kind == "create" {
            if !a
                .after
                .get("ShowOnCashBasisReports")
                .is_some_and(Value::is_boolean)
            {
                c.push(
                    Severity::Error,
                    "cash_basis",
                    seq,
                    org,
                    "/ShowOnCashBasisReports",
                    "the journal doesn't say whether it shows on cash-basis reports".into(),
                    "set cash basis yes or no explicitly",
                );
            }
            if a.after
                .get("Narration")
                .and_then(Value::as_str)
                .is_none_or(|n| n.trim().is_empty())
            {
                c.push(
                    Severity::Error,
                    "narration",
                    seq,
                    org,
                    "/Narration",
                    "the journal has no narration".into(),
                    "say what it is for",
                );
            }
            if ls.len() < 2 {
                c.push(
                    Severity::Error,
                    "lines",
                    seq,
                    org,
                    "/JournalLines",
                    "a journal needs at least two lines".into(),
                    "add the other side",
                );
            }
            for (k, l) in &ls {
                let amt = dec(l, "LineAmount");
                if amt.is_zero() || amt.round_dp(2) != amt {
                    c.push(
                        Severity::Error,
                        "lines",
                        seq,
                        org,
                        &format!("/JournalLines/{k}/LineAmount"),
                        format!("amount {amt}: non-zero, at most 2 decimals"),
                        "fix the amount",
                    );
                }
            }
            // a duplicate of a journal already in Xero (same date, accounts and amounts)
            let mut want: Vec<(String, String)> = ls
                .iter()
                .map(|(_, l)| {
                    (
                        l["AccountCode"].as_str().unwrap_or("").into(),
                        l["LineAmount"].as_str().unwrap_or("").into(),
                    )
                })
                .collect();
            want.sort();
            for d in view.main_payloads_of(&a.tenant, "ManualJournals")? {
                if d.get("Status").and_then(Value::as_str) != Some("POSTED") {
                    continue;
                }
                let n = normalise(&d);
                if n.get("Date") != a.after.get("Date") {
                    continue;
                }
                let mut have: Vec<(String, String)> = lines(&n)
                    .iter()
                    .map(|(_, l)| {
                        (
                            l["AccountCode"].as_str().unwrap_or("").into(),
                            l["LineAmount"].as_str().unwrap_or("").into(),
                        )
                    })
                    .collect();
                have.sort();
                if have == want {
                    c.push(
                        Severity::Error,
                        "duplicate",
                        seq,
                        org,
                        "",
                        format!(
                            "journal {} already posts the same date, accounts and amounts",
                            crate::view::doc_id("ManualJournals", &d)
                        ),
                        "undo this entry, or change it",
                    );
                    break;
                }
            }
            if let Some(limit) = opts.line_limit
                && ls.len() > limit
            {
                c.push(
                    Severity::Warning,
                    "split",
                    seq,
                    org,
                    "/JournalLines",
                    format!(
                        "{} lines exceed Xero's {limit}-line limit: it would be split",
                        ls.len()
                    ),
                    "acknowledge, or split the journal yourself",
                );
            }
        }
    }
    // recodes: the line must have a Xero line id; a reconciled bank transaction may be refused
    if matches!(a.matched, Matched::Op("recode_bank_line")) {
        let mut lineids = changed.iter().filter_map(|p| {
            p.trim_start_matches('/')
                .split('/')
                .nth(1)
                .map(str::to_string)
        });
        if let Some(k) = lineids.next() {
            let has_id = a
                .before
                .as_ref()
                .and_then(|b| b.get("LineItems"))
                .and_then(|l| l.get(&k))
                .and_then(|l| l.get("LineItemID"))
                .and_then(Value::as_str)
                .is_some_and(|s| !s.is_empty());
            if !has_id {
                c.push(
                    Severity::Error,
                    "line_id",
                    seq,
                    org,
                    &format!("/LineItems/{k}"),
                    "the line has no Xero line id in the record; a line is never chosen by position".into(),
                    format!("re-sync the org (`ownpurse sync {org}`); line ids come with the full document"),
                );
            }
        }
    }
    // a transfer can't be voided through the API: its recovery is a reverse transfer
    if matches!(a.matched, super::rules::Matched::Op("create_bank_transfer")) {
        c.push(
            Severity::Warning,
            "transfer",
            seq,
            org,
            "",
            "permanent history: a transfer can't be voided through the API (its recovery is a reverse transfer)".into(),
            "acknowledge after checking the accounts and amount",
        );
    }
    // history notes: permanent, and within the length a note may have
    if matches!(a.matched, super::rules::Matched::Op("add_note")) {
        let notes: Vec<String> = a
            .after
            .get("History")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .skip(
                a.before
                    .as_ref()
                    .and_then(|b| b.get("History"))
                    .and_then(Value::as_array)
                    .map_or(0, Vec::len),
            )
            .filter_map(|n| n.get("Details").and_then(Value::as_str).map(str::to_string))
            .collect();
        for n in &notes {
            if n.trim().is_empty() || n.chars().count() > crate::write::NOTE_MAX {
                c.push(
                    Severity::Error,
                    "note",
                    seq,
                    org,
                    "/History",
                    format!(
                        "a note needs 1 to {} characters ({} given)",
                        crate::write::NOTE_MAX,
                        n.chars().count()
                    ),
                    "shorten the note",
                );
            }
        }
        c.push(
            Severity::Warning,
            "note",
            seq,
            org,
            "/History",
            "permanent: Xero keeps history notes for good".into(),
            "acknowledge after reading the note",
        );
    }
    // attachments: permanent, and the file must be there and allowed
    if let Some(atts) = a.after.get("Attachments").and_then(Value::as_object) {
        let before = a
            .before
            .as_ref()
            .and_then(|b| b.get("Attachments"))
            .and_then(Value::as_object);
        for (name, att) in atts {
            if before.is_some_and(|b| b.contains_key(name)) {
                continue;
            }
            let path = format!(
                "/Attachments/{}",
                name.replace('~', "~0").replace('/', "~1")
            );
            let file = att.get("Path").and_then(Value::as_str).unwrap_or("");
            let sha = att.get("Sha256").and_then(Value::as_str).unwrap_or("");
            let bad = match std::fs::read(file) {
                Err(e) => Some(format!("{file}: {e}")),
                Ok(b) if crate::write::sha256_hex(&b) != sha => {
                    Some(format!("{file} changed since it was added"))
                }
                Ok(b) if b.len() as u64 > crate::write::ops::MAX_ATTACHMENT => {
                    Some(format!("{name} is over Xero's 10 MB limit"))
                }
                Ok(_) if crate::write::ops::mime_of(name).is_none() => {
                    Some(format!("{name}: only PDF, CSV or TXT attach"))
                }
                Ok(_) => None,
            };
            if let Some(msg) = bad {
                c.push(
                    Severity::Error,
                    "file",
                    seq,
                    org,
                    &path,
                    msg,
                    "add the file again",
                );
            }
            c.push(
                Severity::Warning,
                "attachment",
                seq,
                org,
                &path,
                "permanent: an attachment can't be removed through the API".into(),
                "acknowledge after checking the file",
            );
        }
    }
    // lock dates
    if let Ok(Some(lock)) = crate::write::plan::lock_date(record, &a.tenant)
        && !matches!(a.matched, super::rules::Matched::Op("add_note"))
        && a.date.as_str() <= lock.as_str()
    {
        c.push(
            Severity::Error,
            "locked",
            seq,
            org,
            "/Date",
            format!("{} is inside the lock date {lock}; Xero refuses it", a.date),
            "use a date after the lock, or ask for the period to be reopened",
        );
    }
    // the accountant gate
    if a.needs_accountant {
        c.push(
            Severity::Error,
            "accountant",
            seq,
            org,
            "",
            format!(
                "{} is a frozen or prior period: the accountant decides it",
                a.date
            ),
            "n to attach the accountant's reference (an adjusting-entry id or email)",
        );
    }
    // amount outliers per account (a line far above anything the account has seen)
    if a.kind != "void" {
        let l = crate::ledger::Ledger::build(&View::main(record), &a.tenant)
            .map_err(|e| super::ManifestError::new("record", e.to_string()))?;
        for (k, li) in &ls {
            if !line_touched(k) {
                continue;
            }
            let amt = dec(li, "LineAmount").abs();
            let code = li.get("AccountCode").and_then(Value::as_str).unwrap_or("");
            let Some(acct) = l.chart.by_code(code) else {
                continue;
            };
            let seen: Vec<Decimal> = l
                .postings
                .iter()
                .filter(|p| p.account_id == acct.account_id)
                .map(|p| p.amount.abs())
                .collect();
            let max = seen.iter().copied().max().unwrap_or_default();
            if seen.len() >= 3 && !max.is_zero() && amt > max * Decimal::from(5) {
                c.push(
                    Severity::Warning,
                    "outlier",
                    seq,
                    org,
                    &format!("/{line_arr}/{k}/LineAmount"),
                    format!(
                        "{} on {code} is over 5× the largest amount it has seen ({})",
                        crate::money::money_str(amt),
                        crate::money::money_str(max)
                    ),
                    "acknowledge if intended",
                );
            }
        }
    }
    Ok(())
}

/// Warnings from the effects: suspense left non-zero, and a balanced bridge broken.
fn effects(d: &Diff, m: &Manifest, c: &mut Ctx2<'_>) {
    let first_in = |org: &str| m.active().find(|e| e.org == org).map_or(0, |e| e.seq);
    for o in &d.orgs {
        for z in &o.zero {
            if z.main != z.manifest && !z.manifest.is_zero() {
                c.push(
                    Severity::Warning,
                    "suspense",
                    first_in(&o.org),
                    &o.org,
                    "",
                    format!(
                        "{} {} would be {} (must be zero at handoff)",
                        z.code,
                        z.name,
                        crate::money::money_str(z.manifest)
                    ),
                    "acknowledge, or clear it in this manifest",
                );
            }
        }
    }
    for b in &d.bridges {
        if b.main.is_some_and(|v| v.is_zero()) && b.manifest.is_some_and(|v| !v.is_zero()) {
            c.push(
                Severity::Warning,
                "bridge",
                0,
                "",
                &format!("bridge:{}", b.pair),
                format!(
                    "breaks the balanced bridge {}: residual 0.00 → {}",
                    b.label,
                    crate::money::money_str(b.manifest.unwrap_or_default())
                ),
                "acknowledge, or add the other side",
            );
        }
    }
}

impl View<'_> {
    /// Main's documents of an entity (never the overlay).
    pub fn main_payloads_of(&self, tid: &str, entity: &str) -> Result<Vec<Value>> {
        Ok(View::main(self.record()).payloads(tid, entity)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn labels() {
        assert_eq!(label("/JournalLines/n2/AccountCode"), "line n2 account");
        assert_eq!(label("/Narration"), "Narration");
        assert_eq!(label("/Url"), "/Url");
        assert_eq!(label(""), "document");
        assert_eq!(
            did_you_mean("5029", &["5020".into(), "6140".into()]).as_deref(),
            Some("5020")
        );
    }
}
