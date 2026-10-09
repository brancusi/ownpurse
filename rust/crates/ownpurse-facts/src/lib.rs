//! The fact model. Everything ownpurse knows is a datom `[e a v tx added]`; a transaction stamps a batch of
//! them; the schema is facts too; a `Db` is the log up to one transaction, as a value.
//!
//! Pure: no file, socket, process or clock. The store (`ownpurse-store`) writes the log, the projectors
//! (`ownpurse-ingest`) turn source files into transactions.

pub mod db;
pub mod encode;
pub mod kw;
pub mod pull;
pub mod schema;
pub mod time;
pub mod tx;
pub mod value;

pub use db::{Datom, Db, TxError, TxReport, genesis_datoms};
pub use kw::Kw;
pub use pull::{Pattern, pull};
pub use schema::{AttrDef, Schema, Unique};
pub use time::{Date, Instant};
pub use tx::{EntRef, Op, TxData, ValIn};
pub use value::{Eid, Hash, Value, ValueType};

/// Install attributes from their compact JSON definitions (the form `schema/core.json` uses):
/// `[{"ident": "obs/amount", "type": "decimal", "many": false, "unique": "identity", "component": false,
///    "doc": "…"}]`; only `ident` and `type` are required. Returns the transaction data; nothing is written.
pub fn install(defs: &serde_json::Value) -> Result<TxData, String> {
    use serde_json::Value as Json;
    let items = defs
        .as_array()
        .ok_or("attribute definitions are an array of objects")?;
    let mut tx = TxData::new();
    let a = Kw::lit;
    for (i, d) in items.iter().enumerate() {
        let at = |msg: &str| format!("definition {i}: {msg}");
        let s = |k: &str| d.get(k).and_then(Json::as_str);
        let ident = Kw::new(s("ident").ok_or_else(|| at("needs \"ident\""))?)
            .map_err(|e| at(&e.to_string()))?;
        let ty = s("type").and_then(ValueType::from_name).ok_or_else(|| {
            at("needs a \"type\" (ref, string, int, decimal, bool, date, instant, keyword, hash)")
        })?;
        let e = EntRef::temp(format!("attr:{ident}"));
        tx.add(&e, &a("db/ident"), Value::Kw(ident.clone()));
        tx.add(&e, &a("db/valueType"), Value::Kw(schema::type_kw(ty)));
        let many = d.get("many").and_then(Json::as_bool).unwrap_or(false);
        tx.add(
            &e,
            &a("db/cardinality"),
            Value::Kw(schema::cardinality_kw(many)),
        );
        match s("unique") {
            None => {}
            Some("identity") => {
                tx.add(
                    &e,
                    &a("db/unique"),
                    Value::Kw(schema::unique_kw(Unique::Identity)),
                );
            }
            Some("value") => {
                tx.add(
                    &e,
                    &a("db/unique"),
                    Value::Kw(schema::unique_kw(Unique::Value)),
                );
            }
            Some(other) => {
                return Err(at(&format!(
                    "unique is \"identity\" or \"value\", not {other:?}"
                )));
            }
        }
        if d.get("component").and_then(Json::as_bool).unwrap_or(false) {
            tx.add(&e, &a("db/isComponent"), Value::Bool(true));
        }
        if let Some(doc) = s("doc") {
            tx.add(&e, &a("db/doc"), Value::str(doc));
        }
    }
    Ok(tx)
}

/// The shared vocabulary every source is translated into (`schema/core.json`).
pub const CORE_SCHEMA: &str = include_str!("../schema/core.json");
