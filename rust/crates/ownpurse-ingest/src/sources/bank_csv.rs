//! Bank activity downloads (CSV) read through a column mapping.
//!
//! A download is a window onto the bank's list of lines for one account. Lines carry no id, so a line's
//! identity is its fingerprint (account, date, amount, text) plus which of several identical lines it is,
//! counted from the earliest: the two identical coffees on 03-06 are `#1` and `#2` in every download that
//! covers that day, so overlapping downloads name the same lines and re-reading the overlap changes nothing.
//!
//! - Every line points at one document, the account's activity feed (`obs/doc`); each download is its own
//!   document with its window (`doc/period-*`), and each line gains a region per download it appears in.
//! - The running balance is kept on the line (`bank.csv/balance-after`) and checked line by line; a
//!   mismatch is pressure, never fixed.
//! - Pending lines are their own observations (`…|csv-pending|…`, `obs.status/pending`). When a newer
//!   download whose window covers a pending line's date no longer shows it, the line becomes
//!   `obs.status/lapsed` (the authorisation dropped or posted under other words); nothing is retracted.

mod common;

pub(crate) use common::*;

use crate::facts;
use crate::{IngestError, Input, Pressure, Projection, Projector};
use ownpurse_facts::value::parse_decimal;
use ownpurse_facts::{Date, Db, Eid, EntRef, Instant, TxData, ValIn, Value};
use rust_decimal::Decimal;
use serde_json::{Value as Json, json};
use std::collections::{BTreeMap, BTreeSet};

pub struct BankCsv;

impl Projector for BankCsv {
    fn id(&self) -> &'static str {
        "bank.csv@1"
    }

    fn reads(&self) -> &'static str {
        "Bank activity downloads (CSV) with a column mapping"
    }

    fn project(&self, db: &Db, input: &Input<'_>) -> Result<Projection, IngestError> {
        let map_file = input.companion("mapping").ok_or_else(|| {
            IngestError(format!(
                "{}: a bank CSV needs its column mapping (\"with\": {{\"mapping\": \"<file>.json\"}})",
                input.file.name
            ))
        })?;
        let m = Mapping::parse(map_file.bytes)
            .map_err(|e| IngestError(format!("{}: {e}", map_file.name)))?;
        let mut r = Read::new(db, input, m, map_file.hash);
        r.read()?;
        Ok(r.finish())
    }
}

fn install() -> Vec<Json> {
    vec![
        json!({"ident": "bank.csv/type", "type": "string",
               "doc": "The download's own type column: ACH_DEBIT, CHECK, PENDING, …"}),
        json!({"ident": "bank.csv/balance-after", "type": "decimal",
               "doc": "The running balance the download shows after this line"}),
        json!({"ident": "bank.csv/occurrence", "type": "int",
               "doc": "Which of several identical lines (same account, date, amount and text) this is, from the earliest"}),
        json!({"ident": "bank.csv/observed-at", "type": "instant",
               "doc": "When this download was taken: a newer download that covers a line's date is the later word on it"}),
        json!({"ident": "bank.csv/mapping", "type": "ref",
               "doc": "The mapping file this download was read through"}),
    ]
}

// ---------------------------------------------------------------------------------------------------------
// the mapping

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DateFmt {
    Mdy,
    Dmy,
    Ymd,
}

#[derive(Debug, Clone)]
struct Mapping {
    account: AccountSpec,
    preamble: usize,
    account_number_line: Option<usize>,
    window_line: Option<usize>,
    header_line: usize,
    date: String,
    description: String,
    debit: Option<String>,
    credit: Option<String>,
    amount: Option<String>,
    balance: Option<String>,
    check_number: Option<String>,
    kind: Option<String>,
    date_fmt: DateFmt,
    thousands: Option<char>,
    decimal: char,
    newest_first: bool,
    pending: Option<(String, String, String)>,
}

