//! The generic document diff model (design §6, spec manifests §1): for ANY entity, the normalised before
//! and after become field rows (`+` added, `−` removed, `~` changed, unchanged) and, for keyed line arrays,
//! a lines table with changed cells and totals. One model for every entity: a small label table names the
//! fields we know; unknown paths show raw (`/Url`). The TUI renders it (with word highlights inside text
//! fields); the CLI can print it.

use super::normalise::{DECIMALS, dec};
use rust_decimal::Decimal;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    Same,
    Added,
    Removed,
    Changed,
}

impl Mark {
    /// `+`, `−`, `~` or a space.
    pub fn glyph(self) -> &'static str {
        match self {
            Mark::Same => " ",
            Mark::Added => "+",
            Mark::Removed => "−",
            Mark::Changed => "~",
        }
    }
    pub fn of(old: Option<&str>, new: Option<&str>) -> Mark {
        match (old, new) {
            (None, Some(_)) => Mark::Added,
            (Some(_), None) => Mark::Removed,
            (Some(a), Some(b)) if a != b => Mark::Changed,
            _ => Mark::Same,
        }
    }
}

/// How a value renders: text gets word highlights; money a Δ; enums, dates and the rest show old → new.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Text,
    Money,
    Enum,
    Date,
    Bool,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldRow {
    pub mark: Mark,
    /// The patch path (`/Narration`, `/Attachments/support.csv`).
    pub path: String,
    pub label: String,
    /// True when the label comes from the table (else it is the raw path, shown quiet).
    pub known: bool,
    pub old: Option<String>,
    pub new: Option<String>,
    pub kind: ValueKind,
}

