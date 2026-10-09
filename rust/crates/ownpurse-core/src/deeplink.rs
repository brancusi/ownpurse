//! "Open in Xero": a document's page in the Xero web app, through the org switch so it opens in the right
//! organisation: `https://go.xero.com/organisationlogin/default.aspx?shortcode=<ShortCode>&redirecturl=<path>`.
//! The ShortCode comes from the synced Organisation record; without it there is no link (never a broken
//! one). The paths are DATA ([`PATHS`]) so they can be corrected after live checks: none is verified yet
//! against the Demo Company. Local only: building a link never calls Xero.

use crate::Record;
use serde_json::Value;

/// The org switch every link goes through.
pub const ORG_LOGIN: &str = "https://go.xero.com/organisationlogin/default.aspx";

/// `(entity, kind, path template)`: `kind` narrows an entity by its `Type` (`ACCREC` / `ACCPAY` for invoices),
/// empty for any; `{id}` is the document's id. UNVERIFIED until checked on the Demo Company.
pub const PATHS: &[(&str, &str, &str)] = &[
    (
        "BankTransactions",
        "",
        "/Bank/ViewTransaction.aspx?bankTransactionID={id}",
    ),
    ("ManualJournals", "", "/Journal/View.aspx?invoiceID={id}"),
    (
        "Invoices",
        "ACCREC",
        "/AccountsReceivable/View.aspx?InvoiceID={id}",
    ),
    (
        "Invoices",
        "ACCPAY",
        "/AccountsPayable/View.aspx?InvoiceID={id}",
    ),
    // a bank account's reconcile queue: `{id}` is the AccountID
    ("BankRec", "", "/BankRec/BankRec.aspx?accountId={id}"),
];

/// Percent-encoding for a query value (RFC 3986 unreserved characters kept).
pub fn urlencode(s: &str) -> String {
    s.bytes()
        .map(|b| match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (b as char).to_string()
            }
            _ => format!("%{b:02X}"),
        })
        .collect()
}

/// The link for a short code, an entity (with its `Type` when it matters) and an id; None when the entity
/// has no page in [`PATHS`], the short code is missing, or the id is empty.
pub fn url_for(short_code: &str, entity: &str, kind: &str, id: &str) -> Option<String> {
    if short_code.trim().is_empty() || id.trim().is_empty() {
        return None;
    }
    let path = PATHS
        .iter()
        .find(|(e, k, _)| *e == entity && (k.is_empty() || *k == kind))?
        .2
        .replace("{id}", &urlencode(id));
    Some(format!(
        "{ORG_LOGIN}?shortcode={}&redirecturl={}",
        urlencode(short_code),
        urlencode(&path)
    ))
}

/// The org's ShortCode from the record's Organisation document.
pub fn short_code(record: &Record, tenant: &str) -> Option<String> {
    record
        .organisation(tenant)
        .ok()?
        .get("ShortCode")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// `xero_url(record, tenant, entity, id)`: the document's page (its `Type` read from the record for invoices).
pub fn xero_url(record: &Record, tenant: &str, entity: &str, id: &str) -> Option<String> {
    let sc = short_code(record, tenant)?;
    let kind = if entity == "Invoices" {
        record
            .current_rows(tenant, entity)
            .ok()?
            .into_iter()
            .find(|r| r.xero_id == id)?
            .payload
            .get("Type")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    } else {
        String::new()
    };
    url_for(&sc, entity, &kind, id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn links_go_through_the_org_switch_with_the_path_encoded() {
        let u = url_for(
            "!ab1C",
            "BankTransactions",
            "",
            "0000000a-0000-4000-8000-000000000001",
        )
        .unwrap();
        assert_eq!(
            u,
            "https://go.xero.com/organisationlogin/default.aspx?shortcode=%21ab1C&redirecturl=%2FBank%2FViewTransaction.aspx%3FbankTransactionID%3D0000000a-0000-4000-8000-000000000001"
        );
        assert!(
            url_for("x", "Invoices", "ACCPAY", "i-1")
                .unwrap()
                .contains("AccountsPayable")
        );
        assert!(
            url_for("x", "Invoices", "ACCREC", "i-1")
                .unwrap()
                .contains("AccountsReceivable")
        );
        assert!(
            url_for("x", "Invoices", "", "i-1").is_none(),
            "an invoice without its type: no guess"
        );
        assert_eq!(urlencode("a b&c/d"), "a%20b%26c%2Fd");
    }

    #[test]
    fn no_short_code_or_unknown_entity_means_no_link() {
        assert!(url_for("", "ManualJournals", "", "m-1").is_none());
        assert!(url_for("x", "Contacts", "", "c-1").is_none());
        assert!(url_for("x", "ManualJournals", "", "").is_none());
    }
}
