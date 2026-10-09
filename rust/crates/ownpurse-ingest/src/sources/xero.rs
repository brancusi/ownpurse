//! Xero: API response bodies (one collection per file, as the connector fetched them).
//!
//! Two passes over the same snapshot. The mirror keeps every field in Xero's own words
//! (`xero.BankTransaction/Total`). The interpretation says what each document means in the shared vocabulary:
//! accounts, parties, observations (`obs/*`), and, because Xero owns these books, the mirror book's entries.

use crate::facts::fingerprint;
use crate::jsonmirror::{self, JsonSource, Root, doc_id};
use crate::snapshot::Snapshot;
use crate::{IngestError, Input, Pressure, Projection, Projector, temp};
use ownpurse_facts::{Date, Db, Eid, Instant, Kw, ValIn, Value};
use rust_decimal::Decimal;
use serde_json::{Map, Value as Json};

pub struct Xero;

pub static CONFIG: JsonSource = JsonSource {
    ns: "xero",
    root: &[],
    collections: &[
        ("Organisations", "Organisation", "OrganisationID"),
        ("Accounts", "Account", "AccountID"),
        ("TaxRates", "TaxRate", "TaxType"),
        (
            "TrackingCategories",
            "TrackingCategory",
            "TrackingCategoryID",
        ),
        ("Currencies", "Currency", "Code"),
        ("Contacts", "Contact", "ContactID"),
        ("Invoices", "Invoice", "InvoiceID"),
        ("CreditNotes", "CreditNote", "CreditNoteID"),
        ("Payments", "Payment", "PaymentID"),
        ("BankTransactions", "BankTransaction", "BankTransactionID"),
        ("BankTransfers", "BankTransfer", "BankTransferID"),
        ("ManualJournals", "ManualJournal", "ManualJournalID"),
        ("Overpayments", "Overpayment", "OverpaymentID"),
        ("Prepayments", "Prepayment", "PrepaymentID"),
    ],
    refs: &[
        ("Contact", "Contact", "ContactID"),
        ("BankAccount", "Account", "AccountID"),
        ("Account", "Account", "AccountID"),
        ("FromBankAccount", "Account", "AccountID"),
        ("ToBankAccount", "Account", "AccountID"),
        ("Invoice", "Invoice", "InvoiceID"),
        ("CreditNote", "CreditNote", "CreditNoteID"),
        ("Payments", "Payment", "PaymentID"),
        ("CreditNotes", "CreditNote", "CreditNoteID"),
        ("Overpayments", "Overpayment", "OverpaymentID"),
        ("Prepayments", "Prepayment", "PrepaymentID"),
        ("Tracking", "TrackingOption", "TrackingOptionID"),
    ],
    owned: &[("Options", "TrackingOption", "TrackingOptionID")],
    child_ids: &[
        "LineItemID",
        "JournalLineID",
        "AllocationID",
        "ContactPersonID",
        "AddressType",
        "PhoneType",
    ],
    when,
};

/// Xero's dates: `/Date(1773273600000+0000)/` (milliseconds, UTC) and `2026-03-12T00:00:00` (no zone).
/// A business date (`Date`, `DueDate`, …) arrives as a midnight instant; it is stored as a date.
fn when(field: &str, text: &str) -> Option<Value> {
    let business =
        field.ends_with("Date") || field == "DateString" || field.ends_with("DateString");
    if let Some(inner) = text
        .strip_prefix("/Date(")
        .and_then(|r| r.strip_suffix(")/"))
    {
        let ms_text: String = inner
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-')
            .collect();
        let ms: i64 = ms_text.parse().ok()?;
        let instant = Instant::from_unix_millis(ms);
        if business && ms.rem_euclid(86_400_000) == 0 {
            return Some(Value::Date(instant.date()));
        }
        return Some(Value::Instant(instant));
    }
    if business {
        if let Some(d) = text.strip_suffix("T00:00:00") {
            return Date::parse(d).ok().map(Value::Date);
        }
        if let Ok(d) = Date::parse(text) {
            return Some(Value::Date(d));
        }
    }
    None
}

fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

