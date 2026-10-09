//! The 2026 budget, a Google Sheet, read twice: wide to long, periods, computed cells, and what a second read
//! with a moved row, a changed figure and a renamed category does to the log.

mod common;

use common::*;
use ownpurse_facts::{Db, Kw, Value};
use ownpurse_ingest::sources::sheets::Sheets;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

const SRC: &str = "sheet:tallowmere-budget-2026";
const MAPPING: (&str, &str) = ("mapping", "sheets/budget-2026.mapping.json");

fn read1(db: &Db) -> Applied {
    ingest(
        db,
        &Sheets,
        SRC,
        "sheets/budget-2026.values.json",
        &[("formulas", "sheets/budget-2026.formulas.json"), MAPPING],
        "2026-03-20T17:10:00Z",
    )
}

fn read2(db: &Db) -> Applied {
    ingest(
        db,
        &Sheets,
        SRC,
        "sheets/budget-2026-read-2.values.json",
        &[
            ("formulas", "sheets/budget-2026-read-2.formulas.json"),
            MAPPING,
        ],
        "2026-04-02T09:15:00Z",
    )
}

fn dec(s: &str) -> Value {
    Value::Dec(s.parse::<Decimal>().unwrap())
}

fn figure(db: &Db, line: &str, month: &str) -> Option<Value> {
    get(
        db,
        "obs/id",
        &format!("{SRC}/Budget/{line}/{month}"),
        "obs/amount",
    )
}

fn budget_obs(db: &Db) -> usize {
    find(db, "obs/kind", Value::Kw(Kw::lit("obs.kind/budget"))).len()
}

/// (attribute, added) → count, for one ingest's datoms.
fn tally(a: &Applied) -> BTreeMap<(String, bool), usize> {
    let mut out = BTreeMap::new();
    // the attributes live in the db after; the datoms carry their ids
    let report = a.db.log().iter().rev().take(a.asserted + a.retracted);
    for d in report {
        let ident =
            a.db.schema()
                .by_id(d.a)
                .map(|x| x.ident.to_string())
                .unwrap_or_default();
        *out.entry((ident, d.added)).or_default() += 1;
    }
    out
}

#[test]
fn the_first_read_is_one_figure_per_category_per_month() {
    let r = read1(&core());
    assert!(r.tx.is_some());
    // ten categories × twelve months, less the ten blank months of Design and branding
    assert_eq!(budget_obs(&r.db), 110);
    assert_eq!(
        figure(&r.db, "Wholesale coffee sales", "2026-03"),
        Some(dec("9200.00"))
    );
    assert_eq!(figure(&r.db, "Rent", "2026-03"), Some(dec("-2750.00")));
    assert_eq!(
        figure(&r.db, "Interest expense", "2026-03"),
        Some(dec("-122.72"))
    );
    assert_eq!(
        figure(&r.db, "Design and branding", "2026-03"),
        Some(dec("-2400.00"))
    );
    assert_eq!(
        figure(&r.db, "Design and branding", "2026-04"),
        None,
        "a blank cell is no figure, not zero"
    );
    let id = format!("{SRC}/Budget/Rent/2026-03");
    assert_eq!(
        get(&r.db, "obs/id", &id, "budget/period-start"),
        Some(Value::Date(
            ownpurse_facts::Date::parse("2026-03-01").unwrap()
        ))
    );
    assert_eq!(
        get(&r.db, "obs/id", &id, "budget/period-end"),
        Some(Value::Date(
            ownpurse_facts::Date::parse("2026-03-31").unwrap()
        ))
    );
    assert_eq!(
        get(&r.db, "obs/id", &id, "obs/date"),
        None,
        "a plan figure has a period, not a date"
    );
}

