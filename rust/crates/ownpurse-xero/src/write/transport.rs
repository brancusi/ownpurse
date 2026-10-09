//! The write transport (Phase 8): the only code that sends a non-GET request to the Accounting API.
//!
//! Before any byte leaves the machine a write passes, in order:
//! 1. the allow-list (`core::write::allowed`: four method + path shapes, no DELETE);
//! 2. the profile lock (`allow_writes`);
//! 3. the tenant guard: (a) the tenant is one of the profile's orgs, (b) it is not in `forbid_tenants`,
//!    (c) when `require_tenant_name` is set, the connected tenant's name equals it (checked live, once per
//!    tenant per run, before its first write);
//! 4. the host guard: with `OWNPURSE_WRITE_LOOPBACK_ONLY=1` (every test) only a loopback API base is used.
//!
//! Then it sends with the operation's Idempotency-Key. A 429 (Xero processed nothing) waits and resends;
//! a 401 on the first try refreshes the token once; anything else is returned as is, never retried: the run
//! decides (a 5xx or an error after sending is `uncertain`, reconciled by read-back). Every call is logged in
//! `http_log`.

use crate::http::{Resp, py_dumps_sorted, short};
use crate::store::{Writer, iso_micros, now_iso};
use crate::tokens::TokenStore;
use ownpurse_core::write::{
    Body, Method, Response, Send as Outgoing, TransportError, WriteTransport, allowed,
};
use rusqlite::types::Value as Sql;
use serde_json::Value;
use std::collections::HashSet;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Set to `1` in every test: writes and reads only go to a loopback API base.
pub const LOOPBACK_ENV: &str = "OWNPURSE_WRITE_LOOPBACK_ONLY";

/// Why a raw call failed: before anything was sent, or possibly after.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    NotSent(String),
    AfterSend(String),
}

/// The raw HTTP call (any method; the allow-list is checked above it). Tests use a loopback fake server
/// through the real [`UreqWire`], or a scripted wire.
pub trait Wire: Send + Sync {
    fn call(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&[u8]>,
    ) -> Result<Resp, WireError>;
}

pub struct UreqWire {
    agent: ureq::Agent,
}

impl Default for UreqWire {
    fn default() -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_connect(Some(Duration::from_secs(15)))
            .timeout_global(Some(Duration::from_secs(90)))
            .build();
        UreqWire {
            agent: config.into(),
        }
    }
}

fn wire_err(e: ureq::Error) -> WireError {
    use ureq::Error as E;
    match e {
        // nothing reached the server
        E::HostNotFound | E::ConnectionFailed | E::BadUri(_) | E::InvalidProxyUrl => {
            WireError::NotSent(e.to_string())
        }
        E::Timeout(t) => {
            let s = format!("timeout ({t:?})");
            if matches!(t, ureq::Timeout::Resolve | ureq::Timeout::Connect) {
                WireError::NotSent(s)
            } else {
                WireError::AfterSend(s)
            }
        }
        // a broken connection may have delivered the request
        other => WireError::AfterSend(other.to_string()),
    }
}

impl Wire for UreqWire {
    fn call(
        &self,
        method: &str,
        url: &str,
        headers: &[(String, String)],
        body: Option<&[u8]>,
    ) -> Result<Resp, WireError> {
        let mut r = match method {
            "GET" => {
                let mut req = self.agent.get(url);
                for (k, v) in headers {
                    req = req.header(k, v);
                }
                req.call()
            }
            "PUT" | "POST" => {
                let mut req = if method == "PUT" {
                    self.agent.put(url)
                } else {
                    self.agent.post(url)
                };
                for (k, v) in headers {
                    req = req.header(k, v);
                }
                req.send(body.unwrap_or_default())
            }
            m => {
                return Err(WireError::NotSent(format!(
                    "method {m} is not in the transport"
                )));
            }
        }
        .map_err(wire_err)?;
        let status = r.status().as_u16();
        let headers = r
            .headers()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_str().to_ascii_lowercase(),
                    v.to_str().unwrap_or("").to_string(),
                )
            })
            .collect();
        let body = r
            .body_mut()
            .with_config()
            .limit(64 * 1024 * 1024)
            .read_to_vec()
            .map_err(|e| WireError::AfterSend(format!("reading the response: {e}")))?;
        Ok(Resp {
            status,
            headers,
            body,
        })
    }
}

