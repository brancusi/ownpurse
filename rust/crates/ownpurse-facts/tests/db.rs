//! The database's transaction semantics, one behaviour per test.

use ownpurse_facts::{
    CORE_SCHEMA, Db, EntRef, Instant, Kw, Pattern, TxData, Value, encode, install, pull,
};
use serde_json::json;

fn at(n: i64) -> Instant {
    Instant::from_unix_millis(1_773_000_000_000 + n * 1000)
}

fn core() -> Db {
    let defs: serde_json::Value = serde_json::from_str(CORE_SCHEMA).unwrap();
    let r = Db::genesis()
        .with(&install(&defs).unwrap(), &at(0))
        .unwrap();
    r.db_after
}

fn tx(db: &Db, j: serde_json::Value) -> Result<ownpurse_facts::TxReport, ownpurse_facts::TxError> {
    db.with(&TxData::from_json(&j).unwrap(), &at(db.txs().len() as i64))
}

fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

#[test]
fn the_core_schema_installs_and_reinstalling_it_changes_nothing() {
    let db = core();
    assert!(db.attr(&k("obs/amount")).is_some());
    let defs: serde_json::Value = serde_json::from_str(CORE_SCHEMA).unwrap();
    let again = db.with(&install(&defs).unwrap(), &at(9)).unwrap();
    assert_eq!(
        again.tx, None,
        "an identical install is an empty transaction"
    );
}

#[test]
fn identity_upserts_and_card_one_replaces() {
    let db = core();
    let r1 = tx(
        &db,
        json!({"ops": [{"db/id": "a", "obs/id": "bank:1", "obs/amount": "10.00"}]}),
    )
    .unwrap();
    let e = r1.tempids["a"];
    let r2 = tx(
        &r1.db_after,
        json!({"ops": [{"obs/id": "bank:1", "obs/amount": "12.50"}]}),
    )
    .unwrap();
    let db2 = r2.db_after;
    assert_eq!(
        db2.get(e, &k("obs/amount")),
        Some(&Value::Dec("12.50".parse().unwrap()))
    );
    // the old value is retracted in the same transaction, so history has both
    let hist: Vec<String> = db2
        .history(e)
        .iter()
        .filter(|d| db2.schema().ident(d.a) == Some(&k("obs/amount")))
        .map(|d| format!("{}{}", if d.added { '+' } else { '-' }, d.v.text()))
        .collect();
    assert_eq!(hist, ["+10.00", "-10.00", "+12.50"]);
}

#[test]
fn saying_the_same_thing_twice_is_an_empty_transaction() {
    let db = core();
    let op =
        json!({"ops": [{"obs/id": "bank:1", "obs/amount": "10.00", "obs/description": "coffee"}]});
    let r1 = tx(&db, op.clone()).unwrap();
    let r2 = tx(&r1.db_after, op).unwrap();
    assert_eq!(r2.tx, None);
    assert!(r2.datoms.is_empty());
}

#[test]
fn two_temps_with_one_identity_are_one_entity() {
    let db = core();
    let r = tx(
        &db,
        json!({"ops": [
            {"db/id": "x", "commodity/code": "USD"},
            {"db/id": "y", "commodity/code": "USD", "commodity/scale": 2}
        ]}),
    )
    .unwrap();
    assert_eq!(r.tempids["x"], r.tempids["y"]);
}

#[test]
fn lookups_resolve_to_entities_named_earlier_in_the_same_transaction() {
    let db = core();
    let r = tx(
        &db,
        json!({"ops": [
            {"commodity/code": "USD"},
            {"obs/id": "o1", "obs/commodity": {"lookup": ["commodity/code", "USD"]}}
        ]}),
    )
    .unwrap();
    let usd = r
        .db_after
        .lookup(&k("commodity/code"), &Value::str("USD"))
        .unwrap();
    let o = r.db_after.lookup(&k("obs/id"), &Value::str("o1")).unwrap();
    assert_eq!(
        r.db_after.get(o, &k("obs/commodity")),
        Some(&Value::Ref(usd))
    );
}

#[test]
fn every_problem_is_reported_at_once() {
    let db = core();
    let e = tx(
        &db,
        json!({"ops": [
            {"obs/id": "o1", "obs/amount": "1,00", "obs/date": "2026-02-30", "nope/x": 1},
            {"obs/id": "o2", "obs/commodity": {"lookup": ["commodity/code", "XYZ"]}}
        ]}),
    )
    .unwrap_err();
    assert_eq!(e.problems.len(), 4, "{:#?}", e.problems);
}

