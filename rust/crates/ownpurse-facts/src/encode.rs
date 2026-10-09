//! The canonical encoding of datoms and the transaction hash chain. The bytes are frozen by test vectors:
//! changing them is a new encoding version, never an edit.
//!
//! datom   = e:u64be a:u64be tag:u8 value added:u8
//! value   = ref u64be | int i64be | bool u8 | hash 32 bytes | text (string, decimal, date, instant, keyword)
//! text    = len:u32be utf8          (a decimal is its text with its scale: "186.40")
//! tx hash = sha256("ownpurse.tx.v1" 0x00 prev:32 tx:u64be count:u32be datom*)   datoms in sorted order

use crate::db::Datom;
use crate::value::{Eid, Hash, Value};
use sha2::{Digest, Sha256};

pub const DOMAIN: &[u8] = b"ownpurse.tx.v1\0";

fn text(out: &mut Vec<u8>, s: &str) {
    let n = u32::try_from(s.len()).unwrap_or(u32::MAX);
    out.extend_from_slice(&n.to_be_bytes());
    out.extend_from_slice(s.as_bytes());
}

pub fn value_bytes(out: &mut Vec<u8>, v: &Value) {
    out.push(v.value_type().tag());
    match v {
        Value::Ref(e) => out.extend_from_slice(&e.0.to_be_bytes()),
        Value::Int(i) => out.extend_from_slice(&i.to_be_bytes()),
        Value::Bool(b) => out.push(u8::from(*b)),
        Value::Hash(h) => out.extend_from_slice(&h.0),
        Value::Str(s) => text(out, s),
        Value::Dec(d) => text(out, &d.to_string()),
        Value::Date(d) => text(out, d.as_str()),
        Value::Instant(i) => text(out, i.as_str()),
        Value::Kw(k) => text(out, k.as_str()),
    }
}

pub fn datom_bytes(out: &mut Vec<u8>, d: &Datom) {
    out.extend_from_slice(&d.e.0.to_be_bytes());
    out.extend_from_slice(&d.a.0.to_be_bytes());
    value_bytes(out, &d.v);
    out.push(u8::from(d.added));
}

/// The hash of one transaction, chained to the previous one. `datoms` may come in any order.
pub fn tx_hash(prev: &Hash, tx: Eid, datoms: &[Datom]) -> Hash {
    let mut sorted: Vec<&Datom> = datoms.iter().collect();
    sorted.sort();
    let mut buf = Vec::with_capacity(64 + datoms.len() * 48);
    buf.extend_from_slice(DOMAIN);
    buf.extend_from_slice(&prev.0);
    buf.extend_from_slice(&tx.0.to_be_bytes());
    buf.extend_from_slice(
        &u32::try_from(sorted.len())
            .unwrap_or(u32::MAX)
            .to_be_bytes(),
    );
    for d in sorted {
        datom_bytes(&mut buf, d);
    }
    Hash(Sha256::digest(&buf).into())
}

/// SHA-256 of a byte string: a drawer file's name.
pub fn sha256(bytes: &[u8]) -> Hash {
    Hash(Sha256::digest(bytes).into())
}

pub const ZERO_HASH: Hash = Hash([0; 32]);
