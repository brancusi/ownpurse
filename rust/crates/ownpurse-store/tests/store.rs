use ownpurse_facts::{Instant, Kw, TxData, Value};
use ownpurse_store::{Store, StoreError, sql};
use serde_json::json;

fn at(n: i64) -> Instant {
    Instant::from_unix_millis(1_773_000_000_000 + n * 1000)
}

fn tx(j: serde_json::Value) -> TxData {
    TxData::from_json(&j).unwrap()
}

fn fresh() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let s = Store::create(&dir.path().join("books.sqlite"), None, &at(0)).unwrap();
    (dir, s)
}

#[test]
fn a_log_reopens_to_the_same_database_and_head() {
    let (dir, mut s) = fresh();
    s.transact(
        &tx(json!({"ops": [{"obs/id": "o1", "obs/amount": "-186.40"}]})),
        &at(1),
    )
    .unwrap();
    s.transact(
        &tx(json!({"ops": [{"obs/id": "o1", "obs/amount": "-186.41"}]})),
        &at(2),
    )
    .unwrap();
    let (head, len, basis) = (s.head(), s.db().len(), s.db().basis());
    drop(s);
    let s = Store::open(&dir.path().join("books.sqlite"), None).unwrap();
    assert_eq!((s.head(), s.db().len(), s.db().basis()), (head, len, basis));
    let o = s
        .db()
        .lookup(&Kw::lit("obs/id"), &Value::str("o1"))
        .unwrap();
    assert_eq!(
        s.db().get(o, &Kw::lit("obs/amount")).unwrap().text(),
        "-186.41"
    );
}

#[test]
fn an_empty_transaction_writes_nothing() {
    let (_d, mut s) = fresh();
    let t = tx(json!({"ops": [{"obs/id": "o1"}]}));
    s.transact(&t, &at(1)).unwrap();
    let n: i64 = s
        .conn()
        .query_row("SELECT count(*) FROM tx", [], |r| r.get(0))
        .unwrap();
    let r = s.transact(&t, &at(2)).unwrap();
    let m: i64 = s
        .conn()
        .query_row("SELECT count(*) FROM tx", [], |r| r.get(0))
        .unwrap();
    assert_eq!((r.tx, n), (None, m));
}

#[test]
fn the_log_refuses_updates_and_deletes() {
    let (_d, s) = fresh();
    for q in [
        "UPDATE datom SET v = 'x'",
        "DELETE FROM datom",
        "UPDATE tx SET hash = 'x'",
        "DELETE FROM tx",
    ] {
        let e = s.conn().execute(q, []).unwrap_err();
        assert!(e.to_string().contains("append-only"), "{q}: {e}");
    }
}

#[test]
fn a_changed_byte_in_the_log_is_caught() {
    let (dir, mut s) = fresh();
    s.transact(
        &tx(json!({"ops": [{"obs/id": "o1", "obs/amount": "10.00"}]})),
        &at(1),
    )
    .unwrap();
    // someone with the file and a SQLite shell drops the trigger and edits a cent
    s.conn()
        .execute_batch(
            "DROP TRIGGER datom_append_only_u; UPDATE datom SET v = '10.01' WHERE v = '10.00';",
        )
        .unwrap();
    drop(s);
    match Store::open(&dir.path().join("books.sqlite"), None) {
        Err(StoreError::Corrupt(m)) => assert!(m.contains("do not match its hash"), "{m}"),
        Err(other) => panic!("{other}"),
        Ok(_) => panic!("tampering went unnoticed"),
    }
}

#[test]
fn verify_rehashes_the_files_the_log_cites() {
    let (_d, mut s) = fresh();
    let (h, new) = s.drawer().put(b"statement bytes").unwrap();
    assert!(new);
    assert!(
        !s.drawer().put(b"statement bytes").unwrap().1,
        "same bytes, same file"
    );
    let mut t = TxData::new();
    let f = ownpurse_facts::EntRef::temp("f");
    t.add(&f, &Kw::lit("file/sha256"), Value::Hash(h));
    s.transact(&t, &at(1)).unwrap();
    assert_eq!(s.verify().unwrap().files, 1);
    std::fs::write(s.drawer().path(&h), b"statement bytez").unwrap();
    assert!(matches!(s.verify(), Err(StoreError::Corrupt(m)) if m.contains("do not match")));
    std::fs::remove_file(s.drawer().path(&h)).unwrap();
    assert!(matches!(s.verify(), Err(StoreError::Corrupt(m)) if m.contains("missing")));
}

#[test]
fn dsum_is_exact_and_keeps_the_scale() {
    let (_d, s) = fresh();
    sql::register(s.conn()).unwrap();
    let (exact, kind): (String, String) = s
        .conn()
        .query_row(
            "WITH x(v) AS (VALUES ('0.10'), ('0.20'), ('0.30'), ('1234567.89'), ('-1234567.89'))
             SELECT dsum(v), typeof(sum(v)) FROM x",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(exact, "0.60");
    // sum() turns decimal text into a REAL: the scale is gone and large sums can drift
    assert_eq!(kind, "real");
}

#[test]
fn the_views_read_the_log_by_hand() {
    let (_d, mut s) = fresh();
    s.transact(
        &tx(json!({"meta": {"tx/actor": "person:test"}, "ops": [
            {"db/id": "usd", "commodity/code": "USD"},
            {"db/id": "acct", "account/id": "a1", "account/name": "Checking"},
            {"obs/id": "o1", "obs/kind": "obs.kind/money-movement", "obs/amount": "-4.50",
             "obs/commodity": "usd", "obs/account": "acct", "obs/date": "2026-03-02"}
        ]})),
        &at(1),
    )
    .unwrap();
    s.transact(
        &tx(json!({"ops": [{"obs/id": "o1", "obs/amount": "-4.75"}]})),
        &at(2),
    )
    .unwrap();
    let row: (String, String, String, String) = s
        .conn()
        .query_row(
            "SELECT id, amount, commodity, account FROM observations",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    assert_eq!(
        row,
        ("o1".into(), "-4.75".into(), "USD".into(), "Checking".into())
    );
    let hist: Vec<(String, String)> = s
        .conn()
        .prepare("SELECT value, op FROM history WHERE attr = 'obs/amount' ORDER BY tx, op DESC")
        .unwrap()
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(
        hist,
        [
            ("-4.50".into(), "assert".into()),
            ("-4.50".into(), "retract".into()),
            ("-4.75".into(), "assert".into())
        ]
    );
    let actor: String = s
        .conn()
        .query_row(
            "SELECT actor FROM transactions WHERE actor IS NOT NULL AND actor != 'ownpurse'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(actor, "person:test");
}

#[test]
fn create_refuses_to_overwrite_and_open_refuses_a_missing_log() {
    let (dir, _s) = fresh();
    let p = dir.path().join("books.sqlite");
    assert!(matches!(
        Store::create(&p, None, &at(0)),
        Err(StoreError::Exists(_))
    ));
    assert!(matches!(
        Store::open(&dir.path().join("nope.sqlite"), None),
        Err(StoreError::Missing(_))
    ));
}
