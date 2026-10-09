//! XeroHttp against a loopback fake Xero, through the real HTTP stack (ureq): retries, rate limits, the 401
//! refresh, the call log (row-for-row equal to the frozen reference for the same script), and correction B: across every
//! scenario the only non-GET request any part of the API layer sends is the token endpoint's POST.

#![allow(clippy::float_arithmetic)]

mod common;

use common::temp_dir;
use ownpurse_xero::TokenStore;
use ownpurse_xero::http::{HttpSettings, UreqGet, XeroHttp, py_dumps_sorted, short};
use ownpurse_xero::store::Writer;
use serde_json::{Value, json};
use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

const TENANT: &str = "c0ffee00-0000-4000-8000-0000000000a1";

#[derive(Debug, Clone)]
struct Seen {
    method: String,
    path: String,
    headers: Vec<(String, String)>,
}

#[derive(Clone)]
struct Reply {
    status: u16,
    headers: Vec<(&'static str, String)>,
    body: String,
}

fn reply(status: u16, body: &str) -> Reply {
    Reply {
        status,
        headers: vec![],
        body: body.into(),
    }
}

impl Reply {
    fn h(mut self, k: &'static str, v: &str) -> Reply {
        self.headers.push((k, v.into()));
        self
    }
}

/// Loopback fake: POST /connect/token rotates tokens; every other request takes the next scripted reply.
struct Fake {
    base: String,
    seen: Arc<Mutex<Vec<Seen>>>,
}

fn fake(script: Vec<Reply>) -> Fake {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let seen = Arc::new(Mutex::new(Vec::new()));
    let queue = Arc::new(Mutex::new(VecDeque::from(script)));
    let issued = Arc::new(Mutex::new(0u32));
    let (s2, q2) = (seen.clone(), queue.clone());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut first = String::new();
            if reader.read_line(&mut first).unwrap_or(0) == 0 {
                continue;
            }
            let mut parts = first.split_whitespace();
            let method = parts.next().unwrap_or("").to_string();
            let target = parts.next().unwrap_or("").to_string();
            let mut headers = Vec::new();
            let mut len = 0usize;
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).unwrap_or(0) == 0 {
                    break;
                }
                let l = line.trim_end().to_string();
                if l.is_empty() {
                    break;
                }
                if let Some((k, v)) = l.split_once(':') {
                    let k = k.trim().to_ascii_lowercase();
                    if k == "content-length" {
                        len = v.trim().parse().unwrap_or(0);
                    }
                    headers.push((k, v.trim().to_string()));
                }
            }
            let mut body = vec![0u8; len];
            let _ = reader.read_exact(&mut body);
            let path = target.split('?').next().unwrap_or("").to_string();
            s2.lock().unwrap().push(Seen {
                method: method.clone(),
                path: target.clone(),
                headers,
            });
            let r = if method == "POST" && path == "/connect/token" {
                let mut n = issued.lock().unwrap();
                *n += 1;
                reply(
                    200,
                    &json!({"access_token": format!("at-{n}"), "refresh_token": format!("rt-{n}"), "expires_in": 1800})
                        .to_string(),
                )
            } else {
                q2.lock()
                    .unwrap()
                    .pop_front()
                    .unwrap_or_else(|| reply(599, "script exhausted"))
            };
            let mut head = format!(
                "HTTP/1.1 {} X\r\nContent-Length: {}\r\nConnection: close\r\n",
                r.status,
                r.body.len()
            );
            for (k, v) in &r.headers {
                head.push_str(&format!("{k}: {v}\r\n"));
            }
            let _ = stream.write_all(format!("{head}\r\n{}", r.body).as_bytes());
        }
    });
    Fake { base, seen }
}

struct Env {
    dir: PathBuf,
    fake: Fake,
    store: Writer,
    tokens: TokenStore,
    sleeps: Arc<Mutex<Vec<f64>>>,
}

