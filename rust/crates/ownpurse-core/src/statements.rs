//! Financial statements from the local ledger (profit and loss, balance sheet) and Xero's own
//! ProfitAndLoss / BalanceSheet reports read back to the same shape, so the two compare account by account.
//! Amounts are net debit (debit positive) throughout; presentation signs are the caller's.

use crate::balances::{Balances, trial_balance};
use crate::ledger::{Chart, Posting};
use crate::money::{MoneyError, ZERO, money};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;

/// The pseudo account a balance sheet carries the current year's profit under (Xero shows it as a row with
/// no account of its own).
pub const CURRENT_YEAR_EARNINGS: &str = "(current year earnings)";

/// Net debit movement per P&L account between `from` and `to` (inclusive).
pub fn profit_and_loss(postings: &[Posting], chart: &Chart, from: &str, to: &str) -> Balances {
    let mut out = Balances::new();
    for p in postings {
        let d = p.date.as_str();
        if d < from || d > to || !chart.is_pl(&p.account_id) {
            continue;
        }
        *out.entry(p.account_id.clone()).or_insert(ZERO) += p.amount;
    }
    out.retain(|_, v| *v != ZERO);
    out
}

/// Net debit balance per balance-sheet account at `as_of` from the full local history, with earlier years'
/// profit in retained earnings and this fiscal year's under [`CURRENT_YEAR_EARNINGS`].
pub fn balance_sheet(postings: &[Posting], chart: &Chart, as_of: &str, fy_start: &str) -> Balances {
    let tb = trial_balance(postings, chart, as_of, fy_start, None, None);
    let mut out = Balances::new();
    let mut cye = ZERO;
    for (acct, amt) in tb {
        if chart.is_pl(&acct) {
            cye += amt;
        } else {
            out.insert(acct, amt);
        }
    }
    if cye != ZERO {
        out.insert(CURRENT_YEAR_EARNINGS.into(), cye);
    }
    out
}

/// The balance sheet at `as_of` opening from a balance sheet at `opening_date` (Xero's, same basis): its current
/// year earnings close into retained earnings when `opening_date` is before `fy_start`; local postings after
/// `opening_date` are added (this fiscal year's P&L under [`CURRENT_YEAR_EARNINGS`], earlier into retained
/// earnings).
pub fn balance_sheet_from(
    postings: &[Posting],
    chart: &Chart,
    as_of: &str,
    fy_start: &str,
    opening: &Balances,
    opening_date: &str,
) -> Balances {
    let re = chart.system("RETAINEDEARNINGS");
    let mut out = Balances::new();
    for (acct, amt) in opening {
        let into = if acct == CURRENT_YEAR_EARNINGS && opening_date < fy_start {
            &re
        } else {
            acct
        };
        *out.entry(into.clone()).or_insert(ZERO) += *amt;
    }
    for p in postings {
        let d = p.date.as_str();
        if d <= opening_date || d > as_of {
            continue;
        }
        let into = if !chart.is_pl(&p.account_id) {
            p.account_id.as_str()
        } else if d >= fy_start {
            CURRENT_YEAR_EARNINGS
        } else {
            re.as_str()
        };
        *out.entry(into.to_string()).or_insert(ZERO) += p.amount;
    }
    out.retain(|_, v| *v != ZERO);
    out
}

/// A Xero ProfitAndLoss or BalanceSheet report, first value column, per AccountID (net debit), with each row's
/// label and section.
#[derive(Debug, Clone, Default)]
pub struct XeroStatement {
    pub balances: Balances,
    pub labels: BTreeMap<String, String>,
    pub sections: BTreeMap<String, String>,
}

#[derive(Debug)]
pub enum StatementError {
    Shape(String),
    Money(MoneyError),
}

impl std::fmt::Display for StatementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatementError::Shape(s) => write!(f, "Xero report: {s}"),
            StatementError::Money(e) => write!(f, "Xero report: {e}"),
        }
    }
}

impl std::error::Error for StatementError {}

