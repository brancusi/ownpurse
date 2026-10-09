//! Anchors and fingerprints. A classifiable line is one line of a bank transaction, manual journal, invoice
//! or bill in the record (read-only). Its anchor is `org:doc_type:doc_id:line_id` (`ext:<kind>:<id>` names an
//! outside item and is parsed but unused in this release). Its fingerprint (V19) is what a label remembers:
//! if the line changes on a later sync, the label renders stale and keeps its value.

use super::{ClassifyError, Result};
use crate::Record;
use crate::ledger::Chart;
use crate::money::money;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::borrow::Cow;

/// The kind of document a line belongs to (`bt`, `mj`, `inv`, `bill` in anchors).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum DocType {
    BankTransaction,
    ManualJournal,
    Invoice,
    Bill,
}

impl DocType {
    pub fn code(self) -> &'static str {
        match self {
            DocType::BankTransaction => "bt",
            DocType::ManualJournal => "mj",
            DocType::Invoice => "inv",
            DocType::Bill => "bill",
        }
    }
    pub fn parse(s: &str) -> Option<DocType> {
        Some(match s {
            "bt" => DocType::BankTransaction,
            "mj" => DocType::ManualJournal,
            "inv" => DocType::Invoice,
            "bill" => DocType::Bill,
            _ => return None,
        })
    }
    /// The record entity the document lives in.
    pub fn entity(self) -> &'static str {
        match self {
            DocType::BankTransaction => "BankTransactions",
            DocType::ManualJournal => "ManualJournals",
            DocType::Invoice | DocType::Bill => "Invoices",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            DocType::BankTransaction => "bank transaction",
            DocType::ManualJournal => "manual journal",
            DocType::Invoice => "invoice",
            DocType::Bill => "bill",
        }
    }
}

/// A line's address in the classification layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Anchor {
    Line {
        org: String,
        doc_type: DocType,
        doc_id: String,
        line_id: String,
    },
    /// An outside item (receipt, email, statement line): parsed, unused in v0.11.0.
    Ext { kind: String, id: String },
}

impl Anchor {
    pub fn line(org: &str, doc_type: DocType, doc_id: &str, line_id: &str) -> Anchor {
        Anchor::Line {
            org: org.into(),
            doc_type,
            doc_id: doc_id.into(),
            line_id: line_id.into(),
        }
    }

    /// `org:doc_type:doc_id:line_id` or `ext:<kind>:<id>`.
    pub fn parse(s: &str) -> Result<Anchor> {
        let parts: Vec<&str> = s.split(':').collect();
        let bad = || {
            ClassifyError::new(
                "usage",
                format!("anchor '{s}': expected org:doc_type:doc_id:line_id or ext:<kind>:<id>"),
            )
        };
        match parts.as_slice() {
            ["ext", kind, id] if !kind.is_empty() && !id.is_empty() => Ok(Anchor::Ext {
                kind: (*kind).into(),
                id: (*id).into(),
            }),
            [org, dt, doc, line]
                if !org.is_empty() && !doc.is_empty() && !line.is_empty() && *org != "ext" =>
            {
                let doc_type = DocType::parse(dt).ok_or_else(bad)?;
                Ok(Anchor::line(org, doc_type, doc, line))
            }
            _ => Err(bad()),
        }
    }

    pub fn org(&self) -> Option<&str> {
        match self {
            Anchor::Line { org, .. } => Some(org),
            Anchor::Ext { .. } => None,
        }
    }
}

impl std::fmt::Display for Anchor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Anchor::Line {
                org,
                doc_type,
                doc_id,
                line_id,
            } => write!(f, "{org}:{}:{doc_id}:{line_id}", doc_type.code()),
            Anchor::Ext { kind, id } => write!(f, "ext:{kind}:{id}"),
        }
    }
}

/// What a label remembers about the line it was set on (V19).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fingerprint {
    pub org: String,
    pub doc_id: String,
    pub line_id: String,
    pub account: String,
    /// The signed amount as exact decimal text.
    pub amount: String,
    pub date: String,
    /// First 16 hex digits of SHA-256 of the description.
    pub description_hash: String,
}

