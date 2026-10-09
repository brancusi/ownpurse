//! The database as a value. A `Db` is the log up to one transaction plus indexes over the facts that are
//! true at that point. It never changes: `with` returns a new `Db` and leaves this one as it was, and
//! `as_of` gives an earlier one. Nothing here reads a clock or touches a file.

use crate::kw::Kw;
use crate::schema::{
    self, AttrDef, DB_CARDINALITY, DB_DOC, DB_IDENT, DB_IS_COMPONENT, DB_UNIQUE, DB_VALUE_TYPE,
    DEFINING, FIRST_FREE, GENESIS_TX, Schema, TX_INSTANT, Unique,
};
use crate::time::Instant;
use crate::tx::{EntRef, Op, TxData, ValIn, ent_ref};
use crate::value::{Eid, Value, ValueType};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

/// One fact: entity, attribute, value, the transaction that wrote it, and whether it was asserted or
/// retracted.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Datom {
    pub e: Eid,
    pub a: Eid,
    pub v: Value,
    pub tx: Eid,
    pub added: bool,
}

impl fmt::Debug for Datom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let op = if self.added { '+' } else { '-' };
        write!(
            f,
            "[{op} {:?} {:?} {:?} {:?}]",
            self.e, self.a, self.v, self.tx
        )
    }
}

type Eav = BTreeMap<Eid, BTreeMap<Eid, BTreeMap<Value, Eid>>>;
type Ave = BTreeMap<Eid, BTreeMap<Value, BTreeSet<Eid>>>;
type Vae = BTreeMap<Eid, BTreeSet<(Eid, Eid)>>;

#[derive(Clone)]
pub struct Db {
    basis: Eid,
    next: u64,
    log: Arc<Vec<Datom>>,
    txs: Arc<Vec<Eid>>,
    /// entity → attribute → value → the transaction that asserted it (current facts only)
    eav: Arc<Eav>,
    /// attribute → value → entities
    ave: Arc<Ave>,
    /// ref target → (attribute, source entity)
    vae: Arc<Vae>,
    schema: Arc<Schema>,
}

/// What a transaction did. `tx` is `None` when nothing changed: an empty transaction is never written.
#[derive(Clone, Debug)]
pub struct TxReport {
    pub tx: Option<Eid>,
    pub datoms: Vec<Datom>,
    pub tempids: BTreeMap<String, Eid>,
    pub db_after: Db,
}

/// Every problem a transaction has, found in one pass, so the writer fixes them all at once.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxError {
    pub problems: Vec<String>,
}

impl fmt::Display for TxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "transaction refused: {}", self.problems.join("; "))
    }
}

impl std::error::Error for TxError {}

impl fmt::Debug for Db {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Db(basis {:?}, {} datoms, {} attributes)",
            self.basis,
            self.log.len(),
            self.schema.len()
        )
    }
}

/// The datoms of the genesis transaction: the built-in attributes, described by themselves.
pub fn genesis_datoms() -> Vec<Datom> {
    let mut out = Vec::new();
    let mut put = |e: Eid, a: Eid, v: Value| {
        out.push(Datom {
            e,
            a,
            v,
            tx: GENESIS_TX,
            added: true,
        });
    };
    for d in schema::builtins() {
        put(d.id, DB_IDENT, Value::Kw(d.ident.clone()));
        put(d.id, DB_VALUE_TYPE, Value::Kw(schema::type_kw(d.ty)));
        put(
            d.id,
            DB_CARDINALITY,
            Value::Kw(schema::cardinality_kw(d.many)),
        );
        if let Some(u) = d.unique {
            put(d.id, DB_UNIQUE, Value::Kw(schema::unique_kw(u)));
        }
        if let Some(doc) = &d.doc {
            put(d.id, DB_DOC, Value::str(doc));
        }
    }
    put(GENESIS_TX, TX_INSTANT, Value::Instant(epoch()));
    out.sort();
    out
}

fn epoch() -> Instant {
    Instant::from_unix_millis(0)
}

impl Db {
    /// The database after genesis: the built-in attributes and nothing else.
    pub fn genesis() -> Db {
        let mut db = Db::empty();
        db.apply(GENESIS_TX, &genesis_datoms());
        db
    }

    fn empty() -> Db {
        Db {
            basis: Eid(0),
            next: FIRST_FREE,
            log: Arc::default(),
            txs: Arc::default(),
            eav: Arc::default(),
            ave: Arc::default(),
            vae: Arc::default(),
            schema: Arc::default(),
        }
    }

