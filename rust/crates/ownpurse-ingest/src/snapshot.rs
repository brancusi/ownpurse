//! Snapshot sources: a Xero document, a sheet, a QuickBooks record. Each read says what the whole thing looks
//! like now, not what changed. A `Snapshot` collects that desired state, entity by entity, and `diff`
//! turns it into the assertions and retractions that make the log agree, so saying the same thing twice is an
//! empty transaction and an edit becomes exactly the facts that changed.
//!
//! Only attributes the projector owns are diffed (an `owns` predicate): a person's note or a link added by a
//! rule on the same entity is never retracted by a re-read of the source.

use ownpurse_facts::{Db, Eid, EntRef, Kw, TxData, ValIn, Value, ValueType};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone)]
struct Desired {
    identity: (Kw, Value),
    attrs: BTreeMap<Kw, BTreeSet<ValIn>>,
    /// false for entities the snapshot only refers to (a source, a commodity): named, never diffed
    owned: bool,
}

#[derive(Debug, Clone, Default)]
pub struct Snapshot {
    ents: BTreeMap<String, Desired>,
    order: Vec<String>,
    /// entities this read says no longer exist (an entry for a document that was voided)
    gone: Vec<(Kw, Value)>,
}

impl Snapshot {
    pub fn new() -> Snapshot {
        Snapshot::default()
    }

    /// Declare an entity by identity. `key` is the temp id the rest of the snapshot refers to it by.
    pub fn ent(&mut self, key: &str, identity: &Kw, value: Value) -> EntRef {
        if !self.ents.contains_key(key) {
            self.order.push(key.to_string());
            self.ents.insert(
                key.to_string(),
                Desired {
                    identity: (identity.clone(), value),
                    attrs: BTreeMap::new(),
                    owned: true,
                },
            );
        }
        EntRef::Temp(key.to_string())
    }

    /// Name an entity the snapshot refers to but does not describe: its identity is asserted, its other
    /// facts are left alone.
    pub fn refer(&mut self, key: &str, identity: &Kw, value: Value) -> EntRef {
        let e = self.ent(key, identity, value);
        if let Some(d) = self.ents.get_mut(key)
            && d.attrs.is_empty()
        {
            d.owned = false;
        }
        e
    }

    /// Say an entity derived from this source should not exist now; `diff` retracts it if it does.
    pub fn gone(&mut self, identity: &Kw, value: Value) {
        self.gone.push((identity.clone(), value));
    }

    pub fn has(&self, key: &str) -> bool {
        self.ents.contains_key(key)
    }

    pub fn add(&mut self, key: &str, attr: &Kw, v: impl Into<ValIn>) {
        if let Some(d) = self.ents.get_mut(key) {
            d.owned = true;
            d.attrs.entry(attr.clone()).or_default().insert(v.into());
        }
    }

    /// The transaction that makes the log say what this snapshot says, for the attributes `owns` accepts.
    pub fn diff(&self, db: &Db, owns: &dyn Fn(&Kw) -> bool) -> TxData {
        let mut tx = TxData::new();
        let existing: BTreeMap<&str, Eid> = self
            .order
            .iter()
            .filter_map(|k| {
                let d = &self.ents[k];
                db.lookup(&d.identity.0, &d.identity.1)
                    .map(|e| (k.as_str(), e))
            })
            .collect();
        for key in &self.order {
            let d = &self.ents[key];
            let e = EntRef::Temp(key.clone());
            tx.add(&e, &d.identity.0, d.identity.1.clone());
            for (a, vs) in &d.attrs {
                for v in vs {
                    tx.add(&e, a, v.clone());
                }
            }
            if !d.owned {
                continue;
            }
            let Some(&eid) = existing.get(key.as_str()) else {
                continue;
            };
            for (aid, vals) in db.entity(eid).into_iter().flatten() {
                let Some(def) = db.schema().by_id(*aid) else {
                    continue;
                };
                if def.ident == d.identity.0 || !owns(&def.ident) {
                    continue;
                }
                let mut unknown = false;
                let want: BTreeSet<Value> = d
                    .attrs
                    .get(&def.ident)
                    .into_iter()
                    .flatten()
                    .filter_map(|v| match resolve(v, def.ty, &existing, &self.ents) {
                        Resolved::Value(v) => Some(v),
                        Resolved::New => None,
                        Resolved::Unknown => {
                            unknown = true;
                            None
                        }
                    })
                    .collect();
                // a ref to an entity outside the snapshot cannot be compared before the transaction runs;
                // leave this attribute's current values alone rather than retract one that is still true
                if unknown {
                    continue;
                }
                for cur in vals.keys() {
                    if want.contains(cur) {
                        continue;
                    }
                    match (def.component, cur) {
                        (true, Value::Ref(child)) => {
                            tx.retract_entity(&EntRef::Id(*child));
                        }
                        _ => {
                            tx.retract(&EntRef::Id(eid), &def.ident, cur.clone());
                        }
                    }
                }
            }
        }
        for (a, v) in &self.gone {
            if let Some(e) = db.lookup(a, v) {
                tx.retract_entity(&EntRef::Id(e));
            }
        }
        tx
    }
}

enum Resolved {
    Value(Value),
    /// an entity this snapshot creates: equal to nothing stored
    New,
    /// a temp the snapshot does not describe
    Unknown,
}

/// A desired value as it would be stored, when that can be known before the transaction runs.
fn resolve(
    v: &ValIn,
    ty: ValueType,
    existing: &BTreeMap<&str, Eid>,
    ents: &BTreeMap<String, Desired>,
) -> Resolved {
    match v {
        ValIn::Val(val) => Resolved::Value(val.clone()),
        ValIn::Json(j) if ty != ValueType::Ref => {
            Value::from_json(ty, &j.json()).map_or(Resolved::Unknown, Resolved::Value)
        }
        ValIn::Ref(EntRef::Temp(t)) => match existing.get(t.as_str()) {
            Some(e) => Resolved::Value(Value::Ref(*e)),
            None if ents.contains_key(t) => Resolved::New,
            None => Resolved::Unknown,
        },
        ValIn::Ref(EntRef::Id(e)) => Resolved::Value(Value::Ref(*e)),
        _ => Resolved::Unknown,
    }
}
