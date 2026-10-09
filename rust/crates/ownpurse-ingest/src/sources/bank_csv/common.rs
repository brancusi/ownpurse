//! What the bank projectors (CSV downloads and OFX) share: the account a line is on, the facts every bank
//! line carries, and reading back what earlier files said about the same account.

use crate::Input;
use crate::facts::{self, fingerprint};
use ownpurse_facts::{Date, Db, Eid, EntRef, Kw, TxData, ValIn, Value};
use rust_decimal::Decimal;

pub(crate) fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

/// A keyword value built at run time (`obs.status/pending`); `None` if the text is not a keyword.
pub(crate) fn kwv(text: &str) -> Option<Value> {
    Kw::new(text).ok().map(Value::Kw)
}

pub(crate) fn status(name: &str) -> Option<Value> {
    kwv(&format!("obs.status/{name}"))
}

/// A bank account as the shared identities name it: `ridgeway:checking:4821`.
#[derive(Debug, Clone)]
pub(crate) struct AccountSpec {
    pub id: String,
    pub mask: String,
    pub name: Option<String>,
    pub institution: Option<String>,
    pub commodity: String,
    pub scale: Option<i64>,
}

/// Assert the account and its currency; returns the account and commodity temps.
pub(crate) fn account(tx: &mut TxData, input: &Input<'_>, a: &AccountSpec) -> (EntRef, EntRef) {
    let e = facts::account(tx, &a.id);
    tx.add(&e, &k("account/mask"), Value::str(&a.mask));
    if let Some(n) = &a.name {
        tx.add(&e, &k("account/name"), Value::str(n));
    }
    if let Some(i) = &a.institution {
        tx.add(&e, &k("account/institution"), Value::str(i));
    }
    let c = facts::commodity(tx, &a.commodity, a.scale);
    tx.add(&e, &k("account/commodity"), ValIn::Ref(c.clone()));
    tx.add(&e, &k("account/source"), facts::source_ref(&input.source));
    (e, c)
}

/// One bank line, in the shared vocabulary.
pub(crate) struct Line<'a> {
    pub id: String,
    pub doc: &'a EntRef,
    pub region: EntRef,
    pub account: &'a EntRef,
    pub commodity: &'a EntRef,
    pub mask: &'a str,
    pub date: Date,
    pub amount: Decimal,
    pub description: String,
    /// the text the fingerprint is made from (a pending prefix stripped)
    pub match_text: String,
    pub reference: Option<String>,
    pub status: Option<Value>,
}

/// Write the line's shared facts; returns the observation.
pub(crate) fn line(tx: &mut TxData, l: Line<'_>) -> EntRef {
    let o = facts::observation(tx, &l.id, "money-movement");
    tx.add(&o, &k("obs/doc"), ValIn::Ref(l.doc.clone()));
    tx.add(&o, &k("obs/region"), ValIn::Ref(l.region));
    tx.add(&o, &k("obs/account"), ValIn::Ref(l.account.clone()));
    tx.add(&o, &k("obs/commodity"), ValIn::Ref(l.commodity.clone()));
    tx.add(&o, &k("obs/date"), Value::Date(l.date.clone()));
    tx.add(&o, &k("obs/amount"), Value::Dec(l.amount));
    tx.add(&o, &k("obs/description"), Value::str(&l.description));
    if let Some(r) = &l.reference {
        tx.add(&o, &k("obs/reference"), Value::str(r));
    }
    if let Some(s) = l.status {
        tx.add(&o, &k("obs/status"), s);
    }
    let fp = fingerprint(l.mask, l.date.as_str(), l.amount, &l.match_text);
    tx.add(&o, &k("obs/fingerprint"), Value::str(&fp));
    o
}

/// Observations already in the log on this account whose `obs/id` starts with `prefix`.
pub(crate) fn earlier(db: &Db, account_id: &str, prefix: &str) -> Vec<Eid> {
    let (Some(acct), Some(oa)) = (
        db.lookup(&k("account/id"), &Value::str(account_id)),
        db.attr(&k("obs/account")),
    ) else {
        return Vec::new();
    };
    db.with_value(oa.id, &Value::Ref(acct))
        .filter(|e| {
            db.get(*e, &k("obs/id"))
                .and_then(Value::as_str)
                .is_some_and(|id| id.starts_with(prefix))
        })
        .collect()
}

pub(crate) fn date_of(db: &Db, e: Eid) -> Option<Date> {
    match db.get(e, &k("obs/date")) {
        Some(Value::Date(d)) => Some(d.clone()),
        _ => None,
    }
}

pub(crate) fn dec_of(db: &Db, e: Eid, a: &'static str) -> Option<Decimal> {
    db.get(e, &k(a)).and_then(Value::as_dec)
}

pub(crate) fn status_of(db: &Db, e: Eid) -> Option<String> {
    db.get(e, &k("obs/status"))
        .and_then(Value::as_kw)
        .map(|s| s.name().to_string())
}

/// Money as people read it: `-3,215.47`.
pub(crate) fn money(d: Decimal) -> String {
    let s = format!("{:.2}", d.abs());
    let (int, frac) = s.split_once('.').unwrap_or((&s, "00"));
    let mut out = String::new();
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    format!(
        "{}{out}.{frac}",
        if d.is_sign_negative() { "-" } else { "" }
    )
}
