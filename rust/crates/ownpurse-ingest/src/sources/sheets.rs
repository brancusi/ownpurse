//! Google Sheets: `spreadsheets.values.batchGet` responses, read with a mapping an agent wrote.
//!
//! A read is the whole range as it looks now (a snapshot). The main file is the formatted read (what a person
//! sees: `"$1,250.00"`); the `formulas` companion is the same range read with `valueRenderOption=FORMULA`,
//! which says which cells were typed in and which a formula computed. The `mapping` companion says where the
//! header is, which column is the key, what the value columns mean, which rows and columns are computed, and
//! how to read the numbers.
//!
//! The budget mapping reads the sheet wide to long: one `obs.kind/budget` observation per category per month.
//! A budget is a plan, not money that moved (D3), so a figure has a period (`budget/period-start`,
//! `budget/period-end`), never an `obs/date`. Each category row is a `budget.line` entity the figures point
//! to. Computed rows and columns (totals, net) are not stored: they are checked against the figures they
//! are computed from, and the check is reported. A formula inside a figure row (`=B8`: "same as January")
//! is still a plan figure; its cell is marked with `sheet.cell/formula`.
//!
//! Identity is the key column's text: a moved row keeps its figures (only its position changes); a renamed
//! row is a new line, and the old one's figures are retracted.

use crate::snapshot::Snapshot;
use crate::{IngestError, Input, Pressure, Projection, Projector, temp};
use ownpurse_facts::value::parse_decimal;
use ownpurse_facts::{Date, Db, Kw, ValIn, Value, time};
use rust_decimal::Decimal;
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, BTreeSet};

pub struct Sheets;

const ID: &str = "sheet.values@1";

fn k(s: &'static str) -> Kw {
    Kw::lit(s)
}

/// Attributes this projector adds to the core vocabulary.
fn install() -> Vec<Json> {
    let defs = json!([
        {"ident": "budget/line", "type": "ref", "doc": "The budget line (a sheet row, a category) this figure is on"},
        {"ident": "budget/period-start", "type": "date", "doc": "First day of the period a plan figure covers"},
        {"ident": "budget/period-end", "type": "date", "doc": "Last day of the period a plan figure covers"},
        {"ident": "budget.line/id", "type": "string", "unique": "identity", "doc": "<source>/<sheet>/<key>: a budget row, named by its key column"},
        {"ident": "budget.line/doc", "type": "ref"},
        {"ident": "budget.line/name", "type": "string"},
        {"ident": "budget.line/account-code", "type": "string", "doc": "The book account the mapping says the line plans for"},
        {"ident": "budget.line/side", "type": "keyword", "doc": "budget.side/income or budget.side/expense"},
        {"ident": "budget.line/note", "type": "string"},
        {"ident": "budget.line/position", "type": "int", "doc": "Order among the sheet's lines, from 0"},
        {"ident": "sheet.cell/formula", "type": "string", "doc": "The formula behind a cell's value, when a formula computed it"},
        {"ident": "sheet.cell/field", "type": "keyword", "doc": "The attribute this cell supplied, e.g. obs/date"},
        {"ident": "sheet.doc/title", "type": "string", "doc": "The title a sheet or workbook shows above its table"}
    ]);
    defs.as_array().cloned().unwrap_or_default()
}

/// What a re-read may change: the interpretation it writes. `obs/region`, `doc/file` and `doc/source`
/// accumulate (every read that showed the figure is a witness); a person's notes and links are never touched.
fn owns(a: &Kw) -> bool {
    match a.as_str() {
        "obs/region" => false,
        "doc/kind" | "doc/external-id" | "doc/period-start" | "doc/period-end"
        | "sheet.doc/title" => true,
        _ => matches!(a.ns(), "obs" | "budget" | "budget.line"),
    }
}

