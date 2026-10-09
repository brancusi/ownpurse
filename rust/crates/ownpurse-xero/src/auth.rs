//! PKCE login with a loopback receiver (the only interactive step).
//!
//! Scopes (Phase 6 correction A): login requests the configured read scopes PLUS every scope already granted
//! on the stored token, so a re-login never silently drops a permission. Plain login never adds a write
//! scope; `login --write` (Phase 8; only for a profile with `allow_writes`, checked by the CLI before the
//! browser opens) adds the configured `xero.scopes.write`, and the owner grants them in the browser.

use crate::error::ApiError;
use crate::form::FormPost;
use crate::tokens::{TokenStore, system_clock};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};

pub const AUTH_URL: &str = "https://login.xero.com/identity/connect/authorize";

/// `True` for a scope that grants write access (anything under `accounting.` not ending in `.read`).
pub fn is_write_scope(s: &str) -> bool {
    s.starts_with("accounting.") && !s.ends_with(".read")
}

/// The scopes to request: `read` (config order), then every already-granted scope not yet listed.
/// Write scopes in `read` are dropped (a config cannot smuggle one in); granted write scopes are kept.
pub fn login_scopes(read: &[String], granted: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for s in read {
        if !is_write_scope(s) && !out.contains(s) {
            out.push(s.clone());
        }
    }
    for s in granted {
        if !out.contains(s) {
            out.push(s.clone());
        }
    }
    out
}

fn random_bytes(n: usize) -> Result<Vec<u8>, ApiError> {
    let mut buf = vec![0u8; n];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut buf))
        .map_err(|e| ApiError::api(format!("No randomness available ({e})."), ""))?;
    Ok(buf)
}

/// URL-safe base64 without padding.
pub fn b64url(data: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        let chars = [(n >> 18) & 63, (n >> 12) & 63, (n >> 6) & 63, n & 63];
        for c in chars.iter().take(chunk.len() + 1) {
            out.push(A[*c as usize] as char);
        }
    }
    out
}

/// `secrets.token_urlsafe(n)`.
pub fn token_urlsafe(n: usize) -> Result<String, ApiError> {
    Ok(b64url(&random_bytes(n)?))
}

/// The S256 challenge for a verifier.
pub fn challenge(verifier: &str) -> String {
    b64url(&Sha256::digest(verifier.as_bytes()))
}

/// `(verifier, challenge)`.
pub fn pkce_pair() -> Result<(String, String), ApiError> {
    let v = token_urlsafe(48)?;
    let c = challenge(&v);
    Ok((v, c))
}

