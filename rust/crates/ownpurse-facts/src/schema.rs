//! Schema as data. An attribute is an entity with `db/ident`, `db/valueType` and `db/cardinality`; installing
//! one is an ordinary transaction, so the schema has history like everything else. Attributes only grow:
//! a type, cardinality or uniqueness never changes once asserted (accretion).

use crate::kw::Kw;
use crate::value::{Eid, ValueType};
use std::collections::BTreeMap;

pub const DB_IDENT: Eid = Eid(1);
pub const DB_VALUE_TYPE: Eid = Eid(2);
pub const DB_CARDINALITY: Eid = Eid(3);
pub const DB_UNIQUE: Eid = Eid(4);
pub const DB_IS_COMPONENT: Eid = Eid(5);
pub const DB_DOC: Eid = Eid(6);
pub const TX_INSTANT: Eid = Eid(7);
/// The genesis transaction that installs the seven built-in attributes.
pub const GENESIS_TX: Eid = Eid(8);
/// The first id the log hands out after genesis.
pub const FIRST_FREE: u64 = 9;

/// The attributes that define attributes. Changing any of these on an installed attribute is refused.
pub const DEFINING: [Eid; 3] = [DB_VALUE_TYPE, DB_CARDINALITY, DB_UNIQUE];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Unique {
    /// One entity per value, and a transaction naming the value upserts into that entity.
    Identity,
    /// One entity per value; naming it again is an error.
    Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrDef {
    pub id: Eid,
    pub ident: Kw,
    pub ty: ValueType,
    pub many: bool,
    pub unique: Option<Unique>,
    pub component: bool,
    pub doc: Option<String>,
}

/// The installed attributes, by ident and by id.
#[derive(Debug, Clone, Default)]
pub struct Schema {
    by_ident: BTreeMap<Kw, AttrDef>,
    by_id: BTreeMap<Eid, Kw>,
}

/// The built-in attributes, as definitions. Genesis writes them as facts about themselves.
pub fn builtins() -> Vec<AttrDef> {
    let def = |id, ident, ty, unique, doc: &str| AttrDef {
        id,
        ident: Kw::lit(ident),
        ty,
        many: false,
        unique,
        component: false,
        doc: Some(doc.to_string()),
    };
    vec![
        def(
            DB_IDENT,
            "db/ident",
            ValueType::Kw,
            Some(Unique::Identity),
            "The name of an attribute",
        ),
        def(
            DB_VALUE_TYPE,
            "db/valueType",
            ValueType::Kw,
            None,
            "The type of an attribute's values",
        ),
        def(
            DB_CARDINALITY,
            "db/cardinality",
            ValueType::Kw,
            None,
            "One value per entity, or many",
        ),
        def(
            DB_UNIQUE,
            "db/unique",
            ValueType::Kw,
            None,
            "Identity (upserts) or value (refuses duplicates)",
        ),
        def(
            DB_IS_COMPONENT,
            "db/isComponent",
            ValueType::Bool,
            None,
            "A ref whose target belongs to its parent",
        ),
        def(
            DB_DOC,
            "db/doc",
            ValueType::Str,
            None,
            "What an attribute means",
        ),
        def(
            TX_INSTANT,
            "tx/instant",
            ValueType::Instant,
            None,
            "When a transaction entered the log",
        ),
    ]
}

pub fn type_kw(ty: ValueType) -> Kw {
    Kw::lit(match ty {
        ValueType::Ref => "db.type/ref",
        ValueType::Str => "db.type/string",
        ValueType::Int => "db.type/int",
        ValueType::Dec => "db.type/decimal",
        ValueType::Bool => "db.type/bool",
        ValueType::Date => "db.type/date",
        ValueType::Instant => "db.type/instant",
        ValueType::Kw => "db.type/keyword",
        ValueType::Hash => "db.type/hash",
    })
}

pub fn cardinality_kw(many: bool) -> Kw {
    Kw::lit(if many {
        "db.cardinality/many"
    } else {
        "db.cardinality/one"
    })
}

pub fn unique_kw(u: Unique) -> Kw {
    Kw::lit(match u {
        Unique::Identity => "db.unique/identity",
        Unique::Value => "db.unique/value",
    })
}

impl Schema {
    pub fn insert(&mut self, def: AttrDef) {
        self.by_id.insert(def.id, def.ident.clone());
        self.by_ident.insert(def.ident.clone(), def);
    }

    pub fn get(&self, ident: &Kw) -> Option<&AttrDef> {
        self.by_ident.get(ident)
    }

    pub fn by_id(&self, id: Eid) -> Option<&AttrDef> {
        self.by_id.get(&id).and_then(|k| self.by_ident.get(k))
    }

    pub fn ident(&self, id: Eid) -> Option<&Kw> {
        self.by_id.get(&id)
    }

    pub fn iter(&self) -> impl Iterator<Item = &AttrDef> {
        self.by_ident.values()
    }

    pub fn len(&self) -> usize {
        self.by_ident.len()
    }

    pub fn is_empty(&self) -> bool {
        self.by_ident.is_empty()
    }
}

/// Read an attribute definition back from its facts (the inverse of genesis and of installing one).
pub fn def_from_facts(
    id: Eid,
    ident: Kw,
    ty: Option<&Kw>,
    card: Option<&Kw>,
    unique: Option<&Kw>,
    component: bool,
    doc: Option<&str>,
) -> Option<AttrDef> {
    let ty = ty.and_then(|k| (k.ns() == "db.type").then(|| k.name()))?;
    let ty = ValueType::ALL
        .into_iter()
        .find(|t| type_kw(*t).name() == ty)?;
    let many = match card?.as_str() {
        "db.cardinality/one" => false,
        "db.cardinality/many" => true,
        _ => return None,
    };
    let unique = match unique.map(Kw::as_str) {
        None => None,
        Some("db.unique/identity") => Some(Unique::Identity),
        Some("db.unique/value") => Some(Unique::Value),
        Some(_) => return None,
    };
    Some(AttrDef {
        id,
        ident,
        ty,
        many,
        unique,
        component,
        doc: doc.map(str::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_type_has_a_keyword_that_reads_back() {
        for ty in ValueType::ALL {
            let d = def_from_facts(
                Eid(100),
                Kw::lit("t/x"),
                Some(&type_kw(ty)),
                Some(&cardinality_kw(true)),
                Some(&unique_kw(Unique::Value)),
                false,
                None,
            )
            .unwrap();
            assert_eq!((d.ty, d.many, d.unique), (ty, true, Some(Unique::Value)));
        }
    }

    #[test]
    fn builtin_ids_are_stable() {
        let ids: Vec<u64> = builtins().iter().map(|d| d.id.0).collect();
        assert_eq!(ids, vec![1, 2, 3, 4, 5, 6, 7]);
        const { assert!(GENESIS_TX.0 < FIRST_FREE) };
    }
}
