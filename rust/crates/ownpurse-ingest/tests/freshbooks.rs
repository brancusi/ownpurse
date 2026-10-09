//! FreshBooks: Saltmarsh Cupping Lab LLC's books, the supplier side of Xero bill SC-0057.

mod common;

use common::*;
use ownpurse_facts::{Date, Db, Eid, Kw, Value};
use ownpurse_ingest::sources::freshbooks::FreshBooks;
use rust_decimal::Decimal;
use std::str::FromStr;

const SRC: &str = "freshbooks:saltmarsh";
const PULL_1: &str = "2026-03-20T17:05:00Z";
const PULL_2: &str = "2026-04-02T09:12:00Z";
const FIRST: &[&str] = &[
    "freshbooks/identity.json",
    "freshbooks/clients.json",
    "freshbooks/invoices-2026-03-20.json",
    "freshbooks/expenses.json",
];

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn obs(db: &Db, id: &str, attr: &'static str) -> Option<Value> {
    get(db, "obs/id", &format!("{SRC}/{id}"), attr)
}

fn pull_1() -> Db {
    let mut db = core();
    for f in FIRST {
        db = ingest(&db, &FreshBooks, SRC, f, &[], PULL_1).db;
    }
    db
}

/// Both pulls: the payment, then the invoice as it reads once paid.
fn pull_2() -> Db {
    let db = ingest(
        &pull_1(),
        &FreshBooks,
        SRC,
        "freshbooks/payments.json",
        &[],
        PULL_2,
    )
    .db;
    ingest(
        &db,
        &FreshBooks,
        SRC,
        "freshbooks/invoices-2026-04-02.json",
        &[],
        PULL_2,
    )
    .db
}

/// The attributes a transaction touched, as `+attr` and `-attr`, sorted.
fn touched(db: &Db, tx: Eid) -> Vec<String> {
    let mut out: Vec<String> = db
        .log()
        .iter()
        .filter(|d| d.tx == tx && d.e != tx)
        .map(|d| {
            let a = db
                .schema()
                .ident(d.a)
                .map(Kw::to_string)
                .unwrap_or_default();
            format!("{}{a}", if d.added { '+' } else { '-' })
        })
        .collect();
    out.sort();
    out
}

#[test]
fn every_sample_loads() {
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(samples().join("freshbooks/manifest.json")).unwrap())
            .unwrap();
    let mut db = core();
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), 6);
    for f in files {
        let a = ingest(
            &db,
            &FreshBooks,
            f["source"].as_str().unwrap(),
            f["file"].as_str().unwrap(),
            &[],
            f["observed_at"].as_str().unwrap(),
        );
        assert!(a.tx.is_some(), "{f}");
        db = a.db;
    }
}

#[test]
fn reading_the_same_file_again_changes_nothing() {
    let mut db = core();
    for f in FIRST.iter().chain(&[
        "freshbooks/payments.json",
        "freshbooks/invoices-2026-04-02.json",
    ]) {
        db = ingest(&db, &FreshBooks, SRC, f, &[], PULL_2).db;
        let again = ingest(&db, &FreshBooks, SRC, f, &[], PULL_2);
        assert_eq!(again.tx, None, "{f}");
    }
}

/// A snapshot read says "this is the document now". Read an older file after a newer one and the invoice is
/// rolled back to unpaid: nothing in the model knows the file is older (FreshBooks' `updated`, QuickBooks'
/// `SyncToken` would say so, but the projector does not compare them).
#[test]
/// Still open: the old file is read late, so its read time is the newest. Only the source's own version
/// (`updated`) could say it is older; see the edges doc (`version` on `JsonSource`, `tx/source-time`).
fn an_older_file_read_late_rolls_the_invoice_back() {
    let a = ingest(
        &pull_2(),
        &FreshBooks,
        SRC,
        "freshbooks/invoices-2026-03-20.json",
        &[],
        PULL_2,
    );
    assert!(a.tx.is_some());
    assert_eq!(
        obs(&a.db, "invoice/57", "obs/status"),
        Some(Value::Kw(Kw::lit("obs.status/sent")))
    );
}

#[test]
fn an_older_read_replayed_with_its_own_time_is_skipped() {
    let a = ingest(
        &pull_2(),
        &FreshBooks,
        SRC,
        "freshbooks/invoices-2026-03-20.json",
        &[],
        PULL_1,
    );
    assert_eq!(a.tx, None);
    assert!(
        common::pressure_kinds(&a.plan)
            .iter()
            .any(|k| k == "stale-read")
    );
}

