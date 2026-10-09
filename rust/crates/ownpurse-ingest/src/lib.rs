//! Projectors: pure functions from source files to transactions of facts.
//!
//! The ingest API in four steps; only the first and last touch the disk, and they live in the shell:
//!
//! 1. the shell puts every input file's bytes in the drawer and gets back their hashes;
//! 2. it builds an `Input`: the main file, its companions (a column mapping, the PDF an extraction reads),
//!    the source, and when the source was read;
//! 3. `plan(db, projector, input)` returns a `Plan`: attributes to install, the transaction, and the
//!    pressure the projector felt (where the source did not fit the model cleanly);
//! 4. the shell transacts the install, then the transaction, and the pressure is in the log as
//!    `tx/pressure` facts.
//!
//! A projector never writes; it reads the current `Db` only to diff a snapshot source (a Xero document, a
//! sheet) against what it said last time.

pub mod facts;
pub mod jsonmirror;
pub mod roundtrip;
pub mod rules;
pub mod snapshot;

pub mod sources;

use ownpurse_facts::{Db, EntRef, Hash, Instant, Kw, TxData, Value};
use std::fmt;

/// One file going in: its bytes are already in the drawer under `hash`.
#[derive(Debug, Clone)]
pub struct FileIn<'a> {
    pub hash: Hash,
    pub name: &'a str,
    pub media_type: &'a str,
    pub bytes: &'a [u8],
    /// What the file is to this ingest: "main", "mapping", "reads" (the file an extraction reads), …
    pub role: &'a str,
}

/// Where the files came from: one Xero org, one bank account's downloads, one wallet, one sheet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpec {
    /// Stable id, e.g. `xero:tallowmere`, `bank:ridgeway-4821`, `eth:0x51ab…`
    pub id: String,
    /// e.g. `xero`, `qbo`, `freshbooks`, `bank`, `sheet`, `eth`
    pub kind: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Input<'a> {
    pub file: FileIn<'a>,
    pub with: Vec<FileIn<'a>>,
    pub source: SourceSpec,
    pub observed_at: Instant,
}

impl<'a> Input<'a> {
    /// The companion file with this role.
    pub fn companion(&self, role: &str) -> Option<&FileIn<'a>> {
        self.with.iter().find(|f| f.role == role)
    }
}

/// A place where a source did not fit the model cleanly. These are the edges this phase looks for.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pressure {
    /// A short kebab-case kind: `no-identity`, `embedded-copy`, `type-conflict`, `order-lost`, `sign`,
    /// `derived-value`, `precision`, `null`, …
    pub kind: String,
    /// Where in the file (a JSON pointer, `row:12`, `page:2`).
    pub at: String,
    pub detail: String,
}

impl Pressure {
    pub fn new(kind: &str, at: impl Into<String>, detail: impl Into<String>) -> Pressure {
        Pressure {
            kind: kind.to_string(),
            at: at.into(),
            detail: detail.into(),
        }
    }

    pub fn line(&self) -> String {
        format!("{} | {} | {}", self.kind, self.at, self.detail)
    }
}

/// What a projector produces.
#[derive(Debug, Clone, Default)]
pub struct Projection {
    /// Attribute definitions (the compact form of `ownpurse_facts::install`) this projection needs.
    pub install: Vec<serde_json::Value>,
    pub tx: TxData,
    pub pressure: Vec<Pressure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IngestError(pub String);

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for IngestError {}

pub trait Projector: Sync {
    /// `name@version`, e.g. `xero.json@1`. Recorded on every transaction it writes.
    fn id(&self) -> &'static str;
    /// One line: what files it reads.
    fn reads(&self) -> &'static str;
    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError>;
}

/// What the shell transacts, in order: `install` (if not empty), then `tx`.
#[derive(Debug, Clone)]
pub struct Plan {
    pub install: TxData,
    pub tx: TxData,
    pub pressure: Vec<Pressure>,
}

/// Run a projector and wrap its transaction in the envelope every ingest shares: the files (with their
/// names, types and sizes), the source, and the metadata that says who wrote it and from what.
pub fn plan(db: &Db, p: &dyn Projector, input: &Input<'_>) -> Result<Plan, IngestError> {
    let proj = p.project(db, input)?;
    let mut install = ownpurse_facts::install(&serde_json::Value::Array(proj.install))
        .map_err(|e| IngestError(format!("{}: bad attribute definitions: {e}", p.id())))?;
    if !install.is_empty() {
        install.meta(
            &Kw::lit("tx/actor"),
            Value::str(&format!("projector:{}", p.id())),
        );
        install.meta(
            &Kw::lit("tx/reason"),
            Value::str(&format!("attributes first seen in {}", input.file.name)),
        );
    }
    let mut tx = facts::envelope(input);
    let k = Kw::lit;
    tx.meta(&k("tx/projector"), Value::str(p.id()));
    tx.meta(&k("tx/actor"), Value::str(&format!("projector:{}", p.id())));
    tx.meta(
        &k("tx/observed-at"),
        Value::Instant(input.observed_at.clone()),
    );
    tx.meta(&k("tx/source"), facts::source_ref(&input.source));
    for f in std::iter::once(&input.file).chain(&input.with) {
        tx.meta(&k("tx/evidence"), facts::file_ref(&f.hash));
    }
    let mut pressure = proj.pressure;
    pressure.sort();
    pressure.dedup();
    for pr in &pressure {
        tx.meta(&k("tx/pressure"), Value::str(&pr.line()));
    }
    tx.extend(proj.tx);
    Ok(Plan {
        install,
        tx,
        pressure,
    })
}

/// Every projector this build knows.
pub fn projectors() -> Vec<&'static dyn Projector> {
    sources::all()
}

pub fn projector(id: &str) -> Option<&'static dyn Projector> {
    let want = id.split('@').next().unwrap_or(id);
    projectors()
        .into_iter()
        .find(|p| p.id() == id || p.id().split('@').next() == Some(want))
}

/// A temp entity reference keyed by its identity, shared by every helper so two helpers naming the same
/// thing in one transaction name the same entity.
pub fn temp(key: &str) -> EntRef {
    EntRef::Temp(key.to_string())
}

/// A snapshot source's guard: if the document was last read after this read was taken, the read is older and
/// must not overwrite what the newer one said. Returns the empty projection to use instead.
pub fn stale(db: &Db, doc_id: &str, input: &Input<'_>) -> Option<Projection> {
    let last = jsonmirror::last_read(db, doc_id).filter(|last| *last > input.observed_at)?;
    Some(Projection {
        pressure: vec![Pressure::new(
            "stale-read",
            doc_id,
            format!(
                "a read from before the last one ({last}); an older snapshot never overwrites a newer one"
            ),
        )],
        ..Projection::default()
    })
}
