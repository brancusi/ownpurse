//! OFX 1.02 (SGML) statement for account 4821, March 2026.

mod common;

use common::*;
use ownpurse_facts::{Db, Eid, Kw, Value};
use ownpurse_ingest::sources::bank_csv::BankCsv;
use ownpurse_ingest::sources::ofx::Ofx;
use rust_decimal::Decimal;
use std::collections::BTreeSet;
use std::path::Path;
use std::str::FromStr;

const SRC: &str = "bank:ridgeway-4821";
const OFX: &str = "bank/2026-03.ofx";
const OFX_AT: &str = "2026-04-02T13:15:00Z";

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn ofx(db: &Db, file: &str) -> Applied {
    ingest(db, &Ofx, SRC, file, &[], OFX_AT)
}

fn csv_late(db: &Db) -> Applied {
    ingest(
        db,
        &BankCsv,
        SRC,
        "bank/activity-2026-04-02.csv",
        &[("mapping", "bank/ridgeway-csv-mapping.json")],
        "2026-04-02T13:10:00Z",
    )
}

fn val(db: &Db, e: Eid, a: &'static str) -> Option<Value> {
    db.get(e, &Kw::lit(a)).cloned()
}

fn id(db: &Db, e: Eid) -> String {
    val(db, e, "obs/id").map(|v| v.text()).unwrap_or_default()
}

fn of_kind(db: &Db, kind: &'static str, prefix: &str) -> Vec<Eid> {
    find(db, "obs/kind", Value::Kw(Kw::lit(kind)))
        .into_iter()
        .filter(|e| id(db, *e).starts_with(prefix))
        .collect()
}

fn ofx_lines(db: &Db) -> Vec<Eid> {
    of_kind(db, "obs.kind/money-movement", "bank:ridgeway-4821|ofx|")
}

fn amount(db: &Db, e: Eid) -> Decimal {
    val(db, e, "obs/amount").and_then(|v| v.as_dec()).unwrap()
}

fn altered(from: &str, to: &str, name: &str) -> String {
    let text = std::fs::read_to_string(samples().join(OFX)).unwrap();
    assert!(text.contains(from), "no {from:?}");
    let p = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&p, text.replacen(from, to, 1)).unwrap();
    p.to_str().unwrap().to_string()
}

#[test]
fn the_statement_loads() {
    let a = ofx(&core(), OFX);
    assert!(a.tx.is_some());
    let lines = ofx_lines(&a.db);
    assert_eq!(lines.len(), 13);
    let total: Decimal = lines.iter().map(|e| amount(&a.db, *e)).sum();
    assert_eq!(d("24310.55") + total, d("13413.22"));
    let bal = of_kind(&a.db, "obs.kind/balance", "bank:ridgeway-4821|ofx|");
    assert_eq!(bal.len(), 2);
    let ledger = bal
        .iter()
        .find(|e| {
            val(&a.db, **e, "bank.ofx/balance-kind")
                == Some(Value::Kw(Kw::lit("bank.ofx.balance/ledger")))
        })
        .unwrap();
    assert_eq!(amount(&a.db, *ledger), d("13413.22"));
    assert_eq!(
        val(&a.db, *ledger, "obs/date").map(|v| v.text()),
        Some("2026-03-31".into())
    );
    let acct =
        a.db.lookup(
            &Kw::lit("account/id"),
            &Value::str("ridgeway:checking:4821"),
        )
        .unwrap();
    assert_eq!(val(&a.db, acct, "account/mask"), Some(Value::str("4821")));
    let kinds = pressure_kinds(&a.plan);
    for k in [
        "account-identity",
        "balance-unchecked",
        "placeholder-time",
        "truncated",
        "zone-mixed",
    ] {
        assert!(kinds.contains(&k.to_string()), "{k} missing from {kinds:?}");
    }
    let unchecked = a
        .plan
        .pressure
        .iter()
        .find(|p| p.kind == "balance-unchecked")
        .unwrap();
    assert!(
        unchecked.detail.contains("opening of 24,310.55"),
        "{}",
        unchecked.detail
    );
}

#[test]
fn reingesting_the_statement_changes_nothing() {
    let a = ofx(&core(), OFX);
    assert_eq!(ofx(&a.db, OFX).tx, None);
}

#[test]
fn fitid_is_the_identity() {
    let a = ofx(&core(), OFX);
    let coffee = a.db.lookup(
        &Kw::lit("obs/id"),
        &Value::str("bank:ridgeway-4821|ofx|ridgeway:checking:4821|4821202603060002"),
    );
    let coffee = coffee.expect("the second coffee by FITID");
    // the same statement downloaded again later (other bytes, same lines) names the same observations
    let again = altered(
        "<DTSERVER>20260402091500",
        "<DTSERVER>20260403080000",
        "2026-03-again.ofx",
    );
    let b = ofx(&a.db, &again);
    assert!(b.tx.is_some(), "a new file is recorded");
    assert_eq!(ofx_lines(&b.db).len(), 13, "no new lines");
    assert_eq!(
        b.db.lookup(&Kw::lit("obs/id"), &Value::str(&id(&a.db, coffee))),
        Some(coffee)
    );
    let ra = b.db.attr(&Kw::lit("obs/region")).unwrap().id;
    assert_eq!(b.db.values(coffee, ra).count(), 2, "one region per file");
    // a reused FITID with a different amount is pressure
    let reused = altered(
        "<TRNAMT>-4.75\n<FITID>4821202603060002",
        "<TRNAMT>-4.85\n<FITID>4821202603060002",
        "2026-03-reused.ofx",
    );
    let c = ofx(&a.db, &reused);
    assert!(pressure_kinds(&c.plan).contains(&"fitid-reused".to_string()));
}

