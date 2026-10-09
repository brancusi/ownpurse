//! Token store behaviour, one process: freshness, rotation, failures leave the file untouched, unknown fields
//! kept, mode 0600, no temp files left, the lock file name every version uses.

#![allow(clippy::float_arithmetic)] // token timestamps are float seconds, as stored

use ownpurse_xero::TokenStore;
use ownpurse_xero::form::FormPost;
use serde_json::{Value, json};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

type Calls = Arc<Mutex<Vec<(String, Vec<(String, String)>)>>>;
type Reply = Arc<Mutex<Option<Result<(u16, String), String>>>>;

const NOW: f64 = 1_800_000_000.0;
const URL: &str = "http://127.0.0.1:9/connect/token";

/// A scripted token endpoint: records every POST and answers with the next reply.
#[derive(Clone, Default)]
struct Fake {
    calls: Calls,
    reply: Reply,
}

impl Fake {
    fn answering(r: Result<(u16, String), String>) -> Fake {
        let f = Fake::default();
        *f.reply.lock().unwrap() = Some(r);
        f
    }
    fn calls(&self) -> usize {
        self.calls.lock().unwrap().len()
    }
}

impl FormPost for Fake {
    fn post_form(&self, url: &str, form: &[(&str, &str)]) -> Result<(u16, String), String> {
        self.calls.lock().unwrap().push((
            url.to_string(),
            form.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        ));
        self.reply.lock().unwrap().clone().expect("unexpected POST")
    }
}

