//! QuickBooks Online: v3 API query responses (`{"QueryResponse": {"Invoice": [...]}, "time": ...}`), one
//! entity type per file.
//!
//! Three steps. A normalising pass rewrites QuickBooks' references (`"CustomerRef": {"value": "58"}`, where
//! the field name says what is referenced, and `LinkedTxn`/`EntityRef`, where a sibling value does) into the
//! shape the generic mirror understands, and notes what QuickBooks' JSON does that the model does not
//! (offsets on timestamps, the response envelope). The mirror then keeps every field in QuickBooks' own words
//! (`qbo.Invoice/DocNumber`). The interpretation says what each document means in the shared vocabulary,
//! from Quillfeather's side of its own books: invoices, payments, purchases, and the mirror book's entries.

pub(crate) mod kit;

use crate::jsonmirror::{self, JsonSource, Root, doc_id};
use crate::{IngestError, Input, Pressure, Projection, Projector};
use kit::{Kit, Leg, dec, k, kw, s};
use ownpurse_facts::{Date, Db, Eid, Instant, Kw, ValIn, Value};
use rust_decimal::Decimal;
use serde_json::{Map, Value as Json};
use std::collections::BTreeSet;

pub struct Qbo;

const NS: &str = "qbo";

/// Kinds a reference can name. A reference is rewritten to `{"@<Kind>": id}` so the mirror's `ref_ids`
/// (which find a reference by an id field it holds) can see it.
static REF_IDS: &[(&str, &str)] = &[
    ("@Account", "Account"),
    ("@Bill", "Bill"),
    ("@BillPayment", "BillPayment"),
    ("@Class", "Class"),
    ("@CreditMemo", "CreditMemo"),
    ("@Currency", "Currency"),
    ("@Customer", "Customer"),
    ("@Department", "Department"),
    ("@Deposit", "Deposit"),
    ("@Employee", "Employee"),
    ("@Estimate", "Estimate"),
    ("@Invoice", "Invoice"),
    ("@Item", "Item"),
    ("@JournalEntry", "JournalEntry"),
    ("@Payment", "Payment"),
    ("@PaymentMethod", "PaymentMethod"),
    ("@Purchase", "Purchase"),
    ("@RefundReceipt", "RefundReceipt"),
    ("@SalesReceipt", "SalesReceipt"),
    ("@TaxCode", "TaxCode"),
    ("@TaxRate", "TaxRate"),
    ("@Term", "Term"),
    ("@Transfer", "Transfer"),
    ("@User", "User"),
    ("@Vendor", "Vendor"),
];

pub static CONFIG: JsonSource = JsonSource {
    ns: NS,
    root: &["QueryResponse"],
    collections: &[
        ("CompanyInfo", "CompanyInfo", "Id"),
        ("Account", "Account", "Id"),
        ("Customer", "Customer", "Id"),
        ("Vendor", "Vendor", "Id"),
        ("Item", "Item", "Id"),
        ("Invoice", "Invoice", "Id"),
        ("Payment", "Payment", "Id"),
        ("Purchase", "Purchase", "Id"),
        ("JournalEntry", "JournalEntry", "Id"),
        ("Bill", "Bill", "Id"),
        ("BillPayment", "BillPayment", "Id"),
        ("CreditMemo", "CreditMemo", "Id"),
        ("SalesReceipt", "SalesReceipt", "Id"),
        ("Deposit", "Deposit", "Id"),
        ("Transfer", "Transfer", "Id"),
    ],
    refs: &[],
    ref_ids: REF_IDS,
    owned: &[],
    // Not "Id": QuickBooks appends a subtotal line with no Id, and the mirror's position fallback would give it
    // the same key as the line whose Id equals its position (see the edges doc). Lines go by position.
    child_ids: &["DefinitionId"],
    when,
};

/// QuickBooks' dates: `TxnDate`, `DueDate`, … are `YYYY-MM-DD`; timestamps carry the company's offset
/// (`2026-03-04T09:18:52-08:00`) and are stored in UTC.
fn when(field: &str, text: &str) -> Option<Value> {
    if field.ends_with("Date") && text.len() == 10 {
        return Date::parse(text).ok().map(Value::Date);
    }
    if text.len() >= 20 && text.as_bytes().get(10) == Some(&b'T') {
        return Instant::parse_offset(text).ok().map(Value::Instant);
    }
    None
}

