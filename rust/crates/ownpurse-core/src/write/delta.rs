//! Expected report movements of one operation, from the SAME ledger rules every read uses: build the postings
//! of the target document before and after (`Ledger::from_documents` with the org's chart) and subtract. A
//! create has no "before"; a void's "after" is the VOIDED document, which posts nothing. The result is per
//! account; every account not listed must not move.

use super::{ExpectedDelta, Section};
use crate::ledger::{Documents, Ledger};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::BTreeMap;

/// Which document list a payload belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocKind {
    ManualJournal,
    BankTransaction,
    BankTransfer,
}

fn postings(accounts: &[Value], kind: DocKind, doc: Option<&Value>) -> Result<Ledger, String> {
    let mut d = Documents {
        accounts: accounts.to_vec(),
        ..Default::default()
    };
    if let Some(doc) = doc {
        match kind {
            DocKind::ManualJournal => d.manual_journals.push(doc.clone()),
            DocKind::BankTransaction => d.bank_transactions.push(doc.clone()),
            DocKind::BankTransfer => d.bank_transfers.push(doc.clone()),
        }
    }
    Ledger::from_documents(&d).map_err(|e| e.to_string())
}

/// after − before, per account, zero rows dropped, ordered by code then id. Refuses a document that doesn't
/// balance (never planned) and one that posts to an account missing from the chart.
pub fn moves(
    accounts: &[Value],
    kind: DocKind,
    before: Option<&Value>,
    after: Option<&Value>,
) -> Result<Vec<ExpectedDelta>, String> {
    let b = postings(accounts, kind, before)?;
    let a = postings(accounts, kind, after)?;
    if let Some(u) = a.unbalanced.first() {
        return Err(format!(
            "the document does not balance (off by {})",
            u.imbalance
        ));
    }
    let mut by: BTreeMap<String, Decimal> = BTreeMap::new();
    for p in &a.postings {
        *by.entry(p.account_id.clone()).or_default() += p.amount;
    }
    for p in &b.postings {
        *by.entry(p.account_id.clone()).or_default() -= p.amount;
    }
    let chart = &a.chart;
    let mut out = vec![];
    for (id, amount) in by {
        if amount.is_zero() {
            continue;
        }
        if !chart.by_id.contains_key(&id) {
            return Err(format!("it posts to an account not in the chart ({id})"));
        }
        let acct = chart.get(&id);
        out.push(ExpectedDelta {
            section: if chart.is_pl(&id) {
                Section::Pl
            } else {
                Section::Bs
            },
            code: acct.code.clone(),
            name: acct.name.clone(),
            account_id: id,
            amount,
        });
    }
    out.sort_by(|x, y| (&x.code, &x.account_id).cmp(&(&y.code, &y.account_id)));
    Ok(out)
}

/// The net movement of each section: `(bs, pl)`, debit positive. A balanced operation has `bs + pl == 0`;
/// −pl is the profit effect.
pub fn sections(moves: &[ExpectedDelta]) -> (Decimal, Decimal) {
    let mut bs = Decimal::ZERO;
    let mut pl = Decimal::ZERO;
    for m in moves {
        match m.section {
            Section::Pl => pl += m.amount,
            Section::Bs => bs += m.amount,
        }
    }
    (bs, pl)
}
