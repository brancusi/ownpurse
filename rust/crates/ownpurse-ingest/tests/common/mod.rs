//! Shared by every source's tests: an in-memory database with the core vocabulary, and an ingest that plans
//! with a projector and applies the plan the way the store does (install first, then the transaction).
#![allow(dead_code)] // each test file uses a different subset

use ownpurse_facts::encode::sha256;
use ownpurse_facts::{CORE_SCHEMA, Db, Instant, Kw, Value, install};
use ownpurse_ingest::{FileIn, Input, Plan, Projector, SourceSpec};
use std::path::{Path, PathBuf};

pub fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples")
}

pub fn at(s: &str) -> Instant {
    Instant::parse(s).unwrap()
}

pub fn core() -> Db {
    let defs: serde_json::Value = serde_json::from_str(CORE_SCHEMA).unwrap();
    Db::genesis().with(&install(&defs).unwrap(), &at("2026-01-01T00:00:00Z")).unwrap().db_after
}

pub fn source(id: &str) -> SourceSpec {
    SourceSpec { id: id.into(), kind: id.split(':').next().unwrap_or("").into(), name: id.into() }
}

pub struct Applied {
    pub db: Db,
    pub plan: Plan,
    /// the transaction id, or None when nothing changed
    pub tx: Option<ownpurse_facts::Eid>,
    pub asserted: usize,
    pub retracted: usize,
}

/// Ingest `file` (relative to samples/) with companions `with` (role, path relative to samples/).
pub fn ingest(db: &Db, p: &dyn Projector, src: &str, file: &str, with: &[(&str, &str)], observed: &str) -> Applied {
    let read = |f: &str| std::fs::read(samples().join(f)).unwrap_or_else(|e| panic!("{f}: {e}"));
    let main = read(file);
    let comps: Vec<(String, String, Vec<u8>)> =
        with.iter().map(|(r, f)| ((*r).to_string(), (*f).to_string(), read(f))).collect();
    let input = Input {
        file: FileIn { hash: sha256(&main), name: file, media_type: "", bytes: &main, role: "main" },
        with: comps
            .iter()
            .map(|(r, f, b)| FileIn { hash: sha256(b), name: f, media_type: "", bytes: b, role: r })
            .collect(),
        source: source(src),
        observed_at: at(observed),
    };
    let t = at(observed);
    let plan = ownpurse_ingest::plan(db, p, &input).unwrap_or_else(|e| panic!("{file}: {e}"));
    let db = if plan.install.is_empty() {
        db.clone()
    } else {
        db.with(&plan.install, &t).unwrap_or_else(|e| panic!("{file} install: {e}")).db_after
    };
    let plan = ownpurse_ingest::plan(&db, p, &input).unwrap_or_else(|e| panic!("{file}: {e}"));
    let r = db.with(&plan.tx, &t).unwrap_or_else(|e| panic!("{file}: {e}"));
    let asserted = r.datoms.iter().filter(|d| d.added).count();
    Applied { retracted: r.datoms.len() - asserted, asserted, tx: r.tx, db: r.db_after, plan }
}

pub fn get(db: &Db, attr: &'static str, value: &str, want: &'static str) -> Option<Value> {
    let e = db.lookup(&Kw::lit(attr), &Value::str(value))?;
    db.get(e, &Kw::lit(want)).cloned()
}

/// Entities with `attr` = `value` (any attribute, not just unique ones).
pub fn find(db: &Db, attr: &'static str, value: Value) -> Vec<ownpurse_facts::Eid> {
    db.attr(&Kw::lit(attr)).map(|d| db.with_value(d.id, &value).collect()).unwrap_or_default()
}

pub fn pressure_kinds(plan: &Plan) -> Vec<String> {
    let mut k: Vec<String> = plan.pressure.iter().map(|p| p.kind.clone()).collect();
    k.dedup();
    k
}