fn fresh_token(path: &Path, obtained: f64) {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64();
    let t = json!({"access_token": "at-0", "refresh_token": "rt-0", "expires_in": 1800,
        "obtained_at": now - obtained, "refreshed_at": now - obtained, "scope": "openid", "tenants": []});
    std::fs::write(path, t.to_string()).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn env(name: &str, script: Vec<Reply>) -> Env {
    let dir = temp_dir(&format!("http-{name}"));
    let fake = fake(script);
    fresh_token(&dir.join("tokens.json"), 0.0);
    let store = Writer::open(dir.join("record.sqlite")).unwrap();
    let tokens = TokenStore::new(dir.join("tokens.json"), "CLIENT")
        .with_token_url(format!("{}/connect/token", fake.base));
    Env {
        dir,
        fake,
        store,
        tokens,
        sleeps: Arc::new(Mutex::new(Vec::new())),
    }
}

impl Env {
    fn http(&self) -> XeroHttp<'_> {
        let sleeps = self.sleeps.clone();
        XeroHttp::new(
            HttpSettings {
                profile: "demo".into(),
                api_base: format!("{}/api.xro/2.0", self.fake.base),
                connections_url: format!("{}/connections", self.fake.base),
                minute_soft_limit: 55,
                forbid_tenants: vec!["b01d0000-0000-4000-8000-0000000000b2".into()],
            },
            &self.store,
            &self.tokens,
            Box::new(UreqGet::default()),
        )
        .with_sleep(Box::new(move |s, _| sleeps.lock().unwrap().push(s)))
    }
    fn log(&self) -> Vec<Value> {
        let mut st = self
            .store
            .conn()
            .prepare("SELECT tenant_id, method, path, params, status, day_remaining, min_remaining, problem, op_id FROM http_log ORDER BY id")
            .unwrap();
        st.query_map([], |r| {
            Ok(json!([
                r.get::<_, Option<String>>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<String>>(3)?,
                r.get::<_, Option<i64>>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, Option<i64>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, Option<String>>(8)?
            ]))
        })
        .unwrap()
        .map(Result::unwrap)
        .collect()
    }
    /// Correction B, checked on the wire.
    fn only_token_url_gets_non_get(&self) {
        for s in self.fake.seen.lock().unwrap().iter() {
            if s.method != "GET" {
                assert_eq!(
                    (s.method.as_str(), s.path.as_str()),
                    ("POST", "/connect/token"),
                    "{s:?}"
                );
            }
        }
    }
    fn posts(&self) -> usize {
        self.fake
            .seen
            .lock()
            .unwrap()
            .iter()
            .filter(|s| s.method != "GET")
            .count()
    }
}

fn params() -> Vec<(String, Value)> {
    vec![
        ("where".into(), json!("Status==\"ACTIVE\"")),
        ("page".into(), json!(1)),
        ("pageSize".into(), json!(1000)),
    ]
}

#[test]
fn success_keeps_exact_digits_and_logs_the_call() {
    let e = env(
        "ok",
        vec![
            reply(
                200,
                r#"{"Accounts":[{"AccountID":"a-1","Balance":107.39}]}"#,
            )
            .h("X-DayLimit-Remaining", "990")
            .h("X-MinLimit-Remaining", "58"),
        ],
    );
    let v = e.http().get(TENANT, "Accounts", &params(), None).unwrap();
    assert_eq!(
        v["Accounts"][0]["Balance"].to_string(),
        "107.39",
        "money keeps Xero's digits"
    );
    let seen = e.fake.seen.lock().unwrap()[0].clone();
    assert!(seen.path.starts_with("/api.xro/2.0/Accounts?"));
    let hv = |k: &str| {
        seen.headers
            .iter()
            .find(|(n, _)| n == k)
            .map(|(_, v)| v.clone())
    };
    assert_eq!(hv("authorization").as_deref(), Some("Bearer at-0"));
    assert_eq!(hv("xero-tenant-id").as_deref(), Some(TENANT));
    assert_eq!(hv("accept").as_deref(), Some("application/json"));
    assert_eq!(
        e.log(),
        vec![json!([
            TENANT,
            "GET",
            "Accounts",
            r#"{"page": 1, "pageSize": 1000, "where": "Status==\"ACTIVE\""}"#,
            200,
            990,
            58,
            null,
            null
        ])]
    );
    assert_eq!(e.posts(), 0);
}

