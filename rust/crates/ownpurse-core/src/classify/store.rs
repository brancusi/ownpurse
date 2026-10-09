//! The classification store, `db/classify.sqlite`: one append-only, SHA-256-chained `events` table (the only
//! table written) and the materialised reads rebuilt from it (labels, suggestions, notes, groups).
//!
//! Every event carries `actor` and `batch` (one send). Undo appends an `undo` event naming the event ids it
//! reverses (V15); materialising skips reversed events, so the previous value comes back. An undo can itself
//! be undone. One writer at a time: appends take an exclusive `flock` on the database file, append inside
//! `BEGIN IMMEDIATE`, then re-read anything other writers appended.

use super::anchor::{Anchor, Fingerprint};
use super::{Actor, BatchId, ClassifyError, Confidence, Evidence, Result, Source};
use rusqlite::OptionalExtension;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Event kinds (the `kind` column).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    LabelSet,
    LabelCleared,
    NoteAdded,
    GroupCreated,
    GroupMemberAdded,
    GroupMemberRemoved,
    GroupStatus,
    Suggestion,
    ConfigWritten,
    Undo,
}

impl EventKind {
    pub const ALL: [EventKind; 10] = [
        EventKind::LabelSet,
        EventKind::LabelCleared,
        EventKind::NoteAdded,
        EventKind::GroupCreated,
        EventKind::GroupMemberAdded,
        EventKind::GroupMemberRemoved,
        EventKind::GroupStatus,
        EventKind::Suggestion,
        EventKind::ConfigWritten,
        EventKind::Undo,
    ];
    pub fn as_str(self) -> &'static str {
        match self {
            EventKind::LabelSet => "label_set",
            EventKind::LabelCleared => "label_cleared",
            EventKind::NoteAdded => "note_added",
            EventKind::GroupCreated => "group_created",
            EventKind::GroupMemberAdded => "group_member_added",
            EventKind::GroupMemberRemoved => "group_member_removed",
            EventKind::GroupStatus => "group_status",
            EventKind::Suggestion => "suggestion",
            EventKind::ConfigWritten => "config_written",
            EventKind::Undo => "undo",
        }
    }
    pub fn parse(s: &str) -> Option<EventKind> {
        EventKind::ALL.into_iter().find(|k| k.as_str() == s)
    }
}

/// One stored event.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub id: i64,
    pub at: String,
    pub actor: Actor,
    pub batch: BatchId,
    pub kind: EventKind,
    pub payload: Value,
    pub prev_hash: String,
    pub hash: String,
}

/// A confirmed label (current value per anchor and label type).
#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub anchor: Anchor,
    pub label_type: String,
    pub value: String,
    /// Where the value came from: `human`, `agent:<name>`, or `rule:<id>` when a rule suggestion was accepted.
    pub source: Source,
    /// Who confirmed it.
    pub actor: Actor,
    pub confidence: Option<Confidence>,
    pub evidence: Vec<Evidence>,
    /// The line as it was when the label was set (V19).
    pub fingerprint: Fingerprint,
    pub at: String,
    pub event_id: i64,
    /// The send's caption, if any (`A claude · 14:02 · "reclassify these as Harvest Blend"`).
    pub caption: Option<String>,
}

/// The latest suggestion per anchor and label type (not a label until confirmed).
#[derive(Debug, Clone, PartialEq)]
pub struct Suggestion {
    pub anchor: Anchor,
    pub label_type: String,
    pub value: String,
    pub source: Source,
    pub confidence: Confidence,
    pub evidence: Vec<Evidence>,
    pub fingerprint: Fingerprint,
    pub event_id: i64,
}

