//! A synthetic large record for performance work (benches, `xtask demo big`): the demo world's record with
//! thousands of invented documents appended. Invented names and irregular cents only; never real data. The
//! appended versions bypass the record's integrity chain (they are not for `verify`), which no reader checks.

use rusqlite::params;
use serde_json::json;
use std::path::Path;

const PAYEES: [&str; 12] = [
    "Hollowpine Packaging Works",
    "Cordillera Green Imports",
    "Emberline Thermal Services",
    "Brightwater Fuel Co-op",
    "Ridgeback Lab Glassware",
    "Larkspur Grocers Cooperative",
    "Corvid Registry Services",
    "Vellum & Quoin Type Foundry",
    "Wren & Ashby Cafés",
    "Thistlewood Audit Partners",
    "Brackenridge Lane Holdings",
    "Pemberton Ferry Authority",
];
const WORDS: [&str; 10] = [
    "bag proofs for the Harvest Blend",
    "washed lot, 22 sacks",
    "afterburner catalyst service",
    "van fuel",
    "cupping bowls",
    "wholesale beans",
    "annual return filing",
    "wayfinding typeface licence",
    "term loan instalment",
    "management fee",
];
const ACCOUNTS: [&str; 8] = [
    "5310", "5020", "6140", "6520", "4010", "1630", "2410", "6050",
];

/// Copy `base` (a demo record) to `out` and append `docs` invented documents spread over its tenants:
/// about 70% bank transactions (1–3 lines), 20% manual journals (2 lines), 10% invoices and bills. Returns the
/// number of classifiable lines added.
pub fn big_record(base: &Path, out: &Path, docs: usize) -> rusqlite::Result<usize> {
    let _ = std::fs::remove_file(out);
    std::fs::copy(base, out).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(e.to_string()),
        )
    })?;
    crate::record::sqlite_ready();
    let mut c = rusqlite::Connection::open(out)?;
    c.execute_batch("PRAGMA journal_mode=DELETE;")?;
    let tenants: Vec<String> = {
        let mut st = c.prepare(
            "SELECT DISTINCT tenant_id FROM versions WHERE entity='Accounts' ORDER BY tenant_id",
        )?;
        st.query_map([], |r| r.get(0))?
            .collect::<rusqlite::Result<_>>()?
    };
    let tx = c.transaction()?;
    let mut lines = 0usize;
    {
        let mut ins = tx.prepare(
            "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
             VALUES (?1, ?2, ?3, 1, 'synthetic', 'AUTHORISED', '2026-10-05T07:46:00Z', 'sync', ?4)",
        )?;
        for i in 0..docs {
            let tid = &tenants[i % tenants.len()];
            // irregular cents: never .00 or .50
            let cents = 1_013 + (i as i64 * 7_919) % 1_900_000;
            let cents = if cents % 50 == 0 { cents + 7 } else { cents };
            let amt = |k: i64| {
                let c = cents + k * 113;
                let c = if c % 50 == 0 { c + 3 } else { c };
                format!("{}.{:02}", c / 100, c % 100)
            };
            let date = format!("2025-{:02}-{:02}", 1 + i % 12, 1 + (i * 7) % 28);
            let payee = PAYEES[i % PAYEES.len()];
            let word = WORDS[i % WORDS.len()];
            let (entity, id, payload) = match i % 10 {
                0..=6 => {
                    let n = 1 + i % 3;
                    let items: Vec<_> = (0..n)
                        .map(|k| {
                            json!({"AccountCode": ACCOUNTS[(i + k) % ACCOUNTS.len()],
                                   "Description": format!("{word} #{i}.{k}"), "LineAmount": amt(k as i64), "TaxAmount": 0})
                        })
                        .collect();
                    lines += n;
                    let id = format!("syn-bt-{i}");
                    (
                        "BankTransactions",
                        id.clone(),
                        json!({"BankTransactionID": id, "Contact": {"Name": payee},
                               "DateString": format!("{date}T00:00:00"), "IsReconciled": i % 4 != 0,
                               "LineAmountTypes": "NoTax", "LineItems": items, "Status": "AUTHORISED",
                               "Type": if i % 3 == 0 { "RECEIVE" } else { "SPEND" }}),
                    )
                }
                7 | 8 => {
                    lines += 2;
                    let id = format!("syn-mj-{i}");
                    (
                        "ManualJournals",
                        id.clone(),
                        json!({"ManualJournalID": id, "Narration": format!("{word} #{i}"),
                               "DateString": format!("{date}T00:00:00"), "Status": "POSTED",
                               "JournalLines": [
                                   {"AccountCode": ACCOUNTS[i % ACCOUNTS.len()], "LineAmount": amt(0), "TaxAmount": 0},
                                   {"AccountCode": ACCOUNTS[(i + 3) % ACCOUNTS.len()], "LineAmount": format!("-{}", amt(0)), "TaxAmount": 0}]}),
                    )
                }
                _ => {
                    lines += 1;
                    let id = format!("syn-inv-{i}");
                    (
                        "Invoices",
                        id.clone(),
                        json!({"InvoiceID": id, "Type": if i % 20 == 9 { "ACCPAY" } else { "ACCREC" },
                               "Contact": {"Name": payee}, "DateString": format!("{date}T00:00:00"),
                               "Status": "AUTHORISED",
                               "LineItems": [{"AccountCode": ACCOUNTS[i % ACCOUNTS.len()],
                                              "Description": format!("{word} #{i}"), "LineAmount": amt(0)}]}),
                    )
                }
            };
            ins.execute(params![tid, entity, id, payload.to_string()])?;
        }
    }
    tx.commit()?;
    Ok(lines)
}