impl Mapping {
    fn parse(bytes: &[u8]) -> Result<Mapping, String> {
        let j: Json = serde_json::from_slice(bytes).map_err(|e| format!("not JSON ({e})"))?;
        let s = |p: &str| j.pointer(p).and_then(Json::as_str).map(str::to_string);
        let n = |p: &str| {
            j.pointer(p)
                .and_then(Json::as_u64)
                .and_then(|v| usize::try_from(v).ok())
        };
        let need = |p: &str| s(p).ok_or_else(|| format!("the mapping needs {p}"));
        let account = AccountSpec {
            id: need("/account/id")?,
            mask: need("/account/mask")?,
            name: s("/account/name"),
            institution: s("/account/institution"),
            commodity: need("/account/commodity")?,
            scale: j.pointer("/account/scale").and_then(Json::as_i64),
        };
        let date_fmt = match need("/date_format")?.as_str() {
            "MM/DD/YYYY" => DateFmt::Mdy,
            "DD/MM/YYYY" => DateFmt::Dmy,
            "YYYY-MM-DD" => DateFmt::Ymd,
            other => {
                return Err(format!(
                    "date_format {other:?} is not one this reader knows (MM/DD/YYYY, DD/MM/YYYY, YYYY-MM-DD)"
                ));
            }
        };
        let ch = |p: &str| s(p).and_then(|t| t.chars().next());
        let m = Mapping {
            account,
            preamble: n("/preamble/lines").unwrap_or(0),
            account_number_line: n("/preamble/account_number_line"),
            window_line: n("/preamble/window_line"),
            header_line: n("/header_line").unwrap_or(1),
            date: need("/columns/date")?,
            description: need("/columns/description")?,
            debit: s("/columns/debit"),
            credit: s("/columns/credit"),
            amount: s("/columns/amount"),
            balance: s("/columns/balance"),
            check_number: s("/columns/check_number"),
            kind: s("/columns/type"),
            date_fmt,
            thousands: ch("/amounts/thousands_separator"),
            decimal: ch("/amounts/decimal_separator").unwrap_or('.'),
            newest_first: s("/order").as_deref() == Some("newest-first"),
            pending: match (
                s("/pending/column"),
                s("/pending/equals"),
                s("/pending/strip_prefix"),
            ) {
                (Some(c), Some(e), p) => Some((c, e, p.unwrap_or_default())),
                _ => None,
            },
        };
        if m.amount.is_none() && (m.debit.is_none() || m.credit.is_none()) {
            return Err(
                "the mapping needs columns.amount, or columns.debit and columns.credit".into(),
            );
        }
        if m.header_line <= m.preamble {
            return Err("header_line must come after the preamble".into());
        }
        Ok(m)
    }

    fn date(&self, text: &str) -> Option<Date> {
        let t = text.trim();
        let (y, mo, d) = match self.date_fmt {
            DateFmt::Ymd => {
                let mut p = t.split('-');
                (p.next()?, p.next()?, p.next()?)
            }
            DateFmt::Mdy | DateFmt::Dmy => {
                let mut p = t.split('/');
                let (a, b, y) = (p.next()?, p.next()?, p.next()?);
                if self.date_fmt == DateFmt::Mdy {
                    (y, a, b)
                } else {
                    (y, b, a)
                }
            }
        };
        let num = |s: &str| s.parse::<u32>().ok();
        Date::parse(&format!("{:04}-{:02}-{:02}", num(y)?, num(mo)?, num(d)?)).ok()
    }

    /// `"3,215.47"` → 3215.47; `(12.50)` → -12.50; a currency sign is dropped. `None` for an empty cell.
    fn amount(&self, text: &str) -> Result<Option<Decimal>, String> {
        let mut t: String = text.trim().to_string();
        if t.is_empty() {
            return Ok(None);
        }
        let paren = t.starts_with('(') && t.ends_with(')');
        if paren {
            t = t[1..t.len() - 1].to_string();
        }
        let mut clean: String = t
            .chars()
            .filter(|c| Some(*c) != self.thousands && *c != '$' && !c.is_whitespace())
            .collect();
        if self.decimal != '.' {
            clean = clean.replace(self.decimal, ".");
        }
        let clean = clean.strip_prefix('+').unwrap_or(&clean).to_string();
        let d = parse_decimal(&clean).map_err(|e| e.0)?;
        Ok(Some(if paren { -d } else { d }))
    }
}

// ---------------------------------------------------------------------------------------------------------
// reading one download