impl Projector for Sheets {
    fn id(&self) -> &'static str {
        ID
    }

    fn reads(&self) -> &'static str {
        "Google Sheets values responses (formatted, with a FORMULA read beside it) and a mapping"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let bad = |m: String| IngestError(format!("{}: {m}", input.file.name));
        let main: Json =
            serde_json::from_slice(input.file.bytes).map_err(|e| bad(format!("not JSON ({e})")))?;
        let mapping: Json = input
            .companion("mapping")
            .ok_or_else(|| bad("a sheet needs a mapping (--with mapping=<file>)".into()))
            .and_then(|f| {
                serde_json::from_slice(f.bytes).map_err(|e| bad(format!("mapping: not JSON ({e})")))
            })?;
        let map = Mapping::read(&mapping).map_err(bad)?;
        let formulas: Option<Json> = match input.companion("formulas") {
            Some(f) => Some(
                serde_json::from_slice(f.bytes)
                    .map_err(|e| bad(format!("formulas: not JSON ({e})")))?,
            ),
            None => None,
        };
        if let Some(p) = crate::stale(db, &format!("{}/{}", input.source.id, map.sheet), input) {
            return Ok(p);
        }
        let shown = Grid::read(&main, &map.sheet).map_err(bad)?;
        let typed = formulas
            .as_ref()
            .map(|f| Grid::read(f, &map.sheet))
            .transpose()
            .map_err(bad)?;
        let mut b = Budget {
            db,
            input,
            map: &map,
            shown: &shown,
            typed: typed.as_ref(),
            snap: Snapshot::new(),
            pressure: Vec::new(),
        };
        if typed.is_none() {
            b.press(
                "computed-unknown",
                "formulas",
                "no FORMULA read beside this one: typed-in and computed cells look the same",
            );
        }
        b.run()?;
        let tx = b.snap.diff(db, &owns);
        Ok(Projection {
            install: install(),
            tx,
            pressure: b.pressure,
        })
    }
}

/// The mapping, as an agent wrote it.
struct Mapping {
    sheet: String,
    header_row: u32,
    key_column: String,
    currency: String,
    /// header text → (period start, period end)
    periods: BTreeMap<String, (Date, Date)>,
    note_column: Option<String>,
    computed_rows: BTreeSet<String>,
    computed_columns: BTreeSet<String>,
    symbol: String,
    thousands: String,
    decimal: String,
    income: BTreeSet<String>,
    accounts: BTreeMap<String, String>,
}

fn strs(j: Option<&Json>) -> BTreeSet<String> {
    j.and_then(Json::as_array)
        .into_iter()
        .flatten()
        .filter_map(Json::as_str)
        .map(str::to_string)
        .collect()
}

/// `2026-03` → (2026-03-01, 2026-03-31).
fn month(text: &str) -> Option<(Date, Date)> {
    let (y, m) = text.split_once('-')?;
    let (y, m): (i64, u32) = (y.parse().ok()?, m.parse().ok()?);
    if !(1..=12).contains(&m) {
        return None;
    }
    let first = time::days_from_civil(y, m, 1);
    let next = if m == 12 {
        time::days_from_civil(y + 1, 1, 1)
    } else {
        time::days_from_civil(y, m + 1, 1)
    };
    Some((Date::from_days(first), Date::from_days(next - 1)))
}