/// The kind a `…Ref` field names, from its field name (or, for `EntityRef`, its `type`).
fn ref_kind(field: &str, obj: &Map<String, Json>, root_kind: &str) -> Option<String> {
    let kind = match field {
        "EntityRef" => obj.get("type").and_then(Json::as_str)?.to_string(),
        "ParentRef" => root_kind.to_string(),
        "LastModifiedByRef" => "User".into(),
        "SalesTermRef" => "Term".into(),
        "TxnTaxCodeRef" => "TaxCode".into(),
        f if f.ends_with("AccountRef") => "Account".into(),
        f => f.strip_suffix("Ref")?.to_string(),
    };
    REF_IDS.iter().any(|(_, k)| *k == kind).then_some(kind)
}

/// Rewrite references into `{"@Kind": id, …}` and note what does not fit, before the mirror sees the body.
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

    /// `at` is the attribute namespace the fields land in (`qbo.Invoice.Line`).
    fn walk(&mut self, v: &mut Json, at: &str, root_kind: &str) {
        match v {
            Json::Array(items) => {
                for it in items {
                    self.walk(it, at, root_kind);
                }
            }
            Json::Object(obj) => {
                let fields: Vec<String> = obj.keys().cloned().collect();
                for f in fields {
                    let here = format!("{at}/{f}");
                    if let Some(Json::Object(r)) = obj.get_mut(&f)
                        && f.ends_with("Ref")
                        && r.contains_key("value")
                    {
                        self.reference(&f, r, &here, root_kind);
                        continue;
                    }
                    if f == "LinkedTxn"
                        && let Some(Json::Array(items)) = obj.get_mut(&f)
                    {
                        for it in items.iter_mut().filter_map(Json::as_object_mut) {
                            self.linked(it, &here);
                        }
                        continue;
                    }
                    if let Some(Json::Object(w)) = obj.get(&f)
                        && w.len() == 1
                        && w.contains_key("value")
                    {
                        self.once(
                            "value-wrapper",
                            here.clone(),
                            "a {\"value\": …} wrapper that is not a reference (a memo, a currency preference): kept as a component with one attribute",
                        );
                    }
                    if let Some(Json::String(t)) = obj.get(&f)
                        && t.len() >= 25
                        && t.as_bytes().get(10) == Some(&b'T')
                        && !t.ends_with('Z')
                        && Instant::parse_offset(t).is_ok()
                    {
                        self.once(
                            "offset-dropped",
                            here.clone(),
                            "stored in UTC; the company's local offset is not kept (it moves with daylight saving: -08:00 and -07:00 both occur), so the local date and time cannot be recovered",
                        );
                    }
                    if f.ends_with("Ex") && obj.get(&f).is_some_and(|x| x.get("any").is_some()) {
                        self.once(
                            "serialisation-artefact",
                            here.clone(),
                            "an XML binding leaked into the JSON (name, declaredType, scope, nil, …): mirrored as is, meaning only in value.Name/value.Value",
                        );
                    }
                    if let Some(child) = obj.get_mut(&f) {
                        self.walk(child, &format!("{at}.{f}"), root_kind);
                    }
                }
            }
            _ => {}
        }
    }

    fn reference(&mut self, field: &str, r: &mut Map<String, Json>, at: &str, root_kind: &str) {
        let Some(kind) = ref_kind(field, r, root_kind) else {
            self.once(
                "unresolved-ref",
                at.to_string(),
                format!("{field} names a kind this projector does not know; kept as a component"),
            );
            return;
        };
        if field == "EntityRef" {
            r.remove("type");
            self.once(
                "polymorphic-ref",
                at.to_string(),
                "the kind is in the ref's own type field (Vendor, Customer, Employee); the ref points at that kind and type is not stored",
            );
        } else {
            self.once(
                "ref-by-field-name",
                field.to_string(),
                format!("{{\"value\": id}} with the kind only in the field name; read as a ref to {kind}"),
            );
        }
        if let Some(id) = r.remove("value") {
            r.insert(format!("@{kind}"), id);
        }
    }

    fn linked(&mut self, it: &mut Map<String, Json>, at: &str) {
        let kind = it.get("TxnType").and_then(Json::as_str).map(str::to_string);
        let Some(kind) = kind.filter(|k| REF_IDS.iter().any(|(_, r)| r == k)) else {
            self.once(
                "unresolved-ref",
                at.to_string(),
                "LinkedTxn of a kind this projector does not know; kept as a component",
            );
            return;
        };
        self.once(
            "polymorphic-ref",
            at.to_string(),
            "LinkedTxn: the id in TxnId, the kind in the sibling TxnType; read as a ref to that kind",
        );
        if let Some(id) = it.remove("TxnId") {
            it.remove("TxnType");
            it.insert(format!("@{kind}"), id);
        }
    }
}

