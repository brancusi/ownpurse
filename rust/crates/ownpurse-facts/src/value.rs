//! Values: the closed set of things a fact can hold. Money is an exact decimal; there is no float.

use crate::kw::Kw;
use crate::time::{Date, Instant};
use rust_decimal::Decimal;
use serde_json::Value as Json;
use std::fmt;
use std::sync::Arc;

/// An entity id. Transactions are entities too. Ids are allocated by the log, in order.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Eid(pub u64);

impl fmt::Display for Eid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl fmt::Debug for Eid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "e{}", self.0)
    }
}

/// A SHA-256 digest: a drawer file's name.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hash(pub [u8; 32]);

impl Hash {
    pub fn hex(&self) -> String {
        self.0.iter().map(|b| format!("{b:02x}")).collect()
    }

    pub fn parse(text: &str) -> Option<Hash> {
        let t = text.strip_prefix("sha256:").unwrap_or(text);
        if t.len() != 64 {
            return None;
        }
        let mut out = [0u8; 32];
        for (i, slot) in out.iter_mut().enumerate() {
            *slot = u8::from_str_radix(t.get(i * 2..i * 2 + 2)?, 16).ok()?;
        }
        Some(Hash(out))
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sha256:{}", self.hex())
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "sha256:{}…", &self.hex()[..12])
    }
}

/// The type an attribute's values have. The tag is part of the canonical encoding: never renumber.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueType {
    Ref,
    Str,
    Int,
    Dec,
    Bool,
    Date,
    Instant,
    Kw,
    Hash,
}

impl ValueType {
    pub const ALL: [ValueType; 9] = [
        ValueType::Ref,
        ValueType::Str,
        ValueType::Int,
        ValueType::Dec,
        ValueType::Bool,
        ValueType::Date,
        ValueType::Instant,
        ValueType::Kw,
        ValueType::Hash,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ValueType::Ref => "ref",
            ValueType::Str => "string",
            ValueType::Int => "int",
            ValueType::Dec => "decimal",
            ValueType::Bool => "bool",
            ValueType::Date => "date",
            ValueType::Instant => "instant",
            ValueType::Kw => "keyword",
            ValueType::Hash => "hash",
        }
    }

    pub fn from_name(name: &str) -> Option<ValueType> {
        ValueType::ALL.into_iter().find(|t| t.name() == name)
    }

    pub fn tag(self) -> u8 {
        match self {
            ValueType::Ref => 1,
            ValueType::Str => 2,
            ValueType::Int => 3,
            ValueType::Dec => 4,
            ValueType::Bool => 5,
            ValueType::Date => 6,
            ValueType::Instant => 7,
            ValueType::Kw => 8,
            ValueType::Hash => 9,
        }
    }

    pub fn from_tag(tag: u8) -> Option<ValueType> {
        ValueType::ALL.into_iter().find(|t| t.tag() == tag)
    }
}

/// One value. Decimals compare by amount (`1.10 == 1.1`) but keep the scale they were given.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Ref(Eid),
    Str(Arc<str>),
    Int(i64),
    Dec(Decimal),
    Bool(bool),
    Date(Date),
    Instant(Instant),
    Kw(Kw),
    Hash(Hash),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValueError(pub String);

impl fmt::Display for ValueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ValueError {}

/// Exact decimal from text: plain digits only (no exponent, no thousands separators, no currency sign).
pub fn parse_decimal(text: &str) -> Result<Decimal, ValueError> {
    let t = text.trim();
    let plain = !t.is_empty()
        && t.trim_start_matches('-')
            .chars()
            .all(|c| c.is_ascii_digit() || c == '.')
        && t.matches('.').count() <= 1
        && t != "-"
        && t != ".";
    if !plain {
        return Err(ValueError(format!("not a plain decimal: {text:?}")));
    }
    Decimal::from_str_exact(t)
        .map_err(|e| ValueError(format!("not an exact decimal ({e}): {text:?}")))
}

impl Value {
    pub fn str(s: &str) -> Value {
        Value::Str(Arc::from(s))
    }

    pub fn value_type(&self) -> ValueType {
        match self {
            Value::Ref(_) => ValueType::Ref,
            Value::Str(_) => ValueType::Str,
            Value::Int(_) => ValueType::Int,
            Value::Dec(_) => ValueType::Dec,
            Value::Bool(_) => ValueType::Bool,
            Value::Date(_) => ValueType::Date,
            Value::Instant(_) => ValueType::Instant,
            Value::Kw(_) => ValueType::Kw,
            Value::Hash(_) => ValueType::Hash,
        }
    }