impl Fingerprint {
    pub fn to_json(&self) -> Value {
        json!({
            "org": self.org, "doc_id": self.doc_id, "line_id": self.line_id, "account": self.account,
            "amount": self.amount, "date": self.date, "description_hash": self.description_hash,
        })
    }
    pub fn from_json(v: &Value) -> Option<Fingerprint> {
        let s = |k: &str| v.get(k).and_then(Value::as_str).map(str::to_string);
        Some(Fingerprint {
            org: s("org")?,
            doc_id: s("doc_id")?,
            line_id: s("line_id")?,
            account: s("account")?,
            amount: s("amount")?,
            date: s("date")?,
            description_hash: s("description_hash")?,
        })
    }
    /// The fields that differ from `other` (for the stale detail: "amount 41.27 → 44.91").
    pub fn diff(&self, other: &Fingerprint) -> Vec<(&'static str, String, String)> {
        let mut out = vec![];
        let mut f = |k: &'static str, a: &str, b: &str| {
            if a != b {
                out.push((k, a.to_string(), b.to_string()));
            }
        };
        f("org", &self.org, &other.org);
        f("doc_id", &self.doc_id, &other.doc_id);
        f("line_id", &self.line_id, &other.line_id);
        f("account", &self.account, &other.account);
        f("amount", &self.amount, &other.amount);
        f("date", &self.date, &other.date);
        f(
            "description_hash",
            &self.description_hash,
            &other.description_hash,
        );
        out
    }
}

pub fn description_hash(description: &str) -> String {
    let d = Sha256::digest(description.as_bytes());
    d.iter().take(8).map(|b| format!("{b:02x}")).collect()
}

/// One classifiable line with its Xero facts.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub anchor: Anchor,
    /// Org alias.
    pub org: String,
    pub tenant_id: String,
    pub doc_type: DocType,
    pub doc_id: String,
    pub line_id: String,
    /// `YYYY-MM-DD`.
    pub date: String,
    /// Contact name ("" for journals).
    pub payee: String,
    /// The line description (journals: the line description, else the narration).
    pub description: String,
    /// Signed: money out (spend, bills) negative, money in positive; journal lines as posted (debit +).
    pub amount: Decimal,
    pub account_code: String,
    pub account_name: String,
    /// Bank transactions only.
    pub reconciled: Option<bool>,
    pub reference: String,
    pub status: String,
    pub fingerprint: Fingerprint,
    /// The configured org the payee names (by organisation or legal name, see [`org_key`]), if any; never
    /// the line's own org.
    pub counterparty_org: Option<String>,
    /// A bank transfer leg: a `SPEND-TRANSFER`/`RECEIVE-TRANSFER` transaction, or a line posted to a bank
    /// account.
    pub bank_transfer: bool,
}

impl Line {
    /// `YYYY-MM`.
    pub fn month(&self) -> &str {
        self.date.get(..7).unwrap_or("")
    }
}

fn s<'v>(p: &'v Value, k: &str) -> &'v str {
    p.get(k).and_then(Value::as_str).unwrap_or("")
}

fn skip_status(st: &str) -> bool {
    matches!(st, "DELETED" | "VOIDED")
}

/// Every classifiable line of these orgs (`(alias, tenant_id)`), in record order (org, document id, line).
/// Deleted and voided documents are skipped.
/// A name reduced for matching: lowercase letters and digits only, words joined by single spaces, with a
/// trailing company suffix dropped (`Brackenridge Lane Holdings Ltd` → `brackenridge lane holdings`).
pub fn org_key(name: &str) -> String {
    let cleaned: String = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_lowercase().next().unwrap_or(c)
            } else {
                ' '
            }
        })
        .collect();
    let mut words: Vec<&str> = cleaned.split_whitespace().collect();
    const SUFFIXES: [&str; 14] = [
        "ltd",
        "limited",
        "llc",
        "llp",
        "lp",
        "inc",
        "incorporated",
        "co",
        "corp",
        "corporation",
        "gmbh",
        "plc",
        "pty",
        "company",
    ];
    while words.len() > 1 && words.last().is_some_and(|w| SUFFIXES.contains(w)) {
        words.pop();
    }
    words.join(" ")
}

