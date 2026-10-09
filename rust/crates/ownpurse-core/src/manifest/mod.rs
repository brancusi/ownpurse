//! Manifests (notes/MANIFEST-DIFF-DESIGN.md): every proposed change to the books is ONE Xero document
//! before → after, stored as an RFC 6902 JSON Patch against the NORMALISED "before" ([`normalise`]). A
//! manifest is an ordered list of such entries; it is reviewed as a diff, CHECKED locally ([`check`]), and
//! compiled into the sealed write package ([`compile`]) that the Phase 8 write path approves and applies.
//!
//! The store, `db/manifests.sqlite`, is one append-only, SHA-256-chained `events` table (the classify-store
//! pattern). Every event names its manifest; a manifest's state is materialised from its events: undo is a
//! toggle event, an edit is a new entry that replaces an older one, discarding is a marker. Nothing here
//! calls Xero, and nothing here applies anything: there is no "apply" event (the record says what landed).
//!
//! The view of main + a manifest (the ledger, TB, intercompany and every list) is [`crate::view::View`].

pub mod check;
pub mod compile;
pub mod effects;
pub mod fields;
pub mod normalise;
pub mod ops;
pub mod rules;
pub mod stage;

use rusqlite::OptionalExtension;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};

/// A manifest error: a code (`usage`, `not_found`, `conflict`, `precondition`, `store`, `integrity`,
/// `forbidden`, `view_changed`) and a message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestError {
    pub code: &'static str,
    pub message: String,
}