impl Mapping {
    fn read(j: &Json) -> Result<Mapping, String> {
        let s = |key: &str| j.get(key).and_then(Json::as_str).map(str::to_string);
        let need = |key: &str| s(key).ok_or_else(|| format!("mapping needs \"{key}\""));
        let mut periods = BTreeMap::new();
        for (h, p) in j
            .get("periods")
            .and_then(Json::as_object)
            .ok_or("mapping needs \"periods\": {header: \"YYYY-MM\"}")?
        {
            let p = p
                .as_str()
                .and_then(month)
                .ok_or_else(|| format!("mapping: period for {h:?} is not YYYY-MM"))?;
            periods.insert(h.clone(), p);
        }
        let number = j.get("number");
        let ns = |key: &str, d: &str| {
            number
                .and_then(|n| n.get(key))
                .and_then(Json::as_str)
                .unwrap_or(d)
                .to_string()
        };
        Ok(Mapping {
            sheet: need("sheet")?,
            header_row: j
                .get("header_row")
                .and_then(Json::as_u64)
                .and_then(|n| u32::try_from(n).ok())
                .ok_or("mapping needs \"header_row\" (1-based)")?,
            key_column: need("key_column")?,
            currency: s("currency").unwrap_or_else(|| "USD".into()),
            periods,
            note_column: s("note_column"),
            computed_rows: strs(j.get("computed").and_then(|c| c.get("rows"))),
            computed_columns: strs(j.get("computed").and_then(|c| c.get("columns"))),
            symbol: ns("symbol", "$"),
            thousands: ns("thousands", ","),
            decimal: ns("decimal", "."),
            income: strs(j.get("sign").and_then(|c| c.get("income_rows"))),
            accounts: j
                .get("accounts")
                .and_then(Json::as_object)
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| v.as_str().map(|v| (k.clone(), v.to_string())))
                        .collect()
                })
                .unwrap_or_default(),
        })
    }

    /// A formatted amount: `$1,250.00`, `-$396.05`, `($396.05)`.
    fn money(&self, text: &str) -> Result<Decimal, String> {
        let t = text.trim();
        let (neg, t) = if let Some(inner) = t.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
            (true, inner)
        } else if let Some(r) = t.strip_prefix('-') {
            (true, r)
        } else {
            (false, t)
        };
        let mut plain = t
            .trim()
            .trim_start_matches(self.symbol.as_str())
            .to_string();
        if !self.thousands.is_empty() {
            plain = plain.replace(self.thousands.as_str(), "");
        }
        if self.decimal != "." {
            plain = plain.replace(self.decimal.as_str(), ".");
        }
        let d = parse_decimal(&plain).map_err(|e| e.0)?;
        Ok(if neg { -d } else { d })
    }
}

/// One value range, addressed by sheet row (1-based) and column (0-based).
struct Grid {
    range: String,
    spreadsheet: String,
    first_row: u32,
    first_col: u32,
    rows: Vec<Vec<Json>>,
}

/// `A` → 0, `O` → 14, `AA` → 26.
fn col_index(letters: &str) -> Option<u32> {
    let mut n: u32 = 0;
    for c in letters.chars() {
        if !c.is_ascii_uppercase() {
            return None;
        }
        n = n
            .checked_mul(26)?
            .checked_add(u32::from(c as u8 - b'A') + 1)?;
    }
    n.checked_sub(1)
}

pub fn col_letters(mut i: u32) -> String {
    let mut out = Vec::new();
    loop {
        out.push(char::from(b'A' + u8::try_from(i % 26).unwrap_or(0)));
        if i < 26 {
            break;
        }
        i = i / 26 - 1;
    }
    out.iter().rev().collect()
}

/// `C7` → (7, 2).
fn cell_ref(text: &str) -> Option<(u32, u32)> {
    let split = text.find(|c: char| c.is_ascii_digit())?;
    let (l, d) = text.split_at(split);
    Some((d.parse().ok()?, col_index(l)?))
}

impl Grid {
    fn read(j: &Json, sheet: &str) -> Result<Grid, String> {
        let spreadsheet = j
            .get("spreadsheetId")
            .and_then(Json::as_str)
            .unwrap_or("")
            .to_string();
        let vr = j
            .get("valueRanges")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
            .find(|v| {
                v.get("range")
                    .and_then(Json::as_str)
                    .and_then(|r| r.rsplit_once('!'))
                    .is_some_and(|(s, _)| s.trim_matches('\'') == sheet)
            })
            .ok_or_else(|| format!("no value range for sheet {sheet:?}"))?;
        if vr.get("majorDimension").and_then(Json::as_str) == Some("COLUMNS") {
            return Err("a COLUMNS read; read the range with majorDimension=ROWS".into());
        }
        let range = vr
            .get("range")
            .and_then(Json::as_str)
            .unwrap_or("")
            .to_string();
        let start = range
            .rsplit_once('!')
            .map(|(_, r)| r.split(':').next().unwrap_or(r))
            .and_then(cell_ref)
            .ok_or_else(|| format!("range {range:?} is not <sheet>!<A1>:<B2>"))?;
        let rows = vr
            .get("values")
            .and_then(Json::as_array)
            .into_iter()
            .flatten()
            .map(|r| r.as_array().cloned().unwrap_or_default())
            .collect();
        Ok(Grid {
            range,
            spreadsheet,
            first_row: start.0,
            first_col: start.1,
            rows,
        })
    }

