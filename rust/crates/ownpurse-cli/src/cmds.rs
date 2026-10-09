//! Document commands over the record: accounts, org, contacts, txns, journals, transfers, list, show.
//! The read commands, in offline mode.

use crate::app::{App, CliError, Res};
use crate::obj;
use crate::out::{canonical, dec, py_loaded, py_str};
use ownpurse_core::dates::xero_date;
use ownpurse_core::money::money;
use rust_decimal::{Decimal, RoundingStrategy};
use serde_json::{Map, Value};

pub type Row = Map<String, Value>;

// -- payload helpers ------------------------------------------------

/// `p.get(k)` (missing → None).
pub fn g(p: &Row, k: &str) -> Value {
    p.get(k).cloned().unwrap_or(Value::Null)
}

/// `p.get(k) or ""` as text.
pub fn gs(p: &Row, k: &str) -> String {
    match p.get(k) {
        None | Some(Value::Null) => String::new(),
        Some(Value::Bool(false)) => String::new(),
        Some(v) => py_str(v),
    }
}

/// `f"{p.get(k, '')}"`: missing → "", null → "None".
pub fn fs(p: &Row, k: &str) -> String {
    match p.get(k) {
        None => String::new(),
        Some(v) => py_str(v),
    }
}

/// `(p.get(k) or {})` as an object.
pub fn sub<'a>(p: &'a Row, k: &str) -> Option<&'a Row> {
    p.get(k).and_then(Value::as_object)
}

pub fn sub_get(p: &Row, k: &str, inner: &str) -> Value {
    sub(p, k).map(|m| g(m, inner)).unwrap_or(Value::Null)
}

/// A document's date: `DateString` when present, else the `/Date(ms)/` form.
pub fn date_of(p: &Row) -> String {
    let ds = gs(p, "DateString");
    if !ds.is_empty() {
        return ds.chars().take(10).collect();
    }
    p.get("Date")
        .and_then(Value::as_str)
        .and_then(xero_date)
        .unwrap_or_default()
}

/// An amount: exact decimal quantized to cents (banker's rounding), scale 2.
pub fn amt(v: Option<&Value>) -> Res<Decimal> {
    let d = money(v).map_err(CliError::internal)?;
    let mut q = d.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven);
    q.rescale(2);
    Ok(q)
}

pub fn cents_zero() -> Decimal {
    Decimal::new(0, 2)
}

/// Within the date range (inclusive).
pub fn in_range(d: &str, since: Option<&str>, until: Option<&str>) -> bool {
    since.is_none_or(|s| d >= s) && until.is_none_or(|u| d <= u)
}

/// Truthiness of a JSON value (null, false, 0, "" and empty containers are false).
pub fn truthy(v: &Value) -> bool {
    match v {
        Value::Null | Value::Bool(false) => false,
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
        Value::Number(n) => {
            ownpurse_core::money::parse(&n.to_string()).map_or(true, |d| !d.is_zero())
        }
        Value::Bool(true) => true,
    }
}

/// `x or None` / `x or default` for sort keys: the value as text when truthy.
fn or_text(v: &Value, dflt: &str) -> String {
    if truthy(v) {
        py_str(v)
    } else {
        dflt.to_string()
    }
}

fn truncation_hint(out: &mut crate::out::Out, shown: usize, total: usize) {
    if shown < total {
        out.hint(format!(
            "Showing {shown} of {total}; add --full (or --limit N) to see more, --json for everything."
        ));
    }
}

fn with_org(multi: bool, cols: &[&str]) -> Vec<String> {
    let mut v: Vec<String> = Vec::new();
    if multi {
        v.push("org".into());
    }
    v.extend(cols.iter().map(|c| c.to_string()));
    v
}

fn search_hit(p: &Row, needle: Option<&str>) -> bool {
    needle.is_none_or(|s| {
        canonical(&Value::Object(p.clone()))
            .to_lowercase()
            .contains(&s.to_lowercase())
    })
}

// -- commands ------------------------------------------------------------------------------------

