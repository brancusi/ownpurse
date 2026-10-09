//! Link rules: pure functions from a database to the links they propose. A rule never merges entities
//! (decision D4); it says "these are the same event", with a confidence and its own name as the author, and
//! a person can retract it. Proposing the same link again is a no-op (links are identified by kind and ends).
//!
//! `same-event`: observations of one movement of money on one account, from different documents. They agree
//! on the account (by its mask, the only account key every source shows), the amount and commodity, and their
//! dates fall within a few days (a card dinner is dated on the receipt and posted the next day).

use ownpurse_facts::{Db, Eid, EntRef, Kw, TxData, ValIn, Value};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};

/// How far apart two witnesses' dates can be.
pub const MAX_DAYS: i64 = 4;

fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

/// One observation that witnesses a movement of money.
#[derive(Debug, Clone)]
pub struct Witness {
    pub e: Eid,
    pub id: String,
    pub mask: String,
    pub amount: Decimal,
    pub commodity: Option<Eid>,
    pub day: i64,
    /// the document it belongs to (itself, when the observation is the document), so two lines of one
    /// statement are never linked to each other
    pub doc: Eid,
    pub source: Option<Eid>,
    pub text: String,
}

fn day(v: &Value) -> Option<i64> {
    let Value::Date(d) = v else { return None };
    let s = d.as_str();
    let (y, m, dd) = (
        s[0..4].parse().ok()?,
        s[5..7].parse().ok()?,
        s[8..10].parse().ok()?,
    );
    Some(ownpurse_facts::time::days_from_civil(y, m, dd))
}

/// Observations with an account, an amount and a date, of the kinds that are money moving on an account.
pub fn witnesses(db: &Db) -> Vec<Witness> {
    let kinds = [
        "obs.kind/money-movement",
        "obs.kind/transfer-leg",
        "obs.kind/statement-line",
    ];
    let Some(kind_attr) = db.attr(&k("obs/kind")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for kind in kinds {
        let Ok(kw) = Kw::new(kind) else { continue };
        for e in db.with_value(kind_attr.id, &Value::Kw(kw)) {
            let get = |a: &'static str| db.get(e, &k(a));
            let (Some(Value::Ref(acct)), Some(Value::Dec(amount)), Some(date)) =
                (get("obs/account"), get("obs/amount"), get("obs/date"))
            else {
                continue;
            };
            let Some(Value::Str(mask)) = db.get(*acct, &k("account/mask")) else {
                continue;
            };
            let Some(day) = day(date) else { continue };
            let doc = match get("obs/doc") {
                Some(Value::Ref(d)) => *d,
                _ => match get("obs/parent") {
                    Some(Value::Ref(p)) => *p,
                    _ => e,
                },
            };
            let source = [doc, e]
                .iter()
                .find_map(|x| db.get(*x, &k("doc/source")).and_then(Value::as_ref));
            let text = ["obs/counterparty", "obs/description"]
                .iter()
                .find_map(|a| {
                    db.get(e, &Kw::lit(a))
                        .and_then(Value::as_str)
                        .map(str::to_string)
                })
                .unwrap_or_default();
            out.push(Witness {
                e,
                id: get("obs/id")
                    .map(Value::text)
                    .unwrap_or_else(|| e.to_string()),
                mask: mask.to_string(),
                amount: *amount,
                // a payment or a transfer leg often names no currency: it is the account's
                commodity: get("obs/commodity")
                    .or_else(|| db.get(*acct, &k("account/commodity")))
                    .and_then(Value::as_ref),
                day,
                doc,
                source,
                text,
            });
        }
    }
    out
}

fn letters(s: &str) -> BTreeSet<String> {
    s.split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|w| w.len() >= 3)
        .map(str::to_ascii_uppercase)
        .collect()
}

/// 1.00 for the same day and a shared word, less for each day apart and for no shared word.
fn confidence(a: &Witness, b: &Witness) -> Decimal {
    let days = Decimal::from((a.day - b.day).abs());
    let mut c = Decimal::ONE - days * Decimal::new(5, 2);
    if letters(&a.text).is_disjoint(&letters(&b.text)) {
        c -= Decimal::new(10, 2);
    }
    c.max(Decimal::new(50, 2))
}

/// Which stream a witness comes from: its source and its document kind. Within one stream every witness is
/// its own event (two identical coffees are two lines); the same event shows up once per stream.
fn stream(db: &Db, w: &Witness) -> (Option<Eid>, Option<Value>) {
    (w.source, db.get(w.doc, &k("doc/kind")).cloned())
}

/// The `same-event` links: within each (account, amount) group, the witnesses of every two streams are
/// matched one to one, nearest dates first, so two identical coffees pair with two identical bank lines and
/// never with each other.
pub fn same_event(db: &Db) -> TxData {
    let mut groups: BTreeMap<(String, Decimal), Vec<Witness>> = BTreeMap::new();
    for w in witnesses(db) {
        groups
            .entry((w.mask.clone(), w.amount.normalize()))
            .or_default()
            .push(w);
    }
    let mut tx = TxData::new();
    for ws in groups.values() {
        let mut streams: BTreeMap<(Option<Eid>, Option<Value>), Vec<&Witness>> = BTreeMap::new();
        for w in ws {
            streams.entry(stream(db, w)).or_default().push(w);
        }
        let streams: Vec<Vec<&Witness>> = streams.into_values().collect();
        for (i, xs) in streams.iter().enumerate() {
            for ys in &streams[i + 1..] {
                let mut cand: Vec<(i64, &Witness, &Witness)> = xs
                    .iter()
                    .flat_map(|x| ys.iter().map(move |y| ((x.day - y.day).abs(), *x, *y)))
                    .filter(|(d, x, y)| *d <= MAX_DAYS && x.commodity == y.commodity)
                    .collect();
                cand.sort_by_key(|(d, x, y)| (*d, x.e, y.e));
                let mut used: BTreeSet<Eid> = BTreeSet::new();
                for (_, x, y) in cand {
                    if used.contains(&x.e) || used.contains(&y.e) {
                        continue;
                    }
                    used.insert(x.e);
                    used.insert(y.e);
                    link(&mut tx, x, y);
                }
            }
        }
    }
    tx
}

fn link(tx: &mut TxData, x: &Witness, y: &Witness) {
    let (x, y) = if x.id <= y.id { (x, y) } else { (y, x) };
    let id = format!("same-event|{}|{}", x.id, y.id);
    let l = EntRef::Temp(format!("link:{id}"));
    tx.add(&l, &k("link/id"), Value::str(&id));
    tx.add(&l, &k("link/kind"), Value::Kw(k("link.kind/same-event")));
    tx.add(&l, &k("link/from"), ValIn::Val(Value::Ref(x.e)));
    tx.add(&l, &k("link/to"), ValIn::Val(Value::Ref(y.e)));
    tx.add(&l, &k("link/confidence"), Value::Dec(confidence(x, y)));
    tx.add(&l, &k("link/rule"), Value::str("same-event@1"));
}