impl ManifestError {
    pub fn new(code: &'static str, message: impl Into<String>) -> ManifestError {
        ManifestError {
            code,
            message: message.into(),
        }
    }
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for ManifestError {}

impl From<rusqlite::Error> for ManifestError {
    fn from(e: rusqlite::Error) -> Self {
        ManifestError::new("store", e.to_string())
    }
}

impl From<crate::record::RecordError> for ManifestError {
    fn from(e: crate::record::RecordError) -> Self {
        ManifestError::new("record", e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ManifestError>;

// ── events ───────────────────────────────────────────────────────────────────────────────────────

/// Event kinds (the `kind` column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    Create,
    EntryAdd,
    EntryUndo,
    EntryAnnotate,
    /// A warning acknowledged (or the acknowledgement withdrawn).
    IssueAck,
    /// How a reconciled bank line's recode is carried out (`reclass` or `browser`), for one entry or, with
    /// no `seq`, as the manifest's default.
    Resolve,
    /// `manifest plan` sealed a package from this manifest (its path and hash).
    Planned,
    Discard,
}

impl Kind {
    pub const ALL: [Kind; 8] = [
        Kind::Create,
        Kind::EntryAdd,
        Kind::EntryUndo,
        Kind::EntryAnnotate,
        Kind::IssueAck,
        Kind::Resolve,
        Kind::Planned,
        Kind::Discard,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Create => "manifest.create",
            Kind::EntryAdd => "entry.add",
            Kind::EntryUndo => "entry.undo",
            Kind::EntryAnnotate => "entry.annotate",
            Kind::IssueAck => "issue.ack",
            Kind::Resolve => "entry.resolve",
            Kind::Planned => "manifest.planned",
            Kind::Discard => "manifest.discard",
        }
    }
    pub fn parse(s: &str) -> Option<Kind> {
        Kind::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

/// One stored event.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: i64,
    pub at: String,
    /// `human` or `agent:<name>`.
    pub actor: String,
    pub manifest: String,
    pub kind: Kind,
    pub payload: Value,
    pub prev_hash: String,
    pub hash: String,
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS events (
  id INTEGER PRIMARY KEY,
  at TEXT NOT NULL,
  actor TEXT NOT NULL,
  manifest TEXT NOT NULL,
  kind TEXT NOT NULL,
  payload_json TEXT NOT NULL,
  prev_hash TEXT NOT NULL,
  hash TEXT NOT NULL
);
CREATE TRIGGER IF NOT EXISTS events_no_update BEFORE UPDATE ON events
  BEGIN SELECT RAISE(ABORT, 'events is append-only'); END;
CREATE TRIGGER IF NOT EXISTS events_no_delete BEFORE DELETE ON events
  BEGIN SELECT RAISE(ABORT, 'events is append-only'); END;
";

const GENESIS: &str = "0000000000000000000000000000000000000000000000000000000000000000";

/// `sha256(prev_hash + canonical({id, at, actor, manifest, kind, payload}))`.
pub fn event_hash(
    prev: &str,
    id: i64,
    at: &str,
    actor: &str,
    manifest: &str,
    kind: &str,
    payload: &Value,
) -> String {
    let body = json!({
        "id": id, "at": at, "actor": actor, "manifest": manifest, "kind": kind, "payload": payload,
    });
    crate::write::sha256_hex(format!("{prev}{}", crate::canon::canonical(&body)).as_bytes())
}

/// The actor of a write: `human`, or `agent:<name>`.
pub fn actor_ok(a: &str) -> bool {
    a == "human" || a.strip_prefix("agent:").is_some_and(|r| !r.is_empty())
}

/// A manifest name: a slug of lowercase letters, digits and `-` (1–40), not `main`, not starting with `-`.
pub fn name_ok(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 40
        && name != "main"
        && !name.starts_with('-')
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

// ── the store ────────────────────────────────────────────────────────────────────────────────────

/// The open store. Lazily created: a missing file reads as empty; the first append creates it.
pub struct Store {
    path: PathBuf,
    conn: Option<rusqlite::Connection>,
    events: Vec<Event>,
}

/// An exclusive `flock` on the store's folder (one writer at a time; see the classify store for why the
/// folder and not the file).
struct WriteLock(std::fs::File);

impl WriteLock {
    fn take(dir: &Path) -> Result<WriteLock> {
        let f = std::fs::OpenOptions::new()
            .read(true)
            .open(dir)
            .map_err(|e| ManifestError::new("store", format!("{}: {e}", dir.display())))?;
        use std::os::fd::AsRawFd;
        // SAFETY: a valid open descriptor; LOCK_EX blocks until the other holder releases it.
        if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(ManifestError::new(
                "store",
                format!(
                    "lock {}: {}",
                    dir.display(),
                    std::io::Error::last_os_error()
                ),
            ));
        }
        Ok(WriteLock(f))
    }
}

impl Drop for WriteLock {
    fn drop(&mut self) {
        use std::os::fd::AsRawFd;
        // SAFETY: the descriptor is still open (we own the File).
        unsafe {
            libc::flock(self.0.as_raw_fd(), libc::LOCK_UN);
        }
    }
}

impl Store {
    /// Open without creating anything.
    pub fn open(path: &Path) -> Result<Store> {
        crate::record::sqlite_ready();
        let mut s = Store::empty(path);
        if path.exists() {
            s.connect()?;
        }
        s.refresh()?;
        Ok(s)
    }

    /// An empty store for a path that must not be created (`--state` renders).
    pub fn empty(path: &Path) -> Store {
        Store {
            path: path.to_path_buf(),
            conn: None,
            events: vec![],
        }
    }

    fn connect(&mut self) -> Result<()> {
        let conn = rusqlite::Connection::open(&self.path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA journal_mode=DELETE;")?;
        conn.execute_batch(SCHEMA)?;
        self.conn = Some(conn);
        Ok(())
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    pub fn exists(&self) -> bool {
        self.conn.is_some()
    }
    /// The last event id (0 when empty).
    pub fn rev(&self) -> i64 {
        self.events.last().map_or(0, |e| e.id)
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }

    /// Append one event under the writer lock; returns its id.
    pub fn append(
        &mut self,
        actor: &str,
        manifest: &str,
        kind: Kind,
        payload: Value,
    ) -> Result<i64> {
        if !actor_ok(actor) {
            return Err(ManifestError::new(
                "usage",
                format!("actor '{actor}': expected human or agent:<name>"),
            ));
        }
        let dir = match self.path.parent() {
            Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
            _ => PathBuf::from("."),
        };
        std::fs::create_dir_all(&dir)
            .map_err(|e| ManifestError::new("store", format!("{}: {e}", dir.display())))?;
        if self.conn.is_none() {
            self.connect()?;
        }
        let _lock = WriteLock::take(&dir)?;
        let conn = self.conn.as_mut().expect("connected above");
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let (mut id, prev): (i64, String) = tx
            .query_row(
                "SELECT id, hash FROM events ORDER BY id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .unwrap_or((0, GENESIS.to_string()));
        id += 1;
        let at = crate::classify::store::now_iso();
        let hash = event_hash(&prev, id, &at, actor, manifest, kind.as_str(), &payload);
        tx.execute(
            "INSERT INTO events (id, at, actor, manifest, kind, payload_json, prev_hash, hash) \
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
            rusqlite::params![
                id,
                at,
                actor,
                manifest,
                kind.as_str(),
                crate::canon::canonical(&payload),
                prev,
                hash
            ],
        )?;
        tx.commit()?;
        self.refresh()?;
        Ok(id)
    }

    /// Re-read events appended by other writers; true if anything changed.
    pub fn refresh(&mut self) -> Result<bool> {
        if self.conn.is_none() {
            if !self.path.exists() {
                return Ok(false);
            }
            self.connect()?;
        }
        let Some(conn) = self.conn.as_ref() else {
            return Ok(false);
        };
        let rev = self.rev();
        let mut st = conn.prepare_cached(
            "SELECT id, at, actor, manifest, kind, payload_json, prev_hash, hash FROM events \
             WHERE id > ?1 ORDER BY id",
        )?;
        let rows = st.query_map([rev], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
            ))
        })?;
        let mut n = 0;
        for row in rows {
            let (id, at, actor, manifest, kind, text, prev_hash, hash) = row?;
            let kind = Kind::parse(&kind).ok_or_else(|| {
                ManifestError::new("integrity", format!("event {id}: unknown kind '{kind}'"))
            })?;
            let payload: Value = serde_json::from_str(&text).map_err(|e| {
                ManifestError::new("integrity", format!("event {id}: bad JSON: {e}"))
            })?;
            self.events.push(Event {
                id,
                at,
                actor,
                manifest,
                kind,
                payload,
                prev_hash,
                hash,
            });
            n += 1;
        }
        Ok(n > 0)
    }

    /// Prove the chain. Returns the number of events checked.
    pub fn verify(&self) -> Result<usize> {
        let mut prev = GENESIS.to_string();
        for (i, e) in self.events.iter().enumerate() {
            if e.id != i as i64 + 1 {
                return Err(ManifestError::new(
                    "integrity",
                    format!("event ids not dense at {}", e.id),
                ));
            }
            if e.prev_hash != prev {
                return Err(ManifestError::new(
                    "integrity",
                    format!("chain broken at event {}", e.id),
                ));
            }
            let want = event_hash(
                &prev,
                e.id,
                &e.at,
                &e.actor,
                &e.manifest,
                e.kind.as_str(),
                &e.payload,
            );
            if want != e.hash {
                return Err(ManifestError::new(
                    "integrity",
                    format!("chain hash wrong at event {}", e.id),
                ));
            }
            prev = e.hash.clone();
        }
        Ok(self.events.len())
    }

    /// Every manifest, in creation order.
    pub fn manifests(&self) -> Vec<Manifest> {
        materialise(&self.events)
    }

    /// One manifest by name (open or discarded).
    pub fn manifest(&self, name: &str) -> Option<Manifest> {
        self.manifests().into_iter().find(|b| b.name == name)
    }

    /// An open manifest, or `not_found` / `usage` (discarded).
    pub fn open_manifest(&self, name: &str) -> Result<Manifest> {
        match self.manifest(name) {
            None => Err(ManifestError::new(
                "not_found",
                format!("No manifest named `{name}` (ownpurse manifest list)."),
            )),
            Some(m) if m.discarded => Err(ManifestError::new(
                "usage",
                format!("Manifest `{name}` is discarded: it is read-only history."),
            )),
            Some(m) => Ok(m),
        }
    }
}

// ── the model ────────────────────────────────────────────────────────────────────────────────────

/// References an entry rests on.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Refs {
    /// The accountant's decision (an adjusting-entry id, an email, a `decisions` row).
    pub accountant_decision: Option<String>,
    /// Source documents (statement rows, bank lines).
    pub sources: Vec<String>,
    /// The classify label behind it, if any.
    pub classify: Option<String>,
}

impl Refs {
    pub fn to_json(&self) -> Value {
        let mut m = serde_json::Map::new();
        if let Some(a) = &self.accountant_decision {
            m.insert("accountant_decision".into(), json!(a));
        }
        if !self.sources.is_empty() {
            m.insert("sources".into(), json!(self.sources));
        }
        if let Some(c) = &self.classify {
            m.insert("classify".into(), json!(c));
        }
        Value::Object(m)
    }
    pub fn from_json(v: &Value) -> Refs {
        Refs {
            accountant_decision: v["accountant_decision"].as_str().map(str::to_string),
            sources: v["sources"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|x| x.as_str().map(str::to_string))
                        .collect()
                })
                .unwrap_or_default(),
            classify: v["classify"].as_str().map(str::to_string),
        }
    }
}

/// One entry: a document before → after (design §1).
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    /// The `entry.add` event id: the entry's number.
    pub seq: i64,
    pub at: String,
    pub actor: String,
    /// `ManualJournals`, `BankTransactions`, … (the record's entity names).
    pub entity: String,
    /// The org alias.
    pub org: String,
    /// The Xero id, or `new:<seq>` for a document the manifest creates.
    pub id: String,
    /// The fingerprint of the normalised "before" when recorded (None: a new document).
    pub before_fp: Option<String>,
    /// The fingerprint of the normalised "after" (main equal to it means the change landed: posted).
    pub after_fp: Option<String>,
    /// The write path's business fingerprint of "after" in Xero's shape (insensitive to the list / by-id
    /// shapes a read-back may come in): main equal to it also means posted.
    pub after_bfp: Option<String>,
    /// RFC 6902 operations against the normalised "before" (or `{}` for a new document).
    pub patch: Value,
    pub why: String,
    pub refs: Refs,
    pub note: Option<String>,
    /// The entry this one edits; that one shows as "edited → this".
    pub replaces: Option<i64>,
    pub replaced_by: Option<i64>,
    pub undone: bool,
    /// A recode of a RECONCILED bank line (Xero refuses API edits of those): `reclass` (a reclass journal, the
    /// default) or `browser` (unreconcile → edit → re-reconcile, a checklist step). None: not chosen.
    pub resolution: Option<String>,
}