#[test]
fn card_one_refuses_two_values_in_one_transaction() {
    let db = core();
    let e = tx(&db, json!({"ops": [
        ["add", "a", "obs/amount", "1.00"], ["add", "a", "obs/amount", "2.00"], ["add", "a", "obs/id", "z"]
    ]}))
    .unwrap_err();
    assert!(e.problems[0].contains("two values"), "{:?}", e.problems);
}

#[test]
fn unique_values_belong_to_one_entity() {
    let db = core();
    let r = tx(
        &db,
        json!({"ops": [{"db/id": "a", "obs/id": "o1"}, {"db/id": "b", "obs/description": "x"}]}),
    )
    .unwrap();
    let b = r.tempids["b"];
    let e = tx(&r.db_after, json!({"ops": [["add", b.0, "obs/id", "o1"]]})).unwrap_err();
    assert!(
        e.problems[0].contains("already belongs"),
        "{:?}",
        e.problems
    );
}

#[test]
fn installed_attributes_never_change_type() {
    let db = core();
    let e = db
        .with(
            &install(&json!([{"ident": "obs/amount", "type": "string"}])).unwrap(),
            &at(5),
        )
        .unwrap_err();
    assert!(e.problems[0].contains("never changes"), "{:?}", e.problems);
    let ok = db.with(
        &install(&json!([{"ident": "obs/amount", "type": "decimal", "doc": "a better doc"}]))
            .unwrap(),
        &at(5),
    );
    assert!(ok.is_ok(), "a doc can be improved");
}

#[test]
fn retract_entity_takes_components_and_refs_with_it() {
    let db = core();
    let r = tx(
        &db,
        json!({"ops": [
            {"db/id": "acct", "account/id": "a1"},
            {"db/id": "e", "entry/id": "e1", "entry/posting": [
                {"posting/account": "acct", "posting/quantity": "5.00"},
                {"posting/account": "acct", "posting/quantity": "-5.00"}]},
            {"note/target": "e", "note/text": "check this"}
        ]}),
    )
    .unwrap();
    let r2 = tx(
        &r.db_after,
        json!({"ops": [["retractEntity", {"lookup": ["entry/id", "e1"]}]]}),
    )
    .unwrap();
    let db2 = r2.db_after;
    // the entry's 3 facts, 2 x 2 posting facts, and the note's ref; the account and the note text stay
    assert_eq!(r2.datoms.iter().filter(|d| !d.added).count(), 3 + 4 + 1);
    assert!(db2.lookup(&k("entry/id"), &Value::str("e1")).is_none());
    assert!(db2.lookup(&k("account/id"), &Value::str("a1")).is_some());
}

#[test]
fn as_of_and_with_leave_the_original_untouched() {
    let db = core();
    let r1 = tx(&db, json!({"ops": [{"obs/id": "o", "obs/amount": "1.00"}]})).unwrap();
    let t1 = r1.tx.unwrap();
    let r2 = tx(
        &r1.db_after,
        json!({"ops": [{"obs/id": "o", "obs/amount": "2.00"}]}),
    )
    .unwrap();
    let o = r2.db_after.lookup(&k("obs/id"), &Value::str("o")).unwrap();
    let speculative = tx(
        &r2.db_after,
        json!({"ops": [{"obs/id": "o", "obs/amount": "3.00"}]}),
    )
    .unwrap();
    assert_eq!(
        speculative
            .db_after
            .get(o, &k("obs/amount"))
            .unwrap()
            .text(),
        "3.00"
    );
    assert_eq!(r2.db_after.get(o, &k("obs/amount")).unwrap().text(), "2.00");
    assert_eq!(
        r2.db_after
            .as_of(t1)
            .get(o, &k("obs/amount"))
            .unwrap()
            .text(),
        "1.00"
    );
}

#[test]
fn a_log_replays_to_the_same_database() {
    let db = core();
    let r = tx(
        &db,
        json!({"ops": [{"obs/id": "o", "obs/amount": "1.00"}, {"commodity/code": "USD"}]}),
    )
    .unwrap();
    let again = Db::from_log(r.db_after.log().iter().cloned()).unwrap();
    assert_eq!(again.len(), r.db_after.len());
    assert_eq!(again.basis(), r.db_after.basis());
    assert_eq!(again.schema().len(), r.db_after.schema().len());
}

