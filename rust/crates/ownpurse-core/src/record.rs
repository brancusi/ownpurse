//! The local record, opened READ-ONLY. Only the API crate writes it (in WAL mode), so readers and the writer
//! never block each other.

use rusqlite::{Connection, OpenFlags, OptionalExtension, params};
use serde_json::Value;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum RecordError {
    Sqlite(rusqlite::Error),
    Json {
        entity: String,
        xero_id: String,
        error: serde_json::Error,
    },
    Missing(PathBuf),
}

impl std::fmt::Display for RecordError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecordError::Sqlite(e) => write!(f, "record: {e}"),
            RecordError::Json {
                entity,
                xero_id,
                error,
            } => write!(f, "record: {entity} {xero_id}: bad JSON: {error}"),
            RecordError::Missing(p) => write!(f, "no record at {}", p.display()),
        }
    }
}

impl std::error::Error for RecordError {}

impl From<rusqlite::Error> for RecordError {
    fn from(e: rusqlite::Error) -> Self {
        RecordError::Sqlite(e)
    }
}

pub type Result<T> = std::result::Result<T, RecordError>;

/// One current (latest-version) document.
#[derive(Debug, Clone)]
pub struct Row {
    pub xero_id: String,
    pub version: i64,
    pub status: Option<String>,
    pub observed_at: String,
    pub payload: Value,
}

/// A cached report snapshot.
#[derive(Debug, Clone)]
pub struct Report {
    pub fetched_at: String,
    pub payload: Value,
}

#[derive(Debug, Clone)]
pub struct SyncRun {
    pub entity: String,
    pub finished_at: String,
    pub outcome: String,
    pub calls: i64,
    pub fetched: i64,
    pub new_versions: i64,
}

pub struct Record {
    conn: Connection,
    path: PathBuf,
}

/// Canonical params for a TrialBalance snapshot, byte-identical to
/// `canonical({"date": as_of, "paymentsOnly": "false"})` (sorted keys, compact).
pub fn tb_params(as_of: &str) -> String {
    format!(r#"{{"date":"{as_of}","paymentsOnly":"false"}}"#)
}

/// Initialise SQLite once, on one thread, before any connection opens. SQLite's first-time initialisation
/// (`sqlite3_initialize`, which rusqlite runs inside every `Connection::open*`) is not safe to race on arm64:
/// two threads arriving together can see its mutex methods half-published and call a null function
/// (SIGSEGV in `sqlite3MutexInit`; seen once in the agent tests, which open fixtures from 16 threads).
/// `std::sync::Once` serialises the first call and publishes its effects to every later caller.
pub fn sqlite_ready() {
    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        let _ = Connection::open_in_memory();
    });
}