/// What a note is attached to.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum NoteTarget {
    Line(Anchor),
    Group(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Note {
    pub event_id: i64,
    pub target: NoteTarget,
    pub text: String,
    pub actor: Actor,
    pub at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Member {
    pub anchor: Anchor,
    /// `origin`, `mirror`, `settlement` (from `[groups].roles`) or none.
    pub role: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// `g<event id of group_created>`.
    pub id: String,
    pub kind: String,
    pub title: String,
    /// `open`, `complete`, `gap` or `accountant` (a question for the accountant).
    pub status: String,
    pub members: Vec<Member>,
    pub actor: Actor,
    pub at: String,
}

/// Group statuses.
pub const GROUP_STATUSES: [&str; 4] = ["open", "complete", "gap", "accountant"];

/// The materialised reads.
#[derive(Debug, Clone, Default)]
pub struct State {
    pub labels: HashMap<(Anchor, String), Label>,
    pub suggestions: HashMap<(Anchor, String), Suggestion>,
    pub notes: Vec<Note>,
    pub groups: BTreeMap<String, Group>,
    /// Active (not undone) write events per anchor, oldest first (for "undo the last write on this line").
    pub writes_by_anchor: HashMap<Anchor, Vec<i64>>,
}

static PINNED_NOW: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

/// A pinned clock (tests, fixtures): `Some("2026-10-05T09:46:00Z")`, or `None` for the wall clock.
pub fn pin_clock(at: Option<&str>) {
    *PINNED_NOW.lock().unwrap_or_else(|e| e.into_inner()) = at.map(str::to_string);
}

/// Now as `YYYY-MM-DDTHH:MM:SS.ffffffZ`, or the pinned clock.
pub fn now_iso() -> String {
    if let Some(at) = PINNED_NOW.lock().unwrap_or_else(|e| e.into_inner()).clone() {
        return at;
    }
    // the TUI's clock convention: OWNPURSE_NOW pins it (tests, guide renders); `2026-10-05T09:46Z` or full ISO
    if let Ok(v) = std::env::var("OWNPURSE_NOW")
        && v.len() >= 17
        && crate::dates::parse_iso(&v[..10]).is_some()
        && v.ends_with('Z')
    {
        return match v.len() {
            17 => format!("{}:00.000000Z", &v[..16]),
            20 => format!("{}.000000Z", &v[..19]),
            _ => v,
        };
    }
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = d.as_secs() as i64;
    let (days, rem) = (secs.div_euclid(86400), secs.rem_euclid(86400));
    let (y, m, day) = crate::dates::civil_from_days(days);
    format!(
        "{y:04}-{m:02}-{day:02}T{:02}:{:02}:{:02}.{:06}Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60,
        d.subsec_micros()
    )
}

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS events (
  id INTEGER PRIMARY KEY,
  at TEXT NOT NULL,
  actor TEXT NOT NULL,
  batch TEXT NOT NULL,
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

/// The chained hash of one event: `sha256(prev_hash + canonical({id, at, actor, batch, kind, payload}))`.
pub fn event_hash(
    prev: &str,
    id: i64,
    at: &str,
    actor: &str,
    batch: &str,
    kind: &str,
    payload: &Value,
) -> String {
    use sha2::{Digest, Sha256};
    let body = serde_json::json!({
        "id": id, "at": at, "actor": actor, "batch": batch, "kind": kind, "payload": payload,
    });
    let text = format!("{prev}{}", crate::canon::canonical(&body));
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// The open store.
pub struct Store {
    path: PathBuf,
    /// `None` until the file exists (a lazily opened store creates it on the first append).
    conn: Option<rusqlite::Connection>,
    events: Vec<Event>,
    state: State,
}

/// An exclusive `flock` on the store's FOLDER, held until dropped (one writer at a time). Not on the database
/// file itself: on macOS `flock` and SQLite's `fcntl` locks interact, and closing a second descriptor of the
/// database would drop SQLite's own locks. Locking the folder writes no file (V14).
struct WriteLock(std::fs::File);

impl WriteLock {
    fn take(path: &Path) -> Result<WriteLock> {
        let f = std::fs::OpenOptions::new()
            .read(true)
            .open(path)
            .map_err(|e| ClassifyError::new("store", format!("{}: {e}", path.display())))?;
        use std::os::fd::AsRawFd;
        // SAFETY: a valid open descriptor; LOCK_EX blocks until the other holder releases it.
        if unsafe { libc::flock(f.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(ClassifyError::new(
                "store",
                format!(
                    "lock {}: {}",
                    path.display(),
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

fn integrity(msg: String) -> ClassifyError {
    ClassifyError::new("integrity", msg)
}

impl Store {
    /// Open (creating the file and its folder if missing). Calls `record::sqlite_ready()` first.
    pub fn open(path: &Path) -> Result<Store> {
        let mut s = Store::open_lazy(path)?;
        s.ensure()?;
        Ok(s)
    }

    /// Open without creating anything: a missing file reads as an empty store, and the file (and its folder)
    /// is created by the first append. Reads never write.
    pub fn open_lazy(path: &Path) -> Result<Store> {
        crate::record::sqlite_ready();
        let mut s = Store {
            path: path.to_path_buf(),
            conn: None,
            events: vec![],
            state: State::default(),
        };
        if path.exists() {
            s.connect()?;
        }
        s.refresh()?;
        Ok(s)
    }

    /// True once the store file exists (and is open).
    pub fn exists(&self) -> bool {
        self.conn.is_some()
    }

    fn connect(&mut self) -> Result<()> {
        let path = &self.path;
        let conn = rusqlite::Connection::open(path)?;
        conn.busy_timeout(std::time::Duration::from_secs(5))?;
        conn.execute_batch("PRAGMA journal_mode=DELETE;")?;
        conn.execute_batch(SCHEMA)?;
        self.conn = Some(conn);
        Ok(())
    }

    /// Create the file (and its folder) if missing.
    fn ensure(&mut self) -> Result<()> {
        if self.conn.is_some() {
            return Ok(());
        }
        if let Some(dir) = self.path.parent().filter(|d| !d.as_os_str().is_empty()) {
            std::fs::create_dir_all(dir)
                .map_err(|e| ClassifyError::new("store", format!("{}: {e}", dir.display())))?;
        }
        self.connect()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
    /// The last event id (0 when empty): the store revision.
    pub fn rev(&self) -> i64 {
        self.events.last().map_or(0, |e| e.id)
    }
    pub fn events(&self) -> &[Event] {
        &self.events
    }
    pub fn state(&self) -> &State {
        &self.state
    }
    pub fn event(&self, id: i64) -> Option<&Event> {
        // ids are dense from 1
        let i = usize::try_from(id).ok()?.checked_sub(1)?;
        self.events.get(i).filter(|e| e.id == id)
    }

    /// Append events atomically under the writer lock, then pick up anything other writers appended.
    /// Returns the new ids in order.
    pub fn append(
        &mut self,
        actor: &Actor,
        batch: &BatchId,
        items: Vec<(EventKind, Value)>,
    ) -> Result<Vec<i64>> {
        if items.is_empty() {
            return Ok(vec![]);
        }
        let dir = match self.path.parent() {
            Some(d) if !d.as_os_str().is_empty() => d.to_path_buf(),
            _ => PathBuf::from("."),
        };
        self.ensure()?;
        let _lock = WriteLock::take(&dir)?;
        let conn = self.conn.as_mut().expect("ensured above");
        let tx = conn.transaction_with_behavior(rusqlite::TransactionBehavior::Immediate)?;
        let (mut id, mut prev): (i64, String) = tx
            .query_row(
                "SELECT id, hash FROM events ORDER BY id DESC LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?
            .unwrap_or((0, GENESIS.to_string()));
        let at = now_iso();
        let mut ids = vec![];
        for (kind, payload) in items {
            id += 1;
            let text = crate::canon::canonical(&payload);
            let hash = event_hash(
                &prev,
                id,
                &at,
                actor.as_str(),
                &batch.0,
                kind.as_str(),
                &payload,
            );
            tx.execute(
                "INSERT INTO events (id, at, actor, batch, kind, payload_json, prev_hash, hash) \
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8)",
                rusqlite::params![
                    id,
                    at,
                    actor.as_str(),
                    batch.0,
                    kind.as_str(),
                    text,
                    prev,
                    hash
                ],
            )?;
            prev = hash;
            ids.push(id);
        }
        tx.commit()?;
        self.refresh()?;
        Ok(ids)
    }

    /// Re-read events appended by other writers; true if anything changed.
    pub fn refresh(&mut self) -> Result<bool> {
        if self.conn.is_none() {
            if !self.path.exists() {
                return Ok(false);
            }
            // another writer created the file since
            self.connect()?;
        }
        let Some(conn) = self.conn.as_ref() else {
            return Ok(false);
        };
        let rev = self.rev();
        let mut st = conn.prepare_cached(
            "SELECT id, at, actor, batch, kind, payload_json, prev_hash, hash FROM events \
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
        let mut new = vec![];
        for row in rows {
            let (id, at, actor, batch, kind, text, prev_hash, hash) = row?;
            let kind = EventKind::parse(&kind)
                .ok_or_else(|| integrity(format!("event {id}: unknown kind '{kind}'")))?;
            let payload: Value = serde_json::from_str(&text)
                .map_err(|e| integrity(format!("event {id}: bad JSON: {e}")))?;
            new.push(Event {
                id,
                at,
                actor: Actor::parse(&actor)
                    .map_err(|_| integrity(format!("event {id}: bad actor '{actor}'")))?,
                batch: BatchId(batch),
                kind,
                payload,
                prev_hash,
                hash,
            });
        }
        drop(st);
        if new.is_empty() {
            return Ok(false);
        }
        let has_undo = new.iter().any(|e| e.kind == EventKind::Undo);
        self.events.extend(new.iter().cloned());
        if has_undo {
            self.state = materialise(&self.events, &undone_set(&self.events));
        } else {
            for e in &new {
                apply(&mut self.state, e);
            }
        }
        Ok(true)
    }

    /// Ids of events that are currently reversed by an active undo.
    pub fn undone(&self) -> HashSet<i64> {
        undone_set(&self.events)
    }

    /// Prove the chain: every event links to the one before and still hashes to what was stored.
    /// Returns the number of events checked.
    pub fn verify(&self) -> Result<usize> {
        let Some(conn) = self.conn.as_ref() else {
            return Ok(0);
        };
        let mut st = conn.prepare(
            "SELECT id, at, actor, batch, kind, payload_json, prev_hash, hash FROM events ORDER BY id",
        )?;
        let mut rows = st.query([])?;
        let (mut prev, mut n, mut expect_id) = (GENESIS.to_string(), 0usize, 1i64);
        while let Some(r) = rows.next()? {
            let id: i64 = r.get(0)?;
            let text: String = r.get(5)?;
            let prev_hash: String = r.get(6)?;
            let hash: String = r.get(7)?;
            if id != expect_id {
                return Err(integrity(format!("event ids not dense at {id}")));
            }
            if prev_hash != prev {
                return Err(integrity(format!("chain broken at event {id} (prev hash)")));
            }
            let payload: Value = serde_json::from_str(&text)
                .map_err(|e| integrity(format!("event {id}: bad JSON: {e}")))?;
            let want = event_hash(
                &prev,
                id,
                &r.get::<_, String>(1)?,
                &r.get::<_, String>(2)?,
                &r.get::<_, String>(3)?,
                &r.get::<_, String>(4)?,
                &payload,
            );
            if want != hash {
                return Err(integrity(format!("chain hash wrong at event {id}")));
            }
            prev = hash;
            n += 1;
            expect_id += 1;
        }
        Ok(n)
    }
}

/// Events reversed by an active undo. Walk newest first: an undo that is itself undone does nothing.
fn undone_set(events: &[Event]) -> HashSet<i64> {
    let mut undone = HashSet::new();
    for e in events.iter().rev() {
        if e.kind == EventKind::Undo && !undone.contains(&e.id) {
            for id in undo_targets(&e.payload) {
                undone.insert(id);
            }
        }
    }
    undone
}

/// The event ids an undo event names.
pub fn undo_targets(payload: &Value) -> Vec<i64> {
    payload
        .get("events")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_i64).collect())
        .unwrap_or_default()
}

fn materialise(events: &[Event], undone: &HashSet<i64>) -> State {
    let mut st = State::default();
    for e in events {
        if !undone.contains(&e.id) {
            apply(&mut st, e);
        }
    }
    st
}

fn str_of<'v>(v: &'v Value, k: &str) -> &'v str {
    v.get(k).and_then(Value::as_str).unwrap_or("")
}

fn anchor_of(v: &Value) -> Option<Anchor> {
    Anchor::parse(v.get("anchor")?.as_str()?).ok()
}

fn conf_of(v: &Value) -> Option<Confidence> {
    v.get("confidence")
        .and_then(Value::as_str)
        .and_then(|s| s.parse().ok())
}

/// Evidence as stored: `[{condition, matched}]`.
pub fn evidence_json(ev: &[Evidence]) -> Value {
    Value::Array(
        ev.iter()
            .map(|e| serde_json::json!({"condition": e.condition, "matched": e.matched}))
            .collect(),
    )
}

fn evidence_of(v: &Value) -> Vec<Evidence> {
    v.get("evidence")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .map(|e| Evidence {
                    condition: str_of(e, "condition").to_string(),
                    matched: str_of(e, "matched").to_string(),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn empty_fp() -> Fingerprint {
    Fingerprint {
        org: String::new(),
        doc_id: String::new(),
        line_id: String::new(),
        account: String::new(),
        amount: String::new(),
        date: String::new(),
        description_hash: String::new(),
    }
}

fn items(p: &Value, k: &str) -> Vec<Value> {
    p.get(k)
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn touch(st: &mut State, a: &Anchor, id: i64) {
    let v = st.writes_by_anchor.entry(a.clone()).or_default();
    if v.last() != Some(&id) {
        v.push(id);
    }
}

/// The anchors an event writes (for "undo the last write on this line"). Suggestions are not writes.
pub fn event_anchors(e: &Event) -> Vec<Anchor> {
    match e.kind {
        EventKind::LabelSet | EventKind::LabelCleared => {
            let mut v: Vec<Anchor> = items(&e.payload, "labels")
                .iter()
                .filter_map(anchor_of)
                .collect();
            v.dedup();
            v
        }
        EventKind::NoteAdded => e
            .payload
            .get("target")
            .and_then(anchor_of)
            .into_iter()
            .collect(),
        EventKind::GroupMemberAdded | EventKind::GroupMemberRemoved => {
            anchor_of(&e.payload).into_iter().collect()
        }
        _ => vec![],
    }
}

fn apply(st: &mut State, e: &Event) {
    let p = &e.payload;
    let caption = p.get("caption").and_then(Value::as_str).map(str::to_string);
    match e.kind {
        EventKind::LabelSet => {
            for l in items(p, "labels") {
                let Some(anchor) = anchor_of(&l) else {
                    continue;
                };
                let ty = str_of(&l, "type").to_string();
                let source = Source::parse(str_of(&l, "source")).unwrap_or(Source::Human);
                st.labels.insert(
                    (anchor.clone(), ty.clone()),
                    Label {
                        anchor: anchor.clone(),
                        label_type: ty,
                        value: str_of(&l, "value").to_string(),
                        source,
                        actor: e.actor.clone(),
                        confidence: conf_of(&l),
                        evidence: evidence_of(&l),
                        fingerprint: l
                            .get("fingerprint")
                            .and_then(Fingerprint::from_json)
                            .unwrap_or_else(empty_fp),
                        at: e.at.clone(),
                        event_id: e.id,
                        caption: caption.clone(),
                    },
                );
                touch(st, &anchor, e.id);
            }
        }
        EventKind::LabelCleared => {
            for l in items(p, "labels") {
                let Some(anchor) = anchor_of(&l) else {
                    continue;
                };
                st.labels
                    .remove(&(anchor.clone(), str_of(&l, "type").to_string()));
                touch(st, &anchor, e.id);
            }
        }
        EventKind::Suggestion => {
            for l in items(p, "suggestions") {
                let Some(anchor) = anchor_of(&l) else {
                    continue;
                };
                let ty = str_of(&l, "type").to_string();
                let key = (anchor.clone(), ty.clone());
                if str_of(&l, "value").is_empty() {
                    st.suggestions.remove(&key);
                    continue;
                }
                st.suggestions.insert(
                    key,
                    Suggestion {
                        anchor,
                        label_type: ty,
                        value: str_of(&l, "value").to_string(),
                        source: Source::parse(str_of(&l, "source"))
                            .unwrap_or(Source::Rule(String::new())),
                        confidence: conf_of(&l).unwrap_or(Confidence::ONE),
                        evidence: evidence_of(&l),
                        fingerprint: l
                            .get("fingerprint")
                            .and_then(Fingerprint::from_json)
                            .unwrap_or_else(empty_fp),
                        event_id: e.id,
                    },
                );
            }
        }
        EventKind::NoteAdded => {
            let t = p.get("target").cloned().unwrap_or(Value::Null);
            let target = if let Some(a) = anchor_of(&t) {
                touch(st, &a, e.id);
                NoteTarget::Line(a)
            } else {
                NoteTarget::Group(str_of(&t, "group").to_string())
            };
            st.notes.push(Note {
                event_id: e.id,
                target,
                text: str_of(p, "text").to_string(),
                actor: e.actor.clone(),
                at: e.at.clone(),
            });
        }
        EventKind::GroupCreated => {
            let id = format!("g{}", e.id);
            st.groups.insert(
                id.clone(),
                Group {
                    id,
                    kind: str_of(p, "kind").to_string(),
                    title: str_of(p, "title").to_string(),
                    status: "open".into(),
                    members: vec![],
                    actor: e.actor.clone(),
                    at: e.at.clone(),
                },
            );
        }
        EventKind::GroupMemberAdded => {
            let Some(anchor) = anchor_of(p) else { return };
            touch(st, &anchor, e.id);
            if let Some(g) = st.groups.get_mut(str_of(p, "group")) {
                let role = p.get("role").and_then(Value::as_str).map(str::to_string);
                match g.members.iter_mut().find(|m| m.anchor == anchor) {
                    Some(m) => m.role = role,
                    None => g.members.push(Member { anchor, role }),
                }
            }
        }
        EventKind::GroupMemberRemoved => {
            let Some(anchor) = anchor_of(p) else { return };
            touch(st, &anchor, e.id);
            if let Some(g) = st.groups.get_mut(str_of(p, "group")) {
                g.members.retain(|m| m.anchor != anchor);
            }
        }
        EventKind::GroupStatus => {
            if let Some(g) = st.groups.get_mut(str_of(p, "group")) {
                g.status = str_of(p, "status").to_string();
            }
        }
        EventKind::ConfigWritten | EventKind::Undo => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn tmp(name: &str) -> PathBuf {
        let d =
            std::env::temp_dir().join(format!("xa-classify-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d.join("classify.sqlite")
    }

    fn label(anchor: &str, ty: &str, v: &str) -> Value {
        json!({"labels": [{"anchor": anchor, "type": ty, "value": v, "source": "human"}]})
    }

    #[test]
    fn append_chain_verify_and_append_only() {
        let p = tmp("chain");
        let mut s = Store::open(&p).unwrap();
        let (h, b) = (Actor::human(), BatchId::new("b1"));
        let ids = s
            .append(
                &h,
                &b,
                vec![
                    (
                        EventKind::LabelSet,
                        label("roast:bt:bt-1:L1", "project", "harvest"),
                    ),
                    (
                        EventKind::LabelSet,
                        label("roast:bt:bt-1:L1", "bearer", "roast"),
                    ),
                ],
            )
            .unwrap();
        assert_eq!(ids, vec![1, 2]);
        assert_eq!(s.verify().unwrap(), 2);
        assert!(
            s.conn
                .as_ref()
                .unwrap()
                .execute("DELETE FROM events", [])
                .is_err()
        );
        assert!(
            s.conn
                .as_ref()
                .unwrap()
                .execute("UPDATE events SET actor='x'", [])
                .is_err()
        );
        // a second handle sees the first one's writes after refresh
        let mut s2 = Store::open(&p).unwrap();
        assert_eq!(s2.rev(), 2);
        s.append(
            &h,
            &b,
            vec![(
                EventKind::LabelSet,
                label("roast:bt:bt-1:L1", "project", "refit"),
            )],
        )
        .unwrap();
        assert!(s2.refresh().unwrap());
        let a = Anchor::parse("roast:bt:bt-1:L1").unwrap();
        assert_eq!(s2.state().labels[&(a, "project".into())].value, "refit");
    }

    #[test]
    fn undo_and_undo_of_undo() {
        let p = tmp("undo");
        let mut s = Store::open(&p).unwrap();
        let (h, b) = (Actor::human(), BatchId::new("b"));
        let a = Anchor::parse("lab:bt:bt-9:L1").unwrap();
        let key = (a.clone(), "project".to_string());
        s.append(
            &h,
            &b,
            vec![(
                EventKind::LabelSet,
                label("lab:bt:bt-9:L1", "project", "one"),
            )],
        )
        .unwrap();
        s.append(
            &h,
            &b,
            vec![(
                EventKind::LabelSet,
                label("lab:bt:bt-9:L1", "project", "two"),
            )],
        )
        .unwrap();
        assert_eq!(s.state().labels[&key].value, "two");
        s.append(&h, &b, vec![(EventKind::Undo, json!({"events": [2]}))])
            .unwrap();
        assert_eq!(s.state().labels[&key].value, "one");
        assert_eq!(s.state().writes_by_anchor[&a], vec![1]);
        s.append(&h, &b, vec![(EventKind::Undo, json!({"events": [3]}))])
            .unwrap();
        assert_eq!(s.state().labels[&key].value, "two");
        // a fresh open materialises the same state
        let s2 = Store::open(&p).unwrap();
        assert_eq!(s2.state().labels[&key].value, "two");
        assert_eq!(s2.verify().unwrap(), 4);
    }

    #[test]
    fn pinned_clock() {
        pin_clock(Some("2026-10-05T09:46:00.000000Z"));
        assert_eq!(now_iso(), "2026-10-05T09:46:00.000000Z");
    }
}
