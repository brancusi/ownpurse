//! Bank activity downloads (CSV): two overlapping downloads of account 4821 for March 2026.

mod common;

use common::*;
use ownpurse_facts::encode::sha256;
use ownpurse_facts::{Db, Eid, Kw, Value};
use ownpurse_ingest::sources::bank_csv::BankCsv;
use ownpurse_ingest::{FileIn, Input};
use rust_decimal::Decimal;
use std::path::Path;
use std::str::FromStr;

const SRC: &str = "bank:ridgeway-4821";
const MAP: (&str, &str) = ("mapping", "bank/ridgeway-csv-mapping.json");
const EARLY: &str = "bank/activity-2026-03-20.csv";
const LATE: &str = "bank/activity-2026-04-02.csv";
const EARLY_AT: &str = "2026-03-20T17:15:00Z";
const LATE_AT: &str = "2026-04-02T13:10:00Z";

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn early(db: &Db) -> Applied {
    ingest(db, &BankCsv, SRC, EARLY, &[MAP], EARLY_AT)
}

fn late(db: &Db) -> Applied {
    ingest(db, &BankCsv, SRC, LATE, &[MAP], LATE_AT)
}

/// Bank-line observations from the CSV projector: (posted, pending-or-lapsed).
fn lines(db: &Db) -> (Vec<Eid>, Vec<Eid>) {
    let all: Vec<Eid> = find(
        db,
        "obs/kind",
        Value::Kw(Kw::lit("obs.kind/money-movement")),
    )
    .into_iter()
    .filter(|e| id(db, *e).starts_with("bank:ridgeway-4821|csv"))
    .collect();
    all.into_iter()
        .partition(|e| id(db, *e).starts_with("bank:ridgeway-4821|csv|"))
}

