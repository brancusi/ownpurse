//! `history`: a document's history and notes from Xero (`GET {Entity}/{id}/History`) into the record, as a
//! version of the pseudo-entity `History` keyed `<Entity>/<id>` (payload: Xero's `HistoryRecords`, oldest
//! first as Xero lists them). Appended only when the content changed, like a sync; the TUI's document view
//! reads the same row.

use crate::ApiError;
use crate::http::XeroHttp;
use crate::store::Writer;
use serde_json::Value;

/// The entities Xero keeps a history for (the History endpoint exists on these).
pub const ENTITIES: [&str; 14] = [
    "BankTransactions",
    "BankTransfers",
    "BatchPayments",
    "Contacts",
    "CreditNotes",
    "Invoices",
    "Items",
    "ManualJournals",
    "Overpayments",
    "Payments",
    "Prepayments",
    "PurchaseOrders",
    "Quotes",
    "RepeatingInvoices",
];

/// The entity's canonical name (case-insensitive), if it has a history endpoint.
pub fn entity(name: &str) -> Option<&'static str> {
    ENTITIES
        .iter()
        .copied()
        .find(|e| e.eq_ignore_ascii_case(name))
}

/// The record key of a document's history (`BankTransactions/<id>`).
pub fn key(entity: &str, id: &str) -> String {
    format!("{entity}/{id}")
}

/// What `fetch` found.
#[derive(Clone, Debug, PartialEq)]
pub struct Fetched {
    pub records: Vec<Value>,
    /// A new version was appended (the history changed since the last fetch, or it is the first).
    pub new_version: bool,
}

/// GET the document's history (one call) and append it to the record when it changed.
pub fn fetch(
    http: &XeroHttp<'_>,
    store: &Writer,
    tenant_id: &str,
    entity: &str,
    id: &str,
) -> Result<Fetched, ApiError> {
    let body = http.get(tenant_id, &format!("{entity}/{id}/History"), &[], None)?;
    let records: Vec<Value> = body["HistoryRecords"].as_array().cloned().ok_or_else(|| {
        ApiError::api(
            format!("Xero returned no HistoryRecords for {entity} {id}."),
            "Check the id with `ownpurse show`.",
        )
    })?;
    let new_version = record(store, tenant_id, entity, id, &records)?;
    Ok(Fetched {
        records,
        new_version,
    })
}

/// Append a history version (source `sync`) when it differs from the latest one.
pub fn record(
    store: &Writer,
    tenant_id: &str,
    entity: &str,
    id: &str,
    records: &[Value],
) -> Result<bool, ApiError> {
    let mut known = store.latest_hashes(tenant_id, "History")?;
    store.record_version(
        tenant_id,
        "History",
        &key(entity, id),
        &Value::Array(records.to_vec()),
        None,
        None,
        "sync",
        None,
        &mut known,
    )
}

/// One history line for display: (when, change, by, details).
pub fn line(r: &Value) -> (String, String, String, String) {
    let s = |k: &str| r[k].as_str().unwrap_or("").to_string();
    let when = r["DateUTCString"]
        .as_str()
        .map(str::to_string)
        .or_else(|| r["DateUTC"].as_str().and_then(crate::sync::xero_datetime))
        .unwrap_or_default();
    // Xero keeps non-ASCII as HTML entities: shown decoded, whatever wrote it
    let d = |k: &str| ownpurse_core::text::html_unescape(&s(k));
    (when, d("Changes"), d("User"), d("Details"))
}