    fn row(&self, sheet_row: u32) -> &[Json] {
        sheet_row
            .checked_sub(self.first_row)
            .and_then(|i| self.rows.get(usize::try_from(i).ok()?))
            .map_or(&[], Vec::as_slice)
    }

    fn get(&self, sheet_row: u32, col: u32) -> Option<&Json> {
        let i = usize::try_from(col.checked_sub(self.first_col)?).ok()?;
        self.row(sheet_row).get(i)
    }

    fn text(&self, sheet_row: u32, col: u32) -> &str {
        self.get(sheet_row, col)
            .and_then(Json::as_str)
            .unwrap_or("")
    }

    fn last_row(&self) -> u32 {
        self.first_row + u32::try_from(self.rows.len()).unwrap_or(0)
    }

    fn width(&self, sheet_row: u32) -> u32 {
        self.first_col + u32::try_from(self.row(sheet_row).len()).unwrap_or(0)
    }
}

/// A figure row, as read.
struct Line {
    key: String,
    row: u32,
    figures: Vec<Figure>,
}

struct Figure {
    col: u32,
    header: String,
    amount: Decimal,
    formula: Option<String>,
}

struct Budget<'a, 'i> {
    db: &'a Db,
    input: &'a Input<'i>,
    map: &'a Mapping,
    shown: &'a Grid,
    typed: Option<&'a Grid>,
    snap: Snapshot,
    pressure: Vec<Pressure>,
}

