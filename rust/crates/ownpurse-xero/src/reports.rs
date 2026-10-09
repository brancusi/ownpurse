//! `report fetch`: Xero's reports (TrialBalance, BalanceSheet, ProfitAndLoss, BankSummary) into the record's
//! `reports` table: the sync-cached kind (an observation of what Xero reported, appended with its canonical
//! params), not write evidence (those carry `xa_package` / `phase`). `parity` and `tb --since` read the latest
//! row for exactly their params, so a TrialBalance fetched here is what they compare against.

use crate::ApiError;
use crate::http::XeroHttp;
use crate::store::{Writer, now_iso, sha256_hex};
use rusqlite::OptionalExtension;
use rusqlite::types::Value as Sql;
use serde_json::{Map, Value, json};

/// The reports `report fetch` knows.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    TrialBalance,
    BalanceSheet,
    ProfitAndLoss,
    BankSummary,
}

impl Kind {
    /// `trialbalance | tb`, `balancesheet | bs`, `pl | profitandloss`, `banksummary`.
    pub fn parse(s: &str) -> Option<Kind> {
        Some(
            match s.to_ascii_lowercase().replace(['-', '_'], "").as_str() {
                "trialbalance" | "tb" => Kind::TrialBalance,
                "balancesheet" | "bs" => Kind::BalanceSheet,
                "pl" | "profitandloss" | "profitloss" => Kind::ProfitAndLoss,
                "banksummary" | "bank" => Kind::BankSummary,
                _ => return None,
            },
        )
    }

    /// Xero's report name (the `reports.report` column and the URL path).
    pub fn name(self) -> &'static str {
        match self {
            Kind::TrialBalance => "TrialBalance",
            Kind::BalanceSheet => "BalanceSheet",
            Kind::ProfitAndLoss => "ProfitAndLoss",
            Kind::BankSummary => "BankSummary",
        }
    }

    /// A report at one date (TB, BS) or over a period (P&L, bank summary).
    pub fn is_period(self) -> bool {
        matches!(self, Kind::ProfitAndLoss | Kind::BankSummary)
    }

    /// Whether Xero's `paymentsOnly` (cash basis) applies.
    pub fn has_basis(self) -> bool {
        !matches!(self, Kind::BankSummary)
    }
}

/// What to ask for: `date` for a point report, `from`/`to` for a period one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub kind: Kind,
    pub date: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    /// Cash basis (`paymentsOnly=true`).
    pub cash: bool,
}

impl Request {
    /// The query parameters, as Xero names them; the same map (string values) is the record's `params`.
    /// A TrialBalance at a date gives exactly `{"date":…,"paymentsOnly":"false"}`, the params `parity` reads.
    pub fn params(&self) -> Result<Vec<(String, Value)>, ApiError> {
        let mut p: Vec<(String, Value)> = vec![];
        let need = |v: &Option<String>, flag: &str| {
            v.clone().ok_or_else(|| {
                ApiError::usage(
                    format!("{} needs {flag}.", self.kind.name()),
                    "Give the date(s) in YYYY-MM-DD.",
                )
            })
        };
        if self.kind.is_period() {
            p.push(("fromDate".into(), json!(need(&self.from, "--from")?)));
            p.push(("toDate".into(), json!(need(&self.to, "--to")?)));
        } else {
            p.push(("date".into(), json!(need(&self.date, "--date")?)));
        }
        if self.kind.has_basis() {
            p.push((
                "paymentsOnly".into(),
                json!(if self.cash { "true" } else { "false" }),
            ));
        } else if self.cash {
            return Err(ApiError::usage(
                format!("{} has no cash basis.", self.kind.name()),
                "Drop --cash.",
            ));
        }
        for (_, v) in &p {
            let s = v.as_str().unwrap_or("");
            if s != "true" && s != "false" && ownpurse_core::dates::parse_iso(s).is_none() {
                return Err(ApiError::usage(
                    format!("Not a date: {s}."),
                    "Use YYYY-MM-DD.",
                ));
            }
        }
        Ok(p)
    }

    /// The record's canonical params (sorted keys, compact): the key `report()` looks rows up by.
    pub fn params_json(&self) -> Result<String, ApiError> {
        let m: Map<String, Value> = self.params()?.into_iter().collect();
        Ok(ownpurse_core::canon::canonical(&Value::Object(m)))
    }
}

/// One fetched report, as appended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fetched {
    pub tenant_id: String,
    pub report: &'static str,
    pub params: String,
    pub fetched_at: String,
    pub content_hash: String,
    /// The same content as the previous row for these params (Xero's numbers didn't move).
    pub unchanged: bool,
    /// Whether this row came in: a first fetch for these params, or a later one.
    pub first: bool,
    /// The report's rows (sections count as their rows).
    pub rows: usize,
}

/// GET the report and append it to `reports` (one call).
pub fn fetch(
    http: &XeroHttp<'_>,
    store: &Writer,
    tenant_id: &str,
    req: &Request,
) -> Result<Fetched, ApiError> {
    let params = req.params()?;
    let key = req.params_json()?;
    let body = http.get(
        tenant_id,
        &format!("Reports/{}", req.kind.name()),
        &params,
        None,
    )?;
    if body["Reports"].as_array().is_none_or(Vec::is_empty) {
        return Err(ApiError::api(
            format!("Xero returned no {} report.", req.kind.name()),
            "Retry the command; check the org's dates.",
        ));
    }
    let payload = ownpurse_core::canon::canonical(&body);
    let hash = sha256_hex(&payload);
    let prev: Option<String> = store
        .conn()
        .query_row(
            "SELECT content_hash FROM reports WHERE tenant_id = ?1 AND report = ?2 AND params = ?3 \
             ORDER BY id DESC LIMIT 1",
            (tenant_id, req.kind.name(), &key),
            |r| r.get(0),
        )
        .optional()
        .map_err(|e| ApiError::new("integrity", format!("record: {e}"), ""))?;
    let at = now_iso();
    store.append(
        "reports",
        &[
            ("tenant_id", Sql::Text(tenant_id.into())),
            ("report", Sql::Text(req.kind.name().into())),
            ("params", Sql::Text(key.clone())),
            ("fetched_at", Sql::Text(at.clone())),
            ("content_hash", Sql::Text(hash.clone())),
            ("payload", Sql::Text(payload)),
        ],
    )?;
    Ok(Fetched {
        tenant_id: tenant_id.into(),
        report: req.kind.name(),
        params: key,
        fetched_at: at,
        unchanged: prev.as_deref() == Some(hash.as_str()),
        first: prev.is_none(),
        content_hash: hash,
        rows: count_rows(&body["Reports"][0]["Rows"]),
    })
}

fn count_rows(v: &Value) -> usize {
    v.as_array()
        .into_iter()
        .flatten()
        .map(|r| match r["RowType"].as_str() {
            Some("Section") => count_rows(&r["Rows"]),
            Some("Header") => 0,
            _ => 1,
        })
        .sum()
}