#[derive(Debug, Clone)]
struct Row {
    line: u64,
    date: Date,
    description: String,
    match_text: String,
    amount: Decimal,
    balance: Option<Decimal>,
    check: Option<String>,
    kind: Option<String>,
    pending: bool,
}

struct Read<'a, 'i> {
    db: &'a Db,
    input: &'a Input<'i>,
    m: Mapping,
    mapping_hash: ownpurse_facts::Hash,
    window: Option<(Date, Date)>,
    rows: Vec<Row>,
    tx: TxData,
    pressure: Vec<Pressure>,
}

/// The day a line or window starts on, as a number, for comparing dates.
fn day(d: &Date) -> &str {
    d.as_str()
}

impl<'a, 'i> Read<'a, 'i> {
    fn new(
        db: &'a Db,
        input: &'a Input<'i>,
        m: Mapping,
        mapping_hash: ownpurse_facts::Hash,
    ) -> Read<'a, 'i> {
        Read {
            db,
            input,
            m,
            mapping_hash,
            window: None,
            rows: Vec::new(),
            tx: TxData::new(),
            pressure: Vec::new(),
        }
    }

    fn press(&mut self, kind: &str, at: impl Into<String>, detail: impl Into<String>) {
        self.pressure.push(Pressure::new(kind, at, detail));
    }

    fn src(&self) -> &str {
        &self.input.source.id
    }

