//! A loopback fake Xero for the write path's tests (never the real API): an HTTP/1.1 server on
//! `127.0.0.1:<random port>` with Xero-like answers for the four write endpoints and the reads the run needs
//! (connections, a manual journal or bank transaction by id, a marker search, accounts, attachments, the
//! TrialBalance report computed from what it holds).
//!
//! Faults are scripted per request ([`Fault`]): a 429 (nothing applied), a 500, a 2xx carrying
//! ValidationErrors, a dropped connection after the write was applied (the client sees an error after
//! sending), a refused edit of a reconciled bank transaction, a journal line limit, and the Idempotency-Key
//! behaviour (an exact replay returns the first answer; a changed body under the same key is refused or
//! replayed, configurable).

use rust_decimal::Decimal;
use serde_json::{Map, Value, json};
use std::collections::{BTreeMap, VecDeque};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

/// One scripted misbehaviour, consumed by the next write (PUT/POST) whose path starts with `path`.
#[derive(Clone, Debug, PartialEq)]
pub enum Fault {
    /// 429 minute limit with Retry-After 1 (nothing applied).
    RateLimited,
    /// 429 day limit (nothing applied).
    DayLimit,
    /// 500 after applying the write (the client can't tell whether it landed).
    ServerErrorAfterApply,
    /// 503 before applying anything.
    ServerErrorBeforeApply,
    /// Apply, then close the connection without answering.
    DropAfterApply,
    /// 200 with a ValidationErrors element (nothing applied).
    ValidationOn200,
}

/// What a changed body under a used Idempotency-Key gets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyReuse {
    /// 400 with a validation error (nothing applied).
    Refuse,
    /// The first answer again (the change is ignored).
    Replay,
}

#[derive(Debug)]
pub struct State {
    /// tenant id → name.
    pub tenants: BTreeMap<String, String>,
    /// tenant id → accounts (Xero shape: AccountID, Code, Name, Status, Class).
    pub accounts: BTreeMap<String, Vec<Value>>,
    /// tenant id → journal id → journal (Xero shape).
    pub journals: BTreeMap<String, BTreeMap<String, Value>>,
    /// tenant id → bank transaction id → transaction (Xero shape).
    pub bank: BTreeMap<String, BTreeMap<String, Value>>,
    /// (tenant, document id) → attachments (Xero shape, plus the bytes' length); journals and bank
    /// transactions.
    pub attachments: BTreeMap<(String, String), Vec<Value>>,
    /// AttachmentID → the file's bytes (served by `GET …/Attachments/{name}`).
    pub attachment_bytes: BTreeMap<String, Vec<u8>>,
    /// Xero's search index lags a fresh create: a document created through the fake stays out of `where`
    /// searches for this many searches (0: no lag). The GET by id always finds it.
    pub search_lag: u32,
    /// id → searches it is still hidden from.
    pub fresh: std::cell::RefCell<BTreeMap<String, u32>>,
    /// tenant id → bank transfer id → transfer (Xero shape).
    pub transfers: BTreeMap<String, BTreeMap<String, Value>>,
    /// tenant id → contact id → contact (Xero shape: ContactID, Name, ContactStatus, EmailAddress).
    pub contacts: BTreeMap<String, BTreeMap<String, Value>>,
    /// (tenant, `<Entity>/<id>`) → history records (Xero shape: Changes, DateUTCString, DateUTC, User, Details).
    pub history: BTreeMap<(String, String), Vec<Value>>,
    /// Idempotency-Key → (canonical body, status, answer).
    pub keys: BTreeMap<String, (String, u16, Value)>,
    pub key_reuse: KeyReuse,
    /// Manual journal line limit (Xero's real limit is what Milestone 1 measures).
    pub mj_line_limit: usize,
    /// Refuse an update of a reconciled bank transaction.
    pub refuse_reconciled_edit: bool,
    /// Refuse a void of a journal that has attachments.
    pub refuse_void_with_attachment: bool,
    /// On a bank-transaction update, also change this field (to test the round-trip check).
    pub alter_on_update: Option<(String, Value)>,
    /// (path prefix, fault), consumed in order.
    pub faults: VecDeque<(String, Fault)>,
    /// Every request: (method, path with query, Idempotency-Key, body).
    pub log: Vec<(String, String, Option<String>, String)>,
    /// Lock date (YYYY-MM-DD): journals on or before it are refused.
    pub lock_date: Option<String>,
    next_id: u64,
}

impl Default for State {
    fn default() -> Self {
        State {
            tenants: BTreeMap::new(),
            accounts: BTreeMap::new(),
            journals: BTreeMap::new(),
            bank: BTreeMap::new(),
            attachments: BTreeMap::new(),
            attachment_bytes: BTreeMap::new(),
            history: BTreeMap::new(),
            contacts: BTreeMap::new(),
            transfers: BTreeMap::new(),
            search_lag: 0,
            fresh: std::cell::RefCell::new(BTreeMap::new()),
            keys: BTreeMap::new(),
            key_reuse: KeyReuse::Refuse,
            mj_line_limit: 200,
            refuse_reconciled_edit: false,
            refuse_void_with_attachment: false,
            alter_on_update: None,
            faults: VecDeque::new(),
            log: vec![],
            lock_date: None,
            next_id: 1,
        }
    }
}

impl State {
    /// A GUID, as Xero issues them (synthetic: `0000000a-…` journals, `0000000b-…` attachments).
    fn new_id(&mut self, prefix: &str) -> String {
        let n = self.next_id;
        self.next_id += 1;
        let p = match prefix {
            "mj" => "0000000a",
            "att" => "0000000b",
            "bt" => "0000000c",
            "ct" => "0000000d",
            "tr" => "0000000e",
            _ => "0000000f",
        };
        format!("{p}-0000-4000-8000-{n:012}")
    }

    /// A tenant with a small chart: (code, name, class).
    pub fn add_tenant(&mut self, id: &str, name: &str, chart: &[(&str, &str, &str)]) {
        self.tenants.insert(id.into(), name.into());
        let accts = chart
            .iter()
            .map(|(code, nm, class)| {
                json!({"AccountID": format!("acc-{id}-{code}"), "Code": code, "Name": nm, "Type": class,
                       "Status": "ACTIVE", "Class": class})
            })
            .collect();
        self.accounts.insert(id.into(), accts);
    }