impl Entry {
    /// Part of the manifest (not undone, not replaced by an edit).
    pub fn active(&self) -> bool {
        !self.undone && self.replaced_by.is_none()
    }
    pub fn by_agent(&self) -> bool {
        self.actor.starts_with("agent:")
    }
    pub fn is_new(&self) -> bool {
        self.before_fp.is_none()
    }
    /// The patch's operations.
    pub fn ops(&self) -> Vec<json_patch::PatchOperation> {
        serde_json::from_value::<json_patch::Patch>(self.patch.clone())
            .map(|p| p.0)
            .unwrap_or_default()
    }
    /// The paths the patch touches (`/JournalLines/n2/AccountCode`, …).
    pub fn paths(&self) -> Vec<String> {
        self.ops().iter().map(|o| o.path().to_string()).collect()
    }
    /// "after" = patch(before). `before` is the normalised current document (None for a new one).
    pub fn after(&self, before: Option<&Value>) -> Result<Value> {
        let mut doc = before.cloned().unwrap_or_else(|| json!({}));
        json_patch::patch(&mut doc, &self.ops()).map_err(|e| {
            ManifestError::new(
                "patch",
                format!("entry {}: the patch no longer applies ({e})", self.seq),
            )
        })?;
        Ok(doc)
    }
}

