//! Ledger invariants over generated documents (bank transactions, transfers, journals, invoices, credit notes,
//! payments; balanced and deliberately unbalanced; every tax mode):
//!
//! 1. every document's postings sum to zero, or the document is listed unbalanced with exactly that imbalance;
//! 2. a balanced ledger's trial balance has debits == credits (at any date, any fiscal-year start);
//! 3. `opening_for(since)` + postings in [since, as_of] == the closing balance, for balance-sheet accounts, on
//!    both the full-history and the Xero-opening basis.

use ownpurse_core::dates::{add_days, civil_from_days, days_from_civil, iso};
use ownpurse_core::{Basis, BasisKind, Documents, Ledger, trial_balance};
use proptest::prelude::*;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::collections::BTreeMap;

const CODES: [&str; 6] = ["200", "400", "2410", "610", "800", "999"]; // 999 is not in the chart (uncoded)

fn accounts() -> Vec<Value> {
    let a = |id: &str, code: &str, typ: &str, cls: &str, sys: &str| {
        json!({"AccountID": id, "Code": code, "Name": format!("{id} name"), "Type": typ, "Class": cls,
               "SystemAccount": sys, "Status": "ACTIVE"})
    };
    vec![
        a("bank1", "", "BANK", "ASSET", ""),
        a("bank2", "", "BANK", "ASSET", ""),
        a("sales", "200", "REVENUE", "REVENUE", ""),
        a("exp", "400", "EXPENSE", "EXPENSE", ""),
        a("loan", "2410", "CURRLIAB", "LIABILITY", ""),
        a("ar", "610", "CURRENT", "ASSET", "DEBTORS"),
        a("ap", "800", "CURRLIAB", "LIABILITY", "CREDITORS"),
        a("gst", "820", "CURRLIAB", "LIABILITY", "GST"),
        a("re", "960", "EQUITY", "EQUITY", "RETAINEDEARNINGS"),
    ]
}

fn cents(c: i64) -> Decimal {
    Decimal::new(c, 2)
}

fn date_of(day: i64) -> String {
    let (y, m, d) = civil_from_days(days_from_civil(2023, 1, 1) + day);
    iso(y, m, d)
}

#[derive(Debug, Clone)]
struct Line {
    code: usize,
    amount: i64,
    tax: i64,
}

#[derive(Debug, Clone)]
enum Doc {
    Bank {
        spend: bool,
        lines: Vec<Line>,
        mode: usize,
        skew: i64,
        day: i64,
        status: bool,
    },
    Transfer {
        amount: i64,
        day: i64,
    },
    Journal {
        lines: Vec<Line>,
        balance: bool,
        day: i64,
    },
    Invoice {
        credit: bool,
        sales: bool,
        lines: Vec<Line>,
        mode: usize,
        skew: i64,
        day: i64,
    },
    Payment {
        kind: usize,
        amount: i64,
        day: i64,
    },
}

fn line() -> impl Strategy<Value = Line> {
    (
        0..CODES.len(),
        -500_000i64..500_000,
        prop_oneof![Just(0i64), -5_000i64..5_000],
    )
        .prop_map(|(code, amount, tax)| Line { code, amount, tax })
}

fn doc() -> impl Strategy<Value = Doc> {
    let day = 0i64..(3 * 365);
    let skew = prop_oneof![4 => Just(0i64), 1 => -1_000i64..1_000];
    prop_oneof![
        (
            any::<bool>(),
            prop::collection::vec(line(), 1..4),
            0..3usize,
            skew.clone(),
            day.clone(),
            prop::bool::weighted(0.9)
        )
            .prop_map(|(spend, lines, mode, skew, day, status)| Doc::Bank {
                spend,
                lines,
                mode,
                skew,
                day,
                status
            }),
        (1i64..1_000_000, day.clone()).prop_map(|(amount, day)| Doc::Transfer { amount, day }),
        (
            prop::collection::vec(line(), 1..4),
            prop::bool::weighted(0.8),
            day.clone()
        )
            .prop_map(|(lines, balance, day)| Doc::Journal {
                lines,
                balance,
                day
            }),
        (
            any::<bool>(),
            any::<bool>(),
            prop::collection::vec(line(), 1..4),
            0..3usize,
            skew,
            day.clone()
        )
            .prop_map(|(credit, sales, lines, mode, skew, day)| Doc::Invoice {
                credit,
                sales,
                lines,
                mode,
                skew,
                day
            }),
        (0..5usize, 1i64..1_000_000, day).prop_map(|(kind, amount, day)| Doc::Payment {
            kind,
            amount,
            day
        }),
    ]
}

const MODES: [&str; 3] = ["NoTax", "Exclusive", "Inclusive"];