/// Each configured org's name keys (organisation name and legal name, see [`org_key`]).
pub fn org_names(record: &Record, orgs: &[(String, String)]) -> Result<Vec<(String, Vec<String>)>> {
    let mut out = vec![];
    for (alias, tid) in orgs {
        let o = record.organisation(tid)?;
        let mut keys: Vec<String> = ["Name", "LegalName"]
            .iter()
            .filter_map(|k| o.get(*k).and_then(Value::as_str))
            .map(org_key)
            .filter(|k| !k.is_empty())
            .collect();
        keys.dedup();
        out.push((alias.clone(), keys));
    }
    Ok(out)
}

pub fn enumerate(record: &crate::view::View<'_>, orgs: &[(String, String)]) -> Result<Vec<Line>> {
    let mut out = vec![];
    for (alias, tid) in orgs {
        let chart = Chart::new(&record.payloads(tid, "Accounts")?);
        for (entity, kind) in [
            ("BankTransactions", Kind::Bank),
            ("ManualJournals", Kind::Journal),
            ("Invoices", Kind::Invoice),
        ] {
            for d in docs(record, tid, entity)? {
                if skip_status(&d.status) {
                    continue;
                }
                let (doc_type, sign_out) = match kind {
                    Kind::Bank => (DocType::BankTransaction, d.typ.starts_with("SPEND")),
                    Kind::Journal => (DocType::ManualJournal, false),
                    Kind::Invoice if d.typ == "ACCPAY" => (DocType::Bill, true),
                    Kind::Invoice => (DocType::Invoice, false),
                };
                for (i, li) in d.items.iter().enumerate() {
                    let amt = money(li.amount.as_ref()).map_err(|e| {
                        ClassifyError::new("record", format!("{}: line amount: {e}", d.id))
                    })?;
                    let amount = if sign_out { -amt } else { amt };
                    let reconciled = if kind == Kind::Bank {
                        d.reconciled
                    } else {
                        None
                    };
                    out.push(make(
                        alias, tid, &chart, doc_type, &d, li, i, amount, reconciled,
                    ));
                }
            }
        }
    }
    let names = org_names(record, orgs)?;
    if names.iter().any(|(_, k)| !k.is_empty()) {
        let mut cache: std::collections::HashMap<String, Option<String>> = Default::default();
        for l in &mut out {
            if l.payee.is_empty() {
                continue;
            }
            let hit = cache.entry(org_key(&l.payee)).or_insert_with_key(|k| {
                names
                    .iter()
                    .find(|(_, keys)| keys.iter().any(|x| x == k))
                    .map(|(a, _)| a.clone())
            });
            l.counterparty_org = hit.clone().filter(|a| *a != l.org);
        }
    }
    Ok(out)
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Bank,
    Journal,
    Invoice,
}

/// The facts a line needs from its document (whatever the entity).
struct DocFacts {
    id: String,
    typ: String,
    status: String,
    reference: String,
    narration: String,
    payee: String,
    date: String,
    reconciled: Option<bool>,
    items: Vec<ItemFacts>,
}

struct ItemFacts {
    line_item_id: String,
    account: String,
    description: String,
    amount: Option<Value>,
}

/// Only the fields a line needs, borrowed from the payload text: parsing just these is several times faster
/// than building a full `Value` for every document (real Xero payloads carry dozens of fields).
#[derive(serde::Deserialize)]
struct DocT<'a> {
    #[serde(rename = "BankTransactionID", borrow, default)]
    bt_id: Option<Cow<'a, str>>,
    #[serde(rename = "ManualJournalID", borrow, default)]
    mj_id: Option<Cow<'a, str>>,
    #[serde(rename = "InvoiceID", borrow, default)]
    inv_id: Option<Cow<'a, str>>,
    #[serde(rename = "Type", borrow, default)]
    typ: Option<Cow<'a, str>>,
    #[serde(rename = "Status", borrow, default)]
    status: Option<Cow<'a, str>>,
    #[serde(rename = "Reference", borrow, default)]
    reference: Option<Cow<'a, str>>,
    #[serde(rename = "Narration", borrow, default)]
    narration: Option<Cow<'a, str>>,
    #[serde(rename = "DateString", borrow, default)]
    date_string: Option<Cow<'a, str>>,
    #[serde(rename = "Date", borrow, default)]
    date: Option<Cow<'a, str>>,
    #[serde(rename = "Contact", borrow, default)]
    contact: Option<ContactT<'a>>,
    #[serde(rename = "IsReconciled", default)]
    is_reconciled: Option<bool>,
    #[serde(rename = "LineItems", borrow, default)]
    line_items: Option<Vec<ItemT<'a>>>,
    #[serde(rename = "JournalLines", borrow, default)]
    journal_lines: Option<Vec<ItemT<'a>>>,
}

