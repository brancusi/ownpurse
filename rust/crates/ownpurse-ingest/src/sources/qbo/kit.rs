//! What the QuickBooks and FreshBooks interpretations share: naming the shared things (source, file, book,
//! commodity, a referenced document) in a snapshot, observations, mirror entries and in-source links.
//!
//! The Xero projector has the same helpers as private methods; this is a copy shaped for reuse. The edges
//! doc (`docs/plans/edges/qbo-freshbooks.md`) proposes moving it into the crate for every projector.

use crate::jsonmirror::doc_id;
use crate::snapshot::Snapshot;
use crate::{Input, Pressure, temp};
use ownpurse_facts::{Db, Kw, ValIn, Value};
use rust_decimal::Decimal;
use serde_json::{Map, Value as Json};

pub(crate) fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

/// A keyword value from source text: lower case, anything but letters and digits becomes `-`.
/// `Accounts Receivable` → `account.type/accounts-receivable`.
pub(crate) fn kw(ns: &str, name: &str) -> Option<Value> {
    let mut n = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            n.push(c.to_ascii_lowercase());
        } else if !n.ends_with('-') {
            n.push('-');
        }
    }
    let n = n.trim_matches('-');
    Kw::new(&format!("{ns}/{n}")).ok().map(Value::Kw)
}

/// A non-empty string field.
pub(crate) fn s<'j>(o: &'j Map<String, Json>, f: &str) -> Option<&'j str> {
    o.get(f).and_then(Json::as_str).filter(|t| !t.is_empty())
}

/// A decimal field, written as a JSON number or as text.
pub(crate) fn dec(o: &Map<String, Json>, f: &str) -> Option<Decimal> {
    dec_of(o.get(f)?)
}

pub(crate) fn dec_of(v: &Json) -> Option<Decimal> {
    match v {
        Json::Number(n) => ownpurse_facts::value::parse_decimal(&n.to_string()).ok(),
        Json::String(t) => ownpurse_facts::value::parse_decimal(t).ok(),
        _ => None,
    }
}

/// One leg of a mirror entry: the account's snapshot key (if known), the signed quantity (debit positive),
/// a memo.
pub(crate) type Leg = (Option<String>, Decimal, String);

pub(crate) struct Kit<'a, 'i> {
    pub db: &'a Db,
    pub input: &'a Input<'i>,
    pub snap: Snapshot,
    pub pressure: Vec<Pressure>,
    /// written as `link/rule` on the links this projector makes
    pub rule: &'static str,
}

impl Kit<'_, '_> {
    pub fn src(&self) -> &str {
        &self.input.source.id
    }

    pub fn press(&mut self, kind: &str, at: &str, detail: impl Into<String>) {
        self.pressure.push(Pressure::new(kind, at, detail));
    }

    pub fn add(&mut self, key: &str, a: &'static str, v: impl Into<ValIn>) {
        self.snap.add(key, &k(a), v);
    }

    /// A document of this source by kind and id, named but not described: its own file says what is true
    /// about it. Never `add` to the key this returns (that would make the snapshot own it, and a re-read of
    /// this file would retract everything its own file said).
    pub fn refer(&mut self, kind: &str, id: &str) -> String {
        let did = doc_id(self.src(), kind, id);
        let key = format!("doc:{did}");
        self.snap.refer(&key, &k("doc/id"), Value::str(&did));
        key
    }

    /// A document's snapshot key from its doc id, when the db already knows it.
    pub fn refer_doc(&mut self, did: &str) -> String {
        let key = format!("doc:{did}");
        self.snap.refer(&key, &k("doc/id"), Value::str(did));
        key
    }

    pub fn commodity(&mut self, code: &str) -> String {
        let key = format!("commodity:{code}");
        self.snap
            .refer(&key, &k("commodity/code"), Value::str(code));
        key
    }

    pub fn source_key(&mut self) -> String {
        let key = format!("source:{}", self.src());
        self.snap
            .refer(&key, &k("source/id"), Value::str(self.src()));
        key
    }

    pub fn file_key(&mut self) -> String {
        let h = self.input.file.hash;
        let key = format!("file:{}", h.hex());
        self.snap.refer(&key, &k("file/sha256"), Value::Hash(h));
        key
    }

    /// The source's own book: `book/id` is the source id.
    pub fn book(&mut self) -> String {
        let key = format!("book:{}", self.src());
        self.snap.refer(&key, &k("book/id"), Value::str(self.src()));
        key
    }

    /// Describe the book: its name, and that the source owns its entries (a mirror).
    pub fn describe_book(&mut self, name: &str) {
        let book = self.book();
        self.add(&book, "book/name", Value::str(name));
        let src = self.source_key();
        self.add(&book, "book/authority", ValIn::Ref(temp(&src)));
    }

    /// The shared envelope on every top-level document: kind, external id, source, file.
    pub fn document(&mut self, key: &str, kind: &str, external_id: &str) {
        self.add(key, "doc/external-id", Value::str(external_id));
        if let Some(kv) = kw("doc.kind", kind) {
            self.add(key, "doc/kind", kv);
        }
        let (src, file) = (self.source_key(), self.file_key());
        self.add(key, "doc/source", ValIn::Ref(temp(&src)));
        self.add(key, "doc/file", ValIn::Ref(temp(&file)));
    }

