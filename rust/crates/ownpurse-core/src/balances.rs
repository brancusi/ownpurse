//! Trial balances (local and from Xero's cached report), differences, and the balance basis: Xero's opening
//! trial balance at the prior fiscal year-end plus local activity (default; matches Xero), or full local history.

use crate::dates::{add_days, fiscal_year_start};
use crate::ledger::{Chart, Ledger, Posting};
use crate::money::{MoneyError, ZERO, money};
use crate::record::{Record, RecordError};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;

/// Net debit balance per account id. BTreeMap keeps iteration deterministic.
pub type Balances = BTreeMap<String, Decimal>;

/// Net debit balance per account at `as_of`, Xero-style: balance-sheet accounts cumulative, P&L accounts
/// from the fiscal-year start, earlier P&L closed into retained earnings. With `opening` (a TB at
/// `opening_date`), only postings after `opening_date` are added.
pub fn trial_balance(
    postings: &[Posting],
    chart: &Chart,
    as_of: &str,
    fy_start: &str,
    opening: Option<&Balances>,
    opening_date: Option<&str>,
) -> Balances {
    let mut bal: Balances = BTreeMap::new();
    let re = chart.system("RETAINEDEARNINGS");
    if let Some(op) = opening {
        for (acct, amt) in op {
            let into = if chart.is_pl(acct) && opening_date.is_some_and(|d| d < fy_start) {
                &re
            } else {
                acct
            };
            *bal.entry(into.clone()).or_insert(ZERO) += *amt;
        }
    }
    for p in postings {
        if p.date.as_str() > as_of || opening_date.is_some_and(|d| p.date.as_str() <= d) {
            continue;
        }
        let into = if chart.is_pl(&p.account_id) && p.date.as_str() < fy_start {
            &re
        } else {
            &p.account_id
        };
        *bal.entry(into.clone()).or_insert(ZERO) += p.amount;
    }
    bal.retain(|_, v| *v != ZERO);
    bal
}

/// Net debit balance per AccountID from a Xero TrialBalance report (YTD columns), plus the row labels.
pub fn xero_tb_balances(
    report: &Value,
) -> Result<(Balances, BTreeMap<String, String>), XeroTbError> {
    let rep = report
        .get("Reports")
        .and_then(|r| r.get(0))
        .ok_or(XeroTbError::Shape("no Reports[0]".into()))?;
    let mut header: Vec<String> = Vec::new();
    let mut out: Balances = BTreeMap::new();
    let mut labels = BTreeMap::new();
    fn walk(
        rows: &[Value],
        header: &mut Vec<String>,
        out: &mut Balances,
        labels: &mut BTreeMap<String, String>,
    ) -> Result<(), XeroTbError> {
        for r in rows {
            match r.get("RowType").and_then(Value::as_str) {
                Some("Header") => {
                    *header = r
                        .get("Cells")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .map(|c| {
                            c.get("Value")
                                .and_then(Value::as_str)
                                .unwrap_or("")
                                .to_string()
                        })
                        .collect();
                }
                Some("Section") => walk(
                    r.get("Rows")
                        .and_then(Value::as_array)
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]),
                    header,
                    out,
                    labels,
                )?,
                Some("Row") => {
                    let cells: &[Value] = r
                        .get("Cells")
                        .and_then(Value::as_array)
                        .map(|v| v.as_slice())
                        .unwrap_or(&[]);
                    let acct = cells
                        .iter()
                        .flat_map(|c| {
                            c.get("Attributes")
                                .and_then(Value::as_array)
                                .into_iter()
                                .flatten()
                        })
                        .find(|a| a.get("Id").and_then(Value::as_str) == Some("account"))
                        .and_then(|a| a.get("Value").and_then(Value::as_str));
                    let Some(acct) = acct.filter(|a| !a.is_empty()) else {
                        continue;
                    };
                    let mut ytd_dr = ZERO;
                    let mut ytd_cr = ZERO;
                    for (i, c) in cells.iter().enumerate().skip(1) {
                        let v = c
                            .get("Value")
                            .filter(|v| !matches!(v, Value::String(s) if s.is_empty()))
                            .cloned()
                            .unwrap_or(Value::String("0".into()));
                        let amount = money(Some(&v)).map_err(XeroTbError::Money)?;
                        match header.get(i).map(String::as_str) {
                            Some("YTD Debit") => ytd_dr = amount,
                            Some("YTD Credit") => ytd_cr = amount,
                            _ => {}
                        }
                    }
                    *out.entry(acct.to_string()).or_insert(ZERO) += ytd_dr - ytd_cr;
                    labels.insert(
                        acct.to_string(),
                        cells
                            .first()
                            .and_then(|c| c.get("Value"))
                            .and_then(Value::as_str)
                            .unwrap_or("")
                            .to_string(),
                    );
                }
                _ => {}
            }
        }
        Ok(())
    }
    let rows = rep
        .get("Rows")
        .and_then(Value::as_array)
        .map(|v| v.as_slice())
        .unwrap_or(&[]);
    walk(rows, &mut header, &mut out, &mut labels)?;
    if !header.iter().any(|h| h == "YTD Debit") {
        return Err(XeroTbError::Shape(format!(
            "unexpected TrialBalance header: {header:?}"
        )));
    }
    Ok((out, labels))
}