#[derive(serde::Deserialize)]
struct ContactT<'a> {
    #[serde(rename = "Name", borrow, default)]
    name: Option<Cow<'a, str>>,
}

#[derive(serde::Deserialize)]
struct ItemT<'a> {
    #[serde(rename = "LineItemID", borrow, default)]
    line_item_id: Option<Cow<'a, str>>,
    #[serde(rename = "AccountCode", borrow, default)]
    account: Option<Cow<'a, str>>,
    #[serde(rename = "Description", borrow, default)]
    description: Option<Cow<'a, str>>,
    #[serde(rename = "LineAmount", default)]
    amount: Option<Value>,
}

fn own(c: Option<Cow<'_, str>>) -> String {
    c.map(Cow::into_owned).unwrap_or_default()
}

fn doc_date_of(date_string: &str, date: &str) -> String {
    if !date_string.is_empty() {
        return date_string.chars().take(10).collect();
    }
    crate::dates::xero_date(date).unwrap_or_default()
}

impl DocFacts {
    fn from_typed(d: DocT<'_>) -> DocFacts {
        let items = d.line_items.or(d.journal_lines).unwrap_or_default();
        let date = doc_date_of(
            d.date_string.as_deref().unwrap_or(""),
            d.date.as_deref().unwrap_or(""),
        );
        DocFacts {
            id: own(d.bt_id.or(d.mj_id).or(d.inv_id)),
            typ: own(d.typ),
            status: own(d.status),
            reference: own(d.reference),
            narration: own(d.narration),
            payee: own(d.contact.and_then(|c| c.name)),
            date,
            reconciled: d.is_reconciled,
            items: items
                .into_iter()
                .map(|i| ItemFacts {
                    line_item_id: own(i.line_item_id),
                    account: own(i.account),
                    description: own(i.description),
                    amount: i.amount,
                })
                .collect(),
        }
    }

    /// The tolerant path (a field of an unexpected type): read through a full `Value`, as before.
    fn from_value(p: &Value) -> DocFacts {
        let items = p
            .get("LineItems")
            .or_else(|| p.get("JournalLines"))
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .map(|li| ItemFacts {
                        line_item_id: s(li, "LineItemID").into(),
                        account: s(li, "AccountCode").into(),
                        description: s(li, "Description").into(),
                        amount: li.get("LineAmount").cloned(),
                    })
                    .collect()
            })
            .unwrap_or_default();
        let id = ["BankTransactionID", "ManualJournalID", "InvoiceID"]
            .iter()
            .map(|k| s(p, k))
            .find(|v| !v.is_empty())
            .unwrap_or("");
        DocFacts {
            id: id.into(),
            typ: s(p, "Type").into(),
            status: s(p, "Status").into(),
            reference: s(p, "Reference").into(),
            narration: s(p, "Narration").into(),
            payee: p
                .get("Contact")
                .and_then(|c| c.get("Name"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .into(),
            date: crate::dates::doc_date(p),
            reconciled: p.get("IsReconciled").and_then(Value::as_bool),
            items,
        }
    }
}

/// The current documents of one entity, in record order (by Xero id), as line facts.
fn docs(view: &crate::view::View<'_>, tid: &str, entity: &str) -> Result<Vec<DocFacts>> {
    let mut out = main_docs(view, tid, entity)?;
    // the branch's documents (main + branch): replaced by id, then the proposed ones
    let (replaced, added) = view.patched(tid, entity);
    for (id, p) in replaced {
        if let Some(d) = out.iter_mut().find(|d| d.id == id) {
            *d = DocFacts::from_value(&p);
        }
    }
    out.extend(added.iter().map(DocFacts::from_value));
    Ok(out)
}

