//! Values in and out of SQLite, and `dsum()`: exact decimal sums for people querying the log by hand.

use crate::StoreError;
use ownpurse_facts::value::parse_decimal;
use ownpurse_facts::{Date, Eid, Hash, Instant, Kw, Value, ValueType};
use rusqlite::Connection;
use rusqlite::functions::{Aggregate, Context, FunctionFlags};
use rusqlite::types::Value as Sql;
use rust_decimal::Decimal;

pub fn id(e: Eid) -> i64 {
    i64::try_from(e.0).unwrap_or(i64::MAX)
}

pub fn eid(n: i64) -> Result<Eid, StoreError> {
    u64::try_from(n)
        .map(Eid)
        .map_err(|_| StoreError::Corrupt(format!("negative entity id {n}")))
}

/// Refs, ints and bools are INTEGER; hashes are `sha256:<hex>` text; the rest is their text.
pub fn to_sql(v: &Value) -> Sql {
    match v {
        Value::Ref(e) => Sql::Integer(id(*e)),
        Value::Int(i) => Sql::Integer(*i),
        Value::Bool(b) => Sql::Integer(i64::from(*b)),
        other => Sql::Text(other.text()),
    }
}

pub fn from_sql(v: &Sql, tag: u8) -> Result<Value, String> {
    let ty = ValueType::from_tag(tag).ok_or_else(|| format!("unknown value type tag {tag}"))?;
    let bad = || format!("a {} value stored as {v:?}", ty.name());
    Ok(match (ty, v) {
        (ValueType::Ref, Sql::Integer(n)) => Value::Ref(Eid(u64::try_from(*n).map_err(|_| bad())?)),
        (ValueType::Int, Sql::Integer(n)) => Value::Int(*n),
        (ValueType::Bool, Sql::Integer(n)) => Value::Bool(*n != 0),
        (ValueType::Str, Sql::Text(s)) => Value::str(s),
        (ValueType::Dec, Sql::Text(s)) => Value::Dec(parse_decimal(s).map_err(|e| e.0)?),
        (ValueType::Date, Sql::Text(s)) => Value::Date(Date::parse(s).map_err(|e| e.0)?),
        (ValueType::Instant, Sql::Text(s)) => Value::Instant(Instant::parse(s).map_err(|e| e.0)?),
        (ValueType::Kw, Sql::Text(s)) => Value::Kw(Kw::new(s).map_err(|e| e.to_string())?),
        (ValueType::Hash, Sql::Text(s)) => Value::Hash(Hash::parse(s).ok_or_else(bad)?),
        _ => return Err(bad()),
    })
}

struct DSum;

impl Aggregate<Option<Decimal>, Option<String>> for DSum {
    fn init(&self, _: &mut Context<'_>) -> rusqlite::Result<Option<Decimal>> {
        Ok(None)
    }

    fn step(&self, ctx: &mut Context<'_>, acc: &mut Option<Decimal>) -> rusqlite::Result<()> {
        let text: Option<String> = match ctx.get_raw(0) {
            rusqlite::types::ValueRef::Null => None,
            rusqlite::types::ValueRef::Text(t) => Some(String::from_utf8_lossy(t).into_owned()),
            rusqlite::types::ValueRef::Integer(i) => Some(i.to_string()),
            _ => {
                return Err(rusqlite::Error::UserFunctionError(
                    "dsum takes decimal text; a REAL would already have lost cents".into(),
                ));
            }
        };
        if let Some(t) = text {
            let d =
                parse_decimal(&t).map_err(|e| rusqlite::Error::UserFunctionError(e.0.into()))?;
            let sum = acc.unwrap_or(Decimal::ZERO) + d;
            *acc = Some(sum);
        }
        Ok(())
    }

    fn finalize(
        &self,
        _: &mut Context<'_>,
        acc: Option<Option<Decimal>>,
    ) -> rusqlite::Result<Option<String>> {
        Ok(acc.flatten().map(|d| d.to_string()))
    }
}

/// Register `dsum(text)`: an exact decimal sum, returned as text. SQLite's own `sum()` converts decimal text
/// to floating point and can be off by fractions of a cent; use `dsum` for money.
pub fn register(conn: &Connection) -> rusqlite::Result<()> {
    conn.create_aggregate_function(
        "dsum",
        1,
        FunctionFlags::SQLITE_UTF8 | FunctionFlags::SQLITE_DETERMINISTIC,
        DSum,
    )
}
