//! Excel workbooks (.xlsx), read with a mapping an agent wrote. The first mapping is a lender's loan
//! amortisation schedule.
//!
//! A workbook is re-read whole (a snapshot). Each schedule row is one scheduled payment
//! (`obs.kind/scheduled-payment`, status `obs.status/scheduled`) with two child observations, the interest
//! and the principal it splits into (`loan.schedule/component`), whose amounts sum to the payment's. The
//! balance after the payment is an extra on the payment (`loan.schedule/balance-after`).
//!
//! Cells: calamine gives the cached result of a formula as an `f64`. It is turned into an exact decimal
//! through its shortest round-trip text (no float arithmetic), then rounded to the mapping's decimals, and a
//! value that needed rounding (`1004.1199999999999`) is reported. Dates are serial day numbers in the 1900 or
//! 1904 system. Each value's cell is a region; a computed cell carries its formula (`sheet.cell/formula`)
//! and every cell says which attribute it supplied (`sheet.cell/field`).

use crate::snapshot::Snapshot;
use crate::{IngestError, Input, Pressure, Projection, Projector, temp};
use calamine::{Data, Range, Reader, Xlsx as Workbook};
use ownpurse_facts::value::parse_decimal;
use ownpurse_facts::{Date, Db, Kw, ValIn, Value, time};
use rust_decimal::{Decimal, RoundingStrategy};
use serde_json::{Value as Json, json};
use std::collections::BTreeSet;
use std::io::Cursor;

pub struct Xlsx;

const ID: &str = "xlsx@1";

fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

fn install() -> Vec<Json> {
    let defs = json!([
        {"ident": "loan.schedule/loan", "type": "string", "doc": "The lender's loan number"},
        {"ident": "loan.schedule/payment-number", "type": "int"},
        {"ident": "loan.schedule/balance-after", "type": "decimal", "doc": "What the schedule says is owed after this payment (borrower's side: negative)"},
        {"ident": "loan.schedule/component", "type": "keyword", "doc": "loan.component/interest or loan.component/principal"},
        {"ident": "loan.schedule/account-code", "type": "string", "doc": "The book account the mapping says this part belongs to"},
        {"ident": "sheet.cell/formula", "type": "string", "doc": "The formula behind a cell's value, when a formula computed it"},
        {"ident": "sheet.cell/field", "type": "keyword", "doc": "The attribute this cell supplied, e.g. obs/date"},
        {"ident": "sheet.doc/title", "type": "string", "doc": "The title a sheet or workbook shows above its table"}
    ]);
    defs.as_array().cloned().unwrap_or_default()
}

fn owns(a: &Kw) -> bool {
    match a.as_str() {
        "obs/region" => false,
        "doc/kind" | "doc/external-id" | "doc/period-start" | "doc/period-end"
        | "sheet.doc/title" => true,
        _ => matches!(a.ns(), "obs" | "loan.schedule"),
    }
}

/// The calendar date of an Excel serial day number. In the 1900 system day 1 is 1900-01-01 and day 60 is
/// 1900-02-29, a day that never was (Lotus 1-2-3's leap-year bug, kept by Excel); it has no date. In the
/// 1904 system day 0 is 1904-01-01.
pub fn excel_date(serial: i64, date1904: bool) -> Option<Date> {
    if date1904 {
        return Some(Date::from_days(time::days_from_civil(1904, 1, 1) + serial));
    }
    match serial {
        ..=0 | 60 => None,
        1..=59 => Some(Date::from_days(
            time::days_from_civil(1899, 12, 31) + serial,
        )),
        _ => Some(Date::from_days(
            time::days_from_civil(1899, 12, 30) + serial,
        )),
    }
}

/// An exact decimal from a float cell, through its shortest round-trip text: `1004.1199999999999` stays
/// exactly that. No float arithmetic.
pub fn float_decimal(f: f64) -> Option<Decimal> {
    if !f.is_finite() {
        return None;
    }
    parse_decimal(&format!("{f}")).ok()
}