/// An acknowledged warning: `(entry seq, issue code, path)`, by whom and when.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ack {
    pub seq: i64,
    pub code: String,
    pub path: String,
    pub actor: String,
    pub at: String,
}

/// A package sealed from a manifest.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Planned {
    /// The `manifest.planned` event id.
    pub rev: i64,
    pub package: String,
    pub hash: String,
    /// The manifest head the package was compiled from (a later entry event makes the plan stale).
    pub from_rev: i64,
}

/// One manifest.
#[derive(Debug, Clone, PartialEq)]
pub struct Manifest {
    pub name: String,
    pub why: String,
    /// The record version when created.
    pub base: i64,
    pub created_by: String,
    pub created_at: String,
    pub discarded: bool,
    pub entries: Vec<Entry>,
    pub acks: Vec<Ack>,
    pub planned: Option<Planned>,
    /// The manifest's default resolution for reconciled-line recodes (`reclass` unless set otherwise).
    pub default_resolution: String,
    /// The last event that touched it.
    pub head_rev: i64,
    /// The last event that changed its entries (adds, undos, annotations): what a plan is compiled from.
    pub content_rev: i64,
}

impl Manifest {
    pub fn entry(&self, seq: i64) -> Option<&Entry> {
        self.entries.iter().find(|c| c.seq == seq)
    }
    pub fn active(&self) -> impl Iterator<Item = &Entry> {
        self.entries.iter().filter(|c| c.active())
    }
    /// The active entry on a document, if any.
    pub fn on_doc(&self, org: &str, entity: &str, id: &str) -> Option<&Entry> {
        self.active()
            .find(|e| e.org == org && e.entity == entity && e.id == id)
    }
    pub fn acked(&self, seq: i64, code: &str, path: &str) -> Option<&Ack> {
        self.acks
            .iter()
            .find(|a| a.seq == seq && a.code == code && a.path == path)
    }
    /// The current plan, if the manifest hasn't changed since it was sealed.
    pub fn plan(&self) -> Option<&Planned> {
        self.planned
            .as_ref()
            .filter(|p| p.from_rev >= self.content_rev)
    }
}

