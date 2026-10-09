//! The writer is byte-compatible with the frozen reference records: same schema, same row hashes, same
//! chain, so it appends to an existing record and `Writer::verify()` proves the whole chain.

mod common;

use common::{frozen_record, temp_dir};
use ownpurse_xero::store::{Writer, iso_micros, now_iso};
use rusqlite::types::Value as Sql;

fn http_row(tenant: Option<&str>, path: &str, status: i64) -> Vec<(&'static str, Sql)> {
    vec![
        ("at", Sql::Text(now_iso())),
        (
            "tenant_id",
            tenant.map(|t| Sql::Text(t.into())).unwrap_or(Sql::Null),
        ),
        ("method", Sql::Text("GET".into())),
        ("path", Sql::Text(path.into())),
        (
            "params",
            Sql::Text(r#"{"page": 1, "where": "Status==\"ACTIVE\" ünï"}"#.into()),
        ),
        ("status", Sql::Integer(status)),
        ("duration_ms", Sql::Integer(5)),
        ("day_remaining", Sql::Integer(999)),
        ("min_remaining", Sql::Null),
        ("problem", Sql::Null),
        ("op_id", Sql::Null),
    ]
}

#[test]
fn a_new_record_has_the_reference_schema() {
    let d = temp_dir("schema");
    Writer::open(d.join("rust.sqlite")).unwrap();
    // an empty reference record (frozen)
    frozen_record("ref-empty.sqlite", &d.join("ref.sqlite"));
    let dump = |p: &str| {
        ownpurse_core::record::sqlite_ready(); // first SQLite init on one thread
        let c = rusqlite::Connection::open(d.join(p)).unwrap();
        let mut st = c
            .prepare("SELECT type, name, sql FROM sqlite_master ORDER BY type, name")
            .unwrap();
        let rows: Vec<(String, String, Option<String>)> = st
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        let meta: Vec<(String, String)> = c
            .prepare("SELECT key, value FROM meta ORDER BY key")
            .unwrap()
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        (rows, meta)
    };
    // a record that never saw an approval has exactly the frozen reference schema
    assert_eq!(dump("rust.sqlite"), dump("ref.sqlite"));
    // the first approval adds only the Phase 8 objects (the table, its index, its append-only triggers)
    Writer::open(d.join("rust.sqlite"))
        .unwrap()
        .ensure_approvals()
        .unwrap();
    let (mut rows, meta) = dump("rust.sqlite");
    let added: Vec<String> = rows
        .iter()
        .filter(|(_, n, _)| n.starts_with("approvals"))
        .map(|(_, n, _)| n.clone())
        .collect();
    rows.retain(|(_, n, _)| !n.starts_with("approvals"));
    assert_eq!((rows, meta), dump("ref.sqlite"));
    assert_eq!(
        added,
        [
            "approvals_hash",
            "approvals",
            "approvals_no_delete",
            "approvals_no_update"
        ]
    );
}

#[test]
fn appends_to_a_reference_record_verify() {
    let d = temp_dir("chain");
    let p = d.join("record.sqlite");
    // a reference record with one append (frozen), then the writer appends, then the chain verifies
    frozen_record("ref-one-append.sqlite", &p);
    let w = Writer::open(&p).unwrap();
    let a = w
        .append("http_log", &http_row(Some("t-1"), "Accounts", 200))
        .unwrap();
    let b = w
        .append("http_log", &http_row(None, "connections", 200))
        .unwrap();
    assert_eq!((a.as_str(), b.as_str()), ("2", "3"), "row ids are rowids");
    w.append("http_log", &http_row(Some("t-2"), "BankTransactions", 429))
        .unwrap();
    assert_eq!(w.verify().unwrap().0, 4);

    // the row hash of the reference row equals the hash its chain recorded
    let (row_id, chained): (String, String) = w
        .conn()
        .query_row(
            "SELECT row_id, row_hash FROM chain WHERE seq = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(w.row_hash("http_log", &row_id).unwrap(), chained);
}

#[test]
fn history_is_append_only() {
    let d = temp_dir("ao");
    let w = Writer::open(d.join("r.sqlite")).unwrap();
    w.append("http_log", &http_row(Some("t"), "Accounts", 200))
        .unwrap();
    for sql in [
        "UPDATE http_log SET status = 500",
        "DELETE FROM http_log",
        "DELETE FROM chain",
    ] {
        let e = w.conn().execute(sql, []).unwrap_err().to_string();
        assert!(e.contains("append-only"), "{sql}: {e}");
    }
}

#[test]
fn calls_since_counts_per_tenant_including_null() {
    let d = temp_dir("calls");
    let w = Writer::open(d.join("r.sqlite")).unwrap();
    w.append("http_log", &http_row(Some("t"), "A", 200))
        .unwrap();
    w.append("http_log", &http_row(Some("t"), "B", 200))
        .unwrap();
    w.append("http_log", &http_row(None, "connections", 200))
        .unwrap();
    assert_eq!(w.calls_since(Some("t"), "2000-01-01").unwrap(), 2);
    assert_eq!(w.calls_since(None, "2000-01-01").unwrap(), 1);
    assert_eq!(w.calls_since(Some("t"), "2999-01-01").unwrap(), 0);
}

#[test]
fn timestamps_use_the_record_format() {
    assert_eq!(
        iso_micros(std::time::Duration::new(1_759_657_560, 123_456_789)),
        "2025-10-05T09:46:00.123456Z"
    );
    let n = now_iso();
    assert_eq!(n.len(), 27);
    assert!(n.ends_with('Z') && n.as_bytes()[10] == b'T');
}

#[test]
fn an_approvals_table_from_before_demo_standing_is_upgraded_with_its_chain_intact() {
    let d = temp_dir("approvals-v1");
    let p = d.join("record.sqlite");
    let w = Writer::open(&p).unwrap();
    w.conn()
        .execute_batch(
            "CREATE TABLE approvals(id INTEGER PRIMARY KEY, hash TEXT NOT NULL, verbatim TEXT NOT NULL, \
             via TEXT NOT NULL CHECK (via IN ('tty','chat')), where_given TEXT NOT NULL, at TEXT NOT NULL, \
             scope TEXT NOT NULL, recorded_at TEXT NOT NULL) STRICT;",
        )
        .unwrap();
    w.append(
        "approvals",
        &[
            ("hash", Sql::Text("h".into())),
            ("verbatim", Sql::Text("ok h".into())),
            ("via", Sql::Text("tty".into())),
            ("where_given", Sql::Text("tty".into())),
            ("at", Sql::Text("2026-10-07T12:00:00Z".into())),
            ("scope", Sql::Text("\"all\"".into())),
            ("recorded_at", Sql::Text("2026-10-07T12:00:01Z".into())),
        ],
    )
    .unwrap();
    w.ensure_approvals().unwrap();
    w.verify().unwrap();
    w.append(
        "approvals",
        &[
            ("hash", Sql::Text("h2".into())),
            ("verbatim", Sql::Text("standing".into())),
            ("via", Sql::Text("demo_standing".into())),
            ("where_given", Sql::Text("file".into())),
            ("at", Sql::Text("2026-10-07T12:00:00Z".into())),
            ("scope", Sql::Text("\"all\"".into())),
            ("recorded_at", Sql::Text("2026-10-07T12:00:01Z".into())),
        ],
    )
    .unwrap();
    w.verify().unwrap();
    assert!(
        w.conn().execute("DELETE FROM approvals", []).is_err(),
        "still append-only"
    );
}