fn main_docs(record: &Record, tid: &str, entity: &str) -> Result<Vec<DocFacts>> {
    let mut st = record.conn().prepare_cached(
        // the latest version of each document, through the (tenant, entity, xero_id, version) unique index:
        // the same rows as the `current` view, without grouping the whole table per query
        "SELECT v.xero_id, v.payload FROM versions v WHERE v.tenant_id=?1 AND v.entity=?2 \
         AND v.version = (SELECT MAX(w.version) FROM versions w WHERE w.tenant_id=v.tenant_id \
         AND w.entity=v.entity AND w.xero_id=v.xero_id) ORDER BY v.xero_id",
    )?;
    let rows = st.query_map(rusqlite::params![tid, entity], |r| {
        Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
    })?;
    let mut out = vec![];
    for row in rows {
        let (xero_id, text) = row?;
        let facts = match serde_json::from_str::<DocT>(&text) {
            Ok(d) => DocFacts::from_typed(d),
            Err(_) => {
                let v: Value = serde_json::from_str(&text).map_err(|e| {
                    ClassifyError::new("record", format!("{entity} {xero_id}: bad JSON: {e}"))
                })?;
                DocFacts::from_value(&v)
            }
        };
        out.push(facts);
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn make(
    alias: &str,
    tid: &str,
    chart: &Chart,
    doc_type: DocType,
    d: &DocFacts,
    li: &ItemFacts,
    i: usize,
    amount: Decimal,
    reconciled: Option<bool>,
) -> Line {
    let line_id = if li.line_item_id.is_empty() {
        format!("L{}", i + 1)
    } else {
        li.line_item_id.clone()
    };
    let code = li.account.clone();
    let acct = chart.by_code(&code);
    let bank_transfer = d.typ.contains("TRANSFER") || acct.is_some_and(|a| a.kind == "BANK");
    let account_name = acct.map(|a| a.name.clone()).unwrap_or_default();
    let description = if li.description.is_empty() && doc_type == DocType::ManualJournal {
        d.narration.clone()
    } else {
        li.description.clone()
    };
    let fingerprint = Fingerprint {
        org: alias.into(),
        doc_id: d.id.clone(),
        line_id: line_id.clone(),
        account: code.clone(),
        amount: amount.normalize().to_string(),
        date: d.date.clone(),
        description_hash: description_hash(&description),
    };
    Line {
        anchor: Anchor::line(alias, doc_type, &d.id, &line_id),
        org: alias.into(),
        tenant_id: tid.into(),
        doc_type,
        doc_id: d.id.clone(),
        line_id,
        date: d.date.clone(),
        payee: d.payee.clone(),
        description,
        amount,
        account_code: code,
        account_name,
        reconciled,
        reference: d.reference.clone(),
        status: d.status.clone(),
        fingerprint,
        counterparty_org: None,
        bank_transfer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_round_trip() {
        for s in ["roast:bt:bt-401:L1", "hold:mj:mj-7:L2", "ext:receipt:r-19"] {
            assert_eq!(Anchor::parse(s).unwrap().to_string(), s);
        }
        for bad in ["roast:xx:1:2", "roast:bt:1", "", "ext::1", "roast:bt::L1"] {
            assert!(Anchor::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn fingerprint_diff_names_fields() {
        let a = Fingerprint {
            org: "lab".into(),
            doc_id: "bt-1".into(),
            line_id: "L1".into(),
            account: "5510".into(),
            amount: "-41.27".into(),
            date: "2025-05-20".into(),
            description_hash: description_hash("cups"),
        };
        let mut b = a.clone();
        b.amount = "-44.91".into();
        assert_eq!(
            a.diff(&b),
            vec![("amount", "-41.27".to_string(), "-44.91".to_string())]
        );
        assert_eq!(Fingerprint::from_json(&a.to_json()), Some(a));
    }
}