/// The profile facts the guards need.
#[derive(Debug, Clone)]
pub struct WriteSettings {
    pub profile: String,
    pub api_base: String,
    pub connections_url: String,
    /// alias → tenant id.
    pub orgs: Vec<(String, String)>,
    pub forbid_tenants: Vec<String>,
    pub require_tenant_name: Option<String>,
    pub allow_writes: bool,
    /// Only a loopback API base (tests): `OWNPURSE_WRITE_LOOPBACK_ONLY=1`.
    pub loopback_only: bool,
}

impl WriteSettings {
    pub fn loopback_from_env() -> bool {
        std::env::var(LOOPBACK_ENV).is_ok_and(|v| v == "1")
    }
}

pub type Sleep = Box<dyn Fn(f64) + Send + Sync>;

/// The write side of the API, guarded. Implements `core::write::WriteTransport`.
pub struct XeroWriter<'a> {
    pub settings: WriteSettings,
    store: &'a Writer,
    tokens: &'a TokenStore,
    wire: Box<dyn Wire>,
    sleep: Sleep,
    /// Tenants whose name was checked against `require_tenant_name` in this run.
    named: Mutex<HashSet<String>>,
}

fn refused(code: &str, message: impl Into<String>) -> TransportError {
    TransportError::Refused {
        code: code.into(),
        message: message.into(),
    }
}

/// Whether a URL's host is loopback (`127.0.0.1`, `[::1]`, `localhost`).
pub fn is_loopback(url: &str) -> bool {
    let rest = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
    let host_port = rest.split('/').next().unwrap_or("");
    let host = if let Some(h) = host_port.strip_prefix('[') {
        h.split(']').next().unwrap_or("")
    } else {
        host_port.split(':').next().unwrap_or("")
    };
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}

