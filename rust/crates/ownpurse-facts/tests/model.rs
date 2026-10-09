//! Model-based test: random transactions against the `Db` and against a naive model (a set of facts and the
//! rules written out longhand). After every transaction the current facts, every `as_of`, and a replay of the
//! log must agree with the model.

use ownpurse_facts::{Db, Eid, EntRef, Instant, Kw, TxData, Value, install};
use proptest::prelude::*;
use serde_json::json;
use std::collections::BTreeSet;

type Fact = (u64, &'static str, Value);
/// What the model does for a step, given the id the database gave a new entity.
type Expect = Box<dyn FnOnce(&mut Model, u64)>;

#[derive(Debug, Clone)]
enum Step {
    /// add to an existing entity (by index into the live entities) or a new one
    Add {
        target: Option<usize>,
        attr: usize,
        v: i64,
    },
    /// add with a fresh temp that names identity `t/id = id`
    Upsert {
        id: u8,
        attr: usize,
        v: i64,
    },
    Retract {
        pick: usize,
    },
    RetractEntity {
        pick: usize,
    },
}

const ATTRS: [&str; 3] = ["t/one", "t/many", "t/ref"];

fn step() -> impl Strategy<Value = Step> {
    prop_oneof![
        3 => (proptest::option::of(0usize..8), 0usize..3, 0i64..4).prop_map(|(target, attr, v)| Step::Add { target, attr, v }),
        2 => (0u8..3, 0usize..2, 0i64..4).prop_map(|(id, attr, v)| Step::Upsert { id, attr, v }),
        2 => (0usize..64).prop_map(|pick| Step::Retract { pick }),
        1 => (0usize..8).prop_map(|pick| Step::RetractEntity { pick }),
    ]
}

fn base() -> Db {
    let defs = json!([
        {"ident": "t/one", "type": "int"},
        {"ident": "t/many", "type": "int", "many": true},
        {"ident": "t/ref", "type": "ref", "many": true},
        {"ident": "t/id", "type": "string", "unique": "identity"}
    ]);
    Db::genesis()
        .with(&install(&defs).unwrap(), &Instant::from_unix_millis(0))
        .unwrap()
        .db_after
}

/// The current user facts of a `Db` (entities that are neither attributes nor transactions).
fn facts(db: &Db) -> BTreeSet<Fact> {
    let mut out = BTreeSet::new();
    for a in ["t/one", "t/many", "t/ref", "t/id"] {
        let d = db.attr(&Kw::lit(a)).unwrap();
        for (v, e) in db.attr_values(d.id) {
            out.insert((e.0, a, v.clone()));
        }
    }
    out
}

struct Model {
    facts: BTreeSet<Fact>,
}

impl Model {
    fn entities(&self) -> Vec<u64> {
        let s: BTreeSet<u64> = self.facts.iter().map(|f| f.0).collect();
        s.into_iter().collect()
    }

    fn by_identity(&self, id: &str) -> Option<u64> {
        self.facts
            .iter()
            .find(|(_, a, v)| *a == "t/id" && v.as_str() == Some(id))
            .map(|f| f.0)
    }

    fn add(&mut self, e: u64, a: &'static str, v: Value) {
        if a == "t/one" {
            self.facts.retain(|(e2, a2, _)| !(*e2 == e && *a2 == a));
        }
        self.facts.insert((e, a, v));
    }
}

fn val(attr: &str, v: i64, ents: &[u64]) -> Option<Value> {
    if attr == "t/ref" {
        let idx = usize::try_from(v).ok()?;
        ents.get(idx).map(|e| Value::Ref(Eid(*e)))
    } else {
        Some(Value::Int(v))
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]
    #[test]
    fn the_db_agrees_with_the_naive_model(steps in proptest::collection::vec(step(), 1..40)) {
        let mut db = base();
        let mut model = Model { facts: BTreeSet::new() };
        let mut snapshots: Vec<(Eid, BTreeSet<Fact>)> = Vec::new();
        for (n, s) in steps.iter().enumerate() {
            let at = Instant::from_unix_millis(1000 * (n as i64 + 1));
            let ents = model.entities();
            let mut tx = TxData::new();

            let expect: Expect = match s.clone() {
                Step::Add { target, attr, v } => {
                    let a = ATTRS[attr];
                    let Some(value) = val(a, v, &ents) else { continue };
                    let e = match target.and_then(|i| ents.get(i)) {
                        Some(e) => EntRef::Id(Eid(*e)),
                        None => EntRef::temp("new"),
                    };
                    tx.add(&e, &Kw::lit(a), value.clone());
                    let known = if let EntRef::Id(x) = e { Some(x.0) } else { None };
                    Box::new(move |m: &mut Model, new: u64| m.add(known.unwrap_or(new), a, value))
                }
                Step::Upsert { id, attr, v } => {
                    let a = ATTRS[attr];
                    let idv = format!("id{id}");
                    let e = EntRef::temp("u");
                    tx.add(&e, &Kw::lit("t/id"), Value::str(&idv));
                    tx.add(&e, &Kw::lit(a), Value::Int(v));
                    let existing = model.by_identity(&idv);
                    Box::new(move |m: &mut Model, new: u64| {
                        let e = existing.unwrap_or(new);
                        m.add(e, "t/id", Value::str(&idv));
                        m.add(e, a, Value::Int(v));
                    })
                }
                Step::Retract { pick } => {
                    let Some(f) = model.facts.iter().nth(pick % model.facts.len().max(1)).cloned() else { continue };
                    tx.retract(&EntRef::Id(Eid(f.0)), &Kw::lit(f.1), f.2.clone());
                    Box::new(move |m: &mut Model, _| { m.facts.remove(&f); })
                }
                Step::RetractEntity { pick } => {
                    let Some(e) = ents.get(pick % ents.len().max(1)).copied() else { continue };
                    tx.retract_entity(&EntRef::Id(Eid(e)));
                    Box::new(move |m: &mut Model, _| {
                        m.facts.retain(|(e2, _, v)| *e2 != e && v != &Value::Ref(Eid(e)));
                    })
                }
            };
            let r = db.with(&tx, &at).expect("valid by construction");
            let new = r.tempids.get("new").or(r.tempids.get("u")).map_or(0, |e| e.0);
            expect(&mut model, new);
            db = r.db_after;
            prop_assert_eq!(facts(&db), model.facts.clone(), "after step {}: {:?}", n, s);
            if let Some(t) = r.tx { snapshots.push((t, model.facts.clone())); }
        }
        for (t, expected) in &snapshots {
            prop_assert_eq!(&facts(&db.as_of(*t)), expected, "as_of {:?}", t);
        }
        let replayed = Db::from_log(db.log().iter().cloned()).unwrap();
        prop_assert_eq!(facts(&replayed), facts(&db));
    }
}