    /// Rebuild a database from a log, transaction by transaction. The first transaction must be genesis.
    pub fn from_log(datoms: impl IntoIterator<Item = Datom>) -> Result<Db, String> {
        let mut db = Db::empty();
        let mut batch: Vec<Datom> = Vec::new();
        for d in datoms {
            if let Some(last) = batch.last()
                && d.tx != last.tx
            {
                if d.tx < last.tx {
                    return Err(format!("log out of order: {:?} after {:?}", d.tx, last.tx));
                }
                db.apply(last.tx, &batch);
                batch.clear();
            }
            batch.push(d);
        }
        if let Some(last) = batch.last() {
            let tx = last.tx;
            db.apply(tx, &batch);
        }
        match db.txs.first() {
            Some(&GENESIS_TX) => Ok(db),
            _ => Err("the log does not start with genesis".into()),
        }
    }

    pub fn basis(&self) -> Eid {
        self.basis
    }

    pub fn next_id(&self) -> u64 {
        self.next
    }

    pub fn schema(&self) -> &Schema {
        &self.schema
    }

    pub fn attr(&self, ident: &Kw) -> Option<&AttrDef> {
        self.schema.get(ident)
    }

    /// Every datom ever written, in log order (history).
    pub fn log(&self) -> &[Datom] {
        &self.log
    }

    pub fn txs(&self) -> &[Eid] {
        &self.txs
    }

    /// The database as it was right after transaction `t`.
    pub fn as_of(&self, t: Eid) -> Db {
        let keep = self.log.iter().filter(|d| d.tx <= t).cloned();
        Db::from_log(keep).unwrap_or_else(|_| Db::genesis())
    }

    /// Every datom about `e`, assertions and retractions, in log order.
    pub fn history(&self, e: Eid) -> Vec<&Datom> {
        self.log.iter().filter(|d| d.e == e).collect()
    }

    pub fn exists(&self, e: Eid) -> bool {
        self.eav.contains_key(&e)
    }

    /// The current facts about `e`: attribute → value → transaction.
    pub fn entity(&self, e: Eid) -> Option<&BTreeMap<Eid, BTreeMap<Value, Eid>>> {
        self.eav.get(&e)
    }

    pub fn values(&self, e: Eid, a: Eid) -> impl Iterator<Item = &Value> {
        self.eav
            .get(&e)
            .and_then(|m| m.get(&a))
            .into_iter()
            .flat_map(BTreeMap::keys)
    }

    pub fn value(&self, e: Eid, a: Eid) -> Option<&Value> {
        self.values(e, a).next()
    }

    /// The value of `ident` on `e` (first value for a many attribute).
    pub fn get(&self, e: Eid, ident: &Kw) -> Option<&Value> {
        self.schema.get(ident).and_then(|d| self.value(e, d.id))
    }