impl<'a> XeroWriter<'a> {
    pub fn new(
        settings: WriteSettings,
        store: &'a Writer,
        tokens: &'a TokenStore,
        wire: Box<dyn Wire>,
    ) -> XeroWriter<'a> {
        XeroWriter {
            settings,
            store,
            tokens,
            wire,
            sleep: Box::new(|s| std::thread::sleep(Duration::from_secs_f64(s))),
            named: Mutex::new(HashSet::new()),
        }
    }

    pub fn with_sleep(mut self, sleep: Sleep) -> Self {
        self.sleep = sleep;
        self
    }

    fn url(&self, path: &str) -> String {
        format!(
            "{}/{}",
            self.settings.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        )
    }

    fn host_guard(&self, url: &str) -> Result<(), TransportError> {
        if self.settings.loopback_only && !is_loopback(url) {
            return Err(refused(
                "not_loopback",
                format!(
                    "{LOOPBACK_ENV}=1: refusing a non-loopback host ({})",
                    short(url)
                ),
            ));
        }
        Ok(())
    }

    /// (a) one of the profile's orgs, (b) not forbidden.
    pub fn tenant_guard(&self, tenant_id: &str) -> Result<(), TransportError> {
        if !self.settings.orgs.iter().any(|(_, t)| t == tenant_id) {
            return Err(refused(
                "tenant_guard",
                format!(
                    "Tenant {tenant_id} is not one of profile '{}''s orgs.",
                    self.settings.profile
                ),
            ));
        }
        if self.settings.forbid_tenants.iter().any(|t| t == tenant_id) {
            return Err(refused(
                "tenant_guard",
                format!(
                    "Profile '{}' is forbidden from tenant {tenant_id}.",
                    self.settings.profile
                ),
            ));
        }
        Ok(())
    }

    /// (c) the connected tenant's name, once per tenant per run.
    fn name_guard(&mut self, tenant_id: &str) -> Result<(), TransportError> {
        let Some(want) = self.settings.require_tenant_name.clone() else {
            return Ok(());
        };
        if self
            .named
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .contains(tenant_id)
        {
            return Ok(());
        }
        let got = self.tenant_name(tenant_id)?;
        if got != want {
            return Err(refused(
                "tenant_guard",
                format!(
                    "Profile '{}' may only write to '{want}'; tenant {tenant_id} is '{got}'.",
                    self.settings.profile
                ),
            ));
        }
        self.named
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .insert(tenant_id.to_string());
        Ok(())
    }

    /// One HTTP exchange with the rate-limit and token handling; logged. 429 resends (Xero processed
    /// nothing); a 401 on the first try refreshes once.
    fn exchange(
        &self,
        method: &str,
        url: &str,
        tenant_id: Option<&str>,
        extra: &[(String, String)],
        body: Option<&[u8]>,
        key: Option<&str>,
    ) -> Result<Resp, TransportError> {
        self.host_guard(url)?;
        for attempt in 0..7u32 {
            let token = self
                .tokens
                .access_token()
                .map_err(|e| TransportError::NotSent(format!("token: {}", e.message())))?;
            let mut h = vec![
                ("Authorization".to_string(), format!("Bearer {token}")),
                ("Accept".to_string(), "application/json".to_string()),
            ];
            if let Some(t) = tenant_id {
                h.push(("Xero-Tenant-Id".into(), t.into()));
            }
            h.extend(extra.iter().cloned());
            let t0 = Instant::now();
            let r = self.wire.call(method, url, &h, body);
            self.log(method, url, tenant_id, key, &r, t0);
            let r = match r {
                Ok(r) => r,
                Err(WireError::NotSent(m)) => return Err(TransportError::NotSent(m)),
                Err(WireError::AfterSend(m)) => return Err(TransportError::AfterSend(m)),
            };
            if r.status == 401 && attempt == 0 {
                let _ = self.tokens.mark_stale(&token);
                continue;
            }
            if r.status == 429 {
                let problem = r
                    .header("x-rate-limit-problem")
                    .unwrap_or("")
                    .to_lowercase();
                if problem.starts_with("day") {
                    return Err(refused(
                        "daily_limit",
                        "Xero daily call limit reached for this organisation.",
                    ));
                }
                if attempt >= 5 {
                    return Err(refused("rate_limited", "Still rate limited after 5 waits."));
                }
                let wait: u32 = r
                    .header("retry-after")
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(5);
                (self.sleep)(f64::from(wait + 1));
                continue;
            }
            return Ok(r);
        }
        Err(refused("rate_limited", "Gave up after repeated 429/401."))
    }

    fn log(
        &self,
        method: &str,
        url: &str,
        tenant_id: Option<&str>,
        key: Option<&str>,
        r: &Result<Resp, WireError>,
        t0: Instant,
    ) {
        let int_h = |name: &str| match r {
            Ok(r) => r
                .header(name)
                .and_then(|v| v.trim().parse::<i64>().ok())
                .map(Sql::Integer)
                .unwrap_or(Sql::Null),
            Err(_) => Sql::Null,
        };
        let (status, problem) = match r {
            Ok(r) => (
                Sql::Integer(i64::from(r.status)),
                r.header("x-rate-limit-problem")
                    .map(|p| Sql::Text(p.to_string()))
                    .unwrap_or(Sql::Null),
            ),
            Err(WireError::NotSent(m)) => (Sql::Null, Sql::Text(format!("network:not_sent:{m}"))),
            Err(WireError::AfterSend(m)) => {
                (Sql::Null, Sql::Text(format!("network:after_send:{m}")))
            }
        };
        let params = key.map(|k| {
            Sql::Text(py_dumps_sorted(&[(
                "idempotency_key".to_string(),
                Value::String(k.to_string()),
            )]))
        });
        let _ = self.store.append(
            "http_log",
            &[
                ("at", Sql::Text(now_iso())),
                (
                    "tenant_id",
                    tenant_id.map(|t| Sql::Text(t.into())).unwrap_or(Sql::Null),
                ),
                ("method", Sql::Text(method.into())),
                ("path", Sql::Text(short(url))),
                ("params", params.unwrap_or(Sql::Null)),
                ("status", status),
                ("duration_ms", Sql::Integer(t0.elapsed().as_millis() as i64)),
                ("day_remaining", int_h("x-daylimit-remaining")),
                ("min_remaining", int_h("x-minlimit-remaining")),
                ("problem", problem),
                ("op_id", Sql::Null),
            ],
        );
    }

    fn response(r: Resp) -> Response {
        let retry_after = r.header("retry-after").and_then(|v| v.trim().parse().ok());
        let body = if r.body.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&r.body).unwrap_or_else(|_| {
                Value::String(
                    String::from_utf8_lossy(&r.body)
                        .chars()
                        .take(4000)
                        .collect(),
                )
            })
        };
        Response {
            status: r.status,
            body,
            retry_after,
        }
    }
}

