//! Calendar dates and UTC instants as validated, fixed-format text. Fixed formats make text order equal time
//! order, so both sort and index as strings. No clock is read here: callers pass time in.

use std::fmt;
use std::sync::Arc;

/// A calendar date, `YYYY-MM-DD`, with no time zone: the business date a source reports.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Date(Arc<str>);

/// A moment in UTC, `YYYY-MM-DDTHH:MM:SS.ffffffZ` (always six fraction digits).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Instant(Arc<str>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeError(pub String);

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for TimeError {}

fn digits(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn leap(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

fn month_days(y: i64, m: u32) -> u32 {
    match m {
        2 if leap(y) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (i64::from(m) + 9) % 12;
    let doy = (153 * mp + 2) / 5 + i64::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// The civil date of a day count since 1970-01-01.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = u32::try_from(doy - (153 * mp + 2) / 5 + 1).unwrap_or(1);
    let m = u32::try_from(if mp < 10 { mp + 3 } else { mp - 9 }).unwrap_or(1);
    (if m <= 2 { y + 1 } else { y }, m, d)
}

impl Date {
    pub fn parse(text: &str) -> Result<Date, TimeError> {
        let bad = || TimeError(format!("not a date (want YYYY-MM-DD): {text:?}"));
        let b = text.as_bytes();
        if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
            return Err(bad());
        }
        let (y, m, d) = (
            digits(&text[0..4]).ok_or_else(bad)?,
            digits(&text[5..7]).ok_or_else(bad)?,
            digits(&text[8..10]).ok_or_else(bad)?,
        );
        if !(1..=12).contains(&m) || d == 0 || d > month_days(i64::from(y), m) {
            return Err(bad());
        }
        Ok(Date(Arc::from(text)))
    }

    pub fn from_days(days: i64) -> Date {
        let (y, m, d) = civil_from_days(days);
        Date(Arc::from(format!("{y:04}-{m:02}-{d:02}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Instant {
    /// RFC 3339 in UTC (`Z` or `+00:00`), with or without fraction digits (up to six).
    pub fn parse(text: &str) -> Result<Instant, TimeError> {
        let bad = || {
            TimeError(format!(
                "not a UTC instant (want YYYY-MM-DDTHH:MM:SS[.ffffff]Z): {text:?}"
            ))
        };
        let body = text
            .strip_suffix('Z')
            .or_else(|| text.strip_suffix("+00:00"))
            .ok_or_else(bad)?;
        if body.len() < 19 || body.as_bytes()[10] != b'T' {
            return Err(bad());
        }
        let date = Date::parse(&body[..10]).map_err(|_| bad())?;
        let t = &body[11..19];
        let tb = t.as_bytes();
        if tb[2] != b':' || tb[5] != b':' {
            return Err(bad());
        }
        let (h, mi, s) = (
            digits(&t[0..2]).ok_or_else(bad)?,
            digits(&t[3..5]).ok_or_else(bad)?,
            digits(&t[6..8]).ok_or_else(bad)?,
        );
        if h > 23 || mi > 59 || s > 59 {
            return Err(bad());
        }
        let frac = match &body[19..] {
            "" => String::new(),
            f => {
                let f = f.strip_prefix('.').ok_or_else(bad)?;
                if f.is_empty() || f.len() > 6 || digits(f).is_none() {
                    return Err(bad());
                }
                f.to_string()
            }
        };
        Ok(Instant(Arc::from(format!(
            "{}T{h:02}:{mi:02}:{s:02}.{frac:0<6}Z",
            date.as_str()
        ))))
    }

    /// RFC 3339 with any offset (`-08:00`), converted to UTC. The offset itself is not kept: callers that
    /// need the local date must keep the original text too.
    pub fn parse_offset(text: &str) -> Result<Instant, TimeError> {
        if let Ok(i) = Instant::parse(text) {
            return Ok(i);
        }
        let bad = || TimeError(format!("not an RFC 3339 instant: {text:?}"));
        if text.len() < 25 {
            return Err(bad());
        }
        let (body, off) = text.split_at(text.len() - 6);
        let ob = off.as_bytes();
        let sign = match ob[0] {
            b'+' => 1i64,
            b'-' => -1,
            _ => return Err(bad()),
        };
        if ob[3] != b':' {
            return Err(bad());
        }
        let (oh, om) = (
            digits(&off[1..3]).ok_or_else(bad)?,
            digits(&off[4..6]).ok_or_else(bad)?,
        );
        let utc = Instant::parse(&format!("{body}Z"))?;
        let s = utc.as_str();
        let (y, mo, d) = (
            i64::from(digits(&s[0..4]).ok_or_else(bad)?),
            digits(&s[5..7]).ok_or_else(bad)?,
            digits(&s[8..10]).ok_or_else(bad)?,
        );
        let (h, mi, sec) = (
            i64::from(digits(&s[11..13]).ok_or_else(bad)?),
            i64::from(digits(&s[14..16]).ok_or_else(bad)?),
            i64::from(digits(&s[17..19]).ok_or_else(bad)?),
        );
        let micros = i64::from(digits(&s[20..26]).ok_or_else(bad)?);
        let secs = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + sec
            - sign * (i64::from(oh) * 3600 + i64::from(om) * 60);
        let base = Instant::from_unix_millis(secs * 1000);
        Ok(Instant(Arc::from(format!(
            "{}{micros:06}Z",
            &base.as_str()[..20]
        ))))
    }

    /// From milliseconds since the Unix epoch (Xero's `/Date(ms)/`).
    pub fn from_unix_millis(ms: i64) -> Instant {
        let (days, rem) = (ms.div_euclid(86_400_000), ms.rem_euclid(86_400_000));
        let (h, rem) = (rem / 3_600_000, rem % 3_600_000);
        let (mi, rem) = (rem / 60_000, rem % 60_000);
        let (s, milli) = (rem / 1000, rem % 1000);
        Instant(Arc::from(format!(
            "{}T{h:02}:{mi:02}:{s:02}.{:06}Z",
            Date::from_days(days).as_str(),
            milli * 1000
        )))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn date(&self) -> Date {
        Date(Arc::from(&self.0[..10]))
    }
}

macro_rules! text_fmt {
    ($t:ty) => {
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
        impl fmt::Debug for $t {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "#{}", &self.0)
            }
        }
    };
}
// Display and Debug are the same three lines for both text-backed time types.
text_fmt!(Date);
text_fmt!(Instant);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_are_checked_against_the_calendar() {
        assert!(Date::parse("2024-02-29").is_ok());
        for bad in [
            "2025-02-29",
            "2025-13-01",
            "2025-00-10",
            "2025-1-01",
            "25-01-01",
            "2025-04-31",
        ] {
            assert!(Date::parse(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn instants_normalise_to_six_fraction_digits() {
        assert_eq!(
            Instant::parse("2026-03-14T09:30:00Z").unwrap().as_str(),
            "2026-03-14T09:30:00.000000Z"
        );
        assert_eq!(
            Instant::parse("2026-03-14T09:30:00.25+00:00")
                .unwrap()
                .as_str(),
            "2026-03-14T09:30:00.250000Z"
        );
        assert!(Instant::parse("2026-03-14T09:30:00-05:00").is_err());
        assert_eq!(
            Instant::parse_offset("2026-03-14T21:30:00.5-05:00")
                .unwrap()
                .as_str(),
            "2026-03-15T02:30:00.500000Z"
        );
        assert_eq!(
            Instant::parse_offset("2026-03-01T00:15:00+01:00")
                .unwrap()
                .as_str(),
            "2026-02-28T23:15:00.000000Z"
        );
        assert!(Instant::parse("2026-03-14 09:30:00Z").is_err());
    }

    #[test]
    fn unix_millis_round_trip_through_the_civil_calendar() {
        assert_eq!(
            Instant::from_unix_millis(1_717_200_000_000).as_str(),
            "2024-06-01T00:00:00.000000Z"
        );
        assert_eq!(
            Instant::from_unix_millis(-1).as_str(),
            "1969-12-31T23:59:59.999000Z"
        );
        for d in [-800_000, -1, 0, 1, 19_000, 2_000_000] {
            let (y, m, dd) = civil_from_days(d);
            assert_eq!(days_from_civil(y, m, dd), d);
        }
    }
}
