//! The write transport against the loopback fake Xero, through the real HTTP stack: the allow-list, the
//! profile lock, the tenant guard (a, b, c), the loopback guard (no test reaches the network), the
//! Idempotency-Key header, 429 handling, and the call log.

mod common;

use common::temp_dir;
use ownpurse_core::write::{Body, Method, Send, TransportError, WriteRequest, WriteTransport};
use ownpurse_xero::TokenStore;
use ownpurse_xero::store::Writer;
use ownpurse_xero::write::fake::{FakeXero, Fault, State};
use ownpurse_xero::write::{UreqWire, WriteSettings, XeroWriter};
use serde_json::{Value, json};
use std::path::Path;

const T: &str = "c0ffee00-0000-4000-8000-0000000000a1";
const OTHER: &str = "b01d0000-0000-4000-8000-0000000000b2";

fn token(p: &Path) {
    std::fs::write(
        p,
        json!({"access_token": "AT", "refresh_token": "RT", "obtained_at": 4.0e9, "expires_in": 1800,
               "scope": "accounting.manualjournals accounting.banktransactions accounting.attachments"})
        .to_string(),
    )
    .unwrap();
}

fn fake() -> FakeXero {
    let mut s = State::default();
    s.add_tenant(
        T,
        "Brindlewick Demo Co (US)",
        &[
            ("1010", "Operating", "ASSET"),
            ("6310", "Supplies", "EXPENSE"),
            ("2700", "Owner loan", "LIABILITY"),
        ],
    );
    s.add_tenant(OTHER, "Another Org", &[("1010", "Operating", "ASSET")]);
    FakeXero::start(s).unwrap()
}

struct World {
    _dir: std::path::PathBuf,
    store: Writer,
    tokens: TokenStore,
}

fn world(name: &str) -> World {
    let dir = temp_dir(name);
    token(&dir.join("tokens.json"));
    World {
        store: Writer::open(dir.join("record.sqlite")).unwrap(),
        tokens: TokenStore::new(dir.join("tokens.json"), "CLIENT"),
        _dir: dir,
    }
}

fn settings(f: &FakeXero) -> WriteSettings {
    WriteSettings {
        profile: "demo".into(),
        api_base: f.api_base(),
        connections_url: f.connections_url(),
        orgs: vec![("demo".into(), T.into())],
        forbid_tenants: vec![OTHER.into()],
        require_tenant_name: Some("Brindlewick Demo Co (US)".into()),
        allow_writes: true,
        loopback_only: true,
    }
}

fn journal() -> WriteRequest {
    WriteRequest {
        method: Method::Put,
        path: "ManualJournals".into(),
        body: Body::Json {
            json: json!({"ManualJournals": [{"Narration": "Sample tins · XA:p1:op-01", "Date": "2025-03-31",
                "Status": "POSTED", "LineAmountTypes": "NoTax", "ShowOnCashBasisReports": true,
                "JournalLines": [{"AccountCode": "6310", "Description": "tins", "LineAmount": 147.38},
                                 {"AccountCode": "2700", "Description": "tins", "LineAmount": -147.38}]}]}),
        },
    }
}

fn send<'a>(req: &'a WriteRequest, tenant: &'a str) -> Send<'a> {
    Send {
        tenant_id: tenant,
        request: req,
        path: req.path.clone(),
        idempotency_key: "k-0001",
        file: None,
    }
}

#[test]
fn a_guarded_write_lands_with_its_idempotency_key_and_is_logged() {
    let f = fake();
    let w = world("wt-ok");
    let mut x = XeroWriter::new(
        settings(&f),
        &w.store,
        &w.tokens,
        Box::new(UreqWire::default()),
    );
    let req = journal();
    let r = x.send(&send(&req, T)).unwrap();
    assert_eq!(r.status, 200, "{}", r.body);
    assert_eq!(r.body["ManualJournals"][0]["Status"], "POSTED");
    let log = f.with(|s| s.log.clone());
    let put = log.iter().find(|l| l.0 == "PUT").unwrap();
    assert_eq!(put.2.as_deref(), Some("k-0001"));
    // the tenant name was checked first (connections), once
    assert_eq!(log.iter().filter(|l| l.1 == "/connections").count(), 1);
    let again = x.send(&send(&req, T)).unwrap();
    assert_eq!(
        again.body, r.body,
        "an exact replay under the same key is the first answer"
    );
    assert_eq!(
        f.with(|s| s.log.iter().filter(|l| l.1 == "/connections").count()),
        1
    );
    // the call log has the writes
    let n: i64 = w
        .store
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM http_log WHERE method = 'PUT'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 2);
    w.store.verify().unwrap();
}

