//! HTTP client for the Accounting API, GET only.
//!
//! - 429 with `X-Rate-Limit-Problem: day…` → `daily_limit` at once (never sleeps for hours).
//! - 429 minute/concurrent → honour `Retry-After` (+1 s) and retry, at most 5 times.
//! - Network errors (3 tries) and 5xx (2 retries) back off and retry.
//! - 401 on the first attempt marks the access token stale and retries once (one refresh).
//! - Every attempt is appended to `http_log` with the budget headers.
//!
//! There is no POST/PUT here at all (Phase 6 correction B): the transport trait only has `get`; the token
//! endpoint's POST lives in the token store.

use crate::error::ApiError;
use crate::store::{Writer, iso_micros};
use crate::tokens::TokenStore;
use ownpurse_core::canon::json_str;
use rusqlite::types::Value as Sql;
use serde_json::{Map, Value};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// One HTTP response: status, headers (names lower-cased), body.
#[derive(Debug, Clone, Default)]
pub struct Resp {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Resp {
    pub fn header(&self, name: &str) -> Option<&str> {
        let n = name.to_ascii_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| *k == n)
            .map(|(_, v)| v.as_str())
    }
}

/// The only way this crate reaches the API: GET. `Err(kind)` = no HTTP response (`kind` names the failure).
pub trait GetTransport: Send + Sync {
    fn get(
        &self,
        url: &str,
        query: &[(String, String)],
        headers: &[(String, String)],
    ) -> Result<Resp, String>;
}

pub struct UreqGet {
    agent: ureq::Agent,
}

impl Default for UreqGet {
    fn default() -> Self {
        let config = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(60)))
            .build();
        UreqGet {
            agent: config.into(),
        }
    }
}

impl GetTransport for UreqGet {
    fn get(
        &self,
        url: &str,
        query: &[(String, String)],
        headers: &[(String, String)],
    ) -> Result<Resp, String> {
        let mut req = self.agent.get(url);
        for (k, v) in query {
            req = req.query(k, v);
        }
        for (k, v) in headers {
            req = req.header(k, v);
        }
        let mut r = req.call().map_err(|e| match e {
            ureq::Error::Timeout(_) => "TimeoutException".to_string(),
            ureq::Error::Io(_) | ureq::Error::HostNotFound | ureq::Error::ConnectionFailed => {
                "ConnectError".to_string()
            }
            _ => "TransportError".to_string(),
        })?;
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
            .limit(512 * 1024 * 1024)
            .read_to_vec()
            .map_err(|_| "ReadError".to_string())?;
        Ok(Resp {
            status,
            headers,
            body,
        })
    }
}

/// What the client needs from the config.
#[derive(Debug, Clone)]
pub struct HttpSettings {
    pub profile: String,
    pub api_base: String,
    pub connections_url: String,
    pub minute_soft_limit: i64,
    pub forbid_tenants: Vec<String>,
}

/// One logged attempt, for progress reporting (`--progress jsonl`).
#[derive(Debug, Clone)]
pub struct CallInfo {
    pub tenant_id: Option<String>,
    pub path: String,
    pub page: Option<i64>,
    pub status: Option<u16>,
    pub day_remaining: Option<i64>,
}

/// Why the client is about to sleep: `throttle` (our soft per-minute cap), `minute_limit` (Xero 429), `retry`
/// (5xx or network backoff).
pub type Sleep = Box<dyn Fn(f64, &'static str) + Send + Sync>;
pub type OnCall = Box<dyn Fn(&CallInfo) + Send + Sync>;

pub struct XeroHttp<'a> {
    pub settings: HttpSettings,
    pub store: &'a Writer,
    pub tokens: &'a TokenStore,
    transport: Box<dyn GetTransport>,
    sleep: Sleep,
    on_call: Option<OnCall>,
}

/// The path relative to the API host, for the call log.
pub fn short(url: &str) -> String {
    match url.split_once("/api.xro/2.0/") {
        Some((_, rest)) => rest.to_string(),
        None => {
            let no_scheme = url.split_once("://").map(|(_, r)| r).unwrap_or(url);
            no_scheme
                .split_once('/')
                .map(|(_, r)| r)
                .unwrap_or(no_scheme)
                .to_string()
        }
    }
}

/// The params as JSON: sorted keys, ASCII-escaped, `", "` / `": "` separators.
pub fn py_dumps_sorted(params: &[(String, Value)]) -> String {
    let mut items: Vec<&(String, Value)> = params.iter().collect();
    items.sort_by(|a, b| a.0.cmp(&b.0));
    let mut out = String::from("{");
    for (i, (k, v)) in items.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        json_str(&mut out, k, true);
        out.push_str(": ");
        match v {
            Value::String(s) => json_str(&mut out, s, true),
            other => out.push_str(&other.to_string()),
        }
    }
    out.push('}');
    out
}

fn query_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => if *b { "true" } else { "false" }.into(),
        other => other.to_string(),
    }
}