    fn read(&mut self) -> Result<(), IngestError> {
        let name = self.input.file.name;
        let text = std::str::from_utf8(self.input.file.bytes)
            .map_err(|e| IngestError(format!("{name}: not UTF-8 text ({e})")))?;
        let text = text.strip_prefix('\u{feff}').unwrap_or(text);
        // the preamble, by physical line
        let lines: Vec<&str> = text.split('\n').map(|l| l.trim_end_matches('\r')).collect();
        if let Some(n) = self.m.account_number_line {
            let l = lines.get(n - 1).copied().unwrap_or("");
            let digits: String = l.chars().filter(char::is_ascii_digit).collect();
            if !digits.ends_with(&self.m.account.mask) {
                return Err(IngestError(format!(
                    "{name}: line {n} says account {l:?}, but the mapping is for the account ending {}; \
                     use the mapping for that account",
                    self.m.account.mask
                )));
            }
        }
        if let Some(n) = self.m.window_line {
            let l = lines.get(n - 1).copied().unwrap_or("");
            let ds: Vec<Date> = l
                .split(|c: char| c == ',' || c.is_whitespace())
                .filter_map(|t| self.m.date(t))
                .collect();
            match ds.as_slice() {
                [a, b] => self.window = Some((a.clone(), b.clone())),
                _ => self.press(
                    "no-window",
                    format!("row:{n}"),
                    format!("expected two dates in {l:?}; lapsed pending lines cannot be detected"),
                ),
            }
        }
        // the table, from the header line on
        let offset: usize = text
            .split('\n')
            .take(self.m.header_line - 1)
            .map(|l| l.len() + 1)
            .sum();
        let body = text.get(offset.min(text.len())..).unwrap_or("");
        let mut rdr = csv::ReaderBuilder::new()
            .flexible(true)
            .from_reader(body.as_bytes());
        let headers = rdr
            .headers()
            .map_err(|e| IngestError(format!("{name}: no header line ({e})")))?
            .clone();
        let col = |want: &Option<String>| -> Result<Option<usize>, IngestError> {
            match want {
                None => Ok(None),
                Some(w) => headers
                    .iter()
                    .position(|h| h.trim() == w)
                    .map(Some)
                    .ok_or_else(|| {
                        IngestError(format!(
                            "{name}: no column {w:?} (the header has {})",
                            headers.iter().collect::<Vec<_>>().join(", ")
                        ))
                    }),
            }
        };
        let c_date = col(&Some(self.m.date.clone()))?.unwrap_or(0);
        let c_desc = col(&Some(self.m.description.clone()))?.unwrap_or(0);
        let (c_debit, c_credit, c_amount) = (
            col(&self.m.debit)?,
            col(&self.m.credit)?,
            col(&self.m.amount)?,
        );
        let (c_bal, c_check, c_kind) = (
            col(&self.m.balance)?,
            col(&self.m.check_number)?,
            col(&self.m.kind)?,
        );
        let c_pending = match &self.m.pending {
            Some((c, ..)) => col(&Some(c.clone()))?,
            None => None,
        };
        for rec in rdr.records() {
            let rec = rec.map_err(|e| IngestError(format!("{name}: {e}")))?;
            // the physical line the record starts on, counted from the record's byte offset; with CRLF
            // endings the reader puts the start before the previous line's '\n', so step over terminators
            let bytes = body.as_bytes();
            let mut start = rec
                .position()
                .and_then(|p| usize::try_from(p.byte()).ok())
                .unwrap_or(0)
                .min(bytes.len());
            while start < bytes.len() && matches!(bytes[start], b'\r' | b'\n') {
                start += 1;
            }
            let newlines = bytes[..start].iter().filter(|b| **b == b'\n').count();
            let line = (self.m.header_line + newlines) as u64;
            let at = format!("row:{line}");
            let cell = |c: Option<usize>| c.and_then(|c| rec.get(c)).unwrap_or("").trim();
            if rec.iter().all(|f| f.trim().is_empty()) {
                continue;
            }
            let Some(date) = self.m.date(cell(Some(c_date))) else {
                self.press(
                    "unreadable",
                    &at,
                    format!("date {:?} is not {:?}", cell(Some(c_date)), self.m.date_fmt),
                );
                continue;
            };
            let amt = |c| self.m.amount(cell(c));
            let amount = match (amt(c_amount), amt(c_debit), amt(c_credit)) {
                (Ok(Some(a)), _, _) => a,
                (_, Ok(Some(d)), Ok(None)) => {
                    if d.is_sign_negative() {
                        self.press(
                            "sign",
                            &at,
                            "a negative number in the debit column; read as money out",
                        );
                    }
                    -d.abs()
                }
                (_, Ok(None), Ok(Some(c))) => c,
                (_, Ok(Some(d)), Ok(Some(c))) => {
                    self.press(
                        "sign",
                        &at,
                        "both debit and credit are filled; read as credit minus debit",
                    );
                    c - d
                }
                (_, Err(e), _) | (_, _, Err(e)) | (Err(e), _, _) => {
                    self.press("unreadable", &at, e);
                    continue;
                }
                _ => {
                    self.press("unreadable", &at, "no amount in debit, credit or amount");
                    continue;
                }
            };
            let balance = match c_bal.map(|c| self.m.amount(cell(Some(c)))) {
                Some(Ok(b)) => b,
                Some(Err(e)) => {
                    self.press("unreadable", &at, format!("balance: {e}"));
                    None
                }
                None => None,
            };
            let description = cell(Some(c_desc)).to_string();
            let pending = match &self.m.pending {
                Some((_, eq, _)) => cell(c_pending) == eq,
                None => false,
            };
            let match_text = match &self.m.pending {
                Some((_, _, prefix)) if pending && !prefix.is_empty() => description
                    .strip_prefix(prefix.as_str())
                    .unwrap_or(&description)
                    .to_string(),
                _ => description.clone(),
            };
            let opt = |s: &str| (!s.is_empty()).then(|| s.to_string());
            self.rows.push(Row {
                line,
                date,
                description,
                match_text,
                amount,
                balance,
                check: opt(cell(c_check)),
                kind: opt(cell(c_kind)),
                pending,
            });
        }
        if self.m.newest_first {
            self.rows.reverse();
        }
        Ok(())
    }