#[test]
fn refusals_happen_before_anything_is_sent() {
    let f = fake();
    let w = world("wt-refuse");
    let sent = |f: &FakeXero| f.with(|s| s.log.iter().filter(|l| l.0 != "GET").count());
    // not in the allow-list
    let mut x = XeroWriter::new(
        settings(&f),
        &w.store,
        &w.tokens,
        Box::new(UreqWire::default()),
    );
    let bad = WriteRequest {
        method: Method::Post,
        path: "Invoices/inv-1".into(),
        body: Body::Json { json: Value::Null },
    };
    assert!(matches!(
        x.send(&send(&bad, T)),
        Err(TransportError::NotAllowed(_))
    ));
    // the profile lock
    let mut s = settings(&f);
    s.allow_writes = false;
    let mut x = XeroWriter::new(s, &w.store, &w.tokens, Box::new(UreqWire::default()));
    let req = journal();
    assert!(
        matches!(x.send(&send(&req, T)), Err(TransportError::Refused { code, .. }) if code == "writes_disabled")
    );
    // (a) not one of the profile's orgs
    let mut x = XeroWriter::new(
        settings(&f),
        &w.store,
        &w.tokens,
        Box::new(UreqWire::default()),
    );
    assert!(
        matches!(x.send(&send(&req, "1ab00000-0000-4000-8000-0000000000d4")), Err(TransportError::Refused { code, .. }) if code == "tenant_guard")
    );
    // (b) forbidden, even if listed
    let mut s = settings(&f);
    s.orgs.push(("other".into(), OTHER.into()));
    let mut x = XeroWriter::new(s, &w.store, &w.tokens, Box::new(UreqWire::default()));
    assert!(
        matches!(x.send(&send(&req, OTHER)), Err(TransportError::Refused { code, .. }) if code == "tenant_guard")
    );
    // (c) the connected tenant's name differs
    let mut s = settings(&f);
    s.require_tenant_name = Some("Demo Company (US)".into());
    let mut x = XeroWriter::new(s, &w.store, &w.tokens, Box::new(UreqWire::default()));
    match x.send(&send(&req, T)) {
        Err(TransportError::Refused { code, message }) => {
            assert_eq!(code, "tenant_guard");
            assert!(message.contains("Brindlewick Demo Co (US)"), "{message}");
        }
        other => panic!("{other:?}"),
    }
    assert_eq!(sent(&f), 0, "no write reached the server");
}

#[test]
fn no_test_reaches_the_network() {
    // the real host is refused before a socket is opened (loopback-only, as every test runs)
    let w = world("wt-net");
    let mut s = WriteSettings {
        profile: "demo".into(),
        api_base: "https://api.xero.com/api.xro/2.0".into(),
        connections_url: "https://api.xero.com/connections".into(),
        orgs: vec![("demo".into(), T.into())],
        forbid_tenants: vec![],
        require_tenant_name: None,
        allow_writes: true,
        loopback_only: true,
    };
    let mut x = XeroWriter::new(
        s.clone(),
        &w.store,
        &w.tokens,
        Box::new(UreqWire::default()),
    );
    let req = journal();
    assert!(
        matches!(x.send(&send(&req, T)), Err(TransportError::Refused { code, .. }) if code == "not_loopback")
    );
    assert!(
        matches!(x.get(T, "ManualJournals"), Err(TransportError::Refused { code, .. }) if code == "not_loopback")
    );
    s.require_tenant_name = Some("x".into());
    let mut x = XeroWriter::new(s, &w.store, &w.tokens, Box::new(UreqWire::default()));
    assert!(x.tenant_name(T).is_err());
    let n: i64 = w
        .store
        .conn()
        .query_row("SELECT COUNT(*) FROM http_log", [], |r| r.get(0))
        .unwrap();
    assert_eq!(n, 0, "nothing was attempted");
}

#[test]
fn a_429_waits_and_resends_and_a_day_limit_stops() {
    let f = fake();
    let w = world("wt-429");
    f.with(|s| {
        s.faults
            .push_back(("ManualJournals".into(), Fault::RateLimited))
    });
    let slept = std::sync::Arc::new(std::sync::Mutex::new(vec![]));
    let sl = slept.clone();
    let mut x = XeroWriter::new(
        settings(&f),
        &w.store,
        &w.tokens,
        Box::new(UreqWire::default()),
    )
    .with_sleep(Box::new(move |s| sl.lock().unwrap().push(s.to_string())));
    let req = journal();
    let r = x.send(&send(&req, T)).unwrap();
    assert_eq!(r.status, 200);
    assert_eq!(slept.lock().unwrap().as_slice(), ["2"]);
    f.with(|s| {
        s.faults
            .push_back(("ManualJournals".into(), Fault::DayLimit))
    });
    let mut req2 = journal();
    if let Body::Json { json } = &mut req2.body {
        json["ManualJournals"][0]["Narration"] = json!("other · XA:p1:op-02");
    }
    let mut s2 = send(&req2, T);
    s2.idempotency_key = "k-0002";
    assert!(
        matches!(x.send(&s2), Err(TransportError::Refused { code, .. }) if code == "daily_limit")
    );
}

#[test]
fn an_error_after_sending_is_reported_as_such() {
    let f = fake();
    let w = world("wt-drop");
    f.with(|s| {
        s.faults
            .push_back(("ManualJournals".into(), Fault::DropAfterApply))
    });
    let mut x = XeroWriter::new(
        settings(&f),
        &w.store,
        &w.tokens,
        Box::new(UreqWire::default()),
    );
    let req = journal();
    assert!(matches!(
        x.send(&send(&req, T)),
        Err(TransportError::AfterSend(_))
    ));
    // it did land (the read-back would find it)
    let r = x
        .get(
            T,
            "ManualJournals?where=Narration.Contains(%22XA:p1:op-01%22)",
        )
        .unwrap();
    assert_eq!(r.body["ManualJournals"].as_array().map(Vec::len), Some(1));
}
