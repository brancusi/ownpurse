//! Sync goldens: a fake Xero synced in two rounds (a full sync, then an incremental one with one edit) gives
//! exactly the frozen reference versions, sync runs and call log; a sync on top of the frozen reference
//! record records nothing new (content hashes agree) and the mixed record's chain verifies.

#![allow(clippy::float_arithmetic)]

mod common;

use common::temp_dir;
use ownpurse_xero::TokenStore;
use ownpurse_xero::http::{HttpSettings, UreqGet, XeroHttp};
use ownpurse_xero::store::Writer;
use ownpurse_xero::sync::{Syncer, entity, xero_datetime};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};

const TENANT: &str = "c0ffee00-0000-4000-8000-0000000000a1";
const NAMES: [&str; 6] = [
    "Organisation",
    "Accounts",
    "Currencies",
    "BankTransactions",
    "ManualJournals",
    "Invoices",
];

fn date(ms: i64) -> String {
    format!("/Date({ms}+0000)/")
}

fn bt(id: &str, ms: i64, total: &str) -> Value {
    serde_json::from_str(&format!(
        r#"{{"BankTransactionID":"{id}","Type":"SPEND","Status":"AUTHORISED","Total":{total},"UpdatedDateUTC":"{}",
            "Contact":{{"Name":"Café ünï"}},"LineItems":[{{"LineAmount":{total},"AccountCode":"400"}}]}}"#,
        date(ms)
    ))
    .unwrap()
}

/// The fake's answer for a GET (path without the API prefix, query pairs), by round.
fn answer(round: u8, path: &str, q: &[(String, String)]) -> Value {
    let get = |k: &str| q.iter().find(|(n, _)| n == k).map(|(_, v)| v.as_str());
    let page: usize = get("page").and_then(|p| p.parse().ok()).unwrap_or(1);
    match path {
        "Organisation" => json!({"Organisations": [{"OrganisationID": "o-1", "Name": "Tallowmere Coffee Roasting Co.",
            "OrganisationStatus": "ACTIVE", "FinancialYearEndMonth": 12}]}),
        "Accounts" => serde_json::from_str(
            r#"{"Accounts":[{"AccountID":"a-1","Code":"090","Name":"Tallowmere Operating","Status":"ACTIVE","Type":"BANK"},
                            {"AccountID":"a-2","Code":"400","Name":"Advertising","Status":"ACTIVE","Type":"EXPENSE","TaxRate":0.0}]}"#,
        )
        .unwrap(),
        "Currencies" => json!({"Currencies": [{"Code": "USD", "Description": "United States Dollar"}]}),
        "BankTransactions" => {
            let mut all = vec![
                bt("b-1", 1_735_689_600_000, "107.39"),
                bt("b-2", 1_735_776_000_000, "20.5"),
                bt("b-3", 1_735_862_400_000, "1000000.01"),
            ];
            if round == 2 {
                all = vec![bt("b-2", 1_740_000_000_000, "21.50")]; // incremental: one edit
            }
            let chunk: Vec<Value> = all.iter().skip((page - 1) * 2).take(2).cloned().collect();
            let pages = all.len().div_ceil(2).max(1);
            json!({"pagination": {"page": page, "pageSize": 2, "pageCount": pages, "itemCount": all.len()},
                   "BankTransactions": chunk})
        }
        "ManualJournals" => {
            // no pagination block: paging stops on a short page
            let all = [json!({"ManualJournalID": "m-1", "Status": "POSTED", "Narration": "Opening",
                    "UpdatedDateUTC": date(1_735_689_600_000),
                    "JournalLines": [{"LineAmount": 5071.83, "AccountCode": "300"}, {"LineAmount": -5071.83, "AccountCode": "090"}]}),
                json!({"ManualJournalID": "m-2", "Status": "VOIDED", "Narration": "Mistake",
                    "UpdatedDateUTC": date(1_735_700_000_000), "JournalLines": []})];
            let chunk: Vec<Value> = if round == 1 { all.iter().skip((page - 1) * 2).take(2).cloned().collect() } else { vec![] };
            json!({"ManualJournals": chunk})
        }
        "Invoices" => {
            let deleted = get("Statuses") == Some("DELETED,VOIDED");
            let items = match (round, deleted, page) {
                (_, true, 1) => vec![json!({"InvoiceID": "i-9", "Status": "VOIDED", "Total": 99.99,
                    "UpdatedDateUTC": date(1_735_000_000_000)})],
                (1, false, 1) => vec![json!({"InvoiceID": "i-1", "Status": "AUTHORISED", "Total": 1200,
                    "UpdatedDateUTC": date(1_735_689_600_000)})],
                _ => vec![],
            };
            json!({"pagination": {"page": page, "pageSize": 2, "pageCount": 1, "itemCount": items.len()}, "Invoices": items})
        }
        _ => json!({}),
    }
}

