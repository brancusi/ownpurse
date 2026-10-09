//! Exact money. Every amount is a `rust_decimal::Decimal` parsed from the digits Xero sent (JSON numbers are
//! kept as text by serde_json's `arbitrary_precision`), so no amount ever passes through `f64` (invariant V11).

use rust_decimal::{Decimal, RoundingStrategy};
use serde_json::Value;
use std::str::FromStr;

pub const ZERO: Decimal = Decimal::ZERO;

/// Exact decimal from a JSON number or string. `null`, missing and `""` are zero.
/// Panics never: an unparseable value is an error the caller reports.
pub fn money(value: Option<&Value>) -> Result<Decimal, MoneyError> {
    match value {
        None | Some(Value::Null) => Ok(ZERO),
        Some(Value::String(s)) if s.is_empty() => Ok(ZERO),
        Some(Value::String(s)) => parse(s),
        Some(Value::Number(n)) => parse(&n.to_string()),
        Some(other) => Err(MoneyError(other.to_string())),
    }
}

/// Parse decimal text exactly (plain or scientific notation).
pub fn parse(text: &str) -> Result<Decimal, MoneyError> {
    let t = text.trim();
    Decimal::from_str_exact(t)
        .or_else(|_| Decimal::from_scientific(t))
        .or_else(|_| Decimal::from_str(t))
        .map_err(|_| MoneyError(text.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoneyError(pub String);

impl std::fmt::Display for MoneyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "not an exact decimal: {}", self.0)
    }
}

impl std::error::Error for MoneyError {}

/// Two decimals, banker's rounding, e.g. `"-1013.57"`.
/// A negative zero keeps its sign (`"-0.00"`).
pub fn money_str(v: Decimal) -> String {
    let q = v.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven);
    if q.is_zero() && v.is_sign_negative() {
        return "-0.00".into(); // rust_decimal drops the sign when rounding to zero; the record keeps it
    }
    format!("{q:.2}")
}

/// Decimal addition: the result keeps the larger scale of the two operands (`0.00 + 5` is `5.00`).
/// `rust_decimal` returns the other operand unchanged when one side is zero, losing the scale
/// the output shows; amounts printed with their scale must be summed with this.
pub fn add(a: Decimal, b: Decimal) -> Decimal {
    let mut r = a + b;
    let scale = a.scale().max(b.scale());
    if r.scale() < scale {
        r.rescale(scale);
    }
    r
}

/// A sum with the scale rules of `add`.
pub fn sum<I: IntoIterator<Item = Decimal>>(values: I, start: Decimal) -> Decimal {
    values.into_iter().fold(start, add)
}

/// Display form: thousands separators, two decimals, negatives in parentheses: `3,083.19` / `(3,083.19)`.
pub fn fmt(v: Decimal) -> String {
    let q = v.round_dp_with_strategy(2, RoundingStrategy::MidpointNearestEven);
    let neg = q.is_sign_negative() && !q.is_zero();
    let s = format!("{:.2}", q.abs());
    let (int, frac) = s.split_once('.').unwrap_or((&s, "00"));
    let mut grouped = String::with_capacity(int.len() + int.len() / 3);
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    if neg {
        format!("({grouped}.{frac})")
    } else {
        format!("{grouped}.{frac}")
    }
}

/// Balance form: `Dr 3,083.19`, `Cr 3,083.19`, or `0.00`.
pub fn dr_cr(v: Decimal) -> String {
    if v > ZERO {
        format!("Dr {}", fmt(v))
    } else if v < ZERO {
        format!("Cr {}", fmt(-v))
    } else {
        "0.00".into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn d(s: &str) -> Decimal {
        parse(s).unwrap()
    }

    #[test]
    fn parses_json_numbers_exactly() {
        let v: Value =
            serde_json::from_str(r#"{"a": 0.1, "b": 987654.32, "c": "", "d": null, "e": "12.5"}"#)
                .unwrap();
        assert_eq!(money(v.get("a")).unwrap(), d("0.1"));
        assert_eq!(money(v.get("b")).unwrap().to_string(), "987654.32");
        assert_eq!(money(v.get("c")).unwrap(), ZERO);
        assert_eq!(money(v.get("d")).unwrap(), ZERO);
        assert_eq!(money(v.get("missing")).unwrap(), ZERO);
        assert_eq!(money(v.get("e")).unwrap(), d("12.5"));
        assert!(money(Some(&Value::Bool(true))).is_err());
    }

    #[test]
    fn formats_amounts() {
        assert_eq!(fmt(d("1234567.891")), "1,234,567.89");
        assert_eq!(fmt(d("-987654.32")), "(987,654.32)");
        assert_eq!(fmt(d("0")), "0.00");
        assert_eq!(fmt(d("-0.004")), "0.00");
        assert_eq!(money_str(d("-1013.57")), "-1013.57");
        assert_eq!(money_str(d("-63")), "-63.00");
        assert_eq!(money_str(d("2.345")), "2.34"); // banker's rounding
        assert_eq!(money_str(d("2.355")), "2.36");
        assert_eq!(money_str(d("-0.004")), "-0.00"); // the sign of a rounded-away negative is kept
        assert_eq!(money_str(-d("0.00")), "-0.00");
        assert_eq!(money_str(d("0")), "0.00");
        assert_eq!(dr_cr(d("-5")), "Cr 5.00");
        assert_eq!(dr_cr(d("5")), "Dr 5.00");
    }

    #[test]
    fn add_keeps_the_larger_scale() {
        for (a, b, want) in [
            ("0.00", "0", "0.00"),
            ("0", "0.00", "0.00"),
            ("0.00", "5", "5.00"),
            ("5", "0.00", "5.00"),
            ("1.0", "2.00", "3.00"),
            ("5.00", "-5", "0.00"),
            ("0", "0", "0"),
        ] {
            assert_eq!(add(d(a), d(b)).to_string(), want, "{a} + {b}");
        }
        assert_eq!(sum([d("1"), d("2.5")], d("0.00")).to_string(), "3.50");
        assert_eq!(sum([], d("0.00")).to_string(), "0.00");
    }

    proptest! {
        #[test]
        fn money_str_round_trips(cents in -10_000_000_000i64..10_000_000_000i64) {
            let v = Decimal::new(cents, 2);
            prop_assert_eq!(parse(&money_str(v)).unwrap(), v);
        }

        #[test]
        fn fmt_strips_back_to_the_same_value(cents in -10_000_000_000i64..10_000_000_000i64) {
            let v = Decimal::new(cents, 2);
            let s = fmt(v);
            let neg = s.starts_with('(');
            let digits: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.').collect();
            let back = parse(&digits).unwrap();
            prop_assert_eq!(if neg { -back } else { back }, v);
        }
    }
}
