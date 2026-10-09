//! Normalisation: the one document shape every manifest entry is diffed and patched against (design §2).
//!
//! - noise dropped: `UpdatedDateUTC`, `HasAttachments`, Xero-derived totals (`DebitTotal`/`CreditTotal`),
//!   validation chatter, and any `_xa*` key at any depth;
//! - the date is one ISO field, `Date` (`DateString` and `/Date(…)/` folded into it);
//! - line arrays (`JournalLines`, `LineItems`) become maps keyed by `LineItemID`, or by `n<k>` (the line's
//!   1-based position) when a line has none, as manual-journal lines from the API don't; a line's `AccountID`
//!   is dropped when it has an `AccountCode` (the code names the account; recodes change only the code);
//! - `Attachments` become a map keyed by `FileName`;
//! - money and quantities are exact decimal strings with at least 2 decimals (`1240` and `1240.00` are the
//!   same value, so they never diff).
//!
//! [`denormalise`] gives back Xero's shape (arrays in the "before" order, numbers, `Date` + `DateString`) for
//! the ledger and the request builders.

use rust_decimal::Decimal;
use serde_json::{Map, Value};
use std::str::FromStr;

/// Top-level fields that never carry business meaning.
pub const NOISE: [&str; 8] = [
    "UpdatedDateUTC",
    "HasAttachments",
    "DebitTotal",
    "CreditTotal",
    "ValidationErrors",
    "Warnings",
    "StatusAttributeString",
    "HasErrors",
];

/// Fields held as exact decimal strings.
pub const DECIMALS: [&str; 12] = [
    "LineAmount",
    "TaxAmount",
    "UnitAmount",
    "Quantity",
    "Total",
    "SubTotal",
    "TotalTax",
    "AmountDue",
    "AmountPaid",
    "AmountCredited",
    "Amount",
    "CurrencyRate",
];

/// The keyed line arrays.
pub const LINE_ARRAYS: [&str; 2] = ["JournalLines", "LineItems"];

/// The canonical text of a decimal (exact, at least 2 decimals; `None` if it isn't a number).
pub fn decimal_text(v: &Value) -> Option<String> {
    let t = match v {
        Value::Number(n) => n.to_string(),
        Value::String(s) => s.trim().to_string(),
        _ => return None,
    };
    let d = crate::money::parse(&t).ok()?;
    let mut d = d.normalize();
    if d.scale() < 2 {
        d.rescale(2);
    }
    Some(d.to_string())
}

fn strip_marks(v: &Value) -> Value {
    match v {
        Value::Object(m) => Value::Object(
            m.iter()
                .filter(|(k, _)| !k.starts_with("_xa"))
                .map(|(k, v)| (k.clone(), strip_marks(v)))
                .collect(),
        ),
        Value::Array(a) => Value::Array(a.iter().map(strip_marks).collect()),
        o => o.clone(),
    }
}

fn norm_line(line: &Value) -> Value {
    let Value::Object(m) = line else {
        return line.clone();
    };
    let has_code = m
        .get("AccountCode")
        .and_then(Value::as_str)
        .is_some_and(|c| !c.is_empty());
    let mut out = Map::new();
    for (k, v) in m {
        if k == "AccountID" && has_code {
            continue;
        }
        if DECIMALS.contains(&k.as_str())
            && let Some(t) = decimal_text(v)
        {
            out.insert(k.clone(), Value::String(t));
            continue;
        }
        out.insert(k.clone(), v.clone());
    }
    Value::Object(out)
}

/// The key of a line: its `LineItemID`, else `n<k>` (1-based position).
pub fn line_key(line: &Value, index: usize) -> String {
    line.get("LineItemID")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| format!("n{}", index + 1))
}

/// The normalised form of a Xero document of `entity` (any entity: unknown fields pass through).
pub fn normalise(doc: &Value) -> Value {
    let doc = strip_marks(doc);
    let Value::Object(m) = doc else {
        return doc;
    };
    let mut out = Map::new();
    let date = m
        .get("DateString")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
        .map(|s| s.chars().take(10).collect::<String>())
        .or_else(|| {
            m.get("Date")
                .and_then(Value::as_str)
                .map(|s| crate::dates::xero_date(s).unwrap_or_else(|| s.chars().take(10).collect()))
        });
    for (k, v) in &m {
        if NOISE.contains(&k.as_str()) || k == "DateString" {
            continue;
        }
        if k == "Date" {
            if let Some(d) = &date {
                out.insert("Date".into(), Value::String(d.clone()));
            }
            continue;
        }
        if LINE_ARRAYS.contains(&k.as_str())
            && let Value::Array(lines) = v
        {
            let mut lm = Map::new();
            for (i, l) in lines.iter().enumerate() {
                lm.insert(line_key(l, i), norm_line(l));
            }
            out.insert(k.clone(), Value::Object(lm));
            continue;
        }
        if k == "Attachments"
            && let Value::Array(atts) = v
        {
            let mut am = Map::new();
            for a in atts {
                let name = a
                    .get("FileName")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                am.insert(name, a.clone());
            }
            out.insert(k.clone(), Value::Object(am));
            continue;
        }
        if DECIMALS.contains(&k.as_str())
            && let Some(t) = decimal_text(v)
        {
            out.insert(k.clone(), Value::String(t));
            continue;
        }
        out.insert(k.clone(), v.clone());
    }
    if !m.contains_key("Date")
        && let Some(d) = date
    {
        out.insert("Date".into(), Value::String(d));
    }
    Value::Object(out)
}

fn number(t: &str) -> Value {
    serde_json::Number::from_str(t)
        .map(Value::Number)
        .unwrap_or_else(|_| Value::String(t.to_string()))
}