#[test]
fn pull_follows_components_and_reverse_refs() {
    let db = core();
    let r = tx(
        &db,
        json!({"ops": [
            {"db/id": "d", "doc/id": "doc1", "doc/kind": "doc.kind/statement"},
            {"obs/id": "o1", "obs/doc": "d", "obs/amount": "-186.40"},
            {"db/id": "e", "entry/id": "e1", "entry/posting": [{"posting/quantity": "1.00"}]}
        ]}),
    )
    .unwrap();
    let d = r.tempids["d"];
    let p = Pattern::from_json(&json!(["doc/id", {"obs/_doc": ["obs/amount"]}])).unwrap();
    let out = pull(&r.db_after, &p, d);
    assert_eq!(out["obs/_doc"][0]["obs/amount"], json!("-186.40"));
    let e = pull(&r.db_after, &Pattern::wild(), r.tempids["e"]);
    assert_eq!(
        e["entry/posting"][0]["posting/quantity"],
        json!("1.00"),
        "{e}"
    );
}

#[test]
fn transactions_can_carry_metadata_and_be_annotated_later() {
    let db = core();
    let r = tx(
        &db,
        json!({"meta": {"tx/actor": "person:aram", "tx/reason": "opening"},
                           "ops": [{"obs/id": "o"}]}),
    )
    .unwrap();
    let t = r.tx.unwrap();
    assert_eq!(
        r.db_after.get(t, &k("tx/actor")),
        Some(&Value::str("person:aram"))
    );
    let r2 = tx(
        &r.db_after,
        json!({"ops": [["add", t.0, "tx/reason", "opening balances"]]}),
    )
    .unwrap();
    assert_eq!(
        r2.db_after.get(t, &k("tx/reason")).unwrap().text(),
        "opening balances"
    );
}

#[test]
fn metadata_alone_is_not_a_change() {
    let db = core();
    let r = tx(&db, json!({"meta": {"tx/actor": "x"}, "ops": []})).unwrap();
    assert_eq!(r.tx, None);
}

#[test]
fn unknown_ids_and_dangling_temps_are_refused() {
    let db = core();
    let e = tx(&db, json!({"ops": [["add", 999_999, "obs/id", "x"]]})).unwrap_err();
    assert!(e.problems[0].contains("no entity"));
    let e = tx(
        &db,
        json!({"ops": [["add", "a", "obs/id", "x"], ["add", "a", "obs/doc", "ghost"]]}),
    )
    .unwrap_err();
    assert!(
        e.problems[0].contains("no facts of its own"),
        "{:?}",
        e.problems
    );
}

#[test]
fn the_genesis_hash_is_frozen() {
    let h = encode::tx_hash(
        &encode::ZERO_HASH,
        ownpurse_facts::schema::GENESIS_TX,
        &ownpurse_facts::genesis_datoms(),
    );
    assert_eq!(h.hex(), GENESIS_HASH);
}

#[test]
fn every_value_type_has_a_frozen_encoding() {
    let d = |v: Value| {
        let mut out = Vec::new();
        encode::value_bytes(&mut out, &v);
        out.iter().map(|b| format!("{b:02x}")).collect::<String>()
    };
    assert_eq!(
        d(Value::Ref(ownpurse_facts::Eid(258))),
        "010000000000000102"
    );
    assert_eq!(d(Value::str("é")), "0200000002c3a9");
    assert_eq!(d(Value::Int(-1)), "03ffffffffffffffff");
    assert_eq!(
        d(Value::Dec("186.40".parse().unwrap())),
        "04000000063138362e3430"
    );
    assert_eq!(d(Value::Bool(true)), "0501");
    assert_eq!(
        d(Value::Date(
            ownpurse_facts::Date::parse("2026-03-14").unwrap()
        )),
        "060000000a323032362d30332d3134"
    );
    assert_eq!(d(Value::Kw(k("a/b"))), "0800000003612f62");
    assert_eq!(
        d(Value::Hash(ownpurse_facts::Hash([0xab; 32]))),
        format!("09{}", "ab".repeat(32))
    );
    let _ = EntRef::Tx;
}

const GENESIS_HASH: &str = "e075c758bbd97e841a11c58e255ff6f57af7ef657c15e26f2879f61884ab22ff";