/// Line items and the total that balances them under the tax mode.
fn items(lines: &[Line], mode: usize) -> (Vec<Value>, i64) {
    let mut total = 0;
    let v = lines
        .iter()
        .map(|l| {
            total += if MODES[mode] == "Inclusive" {
                l.amount
            } else {
                l.amount + l.tax
            };
            json!({"AccountCode": CODES[l.code], "LineAmount": cents(l.amount).to_string(),
                   "TaxAmount": cents(l.tax).to_string(), "Description": "line"})
        })
        .collect();
    (v, total)
}

fn documents(docs: &[Doc]) -> Documents {
    let mut d = Documents {
        accounts: accounts(),
        ..Default::default()
    };
    for (i, doc) in docs.iter().enumerate() {
        let ds = |day: &i64| format!("{}T00:00:00", date_of(*day));
        match doc {
            Doc::Bank {
                spend,
                lines,
                mode,
                skew,
                day,
                status,
            } => {
                let (li, total) = items(lines, *mode);
                d.bank_transactions.push(json!({
                    "BankTransactionID": format!("bt-{i}"), "Type": if *spend { "SPEND" } else { "RECEIVE" },
                    "Status": if *status { "AUTHORISED" } else { "DELETED" }, "LineAmountTypes": MODES[*mode],
                    "DateString": ds(day), "BankAccount": {"AccountID": "bank1"}, "Contact": {"Name": "Someone"},
                    "Total": cents(total + skew).to_string(), "LineItems": li}));
            }
            Doc::Transfer { amount, day } => d.bank_transfers.push(json!({
                "BankTransferID": format!("tr-{i}"), "Amount": cents(*amount).to_string(), "DateString": ds(day),
                "FromBankAccount": {"AccountID": "bank1"}, "ToBankAccount": {"AccountID": "bank2"}})),
            Doc::Journal {
                lines,
                balance,
                day,
            } => {
                let mut jl: Vec<Value> = lines
                    .iter()
                    .map(|l| {
                        json!({"AccountCode": CODES[l.code], "LineAmount": cents(l.amount).to_string(),
                           "TaxAmount": "0"})
                    })
                    .collect();
                if *balance {
                    let s: i64 = lines.iter().map(|l| l.amount).sum();
                    jl.push(
                        json!({"AccountCode": "2410", "LineAmount": cents(-s).to_string(), "TaxAmount": "0"}),
                    );
                }
                d.manual_journals.push(json!({"ManualJournalID": format!("mj-{i}"), "Status": "POSTED",
                    "DateString": ds(day), "Narration": "n", "JournalLines": jl}));
            }
            Doc::Invoice {
                credit,
                sales,
                lines,
                mode,
                skew,
                day,
            } => {
                let (li, total) = items(lines, *mode);
                let t = match (credit, sales) {
                    (false, true) => "ACCREC",
                    (false, false) => "ACCPAY",
                    (true, true) => "ACCRECCREDIT",
                    (true, false) => "ACCPAYCREDIT",
                };
                let mut v = json!({"Type": t, "Status": "AUTHORISED", "LineAmountTypes": MODES[*mode],
                    "DateString": ds(day), "Contact": {"Name": "Customer"},
                    "Total": cents(total + skew).to_string(), "LineItems": li});
                if *credit {
                    v["CreditNoteID"] = format!("cn-{i}").into();
                    d.credit_notes.push(v);
                } else {
                    v["InvoiceID"] = format!("inv-{i}").into();
                    d.invoices.push(v);
                }
            }
            Doc::Payment { kind, amount, day } => {
                let pt = [
                    "ACCRECPAYMENT",
                    "ACCPAYPAYMENT",
                    "ARCREDITPAYMENT",
                    "APCREDITPAYMENT",
                    "APPREPAYMENTPAYMENT",
                ][*kind];
                d.payments.push(json!({"PaymentID": format!("pay-{i}"), "Status": "AUTHORISED",
                    "PaymentType": pt, "Amount": cents(*amount).to_string(), "DateString": ds(day),
                    "Account": {"AccountID": "bank1"}}));
            }
        }
    }
    d
}

fn sum_by_doc(l: &Ledger) -> BTreeMap<(&'static str, String), Decimal> {
    let mut m = BTreeMap::new();
    for p in &l.postings {
        *m.entry((p.source, p.source_id.clone()))
            .or_insert(Decimal::ZERO) += p.amount;
    }
    m
}

