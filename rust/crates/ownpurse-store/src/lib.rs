//! The store: the log in SQLite and the drawer on disk, around a `Db` value held in memory.
//!
//! `Store::create` writes genesis and the core vocabulary; `transact` resolves a transaction against the
//! current `Db` (pure, in `ownpurse-facts`) and appends what it produced in one SQLite transaction, chained by
//! hash. `verify` re-derives the chain and re-hashes every file the log cites.

pub mod drawer;
pub mod sql;

pub use drawer::Drawer;

use ownpurse_facts::encode::{ZERO_HASH, tx_hash};
use ownpurse_facts::{
    CORE_SCHEMA, Datom, Db, Eid, Hash, Instant, Kw, TxData, TxError, TxReport, Value, install,
};
use rusqlite::{Connection, OpenFlags, params};
use std::fmt;
use std::path::{Path, PathBuf};

pub const FORMAT: &str = "ownpurse.log.v1";

#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Io(std::io::Error),
    Tx(TxError),
    Corrupt(String),
    Exists(PathBuf),
    Missing(PathBuf),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoreError::Sqlite(e) => write!(f, "log: {e}"),
            StoreError::Io(e) => write!(f, "drawer: {e}"),
            StoreError::Tx(e) => write!(f, "{e}"),
            StoreError::Corrupt(m) => write!(f, "the log does not verify: {m}"),
            StoreError::Exists(p) => write!(f, "{} already exists; open it instead", p.display()),
            StoreError::Missing(p) => {
                write!(f, "no log at {}; create one with `init`", p.display())
            }
        }
    }
}

impl std::error::Error for StoreError {}

impl From<rusqlite::Error> for StoreError {
    fn from(e: rusqlite::Error) -> Self {
        StoreError::Sqlite(e)
    }
}

impl From<std::io::Error> for StoreError {
    fn from(e: std::io::Error) -> Self {
        StoreError::Io(e)
    }
}

impl From<TxError> for StoreError {
    fn from(e: TxError) -> Self {
        StoreError::Tx(e)
    }
}

pub struct Store {
    conn: Connection,
    db: Db,
    head: Hash,
    drawer: Drawer,
    path: PathBuf,
}

/// The drawer that sits beside a log: `books.sqlite` → `books.drawer/`.
pub fn default_drawer(log: &Path) -> PathBuf {
    log.with_extension("drawer")
}

/// What `verify` found.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Verified {
    pub transactions: usize,
    pub datoms: usize,
    pub files: usize,
    pub redacted: usize,
    pub head: String,
}

impl Store {
    /// A new log with genesis and the core vocabulary, at `instant`.
    pub fn create(
        path: &Path,
        drawer: Option<PathBuf>,
        instant: &Instant,
    ) -> Result<Store, StoreError> {
        if path.exists() {
            return Err(StoreError::Exists(path.to_path_buf()));
        }
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = FULL;")?;
        conn.execute_batch(include_str!("schema.sql"))?;
        conn.execute(
            "INSERT INTO meta(key, value) VALUES ('format', ?1)",
            params![FORMAT],
        )?;
        let mut store = Store {
            conn,
            db: Db::genesis(),
            head: ZERO_HASH,
            drawer: Drawer::new(drawer.unwrap_or_else(|| default_drawer(path))),
            path: path.to_path_buf(),
        };
        let genesis: Vec<Datom> = store.db.log().to_vec();
        store.append(
            ownpurse_facts::schema::GENESIS_TX,
            &genesis,
            &Instant::from_unix_millis(0),
        )?;
        let defs: serde_json::Value = serde_json::from_str(CORE_SCHEMA)
            .map_err(|e| StoreError::Corrupt(format!("core schema: {e}")))?;
        // A transaction cannot use an attribute it installs, so this one carries no tx/actor; the next can.
        let tx = install(&defs).map_err(StoreError::Corrupt)?;
        store.transact(&tx, instant)?;
        Ok(store)
    }

