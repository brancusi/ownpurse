//! The record's WRITE side: appends, byte-compatible with the frozen reference records, so `verify()` proves
//! the chain of a record written by any version.
//!
//! Only the API layer opens this (sync, the call log); core's `Record` stays read-only. History tables are
//! append-only (triggers), every appended row is linked into the SHA-256 `chain` inside one
//! `BEGIN IMMEDIATE` transaction, and the schema/views text is fixed (`schema.sql`, `views.sql`; a new record
//! must equal the frozen reference schema, checked by a test).

use crate::error::ApiError;
use ownpurse_core::canon::canonical;
use rusqlite::types::Value as Sql;
use rusqlite::{Connection, OptionalExtension, params};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const SCHEMA_VERSION: i64 = 1;
pub const APPEND_ONLY: [&str; 10] = [
    "versions",
    "reports",
    "sync_runs",
    "http_log",
    "operations",
    "op_events",
    "approvals",
    "packages",
    "verifications",
    "chain",
];
/// Tables created on first use (Phase 8), not by `migrate`: a record that never wrote keeps the frozen schema.
pub const LAZY: [&str; 3] = ["approvals", "packages", "verifications"];
const SCHEMA: &str = include_str!("schema.sql");
/// Additive tables after the frozen reference schema (Phase 8: `approvals`).
pub const SCHEMA_WRITES: &str = include_str!("schema_writes.sql");
/// The `packages` table (Phase 8), created on the first seal.
pub const SCHEMA_PACKAGES: &str = include_str!("schema_packages.sql");
/// The `verifications` table (v0.13.0), created on the first `write verify`.
pub const SCHEMA_VERIFICATIONS: &str = include_str!("schema_verifications.sql");
const VIEWS: &str = include_str!("views.sql");

pub struct Writer {
    conn: Connection,
    pub path: PathBuf,
}

/// A pinned clock for `now_iso` (the demo/fixture generator only: its records must be byte-identical run to
/// run). `None` (the default) is the wall clock; the CLI and sync never set it.
static PINNED_NOW: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// Pin (`Some("2026-10-05T07:46:00Z")`) or release (`None`) the clock `now_iso` reads.
pub fn pin_clock(at: Option<&str>) {
    *PINNED_NOW.lock().unwrap_or_else(|e| e.into_inner()) = at.map(str::to_string);
}

/// `datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%S.%fZ")`, or the pinned clock (`pin_clock`).
pub fn now_iso() -> String {
    if let Some(at) = PINNED_NOW.lock().unwrap_or_else(|e| e.into_inner()).clone() {
        return at;
    }
    iso_micros(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default(),
    )
}

/// A duration since the epoch as `YYYY-MM-DDTHH:MM:SS.ffffffZ`.
pub fn iso_micros(d: std::time::Duration) -> String {
    let secs = d.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let (y, m, day) = ownpurse_core::dates::civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{day:02}T{:02}:{:02}:{:02}.{:06}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        d.subsec_micros()
    )
}

/// `str(uuid.uuid4())`.
pub fn new_id() -> Result<String, ApiError> {
    let mut b = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| std::io::Read::read_exact(&mut f, &mut b))
        .map_err(|e| ApiError::api(format!("No randomness available ({e})."), ""))?;
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    Ok(format!(
        "{}-{}-{}-{}-{}",
        &h[0..8],
        &h[8..12],
        &h[12..16],
        &h[16..20],
        &h[20..32]
    ))
}