impl WriteTransport for XeroWriter<'_> {
    fn send(&mut self, s: &Outgoing<'_>) -> Result<Response, TransportError> {
        let method = s.request.method;
        if !allowed(method, &s.path) {
            return Err(TransportError::NotAllowed(format!("{method:?} {}", s.path)));
        }
        if !self.settings.allow_writes {
            return Err(refused(
                "writes_disabled",
                format!(
                    "Profile '{}' does not set allow_writes.",
                    self.settings.profile
                ),
            ));
        }
        self.tenant_guard(s.tenant_id)?;
        let url = self.url(&s.path);
        self.host_guard(&url)?;
        self.name_guard(s.tenant_id)?;
        let (bytes, ctype) = match &s.request.body {
            Body::Json { json } => (
                ownpurse_core::canon::canonical(json).into_bytes(),
                "application/json".to_string(),
            ),
            Body::File { sha256, mime } => {
                let b = s
                    .file
                    .ok_or_else(|| refused("usage", "an attachment send carries no file bytes"))?;
                if ownpurse_core::write::sha256_hex(b) != *sha256 {
                    return Err(refused(
                        "precondition",
                        "the attachment's bytes changed since the package was sealed",
                    ));
                }
                (b.to_vec(), mime.clone())
            }
        };
        let extra = vec![
            ("Content-Type".to_string(), ctype),
            ("Idempotency-Key".to_string(), s.idempotency_key.to_string()),
        ];
        let m = match method {
            Method::Put => "PUT",
            Method::Post => "POST",
        };
        self.exchange(
            m,
            &url,
            Some(s.tenant_id),
            &extra,
            Some(&bytes),
            Some(s.idempotency_key),
        )
        .map(Self::response)
    }

    fn get(&mut self, tenant_id: &str, path_and_query: &str) -> Result<Response, TransportError> {
        self.tenant_guard(tenant_id)?;
        let url = self.url(path_and_query);
        self.exchange("GET", &url, Some(tenant_id), &[], None, None)
            .map(Self::response)
    }

    fn tenant_name(&mut self, tenant_id: &str) -> Result<String, TransportError> {
        let url = self.settings.connections_url.clone();
        let r = self.exchange("GET", &url, None, &[], None, None)?;
        if r.status != 200 {
            return Err(refused(
                "tenant_guard",
                format!("the connections call answered HTTP {}", r.status),
            ));
        }
        let all: Value = serde_json::from_slice(&r.body).unwrap_or(Value::Null);
        all.as_array()
            .into_iter()
            .flatten()
            .find(|c| c["tenantId"] == tenant_id)
            .and_then(|c| c["tenantName"].as_str().map(str::to_string))
            .ok_or_else(|| {
                refused(
                    "tenant_guard",
                    format!("tenant {tenant_id} is not connected to this token"),
                )
            })
    }

    fn scopes(&self) -> Vec<String> {
        self.tokens.granted_scopes().unwrap_or_default()
    }
}

/// Milliseconds since the epoch as ISO (for evidence timestamps).
pub fn iso_now() -> String {
    iso_micros(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loopback_hosts() {
        assert!(is_loopback("http://127.0.0.1:8123/api.xro/2.0"));
        assert!(is_loopback("http://localhost:9/x"));
        assert!(is_loopback("http://[::1]:9/x"));
        assert!(!is_loopback("https://api.xero.com/api.xro/2.0"));
        assert!(!is_loopback("https://127.0.0.1.evil.example/x"));
    }
}