impl Projector for Xlsx {
    fn id(&self) -> &'static str {
        ID
    }

    fn reads(&self) -> &'static str {
        "Excel workbooks (.xlsx) with a mapping; first: a loan amortisation schedule"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let bad = |m: String| IngestError(format!("{}: {m}", input.file.name));
        let mapping: Json = input
            .companion("mapping")
            .ok_or_else(|| bad("a workbook needs a mapping (--with mapping=<file>)".into()))
            .and_then(|f| {
                serde_json::from_slice(f.bytes).map_err(|e| bad(format!("mapping: not JSON ({e})")))
            })?;
        let map = Mapping::read(&mapping).map_err(bad)?;
        if let Some(p) = crate::stale(db, &format!("{}/{}", input.source.id, map.sheet), input) {
            return Ok(p);
        }
        let mut wb: Workbook<_> = Workbook::new(Cursor::new(input.file.bytes))
            .map_err(|e| bad(format!("not an .xlsx workbook ({e})")))?;
        let date1904 = wb.has_1904_epoch();
        let values = wb
            .worksheet_range(&map.sheet)
            .map_err(|e| bad(format!("sheet {:?}: {e}", map.sheet)))?;
        let formulas = wb
            .worksheet_formula(&map.sheet)
            .map_err(|e| bad(format!("sheet {:?} formulas: {e}", map.sheet)))?;
        let merged = wb.merge_cells_by_sheet_name(&map.sheet).unwrap_or_default();
        let mut s = Schedule {
            db,
            input,
            map: &map,
            values: &values,
            formulas: &formulas,
            date1904,
            snap: Snapshot::new(),
            pressure: Vec::new(),
        };
        if date1904 {
            s.press(
                "date-system",
                "xl/workbook.xml",
                "the workbook counts days from 1904",
            );
        }
        for m in &merged {
            let at = format!(
                "cell:{}!{}:{}",
                map.sheet,
                a1(m.start.0, m.start.1),
                a1(m.end.0, m.end.1)
            );
            s.press(
                "merged",
                at,
                "a merged range: its value is in the top-left cell, the rest read as empty",
            );
        }
        s.run()?;
        let tx = s.snap.diff(db, &owns);
        Ok(Projection {
            install: install(),
            tx,
            pressure: s.pressure,
        })
    }
}

struct Mapping {
    sheet: String,
    header_row: u32,
    key_column: String,
    loan: String,
    currency: String,
    date: String,
    payment: String,
    interest: String,
    principal: String,
    balance: String,
    decimals: u32,
    interest_account: Option<String>,
    principal_account: Option<String>,
}