fn normalise(body: &mut Json) -> Result<Vec<Pressure>, IngestError> {
    if let Some(f) = body.get("Fault").or_else(|| body.get("fault")) {
        return Err(IngestError(format!(
            "qbo: the response is an error, not data: {f}"
        )));
    }
    let mut n = Normaliser {
        pressure: Vec::new(),
        seen: BTreeSet::new(),
    };
    if let Some(top) = body.as_object() {
        for f in top.keys().filter(|f| *f != "QueryResponse") {
            n.once(
                "envelope",
                format!("/{f}"),
                "outside the collection (the server's read time); not stored: tx/observed-at is the shell's",
            );
        }
    }
    let Some(qr) = body.get_mut("QueryResponse").and_then(Json::as_object_mut) else {
        return Ok(n.pressure);
    };
    let keys: Vec<String> = qr.keys().cloned().collect();
    for key in keys {
        let known = CONFIG.collections.iter().any(|(l, _, _)| *l == key);
        if !known {
            n.once(
                "envelope",
                format!("/QueryResponse/{key}"),
                "paging (startPosition, maxResults); not stored, so a file cannot say it was the whole list",
            );
            continue;
        }
        if let Some(items) = qr.get_mut(&key) {
            n.walk(items, &format!("{NS}.{key}"), &key);
        }
    }
    Ok(n.pressure)
}

impl Projector for Qbo {
    fn id(&self) -> &'static str {
        "qbo.json@1"
    }

    fn reads(&self) -> &'static str {
        "QuickBooks Online API query responses (CompanyInfo, Account, Customer, Item, Invoice, Payment, Purchase, JournalEntry, …), one entity type per file"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let mut body: Json = serde_json::from_slice(input.file.bytes)
            .map_err(|e| IngestError(format!("{}: not JSON ({e})", input.file.name)))?;
        let mut pressure = normalise(&mut body)?;
        let m = jsonmirror::mirror(&CONFIG, db, input, &body)?;
        pressure.extend(m.pressure);
        let mut it = Interp(Kit {
            db,
            input,
            snap: m.snap,
            pressure,
            rule: "qbo.json@1",
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

/// The (kind, id) a rewritten reference names.
fn rid<'j>(o: &'j Map<String, Json>, field: &str) -> Option<(&'j str, &'j str)> {
    o.get(field)?
        .as_object()?
        .iter()
        .find_map(|(key, v)| Some((key.strip_prefix('@')?, v.as_str()?)))
}

/// The display name a reference carries (a copy of the target's name as of this document).
fn ref_name<'j>(o: &'j Map<String, Json>, field: &str) -> Option<&'j str> {
    o.get(field)?
        .get("name")
        .and_then(Json::as_str)
        .filter(|n| !n.is_empty())
}

fn date(o: &Map<String, Json>) -> Option<Value> {
    s(o, "TxnDate").and_then(|t| when("TxnDate", t))
}

fn lines(o: &Map<String, Json>) -> impl Iterator<Item = (usize, &Map<String, Json>)> {
    o.get("Line")
        .and_then(Json::as_array)
        .into_iter()
        .flatten()
        .enumerate()
        .filter_map(|(i, l)| Some((i, l.as_object()?)))
}