/// Loopback fake Xero: token endpoint + API, answering by the shared `round`.
fn serve(round: Arc<Mutex<u8>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut first = String::new();
            if reader.read_line(&mut first).unwrap_or(0) == 0 {
                continue;
            }
            let mut len = 0usize;
            loop {
                let mut l = String::new();
                if reader.read_line(&mut l).unwrap_or(0) == 0 || l.trim().is_empty() {
                    break;
                }
                if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0u8; len];
            let _ = reader.read_exact(&mut body);
            let mut parts = first.split_whitespace();
            let (method, target) = (parts.next().unwrap_or(""), parts.next().unwrap_or(""));
            let (path, query) = target.split_once('?').unwrap_or((target, ""));
            let q: Vec<(String, String)> = query
                .split('&')
                .filter(|s| !s.is_empty())
                .map(|kv| {
                    let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
                    let dec = |s: &str| {
                        let s = s.replace('+', " ");
                        let mut out = Vec::new();
                        let b = s.as_bytes();
                        let mut i = 0;
                        while i < b.len() {
                            if b[i] == b'%' && i + 2 < b.len() {
                                out.push(u8::from_str_radix(&s[i + 1..i + 3], 16).unwrap());
                                i += 3;
                            } else {
                                out.push(b[i]);
                                i += 1;
                            }
                        }
                        String::from_utf8(out).unwrap()
                    };
                    (dec(k), dec(v))
                })
                .collect();
            let reply = if method == "POST" {
                json!({"access_token": "at-x", "refresh_token": "rt-x", "expires_in": 1800})
                    .to_string()
            } else {
                let r = *round.lock().unwrap();
                answer(r, path.trim_start_matches("/api.xro/2.0/"), &q).to_string()
            };
            let _ = stream.write_all(
                format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-DayLimit-Remaining: 900\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
                    reply.len()
                )
                .as_bytes(),
            );
        }
    });
    base
}

fn token(path: &Path) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    std::fs::write(
        path,
        json!({"access_token": "at-0", "refresh_token": "rt-0", "expires_in": 1800, "obtained_at": now,
               "refreshed_at": now, "scope": "openid", "tenants": []})
        .to_string(),
    )
    .unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn rust_sync(dir: &Path, base: &str) -> Vec<(String, String, usize, usize, i64)> {
    let store = Writer::open(dir.join("record.sqlite")).unwrap();
    let tokens = TokenStore::new(dir.join("tokens.json"), "CLIENT")
        .with_token_url(format!("{base}/connect/token"));
    let http = XeroHttp::new(
        HttpSettings {
            profile: "p".into(),
            api_base: format!("{base}/api.xro/2.0"),
            connections_url: format!("{base}/connections"),
            minute_soft_limit: 1000,
            forbid_tenants: vec![],
        },
        &store,
        &tokens,
        Box::new(UreqGet::default()),
    )
    .with_sleep(Box::new(|_, _| {}));
    let mut sy = Syncer::new(&store, &http);
    sy.page_size = 2;
    NAMES
        .iter()
        .map(|n| {
            let r = sy.sync(TENANT, &entity(n).unwrap(), false).unwrap();
            (r.entity, r.mode, r.fetched, r.new_versions, r.calls)
        })
        .collect()
}