/// Attributes the Xero projector owns on the entities it describes: everything in its own namespace and the
/// interpretation it writes. `doc/file` and `doc/source` accumulate; a person's notes and links are never
/// touched by a re-read.
fn owns(a: &Kw) -> bool {
    let ns = a.ns();
    ns == "xero"
        || ns.starts_with("xero.")
        || matches!(
            ns,
            "obs" | "account" | "party" | "entry" | "posting" | "book" | "link"
        )
        || matches!(a.as_str(), "doc/kind" | "doc/external-id")
}

impl Projector for Xero {
    fn id(&self) -> &'static str {
        "xero.json@1"
    }

    fn reads(&self) -> &'static str {
        "Xero API response bodies (Accounts, Contacts, Invoices, Payments, BankTransactions, …), one per file"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let body: Json = serde_json::from_slice(input.file.bytes)
            .map_err(|e| IngestError(format!("{}: not JSON ({e})", input.file.name)))?;
        let m = jsonmirror::mirror(&CONFIG, db, input, &body)?;
        let mut it = Interp {
            db,
            input,
            snap: m.snap,
            pressure: m.pressure,
        };
        for root in &m.roots {
            it.root(root);
        }
        let tx = it.snap.diff(db, &owns);
        Ok(Projection {
            install: m.install,
            tx,
            pressure: it.pressure,
        })
    }
}

struct Interp<'a, 'i> {
    db: &'a Db,
    input: &'a Input<'i>,
    snap: Snapshot,
    pressure: Vec<Pressure>,
}

fn s<'j>(o: &'j Map<String, Json>, f: &str) -> Option<&'j str> {
    o.get(f).and_then(Json::as_str).filter(|t| !t.is_empty())
}

fn dec(o: &Map<String, Json>, f: &str) -> Option<Decimal> {
    match o.get(f)? {
        Json::Number(n) => ownpurse_facts::value::parse_decimal(&n.to_string()).ok(),
        Json::String(t) => ownpurse_facts::value::parse_decimal(t).ok(),
        _ => None,
    }
}

fn kw(ns: &str, name: &str) -> Option<Value> {
    let n: String = name.to_ascii_lowercase().replace('_', "-");
    Kw::new(&format!("{ns}/{n}")).ok().map(Value::Kw)
}

impl Interp<'_, '_> {
    fn src(&self) -> &str {
        &self.input.source.id
    }

    fn press(&mut self, kind: &str, at: &str, detail: impl Into<String>) {
        self.pressure.push(Pressure::new(kind, at, detail));
    }

    fn add(&mut self, key: &str, a: &'static str, v: impl Into<ValIn>) {
        self.snap.add(key, &k(a), v);
    }

    fn date(&self, o: &Map<String, Json>) -> Option<Value> {
        s(o, "DateString")
            .and_then(|t| when("DateString", t))
            .or_else(|| s(o, "Date").and_then(|t| when("Date", t)))
    }

    /// A referenced entity's snapshot key, declared so refs to it resolve.
    fn refer(&mut self, kind: &str, id: &str) -> String {
        let did = doc_id(self.src(), kind, id);
        let key = format!("doc:{did}");
        self.snap.refer(&key, &k("doc/id"), Value::str(&did));
        key
    }

