//! Token store shared by every process on this machine: one file, one lock.
//!
//! Xero refresh tokens rotate on every use, so a refresh must be serialised across processes:
//! take an exclusive `flock` on `<token file>.lock`, re-read the
//! file (another process may already have refreshed), refresh, and atomically replace the file (temp file,
//! mode 0600, fsync, rename) before the new token is used. The file is never deleted or truncated: a failed
//! refresh leaves it in place. Unknown fields are kept on every rewrite.

use crate::error::ApiError;
use crate::form::{FormPost, UreqForm};
use serde_json::{Map, Value};
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::io::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

pub const TOKEN_URL: &str = "https://identity.xero.com/connect/token";
pub const EARLY_REFRESH_SECONDS: f64 = 120.0;
pub const REFRESH_TOKEN_DAYS: f64 = 60.0;
pub const WARN_AFTER_DAYS: f64 = 45.0;

pub type Tok = Map<String, Value>;
pub type Clock = Box<dyn Fn() -> f64 + Send + Sync>;

pub fn system_clock() -> f64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs_f64())
        .unwrap_or(0.0)
}

pub struct TokenStore {
    pub path: PathBuf,
    pub client_id: String,
    pub token_url: String,
    http: Box<dyn FormPost>,
    clock: Clock,
}

/// Held while the cross-process lock is taken; dropping it closes the file, which releases the lock.
pub struct LockGuard {
    _file: File,
}

fn num(tok: &Tok, k: &str) -> Option<f64> {
    tok.get(k).and_then(Value::as_f64)
}

fn f64_value(x: f64) -> Value {
    serde_json::Number::from_f64(x)
        .map(Value::Number)
        .unwrap_or(Value::Null)
}

static TMP_SEQ: AtomicU64 = AtomicU64::new(0);

impl TokenStore {
    pub fn new(path: impl Into<PathBuf>, client_id: impl Into<String>) -> TokenStore {
        TokenStore {
            path: path.into(),
            client_id: client_id.into(),
            token_url: TOKEN_URL.into(),
            http: Box::new(UreqForm::default()),
            clock: Box::new(system_clock),
        }
    }

    pub fn with_token_url(mut self, url: impl Into<String>) -> Self {
        self.token_url = url.into();
        self
    }
    pub fn with_http(mut self, http: Box<dyn FormPost>) -> Self {
        self.http = http;
        self
    }
    pub fn with_clock(mut self, clock: Clock) -> Self {
        self.clock = clock;
        self
    }

    fn lock_path(&self) -> PathBuf {
        let mut name = self
            .path
            .file_name()
            .map(|n| n.to_os_string())
            .unwrap_or_default();
        name.push(".lock");
        self.path.with_file_name(name)
    }

