//! The lender's schedule for loan 2207, an Excel workbook: serial dates, cached formula results as floats,
//! and one scheduled payment that splits into interest and principal.

mod common;

use common::*;
use ownpurse_facts::{Date, Db, Kw, Value};
use ownpurse_ingest::sources::xlsx::{Xlsx, excel_date, float_decimal};
use rust_decimal::Decimal;

const SRC: &str = "xlsx:ridgeway-loan-2207";
const DOC: &str = "xlsx:ridgeway-loan-2207/Schedule";

fn load(db: &Db) -> Applied {
    ingest(
        db,
        &Xlsx,
        SRC,
        "xlsx/loan-2207-schedule.xlsx",
        &[("mapping", "xlsx/loan-2207.mapping.json")],
        "2026-03-20T17:20:00Z",
    )
}

fn dec(s: &str) -> Value {
    Value::Dec(s.parse::<Decimal>().unwrap())
}

fn date(s: &str) -> Value {
    Value::Date(Date::parse(s).unwrap())
}

fn obs(db: &Db, id: &str, attr: &'static str) -> Option<Value> {
    get(db, "obs/id", &format!("{DOC}/{id}"), attr)
}

#[test]
fn every_payment_row_loads() {
    let r = load(&core());
    assert!(r.tx.is_some());
    let payments = find(
        &r.db,
        "obs/kind",
        Value::Kw(Kw::lit("obs.kind/scheduled-payment")),
    );
    let parts = find(
        &r.db,
        "obs/kind",
        Value::Kw(Kw::lit("obs.kind/scheduled-payment-part")),
    );
    assert_eq!(
        (payments.len(), parts.len()),
        (12, 24),
        "twelve payments in 2026; the totals row is not one"
    );
    assert_eq!(
        obs(&r.db, "payment/15", "obs/date"),
        Some(date("2026-01-28"))
    );
    assert_eq!(
        obs(&r.db, "payment/26", "obs/date"),
        Some(date("2026-12-28"))
    );
    assert_eq!(
        get(&r.db, "doc/id", DOC, "sheet.doc/title"),
        Some(Value::str(
            "Ridgeway Federal Bank: loan 2207 amortisation schedule"
        ))
    );
    assert!(pressure_kinds(&r.plan).contains(&"merged".to_string()));
}

#[test]
fn the_march_payment_splits_into_interest_and_principal() {
    let r = load(&core());
    assert_eq!(
        obs(&r.db, "payment/17", "obs/date"),
        Some(date("2026-03-28"))
    );
    assert_eq!(
        obs(&r.db, "payment/17", "obs/amount"),
        Some(dec("-1126.84"))
    );
    assert_eq!(
        obs(&r.db, "payment/17/interest", "obs/amount"),
        Some(dec("-122.72"))
    );
    assert_eq!(
        obs(&r.db, "payment/17/principal", "obs/amount"),
        Some(dec("-1004.12"))
    );
    assert_eq!(
        obs(&r.db, "payment/17", "loan.schedule/balance-after"),
        Some(dec("-60355.88"))
    );
    // the balance before March, from February's row
    assert_eq!(
        obs(&r.db, "payment/16", "loan.schedule/balance-after"),
        Some(dec("-61360.00"))
    );
    assert_eq!(
        obs(&r.db, "payment/17/interest", "loan.schedule/component"),
        Some(Value::Kw(Kw::lit("loan.component/interest")))
    );
    assert_eq!(
        obs(&r.db, "payment/17/principal", "loan.schedule/account-code"),
        Some(Value::str("2500"))
    );
    let parent =
        r.db.lookup(
            &Kw::lit("obs/id"),
            &Value::str(&format!("{DOC}/payment/17")),
        )
        .unwrap();
    assert_eq!(
        obs(&r.db, "payment/17/interest", "obs/parent"),
        Some(Value::Ref(parent))
    );
    // every row's parts sum to its payment, and the totals row agrees
    assert!(
        !pressure_kinds(&r.plan)
            .iter()
            .any(|k| k == "unbalanced" || k == "check-failed"),
        "{:#?}",
        r.plan.pressure
    );
    assert_eq!(
        r.plan
            .pressure
            .iter()
            .filter(|p| p.kind == "derived-value")
            .count(),
        1 + 3
    );
}

#[test]
fn computed_cells_keep_their_formulas() {
    let r = load(&core());
    let formulas: Vec<Value> =
        r.db.attr(&Kw::lit("sheet.cell/formula"))
            .map(|a| r.db.attr_values(a.id).map(|(v, _)| v.clone()).collect())
            .unwrap();
    assert!(
        formulas.contains(&Value::str("=ROUND(F6*$B$2/12,2)")),
        "March's interest"
    );
    assert!(
        formulas.contains(&Value::str("=EDATE(B6,1)")),
        "March's date"
    );
    // the payment column is typed in: 12 payments × (date, interest, principal, balance) + 11 numbers
    assert_eq!(formulas.len(), 12 * 4 + 11);
}

#[test]
fn floats_in_the_cells_become_exact_cents() {
    let r = load(&core());
    // calamine reads 1004.1199999999999 from the cell; the text of the double, not arithmetic, gets the cents
    assert_eq!(
        float_decimal(1004.1199999999999),
        Some("1004.1199999999999".parse().unwrap())
    );
    let floats: Vec<_> = r
        .plan
        .pressure
        .iter()
        .filter(|p| p.kind == "float")
        .collect();
    assert!(
        floats
            .iter()
            .any(|p| p.at == "cell:Schedule!E7" && p.detail.contains("1004.1199999999999"))
    );
    assert!(
        floats
            .iter()
            .any(|p| p.at == "cell:Schedule!F6" && p.detail.contains("61359.99999999999"))
    );
}

#[test]
fn excel_serial_dates_convert_in_both_systems() {
    assert_eq!(excel_date(46109, false).unwrap().as_str(), "2026-03-28");
    assert_eq!(excel_date(1, false).unwrap().as_str(), "1900-01-01");
    assert_eq!(excel_date(59, false).unwrap().as_str(), "1900-02-28");
    assert_eq!(excel_date(60, false), None, "1900-02-29 never was");
    assert_eq!(excel_date(61, false).unwrap().as_str(), "1900-03-01");
    assert_eq!(excel_date(0, true).unwrap().as_str(), "1904-01-01");
    assert_eq!(
        excel_date(46109 - 1462, true).unwrap().as_str(),
        "2026-03-28"
    );
}

#[test]
fn reading_the_same_workbook_again_changes_nothing() {
    let r = load(&core());
    assert_eq!(load(&r.db).tx, None);
}
