//! UTC time without a date library: seconds since the epoch, parsed from the record's ISO timestamps.
//! The clock is injected (`App::now`), pinned in tests and guide renders (`OWNPURSE_NOW`), so ages
//! never drift in a snapshot. Ages are minute-granular in the UI (spec §8: redraw when a minute passes).

use ownpurse_core::dates::{civil_from_days, days_from_civil, iso, parse_iso};

/// `YYYY-MM-DD[THH:MM[:SS[.fff]]][Z|±hh:mm]` → seconds since 1970-01-01 UTC. Offsets other than UTC
/// are not expected in the record (it holds UTC) and are ignored.
pub fn parse_utc(s: &str) -> Option<i64> {
    let (y, m, d) = parse_iso(s)?;
    let rest = s.get(10..).unwrap_or("");
    let (mut hh, mut mm, mut ss) = (0i64, 0i64, 0i64);
    if let Some(t) = rest.strip_prefix('T').or_else(|| rest.strip_prefix(' ')) {
        hh = t.get(0..2)?.parse().ok()?;
        mm = t.get(3..5)?.parse().ok()?;
        if t.get(5..6) == Some(":") {
            ss = t.get(6..8)?.parse().ok()?;
        }
    }
    Some(days_from_civil(y, m, d) * 86_400 + hh * 3_600 + mm * 60 + ss)
}

/// The real clock.
pub fn system_now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// `OWNPURSE_NOW` when set (and parseable), else the real clock.
pub fn env_now() -> i64 {
    std::env::var("OWNPURSE_NOW")
        .ok()
        .and_then(|v| parse_utc(&v))
        .unwrap_or_else(system_now)
}

/// The UTC date of an instant, `YYYY-MM-DD`.
pub fn date_of(t: i64) -> String {
    let (y, m, d) = civil_from_days(t.div_euclid(86_400));
    iso(y, m, d)
}

/// `HH:MM` of an ISO timestamp (as written in the record).
pub fn hhmm(iso_ts: &str) -> String {
    iso_ts.get(11..16).unwrap_or("--:--").to_string()
}

/// Age: the largest whole unit of d / h / m, else seconds.
pub fn ago(seconds: i64) -> String {
    let s = seconds.max(0);
    for (unit, n) in [("d", 86_400), ("h", 3_600), ("m", 60)] {
        if s >= n {
            return format!("{}{unit}", s / n);
        }
    }
    format!("{s}s")
}

/// A count and its noun, singular or plural.
/// The one place count wording is decided.
pub fn plural(n: i64, one: &str, many: &str) -> String {
    if n == 1 {
        format!("1 {one}")
    } else {
        format!("{} {many}", int(n))
    }
}

/// Thousands separators for a count: `1,000`.
pub fn int(n: i64) -> String {
    let neg = n < 0;
    let digits = n.unsigned_abs().to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    for (i, ch) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if neg { format!("-{out}") } else { out }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_ages() {
        let a = parse_utc("2026-10-05T07:46:00").unwrap();
        let b = parse_utc("2026-10-05T09:46Z").unwrap();
        assert_eq!(b - a, 7_200);
        assert_eq!(ago(b - a), "2h");
        assert_eq!(ago(59), "59s");
        assert_eq!(ago(3 * 86_400 + 5), "3d");
        assert_eq!(date_of(b), "2026-10-05");
        assert_eq!(hhmm("2026-10-05T07:46:00"), "07:46");
        assert_eq!(int(1000), "1,000");
        assert_eq!(int(977), "977");
        assert_eq!(int(-1234567), "-1,234,567");
        assert!(parse_utc("nope").is_none());
    }
}