fn dump(dir: &Path) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    ownpurse_core::record::sqlite_ready(); // first SQLite init on one thread
    let c = rusqlite::Connection::open(dir.join("record.sqlite")).unwrap();
    let rows = |sql: &str, n: usize| -> Vec<Value> {
        let mut st = c.prepare(sql).unwrap();
        st.query_map([], |r| {
            let mut v = Vec::new();
            for i in 0..n {
                v.push(match r.get::<_, rusqlite::types::Value>(i)? {
                    rusqlite::types::Value::Null => Value::Null,
                    rusqlite::types::Value::Integer(x) => json!(x),
                    rusqlite::types::Value::Text(s) => json!(s),
                    other => json!(format!("{other:?}")),
                });
            }
            Ok(Value::Array(v))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
    };
    (
        rows(
            "SELECT tenant_id, entity, xero_id, version, content_hash, status, xero_updated_utc, source, payload FROM versions ORDER BY id",
            9,
        ),
        rows(
            "SELECT tenant_id, entity, mode, outcome, calls, fetched, new_versions, high_water, detail FROM sync_runs ORDER BY rowid",
            9,
        ),
        rows(
            "SELECT tenant_id, method, path, params, status, day_remaining FROM http_log ORDER BY id",
            6,
        ),
    )
}

#[test]
fn sync_equals_the_frozen_reference_and_agrees_on_hashes() {
    let round = Arc::new(Mutex::new(1u8));
    let base = serve(round.clone());

    // the reference: two rounds' results, dumps and record (frozen)
    let pd = temp_dir("sync-ref");
    token(&pd.join("tokens.json"));
    let fz: Value = serde_json::from_str(&common::frozen_text("sync-ref.json")).unwrap();
    let p1: Vec<Value> = serde_json::from_value(fz["p1"].clone()).unwrap();
    let p2: Vec<Value> = serde_json::from_value(fz["p2"].clone()).unwrap();
    common::frozen_record("sync-ref-record.sqlite", &pd.join("record.sqlite"));

    // the syncer: same rounds into its own record
    let rd = temp_dir("sync-rs");
    token(&rd.join("tokens.json"));
    *round.lock().unwrap() = 1;
    let r1 = rust_sync(&rd, &base);
    *round.lock().unwrap() = 2;
    let r2 = rust_sync(&rd, &base);

    let summary = |v: &[Value]| -> Vec<(String, String, usize, usize, i64)> {
        v.iter()
            .map(|x| {
                (
                    x["entity"].as_str().unwrap().into(),
                    x["mode"].as_str().unwrap().into(),
                    x["fetched"].as_u64().unwrap() as usize,
                    x["new_versions"].as_u64().unwrap() as usize,
                    x["calls"].as_i64().unwrap(),
                )
            })
            .collect()
    };
    assert_eq!(r1, summary(&p1), "round 1 results");
    assert_eq!(r2, summary(&p2), "round 2 results");
    assert!(
        r2.iter()
            .any(|x| x.0 == "BankTransactions" && x.1 == "incremental" && x.3 == 1),
        "{r2:?}"
    );

    let arr = |k: &str| fz[k].as_array().unwrap().clone();
    let (pv, ps, pl) = (arr("versions"), arr("runs"), arr("log"));
    let (rv, rs, rl) = dump(&rd);
    assert_eq!(rv.len(), pv.len(), "version count");
    for (a, b) in rv.iter().zip(&pv) {
        assert_eq!(a, b, "versions differ");
    }
    assert_eq!(rs, ps, "sync runs differ");
    assert_eq!(rl, pl, "call logs differ");
    assert!(
        pv.iter()
            .any(|v| v[8].as_str().unwrap().contains("\"Total\":107.39")),
        "exact digits stored"
    );

    // a full sync on top of the reference record, against the round-2 data: hashes agree, nothing new for
    // documents that did not change, and the mixed record's chain verifies.
    *round.lock().unwrap() = 2;
    let store = Writer::open(pd.join("record.sqlite")).unwrap();
    let tokens = TokenStore::new(pd.join("tokens.json"), "CLIENT")
        .with_token_url(format!("{base}/connect/token"));
    let http = XeroHttp::new(
        HttpSettings {
            profile: "p".into(),
            api_base: format!("{base}/api.xro/2.0"),
            connections_url: format!("{base}/connections"),
            minute_soft_limit: 1000,
            forbid_tenants: vec![],
        },
        &store,
        &tokens,
        Box::new(UreqGet::default()),
    )
    .with_sleep(Box::new(|_, _| {}));
    let mut sy = Syncer::new(&store, &http);
    sy.page_size = 2;
    for n in ["Organisation", "Accounts", "Currencies"] {
        let r = sy.sync(TENANT, &entity(n).unwrap(), true).unwrap();
        assert_eq!(r.new_versions, 0, "{n}: the same payloads hash identically");
    }
    let (entries, _) = store.verify().unwrap();
    assert!(entries > 0);
}

#[test]
fn a_failed_sync_is_recorded() {
    let d = temp_dir("sync-fail");
    token(&d.join("tokens.json"));
    // nothing listens on this port: every attempt is a network error
    let port = TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let base = format!("http://127.0.0.1:{port}");
    let store = Writer::open(d.join("record.sqlite")).unwrap();
    let tokens = TokenStore::new(d.join("tokens.json"), "CLIENT");
    let http = XeroHttp::new(
        HttpSettings {
            profile: "p".into(),
            api_base: format!("{base}/api.xro/2.0"),
            connections_url: format!("{base}/connections"),
            minute_soft_limit: 1000,
            forbid_tenants: vec![],
        },
        &store,
        &tokens,
        Box::new(UreqGet::default()),
    )
    .with_sleep(Box::new(|_, _| {}));
    let err = Syncer::new(&store, &http)
        .sync(TENANT, &entity("accounts").unwrap(), false)
        .unwrap_err();
    assert_eq!(err.code(), "api_error");
    let (outcome, detail): (String, String) = store
        .conn()
        .query_row("SELECT outcome, detail FROM sync_runs", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!(outcome, "error");
    assert!(
        detail.starts_with(r#"{"error": "ApiError", "message": "Network error on GET Accounts"#),
        "{detail}"
    );
    let versions: i64 = store
        .conn()
        .query_row("SELECT COUNT(*) FROM versions", [], |r| r.get(0))
        .unwrap();
    assert_eq!(versions, 0);
}

#[test]
fn xero_dates_normalise_to_utc() {
    for (c, want) in [
        ("/Date(1735689600000+0000)/", "2025-01-01T00:00:00Z"),
        ("/Date(1735689600999)/", "2025-01-01T00:00:00Z"),
        ("/Date(-86400001+0000)/", "1969-12-30T23:59:59Z"),
        ("/Date(+5)/", "/Date(+5)/"),
        ("2025-01-02T03:04:05", "2025-01-02T03:04:05Z"),
        ("2025-01-02T03:04:05Z", "2025-01-02T03:04:05Z"),
        ("2025-01-02T03:04:05.123+02:00", "2025-01-02T01:04:05Z"),
        ("not a date", "not a date"),
    ] {
        assert_eq!(xero_datetime(c).unwrap_or_default(), want, "{c}");
    }
    assert_eq!(xero_datetime(""), None);
}

/// Rewrites the frozen sync reference (`sync-ref.json`, `sync-ref-record.sqlite`) from this syncer and the fake
/// above. Run only when the fake's data changes, and review the diff: a regenerated reference proves stability,
/// not independent agreement (docs/TEST-GAPS.md).
#[test]
#[ignore = "writes rust/tests/fixtures/frozen/sync-ref*"]
fn write_sync_reference() {
    let round = Arc::new(Mutex::new(1u8));
    let base = serve(round.clone());
    let d = temp_dir("sync-ref-write");
    token(&d.join("tokens.json"));
    let as_json = |v: Vec<(String, String, usize, usize, i64)>| -> Vec<Value> {
        v.into_iter()
            .map(|(e, m, f, n, c)| json!({"entity": e, "mode": m, "fetched": f, "new_versions": n, "calls": c}))
            .collect()
    };
    *round.lock().unwrap() = 1;
    let p1 = as_json(rust_sync(&d, &base));
    *round.lock().unwrap() = 2;
    let p2 = as_json(rust_sync(&d, &base));
    let (v, r, l) = dump(&d);
    let doc = json!({"p1": p1, "p2": p2, "versions": v, "runs": r, "log": l});
    std::fs::write(
        common::frozen("sync-ref.json"),
        serde_json::to_string_pretty(&doc).unwrap(),
    )
    .unwrap();
    let c = rusqlite::Connection::open(d.join("record.sqlite")).unwrap();
    c.execute_batch("PRAGMA wal_checkpoint(TRUNCATE); PRAGMA journal_mode=DELETE;")
        .unwrap();
    drop(c);
    std::fs::copy(
        d.join("record.sqlite"),
        common::frozen("sync-ref-record.sqlite"),
    )
    .unwrap();
}