/// `urllib.parse.quote_plus`.
pub fn quote_plus(s: &str) -> String {
    let mut out = String::new();
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'.' | b'-' | b'~' => {
                out.push(b as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn unquote_plus(s: &str) -> String {
    let hex = |b: u8| (b as char).to_digit(16).map(|d| d as u8);
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'+' => out.push(b' '),
            b'%' if i + 2 < bytes.len() => match (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                (Some(h), Some(l)) => {
                    out.push(h * 16 + l);
                    i += 2;
                }
                _ => out.push(b'%'),
            },
            b => out.push(b),
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

/// The consent URL (fixed parameter order, `urlencode` quoting).
pub fn consent_url(
    auth_url: &str,
    client_id: &str,
    redirect_uri: &str,
    scopes: &[String],
    state: &str,
    challenge: &str,
) -> String {
    let params = [
        ("response_type", "code".to_string()),
        ("client_id", client_id.to_string()),
        ("redirect_uri", redirect_uri.to_string()),
        ("scope", scopes.join(" ")),
        ("state", state.to_string()),
        ("code_challenge", challenge.to_string()),
        ("code_challenge_method", "S256".to_string()),
    ];
    let q: Vec<String> = params
        .iter()
        .map(|(k, v)| format!("{}={}", quote_plus(k), quote_plus(v)))
        .collect();
    format!("{auth_url}?{}", q.join("&"))
}

/// Split a redirect URI into (port, path).
fn port_and_path(redirect_uri: &str) -> (u16, String) {
    let rest = redirect_uri
        .split_once("://")
        .map(|(_, r)| r)
        .unwrap_or(redirect_uri);
    let (hostport, path) = match rest.split_once('/') {
        Some((h, p)) => (h, format!("/{p}")),
        None => (rest, "/".to_string()),
    };
    let port = hostport
        .rsplit_once(':')
        .and_then(|(_, p)| p.parse().ok())
        .unwrap_or(80);
    (port, path)
}

/// Listen on the redirect URI until the browser brings back the code for `state` (or an error, or the
/// timeout). `ready` runs once the socket is bound, so a fast redirect can never arrive before we listen.
/// Authorization codes are never logged or printed.
pub fn wait_for_code(
    redirect_uri: &str,
    state: &str,
    timeout: Duration,
    ready: &mut dyn FnMut(),
) -> Result<String, ApiError> {
    let (port, _) = port_and_path(redirect_uri);
    let listener = TcpListener::bind(("127.0.0.1", port)).map_err(|e| {
        ApiError::needs_login(
            format!("Cannot listen on {redirect_uri} ({e})."),
            "Free the port, or set xero.redirect_uri (and register it on the Xero app).",
        )
    })?;
    wait_on(listener, redirect_uri, state, timeout, ready)
}

/// `wait_for_code` on an already-bound listener (tests bind port 0 to avoid port races).
pub fn wait_on(
    listener: TcpListener,
    redirect_uri: &str,
    state: &str,
    timeout: Duration,
    ready: &mut dyn FnMut(),
) -> Result<String, ApiError> {
    let (_, expected_path) = port_and_path(redirect_uri);
    listener
        .set_nonblocking(true)
        .map_err(|e| ApiError::api(format!("listener: {e}"), ""))?;
    ready();
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let mut stream = match listener.accept() {
            Ok((s, _)) => s,
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(_) => continue,
        };
        let _ = stream.set_nonblocking(false);
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut first = String::new();
        let mut reader = BufReader::new(match stream.try_clone() {
            Ok(s) => s,
            Err(_) => continue,
        });
        if reader.read_line(&mut first).is_err() {
            continue;
        }
        // drain the headers
        loop {
            let mut l = String::new();
            match reader.read_line(&mut l) {
                Ok(0) | Err(_) => break,
                Ok(_) if l.trim().is_empty() => break,
                Ok(_) => {}
            }
        }
        let target = first.split_whitespace().nth(1).unwrap_or("");
        let (path, query) = target.split_once('?').unwrap_or((target, ""));
        let mut q: Vec<(String, String)> = Vec::new();
        for kv in query.split('&').filter(|s| !s.is_empty()) {
            let (k, v) = kv.split_once('=').unwrap_or((kv, ""));
            q.push((unquote_plus(k), unquote_plus(v)));
        }
        let all = |k: &str| -> Vec<&String> {
            q.iter().filter(|(n, _)| n == k).map(|(_, v)| v).collect()
        };
        let ok = path == expected_path && all("state") == vec![state];
        let mut result: Option<Result<String, ApiError>> = None;
        if ok {
            if let Some(err) = all("error").first() {
                result = Some(Err(ApiError::needs_login(
                    format!("Xero consent was not granted ({err})."),
                    "Run `ownpurse login` again.",
                )));
            } else if all("code").len() == 1 {
                result = Some(Ok(all("code")[0].clone()));
            }
        }
        let msg = if ok {
            "Xero login received. You can close this tab."
        } else {
            "Unexpected callback; keep the Xero tab open."
        };
        let body = format!("<h2>{msg}</h2>");
        let _ = stream.write_all(
            format!(
                "HTTP/1.1 {}\r\nContent-Type: text/html; charset=utf-8\r\nCache-Control: no-store\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                if ok { "200 OK" } else { "400 Bad Request" },
                body.len()
            )
            .as_bytes(),
        );
        if let Some(r) = result {
            return r;
        }
    }
    Err(ApiError::needs_login(
        "Timed out waiting for the browser consent.",
        "Run `ownpurse login` again.",
    ))
}

/// Exchange the code for tokens and store them (under the lock). Keeps unknown fields of an existing file;
/// clears `tenants` (a new consent can change the orgs; `connections` fills them again).
pub fn exchange_and_store(
    tokens: &TokenStore,
    http: &dyn FormPost,
    token_url: &str,
    client_id: &str,
    redirect_uri: &str,
    code: &str,
    verifier: &str,
) -> Result<serde_json::Map<String, Value>, ApiError> {
    let (status, body) = http
        .post_form(
            token_url,
            &[
                ("grant_type", "authorization_code"),
                ("client_id", client_id),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("code_verifier", verifier),
            ],
        )
        .map_err(|_| {
            ApiError::api(
                "Token exchange failed (network error).",
                "Run `ownpurse login` again.",
            )
        })?;
    if status != 200 {
        return Err(ApiError::api(
            format!("Token exchange failed (HTTP {status})."),
            "Run `ownpurse login` again.",
        ));
    }
    let resp = match serde_json::from_str::<Value>(&body) {
        Ok(Value::Object(m)) => m,
        _ => {
            return Err(ApiError::api(
                "Token response incomplete.",
                "Run `ownpurse login` again.",
            ));
        }
    };
    let present = |k: &str| {
        resp.get(k)
            .and_then(Value::as_str)
            .is_some_and(|s| !s.is_empty())
    };
    if !present("access_token") || !present("refresh_token") {
        return Err(ApiError::api(
            "Token response incomplete.",
            "Run `ownpurse login` again.",
        ));
    }
    let _g = tokens.lock()?;
    let mut tok = tokens.load().ok().flatten().unwrap_or_default();
    for (k, v) in resp {
        tok.insert(k, v);
    }
    let now = serde_json::Number::from_f64(system_clock())
        .map(Value::Number)
        .unwrap_or(Value::Null);
    tok.insert("obtained_at".into(), now.clone());
    tok.insert("refreshed_at".into(), now);
    tok.insert("tenants".into(), Value::Array(vec![]));
    tokens.save(&tok)?;
    Ok(tok)
}

/// Options for one login.
pub struct Login<'a> {
    pub auth_url: &'a str,
    pub token_url: &'a str,
    pub client_id: &'a str,
    pub redirect_uri: &'a str,
    pub read_scopes: &'a [String],
    /// `login --write`: the write scopes to request as well (empty for a plain login).
    pub write_scopes: &'a [String],
    pub timeout: Duration,
    /// A listener already bound to the redirect port (tests); `None` binds it here.
    pub bound: Option<TcpListener>,
}

/// The whole flow: scopes (read ∪ granted), PKCE, consent URL, wait for the code (`announce` gets the URL once
/// the receiver listens; the caller opens the browser there), exchange, store.
pub fn login(
    opts: &Login<'_>,
    tokens: &TokenStore,
    http: &dyn FormPost,
    announce: &mut dyn FnMut(&str),
) -> Result<serde_json::Map<String, Value>, ApiError> {
    let granted = tokens.granted_scopes().unwrap_or_default();
    let mut scopes = login_scopes(opts.read_scopes, &granted);
    for w in opts.write_scopes {
        if !scopes.contains(w) {
            scopes.push(w.clone());
        }
    }
    let (verifier, chal) = pkce_pair()?;
    let state = token_urlsafe(32)?;
    let url = consent_url(
        opts.auth_url,
        opts.client_id,
        opts.redirect_uri,
        &scopes,
        &state,
        &chal,
    );
    let mut ready = || announce(&url);
    let code = match opts.bound.as_ref().map(TcpListener::try_clone) {
        Some(Ok(l)) => wait_on(l, opts.redirect_uri, &state, opts.timeout, &mut ready)?,
        _ => wait_for_code(opts.redirect_uri, &state, opts.timeout, &mut ready)?,
    };
    exchange_and_store(
        tokens,
        http,
        opts.token_url,
        opts.client_id,
        opts.redirect_uri,
        &code,
        &verifier,
    )
}