pub fn sha256_hex(text: &str) -> String {
    let mut h = Sha256::new();
    h.update(text.as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn db_err(e: rusqlite::Error) -> ApiError {
    ApiError::api(
        format!("record: {e}"),
        "The record was not changed by this operation.",
    )
}

impl Writer {
    /// Open (creating it if needed) with WAL, foreign keys and a 10 s busy timeout.
    pub fn open(path: impl AsRef<Path>) -> Result<Writer, ApiError> {
        let path = path.as_ref().to_path_buf();
        if let Some(dir) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir).map_err(|e| {
                ApiError::api(
                    format!("Could not create {} ({e}).", dir.display()),
                    "Check the folder's permissions.",
                )
            })?;
        }
        ownpurse_core::record::sqlite_ready();
        let conn = Connection::open(&path).map_err(db_err)?;
        conn.busy_timeout(std::time::Duration::from_secs(10))
            .map_err(db_err)?;
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(db_err)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(db_err)?;
        let w = Writer { conn, path };
        w.migrate()?;
        Ok(w)
    }

    fn migrate(&self) -> Result<(), ApiError> {
        let fresh: bool = self
            .conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='versions'",
                [],
                |_| Ok(()),
            )
            .optional()
            .map_err(db_err)?
            .is_none();
        self.conn.execute_batch(SCHEMA).map_err(db_err)?;
        self.conn.execute_batch(VIEWS).map_err(db_err)?;
        let has_rows = self
            .conn
            .query_row("SELECT 1 FROM versions LIMIT 1", [], |_| Ok(()))
            .optional()
            .map_err(db_err)?
            .is_some();
        let fmt = if fresh || !has_rows {
            "decimal"
        } else {
            "float-legacy"
        };
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta VALUES ('number_format', ?1)",
                params![fmt],
            )
            .map_err(db_err)?;
        for t in APPEND_ONLY.iter().filter(|t| !LAZY.contains(t)) {
            for op in ["UPDATE", "DELETE"] {
                self.conn
                    .execute_batch(&format!(
                        "CREATE TRIGGER IF NOT EXISTS {t}_no_{} BEFORE {op} ON {t} \
                         BEGIN SELECT RAISE(ABORT, '{t} is append-only'); END",
                        op.to_lowercase()
                    ))
                    .map_err(db_err)?;
            }
        }
        self.conn
            .execute(
                "INSERT OR IGNORE INTO meta VALUES ('schema_version', ?1)",
                params![SCHEMA_VERSION.to_string()],
            )
            .map_err(db_err)?;
        Ok(())
    }

    /// Whether the record has a table.
    pub fn has_table(&self, name: &str) -> Result<bool, ApiError> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name=?1",
                [name],
                |_| Ok(()),
            )
            .optional()
            .map_err(db_err)?
            .is_some())
    }

    /// Create the `packages` table, its index and its append-only triggers (idempotent).
    pub fn ensure_packages(&self) -> Result<(), ApiError> {
        self.ensure_lazy("packages", SCHEMA_PACKAGES)
    }

    /// Create the `verifications` table, its index and its append-only triggers (idempotent).
    pub fn ensure_verifications(&self) -> Result<(), ApiError> {
        self.ensure_lazy("verifications", SCHEMA_VERIFICATIONS)
    }

    fn ensure_lazy(&self, tbl: &str, schema: &str) -> Result<(), ApiError> {
        self.conn.execute_batch(schema).map_err(db_err)?;
        for op in ["UPDATE", "DELETE"] {
            self.conn
                .execute_batch(&format!(
                    "CREATE TRIGGER IF NOT EXISTS {tbl}_no_{} BEFORE {op} ON {tbl} \
                     BEGIN SELECT RAISE(ABORT, '{tbl} is append-only'); END",
                    op.to_lowercase()
                ))
                .map_err(db_err)?;
        }
        Ok(())
    }

    /// Whether the record has the Phase 8 `approvals` table (created on the first approval, so a record that
    /// never saw one keeps the frozen reference schema byte for byte).
    pub fn has_approvals(&self) -> Result<bool, ApiError> {
        Ok(self
            .conn
            .query_row(
                "SELECT 1 FROM sqlite_master WHERE type='table' AND name='approvals'",
                [],
                |_| Ok(()),
            )
            .optional()
            .map_err(db_err)?
            .is_some())
    }

    /// Create the `approvals` table, its index and its append-only triggers (idempotent).
    pub fn ensure_approvals(&self) -> Result<(), ApiError> {
        // an approvals table from before `demo_standing` (its CHECK lists only tty and chat): rebuilt with the
        // same rows and ids, so every row hash, and with it the chain, is unchanged
        let sql: Option<String> = self
            .conn
            .query_row(
                "SELECT sql FROM sqlite_master WHERE type='table' AND name='approvals'",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_err)?;
        if sql.is_some_and(|q| !q.contains("demo_standing")) {
            self.tx(|| {
                self.conn
                    .execute_batch(
                        "DROP TRIGGER IF EXISTS approvals_no_update; DROP TRIGGER IF EXISTS approvals_no_delete; \
                         DROP INDEX IF EXISTS approvals_hash; ALTER TABLE approvals RENAME TO approvals_v1;",
                    )
                    .map_err(db_err)?;
                self.conn.execute_batch(SCHEMA_WRITES).map_err(db_err)?;
                self.conn
                    .execute_batch(
                        "INSERT INTO approvals SELECT * FROM approvals_v1; DROP TABLE approvals_v1;",
                    )
                    .map_err(db_err)?;
                Ok(())
            })?;
        }
        self.conn.execute_batch(SCHEMA_WRITES).map_err(db_err)?;
        for op in ["UPDATE", "DELETE"] {
            self.conn
                .execute_batch(&format!(
                    "CREATE TRIGGER IF NOT EXISTS approvals_no_{} BEFORE {op} ON approvals \
                     BEGIN SELECT RAISE(ABORT, 'approvals is append-only'); END",
                    op.to_lowercase()
                ))
                .map_err(db_err)?;
        }
        Ok(())
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    /// Append one row and its chain entry atomically. Returns the row id
    /// (`id`, else `op_id`, else the rowid).
    pub fn append(&self, tbl: &str, row: &[(&str, Sql)]) -> Result<String, ApiError> {
        assert!(
            APPEND_ONLY.contains(&tbl) && tbl != "chain",
            "append to {tbl}"
        );
        let in_tx = !self.conn.is_autocommit();
        if !in_tx {
            self.conn.execute_batch("BEGIN IMMEDIATE").map_err(db_err)?;
        }
        let result = (|| {
            let cols: Vec<&str> = row.iter().map(|(c, _)| *c).collect();
            let marks: Vec<String> = (1..=row.len()).map(|i| format!("?{i}")).collect();
            let vals: Vec<&Sql> = row.iter().map(|(_, v)| v).collect();
            self.conn
                .execute(
                    &format!(
                        "INSERT INTO {tbl} ({}) VALUES ({})",
                        cols.join(", "),
                        marks.join(", ")
                    ),
                    rusqlite::params_from_iter(vals),
                )
                .map_err(db_err)?;
            let given = row
                .iter()
                // the table's own key (`op_events` carries an `op_id` column that is not its key)
                .find(|(c, _)| *c == Self::pk(tbl))
                .and_then(|(_, v)| match v {
                    Sql::Text(s) => Some(s.clone()),
                    Sql::Integer(i) => Some(i.to_string()),
                    _ => None,
                });
            let row_id = given.unwrap_or_else(|| self.conn.last_insert_rowid().to_string());
            self.chain(tbl, &row_id)?;
            Ok(row_id)
        })();
        if !in_tx {
            let end = if result.is_ok() { "COMMIT" } else { "ROLLBACK" };
            self.conn.execute_batch(end).map_err(db_err)?;
        }
        result
    }

    fn pk(tbl: &str) -> &'static str {
        if tbl == "operations" { "op_id" } else { "id" }
    }

    /// The row hash: sha256 of `canonical({column: value})` for the row as stored.
    pub fn row_hash(&self, tbl: &str, row_id: &str) -> Result<String, ApiError> {
        let mut st = self
            .conn
            .prepare(&format!("SELECT * FROM {tbl} WHERE {} = ?1", Self::pk(tbl)))
            .map_err(db_err)?;
        let names: Vec<String> = st.column_names().iter().map(|s| s.to_string()).collect();
        let m = st
            .query_row(params![row_id], |r| {
                let mut m = Map::new();
                for (i, n) in names.iter().enumerate() {
                    let v = match r.get::<_, Sql>(i)? {
                        Sql::Null => Value::Null,
                        Sql::Integer(x) => Value::from(x),
                        Sql::Text(s) => Value::String(s),
                        Sql::Real(x) => serde_json::Number::from_f64(x)
                            .map(Value::Number)
                            .unwrap_or(Value::Null),
                        Sql::Blob(_) => Value::Null,
                    };
                    m.insert(n.clone(), v);
                }
                Ok(m)
            })
            .optional()
            .map_err(db_err)?
            .ok_or_else(|| {
                ApiError::new(
                    "integrity",
                    format!("{tbl} row {row_id} referenced by chain is missing"),
                    "",
                )
            })?;
        Ok(sha256_hex(&canonical(&Value::Object(m))))
    }

    /// Prove the chain: every entry links to the one before, every chained row still hashes to what was
    /// recorded, and every append-only table has exactly one entry per row. Returns (entries, head hash).
    pub fn verify(&self) -> Result<(i64, String), ApiError> {
        let integrity = |m: String| ApiError::new("integrity", m, "");
        let mut st = self
            .conn
            .prepare("SELECT seq, tbl, row_id, row_hash, prev_hash, hash FROM chain ORDER BY seq")
            .map_err(db_err)?;
        let rows: Vec<(i64, String, String, String, String, String)> = st
            .query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                ))
            })
            .map_err(db_err)?
            .collect::<Result<_, _>>()
            .map_err(db_err)?;
        let mut prev = "0".repeat(64);
        let mut n = 0;
        for (seq, tbl, row_id, row_hash, prev_hash, hash) in rows {
            if prev_hash != prev {
                return Err(integrity(format!(
                    "chain broken at seq {seq} (prev hash mismatch)"
                )));
            }
            if self.row_hash(&tbl, &row_id)? != row_hash {
                return Err(integrity(format!(
                    "{tbl} row {row_id} changed after it was recorded"
                )));
            }
            if sha256_hex(&format!("{prev}{tbl}{row_id}{row_hash}")) != hash {
                return Err(integrity(format!("chain hash wrong at seq {seq}")));
            }
            prev = hash;
            n += 1;
        }
        let mut present = vec![];
        for t in LAZY {
            if self.has_table(t)? {
                present.push(t);
            }
        }
        for t in APPEND_ONLY[..APPEND_ONLY.len() - 1]
            .iter()
            .filter(|t| !LAZY.contains(t) || present.contains(t))
        {
            let count = |sql: &str| -> Result<i64, ApiError> {
                self.conn
                    .query_row(sql, params![*t], |r| r.get(0))
                    .map_err(db_err)
            };
            let rows = self
                .conn
                .query_row(&format!("SELECT COUNT(*) FROM {t}"), [], |r| {
                    r.get::<_, i64>(0)
                })
                .map_err(db_err)?;
            let chained = count("SELECT COUNT(*) FROM chain WHERE tbl = ?1")?;
            if rows != chained {
                return Err(integrity(format!(
                    "{t}: {rows} rows but {chained} chain entries"
                )));
            }
        }
        Ok((n, prev))
    }

    fn chain(&self, tbl: &str, row_id: &str) -> Result<(), ApiError> {
        let prev: String = self
            .conn
            .query_row(
                "SELECT hash FROM chain ORDER BY seq DESC LIMIT 1",
                [],
                |r| r.get(0),
            )
            .optional()
            .map_err(db_err)?
            .unwrap_or_else(|| "0".repeat(64));
        let row_hash = self.row_hash(tbl, row_id)?;
        let hash = sha256_hex(&format!("{prev}{tbl}{row_id}{row_hash}"));
        self.conn
            .execute(
                "INSERT INTO chain (at, tbl, row_id, row_hash, prev_hash, hash) VALUES (?1,?2,?3,?4,?5,?6)",
                params![now_iso(), tbl, row_id, row_hash, prev, hash],
            )
            .map_err(db_err)?;
        Ok(())
    }

    /// Calls logged for this tenant (or tenant-less calls) since `since_iso`.
    pub fn calls_since(&self, tenant_id: Option<&str>, since_iso: &str) -> Result<i64, ApiError> {
        self.conn
            .query_row(
                "SELECT COUNT(*) FROM http_log WHERE tenant_id IS ?1 AND at >= ?2",
                params![tenant_id, since_iso],
                |r| r.get(0),
            )
            .map_err(db_err)
    }

    /// Run `f` in one `BEGIN IMMEDIATE` transaction (nested calls join the outer one).
    pub fn tx<T>(&self, f: impl FnOnce() -> Result<T, ApiError>) -> Result<T, ApiError> {
        if !self.conn.is_autocommit() {
            return f();
        }
        self.conn.execute_batch("BEGIN IMMEDIATE").map_err(db_err)?;
        match f() {
            Ok(v) => {
                self.conn.execute_batch("COMMIT").map_err(db_err)?;
                Ok(v)
            }
            Err(e) => {
                let _ = self.conn.execute_batch("ROLLBACK");
                Err(e)
            }
        }
    }

    /// Latest (version, content_hash) per document of an entity.
    pub fn latest_hashes(
        &self,
        tenant_id: &str,
        entity: &str,
    ) -> Result<HashMap<String, (i64, String)>, ApiError> {
        let mut st = self
            .conn
            .prepare("SELECT xero_id, version, content_hash FROM current WHERE tenant_id=?1 AND entity=?2")
            .map_err(db_err)?;
        let rows = st
            .query_map(params![tenant_id, entity], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    (r.get::<_, i64>(1)?, r.get::<_, String>(2)?),
                ))
            })
            .map_err(db_err)?;
        rows.collect::<Result<_, _>>().map_err(db_err)
    }

    /// Append a version when the content differs from the latest. True if appended.
    #[allow(clippy::too_many_arguments)]
    pub fn record_version(
        &self,
        tenant_id: &str,
        entity: &str,
        xero_id: &str,
        payload: &Value,
        status: Option<&str>,
        updated_utc: Option<&str>,
        source: &str,
        sync_run_id: Option<&str>,
        known: &mut HashMap<String, (i64, String)>,
    ) -> Result<bool, ApiError> {
        self.record_version_by(
            tenant_id,
            entity,
            xero_id,
            payload,
            status,
            updated_utc,
            source,
            sync_run_id,
            None,
            known,
        )
    }

    /// `record_version` with the write operation that caused it (`caused_by_op`; a write's read-back).
    #[allow(clippy::too_many_arguments)]
    pub fn record_version_by(
        &self,
        tenant_id: &str,
        entity: &str,
        xero_id: &str,
        payload: &Value,
        status: Option<&str>,
        updated_utc: Option<&str>,
        source: &str,
        sync_run_id: Option<&str>,
        caused_by_op: Option<&str>,
        known: &mut HashMap<String, (i64, String)>,
    ) -> Result<bool, ApiError> {
        let text = canonical(payload);
        let h = sha256_hex(&text);
        let latest = known.get(xero_id).cloned();
        if latest.as_ref().is_some_and(|l| l.1 == h) {
            return Ok(false);
        }
        let version = latest.map(|l| l.0 + 1).unwrap_or(1);
        let opt = |v: Option<&str>| v.map(|s| Sql::Text(s.into())).unwrap_or(Sql::Null);
        self.append(
            "versions",
            &[
                ("tenant_id", Sql::Text(tenant_id.into())),
                ("entity", Sql::Text(entity.into())),
                ("xero_id", Sql::Text(xero_id.into())),
                ("version", Sql::Integer(version)),
                ("content_hash", Sql::Text(h.clone())),
                ("status", opt(status)),
                ("xero_updated_utc", opt(updated_utc)),
                ("observed_at", Sql::Text(now_iso())),
                ("source", Sql::Text(source.into())),
                ("sync_run_id", opt(sync_run_id)),
                ("caused_by_op", opt(caused_by_op)),
                ("payload", Sql::Text(text)),
            ],
        )?;
        known.insert(xero_id.to_string(), (version, h));
        Ok(true)
    }

    /// `finished_at` of the latest successful sync of an entity (optionally of one mode).
    pub fn last_sync(
        &self,
        tenant_id: &str,
        entity: &str,
        mode: Option<&str>,
    ) -> Result<Option<String>, ApiError> {
        let mut q = String::from(
            "SELECT finished_at FROM sync_runs WHERE tenant_id=?1 AND entity=?2 AND outcome='ok'",
        );
        if mode.is_some() {
            q.push_str(" AND mode=?3");
        }
        q.push_str(" ORDER BY finished_at DESC LIMIT 1");
        let r = match mode {
            Some(m) => self
                .conn
                .query_row(&q, params![tenant_id, entity, m], |r| r.get(0)),
            None => self
                .conn
                .query_row(&q, params![tenant_id, entity], |r| r.get(0)),
        };
        r.optional().map_err(db_err)
    }

    pub fn high_water(&self, tenant_id: &str, entity: &str) -> Result<Option<String>, ApiError> {
        self.conn
            .query_row(
                "SELECT MAX(high_water) FROM sync_runs WHERE tenant_id=?1 AND entity=?2 AND outcome='ok'",
                params![tenant_id, entity],
                |r| r.get(0),
            )
            .map_err(db_err)
    }
}