#[test]
fn unauthorised_once_refreshes_once_and_retries() {
    let e = env(
        "401",
        vec![reply(401, "{}"), reply(200, r#"{"Accounts":[]}"#)],
    );
    e.http().get(TENANT, "Accounts", &[], None).unwrap();
    assert_eq!(e.posts(), 1, "exactly one refresh");
    let seen = e.fake.seen.lock().unwrap().clone();
    let auth: Vec<String> = seen
        .iter()
        .filter(|s| s.method == "GET")
        .map(|s| {
            s.headers
                .iter()
                .find(|(k, _)| k == "authorization")
                .unwrap()
                .1
                .clone()
        })
        .collect();
    assert_eq!(auth, vec!["Bearer at-0", "Bearer at-1"]);
    e.only_token_url_gets_non_get();
    let saved: Value =
        serde_json::from_str(&std::fs::read_to_string(e.dir.join("tokens.json")).unwrap()).unwrap();
    assert_eq!(saved["refresh_token"], "rt-1");
}

#[test]
fn unauthorised_twice_needs_login() {
    let e = env("401x2", vec![reply(401, "{}"), reply(401, "{}")]);
    let err = e.http().get(TENANT, "Accounts", &[], None).unwrap_err();
    assert_eq!(err.code(), "needs_login");
    assert_eq!(e.posts(), 1);
    e.only_token_url_gets_non_get();
}

#[test]
fn minute_limit_waits_retry_after_plus_one() {
    let e = env(
        "429m",
        vec![
            reply(429, "{}")
                .h("Retry-After", "2")
                .h("X-Rate-Limit-Problem", "minute"),
            reply(200, r#"{"Accounts":[]}"#),
        ],
    );
    e.http().get(TENANT, "Accounts", &[], None).unwrap();
    assert_eq!(*e.sleeps.lock().unwrap(), vec![3.0]);
    assert_eq!(e.log()[0][7], "minute");
}

#[test]
fn daily_limit_fails_at_once_without_sleeping() {
    let e = env(
        "429d",
        vec![reply(429, "{}").h("X-Rate-Limit-Problem", "day")],
    );
    let err = e.http().get(TENANT, "Accounts", &[], None).unwrap_err();
    assert_eq!(err.code(), "daily_limit");
    assert_eq!(
        err.hints,
        [
            "Use cached data (`--offline`).",
            "Xero resets each org's daily limit at its own time; `ownpurse usage` shows what's left."
        ]
    );
    assert!(e.sleeps.lock().unwrap().is_empty());
}

#[test]
fn server_errors_back_off_then_succeed() {
    let e = env(
        "5xx",
        vec![reply(503, "busy"), reply(502, "busy"), reply(200, "{}")],
    );
    e.http().get(TENANT, "Accounts", &[], None).unwrap();
    assert_eq!(*e.sleeps.lock().unwrap(), vec![2.0, 4.0]);
    assert_eq!(e.log().len(), 3, "every attempt is logged");
}

#[test]
fn client_errors_map_to_cli_codes() {
    for (status, code) in [
        (403u16, "forbidden"),
        (404, "not_found"),
        (400, "api_error"),
    ] {
        let e = env(
            &format!("c{status}"),
            vec![reply(status, r#"{"Message":"nope"}"#)],
        );
        let err = e.http().get(TENANT, "Accounts", &[], None).unwrap_err();
        assert_eq!(err.code(), code);
        if status != 400 || err.body.is_some() {
            assert!(err.body.as_deref().unwrap_or("").contains("nope"));
        }
    }
}

#[test]
fn not_modified_is_an_empty_object() {
    let e = env("304", vec![reply(304, "")]);
    let v = e
        .http()
        .get(
            TENANT,
            "Accounts",
            &[],
            Some("Mon, 05 Oct 2026 09:00:00 GMT"),
        )
        .unwrap();
    assert_eq!(v, json!({}));
    let seen = e.fake.seen.lock().unwrap()[0].clone();
    assert!(
        seen.headers
            .iter()
            .any(|(k, v)| k == "if-modified-since" && v.contains("2026"))
    );
}

#[test]
fn forbidden_tenant_never_reaches_the_network() {
    let e = env("forbid", vec![]);
    let err = e
        .http()
        .get(
            "b01d0000-0000-4000-8000-0000000000b2",
            "Accounts",
            &[],
            None,
        )
        .unwrap_err();
    assert_eq!(err.code(), "usage");
    assert!(e.fake.seen.lock().unwrap().is_empty());
    assert!(e.log().is_empty());
}

#[test]
fn connections_are_stored_in_the_token_file() {
    let e = env(
        "conn",
        vec![reply(200, &json!([{"tenantId": TENANT, "tenantName": "Tallowmere Coffee Roasting Co.", "tenantType": "ORGANISATION", "id": "x"}]).to_string())],
    );
    let t = e.http().connections().unwrap();
    assert_eq!(t[0]["tenantName"], "Tallowmere Coffee Roasting Co.");
    assert!(
        t[0].get("id").is_none(),
        "only the three fields the record keeps"
    );
    let saved: Value =
        serde_json::from_str(&std::fs::read_to_string(e.dir.join("tokens.json")).unwrap()).unwrap();
    assert_eq!(saved["tenants"][0]["tenantId"], TENANT);
    assert_eq!(e.log()[0][2], "connections");
    assert_eq!(
        e.log()[0][0],
        Value::Null,
        "connections are logged without a tenant"
    );
}

#[test]
fn helpers_shorten_paths_and_dump_sorted_ascii_json() {
    assert_eq!(
        short("https://api.xero.com/api.xro/2.0/Accounts"),
        "Accounts"
    );
    assert_eq!(short("https://api.xero.com/connections"), "connections");
    assert_eq!(
        py_dumps_sorted(&[("b".into(), json!("é")), ("a".into(), json!(2))]),
        "{\"a\": 2, \"b\": \"\\u00e9\"}"
    );
}

/// A fixed retry script gives exactly the frozen reference call log (http_log rows, except time columns).
#[test]
fn call_log_equals_the_frozen_reference_for_the_same_script() {
    let script = || {
        vec![
            reply(401, "{}"),
            reply(503, "busy"),
            reply(429, "{}")
                .h("Retry-After", "1")
                .h("X-Rate-Limit-Problem", "minute"),
            reply(200, r#"{"Accounts":[{"AccountID":"a-1"}]}"#)
                .h("X-DayLimit-Remaining", "987")
                .h("X-MinLimit-Remaining", "57"),
        ]
    };
    let rust = env("parity-rs", script());
    rust.http()
        .get(TENANT, "Accounts", &params(), None)
        .unwrap();

    let ref_json = common::frozen_text("http-call-log-ref.json");
    let reference: Vec<Value> = serde_json::from_str(&ref_json).unwrap();
    assert_eq!(reference.len(), 4, "{reference:?}");
    assert_eq!(rust.log(), reference);
    rust.only_token_url_gets_non_get();
}
