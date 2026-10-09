//! `pl` and `bs`: the profit and loss and the balance sheet from the local ledger (accrual or cash basis),
//! checked account by account against Xero's own report for the same dates and basis when the record holds
//! one (`report fetch`). Zero calls.

use crate::app::{App, CliError, Res};
use crate::cmds::Row;
use crate::obj;
use crate::out::dec;
use ownpurse_core::balances::{Balances, compare};
use ownpurse_core::dates::fiscal_year_start;
use ownpurse_core::statements::{
    CURRENT_YEAR_EARNINGS, balance_sheet, balance_sheet_from, profit_and_loss, xero_statement,
};
use ownpurse_core::{Chart, Ledger, LedgerBasis};
use rust_decimal::Decimal;
use serde_json::json;
use std::collections::BTreeSet;

pub struct StatementArgs {
    pub org: String,
    /// `pl`: the period; `bs`: `to` is the as-of date
    pub from: Option<String>,
    pub to: String,
    pub cash: bool,
    pub strict: bool,
    /// `bs`: open from Xero's balance sheet at this date (default: the prior fiscal year-end when cached)
    pub since: Option<String>,
    /// `bs`: local documents only
    pub full_history: bool,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Pl,
    Bs,
}

pub fn pl(app: &mut App, a: &StatementArgs) -> Res<()> {
    statement(app, a, Kind::Pl)
}

pub fn bs(app: &mut App, a: &StatementArgs) -> Res<()> {
    statement(app, a, Kind::Bs)
}

fn fy_start(app: &mut App, tid: &str, as_of: &str) -> Res<String> {
    let (m, d) = app.record()?.fy_end(tid)?;
    fiscal_year_start(as_of, m, d).ok_or_else(|| CliError::usage(format!("not a date: {as_of}")))
}

/// The section a row belongs in and whether it shows credit-positive.
fn place(chart: &Chart, id: &str, kind: Kind) -> (&'static str, bool) {
    if id == CURRENT_YEAR_EARNINGS {
        return ("Equity", true);
    }
    let a = chart.get(id);
    match (kind, a.class.as_str()) {
        (Kind::Pl, "REVENUE") => ("Revenue", true),
        (Kind::Pl, _) if a.kind == "DIRECTCOSTS" => ("Cost of sales", false),
        (Kind::Pl, _) => ("Expenses", false),
        (Kind::Bs, "ASSET") => ("Assets", false),
        (Kind::Bs, "LIABILITY") => ("Liabilities", true),
        (Kind::Bs, _) => ("Equity", true),
    }
}

const ORDER: [&str; 6] = [
    "Revenue",
    "Cost of sales",
    "Expenses",
    "Assets",
    "Liabilities",
    "Equity",
];