fn int_header(r: &Resp, name: &str) -> Option<i64> {
    r.header(name).and_then(|v| v.trim().parse().ok())
}

impl<'a> XeroHttp<'a> {
    pub fn new(
        settings: HttpSettings,
        store: &'a Writer,
        tokens: &'a TokenStore,
        transport: Box<dyn GetTransport>,
    ) -> XeroHttp<'a> {
        XeroHttp {
            settings,
            store,
            tokens,
            transport,
            sleep: Box::new(|s, _| std::thread::sleep(Duration::from_secs_f64(s))),
            on_call: None,
        }
    }

    pub fn with_sleep(mut self, sleep: Sleep) -> Self {
        self.sleep = sleep;
        self
    }

    pub fn with_on_call(mut self, f: OnCall) -> Self {
        self.on_call = Some(f);
        self
    }

    // -- tenants -------------------------------------------------------------------------------------------

    /// The connected organisations (also stored in the token file).
    pub fn connections(&self) -> Result<Vec<Map<String, Value>>, ApiError> {
        let r = self.send(&self.settings.connections_url.clone(), None, &[], &[])?;
        let all: Value = serde_json::from_slice(&r.body)
            .map_err(|_| ApiError::api("Unexpected connections response.", "Retry the command."))?;
        let tenants: Vec<Map<String, Value>> = all
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|c| {
                        let mut m = Map::new();
                        for k in ["tenantId", "tenantName", "tenantType"] {
                            m.insert(k.into(), c.get(k).cloned().unwrap_or(Value::Null));
                        }
                        m
                    })
                    .collect()
            })
            .unwrap_or_default();
        self.tokens.set_tenants(Value::Array(
            tenants.iter().cloned().map(Value::Object).collect(),
        ))?;
        Ok(tenants)
    }

    pub fn guard_tenant(&self, tenant_id: &str) -> Result<(), ApiError> {
        if self.settings.forbid_tenants.iter().any(|t| t == tenant_id) {
            return Err(ApiError::new(
                "usage",
                format!(
                    "Profile '{}' is forbidden from tenant {tenant_id}.",
                    self.settings.profile
                ),
                "",
            ));
        }
        Ok(())
    }

    // -- requests ------------------------------------------------------------------------------------------

    /// GET an Accounting API path for a tenant. A 304 (not modified) gives an empty object.
    pub fn get(
        &self,
        tenant_id: &str,
        path: &str,
        params: &[(String, Value)],
        if_modified_since: Option<&str>,
    ) -> Result<Value, ApiError> {
        let headers: Vec<(String, String)> = if_modified_since
            .map(|v| vec![("If-Modified-Since".to_string(), v.to_string())])
            .unwrap_or_default();
        let url = format!(
            "{}/{}",
            self.settings.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let r = self.send(&url, Some(tenant_id), params, &headers)?;
        if r.status == 304 {
            return Ok(Value::Object(Map::new()));
        }
        serde_json::from_slice(&r.body).map_err(|e| {
            ApiError::api(
                format!("Xero returned invalid JSON for {} ({e}).", short(&url)),
                "Retry the command.",
            )
        })
    }

    /// GET a file (an attachment's bytes): the raw body, any content type.
    pub fn get_bytes(&self, tenant_id: &str, path: &str) -> Result<Vec<u8>, ApiError> {
        let url = format!(
            "{}/{}",
            self.settings.api_base.trim_end_matches('/'),
            path.trim_start_matches('/')
        );
        let r = self.send(
            &url,
            Some(tenant_id),
            &[],
            &[("Accept".to_string(), "application/octet-stream".to_string())],
        )?;
        Ok(r.body)
    }

    fn throttle(&self, tenant_id: Option<&str>) -> Result<(), ApiError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let since = iso_micros(now.saturating_sub(Duration::from_secs(60)));
        if self.store.calls_since(tenant_id, &since)? >= self.settings.minute_soft_limit {
            (self.sleep)(5.0, "throttle");
        }
        Ok(())
    }

    fn send(
        &self,
        url: &str,
        tenant_id: Option<&str>,
        params: &[(String, Value)],
        extra: &[(String, String)],
    ) -> Result<Resp, ApiError> {
        if let Some(t) = tenant_id {
            self.guard_tenant(t)?;
        }
        let query: Vec<(String, String)> = params
            .iter()
            .map(|(k, v)| (k.clone(), query_value(v)))
            .collect();
        let mut network_tries = 0u32;
        for attempt in 0..8u32 {
            self.throttle(tenant_id)?;
            let token = self.tokens.access_token()?;
            let mut h = vec![("Authorization".to_string(), format!("Bearer {token}"))];
            // JSON unless the caller asks for something else (a file download)
            if !extra.iter().any(|(k, _)| k.eq_ignore_ascii_case("Accept")) {
                h.push(("Accept".to_string(), "application/json".to_string()));
            }
            if let Some(t) = tenant_id {
                h.push(("Xero-Tenant-Id".into(), t.into()));
            }
            h.extend(extra.iter().cloned());
            let t0 = Instant::now();
            let r = match self.transport.get(url, &query, &h) {
                Ok(r) => r,
                Err(kind) => {
                    self.log(
                        url,
                        tenant_id,
                        params,
                        None,
                        t0,
                        None,
                        Some(format!("network:{kind}")),
                    )?;
                    network_tries += 1;
                    if network_tries < 3 {
                        (self.sleep)(f64::from(2u32.pow(network_tries)), "retry");
                        continue;
                    }
                    return Err(ApiError::api(
                        format!("Network error on GET {} ({kind}).", short(url)),
                        "Retry the command.",
                    ));
                }
            };
            let problem = r.header("x-rate-limit-problem").map(str::to_string);
            self.log(
                url,
                tenant_id,
                params,
                Some(r.status),
                t0,
                Some(&r),
                problem.clone(),
            )?;
            if r.status == 401 && attempt == 0 {
                // access token revoked/expired early: mark it stale (unless another process already
                // replaced it) so the next attempt refreshes, once
                self.tokens.mark_stale(&token)?;
                continue;
            }
            if r.status == 429 {
                let p = problem.unwrap_or_default();
                if p.to_lowercase().starts_with("day") {
                    return Err(ApiError::new(
                        "daily_limit",
                        "Xero daily call limit reached for this organisation.",
                        "Use cached data (`--offline`).",
                    )
                    .with_hint(
                        "Xero resets each org's daily limit at its own time; `ownpurse usage` shows what's left.",
                    ));
                }
                let wait: u32 = r
                    .header("retry-after")
                    .and_then(|v| v.trim().parse().ok())
                    .unwrap_or(5);
                if attempt >= 5 {
                    return Err(ApiError::new(
                        "rate_limited",
                        format!("Still rate limited after {attempt} retries ({p})."),
                        "Wait a minute and retry.",
                    ));
                }
                (self.sleep)(f64::from(wait + 1), "minute_limit");
                continue;
            }
            if r.status >= 500 && attempt < 2 {
                (self.sleep)(f64::from(2u32.pow(attempt + 1)), "retry");
                continue;
            }
            if r.status < 400 || r.status == 304 {
                return Ok(r);
            }
            return Err(Self::raise(&r, url));
        }
        Err(ApiError::api(format!("Gave up on GET {}.", short(url)), ""))
    }

    fn raise(r: &Resp, url: &str) -> ApiError {
        let text: String = String::from_utf8_lossy(&r.body)
            .chars()
            .take(1500)
            .collect();
        match r.status {
            401 => {
                ApiError::needs_login("Xero rejected the access token.", "Run `ownpurse login`.")
            }
            403 => ApiError::new(
                "forbidden",
                format!(
                    "Forbidden: GET {} — missing scope or org access.",
                    short(url)
                ),
                "Run `ownpurse` to see granted scopes; `ownpurse login` to re-consent.",
            )
            .with_body(text),
            404 => ApiError::new("not_found", format!("Not found: {}.", short(url)), "")
                .with_body(text),
            s => ApiError::api(format!("Xero HTTP {s} on GET {}.", short(url)), "").with_body(text),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn log(
        &self,
        url: &str,
        tenant_id: Option<&str>,
        params: &[(String, Value)],
        status: Option<u16>,
        t0: Instant,
        resp: Option<&Resp>,
        problem: Option<String>,
    ) -> Result<(), ApiError> {
        let opt_i = |v: Option<i64>| v.map(Sql::Integer).unwrap_or(Sql::Null);
        if let Some(f) = &self.on_call {
            f(&CallInfo {
                tenant_id: tenant_id.map(str::to_string),
                path: short(url),
                page: params
                    .iter()
                    .find(|(k, _)| k == "page")
                    .and_then(|(_, v)| v.as_i64()),
                status,
                day_remaining: resp.and_then(|r| int_header(r, "x-daylimit-remaining")),
            });
        }
        self.store.append(
            "http_log",
            &[
                ("at", Sql::Text(crate::store::now_iso())),
                (
                    "tenant_id",
                    tenant_id.map(|t| Sql::Text(t.into())).unwrap_or(Sql::Null),
                ),
                ("method", Sql::Text("GET".into())),
                ("path", Sql::Text(short(url))),
                (
                    "params",
                    if params.is_empty() {
                        Sql::Null
                    } else {
                        Sql::Text(py_dumps_sorted(params))
                    },
                ),
                ("status", opt_i(status.map(i64::from))),
                ("duration_ms", Sql::Integer(t0.elapsed().as_millis() as i64)),
                (
                    "day_remaining",
                    opt_i(resp.and_then(|r| int_header(r, "x-daylimit-remaining"))),
                ),
                (
                    "min_remaining",
                    opt_i(resp.and_then(|r| int_header(r, "x-minlimit-remaining"))),
                ),
                ("problem", problem.map(Sql::Text).unwrap_or(Sql::Null)),
                ("op_id", Sql::Null),
            ],
        )?;
        Ok(())
    }
}