/// A line's detail object: QuickBooks names it after the line's `DetailType`.
fn detail(l: &Map<String, Json>) -> (&str, Option<&Map<String, Json>>) {
    let ty = s(l, "DetailType").unwrap_or("");
    (ty, l.get(ty).and_then(Json::as_object))
}

struct Interp<'a, 'i>(Kit<'a, 'i>);

impl Interp<'_, '_> {
    fn root(&mut self, r: &Root<'_>) {
        let key = r.key.clone();
        self.0.document(&key, &format!("qbo-{}", r.kind), &r.id);
        let o = r.json;
        match r.kind {
            "CompanyInfo" => self.company(o),
            "Account" => self.account(r),
            "Customer" | "Vendor" | "Employee" => {
                self.0.add(&key, "party/id", Value::str(&r.doc_id));
                if let Some(n) = s(o, "DisplayName") {
                    self.0.add(&key, "party/name", Value::str(n));
                }
            }
            "Invoice" => self.invoice(r),
            "Payment" => self.payment(r),
            "Purchase" => self.purchase(r),
            "JournalEntry" => self.journal(r),
            _ => {}
        }
    }

    fn currency(&mut self, o: &Map<String, Json>) -> Option<String> {
        rid(o, "CurrencyRef").map(|(_, c)| c.to_string())
    }

    fn commodity(&mut self, key: &str, attr: &'static str, o: &Map<String, Json>) {
        if let Some(c) = self.currency(o) {
            let ck = self.0.commodity(&c);
            self.0.add(key, attr, ValIn::Ref(crate::temp(&ck)));
        }
    }

    fn company(&mut self, o: &Map<String, Json>) {
        if let Some(n) = s(o, "CompanyName") {
            self.0.describe_book(n);
        }
        self.0.press(
            "missing-context",
            "qbo.CompanyInfo",
            "the home currency is in Preferences, not CompanyInfo: book/commodity is not set from this file",
        );
    }