fn statement(app: &mut App, a: &StatementArgs, kind: Kind) -> Res<()> {
    let tids = app.resolve(&a.org)?;
    if tids.len() != 1 {
        return Err(CliError::usage("pl and bs take one org.".to_string()));
    }
    let tid = &tids[0];
    let basis = if a.cash {
        LedgerBasis::Cash
    } else {
        LedgerBasis::Accrual
    };
    app.ensure_ledger(tid)?;
    let lb = {
        let v = app.view()?;
        Ledger::build_basis(&v, tid, basis)?
    };
    let fy = fy_start(app, tid, &a.to)?;
    let from = a.from.clone().unwrap_or_else(|| fy.clone());
    let paid = if a.cash { "true" } else { "false" };
    // bs: the opening, Xero's own balance sheet (same basis) at the prior fiscal year-end unless told otherwise
    let mut opened: Option<(String, String)> = None;
    let mut opening: Option<Balances> = None;
    if kind == Kind::Bs && !a.full_history {
        let date = match &a.since {
            Some(d) => d.clone(),
            None => ownpurse_core::dates::add_days(&fy, -1).unwrap_or_default(),
        };
        let p = ownpurse_core::canon::canonical(&json!({"date": date, "paymentsOnly": paid}));
        match app.record()?.report(tid, "BalanceSheet", &p)? {
            Some(r) => {
                let st =
                    xero_statement(&r.payload).map_err(|e| CliError::internal(e.to_string()))?;
                opening = Some(st.balances);
                opened = Some((date, r.fetched_at));
            }
            None if a.since.is_some() => {
                return Err(CliError::usage(format!(
                    "No Xero balance sheet cached at {date} ({} basis).",
                    if a.cash { "cash" } else { "accrual" }
                ))
                .hint(format!(
                    "`ownpurse report fetch {} balancesheet --date {date}{}` (one call), or --full-history.",
                    app.alias_of(tid),
                    if a.cash { " --cash" } else { "" }
                )));
            }
            None => {}
        }
    }
    let (local, report, params) = match kind {
        Kind::Pl => (
            profit_and_loss(&lb.postings, &lb.chart, &from, &a.to),
            "ProfitAndLoss",
            json!({"fromDate": from, "paymentsOnly": paid, "toDate": a.to}),
        ),
        Kind::Bs => (
            match (&opening, &opened) {
                (Some(op), Some((d, _))) => {
                    balance_sheet_from(&lb.postings, &lb.chart, &a.to, &fy, op, d)
                }
                _ => balance_sheet(&lb.postings, &lb.chart, &a.to, &fy),
            },
            "BalanceSheet",
            json!({"date": a.to, "paymentsOnly": paid}),
        ),
    };
    let params = ownpurse_core::canon::canonical(&params);
    let cached = app.record()?.report(tid, report, &params)?;
    let xero = match &cached {
        Some(r) => Some(xero_statement(&r.payload).map_err(|e| CliError::internal(e.to_string()))?),
        None => None,
    };
    let empty = Balances::new();
    let xb = xero.as_ref().map(|x| &x.balances).unwrap_or(&empty);
    let diffs = if xero.is_some() {
        compare(&local, xb)
    } else {
        Vec::new()
    };
    let ids: BTreeSet<&String> = local.keys().chain(xb.keys()).collect();
    let mut rows: Vec<(usize, String, String, Row)> = Vec::new();
    let mut totals: Vec<(&str, Decimal, Decimal)> = Vec::new();
    for id in ids {
        let (section, credit) = place(&lb.chart, id, kind);
        let sign = if credit { -Decimal::ONE } else { Decimal::ONE };
        let acct = lb.chart.get(id);
        let label = if id == CURRENT_YEAR_EARNINGS {
            "Current Year Earnings".to_string()
        } else {
            xero.as_ref()
                .and_then(|x| x.labels.get(id))
                .cloned()
                .unwrap_or_else(|| acct.name.clone())
        };
        let l = local.get(id).copied().unwrap_or_default();
        let x = xb.get(id).copied().unwrap_or_default();
        let mut row = obj![
            ("section", section),
            ("code", acct.code.as_str()),
            ("account", label.as_str()),
            ("amount", dec(sign * l)),
        ];
        if xero.is_some() {
            row.insert("xero".into(), dec(sign * x));
            row.insert("difference".into(), dec(sign * (l - x)));
        }
        let pos = ORDER.iter().position(|s| *s == section).unwrap_or(9);
        match totals.iter_mut().find(|t| t.0 == section) {
            Some(t) => {
                t.1 += sign * l;
                t.2 += sign * x;
            }
            None => totals.push((section, sign * l, sign * x)),
        }
        rows.push((pos, acct.code.clone(), label, row));
    }
    rows.sort_by(|a, b| (a.0, &a.1, &a.2).cmp(&(b.0, &b.1, &b.2)));
    totals.sort_by_key(|t| ORDER.iter().position(|s| *s == t.0).unwrap_or(9));
    let total_rows: Vec<Row> = totals
        .iter()
        .map(|(s, l, x)| {
            let mut r = obj![("section", *s), ("total", dec(*l))];
            if xero.is_some() {
                r.insert("xero".into(), dec(*x));
            }
            r
        })
        .collect();
    let alias = app.alias_of(tid);
    let mut out = app.out();
    crate::manifest::view_note(&mut out, app);
    out.field("org", alias.as_str());
    match kind {
        Kind::Pl => {
            out.field("from", from.as_str()).field("to", a.to.as_str());
            let net: Decimal = -local.values().copied().sum::<Decimal>();
            out.field("net_income", dec(net));
        }
        Kind::Bs => {
            out.field("as_of", a.to.as_str());
            let sum: Decimal = local.values().copied().sum();
            out.field("balanced", sum == Decimal::ZERO);
            match &opened {
                Some((d, _)) => out.field(
                    "opening",
                    format!("Xero balance sheet {d} + local activity"),
                ),
                None => out.field("opening", "local full history"),
            };
        }
    }
    out.field("basis", if a.cash { "cash" } else { "accrual" });
    match &cached {
        Some(r) => {
            out.field("xero_report", r.fetched_at.as_str())
                .field("matches_xero", diffs.is_empty())
                .field("differences", diffs.len());
        }
        None => {
            out.field("xero_report", "none cached");
        }
    }
    let cols: Vec<&str> = if xero.is_some() {
        vec!["section", "code", "account", "amount", "xero", "difference"]
    } else {
        vec!["section", "code", "account", "amount"]
    };
    let all: Vec<Row> = rows.into_iter().map(|r| r.3).collect();
    let n = all.len();
    let shown = app.limit(&all);
    out.table("lines", &cols, shown, Some(n));
    let tcols: Vec<&str> = if xero.is_some() {
        vec!["section", "total", "xero"]
    } else {
        vec!["section", "total"]
    };
    out.table("totals", &tcols, total_rows, None);
    if cached.is_none() {
        let k = if kind == Kind::Pl {
            "pl"
        } else {
            "balancesheet"
        };
        let span = if kind == Kind::Pl {
            format!("--from {from} --to {}", a.to)
        } else {
            format!("--date {}", a.to)
        };
        out.hint(format!(
            "No Xero report cached for these dates and basis: `ownpurse report fetch {alias} {k} {span}{}` (one call) to check it.",
            if a.cash { " --cash" } else { "" }
        ));
    } else if !diffs.is_empty() {
        out.hint(
            "Drill in with `ownpurse gl <org> --account <code|name> --from <date> --to <date>`."
                .to_string(),
        );
    }
    out.emit();
    if a.strict && !diffs.is_empty() {
        return Err(CliError::runtime(
            "statement_mismatch",
            format!("{} account(s) differ from Xero's report.", diffs.len()),
        ));
    }
    Ok(())
}
