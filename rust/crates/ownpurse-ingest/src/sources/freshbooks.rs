//! FreshBooks: API responses (`{"response": {"result": {"invoices": [...], "page": 1, …}}}`), one list per
//! file, plus the identity response (`/auth/api/v1/users/me`) that names the business.
//!
//! A normalising pass turns FreshBooks' plain-number foreign keys (`customerid: 12`) into references the
//! generic mirror understands and notes what FreshBooks' JSON does that the model does not (money objects,
//! integer enums, two ids for one thing, wall-clock times with no zone, zero meaning none). The mirror keeps
//! every field in FreshBooks' words (`freshbooks.invoice/invoice_number`); the interpretation says what each
//! document means from Saltmarsh's side: invoices, payments, expenses. FreshBooks' invoices, payments and
//! expenses name no ledger accounts, so this projector writes no mirror entries.

use crate::jsonmirror::{self, JsonSource, Root, doc_id};
use crate::sources::qbo::kit::{self, Kit, dec_of, k, kw, s};
use crate::{IngestError, Input, Pressure, Projection, Projector, temp};
use ownpurse_facts::{Date, Db, Instant, Kw, ValIn, Value};
use rust_decimal::Decimal;
use serde_json::{Map, Value as Json};
use std::collections::BTreeSet;

pub struct FreshBooks;

const NS: &str = "freshbooks";

/// Plain-number fields that name another list's item: (field, kind).
const FOREIGN: &[(&str, &str)] = &[
    ("customerid", "client"),
    ("clientid", "client"),
    ("invoiceid", "invoice"),
    ("expenseid", "expense"),
    ("categoryid", "category"),
    ("staffid", "staff"),
    ("projectid", "project"),
];

/// The second id each kind carries for itself, beside `id`.
const OWN_ID: &[(&str, &str)] = &[
    ("invoice", "invoiceid"),
    ("expense", "expenseid"),
    ("payment", "logid"),
    ("client", "userid"),
];

pub static CONFIG: JsonSource = JsonSource {
    ns: NS,
    root: &["response", "result"],
    collections: &[
        ("invoices", "invoice", "id"),
        ("payments", "payment", "id"),
        ("clients", "client", "id"),
        ("expenses", "expense", "id"),
        ("categories", "category", "id"),
    ],
    ref_ids: &[
        ("@client", "client"),
        ("@invoice", "invoice"),
        ("@expense", "expense"),
        ("@category", "category"),
        ("@staff", "staff"),
        ("@project", "project"),
    ],
    owned: &[],
    child_ids: &["lineid"],
    when,
};

/// The identity response: `{"response": {"id": …, "business_memberships": [...]}}`.
pub static IDENTITY: JsonSource = JsonSource {
    ns: NS,
    root: &["response"],
    collections: &[("business_memberships", "business_membership", "id")],
    ref_ids: &[],
    owned: &[],
    child_ids: &[],
    when,
};

/// Fields FreshBooks sends as text that are numbers.
const NUMERIC_TEXT: &[&str] = &[
    "amount",
    "qty",
    "taxAmount1",
    "taxAmount2",
    "taxPercent1",
    "taxPercent2",
    "markup_percent",
    "discount_value",
];

/// FreshBooks' dates: `create_date`, `due_date`, `date` are `YYYY-MM-DD`. Wall-clock times
/// (`2026-03-11 14:02:51`) carry no zone and stay text. The identity API's times are UTC (`…Z`).
fn when(field: &str, text: &str) -> Option<Value> {
    if NUMERIC_TEXT.contains(&field) {
        return ownpurse_facts::value::parse_decimal(text)
            .ok()
            .map(Value::Dec);
    }
    let datey = field == "date" || field.ends_with("_date") || field.starts_with("date_");
    if datey && text.len() == 10 {
        return Date::parse(text).ok().map(Value::Date);
    }
    if text.ends_with('Z') {
        return Instant::parse(text).ok().map(Value::Instant);
    }
    None
}

fn is_wall_clock(t: &str) -> bool {
    let b = t.as_bytes();
    b.len() == 19 && b[10] == b' ' && b[13] == b':' && Date::parse(&t[..10]).is_ok()
}

