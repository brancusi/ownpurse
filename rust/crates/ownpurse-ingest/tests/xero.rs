//! The Xero projector against the two sample pulls (2026-03-20 and 2026-04-02).

mod common;

use common::{find, get, ingest};
use ownpurse_facts::{Db, Kw, Value};
use ownpurse_ingest::roundtrip::{canonical_source, render};
use ownpurse_ingest::sources::xero::{CONFIG, Xero};
use rust_decimal::Decimal;
use serde_json::Value as Json;

const SRC: &str = "xero:tallowmere";
const ORDER: [&str; 11] = [
    "Organisations",
    "Currencies",
    "TaxRates",
    "Accounts",
    "TrackingCategories",
    "Contacts",
    "Invoices",
    "Payments",
    "BankTransactions",
    "BankTransfers",
    "ManualJournals",
];
const PULL1: &str = "2026-03-20T17:05:00Z";
const PULL2: &str = "2026-04-02T09:12:00Z";

fn load(db: &Db, pull: u8) -> Db {
    let at = if pull == 1 { PULL1 } else { PULL2 };
    ORDER.iter().fold(db.clone(), |db, n| {
        ingest(
            &db,
            &Xero,
            SRC,
            &format!("xero/pull-{pull}/{n}.json"),
            &[],
            at,
        )
        .db
    })
}

fn both() -> (Db, Db) {
    let one = load(&common::core(), 1);
    let two = load(&one, 2);
    (one, two)
}

fn doc(kind: &str, id: &str) -> String {
    format!("{SRC}/{kind}/{id}")
}

fn dec(s: &str) -> Value {
    Value::Dec(s.parse::<Decimal>().unwrap())
}

const INV_0415: &str = "c0ffee03-0000-4000-8000-000000000002";
const INV_0416: &str = "c0ffee03-0000-4000-8000-000000000003";
const DINNER: &str = "c0ffee05-0000-4000-8000-000000000003";

#[test]
fn reading_the_same_file_twice_changes_nothing() {
    let (one, _) = both();
    for n in ORDER {
        let r = ingest(
            &one,
            &Xero,
            SRC,
            &format!("xero/pull-1/{n}.json"),
            &[],
            PULL1,
        );
        assert_eq!(
            r.tx, None,
            "{n}: {} asserted, {} retracted",
            r.asserted, r.retracted
        );
    }
}

#[test]
fn an_older_read_never_overwrites_a_newer_one() {
    let (_, two) = both();
    let r = ingest(&two, &Xero, SRC, "xero/pull-1/Invoices.json", &[], PULL1);
    assert_eq!(
        r.tx, None,
        "{} asserted, {} retracted",
        r.asserted, r.retracted
    );
    assert!(
        common::pressure_kinds(&r.plan)
            .iter()
            .any(|k| k == "stale-read")
    );
    assert_eq!(
        get(&r.db, "doc/id", &doc("Invoice", INV_0415), "obs/amount"),
        Some(dec("1084.57"))
    );
}

#[test]
fn every_document_rebuilds_from_its_facts() {
    let (_, db) = both();
    let mut checked = 0;
    for n in ORDER {
        let text = std::fs::read_to_string(common::samples().join(format!("xero/pull-2/{n}.json")))
            .unwrap();
        let body: Json = serde_json::from_str(&text).unwrap();
        for (list, kind, idf) in CONFIG.collections {
            for item in body
                .get(list)
                .and_then(Json::as_array)
                .into_iter()
                .flatten()
            {
                let o = item.as_object().unwrap();
                let id = o[*idf].as_str().unwrap();
                let e = db
                    .lookup(&Kw::lit("doc/id"), &Value::str(&doc(kind, id)))
                    .unwrap();
                let want = canonical_source(&CONFIG, o);
                let got = render(&db, &CONFIG, e, &format!("xero.{kind}"));
                assert_eq!(got, want, "{kind} {id}");
                checked += 1;
            }
        }
    }
    // 1 organisation, 1 currency, 2 tax rates, 18 accounts, 1 tracking category, 10 contacts, 7 invoices,
    // 4 payments, 10 bank transactions, 1 transfer, 1 journal
    assert_eq!(checked, 56);
}

#[test]
fn an_edited_invoice_changes_only_what_changed() {
    let (one, two) = both();
    let inv = doc("Invoice", INV_0415);
    assert_eq!(get(&one, "doc/id", &inv, "obs/amount"), Some(dec("929.63")));
    assert_eq!(
        get(&two, "doc/id", &inv, "obs/amount"),
        Some(dec("1084.57"))
    );
    // the edit, datom by datom: the second read retracts the old values and asserts the new ones
    let e = two.lookup(&Kw::lit("doc/id"), &Value::str(&inv)).unwrap();
    let changed: Vec<String> = two
        .history(e)
        .iter()
        .filter(|d| !d.added)
        .map(|d| two.schema().ident(d.a).unwrap().to_string())
        .collect();
    let mut changed = changed;
    changed.sort();
    assert_eq!(
        changed,
        [
            "obs/amount",
            "xero.Invoice/AmountDue",
            "xero.Invoice/SubTotal",
            "xero.Invoice/Total",
            "xero.Invoice/TotalTax",
            "xero.Invoice/UpdatedDateUTC"
        ]
    );
}

