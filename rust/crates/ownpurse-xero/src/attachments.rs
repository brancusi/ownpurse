//! `attachments`: a document's attachments from Xero (`GET {Entity}/{id}/Attachments`) into the record, one
//! version per attachment of the pseudo-entity `Attachments` keyed `<Entity>/<id>/<AttachmentID>` (the
//! TUI's document view reads the same rows), and a file's bytes (`GET {Entity}/{id}/Attachments/{name}`)
//! saved where asked. Reads only.

use crate::ApiError;
use crate::http::XeroHttp;
use crate::store::Writer;
use serde_json::Value;

/// The entities Xero keeps attachments for (the endpoint exists on these).
pub const ENTITIES: [&str; 9] = [
    "BankTransactions",
    "BankTransfers",
    "Contacts",
    "CreditNotes",
    "Invoices",
    "ManualJournals",
    "Receipts",
    "RepeatingInvoices",
    "PurchaseOrders",
];

/// The entity's canonical name (case-insensitive), if it has attachments.
pub fn entity(name: &str) -> Option<&'static str> {
    ENTITIES
        .iter()
        .copied()
        .find(|e| e.eq_ignore_ascii_case(name))
}

/// The record key of one attachment.
pub fn key(entity: &str, id: &str, attachment_id: &str) -> String {
    format!("{entity}/{id}/{attachment_id}")
}

/// GET the list (one call); each attachment is appended to the record when it is new or changed. Returns
/// the list as Xero gave it and how many versions were added.
pub fn list(
    http: &XeroHttp<'_>,
    store: &Writer,
    tenant_id: &str,
    entity: &str,
    id: &str,
) -> Result<(Vec<Value>, usize), ApiError> {
    let body = http.get(tenant_id, &format!("{entity}/{id}/Attachments"), &[], None)?;
    let items = body["Attachments"].as_array().cloned().ok_or_else(|| {
        ApiError::api(
            format!("Xero returned no Attachments list for {entity} {id}."),
            "Check the id with `ownpurse show`.",
        )
    })?;
    let mut known = store.latest_hashes(tenant_id, "Attachments")?;
    let mut added = 0;
    for a in &items {
        let aid = a["AttachmentID"]
            .as_str()
            .or(a["FileName"].as_str())
            .unwrap_or("");
        if store.record_version(
            tenant_id,
            "Attachments",
            &key(entity, id, aid),
            a,
            None,
            None,
            "sync",
            None,
            &mut known,
        )? {
            added += 1;
        }
    }
    Ok((items, added))
}

/// The bytes of one attachment, by its file name (one call).
pub fn download(
    http: &XeroHttp<'_>,
    tenant_id: &str,
    entity: &str,
    id: &str,
    file_name: &str,
) -> Result<Vec<u8>, ApiError> {
    let name: String = file_name
        .bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect();
    http.get_bytes(tenant_id, &format!("{entity}/{id}/Attachments/{name}"))
}