impl FieldRow {
    /// For money: new − old.
    pub fn delta(&self) -> Option<Decimal> {
        if self.kind != ValueKind::Money {
            return None;
        }
        let p = |s: &Option<String>| {
            s.as_deref()
                .and_then(|t| crate::money::parse(t).ok())
                .unwrap_or_default()
        };
        Some(p(&self.new) - p(&self.old))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineCell {
    pub old: Option<String>,
    pub new: Option<String>,
}

impl LineCell {
    pub fn changed(&self) -> bool {
        self.old != self.new
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineRow {
    pub mark: Mark,
    /// The line's key (`n2`, a LineItemID).
    pub key: String,
    /// The line's patch path (`/JournalLines/n2`).
    pub path: String,
    pub cells: Vec<LineCell>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Column {
    pub label: &'static str,
    pub kind: ValueKind,
    /// The field it reads (`AccountCode`), or a derived column (`debit` / `credit` from `LineAmount`).
    pub field: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Totals {
    pub debits: Decimal,
    pub credits: Decimal,
    /// Journals: debits − credits (zero when balanced); other documents: None.
    pub imbalance: Option<Decimal>,
    /// The lines would all be removed (a void): the totals are what disappears.
    pub removed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lines {
    /// `JournalLines` or `LineItems`.
    pub key: String,
    pub columns: Vec<Column>,
    pub rows: Vec<LineRow>,
    pub totals: Totals,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocDiff {
    pub entity: String,
    /// `create`, `change` or `void`.
    pub kind: &'static str,
    pub fields: Vec<FieldRow>,
    pub lines: Option<Lines>,
}

impl DocDiff {
    pub fn changed_fields(&self) -> usize {
        self.fields.iter().filter(|f| f.mark != Mark::Same).count()
    }
    pub fn changed_lines(&self) -> usize {
        self.lines.as_ref().map_or(0, |l| {
            l.rows.iter().filter(|r| r.mark != Mark::Same).count()
        })
    }
    /// `changed: 2 fields · 1 line`.
    pub fn summary(&self) -> String {
        let f = self.changed_fields();
        let l = self.changed_lines();
        let mut parts = vec![];
        if f > 0 {
            parts.push(format!("{f} field{}", if f == 1 { "" } else { "s" }));
        }
        if l > 0 {
            parts.push(format!("{l} line{}", if l == 1 { "" } else { "s" }));
        }
        if parts.is_empty() {
            "no changes".into()
        } else {
            parts.join(" · ")
        }
    }
}

/// The short code of an entity for lists (`MJ`, `BT`, `INV`).
pub fn entity_code(entity: &str) -> String {
    match entity {
        "ManualJournals" => "MJ".into(),
        "BankTransactions" => "BT".into(),
        "Invoices" => "INV".into(),
        "CreditNotes" => "CN".into(),
        "Contacts" => "CT".into(),
        "Payments" => "PAY".into(),
        "BankTransfers" => "TR".into(),
        other => other.chars().filter(|c| c.is_ascii_uppercase()).collect(),
    }
}

/// The human name of an entity.
pub fn entity_label(entity: &str) -> String {
    match entity {
        "ManualJournals" => "manual journal".into(),
        "BankTransactions" => "bank transaction".into(),
        "Invoices" => "invoice".into(),
        "CreditNotes" => "credit note".into(),
        "Contacts" => "contact".into(),
        "BankTransfers" => "bank transfer".into(),
        "Payments" => "payment".into(),
        other => other.to_string(),
    }
}

/// The label table: `(field, label, kind)` in display order. Fields not listed show raw, after these.
const FIELDS: [(&str, &str, ValueKind); 16] = [
    ("Date", "Date", ValueKind::Date),
    ("DueDate", "Due", ValueKind::Date),
    ("Type", "Type", ValueKind::Enum),
    ("Status", "Status", ValueKind::Enum),
    ("Contact", "Contact", ValueKind::Text),
    ("BankAccount", "Bank account", ValueKind::Raw),
    ("Reference", "Reference", ValueKind::Text),
    ("Narration", "Narration", ValueKind::Text),
    ("Name", "Name", ValueKind::Text),
    ("LineAmountTypes", "Amounts", ValueKind::Enum),
    ("ShowOnCashBasisReports", "Cash basis", ValueKind::Bool),
    ("IsReconciled", "Reconciled", ValueKind::Bool),
    ("SubTotal", "Subtotal", ValueKind::Money),
    ("TotalTax", "Tax", ValueKind::Money),
    ("Total", "Total", ValueKind::Money),
    ("CurrencyCode", "Currency", ValueKind::Enum),
];

/// Fields never shown as rows (ids, the keyed arrays, Xero bookkeeping).
const HIDDEN: [&str; 6] = [
    "BankTransactionID",
    "ManualJournalID",
    "InvoiceID",
    "ContactID",
    "JournalLines",
    "LineItems",
];

/// A value as display text: objects by their name/code, booleans yes/no.
pub fn text_of(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Bool(b) => if *b { "yes" } else { "no" }.into(),
        Value::Null => "—".into(),
        Value::Number(n) => n.to_string(),
        Value::Object(o) => {
            for k in ["Name", "Code", "FileName", "AccountID", "ContactID"] {
                if let Some(Value::String(s)) = o.get(k) {
                    return s.clone();
                }
            }
            crate::canon::canonical(v)
        }
        Value::Array(_) => crate::canon::canonical(v),
    }
}

fn kind_of(field: &str, v: Option<&Value>) -> (String, bool, ValueKind) {
    if let Some((_, l, k)) = FIELDS.iter().find(|(f, _, _)| *f == field) {
        return ((*l).into(), true, *k);
    }
    let kind = if DECIMALS.contains(&field) {
        ValueKind::Money
    } else {
        match v {
            Some(Value::Bool(_)) => ValueKind::Bool,
            Some(Value::String(_)) => ValueKind::Text,
            _ => ValueKind::Raw,
        }
    };
    (format!("/{field}"), false, kind)
}

fn esc(k: &str) -> String {
    k.replace('~', "~0").replace('/', "~1")
}

/// The model for one document change. `before` None: a new document.
pub fn doc_diff(
    entity: &str,
    kind: &'static str,
    before: Option<&Value>,
    after: &Value,
) -> DocDiff {
    let empty = serde_json::Map::new();
    let b = before.and_then(Value::as_object).unwrap_or(&empty);
    let a = after.as_object().unwrap_or(&empty);
    // the header fields: the table's order, then the rest alphabetically
    let mut keys: Vec<String> = FIELDS
        .iter()
        .map(|(f, _, _)| f.to_string())
        .filter(|f| a.contains_key(f) || b.contains_key(f))
        .collect();
    let mut rest: Vec<String> = a
        .keys()
        .chain(b.keys())
        .filter(|k| !keys.contains(k) && !HIDDEN.contains(&k.as_str()) && *k != "Attachments")
        .cloned()
        .collect();
    rest.sort();
    rest.dedup();
    keys.extend(rest);
    let mut fields = vec![];
    for k in &keys {
        let old = b.get(k);
        let new = a.get(k);
        let (label, known, vk) = kind_of(k, new.or(old));
        let o = old.map(text_of);
        let n = new.map(text_of);
        fields.push(FieldRow {
            mark: Mark::of(o.as_deref(), n.as_deref()),
            path: format!("/{}", esc(k)),
            label,
            known,
            old: o,
            new: n,
            kind: vk,
        });
    }
    // attachments: one row each
    let atts = |m: &serde_json::Map<String, Value>| {
        m.get("Attachments")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_default()
    };
    let (ba, aa) = (atts(b), atts(a));
    let mut names: Vec<&String> = ba.keys().chain(aa.keys()).collect();
    names.sort();
    names.dedup();
    for n in names {
        let desc = |v: &Value| {
            let size = v.get("ContentLength").and_then(Value::as_u64);
            match size {
                Some(s) => format!("{n} · {s} bytes"),
                None => n.clone(),
            }
        };
        let o = ba.get(n).map(desc);
        let nw = aa.get(n).map(desc);
        fields.push(FieldRow {
            mark: Mark::of(o.as_deref(), nw.as_deref()),
            path: format!("/Attachments/{}", esc(n)),
            label: format!("attachment {n}"),
            known: true,
            old: o,
            new: nw,
            kind: ValueKind::Raw,
        });
    }
    let lines = lines(entity, kind, b, a);
    DocDiff {
        entity: entity.into(),
        kind,
        fields,
        lines,
    }
}

fn lines(
    entity: &str,
    kind: &'static str,
    b: &serde_json::Map<String, Value>,
    a: &serde_json::Map<String, Value>,
) -> Option<Lines> {
    let key = ["JournalLines", "LineItems"]
        .into_iter()
        .find(|k| a.contains_key(*k) || b.contains_key(*k))?;
    let empty = serde_json::Map::new();
    let bl = b.get(key).and_then(Value::as_object).unwrap_or(&empty);
    let al = a.get(key).and_then(Value::as_object).unwrap_or(&empty);
    let columns: Vec<Column> = if entity == "ManualJournals" || key == "JournalLines" {
        vec![
            Column {
                label: "account",
                kind: ValueKind::Enum,
                field: "AccountCode",
            },
            Column {
                label: "description",
                kind: ValueKind::Text,
                field: "Description",
            },
            Column {
                label: "debit",
                kind: ValueKind::Money,
                field: "debit",
            },
            Column {
                label: "credit",
                kind: ValueKind::Money,
                field: "credit",
            },
        ]
    } else {
        vec![
            Column {
                label: "account",
                kind: ValueKind::Enum,
                field: "AccountCode",
            },
            Column {
                label: "description",
                kind: ValueKind::Text,
                field: "Description",
            },
            Column {
                label: "amount",
                kind: ValueKind::Money,
                field: "LineAmount",
            },
            Column {
                label: "tax",
                kind: ValueKind::Money,
                field: "TaxAmount",
            },
        ]
    };
    let cell = |l: Option<&Value>, c: &Column| -> Option<String> {
        let l = l?;
        match c.field {
            "debit" | "credit" => {
                let amt = dec(l, "LineAmount");
                let v = if c.field == "debit" { amt } else { -amt };
                Some(if v > Decimal::ZERO {
                    crate::money::money_str(v)
                } else {
                    String::new()
                })
            }
            f => Some(l.get(f).map(text_of).unwrap_or_default()),
        }
    };
    // keys: before's order, then new ones
    let mut ks: Vec<&String> = bl.keys().collect();
    ks.extend(al.keys().filter(|k| !bl.contains_key(*k)));
    let mut rows = vec![];
    for k in ks {
        let (o, n) = (bl.get(k), al.get(k));
        let cells: Vec<LineCell> = columns
            .iter()
            .map(|c| LineCell {
                old: cell(o, c),
                new: cell(n, c),
            })
            .collect();
        let mark = match (o, n) {
            (None, Some(_)) => Mark::Added,
            (Some(_), None) => Mark::Removed,
            (Some(x), Some(y)) if x != y => Mark::Changed,
            _ => Mark::Same,
        };
        rows.push(LineRow {
            mark,
            path: format!("/{key}/{}", esc(k)),
            key: k.clone(),
            cells,
        });
    }
    let removed = kind == "void";
    let src = if removed { bl } else { al };
    let (mut debits, mut credits) = (Decimal::ZERO, Decimal::ZERO);
    for l in src.values() {
        let amt = dec(l, "LineAmount");
        if amt > Decimal::ZERO {
            debits += amt
        } else {
            credits -= amt
        }
    }
    let journal = key == "JournalLines";
    Some(Lines {
        key: key.into(),
        columns,
        rows,
        totals: Totals {
            debits,
            credits,
            imbalance: journal.then(|| debits - credits),
            removed,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manifest::normalise::normalise;
    use serde_json::json;

    #[test]
    fn a_changed_journal_new_one_and_unknown_entity() {
        let before = normalise(
            &json!({"ManualJournalID": "m", "Date": "2025-12-21", "Status": "POSTED",
            "Narration": "owner paid costs", "JournalLines": [
                {"AccountCode": "6140", "LineAmount": 1148.63}, {"AccountCode": "2700", "LineAmount": -1148.63}]}),
        );
        let mut after = before.clone();
        after["Narration"] = json!("owner paid costs, December");
        after["JournalLines"]["n1"]["AccountCode"] = json!("5020");
        let d = doc_diff("ManualJournals", "change", Some(&before), &after);
        assert_eq!(d.summary(), "1 field · 1 line");
        let n = d.fields.iter().find(|f| f.label == "Narration").unwrap();
        assert_eq!((n.mark, n.kind), (Mark::Changed, ValueKind::Text));
        let l = d.lines.as_ref().unwrap();
        assert_eq!(l.rows[0].mark, Mark::Changed);
        assert_eq!(
            l.rows[0].cells[0],
            LineCell {
                old: Some("6140".into()),
                new: Some("5020".into())
            }
        );
        assert_eq!(l.totals.imbalance, Some(Decimal::ZERO));
        let new = doc_diff("ManualJournals", "create", None, &after);
        assert!(new.fields.iter().all(|f| f.mark == Mark::Added));
        assert!(
            new.lines
                .unwrap()
                .rows
                .iter()
                .all(|r| r.mark == Mark::Added)
        );
        let c = doc_diff(
            "Contacts",
            "change",
            Some(&json!({"Name": "A", "Url": "x"})),
            &json!({"Name": "B", "Url": "y"}),
        );
        let url = c.fields.iter().find(|f| f.path == "/Url").unwrap();
        assert!(!url.known && url.label == "/Url" && url.mark == Mark::Changed);
        let mut amt = before.clone();
        amt["Total"] = json!("1240.00");
        let mut b2 = before.clone();
        b2["Total"] = json!("1148.63");
        let t = doc_diff("ManualJournals", "change", Some(&b2), &amt);
        assert_eq!(
            t.fields
                .iter()
                .find(|f| f.label == "Total")
                .unwrap()
                .delta(),
            Some("91.37".parse().unwrap())
        );
    }
}