fn materialise(events: &[Event]) -> Vec<Manifest> {
    let mut out: Vec<Manifest> = vec![];
    for e in events {
        if e.kind == Kind::Create {
            out.push(Manifest {
                name: e.manifest.clone(),
                why: e.payload["why"].as_str().unwrap_or("").to_string(),
                base: e.payload["base"].as_i64().unwrap_or(0),
                created_by: e.actor.clone(),
                created_at: e.at.clone(),
                discarded: false,
                entries: vec![],
                acks: vec![],
                planned: None,
                default_resolution: "reclass".into(),
                head_rev: e.id,
                content_rev: e.id,
            });
            continue;
        }
        let Some(m) = out.iter_mut().rev().find(|b| b.name == e.manifest) else {
            continue;
        };
        m.head_rev = e.id;
        let p = &e.payload;
        match e.kind {
            Kind::EntryAdd => {
                m.content_rev = e.id;
                let replaces = p["replaces"].as_i64();
                if let Some(r) = replaces
                    && let Some(old) = m.entries.iter_mut().find(|c| c.seq == r)
                {
                    old.replaced_by = Some(e.id);
                }
                let id = p["id"].as_str().unwrap_or("").to_string();
                m.entries.push(Entry {
                    seq: e.id,
                    at: e.at.clone(),
                    actor: e.actor.clone(),
                    entity: p["entity"].as_str().unwrap_or("").to_string(),
                    org: p["org"].as_str().unwrap_or("").to_string(),
                    id: if id.is_empty() || id == "new" {
                        format!("new:{}", e.id)
                    } else {
                        id
                    },
                    before_fp: p["before_fp"].as_str().map(str::to_string),
                    after_fp: p["after_fp"].as_str().map(str::to_string),
                    after_bfp: p["after_bfp"].as_str().map(str::to_string),
                    patch: p["patch"].clone(),
                    why: p["why"].as_str().unwrap_or("").to_string(),
                    refs: Refs::from_json(&p["refs"]),
                    note: p["note"].as_str().map(str::to_string),
                    replaces,
                    replaced_by: None,
                    undone: false,
                    resolution: p["resolution"].as_str().map(str::to_string),
                });
            }
            Kind::EntryUndo => {
                m.content_rev = e.id;
                let seq = p["seq"].as_i64().unwrap_or(0);
                let undone = p["undone"].as_bool().unwrap_or(true);
                if let Some(c) = m.entries.iter_mut().find(|c| c.seq == seq) {
                    c.undone = undone;
                }
            }
            Kind::EntryAnnotate => {
                m.content_rev = e.id;
                let seq = p["seq"].as_i64().unwrap_or(0);
                if let Some(c) = m.entries.iter_mut().find(|c| c.seq == seq) {
                    if let Some(a) = p.get("accountant").and_then(Value::as_str) {
                        c.refs.accountant_decision = Some(a.to_string()).filter(|s| !s.is_empty());
                    }
                    if let Some(n) = p.get("note").and_then(Value::as_str) {
                        c.note = Some(n.to_string()).filter(|s| !s.is_empty());
                    }
                }
            }
            Kind::IssueAck => {
                let seq = p["seq"].as_i64().unwrap_or(0);
                let code = p["code"].as_str().unwrap_or("").to_string();
                let path = p["path"].as_str().unwrap_or("").to_string();
                m.acks
                    .retain(|a| !(a.seq == seq && a.code == code && a.path == path));
                if p["acked"].as_bool().unwrap_or(true) {
                    m.acks.push(Ack {
                        seq,
                        code,
                        path,
                        actor: e.actor.clone(),
                        at: e.at.clone(),
                    });
                }
            }
            Kind::Planned => {
                m.planned = Some(Planned {
                    rev: e.id,
                    package: p["package"].as_str().unwrap_or("").to_string(),
                    hash: p["hash"].as_str().unwrap_or("").to_string(),
                    from_rev: p["from_rev"].as_i64().unwrap_or(0),
                });
            }
            Kind::Resolve => {
                m.content_rev = e.id;
                let r = p["resolution"].as_str().unwrap_or("reclass").to_string();
                match p["seq"].as_i64() {
                    Some(seq) => {
                        if let Some(c) = m.entries.iter_mut().find(|c| c.seq == seq) {
                            c.resolution = Some(r);
                        }
                    }
                    None => m.default_resolution = r,
                }
            }
            Kind::Discard => m.discarded = true,
            Kind::Create => {}
        }
    }
    out
}

