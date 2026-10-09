//! Readers share the record: two read-only `Record`s on two threads, while a third connection holds an open
//! read transaction, all see identical results. The fixture record is in WAL mode (the writer sets it), so
//! readers never block each other or a writer.

mod common;

use ownpurse_core::{Ledger, Mode, Record, resolve};
use rusqlite::{Connection, OpenFlags};
use std::sync::{Arc, Barrier};

type Snapshot = (
    usize,
    Vec<(String, String, String)>,
    Option<String>,
    Option<bool>,
);

fn read_all(path: &std::path::Path, tenants: &[String], barrier: &Barrier) -> Vec<Snapshot> {
    let record = Record::open(path).unwrap();
    barrier.wait(); // both readers are open at once, inside the other connection's read transaction
    tenants
        .iter()
        .map(|tid| {
            let l = Ledger::build(&ownpurse_core::view::View::main(&record), tid).unwrap();
            let fy = record.fy_end(tid).unwrap();
            let b = resolve(&record, &l, tid, "2025-12-31", fy, Mode::Auto).unwrap();
            let postings = l
                .postings
                .iter()
                .map(|p| {
                    (
                        p.source_id.clone(),
                        p.account_id.clone(),
                        p.amount.to_string(),
                    )
                })
                .collect();
            (
                l.postings.len(),
                postings,
                record
                    .tb_snapshot(tid, "2025-12-31")
                    .unwrap()
                    .map(|r| r.fetched_at),
                b.matches_xero(),
            )
        })
        .collect()
}

#[test]
fn two_readers_and_an_open_read_transaction_see_the_same_record() {
    let path = common::fixtures().join("record.sqlite");
    let tenants: Vec<String> = common::read_json(&common::fixtures().join("tenants.json"))
        .as_object()
        .unwrap()
        .values()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();

    ownpurse_core::record::sqlite_ready(); // first SQLite init on one thread
    let holder = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let mode: String = holder
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(mode.to_lowercase(), "wal", "the record is in WAL mode");
    holder.execute_batch("BEGIN").unwrap();
    let held: i64 = holder
        .query_row("SELECT COUNT(*) FROM versions", [], |r| r.get(0))
        .unwrap();
    assert!(held > 0);

    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|_| {
            let (path, tenants, barrier) = (path.clone(), tenants.clone(), barrier.clone());
            std::thread::spawn(move || read_all(&path, &tenants, &barrier))
        })
        .collect();
    let results: Vec<Vec<Snapshot>> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    let still: i64 = holder
        .query_row("SELECT COUNT(*) FROM versions", [], |r| r.get(0))
        .unwrap();
    holder.execute_batch("COMMIT").unwrap();
    assert_eq!(held, still, "the open read transaction is undisturbed");
    assert_eq!(
        results[0], results[1],
        "both readers read identical results"
    );
    assert_eq!(results[0].len(), tenants.len());
    let total: usize = results[0].iter().map(|s| s.0).sum();
    assert!(total > 1000, "{total} postings");
}