#[test]
fn the_paid_invoice_asserts_only_what_the_payment_changed() {
    let db = ingest(
        &pull_1(),
        &FreshBooks,
        SRC,
        "freshbooks/payments.json",
        &[],
        PULL_2,
    )
    .db;
    let a = ingest(
        &db,
        &FreshBooks,
        SRC,
        "freshbooks/invoices-2026-04-02.json",
        &[],
        PULL_2,
    );
    let tx = a.tx.expect("the change is a transaction");
    // the new file and the invoice's link to it; then what FreshBooks recomputed when the payment arrived:
    // four statuses, outstanding and paid, date_paid (new), updated, and the interpreted obs/status
    assert_eq!(
        touched(&a.db, tx),
        [
            "+doc/file",
            "+file/media-type",
            "+file/name",
            "+file/sha256",
            "+file/size",
            "+freshbooks.invoice.outstanding/amount",
            "+freshbooks.invoice.paid/amount",
            "+freshbooks.invoice/date_paid",
            "+freshbooks.invoice/display_status",
            "+freshbooks.invoice/payment_status",
            "+freshbooks.invoice/status",
            "+freshbooks.invoice/updated",
            "+freshbooks.invoice/v3_status",
            "+obs/status",
            "-freshbooks.invoice.outstanding/amount",
            "-freshbooks.invoice.paid/amount",
            "-freshbooks.invoice/display_status",
            "-freshbooks.invoice/payment_status",
            "-freshbooks.invoice/status",
            "-freshbooks.invoice/updated",
            "-freshbooks.invoice/v3_status",
            "-obs/status",
        ]
    );
    assert_eq!(a.retracted, 8);
}

#[test]
fn the_invoice_is_a_receivable_of_362_18_on_11_march() {
    let db = pull_1();
    assert_eq!(
        obs(&db, "invoice/57", "obs/amount"),
        Some(Value::Dec(d("362.18")))
    );
    assert_eq!(
        obs(&db, "invoice/57", "obs/date"),
        Some(Value::Date(Date::parse("2026-03-11").unwrap()))
    );
    assert_eq!(
        obs(&db, "invoice/57", "obs/status"),
        Some(Value::Kw(Kw::lit("obs.status/sent")))
    );
    let paid = pull_2();
    assert_eq!(
        obs(&paid, "invoice/57", "obs/status"),
        Some(Value::Kw(Kw::lit("obs.status/paid")))
    );
}

#[test]
fn money_objects_become_amount_and_commodity() {
    let db = pull_1();
    let usd = db
        .lookup(&Kw::lit("commodity/code"), &Value::str("USD"))
        .unwrap();
    assert_eq!(
        obs(&db, "invoice/57", "obs/commodity"),
        Some(Value::Ref(usd))
    );
    // the mirror keeps each money object as its own entity
    let amount = get(
        &db,
        "freshbooks/path",
        &format!("{SRC}/invoice/57/amount"),
        "freshbooks.invoice.amount/amount",
    );
    assert_eq!(amount, Some(Value::Dec(d("362.18"))));
}

#[test]
fn the_lines_add_up_to_the_invoice() {
    let db = pull_1();
    assert_eq!(
        obs(&db, "invoice/57/lines/1", "obs/amount"),
        Some(Value::Dec(d("294.00")))
    );
    assert_eq!(
        obs(&db, "invoice/57/lines/1", "obs/description"),
        Some(Value::str("Cupping sessions"))
    );
    assert_eq!(
        obs(&db, "invoice/57/lines/2", "obs/amount"),
        Some(Value::Dec(d("68.18")))
    );
    assert_eq!(
        obs(&db, "invoice/57/lines/2", "obs/description"),
        Some(Value::str("Sample shipping"))
    );
    // quantity arrives as text; the mirror reads it as a number
    let qty = get(
        &db,
        "freshbooks/path",
        &format!("{SRC}/invoice/57/lines/1"),
        "freshbooks.invoice.lines/qty",
    );
    assert_eq!(qty, Some(Value::Dec(d("3"))));
}