// ── building entries ─────────────────────────────────────────────────────────────────────────────

/// The patch that turns normalised `before` (or `{}`) into normalised `after`.
pub fn diff(before: Option<&Value>, after: &Value) -> Value {
    let empty = json!({});
    serde_json::to_value(json_patch::diff(before.unwrap_or(&empty), after))
        .unwrap_or_else(|_| json!([]))
}

/// The `entry.add` payload for a document change. `before` is the document as main holds it now (raw, as
/// in the record; None for a new document), `after` the wanted document (raw or normalised).
#[allow(clippy::too_many_arguments)]
pub fn entry_payload(
    entity: &str,
    org: &str,
    id: Option<&str>,
    before: Option<&Value>,
    after: &Value,
    why: &str,
    refs: &Refs,
    replaces: Option<i64>,
) -> Value {
    let b = before.map(normalise::normalise);
    let a = normalise::normalise(after);
    json!({
        "entity": entity,
        "org": org,
        "id": id.unwrap_or("new"),
        "before_fp": b.as_ref().map(normalise::fingerprint),
        "after_fp": normalise::fingerprint(&a),
        "after_bfp": crate::write::fingerprint(&normalise::denormalise(&a, b.as_ref())),
        "patch": diff(b.as_ref(), &a),
        "why": why,
        "refs": refs.to_json(),
        "replaces": replaces,
    })
}