    fn account(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.as_str());
        self.0.add(key, "account/id", Value::str(&r.doc_id));
        if let Some(n) = s(o, "Name") {
            self.0.add(key, "account/name", Value::str(n));
        }
        if let Some(c) = s(o, "AcctNum") {
            self.0.add(key, "account/code", Value::str(c));
        }
        if let Some(t) = s(o, "AccountType").and_then(|t| kw("account.type", t)) {
            self.0.add(key, "account/type", t);
        }
        let book = self.0.book();
        self.0
            .add(key, "account/book", ValIn::Ref(crate::temp(&book)));
        self.commodity(key, "account/commodity", o);
        if o.contains_key("CurrentBalance") {
            self.0.press(
                "derived-value",
                "qbo.Account/CurrentBalance",
                "a running balance QuickBooks computes from its entries: mirrored, so every posting elsewhere shows up here as an edit",
            );
        }
    }

    /// The receivable account: the invoice may name it; usually it does not, and QuickBooks uses the one
    /// account of type Accounts Receivable.
    fn by_type(&mut self, ty: &str, at: &str) -> Option<String> {
        let db = self.0.db;
        let found: Vec<String> = (|| {
            let a = db.attr(&k("qbo.Account/AccountType"))?.id;
            let src = db.lookup(&k("source/id"), &Value::str(self.0.src()))?;
            let ds = db.attr(&k("doc/source"))?.id;
            Some(
                db.with_value(a, &Value::str(ty))
                    .filter(|e| db.values(*e, ds).any(|v| v == &Value::Ref(src)))
                    .filter_map(|e| db.get(e, &k("doc/id")).and_then(Value::as_str))
                    .map(str::to_string)
                    .collect(),
            )
        })()
        .unwrap_or_default();
        match found.as_slice() {
            [one] => Some(self.0.refer_doc(one)),
            [] => {
                self.0.press(
                    "unresolved-ref",
                    at,
                    format!("no {ty} account known yet (ingest Account first)"),
                );
                None
            }
            _ => {
                self.0.press(
                    "unresolved-ref",
                    at,
                    format!(
                        "{} accounts of type {ty}; the document does not say which",
                        found.len()
                    ),
                );
                None
            }
        }
    }

    fn account_ref(&mut self, o: &Map<String, Json>, field: &str) -> Option<String> {
        rid(o, field).map(|(kind, id)| self.0.refer(kind, id))
    }

    /// The income account a sales line posts to: on the line (`ItemAccountRef`, newer API versions), else
    /// on the item, which this source must have described already.
    fn income_account(&mut self, d: &Map<String, Json>, at: &str) -> Option<String> {
        if let Some(a) = self.account_ref(d, "ItemAccountRef") {
            return Some(a);
        }
        let (_, item) = rid(d, "ItemRef")?;
        let did = doc_id(self.0.src(), "Item", item);
        let e: Option<Eid> = self.0.db.lookup(&k("doc/id"), &Value::str(&did));
        match e.and_then(|e| self.0.target_doc(e, &k("qbo.Item/IncomeAccountRef"))) {
            Some(acct) => Some(self.0.refer_doc(&acct)),
            None => {
                self.0.press(
                    "unresolved-ref",
                    at,
                    format!("item {item}'s income account is not known yet (ingest Item first)"),
                );
                None
            }
        }
    }

    fn invoice(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        self.0.observe(&key, &r.doc_id, "invoice");
        let total = dec(o, "TotalAmt");
        if let Some(t) = total {
            self.0.add(&key, "obs/amount", Value::Dec(t));
        }
        let date = date(o);
        if let Some(d) = &date {
            self.0.add(&key, "obs/date", d.clone());
        }
        self.commodity(&key, "obs/commodity", o);
        if let Some(n) = s(o, "DocNumber") {
            self.0.add(&key, "obs/reference", Value::str(n));
        }
        if let Some(n) = ref_name(o, "CustomerRef") {
            self.0.add(&key, "obs/counterparty", Value::str(n));
        }
        // QuickBooks invoices have no status: open, partly paid and paid are what Balance says against TotalAmt.
        if let (Some(t), Some(b)) = (total, dec(o, "Balance")) {
            let st = if b.is_zero() {
                "paid"
            } else if b == t {
                "open"
            } else {
                "partly-paid"
            };
            if let Some(v) = kw("obs.status", st) {
                self.0.add(&key, "obs/status", v);
            }
            self.0.press(
                "derived-value",
                "qbo.Invoice/Balance",
                "no status field: obs/status is derived from Balance and TotalAmt, both computed by QuickBooks",
            );
        }
        let ar = match self.account_ref(o, "ARAccountRef") {
            Some(a) => Some(a),
            None => self.by_type("Accounts Receivable", "qbo.Invoice/ARAccountRef"),
        };
        let mut legs: Vec<Leg> = vec![(ar, total.unwrap_or_default(), String::new())];
        for (i, l) in lines(o) {
            let lk = format!("{key}/Line/@{i}");
            let (ty, d) = detail(l);
            let amount = dec(l, "Amount").unwrap_or_default();
            let description = s(l, "Description").unwrap_or("").to_string();
            match (ty, d) {
                ("SubTotalLineDetail", _) => {
                    self.0.press(
                        "derived-value",
                        "qbo.Invoice.Line/SubTotalLineDetail",
                        "QuickBooks appends a subtotal line (the sum of the lines above it): mirrored, not observed or posted",
                    );
                    continue;
                }
                ("SalesItemLineDetail", Some(d)) => {
                    let acct = self.income_account(d, "qbo.Invoice.Line.SalesItemLineDetail");
                    legs.push((acct, -amount, description.clone()));
                }
                ("DiscountLineDetail", Some(d)) => {
                    let acct = self.account_ref(d, "DiscountAccountRef");
                    legs.push((acct, amount, description.clone()));
                }
                ("DescriptionOnly", _) => {}
                _ => self.0.press(
                    "polymorphic-line",
                    &format!("qbo.Invoice.Line/{ty}"),
                    "a line kind this projector does not interpret: mirrored only",
                ),
            }
            if !self.0.snap.has(&lk) {
                continue;
            }
            self.0
                .line(&lk, &format!("{}/Line/@{i}", r.doc_id), &key, i);
            let signed = if ty == "DiscountLineDetail" {
                -amount
            } else {
                amount
            };
            if ty != "DescriptionOnly" {
                self.0.add(&lk, "obs/amount", Value::Dec(signed));
            }
            if !description.is_empty() {
                self.0.add(&lk, "obs/description", Value::str(&description));
            }
        }
        if o.get("Line").is_some() {
            self.0.press(
                "polymorphic-line",
                "qbo.Invoice.Line/DetailType",
                "each line names its kind in DetailType and keeps its fields in an object of that name: one attribute namespace per kind",
            );
        }
        if let Some(t) = o
            .get("TxnTaxDetail")
            .and_then(Json::as_object)
            .and_then(|t| dec(t, "TotalTax"))
            .filter(|t| !t.is_zero())
        {
            self.0.press(
                "unresolved-ref",
                "qbo.Invoice/TxnTaxDetail",
                format!("tax of {t}: the tax liability account is behind TaxCode → TaxRate → AgencyRef, not read here"),
            );
        }
        match date {
            Some(d) => {
                let cur = self.currency(o);
                let who = ref_name(o, "CustomerRef").unwrap_or("").to_string();
                self.0.entry(&key, &r.doc_id, d, &who, cur.as_deref(), legs);
            }
            None => self.0.no_entry(&r.doc_id),
        }
    }

    fn payment(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        self.0.observe(&key, &r.doc_id, "payment");
        let total = dec(o, "TotalAmt");
        if let Some(t) = total {
            self.0.add(&key, "obs/amount", Value::Dec(t));
        }
        let date = date(o);
        if let Some(d) = &date {
            self.0.add(&key, "obs/date", d.clone());
        }
        self.commodity(&key, "obs/commodity", o);
        let deposit = self.account_ref(o, "DepositToAccountRef");
        match &deposit {
            Some(a) => self
                .0
                .add(&key, "obs/account", ValIn::Ref(crate::temp(a))),
            None => self.0.press(
                "unresolved-ref",
                "qbo.Payment/DepositToAccountRef",
                "no deposit account: QuickBooks then uses Undeposited Funds, which the document does not name",
            ),
        }
        let who = ref_name(o, "CustomerRef").unwrap_or("").to_string();
        if !who.is_empty() {
            self.0.add(&key, "obs/counterparty", Value::str(&who));
        }
        if let Some(n) = s(o, "PaymentRefNum") {
            self.0.add(&key, "obs/reference", Value::str(n));
        }
        for (_, l) in lines(o) {
            let amount = dec(l, "Amount");
            for t in l
                .get("LinkedTxn")
                .and_then(Json::as_array)
                .into_iter()
                .flatten()
            {
                let Some(t) = t.as_object() else { continue };
                let Some((kind, id)) = t
                    .iter()
                    .find_map(|(f, v)| Some((f.strip_prefix('@')?, v.as_str()?)))
                else {
                    continue;
                };
                let tk = self.0.refer(kind, id);
                let tid = doc_id(self.0.src(), kind, id);
                let lk = self.0.link("settles", (&key, &r.doc_id), (&tk, &tid));
                if let Some(a) = amount {
                    self.0.add(&lk, "link/amount", Value::Dec(a));
                }
            }
        }
        if dec(o, "UnappliedAmt").is_some_and(|u| !u.is_zero()) {
            self.0.press(
                "derived-value",
                "qbo.Payment/UnappliedAmt",
                "part of the payment is not applied to an invoice; it stays a credit on the receivable",
            );
        }
        let ar = self.by_type("Accounts Receivable", "qbo.Payment");
        match (date, total) {
            (Some(d), Some(t)) => {
                let legs = vec![(deposit, t, String::new()), (ar, -t, String::new())];
                let cur = self.currency(o);
                self.0.entry(&key, &r.doc_id, d, &who, cur.as_deref(), legs);
            }
            _ => self.0.no_entry(&r.doc_id),
        }
    }

    fn purchase(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        self.0.observe(&key, &r.doc_id, "money-movement");
        // money out of the payment account, unless it is a refund (`Credit`)
        let sign = if o.get("Credit").and_then(Json::as_bool) == Some(true) {
            Decimal::ONE
        } else {
            Decimal::NEGATIVE_ONE
        };
        let total = dec(o, "TotalAmt");
        if let Some(t) = total {
            self.0.add(&key, "obs/amount", Value::Dec(sign * t));
        }
        let date = date(o);
        if let Some(d) = &date {
            self.0.add(&key, "obs/date", d.clone());
        }
        self.commodity(&key, "obs/commodity", o);
        let paid_from = self.account_ref(o, "AccountRef");
        if let Some(a) = &paid_from {
            self.0.add(&key, "obs/account", ValIn::Ref(crate::temp(a)));
        }
        let who = ref_name(o, "EntityRef").unwrap_or("").to_string();
        if !who.is_empty() {
            self.0.add(&key, "obs/counterparty", Value::str(&who));
        }
        if let Some(n) = s(o, "DocNumber") {
            self.0.add(&key, "obs/reference", Value::str(n));
        }
        if let Some(n) = s(o, "PrivateNote") {
            self.0.add(&key, "obs/description", Value::str(n));
        }
        let mut legs: Vec<Leg> = vec![(paid_from, sign * total.unwrap_or_default(), String::new())];
        for (i, l) in lines(o) {
            let lk = format!("{key}/Line/@{i}");
            let (ty, d) = detail(l);
            let amount = dec(l, "Amount").unwrap_or_default();
            let description = s(l, "Description").unwrap_or("").to_string();
            let acct = match (ty, d) {
                ("AccountBasedExpenseLineDetail", Some(d)) => self.account_ref(d, "AccountRef"),
                ("ItemBasedExpenseLineDetail", Some(d)) => {
                    let item = rid(d, "ItemRef").map(|(_, id)| id.to_string());
                    let e = item.and_then(|id| {
                        self.0.db.lookup(
                            &k("doc/id"),
                            &Value::str(&doc_id(self.0.src(), "Item", &id)),
                        )
                    });
                    e.and_then(|e| self.0.target_doc(e, &k("qbo.Item/ExpenseAccountRef")))
                        .map(|a| self.0.refer_doc(&a))
                }
                _ => {
                    self.0.press(
                        "polymorphic-line",
                        &format!("qbo.Purchase.Line/{ty}"),
                        "a line kind this projector does not interpret: mirrored only",
                    );
                    None
                }
            };
            legs.push((acct, -sign * amount, description.clone()));
            if !self.0.snap.has(&lk) {
                continue;
            }
            self.0
                .line(&lk, &format!("{}/Line/@{i}", r.doc_id), &key, i);
            self.0.add(&lk, "obs/amount", Value::Dec(sign * amount));
            if !description.is_empty() {
                self.0.add(&lk, "obs/description", Value::str(&description));
            }
        }
        match date {
            Some(d) => {
                let cur = self.currency(o);
                self.0.entry(&key, &r.doc_id, d, &who, cur.as_deref(), legs);
            }
            None => self.0.no_entry(&r.doc_id),
        }
    }

    fn journal(&mut self, r: &Root<'_>) {
        let o = r.json;
        let Some(date) = date(o) else {
            self.0.no_entry(&r.doc_id);
            return;
        };
        let mut legs: Vec<Leg> = Vec::new();
        for (_, l) in lines(o) {
            let (ty, d) = detail(l);
            let Some(d) = d.filter(|_| ty == "JournalEntryLineDetail") else {
                continue;
            };
            let amount = dec(l, "Amount").unwrap_or_default();
            // amounts are positive; the side is an enum beside them
            let q = match s(d, "PostingType") {
                Some("Credit") => -amount,
                _ => amount,
            };
            let acct = self.account_ref(d, "AccountRef");
            legs.push((acct, q, s(l, "Description").unwrap_or("").to_string()));
        }
        self.0.press(
            "sign",
            "qbo.JournalEntry.Line.JournalEntryLineDetail/PostingType",
            "amounts are unsigned with Debit or Credit beside them; postings are signed, debit positive",
        );
        let narration = s(o, "PrivateNote").unwrap_or("").to_string();
        let cur = self.currency(o);
        self.0
            .entry(&r.key, &r.doc_id, date, &narration, cur.as_deref(), legs);
    }
}