fn is_money(v: &Json) -> bool {
    v.as_object().is_some_and(|m| {
        m.len() == 2 && m.get("amount").is_some_and(Json::is_string) && m.contains_key("code")
    })
}

struct Normaliser {
    pressure: Vec<Pressure>,
    seen: BTreeSet<(String, String)>,
}

impl Normaliser {
    fn once(&mut self, kind: &str, at: String, detail: impl Into<String>) {
        if self.seen.insert((kind.to_string(), at.clone())) {
            self.pressure.push(Pressure::new(kind, at, detail));
        }
    }

    /// `at` is the attribute namespace this object's fields land in (`freshbooks.invoice.lines`).
    fn walk(&mut self, obj: &mut Map<String, Json>, at: &str, own_id: Option<&str>) {
        let fields: Vec<String> = obj.keys().cloned().collect();
        for f in fields {
            let here = format!("{at}/{f}");
            if Some(f.as_str()) == own_id {
                self.once(
                    "two-ids",
                    here.clone(),
                    "the same id twice (id and this): both mirrored; id is the identity",
                );
                continue;
            }
            if let Some((_, kind)) = FOREIGN.iter().find(|(name, _)| *name == f) {
                self.foreign(obj, &f, kind, &here);
                continue;
            }
            let Some(v) = obj.get_mut(&f) else { continue };
            match v {
                Json::Object(child) => {
                    if is_money(&Json::Object(child.clone())) {
                        self.once(
                            "money-object",
                            here.clone(),
                            "money as {amount, code}: mirrored as a component entity per amount; read as obs/amount plus obs/commodity",
                        );
                    }
                    self.walk(child, &format!("{at}.{f}"), None);
                }
                Json::Array(items) => {
                    for it in items.iter_mut().filter_map(Json::as_object_mut) {
                        self.walk(it, &format!("{at}.{f}"), None);
                    }
                }
                Json::String(t) if is_wall_clock(t) => self.once(
                    "no-zone",
                    here.clone(),
                    "a wall-clock time with no zone (FreshBooks keeps these in the account's zone, or US Eastern for older fields): kept as text",
                ),
                Json::Number(_) if matches!(f.as_str(), "status" | "vis_state") => self.once(
                    "enum-int",
                    here.clone(),
                    "an enumeration sent as an integer: mirrored as a number; its meaning is in FreshBooks' docs, not the data",
                ),
                _ => {}
            }
        }
    }

    fn foreign(&mut self, obj: &mut Map<String, Json>, f: &str, kind: &str, at: &str) {
        let id = match obj.get(f) {
            Some(Json::Number(n)) => n.to_string(),
            Some(Json::String(t)) if !t.is_empty() => t.clone(),
            _ => return,
        };
        if id == "0" {
            self.once(
                "zero-as-none",
                at.to_string(),
                "0 means no such thing: kept as the number 0, not a reference",
            );
            return;
        }
        self.once(
            "scalar-ref",
            at.to_string(),
            format!("a plain number naming another document ({kind}): read as a ref to it"),
        );
        let mut r = Map::new();
        r.insert(format!("@{kind}"), Json::String(id));
        obj.insert(f.to_string(), Json::Object(r));
    }
}

fn normalise(body: &mut Json) -> Vec<Pressure> {
    let mut n = Normaliser {
        pressure: Vec::new(),
        seen: BTreeSet::new(),
    };
    let Some(result) = body
        .get_mut("response")
        .and_then(|r| r.get_mut("result"))
        .and_then(Json::as_object_mut)
    else {
        return n.pressure;
    };
    let keys: Vec<String> = result.keys().cloned().collect();
    for key in keys {
        let Some((_, kind, _)) = CONFIG.collections.iter().find(|(l, _, _)| *l == key) else {
            n.once(
                "envelope",
                format!("/response/result/{key}"),
                "paging (page, pages, per_page, total): not stored, so a file cannot say it was the whole list",
            );
            continue;
        };
        let own = OWN_ID.iter().find(|(k, _)| k == kind).map(|(_, f)| *f);
        let at = format!("{NS}.{kind}");
        for it in result
            .get_mut(&key)
            .and_then(Json::as_array_mut)
            .into_iter()
            .flatten()
            .filter_map(Json::as_object_mut)
        {
            n.walk(it, &at, own);
            if *kind == "invoice" && it.contains_key("organization") {
                n.once(
                    "flat-copy",
                    at.clone(),
                    "the client's name and address are copied onto the invoice as plain fields (fname, organization, street, …) beside customerid: mirrored as the invoice's own",
                );
            }
        }
    }
    n.pressure
}