pub struct AccountsArgs {
    pub org: String,
    pub bank: bool,
    pub kind: Option<String>,
    pub search: Option<String>,
    pub archived: bool,
}

pub fn accounts(app: &mut App, a: &AccountsArgs) -> Res<()> {
    let mut rows: Vec<Row> = Vec::new();
    let mut as_of = Value::Null;
    for tid in app.resolve(&a.org)? {
        let (data, at) = app.rows_of(&tid, "Accounts")?;
        as_of = Value::String(at);
        for p in &data {
            if a.bank && p.get("Type").and_then(Value::as_str) != Some("BANK") {
                continue;
            }
            if let Some(t) = &a.kind
                && fs(p, "Type").to_uppercase() != t.to_uppercase()
            {
                continue;
            }
            if !a.archived && p.get("Status").and_then(Value::as_str) == Some("ARCHIVED") {
                continue;
            }
            if let Some(s) = &a.search {
                let hay = format!("{} {}", fs(p, "Code"), fs(p, "Name")).to_lowercase();
                if !hay.contains(&s.to_lowercase()) {
                    continue;
                }
            }
            rows.push(obj![
                ("org", app.alias_of(&tid)),
                ("code", g(p, "Code")),
                ("name", g(p, "Name")),
                ("type", g(p, "Type")),
                ("status", g(p, "Status")),
                ("bank_account", g(p, "BankAccountNumber")),
                ("id", g(p, "AccountID")),
            ]);
        }
    }
    rows.sort_by_cached_key(|r| {
        (
            gs(r, "org"),
            or_text(&g(r, "code"), "~"),
            or_text(&g(r, "name"), ""),
        )
    });
    let mut cols = vec!["org", "code", "name", "type"];
    if a.bank {
        cols.push("bank_account");
    }
    if app.g.full {
        cols.extend(["status", "id"]);
    }
    let shown = app.limit(&rows);
    let mut out = app.out();
    out.field("as_of", as_of);
    let n = shown.len();
    out.table("accounts", &cols, shown, Some(rows.len()));
    truncation_hint(&mut out, n, rows.len());
    if !a.bank {
        out.hint("Add --bank for bank accounts only; --search <text> to filter.");
    }
    out.emit();
    Ok(())
}

pub fn org(app: &mut App, org: &str) -> Res<()> {
    let mut rows: Vec<Row> = Vec::new();
    for tid in app.resolve(org)? {
        let (data, _) = app.rows_of(&tid, "Organisation")?;
        for p in &data {
            let lock = |k: &str| {
                let s: String = gs(p, k).chars().take(10).collect();
                if s.is_empty() {
                    Value::Null
                } else {
                    Value::String(s)
                }
            };
            rows.push(obj![
                ("org", app.alias_of(&tid)),
                ("name", g(p, "Name")),
                ("class", g(p, "Class")),
                ("edition", g(p, "Edition")),
                ("type", g(p, "OrganisationType")),
                ("currency", g(p, "BaseCurrency")),
                (
                    "fy_end",
                    format!(
                        "{}/{}",
                        py_str(&g(p, "FinancialYearEndMonth")),
                        py_str(&g(p, "FinancialYearEndDay"))
                    )
                ),
                ("period_lock", lock("PeriodLockDate")),
                ("year_end_lock", lock("EndOfYearLockDate")),
            ]);
        }
    }
    let unlocked = rows
        .iter()
        .any(|r| !truthy(&g(r, "period_lock")) && !truthy(&g(r, "year_end_lock")));
    let mut out = app.out();
    out.table(
        "organisations",
        &[
            "org",
            "name",
            "class",
            "type",
            "currency",
            "fy_end",
            "period_lock",
            "year_end_lock",
        ],
        rows,
        None,
    );
    if unlocked {
        out.hint(
            "No lock dates set; they can only be set in the Xero web UI (Settings → Financial settings).",
        );
    }
    out.emit();
    Ok(())
}

