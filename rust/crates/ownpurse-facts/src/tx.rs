//! Transaction data: what a person, an agent or a projector asks the log to add. Pure data; `Db::with`
//! resolves it against a database value.
//!
//! The JSON form (what `ownpurse-facts transact` and the API accept):
//!
//! ```json
//! {"meta": {"tx/actor": "agent:tally", "tx/reason": "receipt for the client dinner"},
//!  "ops": [
//!    {"db/id": "r", "obs/kind": "obs.kind/receipt", "obs/amount": "186.40",
//!     "obs/region": {"region/locator": "page:1"}},
//!    ["add", 1042, "note/text", "signed by the office manager"],
//!    ["retract", 1042, "obs/amount", "186.40"],
//!    ["retractEntity", {"lookup": ["obs/id", "bank:…"]}]]}
//! ```
//!
//! Entity references: a number is an entity id, a string is a temporary id local to the transaction,
//! `{"lookup": [attr, value]}` finds an entity by a unique attribute, and `"tx"` is the transaction itself.
//! A ref-typed value may also be a nested map, which becomes a new entity (a component, usually).
//! Arrays always mean several values; they are never a lookup.

use crate::kw::Kw;
use crate::value::{Eid, Value};
use serde_json::{Map, Value as Json};
use std::fmt;

/// Which entity an operation is about.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntRef {
    Id(Eid),
    Temp(String),
    Lookup(Kw, Box<ValIn>),
    /// The transaction being built.
    Tx,
}

/// A value before resolution: a final value, a reference to resolve, or JSON to check against the schema.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValIn {
    Val(Value),
    Ref(EntRef),
    Json(JsonText),
}

/// JSON kept as its canonical text so `ValIn` can be ordered and compared.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct JsonText(pub String);

impl JsonText {
    pub fn of(j: &Json) -> JsonText {
        JsonText(j.to_string())
    }