    /// Entities whose `a` currently equals `v`.
    pub fn with_value(&self, a: Eid, v: &Value) -> impl Iterator<Item = Eid> + '_ {
        self.ave
            .get(&a)
            .and_then(|m| m.get(v))
            .into_iter()
            .flatten()
            .copied()
    }

    /// Every (value, entity) for attribute `a`, in value order.
    pub fn attr_values(&self, a: Eid) -> impl Iterator<Item = (&Value, Eid)> {
        self.ave
            .get(&a)
            .into_iter()
            .flat_map(|m| m.iter().flat_map(|(v, es)| es.iter().map(move |e| (v, *e))))
    }

    /// (attribute, entity) pairs whose ref points at `target`.
    pub fn refs_to(&self, target: Eid) -> impl Iterator<Item = (Eid, Eid)> + '_ {
        self.vae.get(&target).into_iter().flatten().copied()
    }

    /// The entity holding a unique attribute's value.
    pub fn lookup(&self, ident: &Kw, v: &Value) -> Option<Eid> {
        let d = self.schema.get(ident)?;
        d.unique?;
        self.with_value(d.id, v).next()
    }

    /// Count of current facts.
    pub fn len(&self) -> usize {
        self.eav
            .values()
            .flat_map(BTreeMap::values)
            .map(BTreeMap::len)
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn apply(&mut self, tx: Eid, datoms: &[Datom]) {
        let (log, eav, ave, vae) = (
            Arc::make_mut(&mut self.log),
            Arc::make_mut(&mut self.eav),
            Arc::make_mut(&mut self.ave),
            Arc::make_mut(&mut self.vae),
        );
        let mut schema_touched = false;
        for d in datoms {
            log.push(d.clone());
            self.next = self.next.max(d.e.0 + 1);
            schema_touched |= d.a.0 <= DB_DOC.0;
            if d.added {
                eav.entry(d.e)
                    .or_default()
                    .entry(d.a)
                    .or_default()
                    .insert(d.v.clone(), d.tx);
                ave.entry(d.a)
                    .or_default()
                    .entry(d.v.clone())
                    .or_default()
                    .insert(d.e);
                if let Value::Ref(t) = d.v {
                    vae.entry(t).or_default().insert((d.a, d.e));
                }
            } else {
                remove_nested(eav, d.e, d.a, &d.v);
                if let Some(m) = ave.get_mut(&d.a)
                    && let Some(es) = m.get_mut(&d.v)
                {
                    es.remove(&d.e);
                    if es.is_empty() {
                        m.remove(&d.v);
                    }
                }
                if let Value::Ref(t) = d.v
                    && let Some(s) = vae.get_mut(&t)
                {
                    s.remove(&(d.a, d.e));
                    if s.is_empty() {
                        vae.remove(&t);
                    }
                }
            }
        }
        self.next = self.next.max(tx.0 + 1);
        Arc::make_mut(&mut self.txs).push(tx);
        self.basis = tx;
        if schema_touched {
            self.schema = Arc::new(self.read_schema());
        }
    }

    fn read_schema(&self) -> Schema {
        let mut s = Schema::default();
        let kw = |e, a| self.value(e, a).and_then(Value::as_kw);
        for (_, e) in self.attr_values(DB_VALUE_TYPE) {
            let Some(ident) = kw(e, DB_IDENT) else {
                continue;
            };
            let component = matches!(self.value(e, DB_IS_COMPONENT), Some(Value::Bool(true)));
            let doc = self.value(e, DB_DOC).and_then(Value::as_str);
            if let Some(def) = schema::def_from_facts(
                e,
                ident.clone(),
                kw(e, DB_VALUE_TYPE),
                kw(e, DB_CARDINALITY),
                kw(e, DB_UNIQUE),
                component,
                doc,
            ) {
                s.insert(def);
            }
        }
        s
    }

    /// Resolve a transaction against this database and return the database after it. Nothing is written
    /// anywhere: the store appends `report.datoms` to the log, and a manifest simply keeps `db_after`.
    pub fn with(&self, data: &TxData, instant: &Instant) -> Result<TxReport, TxError> {
        Resolver::new(self).run(data, instant)
    }
}

fn remove_nested(eav: &mut Eav, e: Eid, a: Eid, v: &Value) {
    if let Some(am) = eav.get_mut(&e) {
        if let Some(vm) = am.get_mut(&a) {
            vm.remove(v);
            if vm.is_empty() {
                am.remove(&a);
            }
        }
        if am.is_empty() {
            eav.remove(&e);
        }
    }
}

/// One transaction's resolution: temp ids, upserts, lookups, cardinality, uniqueness and accretion.
struct Resolver<'a> {
    db: &'a Db,
    tx: Eid,
    next: u64,
    /// temp id → the temp it was unified with (two temps naming the same identity are one entity)
    alias: BTreeMap<String, String>,
    temps: BTreeMap<String, Eid>,
    /// (identity attribute, value) → temp claiming it in this transaction
    claims: BTreeMap<(Eid, Value), String>,
    /// temps used as ref values (first op that did), to refuse ones that never get facts of their own
    value_temps: BTreeMap<String, usize>,
    /// every temp named in the transaction, for the report
    named: BTreeSet<String>,
    problems: Vec<String>,
}

type Fact = (Eid, Eid, Value);