pub fn contacts(app: &mut App, org: &str, search: Option<&str>, archived: bool) -> Res<()> {
    let tids = app.resolve(org)?;
    let mut rows: Vec<Row> = Vec::new();
    for tid in &tids {
        let (data, _) = app.rows_of(tid, "Contacts")?;
        for p in &data {
            if let Some(s) = search
                && !gs(p, "Name").to_lowercase().contains(&s.to_lowercase())
            {
                continue;
            }
            if !archived && p.get("ContactStatus").and_then(Value::as_str) == Some("ARCHIVED") {
                continue;
            }
            rows.push(obj![
                ("org", app.alias_of(tid)),
                ("name", g(p, "Name")),
                ("status", g(p, "ContactStatus")),
                ("supplier", g(p, "IsSupplier")),
                ("customer", g(p, "IsCustomer")),
                ("email", g(p, "EmailAddress")),
                ("id", g(p, "ContactID")),
            ]);
        }
    }
    let as_of = app.as_of_min(&tids, "Contacts")?;
    rows.sort_by_cached_key(|r| (gs(r, "org"), gs(r, "name").to_lowercase()));
    let shown = app.limit(&rows);
    let mut out = app.out();
    out.field("as_of", as_of);
    let n = shown.len();
    out.table_cols(
        "contacts",
        with_org(
            tids.len() > 1,
            &["name", "status", "supplier", "customer", "id"],
        ),
        shown,
        Some(rows.len()),
    );
    truncation_hint(&mut out, n, rows.len());
    out.emit();
    Ok(())
}

pub struct TxnArgs {
    pub org: String,
    pub since: Option<String>,
    pub until: Option<String>,
    pub account: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub deleted: bool,
}

const RECEIVE: [&str; 4] = [
    "RECEIVE",
    "RECEIVE-OVERPAYMENT",
    "RECEIVE-PREPAYMENT",
    "RECEIVE-TRANSFER",
];

pub fn txns(app: &mut App, a: &TxnArgs) -> Res<()> {
    let tids = app.resolve(&a.org)?;
    let mut rows: Vec<Row> = Vec::new();
    let mut net = cents_zero();
    for tid in &tids {
        // "Open in Xero" (local: the org's ShortCode from the record; no link without one)
        let sc = ownpurse_core::deeplink::short_code(app.record()?, tid);
        let (data, _) = app.rows_of(tid, "BankTransactions")?;
        for p in &data {
            let d = date_of(p);
            let empty = Map::new();
            let bank = sub(p, "BankAccount").unwrap_or(&empty);
            if !in_range(&d, a.since.as_deref(), a.until.as_deref()) {
                continue;
            }
            if let Some(acc) = &a.account {
                let hay = format!("{} {}", fs(bank, "Code"), fs(bank, "Name")).to_lowercase();
                if !hay.contains(&acc.to_lowercase()) {
                    continue;
                }
            }
            let status = p.get("Status").and_then(Value::as_str);
            if let Some(s) = &a.status
                && status != Some(s.to_uppercase().as_str())
            {
                continue;
            }
            if a.status.is_none() && !a.deleted && status == Some("DELETED") {
                continue;
            }
            if !search_hit(p, a.search.as_deref()) {
                continue;
            }
            let mut codes: Vec<String> = p
                .get("LineItems")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_object)
                .map(|li| gs(li, "AccountCode"))
                .collect();
            codes.sort();
            codes.dedup();
            let total = amt(p.get("Total"))?;
            let kind = g(p, "Type");
            if RECEIVE.contains(&kind.as_str().unwrap_or("")) {
                net += total;
            } else {
                net -= total;
            }
            rows.push(obj![
                ("org", app.alias_of(tid)),
                ("date", d),
                ("type", kind),
                ("bank", g(bank, "Name")),
                ("contact", sub_get(p, "Contact", "Name")),
                ("reference", g(p, "Reference")),
                ("total", dec(total)),
                ("codes", codes.join("|")),
                ("rec", g(p, "IsReconciled")),
                ("status", g(p, "Status")),
                ("id", g(p, "BankTransactionID")),
                (
                    "url",
                    sc.as_deref().and_then(|c| ownpurse_core::deeplink::url_for(
                        c,
                        "BankTransactions",
                        "",
                        &gs(p, "BankTransactionID")
                    ))
                ),
            ]);
        }
    }
    let as_of = app.as_of_min(&tids, "BankTransactions")?;
    rows.sort_by_cached_key(|r| (gs(r, "org"), gs(r, "date"), gs(r, "id")));
    let shown = app.limit(&rows);
    let mut out = app.out();
    out.field("as_of", as_of)
        .field("net_receive_minus_spend", dec(net));
    let n = shown.len();
    out.table_cols(
        "bank_transactions",
        with_org(
            tids.len() > 1,
            &[
                "date", "type", "bank", "contact", "total", "codes", "rec", "id",
            ],
        ),
        shown,
        Some(rows.len()),
    );
    truncation_hint(&mut out, n, rows.len());
    out.hint("Run `ownpurse show <org> BankTransactions <id>` for line items and history.");
    out.emit();
    Ok(())
}