#[test]
fn a_voided_invoice_keeps_its_history() {
    let (one, two) = both();
    let inv = doc("Invoice", INV_0416);
    let status = |db: &Db| get(db, "doc/id", &inv, "obs/status").map(|v| v.text());
    assert_eq!(status(&one).as_deref(), Some("obs.status/authorised"));
    assert_eq!(status(&two).as_deref(), Some("obs.status/voided"));
    let t1 = *one.txs().last().unwrap();
    assert_eq!(
        status(&two.as_of(t1)).as_deref(),
        Some("obs.status/authorised")
    );
}

#[test]
fn the_dinner_gains_a_reference_and_keeps_its_entry() {
    let (one, two) = both();
    let bt = doc("BankTransaction", DINNER);
    assert_eq!(get(&one, "doc/id", &bt, "obs/reference"), None);
    assert_eq!(
        get(&two, "doc/id", &bt, "obs/reference"),
        Some(Value::str("Client dinner, Ostley Joinery"))
    );
    assert_eq!(get(&two, "doc/id", &bt, "obs/amount"), Some(dec("-186.40")));
    let entry = two
        .lookup(&Kw::lit("entry/id"), &Value::str(&format!("{bt}#entry")))
        .unwrap();
    let links = find(&two, "link/from", Value::Ref(entry));
    assert_eq!(links.len(), 1, "the entry mirrors the bank transaction");
}

#[test]
fn every_mirror_entry_balances() {
    let (_, db) = both();
    let posting = db.attr(&Kw::lit("entry/posting")).unwrap().id;
    let qty = db.attr(&Kw::lit("posting/quantity")).unwrap().id;
    let entries: Vec<_> = db
        .attr_values(db.attr(&Kw::lit("entry/id")).unwrap().id)
        .map(|(_, e)| e)
        .collect();
    // 8 authorised spend/receive money transactions (not the transfer legs) and the posted journal
    assert_eq!(entries.len(), 9);
    for e in entries {
        let sum: Decimal = db
            .values(e, posting)
            .filter_map(Value::as_ref)
            .filter_map(|p| db.value(p, qty).and_then(Value::as_dec))
            .sum();
        assert!(sum.is_zero(), "{e:?} is off by {sum}");
    }
}

#[test]
fn the_operating_account_moves_by_the_months_bank_lines_xero_knows() {
    let (_, db) = both();
    // spend and receive money only; payments and the transfer post through other documents
    let total: Decimal = find(
        &db,
        "obs/kind",
        Value::Kw(Kw::lit("obs.kind/money-movement")),
    )
    .into_iter()
    .filter_map(|e| db.get(e, &Kw::lit("obs/amount")).and_then(Value::as_dec))
    .sum();
    // coffees -4.75 -4.75, dinner -186.40, rent -2,750.00, Stripe +2,318.06, loan -1,126.84, fee -12.50,
    // interest +1.37
    assert_eq!(total, "-1765.81".parse::<Decimal>().unwrap());
}

#[test]
fn payments_settle_their_invoices_with_an_amount() {
    let (_, db) = both();
    let settles = find(&db, "link/kind", Value::Kw(Kw::lit("link.kind/settles")));
    assert_eq!(settles.len(), 4);
    for l in settles {
        assert!(db.get(l, &Kw::lit("link/amount")).is_some());
    }
}

#[test]
fn the_pressure_is_reported() {
    let r = ingest(
        &common::core(),
        &Xero,
        SRC,
        "xero/pull-1/BankTransfers.json",
        &[],
        PULL1,
    );
    let kinds = common::pressure_kinds(&r.plan);
    for k in ["embedded-copy", "two-accounts"] {
        assert!(kinds.iter().any(|x| x == k), "{k} missing from {kinds:?}");
    }
}

#[test]
fn the_same_event_rule_joins_xeros_two_records_of_the_transfer() {
    let (_, db) = both();
    let tx = ownpurse_ingest::rules::same_event(&db);
    let r = db.with(&tx, &common::at("2026-04-02T10:00:00Z")).unwrap();
    let links = find(
        &r.db_after,
        "link/kind",
        Value::Kw(Kw::lit("link.kind/same-event")),
    );
    // each side of the transfer twice in Xero: the SPEND-TRANSFER / RECEIVE-TRANSFER bank transactions and the
    // BankTransfer's two legs; one link per side. The two identical coffees are two events, never linked.
    assert_eq!(links.len(), 2);
    let again = r
        .db_after
        .with(
            &ownpurse_ingest::rules::same_event(&r.db_after),
            &common::at("2026-04-02T10:00:01Z"),
        )
        .unwrap();
    assert_eq!(
        again.tx, None,
        "proposing the same links again changes nothing"
    );
}