    /// Open a log, replay it into a `Db`, and check the chain as it loads.
    pub fn open(path: &Path, drawer: Option<PathBuf>) -> Result<Store, StoreError> {
        if !path.exists() {
            return Err(StoreError::Missing(path.to_path_buf()));
        }
        let conn = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_WRITE)?;
        let (db, head) = load(&conn)?;
        Ok(Store {
            conn,
            db,
            head,
            drawer: Drawer::new(drawer.unwrap_or_else(|| default_drawer(path))),
            path: path.to_path_buf(),
        })
    }

    pub fn db(&self) -> &Db {
        &self.db
    }

    pub fn drawer(&self) -> &Drawer {
        &self.drawer
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn head(&self) -> Hash {
        self.head
    }

    /// Resolve and append. An empty transaction (nothing new) writes nothing and returns `tx: None`.
    pub fn transact(&mut self, data: &TxData, instant: &Instant) -> Result<TxReport, StoreError> {
        let report = self.db.with(data, instant)?;
        if let Some(t) = report.tx {
            self.append(t, &report.datoms, instant)?;
            self.db = report.db_after.clone();
        }
        Ok(report)
    }

    fn append(&mut self, t: Eid, datoms: &[Datom], instant: &Instant) -> Result<(), StoreError> {
        let hash = tx_hash(&self.head, t, datoms);
        let sqltx = self.conn.transaction()?;
        sqltx.execute(
            "INSERT INTO tx(t, instant, prev, hash) VALUES (?1, ?2, ?3, ?4)",
            params![sql::id(t), instant.as_str(), self.head.hex(), hash.hex()],
        )?;
        {
            let mut ins = sqltx.prepare_cached(
                "INSERT INTO datom(e, a, v, vt, tx, added) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )?;
            for d in datoms {
                ins.execute(params![
                    sql::id(d.e),
                    sql::id(d.a),
                    sql::to_sql(&d.v),
                    d.v.value_type().tag(),
                    sql::id(t),
                    d.added
                ])?;
            }
        }
        sqltx.commit()?;
        self.head = hash;
        Ok(())
    }

    /// Re-derive the chain from the stored datoms and re-hash every file the log cites (unless redacted).
    pub fn verify(&self) -> Result<Verified, StoreError> {
        let (db, head) = load(&self.conn)?;
        let mut out = Verified {
            transactions: db.txs().len(),
            datoms: db.log().len(),
            head: head.hex(),
            ..Verified::default()
        };
        let (Some(sha), Some(redacted)) = (
            db.attr(&Kw::lit("file/sha256")),
            db.attr(&Kw::lit("file/redacted")),
        ) else {
            return Ok(out);
        };
        for (v, e) in db.attr_values(sha.id) {
            let Value::Hash(h) = v else { continue };
            if db.value(e, redacted.id).is_some() {
                out.redacted += 1;
                continue;
            }
            let bytes = self.drawer.get(h).map_err(|err| {
                StoreError::Corrupt(format!("{h} is cited but missing from the drawer ({err})"))
            })?;
            if ownpurse_facts::encode::sha256(&bytes) != *h {
                return Err(StoreError::Corrupt(format!(
                    "{h}: the drawer's bytes do not match their name"
                )));
            }
            out.files += 1;
        }
        Ok(out)
    }
}

/// Read the log in order, re-deriving each transaction's hash and comparing it with the stored one.
fn load(conn: &Connection) -> Result<(Db, Hash), StoreError> {
    let format: String = conn
        .query_row("SELECT value FROM meta WHERE key = 'format'", [], |r| {
            r.get(0)
        })
        .map_err(|_| StoreError::Corrupt("not an ownpurse log (no format)".into()))?;
    if format != FORMAT {
        return Err(StoreError::Corrupt(format!(
            "unknown log format {format:?}"
        )));
    }
    let mut datoms: Vec<Datom> = Vec::new();
    let mut st = conn.prepare("SELECT e, a, v, vt, tx, added FROM datom ORDER BY tx, rowid")?;
    let rows = st.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, i64>(1)?,
            r.get::<_, rusqlite::types::Value>(2)?,
            r.get::<_, u8>(3)?,
            r.get::<_, i64>(4)?,
            r.get::<_, bool>(5)?,
        ))
    })?;
    for row in rows {
        let (e, a, v, vt, tx, added): (i64, i64, rusqlite::types::Value, u8, i64, bool) = row?;
        let v = sql::from_sql(&v, vt).map_err(StoreError::Corrupt)?;
        datoms.push(Datom {
            e: sql::eid(e)?,
            a: sql::eid(a)?,
            v,
            tx: sql::eid(tx)?,
            added,
        });
    }
    let mut head = ZERO_HASH;
    let mut st = conn.prepare("SELECT t, prev, hash FROM tx ORDER BY t")?;
    let txs: Vec<(i64, String, String)> = st
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
        .collect::<Result<_, _>>()?;
    let mut i = 0;
    for (t, prev, hash) in txs {
        let t = sql::eid(t)?;
        let start = i;
        while i < datoms.len() && datoms[i].tx == t {
            i += 1;
        }
        if prev != head.hex() {
            return Err(StoreError::Corrupt(format!(
                "transaction {t}: the chain is broken (prev does not match)"
            )));
        }
        let got = tx_hash(&head, t, &datoms[start..i]);
        if got.hex() != hash {
            return Err(StoreError::Corrupt(format!(
                "transaction {t}: its datoms do not match its hash"
            )));
        }
        head = got;
    }
    if i != datoms.len() {
        return Err(StoreError::Corrupt("datoms without a transaction".into()));
    }
    let db = Db::from_log(datoms).map_err(StoreError::Corrupt)?;
    Ok((db, head))
}