/// Read a Xero ProfitAndLoss or BalanceSheet payload. Xero shows each section in its natural sign: on a P&L,
/// "Less …" sections (costs, expenses) debit-positive and the rest credit-positive; on a balance sheet,
/// assets debit-positive and everything from the first liabilities or equity section on credit-positive.
pub fn xero_statement(report: &Value) -> Result<XeroStatement, StatementError> {
    let rep = report
        .get("Reports")
        .and_then(|r| r.get(0))
        .ok_or_else(|| StatementError::Shape("no Reports[0]".into()))?;
    let id = rep.get("ReportID").and_then(Value::as_str).unwrap_or("");
    let balance_sheet = id == "BalanceSheet";
    if !balance_sheet && id != "ProfitAndLoss" {
        return Err(StatementError::Shape(format!(
            "not a ProfitAndLoss or BalanceSheet report ({id})"
        )));
    }
    let mut st = XeroStatement::default();
    let mut credit_side = false;
    for sec in rep
        .get("Rows")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if sec.get("RowType").and_then(Value::as_str) != Some("Section") {
            continue;
        }
        let title = sec.get("Title").and_then(Value::as_str).unwrap_or("");
        if balance_sheet {
            if title.contains("Liabilit") || title.contains("Equity") {
                credit_side = true;
            }
        } else if !title.is_empty() {
            credit_side = !title.starts_with("Less");
        }
        for row in sec
            .get("Rows")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if row.get("RowType").and_then(Value::as_str) != Some("Row") {
                continue;
            }
            let cells = row.get("Cells").and_then(Value::as_array);
            let first = cells.and_then(|c| c.first());
            let label = first
                .and_then(|c| c.get("Value"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let account = first
                .and_then(|c| c.get("Attributes"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .find(|a| a.get("Id").and_then(Value::as_str) == Some("account"))
                .and_then(|a| a.get("Value"))
                .and_then(Value::as_str)
                .map(str::to_string);
            // Xero gives this row an account id of its own; it is the year's profit, not an account
            let key = match account {
                _ if label == "Current Year Earnings" => CURRENT_YEAR_EARNINGS.into(),
                Some(a) => a,
                None => continue,
            };
            let shown = money(cells.and_then(|c| c.get(1)).and_then(|c| c.get("Value")))
                .map_err(StatementError::Money)?;
            let net = if credit_side { -shown } else { shown };
            *st.balances.entry(key.clone()).or_insert(ZERO) += net;
            st.labels.insert(key.clone(), label);
            st.sections.insert(key, title.to_string());
        }
    }
    st.balances.retain(|_, v| *v != Decimal::ZERO);
    Ok(st)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn row(id: Option<&str>, label: &str, v: &str) -> Value {
        let mut first = json!({"Value": label});
        if let Some(i) = id {
            first["Attributes"] = json!([{"Id": "account", "Value": i}]);
        }
        json!({"RowType": "Row", "Cells": [first, {"Value": v}]})
    }

    fn books() -> crate::ledger::Documents {
        let acct = |id: &str, code: &str, class: &str, kind: &str, system: &str| {
            json!({"AccountID": id, "Code": code, "Name": id, "Class": class, "Type": kind,
                   "SystemAccount": system, "Status": "ACTIVE"})
        };
        crate::ledger::Documents {
            accounts: vec![
                acct("bank", "090", "ASSET", "BANK", ""),
                acct("ar", "610", "ASSET", "CURRENT", "DEBTORS"),
                acct("re", "960", "EQUITY", "EQUITY", "RETAINEDEARNINGS"),
                acct("sales", "200", "REVENUE", "REVENUE", ""),
                acct("svc", "210", "REVENUE", "REVENUE", ""),
                acct("rent", "469", "EXPENSE", "EXPENSE", ""),
                acct("loan", "270", "LIABILITY", "CURRENTLIABILITY", ""),
            ],
            invoices: vec![
                json!({"InvoiceID": "inv-1", "Type": "ACCREC", "Status": "AUTHORISED",
                "InvoiceNumber": "INV-1", "Date": "2025-03-01", "LineAmountTypes": "NoTax", "Total": 300,
                "Contact": {"Name": "Client"},
                "LineItems": [{"AccountCode": "200", "LineAmount": 200, "TaxAmount": 0},
                              {"AccountCode": "210", "LineAmount": 100, "TaxAmount": 0}]}),
            ],
            payments: vec![
                json!({"PaymentID": "pay-1", "PaymentType": "ACCRECPAYMENT", "Status": "AUTHORISED",
                "Date": "2025-04-01", "Amount": 100, "Account": {"Code": "090"},
                "Invoice": {"InvoiceID": "inv-1"}}),
            ],
            manual_journals: vec![
                json!({"ManualJournalID": "mj-1", "Status": "POSTED", "Date": "2025-05-01",
                    "ShowOnCashBasisReports": false, "JournalLines": [
                        {"AccountCode": "469", "LineAmount": 50}, {"AccountCode": "270", "LineAmount": -50}]}),
                json!({"ManualJournalID": "mj-2", "Status": "POSTED", "Date": "2025-06-01",
                    "ShowOnCashBasisReports": true, "JournalLines": [
                        {"AccountCode": "469", "LineAmount": 20}, {"AccountCode": "270", "LineAmount": -20}]}),
            ],
            ..Default::default()
        }
    }

    #[test]
    fn cash_basis_posts_invoices_as_paid_and_leaves_out_accrual_journals() {
        use crate::ledger::{Ledger, LedgerBasis};
        let d = |v: i64| Decimal::new(v, 2);
        let cash = Ledger::from_documents_basis(&books(), LedgerBasis::Cash).unwrap();
        let pl = profit_and_loss(&cash.postings, &cash.chart, "2025-01-01", "2025-12-31");
        // a third of the invoice is paid: a third of each line, on the payment date
        assert_eq!(pl["sales"], d(-6667));
        assert_eq!(pl["svc"], d(-3333));
        assert_eq!(pl["rent"], d(2000), "mj-1 is not on cash basis");
        assert!(cash.postings.iter().all(|p| p.account_id != "ar"));
        let bs = balance_sheet(&cash.postings, &cash.chart, "2025-12-31", "2025-01-01");
        assert_eq!(bs["bank"], d(10000));
        assert_eq!(bs[CURRENT_YEAR_EARNINGS], d(-8000));
        assert_eq!(bs.values().copied().sum::<Decimal>(), ZERO);
        let accrual = Ledger::from_documents(&books()).unwrap();
        let pl = profit_and_loss(
            &accrual.postings,
            &accrual.chart,
            "2025-01-01",
            "2025-12-31",
        );
        assert_eq!(
            (pl["sales"], pl["svc"], pl["rent"]),
            (d(-20000), d(-10000), d(7000))
        );
        // opening from a prior balance sheet: its current year earnings close into retained earnings
        let mut opening = Balances::new();
        opening.insert("bank".into(), d(500));
        opening.insert(CURRENT_YEAR_EARNINGS.into(), d(-500));
        let bs = balance_sheet_from(
            &cash.postings,
            &cash.chart,
            "2025-12-31",
            "2025-01-01",
            &opening,
            "2024-12-31",
        );
        assert_eq!(
            (bs["bank"], bs["re"], bs[CURRENT_YEAR_EARNINGS]),
            (d(10500), d(-500), d(-8000))
        );
    }

    #[test]
    fn reads_sections_in_their_natural_sign() {
        let pl = json!({"Reports": [{"ReportID": "ProfitAndLoss", "Rows": [
            {"RowType": "Header", "Cells": []},
            {"RowType": "Section", "Title": "Revenue", "Rows": [row(Some("r"), "Sales", "100.00")]},
            {"RowType": "Section", "Title": "Less Operating Expenses", "Rows": [row(Some("e"), "Rent", "40.00")]},
            {"RowType": "Section", "Title": "Other Income and Expense", "Rows": [row(Some("o"), "Travel", "-5.00")]},
        ]}]});
        let st = xero_statement(&pl).unwrap();
        assert_eq!(st.balances["r"], Decimal::new(-10000, 2));
        assert_eq!(st.balances["e"], Decimal::new(4000, 2));
        assert_eq!(st.balances["o"], Decimal::new(500, 2));
        let bs = json!({"Reports": [{"ReportID": "BalanceSheet", "Rows": [
            {"RowType": "Section", "Title": "Assets", "Rows": []},
            {"RowType": "Section", "Title": "Bank", "Rows": [row(Some("b"), "Checking", "10.00")]},
            {"RowType": "Section", "Title": "Liabilities and Equity", "Rows": []},
            {"RowType": "Section", "Title": "Current Liabilities", "Rows": [row(Some("l"), "Card", "4.00")]},
            {"RowType": "Section", "Title": "Equity", "Rows": [row(None, "Current Year Earnings", "6.00")]},
        ]}]});
        let st = xero_statement(&bs).unwrap();
        assert_eq!(st.balances["b"], Decimal::new(1000, 2));
        assert_eq!(st.balances["l"], Decimal::new(-400, 2));
        assert_eq!(st.balances[CURRENT_YEAR_EARNINGS], Decimal::new(-600, 2));
    }
}