fn dir(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("ownpurse-tok-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn store(path: &PathBuf, fake: &Fake) -> TokenStore {
    TokenStore::new(path, "CLIENT")
        .with_token_url(URL)
        .with_http(Box::new(fake.clone()))
        .with_clock(Box::new(|| NOW))
}

fn write(path: &PathBuf, v: Value) {
    std::fs::write(path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn stale() -> Value {
    json!({"access_token": "old-at", "refresh_token": "rt-1", "expires_in": 1800,
           "obtained_at": NOW - 4000.0, "refreshed_at": NOW - 4000.0, "scope": "openid accounting.settings.read",
           "tenants": [{"tenantId": "t-1"}], "custom_field": "kept"})
}

#[test]
fn fresh_token_is_returned_without_a_request() {
    let d = dir("fresh");
    let p = d.join("tokens.json");
    let mut t = stale();
    t["obtained_at"] = json!(NOW - 60.0);
    write(&p, t);
    let fake = Fake::default();
    assert_eq!(store(&p, &fake).access_token().unwrap(), "old-at");
    assert_eq!(fake.calls(), 0);
}

#[test]
fn early_refresh_window_is_120_seconds() {
    let d = dir("early");
    let p = d.join("tokens.json");
    let mut t = stale();
    t["obtained_at"] = json!(NOW - 1800.0 + 119.0); // expires in 119 s: inside the early window
    write(&p, t);
    let fake = Fake::answering(Ok((
        200,
        r#"{"access_token":"new-at","refresh_token":"rt-2","expires_in":1800}"#.into(),
    )));
    assert_eq!(store(&p, &fake).access_token().unwrap(), "new-at");
    assert_eq!(fake.calls(), 1);
}

#[test]
fn stale_token_rotates_and_persists_before_use() {
    let d = dir("rotate");
    let p = d.join("tokens.json");
    write(&p, stale());
    let fake = Fake::answering(Ok((
        200,
        r#"{"access_token":"new-at","refresh_token":"rt-2","expires_in":1800,"token_type":"Bearer"}"#.into(),
    )));
    let s = store(&p, &fake);
    assert_eq!(s.access_token().unwrap(), "new-at");
    let calls = fake.calls.lock().unwrap().clone();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, URL, "the only POST goes to the token URL");
    assert_eq!(
        calls[0].1,
        vec![
            ("grant_type".into(), "refresh_token".into()),
            ("refresh_token".into(), "rt-1".into()),
            ("client_id".into(), "CLIENT".into())
        ]
    );
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    assert_eq!(saved["refresh_token"], "rt-2");
    assert_eq!(saved["access_token"], "new-at");
    assert_eq!(saved["obtained_at"].as_f64(), Some(NOW));
    assert_eq!(saved["refreshed_at"].as_f64(), Some(NOW));
    assert_eq!(
        saved["tenants"],
        json!([{"tenantId": "t-1"}]),
        "tenants carried over"
    );
    assert_eq!(
        saved["scope"], "openid accounting.settings.read",
        "scope kept when Xero omits it"
    );
    assert_eq!(
        saved["custom_field"], "kept",
        "unknown fields survive a refresh"
    );
    assert_eq!(
        std::fs::metadata(&p).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let leftovers: Vec<_> = std::fs::read_dir(&d)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.starts_with(".tokens-"))
        .collect();
    assert!(leftovers.is_empty(), "temp files left: {leftovers:?}");
    assert!(d.join("tokens.json.lock").exists(), "the lock file name");
}

#[test]
fn rejected_refresh_needs_login_and_keeps_the_file() {
    for status in [400u16, 401] {
        let d = dir(&format!("rej{status}"));
        let p = d.join("tokens.json");
        write(&p, stale());
        let before = std::fs::read(&p).unwrap();
        let fake = Fake::answering(Ok((status, r#"{"error":"invalid_grant"}"#.into())));
        let e = store(&p, &fake).access_token().unwrap_err();
        assert_eq!(e.code(), "needs_login", "{e:?}");
        assert_eq!(e.code(), "needs_login");
        assert_eq!(std::fs::read(&p).unwrap(), before, "file untouched");
    }
}

#[test]
fn server_and_network_failures_keep_the_file() {
    for reply in [
        Ok((503u16, "busy".to_string())),
        Err("connection refused".to_string()),
        Ok((200, "not json".into())),
    ] {
        let d = dir("fail");
        let p = d.join("tokens.json");
        write(&p, stale());
        let before = std::fs::read(&p).unwrap();
        let e = store(&p, &Fake::answering(reply.clone()))
            .access_token()
            .unwrap_err();
        assert_eq!(e.code(), "api_error", "{reply:?}");
        assert!(e.hint().contains("not changed"));
        assert_eq!(std::fs::read(&p).unwrap(), before);
    }
}

#[test]
fn missing_and_damaged_files() {
    let d = dir("missing");
    let p = d.join("tokens.json");
    let e = store(&p, &Fake::default()).access_token().unwrap_err();
    assert_eq!(e.code(), "needs_login");
    assert!(!p.exists(), "never creates a token file on read");

    std::fs::write(&p, "{ not json").unwrap();
    let e = store(&p, &Fake::default()).access_token().unwrap_err();
    assert_eq!(e.code(), "needs_login");
    assert!(e.message().contains("unreadable"));
    assert_eq!(
        std::fs::read_to_string(&p).unwrap(),
        "{ not json",
        "damaged file left in place"
    );

    write(&p, json!({"access_token": "x", "refresh_token": ""}));
    assert_eq!(
        store(&p, &Fake::default())
            .access_token()
            .unwrap_err()
            .code(),
        "needs_login"
    );
}

#[test]
fn mark_stale_only_when_the_token_is_still_ours() {
    let d = dir("stale");
    let p = d.join("tokens.json");
    let mut t = stale();
    t["obtained_at"] = json!(NOW);
    write(&p, t);
    let s = store(&p, &Fake::default());
    s.mark_stale("someone-elses-token").unwrap();
    assert!(s.is_fresh(&s.load().unwrap().unwrap()));
    s.mark_stale("old-at").unwrap();
    let after = s.load().unwrap().unwrap();
    assert!(!s.is_fresh(&after));
    assert_eq!(after["custom_field"], "kept");
}

#[test]
fn set_tenants_keeps_everything_else() {
    let d = dir("tenants");
    let p = d.join("tokens.json");
    write(&p, stale());
    let s = store(&p, &Fake::default());
    s.set_tenants(json!([{"tenantId": "t-2"}])).unwrap();
    let t = s.load().unwrap().unwrap();
    assert_eq!(t["tenants"], json!([{"tenantId": "t-2"}]));
    assert_eq!(t["refresh_token"], "rt-1");
    assert_eq!(t["custom_field"], "kept");
}

#[test]
fn status_and_granted_scopes_never_show_secrets() {
    let d = dir("status");
    let p = d.join("tokens.json");
    write(&p, stale());
    let s = store(&p, &Fake::default());
    let st = Value::Object(s.status().unwrap());
    let text = st.to_string();
    assert!(!text.contains("old-at") && !text.contains("rt-1"), "{text}");
    assert_eq!(st["logged_in"], true);
    assert_eq!(st["access_token_fresh"], false);
    assert_eq!(st["warn"], false);
    assert_eq!(
        s.granted_scopes().unwrap(),
        vec!["openid".to_string(), "accounting.settings.read".to_string()]
    );
    let none = store(&d.join("absent.json"), &Fake::default());
    assert_eq!(
        Value::Object(none.status().unwrap()),
        json!({"logged_in": false})
    );
}

#[test]
fn save_creates_a_private_folder() {
    let d = dir("folder");
    let p = d.join("nested/tokens.json");
    let s = store(&p, &Fake::default());
    s.save(&serde_json::from_value(stale()).unwrap()).unwrap();
    let mode = |x: &std::path::Path| std::fs::metadata(x).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode(&d.join("nested")), 0o700);
    assert_eq!(mode(&p), 0o600);
}