pub struct JournalArgs {
    pub org: String,
    pub since: Option<String>,
    pub until: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

pub fn journals(app: &mut App, a: &JournalArgs) -> Res<()> {
    let tids = app.resolve(&a.org)?;
    let mut rows: Vec<Row> = Vec::new();
    for tid in &tids {
        let sc = ownpurse_core::deeplink::short_code(app.record()?, tid);
        let (data, _) = app.rows_of(tid, "ManualJournals")?;
        for p in &data {
            let d = date_of(p);
            if !in_range(&d, a.since.as_deref(), a.until.as_deref()) {
                continue;
            }
            if let Some(s) = &a.status
                && p.get("Status").and_then(Value::as_str) != Some(s.to_uppercase().as_str())
            {
                continue;
            }
            if !search_hit(p, a.search.as_deref()) {
                continue;
            }
            let lines: Vec<&Value> = p
                .get("JournalLines")
                .and_then(Value::as_array)
                .map(|a| a.iter().collect())
                .unwrap_or_default();
            let mut debit = cents_zero();
            for l in &lines {
                let la = l.get("LineAmount");
                if money(la).map_err(CliError::internal)? > Decimal::ZERO {
                    debit += amt(la)?;
                }
            }
            rows.push(obj![
                ("org", app.alias_of(tid)),
                ("date", d),
                ("status", g(p, "Status")),
                (
                    "narration",
                    gs(p, "Narration").chars().take(80).collect::<String>()
                ),
                ("debits", dec(debit)),
                ("lines", lines.len()),
                ("cash_basis", g(p, "ShowOnCashBasisReports")),
                ("id", g(p, "ManualJournalID")),
                (
                    "url",
                    sc.as_deref().and_then(|c| ownpurse_core::deeplink::url_for(
                        c,
                        "ManualJournals",
                        "",
                        &gs(p, "ManualJournalID")
                    ))
                ),
            ]);
        }
    }
    let as_of = app.as_of_min(&tids, "ManualJournals")?;
    rows.sort_by_cached_key(|r| (gs(r, "org"), gs(r, "date"), gs(r, "id")));
    let shown = app.limit(&rows);
    let mut out = app.out();
    out.field("as_of", as_of);
    let n = shown.len();
    out.table_cols(
        "manual_journals",
        with_org(
            tids.len() > 1,
            &["date", "status", "narration", "debits", "lines", "id"],
        ),
        shown,
        Some(rows.len()),
    );
    truncation_hint(&mut out, n, rows.len());
    out.hint("Run `ownpurse show <org> ManualJournals <id>` for journal lines.");
    out.emit();
    Ok(())
}

pub fn transfers(app: &mut App, org: &str, since: Option<&str>, until: Option<&str>) -> Res<()> {
    let tids = app.resolve(org)?;
    let mut rows: Vec<Row> = Vec::new();
    for tid in &tids {
        let (data, _) = app.rows_of(tid, "BankTransfers")?;
        for p in &data {
            let d = date_of(p);
            if !in_range(&d, since, until) {
                continue;
            }
            rows.push(obj![
                ("org", app.alias_of(tid)),
                ("date", d),
                ("from", sub_get(p, "FromBankAccount", "Name")),
                ("to", sub_get(p, "ToBankAccount", "Name")),
                ("amount", dec(amt(p.get("Amount"))?)),
                ("id", g(p, "BankTransferID")),
            ]);
        }
    }
    let as_of = app.as_of_min(&tids, "BankTransfers")?;
    rows.sort_by_cached_key(|r| (gs(r, "org"), gs(r, "date"), gs(r, "id")));
    let shown = app.limit(&rows);
    let mut out = app.out();
    out.field("as_of", as_of);
    let n = shown.len();
    out.table_cols(
        "bank_transfers",
        with_org(tids.len() > 1, &["date", "from", "to", "amount", "id"]),
        shown,
        Some(rows.len()),
    );
    truncation_hint(&mut out, n, rows.len());
    out.emit();
    Ok(())
}

// -- entities and the generic list ---------------------------------------------------------------

/// (name, id field, status field), in sync order.
pub const ENTITIES: [(&str, &str, &str); 14] = [
    ("Organisation", "OrganisationID", "OrganisationStatus"),
    ("Accounts", "AccountID", "Status"),
    ("TaxRates", "TaxType", "Status"),
    ("TrackingCategories", "TrackingCategoryID", "Status"),
    ("Currencies", "Code", ""),
    ("BankTransfers", "BankTransferID", ""),
    ("Contacts", "ContactID", "ContactStatus"),
    ("BankTransactions", "BankTransactionID", "Status"),
    ("ManualJournals", "ManualJournalID", "Status"),
    ("Invoices", "InvoiceID", "Status"),
    ("CreditNotes", "CreditNoteID", "Status"),
    ("Payments", "PaymentID", "Status"),
    ("Overpayments", "OverpaymentID", "Status"),
    ("Prepayments", "PrepaymentID", "Status"),
];

pub fn entity(name: &str) -> Res<(&'static str, &'static str, &'static str)> {
    ENTITIES
        .iter()
        .find(|(n, _, _)| n.to_lowercase() == name.to_lowercase())
        .copied()
        .ok_or_else(|| {
            CliError::usage(format!("Unknown entity '{name}'."))
                .hint(format!("Entities: {}", ENTITIES.map(|e| e.0).join(", ")))
        })
}

fn list_columns(entity: &str) -> Option<Vec<(&'static str, &'static str)>> {
    Some(match entity {
        "Invoices" => vec![
            ("date", "DateString"),
            ("type", "Type"),
            ("number", "InvoiceNumber"),
            ("contact", "Contact.Name"),
            ("status", "Status"),
            ("total", "Total"),
            ("due", "AmountDue"),
            ("id", "InvoiceID"),
        ],
        "CreditNotes" => vec![
            ("date", "DateString"),
            ("type", "Type"),
            ("number", "CreditNoteNumber"),
            ("contact", "Contact.Name"),
            ("status", "Status"),
            ("total", "Total"),
            ("remaining", "RemainingCredit"),
            ("id", "CreditNoteID"),
        ],
        "Payments" => vec![
            ("date", "DateString"),
            ("type", "PaymentType"),
            ("status", "Status"),
            ("amount", "Amount"),
            ("invoice", "Invoice.InvoiceNumber"),
            ("contact", "Invoice.Contact.Name"),
            ("bank", "Account.Code"),
            ("reference", "Reference"),
            ("id", "PaymentID"),
        ],
        "Overpayments" => vec![
            ("date", "DateString"),
            ("type", "Type"),
            ("contact", "Contact.Name"),
            ("status", "Status"),
            ("total", "Total"),
            ("remaining", "RemainingCredit"),
            ("id", "OverpaymentID"),
        ],
        "Prepayments" => vec![
            ("date", "DateString"),
            ("type", "Type"),
            ("contact", "Contact.Name"),
            ("status", "Status"),
            ("total", "Total"),
            ("remaining", "RemainingCredit"),
            ("id", "PrepaymentID"),
        ],
        "TaxRates" => vec![
            ("name", "Name"),
            ("tax_type", "TaxType"),
            ("rate", "EffectiveRate"),
            ("status", "Status"),
        ],
        "TrackingCategories" => vec![
            ("name", "Name"),
            ("status", "Status"),
            ("id", "TrackingCategoryID"),
        ],
        "Currencies" => vec![("code", "Code"), ("description", "Description")],
        "Contacts" => vec![
            ("name", "Name"),
            ("status", "ContactStatus"),
            ("email", "EmailAddress"),
            ("id", "ContactID"),
        ],
        "Accounts" => vec![
            ("code", "Code"),
            ("name", "Name"),
            ("type", "Type"),
            ("status", "Status"),
            ("id", "AccountID"),
        ],
        "BankTransactions" => vec![
            ("date", "DateString"),
            ("type", "Type"),
            ("status", "Status"),
            ("bank", "BankAccount.Name"),
            ("contact", "Contact.Name"),
            ("total", "Total"),
            ("id", "BankTransactionID"),
        ],
        "BankTransfers" => vec![
            ("date", "DateString"),
            ("from", "FromBankAccount.Name"),
            ("to", "ToBankAccount.Name"),
            ("amount", "Amount"),
            ("id", "BankTransferID"),
        ],
        "ManualJournals" => vec![
            ("date", "DateString"),
            ("status", "Status"),
            ("narration", "Narration"),
            ("id", "ManualJournalID"),
        ],
        "Organisation" => vec![
            ("name", "Name"),
            ("class", "Class"),
            ("currency", "BaseCurrency"),
            ("id", "OrganisationID"),
        ],
        _ => return None,
    })
}

/// A dotted payload path; dicts/lists come back as canonical JSON text.
fn path_value(obj: &Value, dotted: &str) -> Value {
    let mut cur = obj;
    for part in dotted.split('.') {
        cur = match cur {
            Value::Array(a) => match part.parse::<usize>() {
                Ok(i) if part.chars().all(|c| c.is_ascii_digit()) && i < a.len() => &a[i],
                _ => return Value::Null,
            },
            Value::Object(m) => match m.get(part) {
                Some(v) => v,
                None => return Value::Null,
            },
            _ => return Value::Null,
        };
    }
    match cur {
        Value::Object(_) | Value::Array(_) => Value::String(canonical(cur)),
        other => py_loaded(other),
    }
}

pub struct ListArgs {
    pub org: String,
    pub entity: String,
    pub since: Option<String>,
    pub until: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub fields: Option<String>,
}

pub fn list(app: &mut App, a: &ListArgs) -> Res<()> {
    let (ename, id_field, status_field) = entity(&a.entity)?;
    let mut cols: Vec<(String, String)> = list_columns(ename)
        .unwrap_or_else(|| vec![("id", id_field)])
        .into_iter()
        .map(|(n, p)| (n.to_string(), p.to_string()))
        .collect();
    if let Some(f) = &a.fields {
        cols = f
            .split(',')
            .map(|x| (x.to_string(), x.to_string()))
            .collect();
    }
    let tids = app.resolve(&a.org)?;
    let multi = tids.len() > 1;
    let mut rows: Vec<Row> = Vec::new();
    let mut as_of = Value::Null;
    for tid in &tids {
        let (data, at) = app.rows_of(tid, ename)?;
        as_of = Value::String(at);
        for p in &data {
            let d = date_of(p);
            if (a.since.is_some() || a.until.is_some())
                && !(!d.is_empty() && in_range(&d, a.since.as_deref(), a.until.as_deref()))
            {
                continue;
            }
            if let Some(s) = &a.status {
                let v = if status_field.is_empty() {
                    String::new()
                } else {
                    gs(p, status_field)
                };
                if v.to_uppercase() != s.to_uppercase() {
                    continue;
                }
            }
            if !search_hit(p, a.search.as_deref()) {
                continue;
            }
            let mut row = Row::new();
            if multi {
                row.insert("org".into(), Value::String(app.alias_of(tid)));
            }
            let pv = Value::Object(p.clone());
            for (name, path) in &cols {
                let v = if path == "DateString" {
                    Value::String(d.clone())
                } else {
                    path_value(&pv, path)
                };
                row.insert(name.clone(), v);
            }
            rows.push(row);
        }
    }
    if cols.first().is_some_and(|c| c.1 == "DateString") {
        rows.sort_by_cached_key(|r| (or_text(&g(r, "date"), ""), or_text(&g(r, "id"), "")));
    }
    let shown = app.limit(&rows);
    let mut out = app.out();
    out.field("entity", ename).field("as_of", as_of);
    let mut names: Vec<String> = if multi { vec!["org".into()] } else { vec![] };
    names.extend(cols.iter().map(|c| c.0.clone()));
    let n = shown.len();
    out.table_cols("records", names, shown, Some(rows.len()));
    truncation_hint(&mut out, n, rows.len());
    out.hint(format!(
        "`--fields a,b.c` picks payload fields; `ownpurse show <org> {ename} <id>` for one record."
    ));
    out.emit();
    Ok(())
}

pub fn show(app: &mut App, org: &str, entity_name: &str, id: &str) -> Res<()> {
    let tid = app.resolve_one(org)?;
    let (ename, _, _) = entity(entity_name)?;
    app.ensure(&tid, ename)?;
    let hits: Vec<Row> = {
        let rec = app.record()?;
        let mut st = rec.conn().prepare(
        "SELECT * FROM versions WHERE tenant_id=?1 AND entity=?2 AND xero_id LIKE ?3 ORDER BY xero_id, version",
    )?;
        let names: Vec<String> = st.column_names().iter().map(|s| s.to_string()).collect();
        let mut hits: Vec<Row> = Vec::new();
        let mut q = st.query(rusqlite::params![tid, ename, format!("{id}%")])?;
        while let Some(r) = q.next()? {
            let mut m = Row::new();
            for (i, n) in names.iter().enumerate() {
                m.insert(n.clone(), crate::usage::sql_value(r.get_ref(i)?));
            }
            hits.push(m);
        }
        hits
    };
    let mut ids: Vec<String> = hits.iter().map(|h| gs(h, "xero_id")).collect();
    ids.sort();
    ids.dedup();
    if ids.iter().any(|x| x == id) {
        ids = vec![id.to_string()];
    }
    if ids.is_empty() {
        return Err(CliError::usage(format!(
            "No {ename} with id starting '{id}' in the local record."
        ))
        .hint(format!(
            "Run `ownpurse sync {org} {ename}` or check the id."
        )));
    }
    if ids.len() > 1 {
        return Err(
            CliError::usage(format!("Id prefix '{id}' matches {} records.", ids.len()))
                .hint("Use a longer prefix."),
        );
    }
    let versions: Vec<Row> = hits
        .into_iter()
        .filter(|h| gs(h, "xero_id") == ids[0])
        .collect();
    let latest = versions.last().cloned().unwrap_or_default();
    let payload: Value =
        serde_json::from_str(&gs(&latest, "payload")).map_err(CliError::internal)?;
    let url = ownpurse_core::deeplink::xero_url(app.record()?, &tid, ename, &ids[0]);
    let mut out = app.out();
    out.field("entity", ename)
        .field("id", ids[0].as_str())
        .field("status", g(&latest, "status"));
    if let Some(u) = &url {
        out.field("url", u.as_str());
    }
    out.table(
        "versions",
        &[
            "version",
            "observed_at",
            "xero_updated_utc",
            "source",
            "status",
            "caused_by_op",
        ],
        versions,
        None,
    );
    if app.g.json {
        out.field("record", py_loaded(&payload));
    } else {
        out.field("record", canonical(&payload));
    }
    out.emit();
    Ok(())
}

/// `ownpurse open <org> <entity> <id> [--launch]`: the document's page in the Xero web app (through the org
/// switch). Local only: the org's ShortCode from the record, never a Xero call. `BankRec <AccountID>` is a
/// bank account's reconcile queue. `--launch` hands it to the system's `open`.
pub fn open(app: &mut App, org: &str, entity_name: &str, id: &str, launch: bool) -> Res<()> {
    let tid = app.resolve_one(org)?;
    let (ename, full) = if entity_name.eq_ignore_ascii_case("bankrec") {
        ("BankRec", id.to_string())
    } else {
        let (e, _, _) = entity(entity_name)?;
        // an id prefix resolves as in `show`
        let rows = app.record()?.current_rows(&tid, e)?;
        let hits: Vec<String> = rows
            .iter()
            .map(|r| r.xero_id.clone())
            .filter(|x| x == id || x.starts_with(id))
            .collect();
        let exact = hits.iter().find(|x| *x == id).cloned();
        match (exact, hits.len()) {
            (Some(x), _) => (e, x),
            (None, 1) => (e, hits[0].clone()),
            (None, 0) => (e, id.to_string()),
            (None, n) => {
                return Err(
                    CliError::usage(format!("Id prefix '{id}' matches {n} records."))
                        .hint("Use a longer prefix."),
                );
            }
        }
    };
    let Some(url) = ownpurse_core::deeplink::xero_url(app.record()?, &tid, ename, &full) else {
        return Err(CliError::usage(format!(
            "No Xero link for {ename} {full}: the org has no ShortCode in the record yet, or Xero has no page for {ename}."
        ))
        .hint(format!("Run `ownpurse sync {org} Organisation`; links exist for BankTransactions, ManualJournals, Invoices and BankRec.")));
    };
    if launch {
        let st = std::process::Command::new("open")
            .arg(&url)
            .stdin(std::process::Stdio::null())
            .status()
            .map_err(|e| CliError::runtime("open", format!("Could not run `open`: {e}")))?;
        if !st.success() {
            return Err(CliError::runtime(
                "open",
                format!("`open` exited with {st}"),
            ));
        }
    }
    let mut out = app.out();
    out.field("entity", ename)
        .field("id", full.as_str())
        .field("url", url.as_str())
        .field("launched", launch);
    out.hint("The link goes through Xero's org switch; you may be asked to log in.");
    out.emit();
    Ok(())
}

/// `report list <org> [kind]`: the reports the record holds, newest first (zero calls). `source` says whether
/// a row is an observation (`fetch`, from `report fetch`) or write evidence (`write`, a package's before/after
/// trial balance).
pub fn report_list(app: &mut App, org: &str, kind: Option<&str>) -> Res<()> {
    let only = match kind {
        Some(k) => Some(
            ownpurse_xero::reports::Kind::parse(k)
                .ok_or_else(|| {
                    CliError::usage(format!("Unknown report '{k}'."))
                        .hint("Use trialbalance, balancesheet, pl or banksummary.")
                })?
                .name(),
        ),
        None => None,
    };
    let tids = app.resolve(org)?;
    let mut rows = vec![];
    for tid in &tids {
        let alias = app.alias_of(tid);
        let conn = app.record()?.conn();
        let mut st = conn
            .prepare(
                "SELECT report, params, fetched_at, content_hash FROM reports WHERE tenant_id = ?1 \
                 AND (?2 IS NULL OR report = ?2) ORDER BY id DESC",
            )
            .map_err(|e| CliError::internal(e.to_string()))?;
        let found: Vec<(String, String, String, String)> = st
            .query_map(rusqlite::params![tid, only], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?))
            })
            .map_err(|e| CliError::internal(e.to_string()))?
            .flatten()
            .collect();
        for (report, params, at, hash) in found {
            let write = params.contains("\"xa_package\"");
            rows.push(obj![
                ("org", alias.clone()),
                ("report", report),
                ("params", params),
                ("fetched_at", at),
                ("source", if write { "write" } else { "fetch" }),
                ("content_hash", hash.chars().take(12).collect::<String>()),
            ]);
        }
    }
    let mut out = app.out();
    out.table(
        "reports",
        &[
            "org",
            "report",
            "params",
            "fetched_at",
            "source",
            "content_hash",
        ],
        rows,
        None,
    );
    out.hint("Fetch one with `ownpurse report fetch <org> trialbalance --date <YYYY-MM-DD>`.");
    out.emit();
    Ok(())
}