    pub fn as_ref(&self) -> Option<Eid> {
        match self {
            Value::Ref(e) => Some(*e),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Value::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_kw(&self) -> Option<&Kw> {
        match self {
            Value::Kw(k) => Some(k),
            _ => None,
        }
    }

    pub fn as_dec(&self) -> Option<Decimal> {
        match self {
            Value::Dec(d) => Some(*d),
            _ => None,
        }
    }

    /// A non-ref value from JSON, checked against the attribute's type. Decimals come as strings or JSON
    /// numbers (kept exact by `arbitrary_precision`); hashes as hex, with or without `sha256:`.
    pub fn from_json(ty: ValueType, j: &Json) -> Result<Value, ValueError> {
        let want = || ValueError(format!("expected a {} value, got {j}", ty.name()));
        let text = || j.as_str().ok_or_else(want);
        Ok(match ty {
            ValueType::Ref => return Err(want()),
            ValueType::Str => Value::str(text()?),
            ValueType::Int => Value::Int(j.as_i64().ok_or_else(want)?),
            ValueType::Bool => Value::Bool(j.as_bool().ok_or_else(want)?),
            ValueType::Dec => match j {
                Json::String(s) => Value::Dec(parse_decimal(s)?),
                Json::Number(n) => Value::Dec(parse_decimal(&n.to_string())?),
                _ => return Err(want()),
            },
            ValueType::Date => Value::Date(Date::parse(text()?).map_err(|e| ValueError(e.0))?),
            ValueType::Instant => {
                Value::Instant(Instant::parse(text()?).map_err(|e| ValueError(e.0))?)
            }
            ValueType::Kw => Value::Kw(Kw::new(text()?).map_err(|e| ValueError(e.to_string()))?),
            ValueType::Hash => Value::Hash(
                Hash::parse(text()?)
                    .ok_or_else(|| ValueError(format!("not a SHA-256 hex digest: {j}")))?,
            ),
        })
    }

    /// JSON for output: refs as `{"db/id": n}`, decimals as strings (never JSON numbers).
    pub fn to_json(&self) -> Json {
        match self {
            Value::Ref(e) => serde_json::json!({ "db/id": e.0 }),
            Value::Int(i) => Json::from(*i),
            Value::Bool(b) => Json::Bool(*b),
            other => Json::String(other.text()),
        }
    }

    /// Plain text: what SQL views and the CLI show.
    pub fn text(&self) -> String {
        match self {
            Value::Ref(e) => e.0.to_string(),
            Value::Str(s) => s.to_string(),
            Value::Int(i) => i.to_string(),
            Value::Dec(d) => d.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Date(d) => d.to_string(),
            Value::Instant(i) => i.to_string(),
            Value::Kw(k) => k.to_string(),
            Value::Hash(h) => h.to_string(),
        }
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Ref(e) => write!(f, "{e:?}"),
            Value::Str(s) => write!(f, "{s:?}"),
            Value::Kw(k) => write!(f, "{k:?}"),
            Value::Hash(h) => write!(f, "{h:?}"),
            Value::Date(d) => write!(f, "{d:?}"),
            Value::Instant(i) => write!(f, "{i:?}"),
            other => f.write_str(&other.text()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn decimals_are_exact_and_keep_their_scale() {
        let v = Value::from_json(ValueType::Dec, &json!("186.40")).unwrap();
        assert_eq!(v.text(), "186.40");
        assert_eq!(v, Value::Dec(Decimal::new(1864, 1)));
        let n: Json = serde_json::from_str("0.1000000000000000055511151231257827").unwrap();
        // more digits than 28 is refused, never rounded
        assert!(Value::from_json(ValueType::Dec, &n).is_err());
        for bad in ["1e3", "1,000.00", "$5", "", "-", "1.2.3", "NaN"] {
            assert!(parse_decimal(bad).is_err(), "{bad}");
        }
    }

    #[test]
    fn values_are_checked_against_their_type() {
        assert!(Value::from_json(ValueType::Int, &json!("3")).is_err());
        assert!(Value::from_json(ValueType::Date, &json!("2026-02-30")).is_err());
        assert!(Value::from_json(ValueType::Kw, &json!("status/held")).is_ok());
        assert!(Value::from_json(ValueType::Ref, &json!(3)).is_err());
        let h = "ab".repeat(32);
        assert_eq!(
            Value::from_json(ValueType::Hash, &json!(format!("sha256:{h}")))
                .unwrap()
                .text(),
            format!("sha256:{h}")
        );
    }

    #[test]
    fn value_type_tags_are_stable() {
        let tags: Vec<u8> = ValueType::ALL.iter().map(|t| t.tag()).collect();
        assert_eq!(tags, vec![1, 2, 3, 4, 5, 6, 7, 8, 9]);
        for t in ValueType::ALL {
            assert_eq!(ValueType::from_tag(t.tag()), Some(t));
            assert_eq!(ValueType::from_name(t.name()), Some(t));
        }
    }
}