    /// Whether a document exists (the documents this fake holds: journals, bank transactions).
    pub fn has_doc(&self, tenant: &str, entity: &str, id: &str) -> bool {
        match entity {
            "ManualJournals" => self
                .journals
                .get(tenant)
                .is_some_and(|m| m.contains_key(id)),
            "BankTransactions" => self.bank.get(tenant).is_some_and(|m| m.contains_key(id)),
            "Contacts" => self
                .contacts
                .get(tenant)
                .is_some_and(|m| m.contains_key(id)),
            "BankTransfers" => self
                .transfers
                .get(tenant)
                .is_some_and(|m| m.contains_key(id)),
            _ => false,
        }
    }

    /// A document just created: hidden from `search_lag` searches.
    fn created(&self, id: &str) {
        if self.search_lag > 0 {
            self.fresh.borrow_mut().insert(id.into(), self.search_lag);
        }
    }

    /// Whether a search misses this document (the index hasn't caught up): counts the search down.
    fn unindexed(&self, id: &str) -> bool {
        let mut f = self.fresh.borrow_mut();
        match f.get_mut(id) {
            Some(n) if *n > 0 => {
                *n -= 1;
                true
            }
            _ => false,
        }
    }

    /// A contact (ACTIVE).
    pub fn add_contact(&mut self, tenant: &str, id: &str, name: &str) {
        self.contacts.entry(tenant.into()).or_default().insert(
            id.into(),
            json!({"ContactID": id, "Name": name, "ContactStatus": "ACTIVE",
                   "UpdatedDateUTC": "/Date(1741046400000+0000)/"}),
        );
    }

    /// A history record as Xero lists it.
    pub fn history_record(changes: &str, details: &str) -> Value {
        json!({"Changes": changes, "DateUTCString": "2026-10-07T21:14:03", "DateUTC": "/Date(1791407643000)/",
               "User": "ownpurse (API)", "Details": details})
    }

    pub fn account_id(&self, tenant: &str, code: &str) -> Option<String> {
        self.accounts
            .get(tenant)?
            .iter()
            .find(|a| a["Code"] == code)
            .and_then(|a| a["AccountID"].as_str().map(str::to_string))
    }

    /// A bank transaction (SPEND from `bank_code`, one line) the tests can recode.
    pub fn add_bank_spend(
        &mut self,
        tenant: &str,
        id: &str,
        bank_code: &str,
        line_code: &str,
        amount: &str,
        reconciled: bool,
    ) {
        let n = || {
            Value::Number(serde_json::Number::from_string_unchecked(
                amount.to_string(),
            ))
        };
        let bt = json!({
            "BankTransactionID": id, "Type": "SPEND", "Status": "AUTHORISED",
            "Contact": {"ContactID": "c-1", "Name": "Quillfeather Supplies"},
            "BankAccount": {"AccountID": self.account_id(tenant, bank_code), "Code": bank_code},
            "DateString": "2025-03-04T00:00:00", "Date": "/Date(1741046400000+0000)/",
            "LineAmountTypes": "NoTax", "IsReconciled": reconciled,
            "LineItems": [{"LineItemID": format!("{id}-L1"), "Description": "Sample tins",
                           "AccountCode": line_code, "LineAmount": n(),
                           "TaxAmount": 0, "Quantity": 1, "UnitAmount": n()}],
            "SubTotal": n(), "TotalTax": 0, "Total": n(),
            "UpdatedDateUTC": "/Date(1741046400000+0000)/", "CurrencyCode": "USD"
        });
        self.bank
            .entry(tenant.into())
            .or_default()
            .insert(id.into(), bt);
    }
}

/// The running server. Dropping it stops accepting (the listener thread ends on the next connection).
pub struct FakeXero {
    pub addr: std::net::SocketAddr,
    pub state: Arc<Mutex<State>>,
    stop: Arc<std::sync::atomic::AtomicBool>,
}

impl FakeXero {
    pub fn start(state: State) -> std::io::Result<FakeXero> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let addr = listener.local_addr()?;
        let state = Arc::new(Mutex::new(state));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let (st, sp) = (state.clone(), stop.clone());
        std::thread::spawn(move || {
            for conn in listener.incoming() {
                if sp.load(std::sync::atomic::Ordering::SeqCst) {
                    break;
                }
                let Ok(c) = conn else { continue };
                let st = st.clone();
                std::thread::spawn(move || {
                    let _ = serve(c, &st);
                });
            }
        });
        Ok(FakeXero { addr, state, stop })
    }

    /// `http://127.0.0.1:<port>/api.xro/2.0`
    pub fn api_base(&self) -> String {
        format!("http://{}/api.xro/2.0", self.addr)
    }
    pub fn connections_url(&self) -> String {
        format!("http://{}/connections", self.addr)
    }
    pub fn token_url(&self) -> String {
        format!("http://{}/identity/connect/token", self.addr)
    }

    pub fn with<T>(&self, f: impl FnOnce(&mut State) -> T) -> T {
        f(&mut self.state.lock().unwrap_or_else(|e| e.into_inner()))
    }
}

impl Drop for FakeXero {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        let _ = TcpStream::connect(self.addr);
    }
}

struct Req {
    method: String,
    path: String,
    query: String,
    headers: Vec<(String, String)>,
    body: Vec<u8>,
}

impl Req {
    fn header(&self, n: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(n))
            .map(|(_, v)| v.as_str())
    }
}