fn denorm_line(line: &Value) -> Value {
    let Value::Object(m) = line else {
        return line.clone();
    };
    Value::Object(
        m.iter()
            .map(|(k, v)| match (DECIMALS.contains(&k.as_str()), v) {
                (true, Value::String(s)) => (k.clone(), number(s)),
                _ => (k.clone(), v.clone()),
            })
            .collect(),
    )
}

/// Keys in `before`'s order first, then the rest in `after`'s order.
fn ordered<'a>(
    after: &'a Map<String, Value>,
    before: Option<&Value>,
) -> Vec<(&'a String, &'a Value)> {
    let mut out: Vec<(&String, &Value)> = vec![];
    if let Some(Value::Object(b)) = before {
        for k in b.keys() {
            if let Some((ka, va)) = after.get_key_value(k) {
                out.push((ka, va));
            }
        }
    }
    let mut rest: Vec<(&String, &Value)> = after
        .iter()
        .filter(|(k, _)| !out.iter().any(|(o, _)| o == k))
        .collect();
    // positional keys keep their numeric order (n2 before n10)
    rest.sort_by_key(|(k, _)| {
        k.strip_prefix('n')
            .and_then(|n| n.parse::<u64>().ok())
            .unwrap_or(u64::MAX)
    });
    out.extend(rest);
    out
}

/// Xero's shape of a normalised document. `before` (normalised) fixes the line order of existing lines.
pub fn denormalise(after: &Value, before: Option<&Value>) -> Value {
    let Value::Object(m) = after else {
        return after.clone();
    };
    let mut out = Map::new();
    for (k, v) in m {
        if LINE_ARRAYS.contains(&k.as_str())
            && let Value::Object(lines) = v
        {
            let b = before.and_then(|b| b.get(k));
            out.insert(
                k.clone(),
                Value::Array(
                    ordered(lines, b)
                        .into_iter()
                        .map(|(_, l)| denorm_line(l))
                        .collect(),
                ),
            );
            continue;
        }
        if k == "Attachments"
            && let Value::Object(atts) = v
        {
            let b = before.and_then(|b| b.get(k));
            out.insert(
                k.clone(),
                Value::Array(
                    ordered(atts, b)
                        .into_iter()
                        .map(|(_, a)| a.clone())
                        .collect(),
                ),
            );
            continue;
        }
        if k == "Date"
            && let Value::String(d) = v
        {
            out.insert("Date".into(), Value::String(d.clone()));
            out.insert("DateString".into(), Value::String(format!("{d}T00:00:00")));
            continue;
        }
        match (DECIMALS.contains(&k.as_str()), v) {
            (true, Value::String(s)) => out.insert(k.clone(), number(s)),
            _ => out.insert(k.clone(), v.clone()),
        };
    }
    Value::Object(out)
}

/// A decimal field of a normalised value (zero when absent).
pub fn dec(v: &Value, k: &str) -> Decimal {
    v.get(k)
        .and_then(decimal_text)
        .and_then(|t| Decimal::from_str(&t).ok())
        .unwrap_or_default()
}

/// The fingerprint of a normalised document: sha256 of its canonical JSON (normalisation already dropped
/// the noise; the write path's own target fingerprint reads Xero's raw shapes, with line arrays).
pub fn fingerprint(normalised: &Value) -> String {
    crate::write::sha256_hex(crate::canon::canonical(normalised).as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn keyed_lines_exact_money_one_date() {
        let mj = json!({"ManualJournalID": "mj-1", "Date": "/Date(1735603200000+0000)/",
            "DateString": "2024-12-31T00:00:00", "UpdatedDateUTC": "/Date(1)/", "_xa_branch": {"x": 1},
            "JournalLines": [{"AccountCode": "6420", "AccountID": "a-1", "LineAmount": 1240},
                             {"AccountCode": "2420", "LineAmount": -1240.0}]});
        let n = normalise(&mj);
        assert_eq!(
            n,
            json!({"ManualJournalID": "mj-1", "Date": "2024-12-31",
                "JournalLines": {"n1": {"AccountCode": "6420", "LineAmount": "1240.00"},
                                 "n2": {"AccountCode": "2420", "LineAmount": "-1240.00"}}})
        );
        let back = denormalise(&n, Some(&n));
        assert_eq!(
            back["JournalLines"][1]["LineAmount"].to_string(),
            "-1240.00"
        );
        assert_eq!(back["DateString"], "2024-12-31T00:00:00");
        assert_eq!(
            normalise(&back),
            n,
            "normalise ∘ denormalise is the identity"
        );
        let bt = json!({"LineItems": [{"LineItemID": "li-9", "AccountCode": "5510", "LineAmount": "30.5"}],
                        "Attachments": [{"FileName": "a.pdf"}]});
        let n = normalise(&bt);
        assert_eq!(n["LineItems"]["li-9"]["LineAmount"], "30.50");
        assert!(n["Attachments"]["a.pdf"].is_object());
        assert_eq!(
            fingerprint(&n),
            fingerprint(&normalise(&denormalise(&n, None)))
        );
    }

    #[test]
    fn line_order_survives_removes_and_adds() {
        let before = json!({"JournalLines": {"n1": {"a": 1}, "n2": {"a": 2}, "n3": {"a": 3}}});
        let mut after = before.clone();
        after["JournalLines"]
            .as_object_mut()
            .unwrap()
            .swap_remove("n1");
        after["JournalLines"]["n10"] = json!({"a": 10});
        after["JournalLines"]["n4"] = json!({"a": 4});
        let d = denormalise(&after, Some(&before));
        let a: Vec<i64> = d["JournalLines"]
            .as_array()
            .unwrap()
            .iter()
            .map(|l| l["a"].as_i64().unwrap())
            .collect();
        assert_eq!(a, vec![2, 3, 4, 10]);
    }
}