fn id(db: &Db, e: Eid) -> String {
    db.get(e, &Kw::lit("obs/id"))
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn val(db: &Db, e: Eid, a: &'static str) -> Option<Value> {
    db.get(e, &Kw::lit(a)).cloned()
}

fn amount(db: &Db, e: Eid) -> Decimal {
    val(db, e, "obs/amount").and_then(|v| v.as_dec()).unwrap()
}

fn status(db: &Db, e: Eid) -> String {
    val(db, e, "obs/status")
        .map(|v| v.text())
        .unwrap_or_default()
}

/// A copy of a sample with one substring replaced, written where the test helper can read it.
fn altered(file: &str, from: &str, to: &str, name: &str) -> String {
    let text = std::fs::read_to_string(samples().join(file)).unwrap();
    assert!(text.contains(from), "{file} has no {from:?}");
    let p = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    std::fs::write(&p, text.replacen(from, to, 1)).unwrap();
    p.to_str().unwrap().to_string()
}

/// The month table in samples/README.md: date, text, amount.
const MONTH: [(&str, &str, &str); 13] = [
    ("2026-03-02", "DEPOSIT OSTLEY JOINERY", "1846.13"),
    ("2026-03-06", "HARBOR STREET COFFEE", "-4.75"),
    ("2026-03-06", "HARBOR STREET COFFEE", "-4.75"),
    ("2026-03-09", "CHECK 1042", "-3215.47"),
    ("2026-03-13", "COPPER POT BISTRO", "-186.40"),
    ("2026-03-16", "TRANSFER TO SAV 9067", "-5000.00"),
    ("2026-03-18", "QUILLFEATHER DESIGN ACH", "-2400.00"),
    ("2026-03-20", "LARKSPUR PROPERTY RENT", "-2750.00"),
    ("2026-03-24", "SALTMARSH CUPPING LAB", "-362.18"),
    ("2026-03-25", "STRIPE PAYOUT", "2318.06"),
    ("2026-03-28", "RIDGEWAY LOAN PMT 2207", "-1126.84"),
    ("2026-03-31", "MONTHLY SERVICE FEE", "-12.50"),
    ("2026-03-31", "INTEREST PAID", "1.37"),
];

#[test]
fn both_downloads_load() {
    let a = early(&core());
    assert!(a.tx.is_some());
    let (posted, pending) = lines(&a.db);
    assert_eq!((posted.len(), pending.len()), (8, 1));
    let b = late(&core());
    let (posted, pending) = lines(&b.db);
    assert_eq!((posted.len(), pending.len()), (13, 0));
    let acct =
        b.db.lookup(
            &Kw::lit("account/id"),
            &Value::str("ridgeway:checking:4821"),
        )
        .unwrap();
    assert_eq!(val(&b.db, acct, "account/mask"), Some(Value::str("4821")));
    assert_eq!(
        val(&b.db, acct, "account/name"),
        Some(Value::str("Operating checking"))
    );
    assert_eq!(
        val(&b.db, acct, "account/institution"),
        Some(Value::str("Ridgeway Federal Bank"))
    );
}

#[test]
fn reingesting_a_download_changes_nothing() {
    let a = early(&core());
    assert_eq!(early(&a.db).tx, None);
    let b = late(&a.db);
    assert_eq!(late(&b.db).tx, None);
    assert_eq!(
        early(&b.db).tx,
        None,
        "the older download again, after the newer one"
    );
}

#[test]
fn the_later_download_adds_only_new_lines_and_lapses_the_pending_one() {
    let a = early(&core());
    let (before, pend) = lines(&a.db);
    assert_eq!(status(&a.db, pend[0]), "obs.status/pending");
    assert_eq!(amount(&a.db, pend[0]), d("-1.00"));
    assert!(pressure_kinds(&a.plan).contains(&"pending".to_string()));
    let b = late(&a.db);
    let (after, pend_after) = lines(&b.db);
    assert_eq!(
        after.len() - before.len(),
        5,
        "03-24 to 03-31: five new lines"
    );
    assert!(
        before.iter().all(|e| after.contains(e)),
        "the overlap names the same lines"
    );
    assert_eq!(pend_after, pend, "the pending line stays, as a witness");
    assert_eq!(status(&b.db, pend[0]), "obs.status/lapsed");
    assert!(
        b.plan.pressure.iter().any(|p| p.kind == "pending-lapsed"),
        "{:?}",
        b.plan.pressure
    );
    for kind in ["vanished", "order-lost"] {
        assert!(!pressure_kinds(&a.plan).contains(&kind.to_string()));
        assert!(!pressure_kinds(&b.plan).contains(&kind.to_string()));
    }
    let pending_at = a
        .plan
        .pressure
        .iter()
        .find(|p| p.kind == "pending")
        .unwrap();
    assert_eq!(pending_at.at, "row:5");
    // an overlapping line now has a region in each download
    let rent = after
        .iter()
        .find(|e| id(&b.db, **e).contains("LARKSPUR"))
        .unwrap();
    assert_eq!(
        b.db.values(*rent, b.db.attr(&Kw::lit("obs/region")).unwrap().id)
            .count(),
        2
    );
}

#[test]
fn an_older_download_read_late_does_not_revive_the_pending_line() {
    let b = late(&core());
    let a = early(&b.db);
    let (posted, pend) = lines(&a.db);
    assert_eq!(
        posted.len(),
        13,
        "no new posted lines from the older download"
    );
    assert_eq!(status(&a.db, pend[0]), "obs.status/lapsed");
    assert!(pressure_kinds(&a.plan).contains(&"out-of-order".to_string()));
}

#[test]
fn the_two_coffees_are_two_observations() {
    let db = late(&early(&core()).db).db;
    let coffees = find(
        &db,
        "obs/fingerprint",
        Value::str("4821|2026-03-06|-4.75|HARBORSTREETCOFFEE"),
    );
    assert_eq!(coffees.len(), 2);
    let mut seen: Vec<(i64, Decimal)> = coffees
        .iter()
        .map(|e| {
            let n = match val(&db, *e, "bank.csv/occurrence") {
                Some(Value::Int(n)) => n,
                other => panic!("{other:?}"),
            };
            (
                n,
                val(&db, *e, "bank.csv/balance-after")
                    .and_then(|v| v.as_dec())
                    .unwrap(),
            )
        })
        .collect();
    seen.sort();
    assert_eq!(seen, vec![(1, d("26151.93")), (2, d("26147.18"))]);
}

#[test]
fn amounts_dates_and_signs_match_the_month() {
    let db = late(&core()).db;
    let (posted, _) = lines(&db);
    let mut got: Vec<(String, String, Decimal)> = posted
        .iter()
        .map(|e| {
            (
                val(&db, *e, "obs/date").unwrap().text(),
                val(&db, *e, "obs/description").unwrap().text(),
                amount(&db, *e),
            )
        })
        .collect();
    got.sort();
    let mut want: Vec<(String, String, Decimal)> = MONTH
        .iter()
        .map(|(a, b, c)| ((*a).to_string(), (*b).to_string(), d(c)))
        .collect();
    want.sort();
    assert_eq!(got, want);
    let total: Decimal = posted.iter().map(|e| amount(&db, *e)).sum();
    assert_eq!(d("24310.55") + total, d("13413.22"));
    for e in &posted {
        assert_eq!(
            val(&db, *e, "obs/status"),
            Some(Value::Kw(Kw::lit("obs.status/posted")))
        );
    }
}

#[test]
fn the_check_carries_its_number() {
    let db = late(&core()).db;
    let e = find(&db, "obs/reference", Value::str("1042"));
    assert_eq!(e.len(), 1);
    assert_eq!(amount(&db, e[0]), d("-3215.47"));
    assert_eq!(val(&db, e[0], "bank.csv/type"), Some(Value::str("CHECK")));
    assert_eq!(
        val(&db, e[0], "obs/fingerprint"),
        Some(Value::str("4821|2026-03-09|-3215.47|CHECK1042"))
    );
    // line 14 of the file: three preamble lines, the header, then newest first
    let region = val(&db, e[0], "obs/region")
        .and_then(|v| v.as_ref())
        .unwrap();
    assert_eq!(
        val(&db, region, "region/locator"),
        Some(Value::str("row:14"))
    );
}

#[test]
fn running_balances_check_and_a_broken_one_is_pressure() {
    for a in [early(&core()), late(&core())] {
        assert!(
            !a.plan
                .pressure
                .iter()
                .any(|p| p.kind.starts_with("balance")),
            "{:?}",
            a.plan.pressure
        );
    }
    let broken = altered(
        LATE,
        "\"22,745.31\"",
        "\"22,745.32\"",
        "activity-broken-balance.csv",
    );
    let a = ingest(&core(), &BankCsv, SRC, &broken, &[MAP], LATE_AT);
    let bad: Vec<_> = a
        .plan
        .pressure
        .iter()
        .filter(|p| p.kind == "balance-mismatch")
        .collect();
    // the dinner's balance is a cent high, and so the next line no longer follows from it
    assert_eq!(bad.len(), 2, "{bad:?}");
    assert!(bad[0].detail.contains("22,745.32") || bad[1].detail.contains("22,745.32"));
    // the line's amount is not touched: the balance is evidence, not a correction
    let dinner = find(
        &a.db,
        "obs/fingerprint",
        Value::str("4821|2026-03-13|-186.4|COPPERPOTBISTRO"),
    );
    assert_eq!(amount(&a.db, dinner[0]), d("-186.40"));
    assert!(a.tx.is_some(), "the pressure is written with the facts");
}

#[test]
fn a_mapping_for_another_account_is_refused() {
    let csv = std::fs::read(samples().join(EARLY)).unwrap();
    let map = std::fs::read_to_string(samples().join(MAP.1))
        .unwrap()
        .replace("\"4821\"", "\"9067\"");
    let input = Input {
        file: FileIn {
            hash: sha256(&csv),
            name: EARLY,
            media_type: "text/csv",
            bytes: &csv,
            role: "main",
        },
        with: vec![FileIn {
            hash: sha256(map.as_bytes()),
            name: MAP.1,
            media_type: "application/json",
            bytes: map.as_bytes(),
            role: "mapping",
        }],
        source: source(SRC),
        observed_at: at(EARLY_AT),
    };
    let err = ownpurse_ingest::plan(&core(), &BankCsv, &input).unwrap_err();
    assert!(err.0.contains("ending 9067"), "{err}");
    let no_map = Input {
        with: vec![],
        ..input.clone()
    };
    assert!(
        ownpurse_ingest::plan(&core(), &BankCsv, &no_map)
            .unwrap_err()
            .0
            .contains("mapping")
    );
}

#[test]
fn the_mapping_and_the_window_are_recorded() {
    let db = early(&core()).db;
    let docs = find(
        &db,
        "doc/kind",
        Value::Kw(Kw::lit("doc.kind/bank-download")),
    );
    assert_eq!(docs.len(), 1);
    assert_eq!(
        val(&db, docs[0], "doc/period-start").map(|v| v.text()),
        Some("2026-03-01".into())
    );
    assert_eq!(
        val(&db, docs[0], "doc/period-end").map(|v| v.text()),
        Some("2026-03-20".into())
    );
    let map = val(&db, docs[0], "bank.csv/mapping")
        .and_then(|v| v.as_ref())
        .unwrap();
    assert!(val(&db, map, "file/sha256").is_some());
}
