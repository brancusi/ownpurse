//! ISO dates without a date library: Xero `/Date(ms+zzzz)/`, document dates and fiscal-year arithmetic.
//! Days are counted with the civil-from-days algorithm (Howard Hinnant), exact for the proleptic Gregorian
//! calendar.

use serde_json::Value;

/// Days since 1970-01-01 for a civil date.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m as i64 + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Civil date for days since 1970-01-01.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Parse `YYYY-MM-DD` (the first 10 characters of an ISO timestamp).
pub fn parse_iso(s: &str) -> Option<(i64, u32, u32)> {
    let s = s.get(..10)?;
    let mut it = s.split('-');
    let y = it.next()?.parse().ok()?;
    let m: u32 = it.next()?.parse().ok()?;
    let d: u32 = it.next()?.parse().ok()?;
    ((1..=12).contains(&m) && (1..=31).contains(&d)).then_some((y, m, d))
}

pub fn iso(y: i64, m: u32, d: u32) -> String {
    format!("{y:04}-{m:02}-{d:02}")
}

/// `date ± n days` for an ISO date.
pub fn add_days(date: &str, n: i64) -> Option<String> {
    let (y, m, d) = parse_iso(date)?;
    let (y2, m2, d2) = civil_from_days(days_from_civil(y, m, d) + n);
    Some(iso(y2, m2, d2))
}

/// `/Date(1735689600000+0000)/` → `2025-01-01` (UTC). ISO text passes through (first 10 chars).
pub fn xero_date(value: &str) -> Option<String> {
    if let Some(inner) = value
        .strip_prefix("/Date(")
        .and_then(|r| r.strip_suffix(")/"))
    {
        let digits_end = inner[1..]
            .find(['+', '-'])
            .map(|i| i + 1)
            .unwrap_or(inner.len());
        let ms: i64 = inner[..digits_end].parse().ok()?;
        let days = ms.div_euclid(86_400_000);
        let (y, m, d) = civil_from_days(days);
        return Some(iso(y, m, d));
    }
    parse_iso(value).map(|(y, m, d)| iso(y, m, d))
}

/// A document's date as `YYYY-MM-DD`: `DateString` when present, else the `/Date(ms)/` form
/// (ManualJournals, Payments).
pub fn doc_date(p: &Value) -> String {
    if let Some(s) = p
        .get("DateString")
        .and_then(Value::as_str)
        .filter(|s| !s.is_empty())
    {
        return s.chars().take(10).collect();
    }
    p.get("Date")
        .and_then(Value::as_str)
        .and_then(xero_date)
        .unwrap_or_default()
}

/// First day of the fiscal year containing `as_of`, for a year ending on `fy_end_month`/`fy_end_day`.
pub fn fiscal_year_start(as_of: &str, fy_end_month: u32, fy_end_day: u32) -> Option<String> {
    let (y, m, d) = parse_iso(as_of)?;
    let end_this = (m, d) > (fy_end_month, fy_end_day);
    let start_year = if end_this { y } else { y - 1 };
    add_days(&iso(start_year, fy_end_month, fy_end_day), 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn xero_dates() {
        assert_eq!(
            xero_date("/Date(1735689600000+0000)/").unwrap(),
            "2025-01-01"
        );
        assert_eq!(xero_date("/Date(1735689599999)/").unwrap(), "2024-12-31");
        assert_eq!(xero_date("/Date(-86400000+0000)/").unwrap(), "1969-12-31");
        assert_eq!(xero_date("2025-03-01T00:00:00").unwrap(), "2025-03-01");
        let p: Value = serde_json::json!({"Date": "/Date(1740787200000+0000)/"});
        assert_eq!(doc_date(&p), "2025-03-01");
        let p: Value =
            serde_json::json!({"DateString": "2025-04-01T00:00:00", "Date": "/Date(0)/"});
        assert_eq!(doc_date(&p), "2025-04-01");
    }

    #[test]
    fn fiscal_years() {
        assert_eq!(
            fiscal_year_start("2025-12-31", 12, 31).unwrap(),
            "2025-01-01"
        );
        assert_eq!(
            fiscal_year_start("2025-06-30", 12, 31).unwrap(),
            "2025-01-01"
        );
        assert_eq!(
            fiscal_year_start("2025-06-30", 6, 30).unwrap(),
            "2024-07-01"
        );
        assert_eq!(
            fiscal_year_start("2025-07-01", 6, 30).unwrap(),
            "2025-07-01"
        );
        assert_eq!(add_days("2025-01-01", -1).unwrap(), "2024-12-31");
        assert_eq!(add_days("2024-02-28", 1).unwrap(), "2024-02-29");
    }

    proptest! {
        #[test]
        fn civil_round_trip(z in -200_000i64..200_000i64) {
            let (y, m, d) = civil_from_days(z);
            prop_assert_eq!(days_from_civil(y, m, d), z);
        }
    }
}