// ── paths and the current manifest ────────────────────────────────────────────────────────────────

/// Where the manifest store and the current-manifest file live (`manifests.store`, default
/// `db/manifests.sqlite` beside the config; `manifests.current` next to the store).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub store: PathBuf,
    pub current: PathBuf,
    /// Where `manifest plan` writes packages (`db/packages/`).
    pub packages: PathBuf,
}

impl Paths {
    pub fn resolve(config: &crate::config::Config) -> Paths {
        let v = config
            .settings
            .get_str("manifests.store")
            .unwrap_or("db/manifests.sqlite");
        let p = Path::new(v);
        let store = if p.is_absolute() {
            p.to_path_buf()
        } else {
            config.root.join(p)
        };
        Paths::beside(store)
    }

    pub fn beside(store: PathBuf) -> Paths {
        let current = store.with_file_name("manifests.current");
        let packages = store.with_file_name("packages");
        Paths {
            store,
            current,
            packages,
        }
    }

    /// The current manifest name (None = main or no file). Never writes.
    pub fn current(&self) -> Option<String> {
        std::fs::read_to_string(&self.current)
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "main")
    }

    /// Make a manifest (or main) the default for the CLI. Writes `manifests.current` beside the store; a
    /// call that would only write "main" where no file exists writes nothing.
    pub fn set_current(&self, name: Option<&str>) -> Result<()> {
        if name.is_none() && !self.current.exists() {
            return Ok(());
        }
        if let Some(dir) = self.current.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)
                .map_err(|e| ManifestError::new("store", format!("{}: {e}", dir.display())))?;
        }
        let tmp = self.current.with_extension("current.tmp");
        std::fs::write(&tmp, format!("{}\n", name.unwrap_or("main")))
            .and_then(|_| std::fs::rename(&tmp, &self.current))
            .map_err(|e| ManifestError::new("store", format!("{}: {e}", self.current.display())))
    }
}

fn setting<'c>(config: &'c crate::config::Config, key: &str) -> Option<&'c Value> {
    config
        .settings
        .get(&format!("manifests.{key}"))
        .or_else(|| config.settings.get(&format!("branches.{key}")))
}

