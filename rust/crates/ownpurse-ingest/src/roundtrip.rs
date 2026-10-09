//! The mirror's law: a document rebuilt from its facts equals the document as the source sent it, after the
//! transforms the mirror is known to make. Both sides are brought to one canonical JSON form and compared.
//!
//! The known transforms, each also reported as pressure when it happens:
//! - `null` fields and empty lists are absent;
//! - an embedded copy of another entity keeps only its id field;
//! - a list of plain values is a set (sorted, no repeats);
//! - strings the source's conventions call dates or instants are compared as those values;
//! - field names are cleaned to attribute-name characters.

use crate::jsonmirror::{JsonSource, clean, id_text, ref_rule};
use ownpurse_facts::{Db, Eid, Kw, Value, ValueType};
use serde_json::{Map, Value as Json, json};

fn scalar_canon(v: &Value) -> Json {
    match v {
        Value::Bool(b) => json!(b),
        Value::Date(d) => json!(format!("date:{d}")),
        Value::Instant(i) => json!(format!("instant:{i}")),
        other => json!(other.text()),
    }
}

/// The source's JSON in canonical form.
pub fn canonical_source(cfg: &JsonSource, obj: &Map<String, Json>) -> Json {
    let mut out = Map::new();
    for (field, v) in obj {
        let name = clean(field).0;
        let c = match v {
            Json::Null => continue,
            Json::Array(items) if items.is_empty() => continue,
            Json::Object(o) => match reference(cfg, field, o) {
                Some(r) => r,
                None => canonical_source(cfg, o),
            },
            Json::Array(items) if items.iter().all(Json::is_object) => {
                let owned = cfg.owned.iter().any(|(f, _, _)| f == field);
                Json::Array(
                    items
                        .iter()
                        .filter_map(Json::as_object)
                        .map(|o| match (owned, reference(cfg, field, o)) {
                            (false, Some(r)) => r,
                            _ => canonical_source(cfg, o),
                        })
                        .collect(),
                )
            }
            Json::Array(items) => {
                let mut vs: Vec<String> =
                    items.iter().map(|i| scalar_text(cfg, field, i)).collect();
                vs.sort();
                vs.dedup();
                json!(vs)
            }
            scalar => scalar_canon(&scalar_value(cfg, field, scalar)),
        };
        out.insert(name, c);
    }
    Json::Object(out)
}

fn scalar_value(cfg: &JsonSource, field: &str, v: &Json) -> Value {
    match v {
        Json::Bool(b) => Value::Bool(*b),
        Json::Number(n) => ownpurse_facts::value::parse_decimal(&n.to_string())
            .map_or_else(|_| Value::str(&n.to_string()), Value::Dec),
        Json::String(s) => (cfg.when)(field, s).unwrap_or_else(|| Value::str(s)),
        other => Value::str(&other.to_string()),
    }
}

fn scalar_text(cfg: &JsonSource, field: &str, v: &Json) -> String {
    match scalar_canon(&scalar_value(cfg, field, v)) {
        Json::String(s) => s,
        other => other.to_string(),
    }
}

/// An embedded reference reduced to its id field, the way the mirror stores it. Owned children (a tracking
/// category's options) are not references.
fn reference(cfg: &JsonSource, field: &str, o: &Map<String, Json>) -> Option<Json> {
    let (_, idf) = ref_rule(cfg, field)?;
    let id = o.get(idf).and_then(id_text)?;
    Some(json!({ idf: id }))
}

/// A document rebuilt from the facts on entity `e`, whose attributes live in namespace `ns`
/// (`xero.Invoice`), in canonical form.
pub fn render(db: &Db, cfg: &JsonSource, e: Eid, ns: &str) -> Json {
    let mut out = Map::new();
    let index = Kw::new(&format!("{}/index", cfg.ns)).ok();
    for (aid, vals) in db.entity(e).into_iter().flatten() {
        let Some(d) = db.schema().by_id(*aid) else {
            continue;
        };
        if d.ident.ns() != ns {
            continue;
        }
        let field = d.ident.name().to_string();
        let child_ns = format!("{ns}.{field}");
        let rendered: Vec<Json> = if d.ty == ValueType::Ref {
            let mut kids: Vec<(i64, Json)> = vals
                .keys()
                .filter_map(Value::as_ref)
                .map(|t| {
                    let pos = index
                        .as_ref()
                        .and_then(|ix| db.get(t, ix))
                        .and_then(|v| match v {
                            Value::Int(i) => Some(*i),
                            _ => None,
                        })
                        .unwrap_or(0);
                    (pos, target(db, cfg, t, &field, &child_ns, d.component))
                })
                .collect();
            kids.sort_by(|a, b| {
                a.0.cmp(&b.0)
                    .then_with(|| a.1.to_string().cmp(&b.1.to_string()))
            });
            kids.into_iter().map(|(_, j)| j).collect()
        } else {
            vals.keys().map(scalar_canon).collect()
        };
        let v = if d.many && d.ty != ValueType::Ref {
            let mut vs: Vec<String> = rendered
                .iter()
                .map(|j| j.as_str().map_or_else(|| j.to_string(), str::to_string))
                .collect();
            vs.sort();
            vs.dedup();
            json!(vs)
        } else if d.many {
            Json::Array(rendered)
        } else {
            rendered.into_iter().next().unwrap_or(Json::Null)
        };
        out.insert(field, v);
    }
    Json::Object(out)
}

fn target(db: &Db, cfg: &JsonSource, t: Eid, field: &str, child_ns: &str, component: bool) -> Json {
    if component {
        return render(db, cfg, t, child_ns);
    }
    // an owned child is an entity of its own kind, described in full
    if let Some((_, kind, _)) = cfg.owned.iter().find(|(f, _, _)| *f == field) {
        return render(db, cfg, t, &format!("{}.{kind}", cfg.ns));
    }
    // a reference: `<source>/<Kind>/<id>` → {<id field>: id}
    let did = db
        .get(t, &Kw::lit("doc/id"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let id = did.rsplit('/').next().unwrap_or("");
    match ref_rule(cfg, field) {
        Some((_, idf)) => json!({ idf: id }),
        None => json!({ "?": did }),
    }
}