impl Budget<'_, '_> {
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
        format!("cell:{}!{}{row}", self.map.sheet, col_letters(col))
    }

    /// The formula behind a cell, from the FORMULA read.
    fn formula(&self, row: u32, col: u32) -> Option<String> {
        self.typed?
            .get(row, col)
            .and_then(Json::as_str)
            .filter(|t| t.starts_with('='))
            .map(str::to_string)
    }

    fn region(&mut self, locator: &str) -> String {
        let h = self.input.file.hash;
        let id = format!("{}#{locator}", h.hex());
        let key = format!("region:{id}");
        self.snap.ent(&key, &k("region/id"), Value::str(&id));
        let fk = format!("file:{}", h.hex());
        self.snap.refer(&fk, &k("file/sha256"), Value::Hash(h));
        self.add(&key, "region/file", ValIn::Ref(temp(&fk)));
        self.add(&key, "region/locator", Value::str(locator));
        key
    }

    fn run(&mut self) -> Result<(), IngestError> {
        let map = self.map;
        let shown = self.shown;
        let hr = map.header_row;
        let width = shown.width(hr);
        let headers: Vec<(u32, String)> = (shown.first_col..width)
            .map(|c| (c, shown.text(hr, c).trim().to_string()))
            .collect();
        let col_of = |name: &str| headers.iter().find(|(_, h)| h == name).map(|(c, _)| *c);
        let key_col = col_of(&map.key_column).ok_or_else(|| {
            IngestError(format!(
                "{}: no {:?} column in header row {hr}",
                self.input.file.name, map.key_column
            ))
        })?;
        let note_col = map.note_column.as_deref().and_then(col_of);
        let period_cols: Vec<(u32, String)> = headers
            .iter()
            .filter(|(_, h)| map.periods.contains_key(h))
            .cloned()
            .collect();
        for h in map.periods.keys() {
            if col_of(h).is_none() {
                self.press(
                    "missing-column",
                    format!("row:{hr}"),
                    format!("no {h:?} column"),
                );
            }
        }
        for (c, h) in &headers {
            let known = *c == key_col
                || Some(*c) == note_col
                || map.periods.contains_key(h)
                || map.computed_columns.contains(h);
            if !known && !h.is_empty() {
                self.press(
                    "unmapped-column",
                    self.cell(hr, *c),
                    format!("column {h:?} is not in the mapping; not read"),
                );
            }
        }

        // the document: the sheet tab, its title and the span of periods it plans
        let did = format!("{}/{}", self.src(), map.sheet);
        let dk = format!("doc:{did}");
        self.snap.ent(&dk, &k("doc/id"), Value::str(&did));
        self.add(&dk, "doc/kind", Value::Kw(k("doc.kind/budget")));
        if !shown.spreadsheet.is_empty() {
            self.add(&dk, "doc/external-id", Value::str(&shown.spreadsheet));
        }
        let sk = format!("source:{}", self.src());
        self.snap
            .refer(&sk, &k("source/id"), Value::str(self.src()));
        self.add(&dk, "doc/source", ValIn::Ref(temp(&sk)));
        let fk = format!("file:{}", self.input.file.hash.hex());
        self.snap
            .refer(&fk, &k("file/sha256"), Value::Hash(self.input.file.hash));
        self.add(&dk, "doc/file", ValIn::Ref(temp(&fk)));
        if let (Some(first), Some(last)) = (
            map.periods.values().map(|p| p.0.clone()).min(),
            map.periods.values().map(|p| p.1.clone()).max(),
        ) {
            self.add(&dk, "doc/period-start", Value::Date(first));
            self.add(&dk, "doc/period-end", Value::Date(last));
        }
        let title = (shown.first_row..hr)
            .map(|r| shown.text(r, shown.first_col).trim().to_string())
            .find(|t| !t.is_empty());
        if let Some(t) = title {
            self.add(&dk, "sheet.doc/title", Value::str(&t));
        }
        let ck = format!("commodity:{}", map.currency);
        self.snap
            .refer(&ck, &k("commodity/code"), Value::str(&map.currency));
        self.press(
            "period",
            &shown.range,
            "a plan figure covers a month, not a day: stored as budget/period-start and budget/period-end, \
             with no obs/date (core has periods only on documents)",
        );

        // the rows: figure lines, computed rows, blanks
        let mut lines: Vec<Line> = Vec::new();
        let mut computed_rows: Vec<(String, u32)> = Vec::new();
        for row in hr + 1..shown.last_row() {
            let key = shown.text(row, key_col).trim().to_string();
            if key.is_empty() {
                continue;
            }
            if map.computed_rows.contains(&key) {
                computed_rows.push((key, row));
                continue;
            }
            if lines.iter().any(|l| l.key == key) {
                self.press(
                    "duplicate-key",
                    self.cell(row, key_col),
                    format!("{key:?} is on two rows; the second is not read"),
                );
                continue;
            }
            let mut figures = Vec::new();
            for (col, header) in &period_cols {
                let text = shown.text(row, *col).trim().to_string();
                if text.is_empty() {
                    continue;
                }
                let at = self.cell(row, *col);
                let amount = match map.money(&text) {
                    Ok(a) => a,
                    Err(e) => {
                        self.press("unreadable", at, e);
                        continue;
                    }
                };
                let formula = self.formula(row, *col);
                if formula.is_none()
                    && let Some(raw) = self.typed.and_then(|t| t.get(row, *col))
                    && let Json::Number(n) = raw
                    && parse_decimal(&n.to_string()).ok() != Some(amount)
                {
                    self.press(
                        "format",
                        at.clone(),
                        format!("shown as {text}, typed as {n}: the shown value is stored"),
                    );
                }
                figures.push(Figure {
                    col: *col,
                    header: header.clone(),
                    amount,
                    formula,
                });
            }
            lines.push(Line { key, row, figures });
        }

        // figures per (row, col), for checking the computed cells
        let mut grid: BTreeMap<(u32, u32), Decimal> = BTreeMap::new();
        for l in &lines {
            for f in &l.figures {
                grid.insert((l.row, f.col), f.amount);
            }
        }
        self.check_computed(&lines, &computed_rows, &headers, &grid);

        // the lines and their figures
        let doc_lines: BTreeSet<String> = lines.iter().map(|l| self.line_id(&l.key)).collect();
        let mut doc_obs: BTreeSet<String> = BTreeSet::new();
        for (pos, l) in lines.iter().enumerate() {
            let lid = self.line_id(&l.key);
            let lk = format!("line:{lid}");
            self.snap.ent(&lk, &k("budget.line/id"), Value::str(&lid));
            self.add(&lk, "budget.line/doc", ValIn::Ref(temp(&dk)));
            self.add(&lk, "budget.line/name", Value::str(&l.key));
            self.add(
                &lk,
                "budget.line/position",
                Value::Int(i64::try_from(pos).unwrap_or(0)),
            );
            let income = map.income.contains(&l.key);
            self.add(
                &lk,
                "budget.line/side",
                Value::Kw(k(if income {
                    "budget.side/income"
                } else {
                    "budget.side/expense"
                })),
            );
            match map.accounts.get(&l.key) {
                Some(code) => self.add(&lk, "budget.line/account-code", Value::str(code)),
                None => self.press(
                    "unmapped-key",
                    self.cell(l.row, key_col),
                    format!(
                        "{:?} has no account in the mapping (renamed?); read as an expense with no account",
                        l.key
                    ),
                ),
            }
            if let Some(nc) = note_col {
                let note = shown.text(l.row, nc).trim().to_string();
                if !note.is_empty() {
                    self.add(&lk, "budget.line/note", Value::str(&note));
                }
            }
            let formulas: Vec<&Figure> = l.figures.iter().filter(|f| f.formula.is_some()).collect();
            if !formulas.is_empty() {
                self.press(
                    "formula-entry",
                    self.cell(l.row, key_col),
                    format!(
                        "{}: {} of {} figures are formulas ({} …); kept as plan figures, cells marked with sheet.cell/formula",
                        l.key,
                        formulas.len(),
                        l.figures.len(),
                        formulas[0].formula.as_deref().unwrap_or("")
                    ),
                );
            }
            for f in &l.figures {
                let Some((start, end)) = map.periods.get(&f.header).cloned() else {
                    continue;
                };
                let oid = format!("{lid}/{}", &start.as_str()[..7]);
                let ok = format!("obs:{oid}");
                self.snap.ent(&ok, &k("obs/id"), Value::str(&oid));
                self.add(&ok, "obs/kind", Value::Kw(k("obs.kind/budget")));
                self.add(&ok, "obs/doc", ValIn::Ref(temp(&dk)));
                self.add(&ok, "budget/line", ValIn::Ref(temp(&lk)));
                self.add(&ok, "budget/period-start", Value::Date(start));
                self.add(&ok, "budget/period-end", Value::Date(end));
                let signed = if income { f.amount } else { -f.amount };
                self.add(&ok, "obs/amount", Value::Dec(signed));
                self.add(&ok, "obs/commodity", ValIn::Ref(temp(&ck)));
                let rk = self.region(&self.cell(l.row, f.col));
                if let Some(fx) = &f.formula {
                    self.add(&rk, "sheet.cell/formula", Value::str(fx));
                }
                self.add(&ok, "obs/region", ValIn::Ref(temp(&rk)));
                doc_obs.insert(oid);
            }
        }

        // what this sheet said before and does not say now
        self.gone(
            &did,
            "budget.line/doc",
            "budget.line/id",
            &doc_lines,
            "line",
        );
        self.gone(&did, "obs/doc", "obs/id", &doc_obs, "figure");
        Ok(())
    }

    fn line_id(&self, key: &str) -> String {
        format!("{}/{}/{key}", self.src(), self.map.sheet)
    }

    /// Retract what an earlier read of this document said and this one does not.
    fn gone(
        &mut self,
        doc_id: &str,
        doc_attr: &'static str,
        id_attr: &'static str,
        now: &BTreeSet<String>,
        what: &str,
    ) {
        let (Some(doc), Some(a)) = (
            self.db.lookup(&k("doc/id"), &Value::str(doc_id)),
            self.db.attr(&k(doc_attr)).map(|d| d.id),
        ) else {
            return;
        };
        let before: Vec<String> = self
            .db
            .with_value(a, &Value::Ref(doc))
            .filter_map(|e| self.db.get(e, &k(id_attr)).and_then(Value::as_str))
            .map(str::to_string)
            .collect();
        for id in before {
            if now.contains(&id) {
                continue;
            }
            self.snap.gone(&k(id_attr), Value::str(&id));
            if what == "line" {
                self.press(
                    "identity",
                    id.clone(),
                    "this line is not in the read any more and its figures are retracted; \
                     a rename looks exactly like a delete and an add",
                );
            }
        }
    }

    /// Computed rows and columns are not stored; check them against the figures they are computed from.
    fn check_computed(
        &mut self,
        lines: &[Line],
        rows: &[(String, u32)],
        headers: &[(u32, String)],
        figures: &BTreeMap<(u32, u32), Decimal>,
    ) {
        let map = self.map;
        let shown = self.shown;
        // every shown number in the range, computed or not, for evaluating formulas
        let mut values: BTreeMap<(u32, u32), Decimal> = figures.clone();
        for row in shown.first_row..shown.last_row() {
            for col in shown.first_col..shown.width(row) {
                if let Ok(d) = map.money(shown.text(row, col)) {
                    values.entry((row, col)).or_insert(d);
                }
            }
        }
        let computed_cols: Vec<(u32, String)> = headers
            .iter()
            .filter(|(_, h)| map.computed_columns.contains(h))
            .cloned()
            .collect();
        let mut cells: Vec<(u32, u32)> = Vec::new();
        for (_, r) in rows {
            for (c, _) in headers {
                if !shown.text(*r, *c).trim().is_empty() && map.money(shown.text(*r, *c)).is_ok() {
                    cells.push((*r, *c));
                }
            }
        }
        for l in lines {
            for (c, _) in &computed_cols {
                cells.push((l.row, *c));
            }
        }
        let (mut agree, mut unchecked) = (0usize, 0usize);
        for (r, c) in cells {
            let at = self.cell(r, c);
            let Ok(shown_v) = map.money(shown.text(r, c)) else {
                continue;
            };
            let Some(fx) = self.formula(r, c) else {
                self.press(
                    "derived-value",
                    at,
                    "the mapping says this cell is computed but it holds a typed value; not stored",
                );
                continue;
            };
            match eval(&fx, &values) {
                Some(v) if v == shown_v => agree += 1,
                Some(v) => self.press(
                    "check-failed",
                    at,
                    format!("shows {shown_v}, but {fx} over the figures is {v}"),
                ),
                None => unchecked += 1,
            }
        }
        let names: Vec<&str> = rows
            .iter()
            .map(|(n, _)| n.as_str())
            .chain(computed_cols.iter().map(|(_, h)| h.as_str()))
            .collect();
        self.press(
            "derived-value",
            &shown.range,
            format!(
                "computed rows and columns ({}) are not stored; {agree} cells agree with their formulas, \
                 {unchecked} not checked",
                names.join(", ")
            ),
        );
    }
}