    pub fn json(&self) -> Json {
        serde_json::from_str(&self.0).unwrap_or(Json::Null)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    Add(EntRef, Kw, ValIn),
    Retract(EntRef, Kw, ValIn),
    /// Retract every fact about the entity, its components, and every ref to it.
    RetractEntity(EntRef),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TxData {
    pub ops: Vec<Op>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError(pub String);

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ParseError {}

impl From<Value> for ValIn {
    fn from(v: Value) -> ValIn {
        ValIn::Val(v)
    }
}

impl From<EntRef> for ValIn {
    fn from(e: EntRef) -> ValIn {
        ValIn::Ref(e)
    }
}

impl EntRef {
    pub fn temp(name: impl Into<String>) -> EntRef {
        EntRef::Temp(name.into())
    }

    pub fn lookup(attr: &Kw, v: impl Into<ValIn>) -> EntRef {
        EntRef::Lookup(attr.clone(), Box::new(v.into()))
    }
}

impl TxData {
    pub fn new() -> TxData {
        TxData::default()
    }

    pub fn add(&mut self, e: &EntRef, a: &Kw, v: impl Into<ValIn>) -> &mut Self {
        self.ops.push(Op::Add(e.clone(), a.clone(), v.into()));
        self
    }

    pub fn retract(&mut self, e: &EntRef, a: &Kw, v: impl Into<ValIn>) -> &mut Self {
        self.ops.push(Op::Retract(e.clone(), a.clone(), v.into()));
        self
    }

    pub fn retract_entity(&mut self, e: &EntRef) -> &mut Self {
        self.ops.push(Op::RetractEntity(e.clone()));
        self
    }

    /// Transaction metadata: facts about the transaction entity itself.
    pub fn meta(&mut self, a: &Kw, v: impl Into<ValIn>) -> &mut Self {
        self.add(&EntRef::Tx, a, v)
    }

    pub fn extend(&mut self, other: TxData) -> &mut Self {
        self.ops.extend(other.ops);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Parse the JSON form documented at the top of this module.
    pub fn from_json(j: &Json) -> Result<TxData, ParseError> {
        let obj = j.as_object().ok_or_else(|| {
            ParseError("a transaction is an object with \"ops\" (and optional \"meta\")".into())
        })?;
        for k in obj.keys() {
            if k != "ops" && k != "meta" {
                return Err(ParseError(format!(
                    "unknown transaction key {k:?} (want \"ops\" or \"meta\")"
                )));
            }
        }
        let mut p = Parser {
            tx: TxData::new(),
            next_temp: 0,
        };
        if let Some(meta) = obj.get("meta") {
            let m = meta
                .as_object()
                .ok_or_else(|| ParseError("\"meta\" is an object of attribute → value".into()))?;
            p.map_body(&EntRef::Tx, m)?;
        }
        let ops = obj.get("ops").and_then(Json::as_array).ok_or_else(|| {
            ParseError("\"ops\" is an array of maps and [op, e, a, v] lists".into())
        })?;
        for (i, op) in ops.iter().enumerate() {
            p.op(op)
                .map_err(|e| ParseError(format!("ops[{i}]: {}", e.0)))?;
        }
        Ok(p.tx)
    }
}

struct Parser {
    tx: TxData,
    next_temp: usize,
}

fn kw(text: &str) -> Result<Kw, ParseError> {
    Kw::new(text).map_err(|e| ParseError(e.to_string()))
}

/// An entity reference in JSON: number, temp string, `"tx"`, or `{"lookup": [attr, value]}`.
pub fn ent_ref(j: &Json) -> Result<EntRef, ParseError> {
    match j {
        Json::Number(n) => n
            .as_u64()
            .map(|n| EntRef::Id(Eid(n)))
            .ok_or_else(|| ParseError(format!("an entity id is a positive integer, got {n}"))),
        Json::String(s) if s == "tx" => Ok(EntRef::Tx),
        Json::String(s) if !s.is_empty() => Ok(EntRef::Temp(s.clone())),
        Json::Object(m) if m.len() == 1 && m.contains_key("lookup") => {
            match m.get("lookup").and_then(Json::as_array).map(Vec::as_slice) {
                Some([Json::String(a), v]) => Ok(EntRef::Lookup(kw(a)?, Box::new(val_in(v)?))),
                _ => Err(ParseError("a lookup is {\"lookup\": [attr, value]}".into())),
            }
        }
        _ => Err(ParseError(format!("not an entity reference: {j}"))),
    }
}

/// A scalar value in JSON, kept for the schema to check. Lookup objects are refs.
fn val_in(j: &Json) -> Result<ValIn, ParseError> {
    match j {
        Json::Object(m) if m.contains_key("lookup") => Ok(ValIn::Ref(ent_ref(j)?)),
        Json::Array(_) => Err(ParseError("a list is not one value".into())),
        _ => Ok(ValIn::Json(JsonText::of(j))),
    }
}

impl Parser {
    fn op(&mut self, j: &Json) -> Result<(), ParseError> {
        match j {
            Json::Object(m) => {
                let e = match m.get("db/id") {
                    Some(id) => ent_ref(id)?,
                    None => self.fresh(),
                };
                self.map_body(&e, m)
            }
            Json::Array(parts) => match parts.as_slice() {
                [Json::String(op), e, Json::String(a), v] if op == "add" || op == "retract" => {
                    let (e, a) = (ent_ref(e)?, kw(a)?);
                    let v = self.value(v)?;
                    self.tx.ops.push(if op == "add" {
                        Op::Add(e, a, v)
                    } else {
                        Op::Retract(e, a, v)
                    });
                    Ok(())
                }
                [Json::String(op), e] if op == "retractEntity" => {
                    self.tx.ops.push(Op::RetractEntity(ent_ref(e)?));
                    Ok(())
                }
                _ => Err(ParseError(
                    "a list op is [\"add\"|\"retract\", e, attr, value] or [\"retractEntity\", e]"
                        .into(),
                )),
            },
            _ => Err(ParseError(format!("not an op: {j}"))),
        }
    }

    fn fresh(&mut self) -> EntRef {
        self.next_temp += 1;
        EntRef::Temp(format!("_{}", self.next_temp))
    }

    fn map_body(&mut self, e: &EntRef, m: &Map<String, Json>) -> Result<(), ParseError> {
        for (k, v) in m {
            if k == "db/id" {
                continue;
            }
            let a = kw(k)?;
            let values: Vec<&Json> = match v {
                Json::Array(items) => items.iter().collect(),
                one => vec![one],
            };
            for v in values {
                let v = self.value(v)?;
                self.tx.ops.push(Op::Add(e.clone(), a.clone(), v));
            }
        }
        Ok(())
    }

    /// A value position: a nested map (without `lookup`) is a new entity; the rest is `val_in`.
    fn value(&mut self, j: &Json) -> Result<ValIn, ParseError> {
        match j {
            Json::Object(m) if !m.contains_key("lookup") => {
                let child = match m.get("db/id") {
                    Some(id) => ent_ref(id)?,
                    None => self.fresh(),
                };
                self.map_body(&child, m)?;
                Ok(ValIn::Ref(child))
            }
            _ => val_in(j),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn the_documented_example_parses() {
        let tx = TxData::from_json(&json!({
            "meta": {"tx/actor": "agent:tally"},
            "ops": [
                {"db/id": "r", "obs/amount": "186.40", "obs/region": {"region/locator": "page:1"}},
                ["add", 1042, "note/text", "signed"],
                ["retract", 1042, "obs/amount", "186.40"],
                ["retractEntity", {"lookup": ["obs/id", "bank:1"]}]
            ]
        }))
        .unwrap();
        assert_eq!(tx.ops.len(), 7);
        assert!(matches!(&tx.ops[0], Op::Add(EntRef::Tx, a, _) if a.as_str() == "tx/actor"));
        // the nested region becomes its own temp entity, referenced from "r"
        assert!(tx.ops.iter().any(|o| matches!(o, Op::Add(EntRef::Temp(t), a, _) if t == "_1" && a.as_str() == "region/locator")));
        assert!(tx.ops.iter().any(|o| matches!(o, Op::Add(EntRef::Temp(t), a, ValIn::Ref(EntRef::Temp(c))) if t == "r" && a.as_str() == "obs/region" && c == "_1")));
        assert!(
            matches!(&tx.ops[6], Op::RetractEntity(EntRef::Lookup(a, _)) if a.as_str() == "obs/id")
        );
    }

    #[test]
    fn arrays_are_many_values_never_a_lookup() {
        let tx =
            TxData::from_json(&json!({"ops": [{"db/id": "x", "tag/name": ["a", "b"]}]})).unwrap();
        assert_eq!(tx.ops.len(), 2);
        assert!(TxData::from_json(&json!({"ops": [["add", "x", "tag/name", ["a"]]]})).is_err());
    }

    #[test]
    fn errors_name_the_op() {
        let e = TxData::from_json(&json!({"ops": [{"amount": 1}]})).unwrap_err();
        assert!(e.0.starts_with("ops[0]: "), "{}", e.0);
        assert!(TxData::from_json(&json!({"opz": []})).is_err());
    }
}