impl Projector for FreshBooks {
    fn id(&self) -> &'static str {
        "freshbooks.json@1"
    }

    fn reads(&self) -> &'static str {
        "FreshBooks API responses (invoices with lines, payments, clients, expenses) and the users/me identity"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let mut body: Json = serde_json::from_slice(input.file.bytes)
            .map_err(|e| IngestError(format!("{}: not JSON ({e})", input.file.name)))?;
        let identity = body
            .get("response")
            .is_some_and(|r| r.get("result").is_none() && r.get("business_memberships").is_some());
        let mut pressure = normalise(&mut body);
        let cfg = if identity { &IDENTITY } else { &CONFIG };
        if identity {
            pressure.push(Pressure::new(
                "envelope",
                "/response",
                "the signed-in person (id, name, email) sits outside the memberships: not stored",
            ));
        }
        let m = jsonmirror::mirror(cfg, db, input, &body)?;
        pressure.extend(m.pressure);
        let mut it = Interp(Kit {
            db,
            input,
            snap: m.snap,
            pressure,
            rule: "freshbooks.json@1",
        });
        for root in &m.roots {
            it.root(root);
        }
        let Interp(kit) = it;
        let tx = kit.snap.diff(db, &|a: &Kw| kit::owns(NS, a));
        Ok(Projection {
            install: m.install,
            tx,
            pressure: kit.pressure,
        })
    }
}

/// A money object: `{"amount": "362.18", "code": "USD"}`.
fn money<'j>(o: &'j Map<String, Json>, f: &str) -> Option<(Decimal, &'j str)> {
    let m = o.get(f)?.as_object()?;
    let a = dec_of(m.get("amount")?)?;
    let c = m.get("code").and_then(Json::as_str)?;
    Some((a, c))
}

/// The id a rewritten foreign key names.
fn fk<'j>(o: &'j Map<String, Json>, f: &str) -> Option<&'j str> {
    o.get(f)?
        .as_object()?
        .values()
        .next()
        .and_then(Json::as_str)
}

struct Interp<'a, 'i>(Kit<'a, 'i>);

impl Interp<'_, '_> {
    fn root(&mut self, r: &Root<'_>) {
        let key = r.key.clone();
        self.0
            .document(&key, &format!("freshbooks-{}", r.kind), &r.id);
        let o = r.json;
        match r.kind {
            "business_membership" => {
                if let Some(n) = o
                    .get("business")
                    .and_then(|b| b.get("name"))
                    .and_then(Json::as_str)
                {
                    self.0.describe_book(n);
                }
            }
            "client" => {
                self.0.add(&key, "party/id", Value::str(&r.doc_id));
                if let Some(n) = client_name(o) {
                    self.0.add(&key, "party/name", Value::str(&n));
                }
            }
            "invoice" => self.invoice(r),
            "payment" => self.payment(r),
            "expense" => self.expense(r),
            _ => {}
        }
        if matches!(r.kind, "invoice" | "payment" | "expense") {
            self.0.press(
                "no-ledger",
                &format!("{NS}.{}", r.kind),
                "names no ledger account (FreshBooks keeps its chart behind its reports): observations only, no mirror entry",
            );
        }
    }

    fn amount(&mut self, key: &str, o: &Map<String, Json>, sign: Decimal) -> Option<Decimal> {
        let (a, code) = money(o, "amount")?;
        self.0.add(key, "obs/amount", Value::Dec(sign * a));
        let c = self.0.commodity(code);
        self.0.add(key, "obs/commodity", ValIn::Ref(temp(&c)));
        Some(a)
    }

    fn date(&mut self, key: &str, o: &Map<String, Json>, f: &str) {
        if let Some(d) = s(o, f).and_then(|t| Date::parse(t).ok()) {
            self.0.add(key, "obs/date", Value::Date(d));
        }
    }