    fn finish(mut self) -> Projection {
        let rows = std::mem::take(&mut self.rows);
        self.check_order(&rows);
        self.check_balances(&rows);
        let input = self.input;
        let acct_id = self.m.account.id.clone();
        let (acct, cur) = account(&mut self.tx, input, &self.m.account);
        // the feed every line belongs to, and this download as a window onto it
        let feed_id = format!("{}|csv|{acct_id}", self.src());
        let feed = facts::document(&mut self.tx, input, &feed_id, "bank-activity");
        let dl_id = format!(
            "{}|csv-download|{acct_id}|{}",
            self.src(),
            input.file.hash.hex()
        );
        let dl = facts::document(&mut self.tx, input, &dl_id, "bank-download");
        if let Some((a, b)) = &self.window {
            self.tx
                .add(&dl, &k("doc/period-start"), Value::Date(a.clone()));
            self.tx
                .add(&dl, &k("doc/period-end"), Value::Date(b.clone()));
        }
        self.tx.add(
            &dl,
            &k("bank.csv/observed-at"),
            Value::Instant(input.observed_at.clone()),
        );
        let mf = facts::file(&mut self.tx, &self.mapping_hash);
        self.tx.add(&dl, &k("bank.csv/mapping"), ValIn::Ref(mf));
        // lines this download shows, by identity
        let newer = self.newer_downloads(&acct_id);
        let mut seen_ids: BTreeSet<String> = BTreeSet::new();
        let mut count: BTreeMap<(bool, String), i64> = BTreeMap::new();
        let mask = self.m.account.mask.clone();
        for r in &rows {
            let fp = facts::fingerprint(&mask, r.date.as_str(), r.amount, &r.match_text);
            let n = count.entry((r.pending, fp.clone())).or_insert(0);
            *n += 1;
            let kind = if r.pending { "csv-pending" } else { "csv" };
            let id = format!("{}|{kind}|{fp}#{n}", self.src());
            seen_ids.insert(id.clone());
            let existing = self.db.lookup(&k("obs/id"), &Value::str(&id));
            let at = format!("row:{}", r.line);
            let st = if r.pending {
                let lapsed =
                    existing.and_then(|e| status_of(self.db, e)).as_deref() == Some("lapsed");
                let later = newer
                    .iter()
                    .find(|(start, end, _)| day(start) <= day(&r.date) && day(&r.date) <= day(end));
                if let (false, Some((_, _, when))) = (lapsed, later) {
                    self.press(
                        "out-of-order",
                        &at,
                        format!(
                            "pending {} {} is older than a download taken {} that covers its date and does not show it; recorded as lapsed",
                            r.date.as_str(),
                            r.match_text,
                            when.as_str()
                        ),
                    );
                }
                if lapsed || later.is_some() {
                    "lapsed"
                } else {
                    "pending"
                }
            } else {
                "posted"
            };
            if r.pending && st == "pending" {
                self.press(
                    "pending",
                    &at,
                    format!(
                        "{} {} {} is an authorisation, not a posted line; kept apart from posted lines and out of the balance check",
                        r.date.as_str(),
                        r.match_text,
                        money(r.amount)
                    ),
                );
            }
            if let (Some(e), Some(b)) = (existing, r.balance)
                && let Some(old) = dec_of(self.db, e, "bank.csv/balance-after")
                && old != b
            {
                self.press(
                    "balance-changed",
                    &at,
                    format!(
                        "an earlier download showed {} after this line; this one shows {}",
                        money(old),
                        money(b)
                    ),
                );
            }
            let region = facts::region(&mut self.tx, &input.file.hash, &at);
            let o = line(
                &mut self.tx,
                Line {
                    id,
                    doc: &feed,
                    region,
                    account: &acct,
                    commodity: &cur,
                    mask: &mask,
                    date: r.date.clone(),
                    amount: r.amount,
                    description: r.description.clone(),
                    match_text: r.match_text.clone(),
                    reference: r.check.clone(),
                    status: status(st),
                },
            );
            self.tx.add(&o, &k("bank.csv/occurrence"), Value::Int(*n));
            if let Some(t) = &r.kind {
                self.tx.add(&o, &k("bank.csv/type"), Value::str(t));
            }
            if let Some(b) = r.balance {
                self.tx.add(&o, &k("bank.csv/balance-after"), Value::Dec(b));
            }
        }
        self.missing(&acct_id, &seen_ids);
        self.press(
            "no-identity",
            "file",
            "bank CSV lines carry no id; identity is account|date|amount|text plus a same-day occurrence number",
        );
        Projection {
            install: install(),
            tx: self.tx,
            pressure: self.pressure,
        }
    }

    fn check_order(&mut self, rows: &[Row]) {
        let posted: Vec<&Row> = rows.iter().filter(|r| !r.pending).collect();
        for w in posted.windows(2) {
            if day(&w[1].date) < day(&w[0].date) {
                self.press(
                    "order-lost",
                    format!("row:{}", w[1].line),
                    "lines are not in the order the mapping says; occurrence numbers may not match other downloads",
                );
                return;
            }
        }
    }