/// The accountant gate (`manifests.accountant_gate: [{org, through}]` in the project config; the older
/// `branches.accountant_gate` is read too): entries on a gated org dated on or before `through` need an
/// accountant decision reference before planning.
pub fn accountant_gate(config: &crate::config::Config) -> Vec<(String, String)> {
    setting(config, "accountant_gate")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|g| {
                    Some((
                        g.get("org")?.as_str()?.to_string(),
                        g.get("through")?.as_str()?.to_string(),
                    ))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The must-be-zero accounts beyond suspense (`manifests.zero_accounts: {org: [codes]}`).
pub fn zero_accounts(
    config: &crate::config::Config,
) -> std::collections::BTreeMap<String, Vec<String>> {
    setting(config, "zero_accounts")
        .and_then(Value::as_object)
        .map(|m| {
            m.iter()
                .map(|(k, v)| {
                    let codes = v
                        .as_array()
                        .map(|a| {
                            a.iter()
                                .filter_map(|x| x.as_str().map(str::to_string))
                                .collect()
                        })
                        .unwrap_or_default();
                    (k.clone(), codes)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Xero's per-journal line limit, once measured (`manifests.journal_line_limit`; None: no split warning).
pub fn journal_line_limit(config: &crate::config::Config) -> Option<usize> {
    setting(config, "journal_line_limit")
        .and_then(Value::as_u64)
        .map(|n| n as usize)
}

pub use normalise::{denormalise, fingerprint};

/// One line per patch operation, for audit (the `J` raw-patch view and the CLI):
/// `replace /JournalLines/n2/AccountCode  "6140" → "5020"`. `before` is the normalised "before".
pub fn patch_lines(entry: &Entry, before: Option<&Value>) -> Vec<String> {
    use json_patch::PatchOperation as P;
    let show = |v: &Value| crate::canon::canonical(v);
    entry
        .ops()
        .iter()
        .map(|o| {
            let path = o.path().to_string();
            let old = before.and_then(|b| b.pointer(&path));
            match o {
                P::Add(a) => format!("add {path}  {}", show(&a.value)),
                P::Remove(_) => format!(
                    "remove {path}  {}",
                    old.map(show).unwrap_or_else(|| "—".into())
                ),
                P::Replace(r) => format!(
                    "replace {path}  {} → {}",
                    old.map(show).unwrap_or_else(|| "—".into()),
                    show(&r.value)
                ),
                P::Move(m) => format!("move {} → {path}", m.from),
                P::Copy(c) => format!("copy {} → {path}", c.from),
                P::Test(t) => format!("test {path}  {}", show(&t.value)),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("xam-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d.join("db/manifests.sqlite")
    }

    #[test]
    fn store_is_chained_and_materialises() {
        let p = tmp("chain");
        let mut s = Store::open(&p).unwrap();
        assert!(!p.exists(), "opening never creates");
        s.append("human", "dec", Kind::Create, json!({"why": "w", "base": 3}))
            .unwrap();
        let before = json!({"ManualJournalID": "mj-1", "Status": "POSTED"});
        let mut after = before.clone();
        after["Status"] = json!("VOIDED");
        let e = s
            .append(
                "agent:bot",
                "dec",
                Kind::EntryAdd,
                entry_payload(
                    "ManualJournals",
                    "lab",
                    Some("mj-1"),
                    Some(&before),
                    &after,
                    "wrong",
                    &Refs::default(),
                    None,
                ),
            )
            .unwrap();
        s.append(
            "human",
            "dec",
            Kind::IssueAck,
            json!({"seq": e, "code": "x", "path": "/Status"}),
        )
        .unwrap();
        assert_eq!(s.verify().unwrap(), 3);
        let m = s.open_manifest("dec").unwrap();
        let en = m.entry(e).unwrap();
        assert!(en.by_agent() && en.active() && !en.is_new());
        assert_eq!(en.paths(), vec!["/Status".to_string()]);
        let after_n = en.after(Some(&normalise::normalise(&before))).unwrap();
        assert_eq!(after_n["Status"], "VOIDED");
        assert!(m.acked(e, "x", "/Status").is_some());
        assert!(s.append("nobody", "dec", Kind::Discard, json!({})).is_err());
        // a second reader sees the same chain
        let s2 = Store::open(&p).unwrap();
        assert_eq!(s2.rev(), 3);
        assert!(name_ok("dec-cleanup") && !name_ok("main") && !name_ok("Bad"));
    }

    #[test]
    fn set_current_main_without_a_file_writes_nothing() {
        let p = Paths::beside(tmp("cur"));
        p.set_current(None).unwrap();
        assert!(!p.current.exists());
        assert_eq!(p.current(), None);
    }
}