    fn ensure_dir(&self) -> Result<(), ApiError> {
        if let Some(dir) = self.path.parent().filter(|d| !d.as_os_str().is_empty())
            && !dir.exists()
        {
            std::fs::create_dir_all(dir).map_err(|e| io_err("create the token folder", e))?;
            let _ = std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700));
        }
        Ok(())
    }

    // -- file primitives ---------------------------------------------------------------------------------

    /// The token file, or None when there is none. A damaged file is an error and is left in place.
    pub fn load(&self) -> Result<Option<Tok>, ApiError> {
        let text = match std::fs::read_to_string(&self.path) {
            Ok(t) => t,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(e) => return Err(io_err("read the token file", e)),
        };
        match serde_json::from_str::<Value>(&text) {
            Ok(Value::Object(m)) => Ok(Some(m)),
            Ok(_) => Err(unreadable(&self.path, "not a JSON object")),
            Err(e) => Err(unreadable(&self.path, &e.to_string())),
        }
    }

    /// Atomic replace: temp file in the same folder (mode 0600), fsync, rename. Never truncates in place.
    pub fn save(&self, tok: &Tok) -> Result<(), ApiError> {
        self.ensure_dir()?;
        let dir = self
            .path
            .parent()
            .filter(|d| !d.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let tmp = dir.join(format!(
            ".tokens-{}-{}",
            std::process::id(),
            TMP_SEQ.fetch_add(1, Ordering::Relaxed)
        ));
        let result = (|| -> std::io::Result<()> {
            let mut f = OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(&tmp)?;
            f.set_permissions(std::fs::Permissions::from_mode(0o600))?;
            let text = serde_json::to_string_pretty(&Value::Object(tok.clone()))
                .map_err(std::io::Error::other)?;
            f.write_all(text.as_bytes())?;
            f.flush()?;
            f.sync_all()?;
            std::fs::rename(&tmp, &self.path)
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&tmp);
        }
        result.map_err(|e| io_err("write the token file", e))
    }

    /// The cross-process lock (the same file and lock kind).
    pub fn lock(&self) -> Result<LockGuard, ApiError> {
        self.ensure_dir()?;
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(self.lock_path())
            .map_err(|e| io_err("open the token lock", e))?;
        loop {
            // SAFETY: flock on a valid, owned file descriptor; released when `file` is closed.
            let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
            if rc == 0 {
                break;
            }
            let err = std::io::Error::last_os_error();
            if err.kind() != std::io::ErrorKind::Interrupted {
                return Err(io_err("lock the token file", err));
            }
        }
        Ok(LockGuard { _file: file })
    }

    // -- state ---------------------------------------------------------------------------------------------

    #[allow(clippy::float_arithmetic)] // token times are Unix seconds as floats
    pub fn is_fresh(&self, tok: &Tok) -> bool {
        let obtained = num(tok, "obtained_at").unwrap_or(0.0);
        let expires = num(tok, "expires_in").unwrap_or(1800.0);
        (self.clock)() < obtained + expires - EARLY_REFRESH_SECONDS
    }

    /// Login state without secrets (scopes, tenants, freshness, refresh-token idle days).
    #[allow(clippy::float_arithmetic)]
    pub fn status(&self) -> Result<Map<String, Value>, ApiError> {
        let mut out = Map::new();
        let Some(tok) = self.load()? else {
            out.insert("logged_in".into(), Value::Bool(false));
            return Ok(out);
        };
        let last = num(&tok, "refreshed_at")
            .filter(|x| *x != 0.0)
            .or_else(|| num(&tok, "obtained_at"))
            .unwrap_or(0.0);
        let idle_days = ((self.clock)() - last) / 86400.0;
        let round1 = |x: f64| (x * 10.0).round() / 10.0;
        let scopes: Vec<Value> = tok
            .get("scope")
            .and_then(Value::as_str)
            .unwrap_or("")
            .split_whitespace()
            .map(|s| Value::String(s.into()))
            .collect();
        out.insert("logged_in".into(), Value::Bool(true));
        out.insert("scopes".into(), Value::Array(scopes));
        out.insert(
            "tenants".into(),
            tok.get("tenants").cloned().unwrap_or(Value::Array(vec![])),
        );
        out.insert(
            "access_token_fresh".into(),
            Value::Bool(self.is_fresh(&tok)),
        );
        out.insert("refresh_idle_days".into(), f64_value(round1(idle_days)));
        out.insert(
            "refresh_expires_in_days".into(),
            f64_value(round1(REFRESH_TOKEN_DAYS - idle_days)),
        );
        out.insert("warn".into(), Value::Bool(idle_days > WARN_AFTER_DAYS));
        Ok(out)
    }

    /// The granted scopes (space-separated `scope` field), for additive login consent.
    pub fn granted_scopes(&self) -> Result<Vec<String>, ApiError> {
        Ok(self
            .load()?
            .and_then(|t| t.get("scope").and_then(Value::as_str).map(str::to_string))
            .unwrap_or_default()
            .split_whitespace()
            .map(str::to_string)
            .collect())
    }

    pub fn set_tenants(&self, tenants: Value) -> Result<(), ApiError> {
        let _g = self.lock()?;
        let mut tok = self.load()?.unwrap_or_default();
        tok.insert("tenants".into(), tenants);
        self.save(&tok)
    }

    /// A 401 on a request: mark the access token stale (unless another process already replaced it), so the
    /// next `access_token` refreshes once.
    pub fn mark_stale(&self, token: &str) -> Result<(), ApiError> {
        let _g = self.lock()?;
        if let Some(mut tok) = self.load()?
            && tok.get("access_token").and_then(Value::as_str) == Some(token)
        {
            tok.insert("obtained_at".into(), Value::from(0));
            self.save(&tok)?;
        }
        Ok(())
    }

    // -- the one entry point for callers -----------------------------------------------------------------

    pub fn access_token(&self) -> Result<String, ApiError> {
        let tok = self.load()?;
        let Some(tok) = tok.filter(has_refresh) else {
            return Err(ApiError::needs_login(
                format!("Not logged in (no token at {}).", self.path.display()),
                "Run `ownpurse login`.",
            ));
        };
        if self.is_fresh(&tok) {
            return Ok(access(&tok));
        }
        let _g = self.lock()?;
        // another process may have refreshed while we waited
        match self.load()? {
            Some(t) if has_refresh(&t) && self.is_fresh(&t) => Ok(access(&t)),
            Some(t) if has_refresh(&t) => Ok(access(&self.refresh(&t)?)),
            _ => Err(ApiError::needs_login(
                format!("Not logged in (no token at {}).", self.path.display()),
                "Run `ownpurse login`.",
            )),
        }
    }

    /// Call with the lock held. Persists the rotated token before returning it.
    fn refresh(&self, tok: &Tok) -> Result<Tok, ApiError> {
        let rt = tok
            .get("refresh_token")
            .and_then(Value::as_str)
            .unwrap_or("");
        let (status, body) = self
            .http
            .post_form(
                &self.token_url,
                &[
                    ("grant_type", "refresh_token"),
                    ("refresh_token", rt),
                    ("client_id", &self.client_id),
                ],
            )
            .map_err(|_| {
                ApiError::api(
                    "Token refresh failed: network error.",
                    "Retry; the stored token was not changed.",
                )
            })?;
        if status == 400 || status == 401 {
            return Err(ApiError::needs_login(
                "Refresh token rejected by Xero (expired, revoked or already rotated).",
                "Run `ownpurse login`. The old token file is kept for inspection.",
            ));
        }
        if status >= 400 {
            return Err(ApiError::api(
                format!("Token refresh failed: HTTP {status}."),
                "Retry; the stored token was not changed.",
            ));
        }
        let resp = match serde_json::from_str::<Value>(&body) {
            Ok(Value::Object(m)) if m.get("access_token").and_then(Value::as_str).is_some() => m,
            _ => {
                return Err(ApiError::api(
                    "Token refresh failed: unexpected response.",
                    "Retry; the stored token was not changed.",
                ));
            }
        };
        // Start from the stored token so unknown fields survive (a plain refresh would drop them; keeping them is the
        // approved, stricter behaviour), then take everything Xero returned.
        let mut new = tok.clone();
        for (k, v) in resp {
            new.insert(k, v);
        }
        let now = f64_value((self.clock)());
        new.insert("obtained_at".into(), now.clone());
        new.insert("refreshed_at".into(), now);
        new.insert(
            "tenants".into(),
            tok.get("tenants").cloned().unwrap_or(Value::Array(vec![])),
        );
        if !new.contains_key("scope") {
            new.insert("scope".into(), Value::from(""));
        }
        self.save(&new)?; // persist before anyone uses the rotated token
        Ok(new)
    }
}

fn has_refresh(tok: &Tok) -> bool {
    tok.get("refresh_token")
        .and_then(Value::as_str)
        .is_some_and(|s| !s.is_empty())
}

fn access(tok: &Tok) -> String {
    tok.get("access_token")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn unreadable(path: &Path, why: &str) -> ApiError {
    ApiError::needs_login(
        format!("Token file {} is unreadable ({why}).", path.display()),
        "Run `ownpurse login` (the damaged file is left in place).",
    )
}

fn io_err(what: &str, e: std::io::Error) -> ApiError {
    ApiError::api(
        format!("Could not {what} ({e})."),
        "Check the folder's permissions; the stored token was not changed.",
    )
}