    /// Opening + lines = balance, line by line, over posted lines that show a balance.
    fn check_balances(&mut self, rows: &[Row]) {
        let mut prev: Option<Decimal> = None;
        for r in rows.iter().filter(|r| !r.pending) {
            let Some(b) = r.balance else {
                self.press(
                    "null",
                    format!("row:{}", r.line),
                    "a posted line without a running balance; the check restarts after it",
                );
                prev = None;
                continue;
            };
            if let Some(p) = prev
                && p + r.amount != b
            {
                self.press(
                    "balance-mismatch",
                    format!("row:{}", r.line),
                    format!(
                        "{} + {} = {}, but the download shows {}",
                        money(p),
                        money(r.amount),
                        money(p + r.amount),
                        money(b)
                    ),
                );
            }
            prev = Some(b);
        }
    }

    /// Downloads of this account already in the log, taken after this one: (window start, end, taken at).
    fn newer_downloads(&self, acct_id: &str) -> Vec<(Date, Date, Instant)> {
        let prefix = format!("{}|csv-download|{acct_id}|", self.src());
        let Some(oa) = self.db.attr(&k("bank.csv/observed-at")) else {
            return Vec::new();
        };
        self.db
            .attr_values(oa.id)
            .filter_map(|(v, e)| {
                let Value::Instant(when) = v else { return None };
                let id = self.db.get(e, &k("doc/id")).and_then(Value::as_str)?;
                if !id.starts_with(&prefix) || when <= &self.input.observed_at {
                    return None;
                }
                match (
                    self.db.get(e, &k("doc/period-start")),
                    self.db.get(e, &k("doc/period-end")),
                ) {
                    (Some(Value::Date(a)), Some(Value::Date(b))) => {
                        Some((a.clone(), b.clone(), when.clone()))
                    }
                    _ => None,
                }
            })
            .collect()
    }

    /// When an observation was last seen: the newest download that has a region of it.
    fn last_seen(&self, o: Eid) -> Option<Instant> {
        let (ra, rf, df) = (
            self.db.attr(&k("obs/region"))?.id,
            self.db.attr(&k("region/file"))?.id,
            self.db.attr(&k("doc/file"))?.id,
        );
        self.db
            .values(o, ra)
            .filter_map(Value::as_ref)
            .filter_map(|r| self.db.value(r, rf).and_then(Value::as_ref))
            .flat_map(|f| self.db.with_value(df, &Value::Ref(f)).collect::<Vec<_>>())
            .filter_map(|d| match self.db.get(d, &k("bank.csv/observed-at")) {
                Some(Value::Instant(i)) => Some(i.clone()),
                _ => None,
            })
            .max()
    }

    /// Lines an earlier download showed inside this download's window that this newer one does not.
    fn missing(&mut self, acct_id: &str, seen: &BTreeSet<String>) {
        let Some((start, end)) = self.window.clone() else {
            return;
        };
        let prefix = format!("{}|csv", self.src());
        for e in earlier(self.db, acct_id, &prefix) {
            let Some(id) = self
                .db
                .get(e, &k("obs/id"))
                .and_then(Value::as_str)
                .map(str::to_string)
            else {
                continue;
            };
            if seen.contains(&id) {
                continue;
            }
            let Some(d) = date_of(self.db, e) else {
                continue;
            };
            if day(&d) < day(&start) || day(&end) < day(&d) {
                continue;
            }
            if self
                .last_seen(e)
                .is_none_or(|t| t >= self.input.observed_at)
            {
                continue;
            }
            match status_of(self.db, e).as_deref() {
                Some("pending") => {
                    if let Some(s) = status("lapsed") {
                        self.tx.add(&EntRef::Id(e), &k("obs/status"), s);
                    }
                    self.press(
                        "pending-lapsed",
                        &id,
                        format!(
                            "a pending line from {} is not in this newer download that covers its date; marked lapsed",
                            d.as_str()
                        ),
                    );
                }
                Some("lapsed") => {}
                _ => self.press(
                    "vanished",
                    &id,
                    format!(
                        "a posted line from {} that an earlier download showed is not in this newer one; nothing was changed",
                        d.as_str()
                    ),
                ),
            }
        }
    }
}