impl Mapping {
    fn read(j: &Json) -> Result<Mapping, String> {
        let s = |key: &str| j.get(key).and_then(Json::as_str).map(str::to_string);
        let need = |key: &str| s(key).ok_or_else(|| format!("mapping needs \"{key}\""));
        let col = |key: &str| {
            j.get("columns")
                .and_then(|c| c.get(key))
                .and_then(Json::as_str)
                .map(str::to_string)
                .ok_or_else(|| format!("mapping needs \"columns\": {{\"{key}\": <header>}}"))
        };
        let acct = |key: &str| {
            j.get("accounts")
                .and_then(|c| c.get(key))
                .and_then(Json::as_str)
                .map(str::to_string)
        };
        Ok(Mapping {
            sheet: need("sheet")?,
            header_row: j
                .get("header_row")
                .and_then(Json::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .ok_or("mapping needs \"header_row\" (1-based)")?,
            key_column: need("key_column")?,
            loan: need("loan")?,
            currency: s("currency").unwrap_or_else(|| "USD".into()),
            date: col("date")?,
            payment: col("payment")?,
            interest: col("interest")?,
            principal: col("principal")?,
            balance: col("balance")?,
            decimals: j
                .get("decimals")
                .and_then(Json::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .unwrap_or(2),
            interest_account: acct("interest"),
            principal_account: acct("principal"),
        })
    }
}

/// (0-based row, 0-based col) → `A1`.
fn a1(row: u32, col: u32) -> String {
    format!("{}{}", super::sheets::col_letters(col), row + 1)
}

struct Schedule<'a, 'i> {
    db: &'a Db,
    input: &'a Input<'i>,
    map: &'a Mapping,
    values: &'a Range<Data>,
    formulas: &'a Range<String>,
    date1904: bool,
    snap: Snapshot,
    pressure: Vec<Pressure>,
}

/// One payment row, read.
struct Row {
    n: i64,
    row: u32,
    date: Date,
    payment: Decimal,
    interest: Decimal,
    principal: Decimal,
    balance: Decimal,
}

impl Schedule<'_, '_> {
    fn press(&mut self, kind: &str, at: impl Into<String>, detail: impl Into<String>) {
        self.pressure.push(Pressure::new(kind, at, detail));
    }

    fn add(&mut self, key: &str, a: &'static str, v: impl Into<ValIn>) {
        self.snap.add(key, &k(a), v);
    }

    fn src(&self) -> &str {
        &self.input.source.id
    }

    fn cell(&self, row: u32, col: u32) -> String {
        format!("cell:{}!{}", self.map.sheet, a1(row, col))
    }

    fn formula(&self, row: u32, col: u32) -> Option<String> {
        self.formulas
            .get_value((row, col))
            .filter(|f| !f.is_empty())
            .map(|f| format!("={f}"))
    }

    fn text(&self, row: u32, col: u32) -> String {
        match self.values.get_value((row, col)) {
            Some(Data::String(s)) => s.trim().to_string(),
            _ => String::new(),
        }
    }

    /// An amount cell as an exact decimal, rounded to the mapping's decimals.
    fn amount(&mut self, row: u32, col: u32) -> Option<Decimal> {
        let at = self.cell(row, col);
        let exact = match self.values.get_value((row, col))? {
            Data::Int(i) => Decimal::from(*i),
            Data::Float(f) => {
                let Some(d) = float_decimal(*f) else {
                    self.press("unreadable", at, format!("{f} is not a finite number"));
                    return None;
                };
                d
            }
            Data::String(s) => {
                let plain: String = s
                    .chars()
                    .filter(|c| !matches!(c, '$' | ',' | ' '))
                    .collect();
                let Ok(d) = parse_decimal(&plain) else {
                    self.press("unreadable", at, format!("{s:?} is not an amount"));
                    return None;
                };
                self.press(
                    "text-number",
                    at.clone(),
                    format!("an amount typed as text: {s:?}"),
                );
                d
            }
            Data::Empty => return None,
            other => {
                self.press("unreadable", at, format!("{other:?} is not an amount"));
                return None;
            }
        };
        let cents =
            exact.round_dp_with_strategy(self.map.decimals, RoundingStrategy::MidpointAwayFromZero);
        if cents != exact {
            self.press(
                "float",
                at,
                format!(
                    "the cell holds the binary float {exact}; read to {} places as {cents}",
                    self.map.decimals
                ),
            );
        }
        Some(cents)
    }

    fn date(&mut self, row: u32, col: u32) -> Option<Date> {
        let at = self.cell(row, col);
        let serial = match self.values.get_value((row, col))? {
            Data::DateTime(dt) => float_decimal(dt.as_f64())?,
            Data::Float(f) => {
                self.press(
                    "date-format",
                    at.clone(),
                    "a day number with no date format",
                );
                float_decimal(*f)?
            }
            Data::Int(i) => Decimal::from(*i),
            Data::DateTimeIso(s) => return Date::parse(s.get(..10)?).ok(),
            _ => return None,
        };
        if !serial.fract().is_zero() {
            self.press(
                "time-of-day",
                at.clone(),
                format!("serial {serial} has a time of day; the date is kept"),
            );
        }
        let day = i64::try_from(serial.trunc()).ok()?;
        let d = excel_date(day, self.date1904);
        if d.is_none() {
            self.press("date", at, format!("serial {day} is not a calendar date"));
        }
        d
    }

    /// A region for one cell, with its formula when computed and the attribute it supplied.
    fn region(&mut self, row: u32, col: u32, field: &'static str) -> String {
        let h = self.input.file.hash;
        let locator = self.cell(row, col);
        let id = format!("{}#{locator}", h.hex());
        let key = format!("region:{id}");
        self.snap.ent(&key, &k("region/id"), Value::str(&id));
        let fk = format!("file:{}", h.hex());
        self.snap.refer(&fk, &k("file/sha256"), Value::Hash(h));
        self.add(&key, "region/file", ValIn::Ref(temp(&fk)));
        self.add(&key, "region/locator", Value::str(&locator));
        self.add(&key, "sheet.cell/field", Value::Kw(k(field)));
        if let Some(f) = self.formula(row, col) {
            self.add(&key, "sheet.cell/formula", Value::str(&f));
        }
        key
    }

    fn run(&mut self) -> Result<(), IngestError> {
        let map = self.map;
        let hr = map.header_row.saturating_sub(1);
        let (_, end) = self.values.end().unwrap_or((0, 0));
        let start_col = self.values.start().map_or(0, |s| s.1);
        let headers: Vec<(u32, String)> =
            (start_col..=end).map(|c| (c, self.text(hr, c))).collect();
        let col = |name: &str| headers.iter().find(|(_, h)| h == name).map(|(c, _)| *c);
        let need = |name: &str| {
            col(name).ok_or_else(|| {
                IngestError(format!(
                    "{}: no {name:?} column in header row {}",
                    self.input.file.name, map.header_row
                ))
            })
        };
        let (kc, dc, pc, ic, prc, bc) = (
            need(&map.key_column)?,
            need(&map.date)?,
            need(&map.payment)?,
            need(&map.interest)?,
            need(&map.principal)?,
            need(&map.balance)?,
        );

        let did = format!("{}/{}", self.src(), map.sheet);
        let dk = format!("doc:{did}");
        self.snap.ent(&dk, &k("doc/id"), Value::str(&did));
        self.add(&dk, "doc/kind", Value::Kw(k("doc.kind/loan-schedule")));
        self.add(&dk, "doc/external-id", Value::str(&map.loan));
        let sk = format!("source:{}", self.src());
        self.snap
            .refer(&sk, &k("source/id"), Value::str(self.src()));
        self.add(&dk, "doc/source", ValIn::Ref(temp(&sk)));
        let fk = format!("file:{}", self.input.file.hash.hex());
        self.snap
            .refer(&fk, &k("file/sha256"), Value::Hash(self.input.file.hash));
        self.add(&dk, "doc/file", ValIn::Ref(temp(&fk)));
        if let Some(t) = (0..hr)
            .map(|r| self.text(r, start_col))
            .find(|t| !t.is_empty())
        {
            self.add(&dk, "sheet.doc/title", Value::str(&t));
        }
        let ck = format!("commodity:{}", map.currency);
        self.snap
            .refer(&ck, &k("commodity/code"), Value::str(&map.currency));

        // read the rows: a payment has a whole number in the key column
        let mut rows: Vec<Row> = Vec::new();
        let mut other: Vec<(u32, String)> = Vec::new();
        let mut prev_balance: Option<Decimal> = None;
        for row in hr + 1..=end_row(self.values) {
            let n = match self.values.get_value((row, kc)) {
                Some(Data::Int(i)) => Some(*i),
                Some(Data::Float(f)) => float_decimal(*f)
                    .filter(|d| d.fract().is_zero())
                    .and_then(|d| i64::try_from(d).ok()),
                _ => None,
            };
            let Some(n) = n else {
                let label = self.text(row, kc);
                if !label.is_empty() {
                    if prev_balance.is_none() && rows.is_empty() {
                        prev_balance = self.amount(row, bc);
                    }
                    other.push((row, label));
                }
                continue;
            };
            let (Some(date), Some(payment), Some(interest), Some(principal), Some(balance)) = (
                self.date(row, dc),
                self.amount(row, pc),
                self.amount(row, ic),
                self.amount(row, prc),
                self.amount(row, bc),
            ) else {
                self.press(
                    "incomplete",
                    self.cell(row, kc),
                    format!("payment {n} is missing a value; not read"),
                );
                continue;
            };
            if interest + principal != payment {
                self.press(
                    "unbalanced",
                    self.cell(row, kc),
                    format!("payment {n}: {interest} + {principal} is not {payment}"),
                );
            }
            if let Some(p) = prev_balance
                && p - principal != balance
            {
                self.press(
                    "check-failed",
                    self.cell(row, bc),
                    format!("payment {n}: {p} less {principal} is not the balance {balance}"),
                );
            }
            prev_balance = Some(balance);
            rows.push(Row {
                n,
                row,
                date,
                payment,
                interest,
                principal,
                balance,
            });
        }
        self.check_totals(
            &rows,
            &other,
            [(pc, "payment"), (ic, "interest"), (prc, "principal")],
        );
        if let (Some(f), Some(l)) = (rows.first(), rows.last()) {
            self.add(&dk, "doc/period-start", Value::Date(f.date.clone()));
            self.add(&dk, "doc/period-end", Value::Date(l.date.clone()));
        }
        self.press(
            "derived-value",
            format!("cell:{}", map.sheet),
            "interest, principal, balance and dates are formulas; their cached results are the facts, and \
             each cell's region keeps the formula",
        );

        let mut now: BTreeSet<String> = BTreeSet::new();
        for r in &rows {
            let pid = format!("{did}/payment/{}", r.n);
            let pk = format!("obs:{pid}");
            self.snap.ent(&pk, &k("obs/id"), Value::str(&pid));
            self.add(&pk, "obs/kind", Value::Kw(k("obs.kind/scheduled-payment")));
            self.add(&pk, "obs/status", Value::Kw(k("obs.status/scheduled")));
            self.add(&pk, "obs/doc", ValIn::Ref(temp(&dk)));
            self.add(&pk, "obs/date", Value::Date(r.date.clone()));
            self.add(&pk, "obs/amount", Value::Dec(-r.payment));
            self.add(&pk, "obs/commodity", ValIn::Ref(temp(&ck)));
            self.add(
                &pk,
                "obs/reference",
                Value::str(&format!("Loan {} payment {}", map.loan, r.n)),
            );
            self.add(&pk, "loan.schedule/loan", Value::str(&map.loan));
            self.add(&pk, "loan.schedule/payment-number", Value::Int(r.n));
            self.add(&pk, "loan.schedule/balance-after", Value::Dec(-r.balance));
            for (c, field) in [
                (kc, "loan.schedule/payment-number"),
                (dc, "obs/date"),
                (pc, "obs/amount"),
                (bc, "loan.schedule/balance-after"),
            ] {
                let rk = self.region(r.row, c, field);
                self.add(&pk, "obs/region", ValIn::Ref(temp(&rk)));
            }
            now.insert(pid.clone());
            for (i, (part, amount, c, acct)) in [
                ("interest", r.interest, ic, &map.interest_account),
                ("principal", r.principal, prc, &map.principal_account),
            ]
            .into_iter()
            .enumerate()
            {
                let cid = format!("{pid}/{part}");
                let ckey = format!("obs:{cid}");
                self.snap.ent(&ckey, &k("obs/id"), Value::str(&cid));
                self.add(
                    &ckey,
                    "obs/kind",
                    Value::Kw(k("obs.kind/scheduled-payment-part")),
                );
                self.add(&ckey, "obs/status", Value::Kw(k("obs.status/scheduled")));
                self.add(&ckey, "obs/doc", ValIn::Ref(temp(&dk)));
                self.add(&ckey, "obs/parent", ValIn::Ref(temp(&pk)));
                self.add(
                    &ckey,
                    "obs/index",
                    Value::Int(i64::try_from(i).unwrap_or(0)),
                );
                self.add(&ckey, "obs/date", Value::Date(r.date.clone()));
                self.add(&ckey, "obs/amount", Value::Dec(-amount));
                self.add(&ckey, "obs/commodity", ValIn::Ref(temp(&ck)));
                if let Ok(kw) = Kw::new(&format!("loan.component/{part}")) {
                    self.add(&ckey, "loan.schedule/component", Value::Kw(kw));
                }
                if let Some(a) = acct {
                    self.add(&ckey, "loan.schedule/account-code", Value::str(a));
                }
                let rk = self.region(r.row, c, "obs/amount");
                self.add(&ckey, "obs/region", ValIn::Ref(temp(&rk)));
                now.insert(cid);
            }
        }

        // payments an earlier read had and this one does not
        if let (Some(doc), Some(a)) = (
            self.db.lookup(&k("doc/id"), &Value::str(&did)),
            self.db.attr(&k("obs/doc")).map(|d| d.id),
        ) {
            let before: Vec<String> = self
                .db
                .with_value(a, &Value::Ref(doc))
                .filter_map(|e| self.db.get(e, &k("obs/id")).and_then(Value::as_str))
                .map(str::to_string)
                .collect();
            for id in before.into_iter().filter(|id| !now.contains(id)) {
                self.snap.gone(&k("obs/id"), Value::str(&id));
            }
        }
        Ok(())
    }

    /// A row after the payments whose amount cells are formulas is a totals row: check it, do not store it.
    fn check_totals(&mut self, rows: &[Row], other: &[(u32, String)], cols: [(u32, &str); 3]) {
        let last = rows.last().map_or(0, |r| r.row);
        for (row, label) in other {
            if *row < last {
                self.press(
                    "skipped-row",
                    self.cell(*row, 0),
                    format!("{label:?} is not a payment; not read as one"),
                );
                continue;
            }
            for (c, what) in cols {
                let Some(shown) = self.amount(*row, c) else {
                    continue;
                };
                let sum: Decimal = rows
                    .iter()
                    .map(|r| match what {
                        "payment" => r.payment,
                        "interest" => r.interest,
                        _ => r.principal,
                    })
                    .sum();
                if sum == shown {
                    self.press(
                        "derived-value",
                        self.cell(*row, c),
                        format!(
                            "{label}: the {what} total {shown} agrees with its rows; not stored"
                        ),
                    );
                } else {
                    self.press(
                        "check-failed",
                        self.cell(*row, c),
                        format!("{label}: the {what} total shows {shown}, the rows sum to {sum}"),
                    );
                }
            }
        }
    }
}

fn end_row(r: &Range<Data>) -> u32 {
    r.end().map_or(0, |e| e.0)
}