fn read_req(c: &TcpStream) -> std::io::Result<Req> {
    let mut r = BufReader::new(c.try_clone()?);
    let mut line = String::new();
    r.read_line(&mut line)?;
    let mut parts = line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let target = parts.next().unwrap_or("").to_string();
    let (path, query) = target.split_once('?').unwrap_or((&target, ""));
    let mut headers = vec![];
    loop {
        let mut h = String::new();
        r.read_line(&mut h)?;
        let h = h.trim_end();
        if h.is_empty() {
            break;
        }
        if let Some((k, v)) = h.split_once(':') {
            headers.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    let len: usize = headers
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case("content-length"))
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    let mut body = vec![0u8; len];
    r.read_exact(&mut body)?;
    Ok(Req {
        method,
        path: path.to_string(),
        query: query.to_string(),
        headers,
        body,
    })
}

fn write_resp(
    mut c: &TcpStream,
    status: u16,
    extra: &[(&str, String)],
    body: &Value,
) -> std::io::Result<()> {
    let raw = extra.iter().any(|(k, _)| *k == "X-Fake-Raw");
    let text = match body {
        Value::Null => String::new(),
        Value::String(s) if raw => s.clone(),
        _ => body.to_string(),
    };
    let mut head = format!(
        "HTTP/1.1 {status} X\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n",
        if raw {
            "application/octet-stream"
        } else {
            "application/json"
        },
        text.len()
    );
    for (k, v) in extra.iter().filter(|(k, _)| *k != "X-Fake-Raw") {
        head.push_str(&format!("{k}: {v}\r\n"));
    }
    head.push_str("\r\n");
    c.write_all(head.as_bytes())?;
    c.write_all(text.as_bytes())?;
    c.flush()
}

fn url_decode(s: &str) -> String {
    let b = s.as_bytes();
    let mut out = vec![];
    let mut i = 0;
    while i < b.len() {
        match b[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < b.len() => {
                if let Ok(v) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(v);
                    i += 2;
                }
            }
            x => out.push(x),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn validation(msg: &str) -> Value {
    json!({"ErrorNumber": 10, "Type": "ValidationException", "Message": "A validation exception occurred",
           "Elements": [{"ValidationErrors": [{"Message": msg}]}]})
}

fn dec(v: &Value) -> Decimal {
    match v {
        Value::Number(n) => n.to_string().parse().unwrap_or_default(),
        Value::String(s) => s.parse().unwrap_or_default(),
        _ => Decimal::ZERO,
    }
}

/// The answer to one request: (status, extra headers, body), or None to drop the connection.
type Answer = Option<(u16, Vec<(&'static str, String)>, Value)>;

fn serve(c: TcpStream, st: &Mutex<State>) -> std::io::Result<()> {
    let req = read_req(&c)?;
    let answer = {
        let mut s = st.lock().unwrap_or_else(|e| e.into_inner());
        let full = if req.query.is_empty() {
            req.path.clone()
        } else {
            format!("{}?{}", req.path, req.query)
        };
        s.log.push((
            req.method.clone(),
            full,
            req.header("Idempotency-Key").map(str::to_string),
            String::from_utf8_lossy(&req.body).into_owned(),
        ));
        handle(&mut s, &req)
    };
    match answer {
        Some((status, extra, body)) => write_resp(&c, status, &extra, &body),
        None => {
            let _ = c.shutdown(std::net::Shutdown::Both);
            Ok(())
        }
    }
}

fn handle(s: &mut State, req: &Req) -> Answer {
    if req
        .header("Authorization")
        .is_none_or(|a| !a.starts_with("Bearer "))
    {
        return Some((401, vec![], json!({"Title": "Unauthorized"})));
    }
    if req.path == "/connections" {
        let all: Vec<Value> = s
            .tenants
            .iter()
            .map(|(id, n)| json!({"tenantId": id, "tenantName": n, "tenantType": "ORGANISATION"}))
            .collect();
        return Some((200, vec![], Value::Array(all)));
    }
    let Some(rel) = req.path.strip_prefix("/api.xro/2.0/") else {
        return Some((404, vec![], Value::Null));
    };
    let tenant = req.header("Xero-Tenant-Id").unwrap_or("").to_string();
    if !s.tenants.contains_key(&tenant) {
        return Some((
            403,
            vec![],
            json!({"Title": "Forbidden", "Detail": "AuthenticationUnsuccessful"}),
        ));
    }
    let segs: Vec<String> = rel.split('/').map(url_decode).collect();
    if req.method == "GET" {
        let (st, h, mut body) = read(s, &tenant, &segs, &req.query);
        // Xero's API envelope around every list answer
        if st == 200
            && let Some(m) = body.as_object_mut()
            && !m.contains_key("Reports")
        {
            m.insert("Id".into(), json!("fake-envelope-0001"));
            m.insert("Status".into(), json!("OK"));
            m.insert("ProviderName".into(), json!("ownpurse fake"));
            m.insert("DateTimeUTC".into(), json!("/Date(1791331200000)/"));
        }
        return Some((st, h, body));
    }
    // a write: scripted faults first
    let fault = s
        .faults
        .iter()
        .position(|(p, _)| rel.starts_with(p.as_str()))
        .and_then(|i| s.faults.remove(i))
        .map(|(_, f)| f);
    match fault {
        Some(Fault::RateLimited) => {
            return Some((
                429,
                vec![
                    ("Retry-After", "1".into()),
                    ("X-Rate-Limit-Problem", "minute".into()),
                ],
                Value::Null,
            ));
        }
        Some(Fault::DayLimit) => {
            return Some((
                429,
                vec![("X-Rate-Limit-Problem", "day".into())],
                Value::Null,
            ));
        }
        Some(Fault::ServerErrorBeforeApply) => return Some((503, vec![], Value::Null)),
        Some(Fault::ValidationOn200) => {
            return Some((
                200,
                vec![],
                json!({"ManualJournals": [{"ValidationErrors": [{"Message": "Scripted validation error"}],
                                           "StatusAttributeString": "ERROR"}]}),
            ));
        }
        _ => {}
    }
    // idempotency
    let key = req.header("Idempotency-Key").map(str::to_string);
    let body_text = String::from_utf8_lossy(&req.body).into_owned();
    if let Some(k) = &key
        && let Some((b, status, ans)) = s.keys.get(k).cloned()
    {
        if b == body_text || s.key_reuse == KeyReuse::Replay {
            return Some((status, vec![], ans));
        }
        return Some((
            400,
            vec![],
            validation("The Idempotency-Key was already used with a different request body"),
        ));
    }
    let (status, ans) = write(s, &tenant, &segs, req);
    if let Some(k) = key
        && status < 500
    {
        s.keys.insert(k, (body_text, status, ans.clone()));
    }
    match fault {
        Some(Fault::ServerErrorAfterApply) => Some((500, vec![], Value::Null)),
        Some(Fault::DropAfterApply) => None,
        _ => Some((status, vec![], ans)),
    }
}

fn read(
    s: &State,
    tenant: &str,
    segs: &[String],
    query: &str,
) -> (u16, Vec<(&'static str, String)>, Value) {
    let q: BTreeMap<String, String> = query
        .split('&')
        .filter_map(|kv| kv.split_once('='))
        .map(|(k, v)| (url_decode(k), url_decode(v)))
        .collect();
    let none = BTreeMap::new();
    let journals = s.journals.get(tenant).unwrap_or(&none);
    match segs
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .as_slice()
    {
        ["Accounts"] => (
            200,
            vec![],
            json!({"Accounts": s.accounts.get(tenant).cloned().unwrap_or_default()}),
        ),
        ["ManualJournals"] => {
            // `where=Narration.Contains("XA:…")` is the marker search
            let needle = q
                .get("where")
                .and_then(|w| w.split_once("Narration.Contains(\""))
                .and_then(|(_, r)| r.split_once("\")"))
                .map(|(n, _)| n.to_string());
            let found: Vec<Value> = journals
                .values()
                .filter(|j| {
                    needle
                        .as_deref()
                        .is_none_or(|n| j["Narration"].as_str().is_some_and(|t| t.contains(n)))
                })
                .filter(|j| {
                    needle.is_none() || !s.unindexed(j["ManualJournalID"].as_str().unwrap_or(""))
                })
                // as real Xero: a list carries no JournalLines
                .map(|j| {
                    let mut j = j.clone();
                    if let Some(m) = j.as_object_mut() {
                        m.remove("JournalLines");
                    }
                    j
                })
                .collect();
            (200, vec![], json!({"ManualJournals": found}))
        }
        ["ManualJournals", id] => match journals.get(*id) {
            Some(j) => (200, vec![], json!({"ManualJournals": [j]})),
            None => (404, vec![], Value::Null),
        },
        // a file's bytes, as Xero serves them (the raw body; `X-Fake-Raw` tells write_resp so)
        [entity, id, "Attachments", name] if s.has_doc(tenant, entity, id) => {
            let found = s
                .attachments
                .get(&(tenant.to_string(), (*id).to_string()))
                .and_then(|l| l.iter().find(|a| a["FileName"] == *name))
                .and_then(|a| a["AttachmentID"].as_str())
                .and_then(|aid| s.attachment_bytes.get(aid));
            match found {
                Some(b) => (
                    200,
                    vec![("X-Fake-Raw", "1".into())],
                    Value::String(String::from_utf8_lossy(b).into_owned()),
                ),
                None => (404, vec![], Value::Null),
            }
        }
        ["BankTransactions", id, "Attachments"] if s.has_doc(tenant, "BankTransactions", id) => {
            let a = s
                .attachments
                .get(&(tenant.to_string(), (*id).to_string()))
                .cloned()
                .unwrap_or_default();
            (200, vec![], json!({"Attachments": a}))
        }
        ["ManualJournals", id, "Attachments"] => {
            let a = s
                .attachments
                .get(&(tenant.to_string(), (*id).to_string()))
                .cloned()
                .unwrap_or_default();
            (200, vec![], json!({"Attachments": a}))
        }
        [entity, id, "History"] => {
            if !s.has_doc(tenant, entity, id) {
                return (404, vec![], Value::Null);
            }
            let h = s
                .history
                .get(&(tenant.to_string(), format!("{entity}/{id}")))
                .cloned()
                .unwrap_or_default();
            (200, vec![], json!({"HistoryRecords": h}))
        }
        ["BankTransactions", id] => match s.bank.get(tenant).and_then(|b| b.get(*id)) {
            Some(t) => (
                200,
                vec![],
                json!({"BankTransactions": [bank_by_id(s, tenant, t)]}),
            ),
            None => (404, vec![], Value::Null),
        },
        ["Organisation"] => (
            200,
            vec![],
            json!({"Organisations": [{"OrganisationID": tenant, "Name": s.tenants.get(tenant),
                                      "BaseCurrency": "USD", "OrganisationStatus": "ACTIVE", "ShortCode": "!Fk3dQ",
                                      "FinancialYearEndDay": 31, "FinancialYearEndMonth": 12,
                                      "PeriodLockDate": s.lock_date}]}),
        ),
        ["BankTransactions"] => {
            // `where=Reference.Contains("XA:…")` is the marker search
            let needle = q
                .get("where")
                .and_then(|w| w.split_once("Reference.Contains(\""))
                .and_then(|(_, r)| r.split_once("\")"))
                .map(|(n, _)| n.to_string());
            let found: Vec<Value> = s
                .bank
                .get(tenant)
                .map(|m| {
                    m.values()
                        .filter(|t| {
                            needle.as_deref().is_none_or(|n| {
                                t["Reference"].as_str().is_some_and(|r| r.contains(n))
                            })
                        })
                        .filter(|t| {
                            needle.is_none()
                                || !s.unindexed(t["BankTransactionID"].as_str().unwrap_or(""))
                        })
                        .map(bank_in_list)
                        .collect()
                })
                .unwrap_or_default();
            (200, vec![], json!({ "BankTransactions": found }))
        }
        ["Contacts"] => {
            // `where=Name=="…"` (the duplicate guard), else all
            let name = q
                .get("where")
                .and_then(|w| w.split_once("Name==\""))
                .and_then(|(_, r)| r.rsplit_once('"'))
                .map(|(n, _)| n.to_string());
            let found: Vec<Value> = s
                .contacts
                .get(tenant)
                .map(|m| {
                    m.values()
                        .filter(|c| name.as_deref().is_none_or(|n| c["Name"] == n))
                        .filter(|c| {
                            name.is_none() || !s.unindexed(c["ContactID"].as_str().unwrap_or(""))
                        })
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            (200, vec![], json!({ "Contacts": found }))
        }
        ["Accounts", id] => match s
            .accounts
            .get(tenant)
            .and_then(|a| a.iter().find(|x| x["AccountID"] == *id))
        {
            Some(a) => (200, vec![], json!({"Accounts": [a]})),
            None => (404, vec![], Value::Null),
        },
        ["BankTransfers"] => {
            let needle = q
                .get("where")
                .and_then(|w| w.split_once("Reference.Contains(\""))
                .and_then(|(_, r)| r.split_once("\")"))
                .map(|(n, _)| n.to_string());
            let found: Vec<Value> = s
                .transfers
                .get(tenant)
                .map(|m| {
                    m.values()
                        .filter(|t| {
                            needle.as_deref().is_none_or(|n| {
                                t["Reference"].as_str().is_some_and(|r| r.contains(n))
                            })
                        })
                        .filter(|t| {
                            needle.is_none()
                                || !s.unindexed(t["BankTransferID"].as_str().unwrap_or(""))
                        })
                        .cloned()
                        .collect()
                })
                .unwrap_or_default();
            (200, vec![], json!({ "BankTransfers": found }))
        }
        ["BankTransfers", id] => match s.transfers.get(tenant).and_then(|m| m.get(*id)) {
            Some(t) => (200, vec![], json!({"BankTransfers": [t]})),
            None => (404, vec![], Value::Null),
        },
        ["Contacts", id] => match s.contacts.get(tenant).and_then(|m| m.get(*id)) {
            Some(c) => (200, vec![], json!({"Contacts": [c]})),
            None => (404, vec![], Value::Null),
        },
        // the rest of a sync (contacts, invoices, payments …): this fake holds none
        [entity] if entity.chars().all(|c| c.is_ascii_alphabetic()) => {
            (200, vec![], json!({ *entity: [] }))
        }
        ["Reports", "TrialBalance"] => (
            200,
            vec![],
            trial_balance(s, tenant, q.get("date").map(String::as_str)),
        ),
        // the other reports, in Xero's envelope: a header row and one section built from the same balances
        [
            "Reports",
            r @ ("BalanceSheet" | "ProfitAndLoss" | "BankSummary"),
        ] => {
            let date = q
                .get("date")
                .or_else(|| q.get("toDate"))
                .map(String::as_str);
            let tb = trial_balance(s, tenant, date);
            let rows = tb["Reports"][0]["Rows"][1]["Rows"].clone();
            let title = match *r {
                "BalanceSheet" => "Balance Sheet",
                "ProfitAndLoss" => "Profit and Loss",
                _ => "Bank Summary",
            };
            (
                200,
                vec![],
                json!({"Reports": [{"ReportID": r, "ReportName": title, "ReportType": r,
                    "ReportTitles": [title, s.tenants.get(tenant), date],
                    "ReportDate": date, "UpdatedDateUTC": "/Date(1791200000000)/",
                    "Rows": [{"RowType": "Header", "Cells": [{"Value": ""}, {"Value": date}]},
                             {"RowType": "Section", "Title": title, "Rows": rows}]}]}),
            )
        }
        _ => (404, vec![], Value::Null),
    }
}

/// A bank transaction as real Xero lists it (`GET BankTransactions?page=…`): the contact's id and name only,
/// the bank account's id and code, lines without AccountID or TaxType, amounts as stored.
fn bank_in_list(t: &Value) -> Value {
    let mut t = t.clone();
    if let Some(c) = t.get_mut("Contact").and_then(Value::as_object_mut) {
        c.retain(|k, _| k == "ContactID" || k == "Name");
    }
    if let Some(b) = t.get_mut("BankAccount").and_then(Value::as_object_mut) {
        b.retain(|k, _| k == "AccountID" || k == "Code");
    }
    for l in t
        .get_mut("LineItems")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        if let Some(l) = l.as_object_mut() {
            l.remove("AccountID");
            l.remove("TaxType");
            l.remove("Tracking");
        }
    }
    if let Some(m) = t.as_object_mut() {
        m.remove("HasAttachments");
        m.remove("Url");
    }
    t
}

/// A bank transaction as real Xero returns it by id (`GET BankTransactions/{id}`): the full contact, the bank
/// account's name, each line's AccountID, TaxType and Tracking, `HasAttachments`, and amounts with four
/// decimals (`238.6100`).
fn bank_by_id(s: &State, tenant: &str, t: &Value) -> Value {
    let mut t = t.clone();
    let four = |v: &Value| {
        let d = dec(v);
        Value::Number(serde_json::Number::from_string_unchecked(format!("{d:.4}")))
    };
    if let Some(c) = t.get_mut("Contact").and_then(Value::as_object_mut) {
        c.insert("ContactStatus".into(), json!("ACTIVE"));
        c.insert(
            "Addresses".into(),
            json!([{"AddressType": "POBOX"}, {"AddressType": "STREET"}]),
        );
        c.insert("Phones".into(), json!([]));
        c.insert("HasValidationErrors".into(), json!(false));
    }
    let bank_code = t["BankAccount"]["Code"].as_str().unwrap_or("").to_string();
    let bank_name = s
        .accounts
        .get(tenant)
        .and_then(|a| a.iter().find(|x| x["Code"] == bank_code.as_str()))
        .and_then(|a| a["Name"].as_str())
        .unwrap_or("")
        .to_string();
    if let Some(b) = t.get_mut("BankAccount").and_then(Value::as_object_mut) {
        b.insert("Name".into(), json!(bank_name));
    }
    for l in t
        .get_mut("LineItems")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        let code = l["AccountCode"].as_str().unwrap_or("").to_string();
        if let Some(m) = l.as_object_mut() {
            m.insert("AccountID".into(), json!(s.account_id(tenant, &code)));
            m.entry("TaxType").or_insert(json!("NONE"));
            m.entry("Tracking").or_insert(json!([]));
            for k in ["LineAmount", "TaxAmount", "UnitAmount", "Quantity"] {
                if let Some(v) = m.get(k).cloned() {
                    m.insert(k.into(), four(&v));
                }
            }
        }
    }
    if let Some(m) = t.as_object_mut() {
        m.insert("HasAttachments".into(), json!(false));
        for k in ["SubTotal", "TotalTax", "Total"] {
            if let Some(v) = m.get(k).cloned() {
                m.insert(k.into(), four(&v));
            }
        }
    }
    t
}

/// TrialBalance (Xero's shape: a header row, sections of rows with the AccountID attribute, YTD columns)
/// from the POSTED journals and AUTHORISED bank transactions up to `date`.
fn trial_balance(s: &State, tenant: &str, date: Option<&str>) -> Value {
    let mut bal: BTreeMap<String, Decimal> = BTreeMap::new();
    let upto = |d: &str| date.is_none_or(|x| d[..10.min(d.len())] <= *x);
    for j in s.journals.get(tenant).into_iter().flat_map(|m| m.values()) {
        let jd = j["Date"]
            .as_str()
            .and_then(ownpurse_core::dates::xero_date)
            .unwrap_or_default();
        if j["Status"] != "POSTED" || !upto(&jd) {
            continue;
        }
        for l in j["JournalLines"].as_array().into_iter().flatten() {
            if let Some(id) = s.account_id(tenant, l["AccountCode"].as_str().unwrap_or("")) {
                *bal.entry(id).or_default() += dec(&l["LineAmount"]);
            }
        }
    }
    for t in s.bank.get(tenant).into_iter().flat_map(|m| m.values()) {
        if t["Status"] != "AUTHORISED" || !upto(t["DateString"].as_str().unwrap_or("")) {
            continue;
        }
        let sign = if t["Type"] == "SPEND" {
            Decimal::ONE
        } else {
            -Decimal::ONE
        };
        for l in t["LineItems"].as_array().into_iter().flatten() {
            let amt = dec(&l["LineAmount"]) * sign;
            if let Some(id) = s.account_id(tenant, l["AccountCode"].as_str().unwrap_or("")) {
                *bal.entry(id).or_default() += amt;
            }
            if let Some(b) = t["BankAccount"]["AccountID"].as_str() {
                *bal.entry(b.to_string()).or_default() -= amt;
            }
        }
    }
    for t in s.transfers.get(tenant).into_iter().flat_map(|m| m.values()) {
        if !upto(t["DateString"].as_str().unwrap_or("")) {
            continue;
        }
        let a = dec(&t["Amount"]);
        if let Some(f) = t["FromBankAccount"]["AccountID"].as_str() {
            *bal.entry(f.to_string()).or_default() -= a;
        }
        if let Some(to) = t["ToBankAccount"]["AccountID"].as_str() {
            *bal.entry(to.to_string()).or_default() += a;
        }
    }
    let accts = s.accounts.get(tenant).cloned().unwrap_or_default();
    let rows: Vec<Value> = bal
        .iter()
        .filter(|(_, v)| !v.is_zero())
        .map(|(id, v)| {
            let a = accts.iter().find(|a| a["AccountID"] == id.as_str());
            let label = a.map_or(id.clone(), |a| {
                format!("{} ({})", a["Name"].as_str().unwrap_or(""), a["Code"].as_str().unwrap_or(""))
            });
            let (dr, cr) = if v.is_sign_negative() { (Decimal::ZERO, -*v) } else { (*v, Decimal::ZERO) };
            let attr = json!([{"Value": id, "Id": "account"}]);
            json!({"RowType": "Row", "Cells": [
                {"Value": label, "Attributes": attr},
                {"Value": dr.to_string(), "Attributes": attr}, {"Value": cr.to_string(), "Attributes": attr},
                {"Value": dr.to_string(), "Attributes": attr}, {"Value": cr.to_string(), "Attributes": attr}]})
        })
        .collect();
    json!({"Reports": [{"ReportID": "TrialBalance", "ReportName": "Trial Balance", "Rows": [
        {"RowType": "Header", "Cells": [{"Value": "Account"}, {"Value": "Debit"}, {"Value": "Credit"},
                                        {"Value": "YTD Debit"}, {"Value": "YTD Credit"}]},
        {"RowType": "Section", "Title": "Accounts", "Rows": rows}]}]})
}

fn write(s: &mut State, tenant: &str, segs: &[String], req: &Req) -> (u16, Value) {
    let segs: Vec<&str> = segs.iter().map(String::as_str).collect();
    match (req.method.as_str(), segs.as_slice()) {
        // a note in the document's history (Xero answers with the record it added)
        ("PUT", [entity, id, "History"]) => {
            if !s.has_doc(tenant, entity, id) {
                return (404, Value::Null);
            }
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let details = body["HistoryRecords"][0]["Details"]
                .as_str()
                .unwrap_or("")
                .to_string();
            if details.is_empty() || details.chars().count() > 450 {
                return (
                    400,
                    validation("A history note needs Details of at most 450 characters"),
                );
            }
            let r = State::history_record("Note", &details);
            s.history
                .entry((tenant.to_string(), format!("{entity}/{id}")))
                .or_default()
                .push(r.clone());
            (200, json!({"HistoryRecords": [r]}))
        }
        ("PUT", ["BankTransfers"]) => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let t = body["BankTransfers"]
                .get(0)
                .cloned()
                .unwrap_or(body.clone());
            let bank = |id: &str| {
                s.accounts.get(tenant).and_then(|a| {
                    a.iter()
                        .find(|x| {
                            x["AccountID"] == id && x["Type"] == "BANK" && x["Status"] == "ACTIVE"
                        })
                        .cloned()
                })
            };
            let from = t["FromBankAccount"]["AccountID"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let to = t["ToBankAccount"]["AccountID"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let (Some(fa), Some(ta)) = (bank(&from), bank(&to)) else {
                return (
                    400,
                    validation("A valid FromBankAccount and ToBankAccount are required"),
                );
            };
            if from == to {
                return (
                    400,
                    validation("The from and to bank accounts must be different"),
                );
            }
            let amount = dec(&t["Amount"]);
            if amount <= Decimal::ZERO {
                return (400, validation("Amount must be greater than zero"));
            }
            let date = t["Date"].as_str().unwrap_or("").to_string();
            if let Some(lock) = &s.lock_date
                && date.as_str() <= lock.as_str()
            {
                return (
                    400,
                    validation("The document date cannot be before the period lock date"),
                );
            }
            let id = s.new_id("tr");
            s.created(&id);
            let ms = ownpurse_core::dates::parse_iso(&date)
                .map(|(y, m, d)| ownpurse_core::dates::days_from_civil(y, m, d) * 86_400_000)
                .unwrap_or(0);
            let tr = json!({
                "BankTransferID": id,
                "FromBankAccount": {"AccountID": from, "Code": fa["Code"], "Name": fa["Name"]},
                "ToBankAccount": {"AccountID": to, "Code": ta["Code"], "Name": ta["Name"]},
                "Amount": Value::Number(serde_json::Number::from_string_unchecked(amount.to_string())),
                "Date": format!("/Date({ms}+0000)/"), "DateString": format!("{date}T00:00:00"),
                "Reference": t["Reference"], "CurrencyRate": 1.0,
                "FromBankTransactionID": format!("{id}-from"), "ToBankTransactionID": format!("{id}-to"),
                "HasAttachments": false, "CreatedDateUTC": "/Date(1791200000000+0000)/"
            });
            s.transfers
                .entry(tenant.into())
                .or_default()
                .insert(id, tr.clone());
            (200, json!({"BankTransfers": [tr]}))
        }
        ("PUT", ["Contacts"]) => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let c = body["Contacts"].get(0).cloned().unwrap_or(body.clone());
            let name = c["Name"].as_str().unwrap_or("").to_string();
            if name.trim().is_empty() {
                return (400, validation("The contact name is required"));
            }
            if s.contacts
                .get(tenant)
                .is_some_and(|m| m.values().any(|x| x["Name"] == name.as_str()))
            {
                return (
                    400,
                    validation(&format!(
                        "The contact name {name} is already assigned to another contact. The contact name must be unique across all active contacts."
                    )),
                );
            }
            let id = s.new_id("ct");
            s.created(&id);
            s.add_contact(tenant, &id, &name);
            if let Some(e) = c["EmailAddress"].as_str()
                && let Some(x) = s.contacts.get_mut(tenant).and_then(|m| m.get_mut(&id))
            {
                x["EmailAddress"] = json!(e);
            }
            let out = s.contacts[tenant][&id].clone();
            (200, json!({"Contacts": [out]}))
        }
        ("POST", ["Contacts", id]) => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let upd = body["Contacts"].get(0).cloned().unwrap_or(body);
            let Some(c) = s.contacts.get_mut(tenant).and_then(|m| m.get_mut(*id)) else {
                return (404, Value::Null);
            };
            if let Some(st) = upd["ContactStatus"].as_str() {
                c["ContactStatus"] = json!(st);
            }
            c["UpdatedDateUTC"] = json!("/Date(1791200000000+0000)/");
            (200, json!({"Contacts": [c.clone()]}))
        }
        ("PUT", ["BankTransactions"]) => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let t = body["BankTransactions"]
                .get(0)
                .cloned()
                .unwrap_or(body.clone());
            create_bank(s, tenant, t)
        }
        ("PUT", ["ManualJournals"]) => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let j = body["ManualJournals"]
                .get(0)
                .cloned()
                .unwrap_or(body.clone());
            create_journal(s, tenant, j)
        }
        ("POST", ["ManualJournals", id]) => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let upd = body["ManualJournals"].get(0).cloned().unwrap_or(body);
            // as Xero: an id in the body must be a GUID (else ErrorNumber 14, PostDataInvalidException)
            if let Some(v) = upd.get("ManualJournalID").and_then(Value::as_str)
                && !is_guid(v)
            {
                return (
                    400,
                    json!({"ErrorNumber": 14, "Type": "PostDataInvalidException",
                           "Message": format!("Error converting value \"{v}\" to type 'System.Guid'. Path 'ManualJournals[0].ManualJournalID', line 1, position 40.")}),
                );
            }
            let has_att = s
                .attachments
                .get(&(tenant.to_string(), (*id).to_string()))
                .is_some_and(|a| !a.is_empty());
            let refuse_att = s.refuse_void_with_attachment;
            let Some(j) = s.journals.get_mut(tenant).and_then(|m| m.get_mut(*id)) else {
                return (404, Value::Null);
            };
            if upd["Status"] == "VOIDED" {
                if j["Status"] != "POSTED" {
                    return (400, validation("Only POSTED manual journals can be voided"));
                }
                if has_att && refuse_att {
                    return (
                        400,
                        validation("A manual journal with attachments cannot be voided"),
                    );
                }
                j["Status"] = json!("VOIDED");
                j["UpdatedDateUTC"] = json!("/Date(1791200000000+0000)/");
                return (200, json!({"ManualJournals": [j.clone()]}));
            }
            (
                400,
                validation("Only a status change to VOIDED is supported by this fake"),
            )
        }
        ("POST", ["BankTransactions", id]) => {
            let body: Value = serde_json::from_slice(&req.body).unwrap_or(Value::Null);
            let upd = body["BankTransactions"].get(0).cloned().unwrap_or(body);
            let refuse = s.refuse_reconciled_edit;
            let alter = s.alter_on_update.clone();
            let Some(t) = s.bank.get_mut(tenant).and_then(|m| m.get_mut(*id)) else {
                return (404, Value::Null);
            };
            if refuse && t["IsReconciled"] == true {
                return (
                    400,
                    validation("This Bank Transaction cannot be edited as it has been reconciled"),
                );
            }
            // the API's void: Status DELETED (refused once reconciled, whatever the switches)
            if upd["Status"] == "DELETED" {
                if t["IsReconciled"] == true {
                    return (
                        400,
                        validation(
                            "This Bank Transaction cannot be deleted as it has been reconciled",
                        ),
                    );
                }
                t["Status"] = json!("DELETED");
                t["UpdatedDateUTC"] = json!("/Date(1791200000000+0000)/");
                return (200, json!({"BankTransactions": [t.clone()]}));
            }
            if let Some(m) = upd.as_object() {
                for (k, v) in m {
                    if k != "UpdatedDateUTC" {
                        t[k.as_str()] = v.clone();
                    }
                }
            }
            if let Some((k, v)) = alter {
                t[k.as_str()] = v;
            }
            t["UpdatedDateUTC"] = json!("/Date(1791200000000+0000)/");
            (200, json!({"BankTransactions": [t.clone()]}))
        }
        (
            "PUT",
            [
                entity @ ("ManualJournals" | "BankTransactions"),
                id,
                "Attachments",
                name,
            ],
        ) => {
            if !s.has_doc(tenant, entity, id) {
                return (404, Value::Null);
            }
            let aid = s.new_id("att");
            s.attachment_bytes.insert(aid.clone(), req.body.clone());
            let a = json!({"AttachmentID": aid, "FileName": name,
                           "MimeType": req.header("Content-Type").unwrap_or(""),
                           "ContentLength": req.body.len()});
            s.attachments
                .entry((tenant.to_string(), (*id).to_string()))
                .or_default()
                .push(a.clone());
            if let Some(j) = s.journals.get_mut(tenant).and_then(|m| m.get_mut(*id)) {
                j["HasAttachments"] = json!(true);
            }
            if let Some(t) = s.bank.get_mut(tenant).and_then(|m| m.get_mut(*id)) {
                t["HasAttachments"] = json!(true);
            }
            (200, json!({"Attachments": [a]}))
        }
        _ => (405, validation("Not an endpoint of the fake")),
    }
}

/// A bank transaction create, as Xero checks and stores it: SPEND / RECEIVE, a known ACTIVE BANK account by
/// id, a known contact by id (a name the fake doesn't know is refused: real Xero would create a contact),
/// valid line accounts, the lock date; stored with ids, totals and the /Date()/ form.
fn create_bank(s: &mut State, tenant: &str, t: Value) -> (u16, Value) {
    let kind = t["Type"].as_str().unwrap_or("").to_string();
    if kind != "SPEND" && kind != "RECEIVE" {
        return (400, validation("Type must be SPEND or RECEIVE"));
    }
    let bank_id = t["BankAccount"]["AccountID"]
        .as_str()
        .unwrap_or("")
        .to_string();
    let bank = s.accounts.get(tenant).and_then(|a| {
        a.iter()
            .find(|x| {
                x["AccountID"] == bank_id.as_str() && x["Type"] == "BANK" && x["Status"] == "ACTIVE"
            })
            .cloned()
    });
    let Some(bank) = bank else {
        return (400, validation("A valid BankAccount is required"));
    };
    let cid = t["Contact"]["ContactID"].as_str().unwrap_or("").to_string();
    let Some(contact) = s.contacts.get(tenant).and_then(|m| m.get(&cid)).cloned() else {
        return (400, validation("A valid Contact is required"));
    };
    let lines = t["LineItems"].as_array().cloned().unwrap_or_default();
    if lines.is_empty() {
        return (400, validation("A bank transaction needs a line item"));
    }
    for l in &lines {
        let code = l["AccountCode"].as_str().unwrap_or("");
        let ok = s.accounts.get(tenant).is_some_and(|a| {
            a.iter()
                .any(|x| x["Code"] == code && x["Status"] == "ACTIVE" && x["Type"] != "BANK")
        });
        if !ok {
            return (
                400,
                validation(&format!("Account code '{code}' is not a valid code")),
            );
        }
    }
    let date = t["Date"].as_str().unwrap_or("").to_string();
    if let Some(lock) = &s.lock_date
        && date.as_str() <= lock.as_str()
    {
        return (
            400,
            validation("The document date cannot be before the period lock date"),
        );
    }
    let id = s.new_id("bt");
    s.created(&id);
    let ms = ownpurse_core::dates::parse_iso(&date)
        .map(|(y, m, d)| ownpurse_core::dates::days_from_civil(y, m, d) * 86_400_000)
        .unwrap_or(0);
    let n = |d: Decimal| Value::Number(serde_json::Number::from_string_unchecked(d.to_string()));
    let mut total = Decimal::ZERO;
    let stored_lines: Vec<Value> = lines
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let a = dec(&l["LineAmount"]);
            total += a;
            json!({"LineItemID": format!("{id}-L{}", i + 1), "Description": l["Description"],
                   "AccountCode": l["AccountCode"], "LineAmount": n(a),
                   "TaxAmount": 0, "Quantity": 1, "UnitAmount": n(a)})
        })
        .collect();
    let bt = json!({
        "BankTransactionID": id, "Type": kind, "Status": "AUTHORISED",
        "Contact": {"ContactID": cid, "Name": contact["Name"]},
        "BankAccount": {"AccountID": bank_id, "Code": bank["Code"]},
        "DateString": format!("{date}T00:00:00"), "Date": format!("/Date({ms}+0000)/"),
        "Reference": t["Reference"], "LineAmountTypes": "NoTax", "IsReconciled": false,
        "LineItems": stored_lines, "SubTotal": n(total), "TotalTax": 0, "Total": n(total),
        "UpdatedDateUTC": "/Date(1791200000000+0000)/", "CurrencyCode": "USD"
    });
    s.bank
        .entry(tenant.into())
        .or_default()
        .insert(id.clone(), bt.clone());
    (
        200,
        json!({"BankTransactions": [bank_by_id(s, tenant, &bt)]}),
    )
}

