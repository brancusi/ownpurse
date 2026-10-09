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
    /// the projector that said it: two formats from one source (a CSV and an OFX) are two streams
    pub projector: Option<String>,
    pub reference: Option<String>,
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
/// The kinds that are money moving on an account: what `same-event` matches.
pub const MOVEMENTS: [&str; 4] = [
    "obs.kind/money-movement",
    "obs.kind/transfer-leg",
    "obs.kind/statement-line",
    "obs.kind/payment",
];

/// The kinds that back a movement without being one: a receipt, a check.
pub const EVIDENCE: [&str; 2] = ["obs.kind/receipt", "obs.kind/check"];

pub fn witnesses(db: &Db, kinds: &[&str]) -> Vec<Witness> {
    let Some(kind_attr) = db.attr(&k("obs/kind")) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for kind in kinds {
        let Ok(kw) = Kw::new(kind) else { continue };
        let kw_val = Value::Kw(kw.clone());
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
                projector: db
                    .entity(e)
                    .and_then(|m| m.get(&kind_attr.id))
                    .and_then(|vs| vs.get(&kw_val))
                    .and_then(|tx| db.get(*tx, &k("tx/projector")))
                    .map(Value::text),
                reference: get("obs/reference").map(Value::text),
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
type Stream = (Option<Eid>, Option<Value>, Option<String>);

fn stream(db: &Db, w: &Witness) -> Stream {
    (
        w.source,
        db.get(w.doc, &k("doc/kind")).cloned(),
        w.projector.clone(),
    )
}

/// The `same-event` links: within each (account, amount) group, the witnesses of every two streams are
/// matched one to one, nearest dates first, so two identical coffees pair with two identical bank lines and
/// never with each other.
pub fn same_event(db: &Db) -> TxData {
    let mut groups: BTreeMap<(String, Decimal), Vec<Witness>> = BTreeMap::new();
    for w in witnesses(db, &MOVEMENTS) {
        groups
            .entry((w.mask.clone(), w.amount.normalize()))
            .or_default()
            .push(w);
    }
    let mut tx = TxData::new();
    for ws in groups.values() {
        let mut streams: BTreeMap<Stream, Vec<&Witness>> = BTreeMap::new();
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
    propose(
        tx,
        "same-event",
        "same-event@1",
        (x.e, &x.id),
        (y.e, &y.id),
        confidence(x, y),
    );
}

/// One link fact set; its id is `<kind>|<from id>|<to id>`, so proposing it again is a no-op.
fn propose(
    tx: &mut TxData,
    kind: &str,
    rule: &str,
    from: (Eid, &str),
    to: (Eid, &str),
    confidence: Decimal,
) {
    let id = format!("{kind}|{}|{}", from.1, to.1);
    let l = EntRef::Temp(format!("link:{id}"));
    tx.add(&l, &k("link/id"), Value::str(&id));
    if let Ok(kw) = Kw::new(&format!("link.kind/{kind}")) {
        tx.add(&l, &k("link/kind"), Value::Kw(kw));
    }
    tx.add(&l, &k("link/from"), ValIn::Val(Value::Ref(from.0)));
    tx.add(&l, &k("link/to"), ValIn::Val(Value::Ref(to.0)));
    tx.add(&l, &k("link/confidence"), Value::Dec(confidence));
    tx.add(&l, &k("link/rule"), Value::str(rule));
}

fn digits(s: &str) -> String {
    let d: String = s.chars().filter(char::is_ascii_digit).collect();
    d.trim_start_matches('0').to_string()
}

/// `evidences`: a receipt or a check backs the bank lines it paid for. Same account and amount; a receipt
/// posts within `MAX_DAYS` after its date, a check clears up to 30 days after it is written, and a check
/// whose number the line names scores higher. One line per stream: the nearest one on or after the date.
pub fn evidences(db: &Db) -> TxData {
    let lines = witnesses(db, &MOVEMENTS);
    let mut tx = TxData::new();
    for ev in witnesses(db, &EVIDENCE) {
        let is_check = db.get(ev.e, &k("obs/kind")) == Some(&Value::Kw(k("obs.kind/check")));
        let window = if is_check { 30 } else { MAX_DAYS };
        let mut best: BTreeMap<Stream, (i64, &Witness)> = BTreeMap::new();
        for w in lines
            .iter()
            .filter(|w| w.mask == ev.mask && w.amount.normalize() == ev.amount.normalize())
        {
            let lag = w.day - ev.day;
            if !(0..=window).contains(&lag) {
                continue;
            }
            let s = stream(db, w);
            if best.get(&s).is_none_or(|(l, _)| lag < *l) {
                best.insert(s, (lag, w));
            }
        }
        for (lag, w) in best.into_values() {
            let named = ev
                .reference
                .as_deref()
                .map(digits)
                .filter(|d| !d.is_empty());
            let by_number = named.is_some_and(|n| {
                digits(&w.text).contains(&n) || w.reference.as_deref().map(digits) == Some(n)
            });
            let mut c = Decimal::ONE - Decimal::from(lag) * Decimal::new(1, 2);
            if is_check && !by_number {
                c -= Decimal::new(20, 2);
            }
            propose(
                &mut tx,
                "evidences",
                "evidences@1",
                (ev.e, &ev.id),
                (w.e, &w.id),
                c.max(Decimal::new(50, 2)),
            );
        }
    }
    tx
}

/// `counterpart`: one invoice seen from both sides, in two books. The issuer's invoice and the payer's bill
/// have opposite amounts, references with the same digits ("1031" and "QF-1031"), and dates within a week.
pub fn counterpart(db: &Db) -> TxData {
    let mut docs: Vec<(Eid, String, Decimal, i64, String, Option<Eid>)> = Vec::new();
    let Some(kind_attr) = db.attr(&k("obs/kind")) else {
        return TxData::new();
    };
    for kind in ["obs.kind/invoice", "obs.kind/bill"] {
        for e in db.with_value(kind_attr.id, &Value::Kw(k(kind))) {
            let get = |a: &'static str| db.get(e, &k(a));
            let (Some(Value::Dec(amount)), Some(date), Some(r)) =
                (get("obs/amount"), get("obs/date"), get("obs/reference"))
            else {
                continue;
            };
            let Some(d) = day(date) else { continue };
            let id = get("obs/id").map(Value::text).unwrap_or_default();
            let src = [get("obs/doc").and_then(Value::as_ref), Some(e)]
                .into_iter()
                .flatten()
                .find_map(|x| db.get(x, &k("doc/source")).and_then(Value::as_ref));
            docs.push((e, id, *amount, d, digits(&r.text()), src));
        }
    }
    let mut tx = TxData::new();
    for (i, a) in docs.iter().enumerate() {
        for b in &docs[i + 1..] {
            let opposite = (a.2 + b.2).is_zero() && !a.2.is_zero();
            if a.5 != b.5 && opposite && !a.4.is_empty() && a.4 == b.4 && (a.3 - b.3).abs() <= 7 {
                let (x, y) = if a.1 <= b.1 { (a, b) } else { (b, a) };
                propose(
                    &mut tx,
                    "counterpart",
                    "counterpart@1",
                    (x.0, &x.1),
                    (y.0, &y.1),
                    Decimal::new(95, 2),
                );
            }
        }
    }
    tx
}

/// Every rule, in one transaction's worth of proposals.
pub fn all(db: &Db) -> TxData {
    let mut tx = same_event(db);
    tx.extend(evidences(db));
    tx.extend(counterpart(db));
    tx
}