#[test]
fn the_invoice_carries_what_a_rule_needs_to_meet_xero_bill_sc_0057() {
    let db = pull_1();
    // SC-0057 in Xero: 362.18 on 2026-03-11 from Saltmarsh Cupping Lab LLC, reference "Invoice 57"
    assert_eq!(
        obs(&db, "invoice/57", "obs/reference"),
        Some(Value::str("0057"))
    );
    assert_eq!(
        obs(&db, "invoice/57", "obs/counterparty"),
        Some(Value::str("Tallowmere Coffee Roasting Co."))
    );
    assert_eq!(
        get(&db, "book/id", SRC, "book/name"),
        Some(Value::str("Saltmarsh Cupping Lab LLC"))
    );
}

#[test]
fn the_payment_is_money_in_and_settles_the_invoice() {
    let db = pull_2();
    assert_eq!(
        obs(&db, "payment/31", "obs/amount"),
        Some(Value::Dec(d("362.18")))
    );
    assert_eq!(
        obs(&db, "payment/31", "obs/date"),
        Some(Value::Date(Date::parse("2026-03-24").unwrap()))
    );
    assert_eq!(
        obs(&db, "payment/31", "obs/counterparty"),
        Some(Value::str("Tallowmere Coffee Roasting Co."))
    );
    let link = format!("settles|{SRC}/payment/31|{SRC}/invoice/57");
    assert_eq!(
        get(&db, "link/id", &link, "link/amount"),
        Some(Value::Dec(d("362.18")))
    );
}

#[test]
fn plain_number_foreign_keys_become_refs() {
    let db = pull_1();
    let inv = db
        .lookup(
            &Kw::lit("doc/id"),
            &Value::str(&format!("{SRC}/invoice/57")),
        )
        .unwrap();
    let client = db
        .lookup(&Kw::lit("doc/id"), &Value::str(&format!("{SRC}/client/12")))
        .unwrap();
    assert_eq!(
        db.get(inv, &Kw::lit("freshbooks.invoice/customerid")),
        Some(&Value::Ref(client))
    );
    // the invoice's second id for itself stays a number
    assert_eq!(
        db.get(inv, &Kw::lit("freshbooks.invoice/invoiceid")),
        Some(&Value::Dec(d("57")))
    );
    // 0 means none, and stays 0
    let line = db
        .lookup(
            &Kw::lit("freshbooks/path"),
            &Value::str(&format!("{SRC}/invoice/57/lines/1")),
        )
        .unwrap();
    assert_eq!(
        db.get(line, &Kw::lit("freshbooks.invoice.lines/expenseid")),
        Some(&Value::Dec(d("0")))
    );
}

#[test]
fn the_expense_is_money_out_with_no_account() {
    let db = pull_1();
    assert_eq!(
        obs(&db, "expense/88", "obs/amount"),
        Some(Value::Dec(d("-54.30")))
    );
    assert_eq!(
        obs(&db, "expense/88", "obs/counterparty"),
        Some(Value::str("Pelican Freight Co."))
    );
    assert_eq!(obs(&db, "expense/88", "obs/account"), None);
}

#[test]
fn there_are_no_mirror_entries() {
    let db = pull_2();
    assert!(
        db.attr(&Kw::lit("entry/id"))
            .is_some_and(|a| db.attr_values(a.id).next().is_none())
    );
}

#[test]
fn wall_clock_times_stay_text() {
    let db = pull_1();
    assert_eq!(
        get(
            &db,
            "doc/id",
            &format!("{SRC}/invoice/57"),
            "freshbooks.invoice/updated"
        ),
        Some(Value::str("2026-03-11 14:05:12"))
    );
}

#[test]
fn the_pressure_it_reports() {
    let mut db = core();
    let mut kinds = std::collections::BTreeSet::new();
    for f in FIRST {
        let a = ingest(&db, &FreshBooks, SRC, f, &[], PULL_1);
        kinds.extend(pressure_kinds(&a.plan));
        db = a.db;
    }
    for want in [
        "derived-value",
        "enum-int",
        "envelope",
        "flat-copy",
        "money-object",
        "no-ledger",
        "no-zone",
        "null",
        "scalar-ref",
        "two-ids",
        "zero-as-none",
    ] {
        assert!(kinds.contains(want), "{want} not in {kinds:?}");
    }
    assert!(!kinds.contains("type-conflict"), "{kinds:?}");
}