    fn nested_id<'j>(o: &'j Map<String, Json>, field: &str, id: &str) -> Option<&'j str> {
        o.get(field).and_then(|v| v.get(id)).and_then(Json::as_str)
    }

    fn commodity(&mut self, code: &str) -> String {
        let key = format!("commodity:{code}");
        self.snap
            .refer(&key, &k("commodity/code"), Value::str(code));
        key
    }

    fn source_key(&mut self) -> String {
        let key = format!("source:{}", self.src());
        self.snap
            .refer(&key, &k("source/id"), Value::str(self.src()));
        key
    }

    fn file_key(&mut self) -> String {
        let h = self.input.file.hash;
        let key = format!("file:{}", h.hex());
        self.snap.refer(&key, &k("file/sha256"), Value::Hash(h));
        key
    }

    fn book(&mut self) -> String {
        let key = format!("book:{}", self.src());
        self.snap.refer(&key, &k("book/id"), Value::str(self.src()));
        key
    }

    fn root(&mut self, r: &Root<'_>) {
        let key = r.key.clone();
        self.add(&key, "doc/external-id", Value::str(&r.id));
        if let Some(kind) = kw("doc.kind", &format!("xero-{}", r.kind)) {
            self.add(&key, "doc/kind", kind);
        }
        let (src, file) = (self.source_key(), self.file_key());
        self.add(&key, "doc/source", ValIn::Ref(temp(&src)));
        self.add(&key, "doc/file", ValIn::Ref(temp(&file)));
        let o = r.json;
        match r.kind {
            "Organisation" => self.organisation(o),
            "Account" => self.account(&key, o),
            "Contact" => {
                self.add(&key, "party/id", Value::str(&r.doc_id));
                if let Some(n) = s(o, "Name") {
                    self.add(&key, "party/name", Value::str(n));
                }
            }
            "BankTransaction" => self.bank_transaction(r),
            "Invoice" | "CreditNote" => self.invoice(r),
            "Payment" => self.payment(r),
            "BankTransfer" => self.transfer(r),
            "ManualJournal" => self.manual_journal(r),
            _ => {}
        }
    }

    fn organisation(&mut self, o: &Map<String, Json>) {
        let book = self.book();
        if let Some(n) = s(o, "Name") {
            self.add(&book, "book/name", Value::str(n));
        }
        let src = self.source_key();
        self.add(&book, "book/authority", ValIn::Ref(temp(&src)));
        if let Some(c) = s(o, "BaseCurrency") {
            let c = self.commodity(c);
            self.add(&book, "book/commodity", ValIn::Ref(temp(&c)));
        }
    }

    fn account(&mut self, key: &str, o: &Map<String, Json>) {
        let did = key.trim_start_matches("doc:").to_string();
        self.add(key, "account/id", Value::str(&did));
        if let Some(n) = s(o, "Name") {
            self.add(key, "account/name", Value::str(n));
        }
        if let Some(c) = s(o, "Code") {
            self.add(key, "account/code", Value::str(c));
        }
        if let Some(t) = s(o, "Type").and_then(|t| kw("account.type", t)) {
            self.add(key, "account/type", t);
        }
        let book = self.book();
        self.add(key, "account/book", ValIn::Ref(temp(&book)));
        if let Some(num) = s(o, "BankAccountNumber") {
            let digits: String = num.chars().filter(char::is_ascii_digit).collect();
            if digits.len() >= 4 {
                self.add(key, "account/mask", Value::str(&digits[digits.len() - 4..]));
            }
        }
        if let Some(c) = s(o, "CurrencyCode") {
            let c = self.commodity(c);
            self.add(key, "account/commodity", ValIn::Ref(temp(&c)));
        }
    }

    /// The account a line names by code. Xero lines carry `AccountCode`, not `AccountID`, and a code is unique
    /// only within one org, so this looks among the accounts this source already described.
    fn account_by_code(&mut self, code: &str, at: &str) -> Option<String> {
        let ca = self.db.attr(&k("xero.Account/Code"))?.id;
        let src = self.db.lookup(&k("source/id"), &Value::str(self.src()));
        let ds = self.db.attr(&k("doc/source"))?.id;
        let found: Option<Eid> = self
            .db
            .with_value(ca, &Value::str(code))
            .find(|e| src.is_some_and(|s| self.db.values(*e, ds).any(|v| v == &Value::Ref(s))));
        match found.and_then(|e| {
            self.db
                .get(e, &k("doc/id"))
                .and_then(Value::as_str)
                .map(str::to_string)
        }) {
            Some(did) => {
                let key = format!("doc:{did}");
                self.snap.refer(&key, &k("doc/id"), Value::str(&did));
                Some(key)
            }
            None => {
                self.press(
                    "unresolved-ref",
                    at,
                    format!("account code {code} is not known yet (ingest Accounts first)"),
                );
                None
            }
        }
    }

    /// The sales-tax account: Xero marks it with `SystemAccount = GST`.
    fn tax_account(&mut self) -> Option<String> {
        let sa = self.db.attr(&k("xero.Account/SystemAccount"))?.id;
        let e = self.db.with_value(sa, &Value::str("GST")).next()?;
        let did = self.db.get(e, &k("doc/id"))?.as_str()?.to_string();
        let key = format!("doc:{did}");
        self.snap.refer(&key, &k("doc/id"), Value::str(&did));
        Some(key)
    }

    fn observe(&mut self, key: &str, id: &str, kind: &str) {
        self.add(key, "obs/id", Value::str(id));
        if let Some(kw) = kw("obs.kind", kind) {
            self.add(key, "obs/kind", kw);
        }
    }

    fn mask(&self, account_key: &str) -> String {
        let did = account_key.trim_start_matches("doc:");
        self.db
            .lookup(&k("doc/id"), &Value::str(did))
            .and_then(|e| self.db.get(e, &k("account/mask")).cloned())
            .map_or_else(|| did.to_string(), |v| v.text())
    }

    fn bank_transaction(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        let ty = s(o, "Type").unwrap_or("");
        let transfer_leg = ty.ends_with("-TRANSFER");
        self.observe(
            &key,
            &r.doc_id,
            if transfer_leg {
                "transfer-leg"
            } else {
                "money-movement"
            },
        );
        if transfer_leg {
            self.press(
                "duplicate-witness",
                "xero.BankTransaction/Type",
                "Xero writes a bank transaction for each side of a bank transfer, beside the BankTransfer itself",
            );
        }
        let sign = if ty.starts_with("SPEND") {
            Decimal::NEGATIVE_ONE
        } else {
            Decimal::ONE
        };
        let total = dec(o, "Total");
        if let Some(t) = total {
            self.add(&key, "obs/amount", Value::Dec(sign * t));
        }
        let date = self.date(o);
        if let Some(d) = date.clone() {
            self.add(&key, "obs/date", d);
        }
        if let Some(c) = s(o, "CurrencyCode") {
            let c = self.commodity(c);
            self.add(&key, "obs/commodity", ValIn::Ref(temp(&c)));
        }
        let bank =
            Self::nested_id(o, "BankAccount", "AccountID").map(|id| self.refer("Account", id));
        if let Some(b) = &bank {
            self.add(&key, "obs/account", ValIn::Ref(temp(b)));
        }
        let who = o
            .get("Contact")
            .and_then(|c| c.get("Name"))
            .and_then(Json::as_str)
            .unwrap_or("");
        if !who.is_empty() {
            self.add(&key, "obs/counterparty", Value::str(who));
        }
        if let Some(rf) = s(o, "Reference") {
            self.add(&key, "obs/reference", Value::str(rf));
        }
        if let Some(st) = s(o, "Status").and_then(|t| kw("obs.status", t)) {
            self.add(&key, "obs/status", st);
        }
        if let (Some(b), Some(t), Some(d)) = (&bank, total, &date) {
            let fp = fingerprint(&self.mask(b), &d.text(), sign * t, who);
            self.add(&key, "obs/fingerprint", Value::str(&fp));
        }
        let lines = self.lines(r, sign);
        let posted = s(o, "Status") == Some("AUTHORISED");
        if transfer_leg || !posted {
            self.snap
                .gone(&k("entry/id"), Value::str(&format!("{}#entry", r.doc_id)));
            return;
        }
        let (Some(b), Some(t), Some(d)) = (bank, total, date) else {
            return;
        };
        let mut legs = vec![(Some(b), sign * t, String::new())];
        let tax: Decimal = lines.iter().map(|l| l.tax).sum();
        for l in lines {
            let acct = l.code.as_deref().and_then(|c| {
                self.account_by_code(c, "xero.BankTransaction.LineItems/AccountCode")
            });
            legs.push((acct, -sign * l.net, l.description));
        }
        if !tax.is_zero() {
            let ta = self.tax_account();
            legs.push((ta, -sign * tax, "tax".into()));
        }
        self.entry(r, d, who, legs);
    }

    fn lines(&mut self, r: &Root<'_>, sign: Decimal) -> Vec<Line> {
        let o = r.json;
        let inclusive = s(o, "LineAmountTypes") == Some("Inclusive");
        let mut out = Vec::new();
        for (i, li) in o
            .get("LineItems")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
            .enumerate()
        {
            let Some(li) = li.as_object() else { continue };
            let seg = s(li, "LineItemID").map_or_else(|| i.to_string(), str::to_string);
            let lk = format!("{}/LineItems/{seg}", r.key);
            if !self.snap.has(&lk) {
                continue;
            }
            self.observe(&lk, &format!("{}/LineItems/{seg}", r.doc_id), "line");
            self.add(&lk, "obs/parent", ValIn::Ref(temp(&r.key)));
            self.add(&lk, "obs/index", Value::Int(i64::try_from(i).unwrap_or(0)));
            let amount = dec(li, "LineAmount").unwrap_or_default();
            let tax = dec(li, "TaxAmount").unwrap_or_default();
            let net = if inclusive { amount - tax } else { amount };
            self.add(&lk, "obs/amount", Value::Dec(sign * amount));
            let description = s(li, "Description").unwrap_or("").to_string();
            if !description.is_empty() {
                self.add(&lk, "obs/description", Value::str(&description));
            }
            out.push(Line {
                code: s(li, "AccountCode").map(str::to_string),
                net,
                tax,
                description,
            });
        }
        out
    }

    /// The mirror book's entry for a document: held to Xero's authority, so it is posted as Xero says.
    fn entry(
        &mut self,
        r: &Root<'_>,
        date: Value,
        narration: &str,
        legs: Vec<(Option<String>, Decimal, String)>,
    ) {
        let eid = format!("{}#entry", r.doc_id);
        let ek = format!("entry:{eid}");
        self.snap.ent(&ek, &k("entry/id"), Value::str(&eid));
        let book = self.book();
        self.add(&ek, "entry/book", ValIn::Ref(temp(&book)));
        self.add(&ek, "entry/date", date);
        self.add(&ek, "entry/status", Value::Kw(k("entry.status/posted")));
        if !narration.is_empty() {
            self.add(&ek, "entry/narration", Value::str(narration));
        }
        let ccy = s(r.json, "CurrencyCode").map(|c| self.commodity(c));
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
                Value::Int(i64::try_from(i).unwrap_or(0)),
            );
            sum += q;
            if let Some(a) = acct {
                self.add(&pk, "posting/account", ValIn::Ref(temp(&a)));
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
                &r.doc_id,
                format!("the mirror entry is off by {sum}"),
            );
        }
        self.link("mirrors", &ek, &eid, &r.key, &r.doc_id);
    }

    fn link(
        &mut self,
        kind: &str,
        from_key: &str,
        from_id: &str,
        to_key: &str,
        to_id: &str,
    ) -> String {
        let id = format!("{kind}|{from_id}|{to_id}");
        let lk = format!("link:{id}");
        self.snap.ent(&lk, &k("link/id"), Value::str(&id));
        if let Some(kv) = kw("link.kind", kind) {
            self.add(&lk, "link/kind", kv);
        }
        self.add(&lk, "link/from", ValIn::Ref(temp(from_key)));
        self.add(&lk, "link/to", ValIn::Ref(temp(to_key)));
        self.add(&lk, "link/rule", Value::str("xero.json@1"));
        lk
    }

    fn invoice(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        let ty = s(o, "Type").unwrap_or("");
        let (kind, sign) = match ty {
            "ACCREC" => ("invoice", Decimal::ONE),
            "ACCPAY" => ("bill", Decimal::NEGATIVE_ONE),
            "ACCRECCREDIT" => ("credit-note", Decimal::NEGATIVE_ONE),
            "ACCPAYCREDIT" => ("credit-note", Decimal::ONE),
            _ => ("invoice", Decimal::ONE),
        };
        self.observe(&key, &r.doc_id, kind);
        if let Some(t) = dec(o, "Total") {
            self.add(&key, "obs/amount", Value::Dec(sign * t));
        }
        if let Some(d) = self.date(o) {
            self.add(&key, "obs/date", d);
        }
        if let Some(c) = s(o, "CurrencyCode") {
            let c = self.commodity(c);
            self.add(&key, "obs/commodity", ValIn::Ref(temp(&c)));
        }
        if let Some(n) = s(o, "InvoiceNumber").or_else(|| s(o, "CreditNoteNumber")) {
            self.add(&key, "obs/reference", Value::str(n));
        }
        if let Some(n) = o
            .get("Contact")
            .and_then(|c| c.get("Name"))
            .and_then(Json::as_str)
        {
            self.add(&key, "obs/counterparty", Value::str(n));
        }
        if let Some(st) = s(o, "Status").and_then(|t| kw("obs.status", t)) {
            self.add(&key, "obs/status", st);
        }
        self.lines(r, sign);
    }

    fn payment(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        self.observe(&key, &r.doc_id, "payment");
        let sign = match s(o, "PaymentType") {
            Some(t) if t.starts_with("ACCPAY") || t == "ARCREDITPAYMENT" => Decimal::NEGATIVE_ONE,
            _ => Decimal::ONE,
        };
        let amount = dec(o, "Amount");
        if let Some(a) = amount {
            self.add(&key, "obs/amount", Value::Dec(sign * a));
        }
        if let Some(d) = self.date(o) {
            self.add(&key, "obs/date", d);
        }
        if let Some(a) = Self::nested_id(o, "Account", "AccountID") {
            let ak = self.refer("Account", a);
            self.add(&key, "obs/account", ValIn::Ref(temp(&ak)));
        }
        if let Some(rf) = s(o, "Reference") {
            self.add(&key, "obs/reference", Value::str(rf));
        }
        if let Some(st) = s(o, "Status").and_then(|t| kw("obs.status", t)) {
            self.add(&key, "obs/status", st);
        }
        if let Some(inv) = Self::nested_id(o, "Invoice", "InvoiceID") {
            let ik = self.refer("Invoice", inv);
            let iid = doc_id(self.src(), "Invoice", inv);
            let lk = self.link("settles", &key, &r.doc_id, &ik, &iid);
            if let Some(a) = amount {
                self.add(&lk, "link/amount", Value::Dec(a));
            }
        }
    }

    fn transfer(&mut self, r: &Root<'_>) {
        let (o, key) = (r.json, r.key.clone());
        self.observe(&key, &r.doc_id, "transfer");
        let amount = dec(o, "Amount");
        let date = self.date(o);
        if let Some(d) = &date {
            self.add(&key, "obs/date", d.clone());
        }
        // A transfer moves money between two accounts; obs/account holds one. Each side is its own leg.
        self.press(
            "two-accounts",
            "xero.BankTransfer",
            "one transfer, two accounts: modelled as two legs under it",
        );
        for (i, (field, sign)) in [
            ("FromBankAccount", Decimal::NEGATIVE_ONE),
            ("ToBankAccount", Decimal::ONE),
        ]
        .into_iter()
        .enumerate()
        {
            let Some(acct) = Self::nested_id(o, field, "AccountID") else {
                continue;
            };
            let ak = self.refer("Account", acct);
            let lid = format!("{}/{field}", r.doc_id);
            let lk = format!("obs:{lid}");
            self.snap.ent(&lk, &k("obs/id"), Value::str(&lid));
            if let Some(kv) = kw("obs.kind", "transfer-leg") {
                self.add(&lk, "obs/kind", kv);
            }
            self.add(&lk, "obs/parent", ValIn::Ref(temp(&key)));
            self.add(&lk, "obs/index", Value::Int(i64::try_from(i).unwrap_or(0)));
            self.add(&lk, "obs/account", ValIn::Ref(temp(&ak)));
            if let Some(a) = amount {
                self.add(&lk, "obs/amount", Value::Dec(sign * a));
                if let Some(d) = &date {
                    let fp = fingerprint(&self.mask(&ak), &d.text(), sign * a, "");
                    self.add(&lk, "obs/fingerprint", Value::str(&fp));
                }
            }
            if let Some(d) = &date {
                self.add(&lk, "obs/date", d.clone());
            }
        }
    }

    fn manual_journal(&mut self, r: &Root<'_>) {
        let o = r.json;
        let posted = s(o, "Status") == Some("POSTED");
        let Some(date) = self.date(o) else { return };
        if !posted {
            self.snap
                .gone(&k("entry/id"), Value::str(&format!("{}#entry", r.doc_id)));
            return;
        }
        let inclusive = s(o, "LineAmountTypes") == Some("Inclusive");
        let mut legs = Vec::new();
        let mut tax = Decimal::ZERO;
        for jl in o
            .get("JournalLines")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
        {
            let Some(jl) = jl.as_object() else { continue };
            let amount = dec(jl, "LineAmount").unwrap_or_default();
            let t = dec(jl, "TaxAmount").unwrap_or_default();
            tax += t;
            let net = if inclusive { amount - t } else { amount };
            let acct = s(jl, "AccountCode").and_then(|c| {
                self.account_by_code(c, "xero.ManualJournal.JournalLines/AccountCode")
            });
            legs.push((acct, net, s(jl, "Description").unwrap_or("").to_string()));
        }
        if !tax.is_zero() {
            let ta = self.tax_account();
            legs.push((ta, tax, "tax".into()));
        }
        let narration = s(o, "Narration").unwrap_or("").to_string();
        self.entry(r, date, &narration, legs);
    }
}

struct Line {
    code: Option<String>,
    net: Decimal,
    tax: Decimal,
    description: String,
}