    fn invoice(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        self.0.observe(&key, &r.doc_id, "invoice");
        self.amount(&key, o, Decimal::ONE);
        self.date(&key, o, "create_date");
        if let Some(n) = s(o, "invoice_number") {
            self.0.add(&key, "obs/reference", Value::str(n));
        }
        if let Some(n) = s(o, "current_organization").or_else(|| s(o, "organization")) {
            self.0.add(&key, "obs/counterparty", Value::str(n));
        }
        if let Some(st) = s(o, "v3_status").and_then(|t| kw("obs.status", t)) {
            self.0.add(&key, "obs/status", st);
        }
        if o.contains_key("outstanding") {
            self.0.press(
                "derived-value",
                "freshbooks.invoice/outstanding",
                "outstanding, paid, payment_status and the statuses move when a payment arrives: mirrored, so a payment reads as an edit of the invoice",
            );
        }
        if o.contains_key("due_offset_days") {
            self.0.press(
                "derived-value",
                "freshbooks.invoice/due_date",
                "due_date is create_date plus due_offset_days: both mirrored",
            );
        }
        for (i, l) in o
            .get("lines")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let Some(l) = l.as_object() else { continue };
            let seg = l
                .get("lineid")
                .map(|v| v.to_string().trim_matches('"').to_string())
                .unwrap_or_else(|| i.to_string());
            let lk = format!("{key}/lines/{seg}");
            if !self.0.snap.has(&lk) {
                continue;
            }
            self.0
                .line(&lk, &format!("{}/lines/{seg}", r.doc_id), &key, i);
            self.amount(&lk, l, Decimal::ONE);
            if let Some(n) = s(l, "name") {
                self.0.add(&lk, "obs/description", Value::str(n));
            }
        }
    }

    fn payment(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        self.0.observe(&key, &r.doc_id, "payment");
        let amount = self.amount(&key, o, Decimal::ONE);
        self.date(&key, o, "date");
        if let Some(t) = s(o, "type") {
            self.0.add(&key, "obs/description", Value::str(t));
        }
        if let Some(n) = s(o, "note") {
            self.0.add(&key, "obs/reference", Value::str(n));
        }
        // the payment names its client by id only: the name comes from the client, if already read
        if let Some(c) = fk(o, "clientid") {
            let did = doc_id(self.0.src(), "client", c);
            let name = self
                .0
                .db
                .lookup(&k("doc/id"), &Value::str(&did))
                .and_then(|e| self.0.db.get(e, &k("party/name")))
                .and_then(Value::as_str)
                .map(str::to_string);
            match name {
                Some(n) => self.0.add(&key, "obs/counterparty", Value::str(&n)),
                None => self.0.press(
                    "unresolved-ref",
                    "freshbooks.payment/clientid",
                    format!("client {c} is not known yet (ingest clients first)"),
                ),
            }
        }
        if let Some(inv) = fk(o, "invoiceid") {
            let tk = self.0.refer("invoice", inv);
            let tid = doc_id(self.0.src(), "invoice", inv);
            let lk = self.0.link("settles", (&key, &r.doc_id), (&tk, &tid));
            if let Some(a) = amount {
                self.0.add(&lk, "link/amount", Value::Dec(a));
            }
        }
    }

    fn expense(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        self.0.observe(&key, &r.doc_id, "money-movement");
        self.amount(&key, o, Decimal::NEGATIVE_ONE);
        self.date(&key, o, "date");
        if let Some(v) = s(o, "vendor") {
            self.0.add(&key, "obs/counterparty", Value::str(v));
        }
        if let Some(n) = s(o, "notes") {
            self.0.add(&key, "obs/description", Value::str(n));
        }
        if o.get("accountid").is_none_or(Json::is_null) {
            self.0.press(
                "missing-context",
                "freshbooks.expense/accountid",
                "no account the money left from: the observation has no obs/account, so no bank line can be matched by account",
            );
        }
    }
}

fn client_name(o: &Map<String, Json>) -> Option<String> {
    if let Some(org) = s(o, "organization") {
        return Some(org.to_string());
    }
    let full = [s(o, "fname"), s(o, "lname")]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join(" ");
    (!full.is_empty()).then_some(full)
}
