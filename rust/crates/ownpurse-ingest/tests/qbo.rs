//! QuickBooks Online: Quillfeather Design Consultancy LLP's books, the supplier side of Xero bill QF-1031.

mod common;

use common::*;
use ownpurse_facts::{Date, Db, Eid, Kw, Value};
use ownpurse_ingest::sources::qbo::Qbo;
use rust_decimal::Decimal;
use std::str::FromStr;

const SRC: &str = "qbo:quillfeather";
const PULL_1: &str = "2026-03-20T17:05:00Z";
const PULL_2: &str = "2026-04-02T09:12:00Z";
const FIRST: &[&str] = &[
    "qbo/CompanyInfo.json",
    "qbo/Account.json",
    "qbo/Customer.json",
    "qbo/Item.json",
    "qbo/Invoice.json",
    "qbo/Payment.json",
    "qbo/Purchase.json",
    "qbo/JournalEntry.json",
];

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn obs(db: &Db, id: &str, attr: &'static str) -> Option<Value> {
    get(db, "obs/id", &format!("{SRC}/{id}"), attr)
}

/// The first pull, every file in order.
fn pull_1() -> Db {
    let mut db = core();
    for f in FIRST {
        db = ingest(&db, &Qbo, SRC, f, &[], PULL_1).db;
    }
    db
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

fn postings(db: &Db, entry: &str) -> Vec<Decimal> {
    let e = db
        .lookup(&Kw::lit("entry/id"), &Value::str(entry))
        .expect(entry);
    let pa = db.attr(&Kw::lit("entry/posting")).unwrap().id;
    db.values(e, pa)
        .filter_map(|p| db.get(p.as_ref()?, &Kw::lit("posting/quantity"))?.as_dec())
        .collect()
}

#[test]
fn every_sample_loads() {
    let manifest: serde_json::Value =
        serde_json::from_slice(&std::fs::read(samples().join("qbo/manifest.json")).unwrap())
            .unwrap();
    let mut db = core();
    let files = manifest["files"].as_array().unwrap();
    assert_eq!(files.len(), 9);
    for f in files {
        let a = ingest(
            &db,
            &Qbo,
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
    let db = pull_1();
    for f in FIRST {
        let again = ingest(&db, &Qbo, SRC, f, &[], PULL_1);
        assert_eq!(again.tx, None, "{f}");
    }
}

#[test]
fn the_second_read_asserts_only_what_changed() {
    let db = pull_1();
    let a = ingest(&db, &Qbo, SRC, "qbo/Invoice-2026-04-02.json", &[], PULL_2);
    let tx = a.tx.expect("the edit is a transaction");
    // the new file (sha256, name, media type, size) and the invoice's link to it, then the four fields
    // QuickBooks changed: SyncToken, PrivateNote, CustomerMemo and MetaData.LastUpdatedTime
    assert_eq!(
        touched(&a.db, tx),
        [
            "+doc/file",
            "+file/media-type",
            "+file/name",
            "+file/sha256",
            "+file/size",
            "+qbo.Invoice.CustomerMemo/value",
            "+qbo.Invoice.MetaData/LastUpdatedTime",
            "+qbo.Invoice/PrivateNote",
            "+qbo.Invoice/SyncToken",
            "-qbo.Invoice.CustomerMemo/value",
            "-qbo.Invoice.MetaData/LastUpdatedTime",
            "-qbo.Invoice/PrivateNote",
            "-qbo.Invoice/SyncToken",
        ]
    );
    assert_eq!((a.asserted - tx_meta(&a.db, tx), a.retracted), (9, 4));
    assert_eq!(
        get(
            &a.db,
            "doc/id",
            &format!("{SRC}/Invoice/130"),
            "qbo.Invoice/SyncToken"
        ),
        Some(Value::str("2"))
    );
}

/// Datoms about the transaction itself (projector, actor, evidence, pressure, …).
fn tx_meta(db: &Db, tx: Eid) -> usize {
    db.log().iter().filter(|d| d.tx == tx && d.e == tx).count()
}

#[test]
fn the_invoice_is_a_receivable_of_2400_on_4_march() {
    let db = pull_1();
    assert_eq!(
        obs(&db, "Invoice/130", "obs/amount"),
        Some(Value::Dec(d("2400.00")))
    );
    assert_eq!(
        obs(&db, "Invoice/130", "obs/date"),
        Some(Value::Date(Date::parse("2026-03-04").unwrap()))
    );
    assert_eq!(
        obs(&db, "Invoice/130", "obs/kind"),
        Some(Value::Kw(Kw::lit("obs.kind/invoice")))
    );
    assert_eq!(
        obs(&db, "Invoice/130", "obs/status"),
        Some(Value::Kw(Kw::lit("obs.status/paid")))
    );
}

#[test]
fn the_invoice_carries_what_a_rule_needs_to_meet_xero_bill_qf_1031() {
    let db = pull_1();
    // QF-1031 in Xero: 2,400.00 on 2026-03-04 from Quillfeather Design Consultancy LLP
    assert_eq!(
        obs(&db, "Invoice/130", "obs/reference"),
        Some(Value::str("1031"))
    );
    assert_eq!(
        obs(&db, "Invoice/130", "obs/counterparty"),
        Some(Value::str("Tallowmere Coffee Roasting Co."))
    );
    assert_eq!(
        get(&db, "book/id", SRC, "book/name"),
        Some(Value::str("Quillfeather Design Consultancy LLP"))
    );
    let usd = db
        .lookup(&Kw::lit("commodity/code"), &Value::str("USD"))
        .unwrap();
    assert_eq!(
        obs(&db, "Invoice/130", "obs/commodity"),
        Some(Value::Ref(usd))
    );
}

#[test]
fn the_subtotal_line_is_mirrored_but_not_observed() {
    let db = pull_1();
    let lines = find(&db, "obs/kind", Value::Kw(Kw::lit("obs.kind/line")));
    let parent = db
        .lookup(
            &Kw::lit("obs/id"),
            &Value::str(&format!("{SRC}/Invoice/130")),
        )
        .unwrap();
    let of_invoice: Vec<_> = lines
        .iter()
        .filter(|l| db.get(**l, &Kw::lit("obs/parent")) == Some(&Value::Ref(parent)))
        .collect();
    assert_eq!(of_invoice.len(), 1);
    assert_eq!(
        db.get(*of_invoice[0], &Kw::lit("obs/description")),
        Some(&Value::str("Packaging design, spring range"))
    );
    let subtotal = db
        .lookup(
            &Kw::lit("qbo/path"),
            &Value::str(&format!("{SRC}/Invoice/130/Line/@1")),
        )
        .expect("the subtotal line is in the mirror");
    assert_eq!(
        db.get(subtotal, &Kw::lit("qbo.Invoice.Line/DetailType")),
        Some(&Value::str("SubTotalLineDetail"))
    );
}

#[test]
fn the_payment_is_money_in_and_settles_the_invoice() {
    let db = pull_1();
    assert_eq!(
        obs(&db, "Payment/131", "obs/amount"),
        Some(Value::Dec(d("2400.00")))
    );
    assert_eq!(
        obs(&db, "Payment/131", "obs/date"),
        Some(Value::Date(Date::parse("2026-03-18").unwrap()))
    );
    let link = format!("settles|{SRC}/Payment/131|{SRC}/Invoice/130");
    assert_eq!(
        get(&db, "link/id", &link, "link/amount"),
        Some(Value::Dec(d("2400")))
    );
    let checking = db
        .lookup(
            &Kw::lit("doc/id"),
            &Value::str(&format!("{SRC}/Account/35")),
        )
        .unwrap();
    assert_eq!(
        obs(&db, "Payment/131", "obs/account"),
        Some(Value::Ref(checking))
    );
}

#[test]
fn the_card_purchase_is_money_out() {
    let db = pull_1();
    assert_eq!(
        obs(&db, "Purchase/132", "obs/amount"),
        Some(Value::Dec(d("-49.00")))
    );
    assert_eq!(
        obs(&db, "Purchase/132", "obs/counterparty"),
        Some(Value::str("Lumen Vector Software"))
    );
    let card = db
        .lookup(
            &Kw::lit("doc/id"),
            &Value::str(&format!("{SRC}/Account/41")),
        )
        .unwrap();
    assert_eq!(
        obs(&db, "Purchase/132", "obs/account"),
        Some(Value::Ref(card))
    );
}

#[test]
fn refs_by_field_name_point_at_the_named_kind() {
    let db = pull_1();
    let inv = db
        .lookup(
            &Kw::lit("doc/id"),
            &Value::str(&format!("{SRC}/Invoice/130")),
        )
        .unwrap();
    let cust = db
        .lookup(
            &Kw::lit("doc/id"),
            &Value::str(&format!("{SRC}/Customer/58")),
        )
        .unwrap();
    assert_eq!(
        db.get(inv, &Kw::lit("qbo.Invoice/CustomerRef")),
        Some(&Value::Ref(cust))
    );
    assert_eq!(
        db.get(cust, &Kw::lit("party/name")),
        Some(&Value::str("Tallowmere Coffee Roasting Co."))
    );
    // EntityRef's kind comes from its type field
    let pur = db
        .lookup(
            &Kw::lit("doc/id"),
            &Value::str(&format!("{SRC}/Purchase/132")),
        )
        .unwrap();
    let vendor = db
        .get(pur, &Kw::lit("qbo.Purchase/EntityRef"))
        .and_then(Value::as_ref)
        .unwrap();
    assert_eq!(
        db.get(vendor, &Kw::lit("doc/id")),
        Some(&Value::str(&format!("{SRC}/Vendor/64")))
    );
}

#[test]
fn timestamps_with_an_offset_are_stored_in_utc() {
    let db = pull_1();
    // 2026-03-04T09:18:52-08:00
    let created = get(
        &db,
        "qbo/path",
        &format!("{SRC}/Invoice/130/MetaData"),
        "qbo.Invoice.MetaData/CreateTime",
    );
    assert_eq!(
        created.map(|v| v.text()),
        Some("2026-03-04T17:18:52.000000Z".to_string())
    );
}

#[test]
fn every_mirror_entry_balances() {
    let db = pull_1();
    for (doc, want) in [
        ("Invoice/130", ["2400", "-2400"]),
        ("Payment/131", ["2400", "-2400"]),
        ("Purchase/132", ["-49", "49"]),
        ("JournalEntry/133", ["85", "-85"]),
    ] {
        let mut ps = postings(&db, &format!("{SRC}/{doc}#entry"));
        ps.sort();
        let mut want: Vec<Decimal> = want.iter().map(|w| d(w)).collect();
        want.sort();
        assert_eq!(ps, want, "{doc}");
        assert_eq!(ps.iter().sum::<Decimal>(), Decimal::ZERO, "{doc}");
    }
}

#[test]
fn every_posting_names_an_account() {
    let db = pull_1();
    let pa = db.attr(&Kw::lit("posting/account")).unwrap().id;
    let q = db.attr(&Kw::lit("posting/quantity")).unwrap().id;
    let ps: Vec<Eid> = db.attr_values(q).map(|(_, e)| e).collect();
    assert_eq!(ps.len(), 8);
    assert!(ps.iter().all(|p| db.value(*p, pa).is_some()));
}

#[test]
fn the_pressure_it_reports() {
    let mut db = core();
    let mut kinds = std::collections::BTreeSet::new();
    for f in FIRST {
        let a = ingest(&db, &Qbo, SRC, f, &[], PULL_1);
        kinds.extend(pressure_kinds(&a.plan));
        db = a.db;
    }
    for want in [
        "derived-value",
        "embedded-copy",
        "envelope",
        "missing-context",
        "offset-dropped",
        "polymorphic-line",
        "polymorphic-ref",
        "ref-by-field-name",
        "serialisation-artefact",
        "sign",
        "value-wrapper",
    ] {
        assert!(kinds.contains(want), "{want} not in {kinds:?}");
    }
    assert!(!kinds.contains("unbalanced"));
    assert!(!kinds.contains("unresolved-ref"), "{kinds:?}");
}

/// The generic mirror identifies a list item by its id field, or else by its position, in one namespace. An
/// invoice's sales line has `Id: "1"`; the subtotal line QuickBooks appends has no Id and sits at position 1:
/// both got the key `…/Line/1` and the transaction was refused; positions are now keyed `@1`. The projector still does not list
/// `Id` in `child_ids`.
#[test]
fn line_ids_and_positions_no_longer_collide_in_the_generic_mirror() {
    use ownpurse_facts::encode::sha256;
    use ownpurse_ingest::jsonmirror::{self, JsonSource};
    use ownpurse_ingest::{FileIn, Input};
    static BY_ID: JsonSource = JsonSource {
        ns: "qbotest",
        root: &["QueryResponse"],
        collections: &[("Invoice", "Invoice", "Id")],
        refs: &[],
        ref_ids: &[],
        owned: &[],
        child_ids: &["Id"],
        when: |_, _| None,
    };
    let bytes = std::fs::read(samples().join("qbo/Invoice.json")).unwrap();
    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    let input = Input {
        file: FileIn {
            hash: sha256(&bytes),
            name: "Invoice.json",
            media_type: "",
            bytes: &bytes,
            role: "main",
        },
        with: vec![],
        source: source(SRC),
        observed_at: at(PULL_1),
    };
    let db = core();
    let first = jsonmirror::mirror(&BY_ID, &db, &input, &body).unwrap();
    let defs = ownpurse_facts::install(&serde_json::Value::Array(first.install)).unwrap();
    let db = db.with(&defs, &at(PULL_1)).unwrap().db_after;
    let m = jsonmirror::mirror(&BY_ID, &db, &input, &body).unwrap();
    // fixed in the mirror: a position is keyed `@1`, an id `1`
    db.with(&m.snap.diff(&db, &|_| true), &at(PULL_1)).unwrap();
}
