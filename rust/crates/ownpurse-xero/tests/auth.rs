//! Login: PKCE values, the consent URL (frozen), correction A on scopes (never a new write scope,
//! never a dropped grant), the loopback receiver, and a whole login against loopback endpoints.

mod common;

use common::temp_dir;
use ownpurse_xero::TokenStore;
use ownpurse_xero::auth::{
    Login, b64url, challenge, consent_url, is_write_scope, login, login_scopes, quote_plus,
    token_urlsafe, wait_for_code, wait_on,
};
use ownpurse_xero::form::FormPost;
use serde_json::{Value, json};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn s(v: &[&str]) -> Vec<String> {
    v.iter().map(|x| x.to_string()).collect()
}

/// A listener on a free port, kept bound (no race with parallel tests), and its port.
fn bound() -> (TcpListener, u16) {
    let l = TcpListener::bind("127.0.0.1:0").unwrap();
    let p = l.local_addr().unwrap().port();
    (l, p)
}

/// A browser: GET the redirect URI with the given query, return the status line.
fn browse(port: u16, target: &str) -> String {
    for _ in 0..50 {
        if let Ok(mut c) = TcpStream::connect(("127.0.0.1", port)) {
            c.write_all(format!("GET {target} HTTP/1.1\r\nHost: localhost\r\n\r\n").as_bytes())
                .unwrap();
            let mut out = String::new();
            let _ = c.read_to_string(&mut out);
            return out.lines().next().unwrap_or("").to_string();
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    panic!("receiver never listened");
}

#[test]
fn pkce_matches_rfc7636_and_rfc4648() {
    // RFC 7636 appendix B
    assert_eq!(
        challenge("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk"),
        "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
    );
    // base64url without padding (RFC 4648 §5 test vectors, plus bytes that use `-` and `_`)
    for (data, want) in [
        (&b""[..], ""),
        (b"f", "Zg"),
        (b"fo", "Zm8"),
        (b"foo", "Zm9v"),
        (b"\xff\xfe\xfd\x00\x01", "__79AAE"),
        (b"0123456789abcdefghij", "MDEyMzQ1Njc4OWFiY2RlZmdoaWo"),
    ] {
        assert_eq!(b64url(data), want, "{data:?}");
    }
    assert_eq!(
        token_urlsafe(48).unwrap().len(),
        64,
        "48 random bytes, base64url"
    );
    assert_eq!(token_urlsafe(32).unwrap().len(), 43);
    assert_ne!(token_urlsafe(32).unwrap(), token_urlsafe(32).unwrap());
}

#[test]
fn consent_url_is_the_frozen_one() {
    let scopes = s(&[
        "openid",
        "offline_access",
        "accounting.settings.read",
        "accounting.manualjournals",
    ]);
    let got = consent_url(
        "https://login.xero.com/identity/connect/authorize",
        "CLIENT ID",
        "http://localhost:8765/callback",
        &scopes,
        "st/ate+=",
        "chal_lenge-",
    );
    let want = std::fs::read_to_string(common::frozen("consent-url.txt")).unwrap();
    assert_eq!(got, want);
    assert_eq!(quote_plus("a b/c~"), "a+b%2Fc~");
}

#[test]
fn correction_a_never_adds_a_write_scope_and_never_drops_a_grant() {
    let universe = [
        "openid",
        "offline_access",
        "accounting.settings.read",
        "accounting.reports.trialbalance.read",
        "accounting.banktransactions",
        "accounting.manualjournals",
        "accounting.attachments",
    ];
    let defaults: Value =
        serde_json::from_str(include_str!("../../ownpurse-core/defaults.json")).unwrap();
    let default_read: Vec<String> = defaults["xero"]["scopes"]["read"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(
        default_read.iter().all(|x| !is_write_scope(x)),
        "the default read list is read-only"
    );
    let mut read_lists = vec![default_read.clone(), s(&["openid"]), vec![]];
    // a config that tries to smuggle a write scope into the read list
    read_lists.push(s(&["openid", "accounting.manualjournals"]));
    for read in &read_lists {
        for mask in 0u32..(1 << universe.len()) {
            let granted: Vec<String> = universe
                .iter()
                .enumerate()
                .filter(|(i, _)| mask & (1 << i) != 0)
                .map(|(_, x)| x.to_string())
                .collect();
            let req = login_scopes(read, &granted);
            for x in &req {
                assert!(
                    !is_write_scope(x) || granted.contains(x),
                    "new write scope {x} requested (read={read:?}, granted={granted:?})"
                );
            }
            for g in &granted {
                assert!(req.contains(g), "grant {g} dropped");
            }
            for r in read.iter().filter(|x| !is_write_scope(x)) {
                assert!(req.contains(r), "read scope {r} missing");
            }
            let mut dedup = req.clone();
            dedup.dedup();
            assert_eq!(dedup.len(), req.len(), "no duplicates");
        }
    }
    assert!(
        login_scopes(&default_read, &[])
            .iter()
            .all(|x| !is_write_scope(x))
    );
}

#[test]
fn receiver_ignores_wrong_callbacks_then_takes_the_code() {
    let (listener, port) = bound();
    let redirect = format!("http://127.0.0.1:{port}/callback");
    let statuses = Arc::new(Mutex::new(Vec::new()));
    let st = statuses.clone();
    let code = wait_on(
        listener,
        &redirect,
        "good-state",
        Duration::from_secs(10),
        &mut || {
            let st = st.clone();
            std::thread::spawn(move || {
                st.lock()
                    .unwrap()
                    .push(browse(port, "/callback?code=x&state=bad"));
                st.lock()
                    .unwrap()
                    .push(browse(port, "/other?code=x&state=good-state"));
                st.lock()
                    .unwrap()
                    .push(browse(port, "/callback?code=the%2Bcode&state=good-state"));
            });
        },
    )
    .unwrap();
    assert_eq!(code, "the+code");
    std::thread::sleep(Duration::from_millis(100));
    let st = statuses.lock().unwrap();
    assert!(
        st[0].contains("400") && st[1].contains("400") && st[2].contains("200"),
        "{st:?}"
    );
}

#[test]
fn receiver_reports_denied_consent_and_timeouts() {
    let (listener, port) = bound();
    let redirect = format!("http://127.0.0.1:{port}/callback");
    let e = wait_on(
        listener,
        &redirect,
        "s",
        Duration::from_secs(10),
        &mut || {
            std::thread::spawn(move || browse(port, "/callback?error=access_denied&state=s"));
        },
    )
    .unwrap_err();
    assert_eq!(e.code(), "needs_login");
    assert!(e.message().contains("access_denied"));
    let e = wait_on(
        bound().0,
        &redirect,
        "s",
        Duration::from_millis(200),
        &mut || {},
    )
    .unwrap_err();
    assert!(e.message().contains("Timed out"));
}

type Posted = Vec<(String, Vec<(String, String)>)>;

#[derive(Default)]
struct TokenEndpoint {
    calls: Mutex<Posted>,
}

impl FormPost for TokenEndpoint {
    fn post_form(&self, url: &str, form: &[(&str, &str)]) -> Result<(u16, String), String> {
        self.calls.lock().unwrap().push((
            url.into(),
            form.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        ));
        Ok((
            200,
            json!({"access_token": "at-new", "refresh_token": "rt-new", "expires_in": 1800,
                   "scope": "openid accounting.settings.read accounting.manualjournals"})
            .to_string(),
        ))
    }
}

#[test]
fn whole_login_keeps_grants_stores_privately_and_posts_only_to_the_token_url() {
    let d = temp_dir("login");
    let tok_path = d.join("tokens.json");
    // an existing token that already holds a write grant (as on a user's machine) plus an unknown field
    std::fs::write(
        &tok_path,
        json!({"access_token": "a", "refresh_token": "r", "scope": "openid accounting.manualjournals",
               "tenants": [{"tenantId": "t"}], "custom_field": "kept"})
        .to_string(),
    )
    .unwrap();
    let tokens = TokenStore::new(&tok_path, "CLIENT");
    let (listener, port) = bound();
    let redirect = format!("http://127.0.0.1:{port}/callback");
    let endpoint = TokenEndpoint::default();
    let read = s(&["openid", "offline_access", "accounting.settings.read"]);
    let seen_url = Arc::new(Mutex::new(String::new()));
    let su = seen_url.clone();
    let opts = Login {
        write_scopes: &[],
        auth_url: "https://login.example.invalid/authorize",
        token_url: "http://127.0.0.1:9/connect/token",
        client_id: "CLIENT",
        redirect_uri: &redirect,
        read_scopes: &read,
        timeout: Duration::from_secs(10),
        bound: Some(listener),
    };
    let tok = login(&opts, &tokens, &endpoint, &mut |url: &str| {
        *su.lock().unwrap() = url.to_string();
        let state = url
            .split("state=")
            .nth(1)
            .unwrap()
            .split('&')
            .next()
            .unwrap()
            .to_string();
        std::thread::spawn(move || browse(port, &format!("/callback?code=CODE123&state={state}")));
    })
    .unwrap();

    let url = seen_url.lock().unwrap().clone();
    let scope = url
        .split("scope=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();
    assert_eq!(
        scope, "openid+offline_access+accounting.settings.read+accounting.manualjournals",
        "read list, then the existing grant; nothing else"
    );
    assert!(url.contains("code_challenge_method=S256"));

    let calls = endpoint.calls.lock().unwrap();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].0, "http://127.0.0.1:9/connect/token");
    let f: Vec<&str> = calls[0].1.iter().map(|(k, _)| k.as_str()).collect();
    assert_eq!(
        f,
        [
            "grant_type",
            "client_id",
            "code",
            "redirect_uri",
            "code_verifier"
        ]
    );
    assert_eq!(calls[0].1[2].1, "CODE123");
    assert_eq!(calls[0].1[4].1.len(), 64);

    assert_eq!(tok["access_token"], "at-new");
    let saved: Value = serde_json::from_str(&std::fs::read_to_string(&tok_path).unwrap()).unwrap();
    assert_eq!(saved["refresh_token"], "rt-new");
    assert_eq!(
        saved["tenants"],
        json!([]),
        "a new consent clears tenants; connections fills them"
    );
    assert_eq!(saved["custom_field"], "kept");
    assert_eq!(
        std::fs::metadata(&tok_path).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn failed_exchange_keeps_the_old_token() {
    struct Fails;
    impl FormPost for Fails {
        fn post_form(&self, _: &str, _: &[(&str, &str)]) -> Result<(u16, String), String> {
            Ok((400, r#"{"error":"invalid_grant"}"#.into()))
        }
    }
    let d = temp_dir("login-fail");
    let p = d.join("tokens.json");
    std::fs::write(&p, r#"{"access_token":"a","refresh_token":"r"}"#).unwrap();
    let before = std::fs::read(&p).unwrap();
    let e = ownpurse_xero::auth::exchange_and_store(
        &TokenStore::new(&p, "C"),
        &Fails,
        "http://127.0.0.1:9/t",
        "C",
        "http://127.0.0.1:1/cb",
        "code",
        "verifier",
    )
    .unwrap_err();
    assert_eq!(e.code(), "api_error");
    assert_eq!(std::fs::read(&p).unwrap(), before);
}

#[test]
fn busy_redirect_port_is_a_clear_error() {
    let (held, port) = bound();
    let e = wait_for_code(
        &format!("http://127.0.0.1:{port}/callback"),
        "s",
        Duration::from_secs(1),
        &mut || {},
    )
    .unwrap_err();
    assert_eq!(e.code(), "needs_login");
    assert!(
        e.message().starts_with("Cannot listen on"),
        "{}",
        e.message()
    );
    drop(held);
}

#[test]
fn login_write_adds_exactly_the_configured_write_scopes() {
    let d = temp_dir("login-write");
    let tok_path = d.join("tokens.json");
    std::fs::write(
        &tok_path,
        json!({"access_token": "a", "refresh_token": "r", "scope": "openid"}).to_string(),
    )
    .unwrap();
    let tokens = TokenStore::new(&tok_path, "CLIENT");
    let (listener, port) = bound();
    let redirect = format!("http://127.0.0.1:{port}/callback");
    let endpoint = TokenEndpoint::default();
    let read = s(&["openid", "offline_access", "accounting.manualjournals.read"]);
    let write = s(&[
        "accounting.banktransactions",
        "accounting.manualjournals",
        "accounting.attachments",
    ]);
    let seen_url = Arc::new(Mutex::new(String::new()));
    let su = seen_url.clone();
    let opts = Login {
        write_scopes: &write,
        auth_url: "https://login.example.invalid/authorize",
        token_url: "http://127.0.0.1:9/connect/token",
        client_id: "CLIENT",
        redirect_uri: &redirect,
        read_scopes: &read,
        timeout: Duration::from_secs(10),
        bound: Some(listener),
    };
    login(&opts, &tokens, &endpoint, &mut |url: &str| {
        *su.lock().unwrap() = url.to_string();
        let state = url
            .split("state=")
            .nth(1)
            .unwrap()
            .split('&')
            .next()
            .unwrap()
            .to_string();
        std::thread::spawn(move || browse(port, &format!("/callback?code=CODE123&state={state}")));
    })
    .unwrap();
    let url = seen_url.lock().unwrap().clone();
    let scope = url
        .split("scope=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();
    assert_eq!(
        scope,
        "openid+offline_access+accounting.manualjournals.read+accounting.banktransactions+accounting.manualjournals+accounting.attachments"
    );
}