fn create_journal(s: &mut State, tenant: &str, j: Value) -> (u16, Value) {
    let lines = j["JournalLines"].as_array().cloned().unwrap_or_default();
    if lines.len() > s.mj_line_limit {
        return (
            400,
            validation(&format!(
                "A manual journal can have at most {} lines",
                s.mj_line_limit
            )),
        );
    }
    let total: Decimal = lines.iter().map(|l| dec(&l["LineAmount"])).sum();
    if !total.is_zero() || lines.len() < 2 {
        return (
            400,
            validation("The total debits must equal the total credits"),
        );
    }
    for l in &lines {
        let code = l["AccountCode"].as_str().unwrap_or("");
        let ok = s.accounts.get(tenant).is_some_and(|a| {
            a.iter()
                .any(|x| x["Code"] == code && x["Status"] == "ACTIVE")
        });
        if !ok {
            return (
                400,
                validation(&format!("Account code '{code}' is not a valid code")),
            );
        }
    }
    let date = j["Date"].as_str().unwrap_or("").to_string();
    if let Some(lock) = &s.lock_date
        && date.as_str() <= lock.as_str()
    {
        return (
            400,
            validation("The document date cannot be before the period lock date"),
        );
    }
    let id = s.new_id("mj");
    s.created(&id);
    // stored as real Xero returns it: a /Date(ms+0000)/ date, totals, and lines with AccountID, TaxType …
    let ms = ownpurse_core::dates::parse_iso(&date)
        .map(|(y, m, d)| ownpurse_core::dates::days_from_civil(y, m, d) * 86_400_000)
        .unwrap_or(0);
    let mut debit = Decimal::ZERO;
    let lines: Vec<Value> = lines
        .iter()
        .map(|l| {
            let amt = dec(&l["LineAmount"]);
            if amt > Decimal::ZERO {
                debit += amt;
            }
            let mut m = l.as_object().cloned().unwrap_or_default();
            let code = l["AccountCode"].as_str().unwrap_or("");
            m.insert("AccountID".into(), json!(s.account_id(tenant, code)));
            m.insert("IsBlank".into(), json!(false));
            m.insert(
                "TaxAmount".into(),
                Value::Number(serde_json::Number::from_string_unchecked("0.00".into())),
            );
            m.insert("TaxType".into(), json!("NONE"));
            m.insert("Tracking".into(), json!([]));
            Value::Object(m)
        })
        .collect();
    let num =
        |d: Decimal| Value::Number(serde_json::Number::from_string_unchecked(format!("{d:.4}")));
    let mut out: Map<String, Value> = j.as_object().cloned().unwrap_or_default();
    out.insert("Date".into(), json!(format!("/Date({ms}+0000)/")));
    out.insert("JournalLines".into(), Value::Array(lines));
    out.insert("DebitTotal".into(), num(debit));
    out.insert("CreditTotal".into(), num(-debit));
    out.insert("ManualJournalID".into(), json!(id));
    out.insert(
        "Status".into(),
        json!(j["Status"].as_str().unwrap_or("DRAFT")),
    );
    out.insert("UpdatedDateUTC".into(), json!("/Date(1791100000000+0000)/"));
    out.insert("HasAttachments".into(), json!(false));
    let out = Value::Object(out);
    s.journals
        .entry(tenant.into())
        .or_default()
        .insert(id, out.clone());
    (200, json!({"ManualJournals": [out]}))
}

fn is_guid(s: &str) -> bool {
    let b = s.as_bytes();
    b.len() == 36
        && b.iter().enumerate().all(|(i, c)| match i {
            8 | 13 | 18 | 23 => *c == b'-',
            _ => c.is_ascii_hexdigit(),
        })
}