/// Evaluate the formulas a budget uses for totals: sums and differences of cells and `SUM` ranges.
/// Anything else is `None` (not checked).
fn eval(formula: &str, values: &BTreeMap<(u32, u32), Decimal>) -> Option<Decimal> {
    let body = formula.strip_prefix('=')?.replace(' ', "");
    let mut total = Decimal::ZERO;
    let mut rest = body.as_str();
    let mut sign = Decimal::ONE;
    if let Some(r) = rest.strip_prefix('+') {
        rest = r;
    }
    loop {
        let end = rest.find(['+', '-']).unwrap_or(rest.len());
        let term = &rest[..end];
        let v = if let Some(inner) = term.strip_prefix("SUM(").and_then(|t| t.strip_suffix(')')) {
            let (a, b) = inner.split_once(':')?;
            let (a, b) = (
                cell_ref(&a.replace('$', ""))?,
                cell_ref(&b.replace('$', ""))?,
            );
            values
                .range((a.0, 0)..=(b.0, u32::MAX))
                .filter(|((r, c), _)| (a.0..=b.0).contains(r) && (a.1..=b.1).contains(c))
                .map(|(_, v)| *v)
                .sum()
        } else {
            let at = cell_ref(&term.replace('$', ""))?;
            values.get(&at).copied().unwrap_or_default()
        };
        total += sign * v;
        if end == rest.len() {
            return Some(total);
        }
        sign = if rest.as_bytes()[end] == b'-' {
            Decimal::NEGATIVE_ONE
        } else {
            Decimal::ONE
        };
        rest = &rest[end + 1..];
    }
}
