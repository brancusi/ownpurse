//! Pull: fetch the exact shape a caller needs from one entity, as JSON.
//!
//! A pattern is a JSON array of items: `"*"` (every attribute; components are followed), `"obs/amount"`,
//! `{"obs/region": [...]}` (follow a ref with a sub-pattern), and reverse refs written with an underscore
//! before the name, `"obs/_doc"` or `{"obs/_doc": [...]}` (the entities whose `obs/doc` points here).

use crate::db::Db;
use crate::kw::Kw;
use crate::value::{Eid, Value};
use serde_json::{Map, Value as Json, json};
use std::collections::BTreeSet;

const MAX_DEPTH: usize = 12;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Item {
    Wild,
    Attr(Kw, Option<Pattern>),
    Reverse(Kw, Option<Pattern>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pattern(pub Vec<Item>);

impl Pattern {
    pub fn wild() -> Pattern {
        Pattern(vec![Item::Wild])
    }

    pub fn from_json(j: &Json) -> Result<Pattern, String> {
        let items = j.as_array().ok_or("a pull pattern is an array")?;
        let mut out = Vec::new();
        for it in items {
            match it {
                Json::String(s) if s == "*" => out.push(Item::Wild),
                Json::String(s) => out.push(item(s, None)?),
                Json::Object(m) => {
                    for (k, sub) in m {
                        out.push(item(k, Some(Pattern::from_json(sub)?))?);
                    }
                }
                other => return Err(format!("not a pull item: {other}")),
            }
        }
        Ok(Pattern(out))
    }
}

fn item(name: &str, sub: Option<Pattern>) -> Result<Item, String> {
    let k = Kw::new(name).map_err(|e| e.to_string())?;
    match k.name().strip_prefix('_') {
        Some(fwd) => {
            let fwd = Kw::new(&format!("{}/{fwd}", k.ns())).map_err(|e| e.to_string())?;
            Ok(Item::Reverse(fwd, sub))
        }
        None => Ok(Item::Attr(k, sub)),
    }
}

/// Pull `pattern` from entity `e`. Unknown attributes are skipped; a missing entity gives `{"db/id": e}`.
pub fn pull(db: &Db, pattern: &Pattern, e: Eid) -> Json {
    pull_at(db, pattern, e, 0, &mut BTreeSet::new())
}

fn reference(db: &Db, e: Eid) -> Json {
    let mut m = Map::new();
    m.insert("db/id".into(), json!(e.0));
    if let Some(Value::Kw(k)) = db.get(e, &Kw::lit("db/ident")) {
        m.insert("db/ident".into(), json!(k.as_str()));
    }
    Json::Object(m)
}

fn pull_at(db: &Db, pattern: &Pattern, e: Eid, depth: usize, path: &mut BTreeSet<Eid>) -> Json {
    if depth > MAX_DEPTH || !path.insert(e) {
        return reference(db, e);
    }
    let mut out = Map::new();
    out.insert("db/id".into(), json!(e.0));
    for it in &pattern.0 {
        match it {
            Item::Wild => {
                for (a, vals) in db.entity(e).into_iter().flatten() {
                    let Some(d) = db.schema().by_id(*a) else {
                        continue;
                    };
                    let sub = d.component.then(Pattern::wild);
                    let vs: Vec<Json> = vals
                        .keys()
                        .map(|v| render(db, v, sub.as_ref(), depth, path))
                        .collect();
                    out.insert(d.ident.to_string(), shape(d.many, vs));
                }
            }
            Item::Attr(k, sub) => {
                let Some(d) = db.attr(k) else { continue };
                let vs: Vec<Json> = db
                    .values(e, d.id)
                    .map(|v| render(db, v, sub.as_ref(), depth, path))
                    .collect();
                if !vs.is_empty() {
                    out.insert(k.to_string(), shape(d.many, vs));
                }
            }
            Item::Reverse(k, sub) => {
                let Some(d) = db.attr(k) else { continue };
                let srcs: Vec<Json> = db
                    .refs_to(e)
                    .filter(|(a, _)| *a == d.id)
                    .map(|(_, src)| match sub {
                        Some(p) => pull_at(db, p, src, depth + 1, path),
                        None => reference(db, src),
                    })
                    .collect();
                if !srcs.is_empty() {
                    out.insert(format!("{}/_{}", k.ns(), k.name()), Json::Array(srcs));
                }
            }
        }
    }
    path.remove(&e);
    Json::Object(out)
}

fn render(
    db: &Db,
    v: &Value,
    sub: Option<&Pattern>,
    depth: usize,
    path: &mut BTreeSet<Eid>,
) -> Json {
    match (v, sub) {
        (Value::Ref(t), Some(p)) => pull_at(db, p, *t, depth + 1, path),
        (Value::Ref(t), None) => reference(db, *t),
        (other, _) => other.to_json(),
    }
}

fn shape(many: bool, mut vs: Vec<Json>) -> Json {
    if many {
        Json::Array(vs)
    } else {
        vs.pop().unwrap_or(Json::Null)
    }
}