    pub fn observe(&mut self, key: &str, id: &str, kind: &str) {
        self.add(key, "obs/id", Value::str(id));
        if let Some(kv) = kw("obs.kind", kind) {
            self.add(key, "obs/kind", kv);
        }
    }

    /// A line under a parent observation.
    pub fn line(&mut self, key: &str, id: &str, parent: &str, index: usize) {
        self.observe(key, id, "line");
        self.add(key, "obs/parent", ValIn::Ref(temp(parent)));
        self.add(
            key,
            "obs/index",
            Value::Int(i64::try_from(index).unwrap_or(i64::MAX)),
        );
    }

    /// Say this document should have no mirror entry now (a draft, a voided one).
    pub fn no_entry(&mut self, doc: &str) {
        self.snap
            .gone(&k("entry/id"), Value::str(&format!("{doc}#entry")));
    }

    /// The mirror book's entry for a document, held to the source's authority, so posted as it says.
    pub fn entry(
        &mut self,
        doc_key: &str,
        doc: &str,
        date: Value,
        narration: &str,
        commodity: Option<&str>,
        legs: Vec<Leg>,
    ) {
        let eid = format!("{doc}#entry");
        let ek = format!("entry:{eid}");
        self.snap.ent(&ek, &k("entry/id"), Value::str(&eid));
        let book = self.book();
        self.add(&ek, "entry/book", ValIn::Ref(temp(&book)));
        self.add(&ek, "entry/date", date);
        self.add(&ek, "entry/status", Value::Kw(k("entry.status/posted")));
        if !narration.is_empty() {
            self.add(&ek, "entry/narration", Value::str(narration));
        }
        let ccy = commodity.map(|c| self.commodity(c));
        let mut sum = Decimal::ZERO;
        for (i, (acct, q, memo)) in legs.into_iter().enumerate() {
            let pid = format!("{eid}/{i}");
            let pk = format!("posting:{pid}");
            self.snap.ent(&pk, &k("posting/id"), Value::str(&pid));
            self.add(&ek, "entry/posting", ValIn::Ref(temp(&pk)));
            self.add(&pk, "posting/quantity", Value::Dec(q));
            self.add(
                &pk,
                "posting/index",
                Value::Int(i64::try_from(i).unwrap_or(i64::MAX)),
            );
            sum += q;
            match acct {
                Some(a) => self.add(&pk, "posting/account", ValIn::Ref(temp(&a))),
                None => self.press(
                    "unresolved-ref",
                    &pid,
                    "a posting without an account: the document did not say which, and no rule found one",
                ),
            }
            if let Some(c) = &ccy {
                self.add(&pk, "posting/commodity", ValIn::Ref(temp(c)));
            }
            if !memo.is_empty() {
                self.add(&pk, "posting/memo", Value::str(&memo));
            }
        }
        if !sum.is_zero() {
            self.press(
                "unbalanced",
                doc,
                format!("the mirror entry is off by {sum}"),
            );
        }
        self.link("mirrors", (&ek, &eid), (doc_key, doc));
    }

    /// A link inside this source (a payment settles an invoice; an entry mirrors a document).
    pub fn link(&mut self, kind: &str, from: (&str, &str), to: (&str, &str)) -> String {
        let id = format!("{kind}|{}|{}", from.1, to.1);
        let lk = format!("link:{id}");
        self.snap.ent(&lk, &k("link/id"), Value::str(&id));
        if let Some(kv) = kw("link.kind", kind) {
            self.add(&lk, "link/kind", kv);
        }
        self.add(&lk, "link/from", ValIn::Ref(temp(from.0)));
        self.add(&lk, "link/to", ValIn::Ref(temp(to.0)));
        self.add(&lk, "link/rule", Value::str(self.rule));
        lk
    }

    /// The doc id of the entity a ref attribute on `e` points at.
    pub fn target_doc(&self, e: ownpurse_facts::Eid, attr: &Kw) -> Option<String> {
        let t = self.db.get(e, attr)?.as_ref()?;
        self.db
            .get(t, &k("doc/id"))
            .and_then(Value::as_str)
            .map(str::to_string)
    }
}

/// Attributes a projector owns on the entities it describes: everything in its own namespace and the
/// interpretation it writes. `doc/file` and `doc/source` accumulate; notes and links by people or rules
/// are never touched by a re-read.
pub(crate) fn owns(ns_prefix: &str, a: &Kw) -> bool {
    let ns = a.ns();
    ns == ns_prefix
        || ns
            .strip_prefix(ns_prefix)
            .is_some_and(|r| r.starts_with('.'))
        || matches!(
            ns,
            "obs" | "account" | "party" | "entry" | "posting" | "book" | "link"
        )
        || matches!(a.as_str(), "doc/kind" | "doc/external-id")
}