#[derive(Debug)]
pub enum XeroTbError {
    Shape(String),
    Money(MoneyError),
}

impl std::fmt::Display for XeroTbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            XeroTbError::Shape(s) => write!(f, "Xero TrialBalance: {s}"),
            XeroTbError::Money(e) => write!(f, "Xero TrialBalance: {e}"),
        }
    }
}

impl std::error::Error for XeroTbError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diff {
    pub account_id: String,
    pub local: Decimal,
    pub xero: Decimal,
    pub difference: Decimal,
}

/// Accounts whose balances differ, sorted by account id.
pub fn compare(local: &Balances, xero: &Balances) -> Vec<Diff> {
    let keys: std::collections::BTreeSet<&String> = local.keys().chain(xero.keys()).collect();
    keys.into_iter()
        .filter_map(|k| {
            let a = local.get(k).copied().unwrap_or(ZERO);
            let b = xero.get(k).copied().unwrap_or(ZERO);
            (a != b).then(|| Diff {
                account_id: k.clone(),
                local: a,
                xero: b,
                difference: a - b,
            })
        })
        .collect()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Xero opening TB when cached, else full history.
    Auto,
    /// Force full local history.
    History,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BasisKind {
    XeroOpening,
    History,
}

/// Which balances to show and how they compare with Xero (the basis strip, invariant V6).
#[derive(Debug, Clone)]
pub struct Basis {
    pub kind: BasisKind,
    pub as_of: String,
    pub fy_start: String,
    pub opening: Option<Balances>,
    pub opening_date: Option<String>,
    pub xero_at_as_of: Option<Balances>,
    pub xero_fetched: Option<String>,
    pub balances: Balances,
    pub diffs: Vec<Diff>,
}

impl Basis {
    /// Some(true) matches Xero, Some(false) differs, None not compared (no cached Xero TB at as_of).
    pub fn matches_xero(&self) -> Option<bool> {
        self.xero_at_as_of.as_ref().map(|_| self.diffs.is_empty())
    }

    /// One account's balance just before `since` on this basis (P&L opens at zero at the fiscal-year start).
    pub fn opening_for(&self, ledger: &Ledger, account_id: &str, since: &str) -> Decimal {
        let day_before = add_days(since, -1).unwrap_or_default();
        let use_opening = self.kind == BasisKind::XeroOpening
            && self.opening_date.as_deref().is_some_and(|od| since > od);
        let bal = trial_balance(
            &ledger.postings,
            &ledger.chart,
            &day_before,
            &self.fy_start,
            if use_opening {
                self.opening.as_ref()
            } else {
                None
            },
            if use_opening {
                self.opening_date.as_deref()
            } else {
                None
            },
        );
        if ledger.chart.is_pl(account_id) && since <= self.fy_start.as_str() {
            return ZERO;
        }
        bal.get(account_id).copied().unwrap_or(ZERO)
    }
}

#[derive(Debug)]
pub enum BasisError {
    Record(RecordError),
    XeroTb(XeroTbError),
    BadDate(String),
}

impl std::fmt::Display for BasisError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BasisError::Record(e) => e.fmt(f),
            BasisError::XeroTb(e) => e.fmt(f),
            BasisError::BadDate(d) => write!(f, "not a date: {d}"),
        }
    }
}

impl std::error::Error for BasisError {}