#[test]
fn the_coffees_and_the_check() {
    let db = ofx(&core(), OFX).db;
    let coffees = find(
        &db,
        "obs/fingerprint",
        Value::str("4821|2026-03-06|-4.75|HARBORSTREETCOFFEE"),
    );
    assert_eq!(coffees.len(), 2, "two FITIDs, one fingerprint");
    let check = find(&db, "obs/reference", Value::str("1042"));
    assert_eq!(check.len(), 1);
    assert_eq!(amount(&db, check[0]), d("-3215.47"));
    assert_eq!(
        val(&db, check[0], "bank.ofx/trntype"),
        Some(Value::str("CHECK"))
    );
    assert_eq!(
        val(&db, check[0], "bank.ofx/dtposted"),
        Some(Value::str("20260309120000[-4:EDT]"))
    );
    assert_eq!(
        val(&db, check[0], "obs/date").map(|v| v.text()),
        Some("2026-03-09".into())
    );
}

#[test]
fn csv_and_ofx_lines_share_fingerprints_but_stay_apart() {
    let a = csv_late(&core());
    let b = ofx(&a.db, OFX);
    let fps = |prefix: &str| -> BTreeSet<String> {
        of_kind(&b.db, "obs.kind/money-movement", prefix)
            .iter()
            .filter_map(|e| val(&b.db, *e, "obs/fingerprint").map(|v| v.text()))
            .collect()
    };
    let (csv, ofx) = (
        fps("bank:ridgeway-4821|csv|"),
        fps("bank:ridgeway-4821|ofx|"),
    );
    assert_eq!(
        (csv.len(), ofx.len()),
        (12, 12),
        "the coffees share one fingerprint"
    );
    let only_ofx: Vec<_> = ofx.difference(&csv).collect();
    // the truncated name is the one line a plain fingerprint rule misses
    assert_eq!(
        only_ofx,
        vec!["4821|2026-03-24|-362.18|SALTMARSHCUPPINGLABLLCPAYMEN"]
    );
    assert_eq!(
        of_kind(&b.db, "obs.kind/money-movement", "bank:ridgeway-4821|").len(),
        26
    );
}

#[test]
fn the_ledger_balance_is_checked_against_the_csv() {
    let a = csv_late(&core());
    let b = ofx(&a.db, OFX);
    let kinds = pressure_kinds(&b.plan);
    assert!(!kinds.iter().any(|k| k.starts_with("balance")), "{kinds:?}");
    let broken = altered(
        "<BALAMT>13413.22\n<DTASOF>20260331",
        "<BALAMT>13413.23\n<DTASOF>20260331",
        "2026-03-broken.ofx",
    );
    let c = ofx(&a.db, &broken);
    let bad: Vec<_> = c
        .plan
        .pressure
        .iter()
        .filter(|p| p.kind == "balance-disagrees")
        .collect();
    assert_eq!(bad.len(), 1, "{:?}", c.plan.pressure);
    assert!(
        bad[0].detail.contains("13,413.23") && bad[0].detail.contains("13,413.22"),
        "{}",
        bad[0].detail
    );
}

#[test]
fn ofx_2_xml_reads_the_same_lines() {
    // the same statement as OFX 2 XML: every leaf closed
    let sgml = std::fs::read_to_string(samples().join(OFX)).unwrap();
    let mut xml =
        String::from("<?xml version=\"1.0\"?>\n<?OFX OFXHEADER=\"200\" VERSION=\"211\"?>\n");
    for l in sgml.lines().skip_while(|l| !l.starts_with('<')) {
        match (l.find('>'), l.starts_with("</")) {
            (Some(i), false) if i + 1 < l.len() => {
                let tag = &l[1..i];
                xml.push_str(&format!("{l}</{tag}>\n"));
            }
            _ => {
                xml.push_str(l);
                xml.push('\n');
            }
        }
    }
    let p = Path::new(env!("CARGO_TARGET_TMPDIR")).join("2026-03.xml.ofx");
    std::fs::write(&p, xml).unwrap();
    let a = ofx(&core(), p.to_str().unwrap());
    assert_eq!(ofx_lines(&a.db).len(), 13);
    let rent = find(
        &a.db,
        "obs/fingerprint",
        Value::str("4821|2026-03-20|-2750|LARKSPURPROPERTYRENT"),
    );
    assert_eq!(
        val(&a.db, rent[0], "bank.ofx/memo"),
        Some(Value::str("ACH DEBIT LARKSPUR PROPERTY MGMT & LEASING"))
    );
}