fn balanced(docs: &[Doc]) -> Vec<Doc> {
    docs.iter()
        .cloned()
        .map(|d| match d {
            Doc::Bank {
                spend,
                lines,
                mode,
                day,
                status,
                ..
            } => Doc::Bank {
                spend,
                lines,
                mode,
                skew: 0,
                day,
                status,
            },
            Doc::Journal { lines, day, .. } => Doc::Journal {
                lines,
                balance: true,
                day,
            },
            Doc::Invoice {
                credit,
                sales,
                lines,
                mode,
                day,
                ..
            } => Doc::Invoice {
                credit,
                sales,
                lines,
                mode,
                skew: 0,
                day,
            },
            other => other,
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn documents_sum_to_zero_or_are_listed_unbalanced(docs in prop::collection::vec(doc(), 0..40)) {
        let l = Ledger::from_documents(&documents(&docs)).unwrap();
        let sums = sum_by_doc(&l);
        let unbalanced: BTreeMap<(&'static str, String), Decimal> =
            l.unbalanced.iter().map(|u| ((u.source, u.id.clone()), u.imbalance)).collect();
        prop_assert_eq!(unbalanced.len(), l.unbalanced.len(), "a document listed twice");
        for (k, s) in &sums {
            match unbalanced.get(k) {
                Some(imb) => prop_assert_eq!(s, imb, "{:?}: imbalance", k),
                None => prop_assert!(s.is_zero(), "{:?} sums to {} but is not listed unbalanced", k, s),
            }
        }
        for (k, imb) in &unbalanced {
            prop_assert!(!imb.is_zero(), "{:?} listed with zero imbalance", k);
            prop_assert_eq!(sums.get(k).copied().unwrap_or(Decimal::ZERO), *imb, "{:?}", k);
        }
        prop_assert!(l.postings.iter().all(|p| !p.amount.is_zero()), "zero postings are dropped");
    }

    #[test]
    fn balanced_ledger_trial_balance_debits_equal_credits(
        docs in prop::collection::vec(doc(), 0..40), as_of in 0i64..(3 * 365), fy in 0i64..(3 * 365),
    ) {
        let l = Ledger::from_documents(&documents(&balanced(&docs))).unwrap();
        prop_assert!(l.unbalanced.is_empty(), "{:?}", l.unbalanced);
        let tb = trial_balance(&l.postings, &l.chart, &date_of(as_of), &date_of(fy.min(as_of)), None, None);
        let dr: Decimal = tb.values().filter(|v| v.is_sign_positive()).sum();
        let cr: Decimal = tb.values().filter(|v| v.is_sign_negative()).map(|v| -v).sum();
        prop_assert_eq!(dr, cr);
        prop_assert!(tb.values().all(|v| !v.is_zero()), "zero balances are dropped");
    }

    #[test]
    fn opening_plus_period_postings_equals_closing(
        docs in prop::collection::vec(doc(), 0..40), since_off in 0i64..365, xero_opening in any::<bool>(),
    ) {
        let l = Ledger::from_documents(&documents(&docs)).unwrap();
        let as_of = "2025-12-31".to_string();
        let fy_start = "2025-01-01".to_string();
        let since = date_of(2 * 365 + since_off);   // a day inside 2025
        let prior = add_days(&fy_start, -1).unwrap();
        let (kind, opening, opening_date) = if xero_opening {
            let op = trial_balance(&l.postings, &l.chart, &prior, "2024-01-01", None, None);
            (BasisKind::XeroOpening, Some(op), Some(prior))
        } else {
            (BasisKind::History, None, None)
        };
        let balances = trial_balance(&l.postings, &l.chart, &as_of, &fy_start, opening.as_ref(),
                                     opening_date.as_deref());
        let b = Basis { kind, as_of: as_of.clone(), fy_start, opening, opening_date, xero_at_as_of: None,
                        xero_fetched: None, balances, diffs: vec![] };
        let mut ids: Vec<&str> = l.postings.iter().map(|p| p.account_id.as_str()).collect();
        ids.sort();
        ids.dedup();
        for aid in ids.into_iter().filter(|a| !l.chart.is_pl(a)) {
            let period: Decimal = l.postings.iter()
                .filter(|p| p.account_id == aid && p.date >= since && p.date <= as_of)
                .map(|p| p.amount).sum();
            let closing = b.balances.get(aid).copied().unwrap_or(Decimal::ZERO);
            prop_assert_eq!(b.opening_for(&l, aid, &since) + period, closing, "account {} since {}", aid, since);
        }
    }
}

#[test]
fn skewed_documents_are_listed_unbalanced() {
    let line = |code, amount, tax| Line { code, amount, tax };
    let docs = [
        Doc::Bank {
            spend: true,
            lines: vec![line(1, 10_000, 500)],
            mode: 1,
            skew: 7,
            day: 10,
            status: true,
        },
        Doc::Journal {
            lines: vec![line(0, 2_500, 0)],
            balance: false,
            day: 11,
        },
        Doc::Invoice {
            credit: false,
            sales: true,
            lines: vec![line(0, 9_999, 0)],
            mode: 0,
            skew: 0,
            day: 12,
        },
    ];
    let l = Ledger::from_documents(&documents(&docs)).unwrap();
    let got: Vec<(&str, String, Decimal)> = l
        .unbalanced
        .iter()
        .map(|u| (u.source, u.id.clone(), u.imbalance))
        .collect();
    assert_eq!(
        got,
        [
            ("BankTransactions", "bt-0".to_string(), cents(-7)),
            ("ManualJournals", "mj-1".to_string(), cents(2_500)),
        ]
    );
}