/// Resolve the basis for one org at `as_of`.
pub fn resolve(
    record: &Record,
    ledger: &Ledger,
    tenant_id: &str,
    as_of: &str,
    fy_end: (u32, u32),
    mode: Mode,
) -> Result<Basis, BasisError> {
    let fy_start = fiscal_year_start(as_of, fy_end.0, fy_end.1)
        .ok_or_else(|| BasisError::BadDate(as_of.into()))?;
    let prior = add_days(&fy_start, -1).ok_or_else(|| BasisError::BadDate(fy_start.clone()))?;
    let opening_snap = if mode == Mode::Auto {
        record
            .tb_snapshot(tenant_id, &prior)
            .map_err(BasisError::Record)?
    } else {
        None
    };
    let cur = record
        .tb_snapshot(tenant_id, as_of)
        .map_err(BasisError::Record)?;
    let mut b = Basis {
        kind: if opening_snap.is_some() {
            BasisKind::XeroOpening
        } else {
            BasisKind::History
        },
        as_of: as_of.into(),
        fy_start: fy_start.clone(),
        opening: None,
        opening_date: None,
        xero_at_as_of: None,
        xero_fetched: None,
        balances: BTreeMap::new(),
        diffs: vec![],
    };
    if let Some(snap) = opening_snap {
        b.opening = Some(
            xero_tb_balances(&snap.payload)
                .map_err(BasisError::XeroTb)?
                .0,
        );
        b.opening_date = Some(prior);
    }
    b.balances = trial_balance(
        &ledger.postings,
        &ledger.chart,
        as_of,
        &fy_start,
        b.opening.as_ref(),
        b.opening_date.as_deref(),
    );
    // a branch changing a period the Xero opening TB already covers: its effect is added on top (the opening
    // is main's; the branch's proposed postings dated on or before it would otherwise be skipped)
    if let Some(od) = b.opening_date.as_deref() {
        for (acct, amt) in manifest_prior_delta(ledger, od, as_of, &fy_start) {
            *b.balances.entry(acct).or_insert(ZERO) += amt;
        }
        b.balances.retain(|_, v| *v != ZERO);
    }
    if let Some(c) = cur {
        let xb = xero_tb_balances(&c.payload).map_err(BasisError::XeroTb)?.0;
        b.diffs = compare(&b.balances, &xb);
        b.xero_at_as_of = Some(xb);
        b.xero_fetched = Some(c.fetched_at);
    }
    Ok(b)
}

/// The effect of a manifest's documents dated on or before `opening_date` (already inside main's Xero
/// opening TB): a new document adds its postings, a changed one its postings less main's, and a
/// void removes its postings. P&L earlier than `fy_start` lands in retained earnings. Empty on main.
pub fn manifest_prior_delta(
    ledger: &Ledger,
    opening_date: &str,
    as_of: &str,
    fy_start: &str,
) -> Balances {
    let mut out: Balances = BTreeMap::new();
    let re = ledger.chart.system("RETAINEDEARNINGS");
    let mut add = |acct: &str, date: &str, amt: Decimal| {
        let into = if ledger.chart.is_pl(acct) && date < fy_start {
            re.clone()
        } else {
            acct.to_string()
        };
        *out.entry(into).or_insert(ZERO) += amt;
    };
    // every document the manifest creates or changes: its postings now, less main's postings of it
    let mut touched: std::collections::HashSet<(&str, &str)> = ledger
        .postings
        .iter()
        .filter(|p| p.proposed.is_some())
        .map(|p| (p.source, p.source_id.as_str()))
        .collect();
    touched.extend(
        ledger
            .replaced
            .iter()
            .map(|p| (p.source, p.source_id.as_str())),
    );
    let within = |d: &str| d <= opening_date && d <= as_of;
    for p in &ledger.postings {
        if within(&p.date) && touched.contains(&(p.source, p.source_id.as_str())) {
            add(&p.account_id, &p.date, p.amount);
        }
    }
    for p in &ledger.replaced {
        if within(&p.date) {
            add(&p.account_id, &p.date, -p.amount);
        }
    }
    for p in &ledger.ghosts {
        if p.date.as_str() <= opening_date && p.date.as_str() <= as_of {
            add(&p.account_id, &p.date, -p.amount);
        }
    }
    out.retain(|_, v| *v != ZERO);
    out
}