#[test]
fn computed_cells_are_checked_not_stored_and_formula_figures_are_marked() {
    let r = read1(&core());
    // the totals rows are not lines, and no figure comes from the Total column
    for name in ["Total expenses", "Net"] {
        assert!(
            get(
                &r.db,
                "budget.line/id",
                &format!("{SRC}/Budget/{name}"),
                "budget.line/name"
            )
            .is_none()
        );
    }
    assert!(
        !r.db
            .log()
            .iter()
            .any(|d| d.v == Value::str(&format!("{SRC}/Budget/Rent/Total")))
    );
    // every computed cell agreed with its formula
    assert!(
        !pressure_kinds(&r.plan).contains(&"check-failed".to_string()),
        "{:#?}",
        r.plan.pressure
    );
    let derived = r
        .plan
        .pressure
        .iter()
        .find(|p| p.kind == "derived-value")
        .unwrap();
    assert!(
        derived.detail.contains("Total expenses, Net, Total"),
        "{}",
        derived.detail
    );
    assert!(
        derived.detail.contains("36 cells agree"),
        "{}",
        derived.detail
    );
    // Rent's February to December are `=B8`-style formulas: kept as figures, their cells marked
    let formula = Kw::lit("sheet.cell/formula");
    let marked: Vec<_> =
        r.db.attr(&formula)
            .map(|a| r.db.attr_values(a.id).map(|(v, _)| v.clone()).collect())
            .unwrap();
    assert_eq!(
        marked.len(),
        22,
        "Rent and Interest income, eleven months each"
    );
    assert!(marked.contains(&Value::str("=C8")));
    assert!(pressure_kinds(&r.plan).contains(&"formula-entry".to_string()));
}

#[test]
fn reading_the_same_sheet_again_changes_nothing() {
    let r = read1(&core());
    let again = read1(&r.db);
    assert_eq!(again.tx, None);
}

#[test]
fn the_second_read_moves_a_row_changes_a_figure_and_renames_a_line() {
    let r1 = read1(&core());
    let rent = r1
        .db
        .lookup(
            &Kw::lit("obs/id"),
            &Value::str(&format!("{SRC}/Budget/Bank fees/2026-03")),
        )
        .unwrap();
    let r2 = read2(&r1.db);
    let t = tally(&r2);
    let n = |a: &str, added: bool| t.get(&(a.to_string(), added)).copied().unwrap_or(0);

    // the moved row: same figures, same entities; only the positions of the lines between change
    let still = r2
        .db
        .lookup(
            &Kw::lit("obs/id"),
            &Value::str(&format!("{SRC}/Budget/Bank fees/2026-03")),
        )
        .unwrap();
    assert_eq!(rent, still);
    assert_eq!(
        n("budget.line/position", true),
        3 + 1,
        "Interest expense, Cupping and quality, Bank fees; and the renamed line"
    );
    assert_eq!(n("budget.line/position", false), 3 + 1);

    // the changed figure: one retraction, one assertion
    assert_eq!(
        figure(&r2.db, "Green coffee", "2026-06"),
        Some(dec("-3650.00"))
    );
    assert_eq!(
        n("obs/amount", false),
        1 + 2,
        "Green coffee June, and the renamed line's two figures"
    );
    assert_eq!(n("obs/amount", true), 1 + 2);

    // the rename: the old line and its two figures are retracted, a new line with two figures asserted
    assert_eq!(figure(&r2.db, "Design and branding", "2026-03"), None);
    assert_eq!(
        figure(&r2.db, "Brand and design", "2026-03"),
        Some(dec("-2400.00"))
    );
    assert_eq!(n("budget.line/id", false), 1);
    assert_eq!(n("budget.line/id", true), 1);
    assert_eq!(n("obs/id", false), 2);
    assert_eq!(n("obs/id", true), 2);
    assert!(
        get(
            &r2.db,
            "budget.line/id",
            &format!("{SRC}/Budget/Brand and design"),
            "budget.line/account-code"
        )
        .is_none()
    );
    assert!(pressure_kinds(&r2.plan).contains(&"identity".to_string()));
    assert!(pressure_kinds(&r2.plan).contains(&"unmapped-key".to_string()));
    assert_eq!(budget_obs(&r2.db), 110);

    // every figure the second read shows gets a region in the new file: the cost of exact regions
    assert_eq!(n("region/id", true), 110);
    assert_eq!(n("obs/region", true), 110);
    assert_eq!(n("sheet.cell/formula", true), 22);
    assert_eq!(n("doc/file", true), 1);

    // In all: 29 retractions are the change (the old line's 7 facts, its 2 figures' 9 facts each, 3 positions,
    // 1 amount). Of 511 assertions, 26 are the change; 462 are regions (110 cells × 4, 22 formulas), 9 are the
    // two new files and the doc's new file, 14 are the transaction's own facts.
    assert_eq!((r2.asserted, r2.retracted), (511, 29));
}