impl<'a> Resolver<'a> {
    fn new(db: &'a Db) -> Resolver<'a> {
        Resolver {
            db,
            tx: Eid(db.next),
            next: db.next + 1,
            alias: BTreeMap::new(),
            temps: BTreeMap::new(),
            claims: BTreeMap::new(),
            value_temps: BTreeMap::new(),
            named: BTreeSet::new(),
            problems: Vec::new(),
        }
    }

    fn problem(&mut self, i: usize, msg: impl fmt::Display) {
        self.problems.push(format!("op {i}: {msg}"));
    }

    fn def(&mut self, i: usize, a: &Kw) -> Option<AttrDef> {
        let d = self.db.schema.get(a).cloned();
        if d.is_none() {
            self.problem(i, format!("unknown attribute {a} (install it first)"));
        }
        d
    }

    fn canon(&self, t: &str) -> String {
        let mut t = t.to_string();
        while let Some(n) = self.alias.get(&t) {
            t = n.clone();
        }
        t
    }

    /// A scalar (non-ref) value checked against the attribute's type.
    fn scalar(&self, d: &AttrDef, v: &ValIn) -> Result<Value, String> {
        match v {
            ValIn::Val(val) if val.value_type() == d.ty => Ok(val.clone()),
            ValIn::Val(val) => Err(format!(
                "{} takes {} values, got {}",
                d.ident,
                d.ty.name(),
                val.value_type().name()
            )),
            ValIn::Json(j) => {
                Value::from_json(d.ty, &j.json()).map_err(|e| format!("{}: {e}", d.ident))
            }
            ValIn::Ref(_) => Err(format!(
                "{} takes {} values, not a reference",
                d.ident,
                d.ty.name()
            )),
        }
    }

    /// Pass 1: temps that name an identity value become that entity (upsert); temps naming the same new
    /// identity in this transaction become one entity.
    fn upserts(&mut self, data: &TxData) {
        for (i, op) in data.ops.iter().enumerate() {
            let Op::Add(EntRef::Temp(t), a, v) = op else {
                continue;
            };
            let Some(d) = self.db.schema.get(a) else {
                continue;
            };
            if d.unique != Some(Unique::Identity) || d.ty == ValueType::Ref {
                continue;
            }
            let Ok(val) = self.scalar(d, v) else { continue };
            let t = self.canon(t);
            if let Some(existing) = self.db.with_value(d.id, &val).next() {
                match self.temps.get(&t) {
                    Some(&bound) if bound != existing => self.problem(
                        i,
                        format!(
                            "temp id {t:?} names two existing entities ({bound:?} and {existing:?})"
                        ),
                    ),
                    _ => {
                        self.temps.insert(t, existing);
                    }
                }
            } else if let Some(first) = self.claims.get(&(d.id, val.clone())).cloned() {
                if first != t {
                    self.alias.insert(t, first);
                }
            } else {
                self.claims.insert((d.id, val), t);
            }
        }
    }

    fn temp_eid(&mut self, t: &str) -> Eid {
        self.named.insert(t.to_string());
        let t = self.canon(t);
        if let Some(&e) = self.temps.get(&t) {
            return e;
        }
        let e = Eid(self.next);
        self.next += 1;
        self.temps.insert(t, e);
        e
    }

    fn entity(&mut self, i: usize, r: &EntRef) -> Option<Eid> {
        match r {
            EntRef::Tx => Some(self.tx),
            EntRef::Temp(t) => Some(self.temp_eid(t)),
            EntRef::Id(e) if e.0 < self.db.next => Some(*e),
            EntRef::Id(e) => {
                self.problem(
                    i,
                    format!("no entity {} (ids come from the log; use a temp id)", e.0),
                );
                None
            }
            EntRef::Lookup(a, v) => {
                let d = self.def(i, a)?;
                if d.unique.is_none() {
                    self.problem(i, format!("lookup on {a}, which is not unique"));
                    return None;
                }
                let val = match self.scalar(&d, v) {
                    Ok(val) => val,
                    Err(e) => {
                        self.problem(i, e);
                        return None;
                    }
                };
                if let Some(e) = self.db.with_value(d.id, &val).next() {
                    return Some(e);
                }
                if let Some(t) = self.claims.get(&(d.id, val.clone())).cloned() {
                    return Some(self.temp_eid(&t));
                }
                self.problem(i, format!("no entity has {a} = {}", val.text()));
                None
            }
        }
    }

    fn value(&mut self, i: usize, d: &AttrDef, v: &ValIn) -> Option<Value> {
        if d.ty != ValueType::Ref {
            return match self.scalar(d, v) {
                Ok(val) => Some(val),
                Err(e) => {
                    self.problem(i, e);
                    None
                }
            };
        }
        let r = match v {
            ValIn::Ref(r) => r.clone(),
            ValIn::Val(Value::Ref(e)) => EntRef::Id(*e),
            ValIn::Json(j) => match ent_ref(&j.json()) {
                Ok(r) => r,
                Err(e) => {
                    self.problem(i, format!("{}: {e}", d.ident));
                    return None;
                }
            },
            ValIn::Val(other) => {
                self.problem(
                    i,
                    format!("{} takes refs, got {}", d.ident, other.value_type().name()),
                );
                return None;
            }
        };
        if let EntRef::Temp(t) = &r {
            let t = self.canon(t);
            self.value_temps.entry(t).or_insert(i);
        }
        self.entity(i, &r).map(Value::Ref)
    }

    fn run(mut self, data: &TxData, instant: &Instant) -> Result<TxReport, TxError> {
        self.upserts(data);
        let mut adds: BTreeMap<Fact, usize> = BTreeMap::new();
        let mut retracts: BTreeSet<Fact> = BTreeSet::new();
        let mut subjects: BTreeSet<String> = BTreeSet::new();
        for (i, op) in data.ops.iter().enumerate() {
            match op {
                Op::Add(e, a, v) | Op::Retract(e, a, v) => {
                    let add = matches!(op, Op::Add(..));
                    if let (true, EntRef::Temp(t)) = (add, e) {
                        subjects.insert(self.canon(t));
                    }
                    if a == &Kw::lit("tx/instant") {
                        self.problem(i, "tx/instant is set by the log");
                        continue;
                    }
                    let (Some(e), Some(d)) = (self.entity(i, e), self.def(i, a)) else {
                        continue;
                    };
                    let Some(val) = self.value(i, &d, v) else {
                        continue;
                    };
                    if add {
                        adds.entry((e, d.id, val)).or_insert(i);
                    } else if self.db.values(e, d.id).any(|x| x == &val) {
                        retracts.insert((e, d.id, val));
                    }
                }
                Op::RetractEntity(r) => {
                    if r == &EntRef::Tx {
                        self.problem(i, "a transaction cannot retract itself");
                        continue;
                    }
                    if let Some(e) = self.entity(i, r) {
                        self.retract_entity(e, &mut retracts, &mut BTreeSet::new());
                    }
                }
            }
        }
        for (t, i) in std::mem::take(&mut self.value_temps) {
            if !subjects.contains(&t) && !self.temps.get(&t).is_some_and(|e| self.db.exists(*e)) {
                self.problem(
                    i,
                    format!("temp id {t:?} is used as a value but has no facts of its own"),
                );
            }
        }
        self.cardinality_and_uniqueness(&mut adds, &mut retracts);
        self.accretion(&adds, &retracts);
        if !self.problems.is_empty() {
            return Err(TxError {
                problems: self.problems,
            });
        }
        let changes = adds.keys().any(|(e, ..)| *e != self.tx) || !retracts.is_empty();
        let tempids: BTreeMap<String, Eid> = self
            .named
            .iter()
            .filter(|t| !t.starts_with('_'))
            .filter_map(|t| self.temps.get(&self.canon(t)).map(|e| (t.clone(), *e)))
            .collect();
        if !changes {
            return Ok(TxReport {
                tx: None,
                datoms: Vec::new(),
                tempids,
                db_after: self.db.clone(),
            });
        }
        adds.insert((self.tx, TX_INSTANT, Value::Instant(instant.clone())), 0);
        let mut datoms: Vec<Datom> = adds
            .into_keys()
            .map(|(e, a, v)| Datom {
                e,
                a,
                v,
                tx: self.tx,
                added: true,
            })
            .chain(retracts.into_iter().map(|(e, a, v)| Datom {
                e,
                a,
                v,
                tx: self.tx,
                added: false,
            }))
            .collect();
        datoms.sort();
        let mut db_after = self.db.clone();
        db_after.apply(self.tx, &datoms);
        db_after.next = db_after.next.max(self.next);
        Ok(TxReport {
            tx: Some(self.tx),
            datoms,
            tempids,
            db_after,
        })
    }

    fn retract_entity(&self, e: Eid, out: &mut BTreeSet<Fact>, seen: &mut BTreeSet<Eid>) {
        if !seen.insert(e) {
            return;
        }
        if let Some(am) = self.db.eav.get(&e) {
            for (a, vs) in am {
                let component = self.db.schema.by_id(*a).is_some_and(|d| d.component);
                for v in vs.keys() {
                    out.insert((e, *a, v.clone()));
                    if let (true, Value::Ref(child)) = (component, v) {
                        self.retract_entity(*child, out, seen);
                    }
                }
            }
        }
        for (a, src) in self.db.refs_to(e) {
            out.insert((src, a, Value::Ref(e)));
        }
    }

    fn cardinality_and_uniqueness(
        &mut self,
        adds: &mut BTreeMap<Fact, usize>,
        retracts: &mut BTreeSet<Fact>,
    ) {
        for ((e, a, v), &i) in adds.iter() {
            if retracts.contains(&(*e, *a, v.clone())) {
                self.problems
                    .push(format!("op {i}: asserts and retracts the same fact"));
            }
        }
        let mut one: BTreeMap<(Eid, Eid), Vec<(Value, usize)>> = BTreeMap::new();
        for ((e, a, v), &i) in adds.iter() {
            if self.db.schema.by_id(*a).is_some_and(|d| !d.many) {
                one.entry((*e, *a)).or_default().push((v.clone(), i));
            }
        }
        for ((e, a), vals) in one {
            if vals.len() > 1 {
                let ident = self
                    .db
                    .schema
                    .ident(a)
                    .map(Kw::to_string)
                    .unwrap_or_default();
                self.problems.push(format!(
                    "op {}: two values for {ident} on {e:?}, which takes one ({} and {})",
                    vals[1].1,
                    vals[0].0.text(),
                    vals[1].0.text()
                ));
                continue;
            }
            let keep = &vals[0].0;
            for old in self.db.values(e, a) {
                if old != keep {
                    retracts.insert((e, a, old.clone()));
                }
            }
        }
        adds.retain(|(e, a, v), _| !self.db.values(*e, *a).any(|x| x == v));
        let mut seen: BTreeMap<(Eid, Value), Eid> = BTreeMap::new();
        for ((e, a, v), &i) in adds.iter() {
            let Some(d) = self.db.schema.by_id(*a) else {
                continue;
            };
            if d.unique.is_none() {
                continue;
            }
            if let Some(other) = seen.insert((*a, v.clone()), *e)
                && other != *e
            {
                self.problems.push(format!(
                    "op {i}: {} = {} given to two entities",
                    d.ident,
                    v.text()
                ));
            }
            for holder in self.db.with_value(*a, v) {
                if holder != *e && !retracts.contains(&(holder, *a, v.clone())) {
                    self.problems.push(format!(
                        "op {i}: {} = {} already belongs to {holder:?}",
                        d.ident,
                        v.text()
                    ));
                }
            }
        }
    }

    /// Attributes only grow: an installed attribute's type, cardinality and uniqueness never change, and a new
    /// attribute must be complete.
    fn accretion(&mut self, adds: &BTreeMap<Fact, usize>, retracts: &BTreeSet<Fact>) {
        let touched = adds
            .keys()
            .map(|(e, a, _)| (*e, *a))
            .chain(retracts.iter().map(|(e, a, _)| (*e, *a)));
        let mut new_attrs: BTreeSet<Eid> = BTreeSet::new();
        for (e, a) in touched {
            let installed = self.db.schema.by_id(e).is_some();
            if installed && (DEFINING.contains(&a) || a == DB_IDENT || a == DB_IS_COMPONENT) {
                let ident = self
                    .db
                    .schema
                    .ident(e)
                    .map(Kw::to_string)
                    .unwrap_or_default();
                self.problems.push(format!(
                    "attribute {ident} is installed; its definition never changes (add a new attribute)"
                ));
            } else if !installed && a.0 <= DB_DOC.0 {
                new_attrs.insert(e);
            }
        }
        for e in new_attrs {
            let get = |a: Eid| -> Option<Value> {
                adds.keys()
                    .find(|(e2, a2, _)| *e2 == e && *a2 == a)
                    .map(|(_, _, v)| v.clone())
            };
            let kw = |a| get(a).and_then(|v| v.as_kw().cloned());
            let comp = matches!(get(DB_IS_COMPONENT), Some(Value::Bool(true)));
            let doc = get(DB_DOC);
            let Some(ident) = kw(DB_IDENT) else {
                self.problems
                    .push(format!("{e:?}: a new attribute needs db/ident"));
                continue;
            };
            match schema::def_from_facts(
                e,
                ident.clone(),
                kw(DB_VALUE_TYPE).as_ref(),
                kw(DB_CARDINALITY).as_ref(),
                kw(DB_UNIQUE).as_ref(),
                comp,
                doc.as_ref().and_then(Value::as_str),
            ) {
                Some(d) if d.component && d.ty != ValueType::Ref => {
                    self.problems
                        .push(format!("{ident}: only a ref attribute can be a component"));
                }
                Some(_) => {}
                None => self.problems.push(format!(
                    "{ident}: a new attribute needs a valid db/valueType and db/cardinality"
                )),
            }
        }
    }
}