impl Record {
    /// Open read-only. Fails if the file does not exist (never creates one).
    pub fn open(path: impl AsRef<Path>) -> Result<Record> {
        let path = path.as_ref().to_path_buf();
        if !path.exists() {
            return Err(RecordError::Missing(path));
        }
        sqlite_ready();
        let conn = Connection::open_with_flags(
            &path,
            OpenFlags::SQLITE_OPEN_READ_ONLY
                | OpenFlags::SQLITE_OPEN_NO_MUTEX
                | OpenFlags::SQLITE_OPEN_URI,
        )?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        Ok(Record { conn, path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Latest version of every document of `entity` for a tenant, ordered by Xero id. Uses the record's `current` view.
    pub fn current_rows(&self, tenant_id: &str, entity: &str) -> Result<Vec<Row>> {
        let mut st = self.conn.prepare_cached(
            "SELECT xero_id, version, status, observed_at, payload FROM current \
             WHERE tenant_id=?1 AND entity=?2 ORDER BY xero_id",
        )?;
        let rows = st.query_map(params![tenant_id, entity], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (xero_id, version, status, observed_at, text) = row?;
            let payload = serde_json::from_str(&text).map_err(|error| RecordError::Json {
                entity: entity.to_string(),
                xero_id: xero_id.clone(),
                error,
            })?;
            out.push(Row {
                xero_id,
                version,
                status,
                observed_at,
                payload,
            });
        }
        Ok(out)
    }

    /// Payloads only. Crate-private: every reader goes through `View::payloads` (main or main + a branch).
    pub(crate) fn main_payloads(&self, tenant_id: &str, entity: &str) -> Result<Vec<Value>> {
        Ok(self
            .current_rows(tenant_id, entity)?
            .into_iter()
            .map(|r| r.payload)
            .collect())
    }

    /// One current document on main: `(version, payload)`.
    pub(crate) fn main_document(
        &self,
        tenant_id: &str,
        entity: &str,
        xero_id: &str,
    ) -> Result<Option<(i64, Value)>> {
        let row: Option<(i64, String)> = self
            .conn
            .query_row(
                "SELECT version, payload FROM versions WHERE tenant_id=?1 AND entity=?2 AND xero_id=?3 \
                 ORDER BY version DESC LIMIT 1",
                params![tenant_id, entity, xero_id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match row {
            None => Ok(None),
            Some((v, text)) => Ok(Some((
                v,
                serde_json::from_str(&text).map_err(|error| RecordError::Json {
                    entity: entity.to_string(),
                    xero_id: xero_id.to_string(),
                    error,
                })?,
            ))),
        }
    }

    /// The organisation settings payload, or `Value::Null` if never synced.
    pub fn organisation(&self, tenant_id: &str) -> Result<Value> {
        Ok(self
            .main_payloads(tenant_id, "Organisation")?
            .into_iter()
            .next()
            .unwrap_or(Value::Null))
    }

    /// Fiscal year end (month, day); 12/31 when unknown.
    pub fn fy_end(&self, tenant_id: &str) -> Result<(u32, u32)> {
        let o = self.organisation(tenant_id)?;
        let get = |k: &str, dflt: u32| {
            o.get(k)
                .and_then(|v| {
                    v.as_u64()
                        .map(|n| n as u32)
                        .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
                })
                .unwrap_or(dflt)
        };
        Ok((
            get("FinancialYearEndMonth", 12),
            get("FinancialYearEndDay", 31),
        ))
    }

    /// The latest date any TrialBalance is cached for (the year the books are being worked on).
    pub fn latest_tb_date(&self, tenant_id: &str) -> Result<Option<String>> {
        Ok(self.conn.query_row(
            "SELECT MAX(json_extract(params, '$.date')) FROM reports WHERE tenant_id=?1 AND report='TrialBalance'",
            params![tenant_id],
            |r| r.get::<_, Option<String>>(0),
        )?)
    }

    /// Latest cached accrual TrialBalance for exactly this date.
    pub fn tb_snapshot(&self, tenant_id: &str, as_of: &str) -> Result<Option<Report>> {
        self.report(tenant_id, "TrialBalance", &tb_params(as_of))
    }

    /// Latest cached report with exactly these canonical params.
    pub fn report(
        &self,
        tenant_id: &str,
        report: &str,
        params_json: &str,
    ) -> Result<Option<Report>> {
        let row: Option<(String, String)> = self.conn.query_row(
            "SELECT payload, fetched_at FROM reports WHERE tenant_id=?1 AND report=?2 AND params=?3 \
             ORDER BY id DESC LIMIT 1",
            params![tenant_id, report, params_json],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ).optional()?;
        match row {
            None => Ok(None),
            Some((text, fetched_at)) => {
                let payload = serde_json::from_str(&text).map_err(|error| RecordError::Json {
                    entity: report.to_string(),
                    xero_id: params_json.to_string(),
                    error,
                })?;
                Ok(Some(Report {
                    fetched_at,
                    payload,
                }))
            }
        }
    }

    /// Most recent successful sync of any entity for a tenant (record freshness).
    pub fn last_sync(&self, tenant_id: &str) -> Result<Option<String>> {
        Ok(self.conn.query_row(
            "SELECT MAX(finished_at) FROM sync_runs WHERE tenant_id=?1 AND outcome='ok'",
            params![tenant_id],
            |r| r.get::<_, Option<String>>(0),
        )?)
    }

    pub fn sync_runs(&self, tenant_id: &str) -> Result<Vec<SyncRun>> {
        let mut st = self.conn.prepare_cached(
            "SELECT entity, finished_at, outcome, calls, fetched, new_versions FROM sync_runs \
             WHERE tenant_id=?1 ORDER BY finished_at",
        )?;
        let rows = st.query_map(params![tenant_id], |r| {
            Ok(SyncRun {
                entity: r.get(0)?,
                finished_at: r.get(1)?,
                outcome: r.get(2)?,
                calls: r.get(3)?,
                fetched: r.get(4)?,
                new_versions: r.get(5)?,
            })
        })?;
        Ok(rows.collect::<std::result::Result<_, _>>()?)
    }

    /// Highest version id for a tenant: changes whenever anything new is recorded (cache key).
    pub fn max_version_id(&self, tenant_id: &str) -> Result<i64> {
        Ok(self.conn.query_row(
            "SELECT COALESCE(MAX(id),0) FROM versions WHERE tenant_id=?1",
            params![tenant_id],
            |r| r.get(0),
        )?)
    }

    /// Number of logged HTTP calls (tests assert this never changes on read paths).
    pub fn http_calls(&self) -> Result<i64> {
        Ok(self
            .conn
            .query_row("SELECT COUNT(*) FROM http_log", [], |r| r.get(0))?)
    }

    /// Direct access for read-only queries the other modules need (Activity log, usage).
    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
