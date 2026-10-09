//! The record is opened read-only and every accessor answers from the fixture without writing.

use ownpurse_core::record::{Record, RecordError, tb_params};
use std::path::Path;

fn fixture() -> Record {
    Record::open(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/record.sqlite"))
        .unwrap()
}

fn tenants() -> serde_json::Value {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/tenants.json");
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

#[test]
fn missing_record_is_an_error_not_a_new_file() {
    let p = std::env::temp_dir().join("ownpurse-does-not-exist.sqlite");
    let _ = std::fs::remove_file(&p);
    assert!(matches!(Record::open(&p), Err(RecordError::Missing(_))));
    assert!(!p.exists(), "open must never create a record");
    assert!(
        Record::open(&p)
            .err()
            .unwrap()
            .to_string()
            .contains("no record at")
    );
}

#[test]
fn accessors_read_the_fixture() {
    let r = fixture();
    let t = tenants();
    let one = t["one"].as_str().unwrap();
    let hq = t["hq"].as_str().unwrap();
    assert!(r.path().ends_with("record.sqlite"));
    let rows = r.current_rows(one, "Accounts").unwrap();
    assert!(!rows.is_empty());
    assert!(
        rows.windows(2).all(|w| w[0].xero_id <= w[1].xero_id),
        "ordered by Xero id"
    );
    assert!(
        rows.iter()
            .all(|x| x.version >= 1 && !x.observed_at.is_empty())
    );
    assert_eq!(
        ownpurse_core::view::View::main(&r)
            .payloads(one, "Accounts")
            .unwrap()
            .len(),
        rows.len()
    );
    assert_eq!(
        r.organisation(one).unwrap()["Name"],
        "Pewter Lane Ironmongery"
    );
    assert_eq!(r.fy_end(one).unwrap(), (12, 31));
    assert!(r.organisation("nobody").unwrap().is_null());
    assert_eq!(r.fy_end("nobody").unwrap(), (12, 31));

    let tb = r
        .tb_snapshot(one, "2025-12-31")
        .unwrap()
        .expect("cached TB");
    assert_eq!(tb.fetched_at, "2026-10-05T07:46:00Z");
    assert!(tb.payload.get("Reports").is_some());
    assert!(r.tb_snapshot(one, "2025-06-30").unwrap().is_none());
    assert_eq!(
        tb_params("2025-12-31"),
        r#"{"date":"2025-12-31","paymentsOnly":"false"}"#
    );
    assert!(r.report(one, "BalanceSheet", "{}").unwrap().is_none());

    assert!(
        r.last_sync(one).unwrap().is_none(),
        "fixture org one has no sync runs"
    );
    let runs = r.sync_runs(hq).unwrap();
    assert!(!runs.is_empty());
    assert!(
        runs.iter()
            .all(|s| s.outcome == "ok" && s.calls == 0 && s.fetched == 0 && s.new_versions == 0)
    );
    assert!(runs.iter().any(|s| s.entity == "Accounts"));
    assert_eq!(
        r.last_sync(hq).unwrap().as_deref(),
        Some("2026-10-05T07:46:00")
    );
    assert!(r.max_version_id(one).unwrap() > 0);
    assert_eq!(r.max_version_id("nobody").unwrap(), 0);
    assert_eq!(r.http_calls().unwrap(), 0);
    let n: i64 = r
        .conn()
        .query_row("SELECT COUNT(*) FROM versions", [], |x| x.get(0))
        .unwrap();
    assert!(n > 0);
}

#[test]
fn record_is_read_only() {
    let r = fixture();
    let err = r.conn().execute("CREATE TABLE nope(x)", []).unwrap_err();
    assert!(
        err.to_string().to_lowercase().contains("readonly")
            || err.to_string().contains("read-only"),
        "{err}"
    );
}
