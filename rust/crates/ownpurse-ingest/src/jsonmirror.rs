//! The mirror: any JSON API response, decomposed into facts in the source's own vocabulary, losslessly
//! where the model allows and with a pressure note where it does not.
//!
//! Every field becomes an attribute `<ns>.<Kind>/<Field>` (`xero.Invoice/Reference`), typed by its first
//! value and installed on first sight (schemas grow by accretion). Nested objects become component entities
//! (`xero.Invoice.LineItems/LineAmount`); an object under a field the source declares a reference
//! (`Contact`) becomes a ref to that entity instead. Arrays of objects become many components with their position in `<ns>/index`.
//! The result is a `Snapshot`, so a second read of the same document asserts only what changed.

use crate::snapshot::Snapshot;
use crate::{IngestError, Input, Pressure};
use ownpurse_facts::{Db, Kw, ValIn, Value, ValueType};
use serde_json::{Map, Value as Json, json};
use std::collections::BTreeMap;

/// How one source's JSON is shaped. Data, not code: a new source is mostly a new one of these.
pub struct JsonSource {
    /// Attribute namespace prefix: `xero`, `qbo`, `freshbooks`.
    pub ns: &'static str,
    /// Object keys to descend through before the collections (`["QueryResponse"]`, `["response", "result"]`).
    pub root: &'static [&'static str],
    /// Top-level collections: (list key, kind, id field).
    pub collections: &'static [(&'static str, &'static str, &'static str)],
    /// Fields whose object (or list of objects) refers to another entity: (field, kind, id field). Declared
    /// by field name, not guessed from an id inside the object: a journal line holds an `AccountID` and is
    /// still a line, not a reference to the account.
    pub refs: &'static [(&'static str, &'static str, &'static str)],
    /// Lists inside a document whose items are entities of their own kind, described here and nowhere else:
    /// (field, kind, id field). A tracking category's options, say.
    pub owned: &'static [(&'static str, &'static str, &'static str)],
    /// Fields that identify a child object inside its parent (`LineItemID`); otherwise position does.
    pub child_ids: &'static [&'static str],
    /// Turn a string into a date or instant when the source's conventions say it is one.
    pub when: fn(field: &str, text: &str) -> Option<Value>,
}

/// One top-level document found in the response.
#[derive(Debug, Clone)]
pub struct Root<'j> {
    /// The snapshot key (and temp id) of the document's entity: `doc:<doc id>`.
    pub key: String,
    pub kind: &'static str,
    pub id: String,
    pub doc_id: String,
    pub pointer: String,
    pub json: &'j Map<String, Json>,
}

pub struct Mirrored<'j> {
    pub snap: Snapshot,
    pub install: Vec<Json>,
    pub pressure: Vec<Pressure>,
    pub roots: Vec<Root<'j>>,
}

/// A stand-in ref value: type inference only looks at the type.
const REF: Value = Value::Ref(ownpurse_facts::Eid(0));

struct Walker<'a, 'j> {
    cfg: &'a JsonSource,
    db: &'a Db,
    source_id: &'a str,
    snap: Snapshot,
    /// attributes this read wants installed: ident → (type, many, component)
    defs: BTreeMap<Kw, (ValueType, bool, bool)>,
    pressure: Vec<Pressure>,
    roots: Vec<Root<'j>>,
}

/// The document id for a kind and id at a source: `<source id>/<Kind>/<id>`.
pub fn doc_id(source_id: &str, kind: &str, id: &str) -> String {
    format!("{source_id}/{kind}/{id}")
}

/// When a document was last read: the latest `tx/observed-at` among the transactions that cited a file
/// carrying it. Snapshot sources compare it with the incoming read, so replaying an old file is harmless.
pub fn last_read(db: &Db, doc_id: &str) -> Option<ownpurse_facts::Instant> {
    let e = db.lookup(&Kw::lit("doc/id"), &Value::str(doc_id))?;
    let file_attr = db.attr(&Kw::lit("doc/file"))?.id;
    let obs = Kw::lit("tx/observed-at");
    db.entity(e)?
        .get(&file_attr)?
        .values()
        .filter_map(|tx| match db.get(*tx, &obs) {
            Some(Value::Instant(i)) => Some(i.clone()),
            _ => None,
        })
        .max()
}

/// The (kind, id field) a field refers to, if the source declares it a reference.
pub(crate) fn ref_rule(cfg: &JsonSource, field: &str) -> Option<(&'static str, &'static str)> {
    cfg.refs
        .iter()
        .find(|(f, _, _)| *f == field)
        .map(|(_, k, i)| (*k, *i))
}

pub(crate) fn id_text(v: &Json) -> Option<String> {
    match v {
        Json::String(s) if !s.is_empty() => Some(s.clone()),
        Json::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

/// A field name as an attribute name part: letters, digits, `.`, `-`, `_`; anything else becomes `_`.
pub(crate) fn clean(field: &str) -> (String, bool) {
    let c: String = field
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch
            } else {
                '_'
            }
        })
        .collect();
    let changed = c != field;
    (if c.is_empty() { "_".into() } else { c }, changed)
}

pub fn mirror<'j>(
    cfg: &JsonSource,
    db: &Db,
    input: &Input<'_>,
    body: &'j Json,
) -> Result<Mirrored<'j>, IngestError> {
    let mut node = body;
    let mut ptr = String::new();
    for step in cfg.root {
        node = node
            .get(step)
            .ok_or_else(|| IngestError(format!("{}: expected an object at /{step}", cfg.ns)))?;
        ptr.push('/');
        ptr.push_str(step);
    }
    let mut w = Walker {
        cfg,
        db,
        source_id: &input.source.id,
        snap: Snapshot::new(),
        defs: BTreeMap::new(),
        pressure: Vec::new(),
        roots: Vec::new(),
    };
    let path_attr = Kw::new(&format!("{}/path", cfg.ns)).map_err(|e| IngestError(e.to_string()))?;
    let index_attr =
        Kw::new(&format!("{}/index", cfg.ns)).map_err(|e| IngestError(e.to_string()))?;
    w.defs.insert(path_attr, (ValueType::Str, false, false));
    w.defs.insert(index_attr, (ValueType::Int, false, false));
    let mut found = 0;
    for (list_key, kind, id_field) in cfg.collections {
        let Some(items) = node.get(list_key) else {
            continue;
        };
        let Some(items) = items.as_array() else {
            w.press(
                "shape",
                format!("{ptr}/{list_key}"),
                "a collection that is not an array",
            );
            continue;
        };
        found += 1;
        for (i, item) in items.iter().enumerate() {
            let p = format!("{ptr}/{list_key}/{i}");
            let Some(obj) = item.as_object() else {
                w.press("shape", &p, "a collection item that is not an object");
                continue;
            };
            let id = match obj.get(*id_field).and_then(id_text) {
                Some(id) => id,
                None => {
                    w.press("no-identity", kind.to_string(), format!("{kind} without {id_field}; identified by position, which a re-read can shift"));
                    format!("@{i}")
                }
            };
            let did = doc_id(w.source_id, kind, &id);
            if let Some(last) = last_read(db, &did).filter(|last| *last > input.observed_at) {
                w.press(
                    "stale-read",
                    kind.to_string(),
                    format!("a read from before the last one ({last}); an older snapshot never overwrites a newer one"),
                );
                continue;
            }
            let key = format!("doc:{did}");
            w.snap.ent(&key, &Kw::lit("doc/id"), Value::str(&did));
            w.walk(obj, &key, &format!("{}.{kind}", cfg.ns), &did, &p);
            w.roots.push(Root {
                key,
                kind,
                id,
                doc_id: did,
                pointer: p,
                json: obj,
            });
        }
    }
    if found == 0 {
        let keys: Vec<String> = node
            .as_object()
            .map(|m| m.keys().cloned().collect())
            .unwrap_or_default();
        return Err(IngestError(format!(
            "{}: no known collection in this file (keys: {})",
            cfg.ns,
            keys.join(", ")
        )));
    }
    let install = w.install();
    Ok(Mirrored {
        snap: w.snap,
        install,
        pressure: w.pressure,
        roots: w.roots,
    })
}

impl<'a, 'j> Walker<'a, 'j> {
    fn press(&mut self, kind: &str, at: impl Into<String>, detail: impl Into<String>) {
        self.pressure.push(Pressure::new(kind, at, detail));
    }

    /// Check a value against the attribute's type (installed, or first seen in this read). Strings and
    /// decimals convert into each other when exact; anything else is a type conflict.
    fn typed(&mut self, attr: &Kw, v: Value, many: bool, component: bool) -> Option<Value> {
        let have = self
            .db
            .attr(attr)
            .map(|d| (d.ty, d.many))
            .or_else(|| self.defs.get(attr).map(|(t, m, _)| (*t, *m)));
        let Some((ty, was_many)) = have else {
            self.defs
                .insert(attr.clone(), (v.value_type(), many, component));
            return Some(v);
        };
        if was_many != many {
            self.press(
                "cardinality",
                attr.to_string(),
                "a field that is sometimes a list and sometimes one value",
            );
        }
        if ty == v.value_type() {
            return Some(v);
        }
        let converted = match (ty, &v) {
            (ValueType::Str, other) => Some(Value::str(&other.text())),
            (ValueType::Dec, Value::Str(s)) => {
                ownpurse_facts::value::parse_decimal(s).ok().map(Value::Dec)
            }
            _ => None,
        };
        if converted.is_none() {
            self.press(
                "type-conflict",
                attr.to_string(),
                format!(
                    "installed as {}, got {}; value not stored",
                    ty.name(),
                    v.value_type().name()
                ),
            );
        }
        converted
    }

    fn attr(&mut self, attr_ns: &str, field: &str) -> Option<Kw> {
        let (name, changed) = clean(field);
        if changed {
            self.press(
                "field-name",
                format!("{attr_ns}/{field}"),
                format!("stored as {attr_ns}/{name}"),
            );
        }
        Kw::new(&format!("{attr_ns}/{name}")).ok()
    }

    fn scalar(&mut self, field: &str, v: &Json) -> Option<Value> {
        match v {
            Json::Bool(b) => Some(Value::Bool(*b)),
            Json::Number(n) => match ownpurse_facts::value::parse_decimal(&n.to_string()) {
                Ok(d) => Some(Value::Dec(d)),
                Err(_) => {
                    self.press(
                        "precision",
                        field,
                        format!("{n} is not an exact decimal in 28 digits; kept as text"),
                    );
                    Some(Value::str(&n.to_string()))
                }
            },
            Json::String(s) => Some((self.cfg.when)(field, s).unwrap_or_else(|| Value::str(s))),
            _ => None,
        }
    }

    fn walk(
        &mut self,
        obj: &'j Map<String, Json>,
        key: &str,
        attr_ns: &str,
        path: &str,
        ptr: &str,
    ) {
        for (field, v) in obj {
            let Some(attr) = self.attr(attr_ns, field) else {
                continue;
            };
            let p = format!("{ptr}/{field}");
            match v {
                Json::Null => self.press(
                    "null",
                    attr.to_string(),
                    "null is stored as no fact: null and absent read the same",
                ),
                Json::Object(child) => {
                    if let Some(target) = self.reference(child, &attr, field) {
                        if self.typed(&attr, REF, false, false).is_some() {
                            self.snap.add(key, &attr, ValIn::Ref(target));
                        }
                    } else {
                        let ck = format!("{key}{}", &p[ptr.len()..]);
                        self.component(&ck, &format!("{path}/{field}"));
                        if self.typed(&attr, REF, false, true).is_some() {
                            self.snap.add(key, &attr, ValIn::Ref(crate::temp(&ck)));
                        }
                        self.walk(
                            child,
                            &ck,
                            &format!("{attr_ns}.{}", clean(field).0),
                            &format!("{path}/{field}"),
                            &p,
                        );
                    }
                }
                Json::Array(items) => self.array(items, key, attr_ns, &attr, field, path, &p),
                scalar => {
                    if let Some(val) = self.scalar(field, scalar)
                        && let Some(val) = self.typed(&attr, val, false, false)
                    {
                        self.snap.add(key, &attr, val);
                    }
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)] // one call site; a struct for these would only rename them
    fn array(
        &mut self,
        items: &'j [Json],
        key: &str,
        attr_ns: &str,
        attr: &Kw,
        field: &str,
        path: &str,
        p: &str,
    ) {
        if items.is_empty() {
            self.press(
                "empty-list",
                attr.to_string(),
                "an empty list is stored as no fact: empty and absent read the same",
            );
            return;
        }
        if let (true, Some((_, kind, idf))) = (
            items.iter().all(Json::is_object),
            self.cfg.owned.iter().find(|(f, _, _)| *f == field),
        ) {
            for (i, item) in items.iter().enumerate() {
                let Some(obj) = item.as_object() else {
                    continue;
                };
                let Some(id) = obj.get(*idf).and_then(id_text) else {
                    self.press(
                        "no-identity",
                        kind.to_string(),
                        format!("{kind} without {idf}; not stored"),
                    );
                    continue;
                };
                let did = doc_id(self.source_id, kind, &id);
                let ck = format!("doc:{did}");
                self.snap.ent(&ck, &Kw::lit("doc/id"), Value::str(&did));
                // the list's order is the parent's to keep; without this a re-render reorders it
                if let Ok(ix) = Kw::new(&format!("{}/index", self.cfg.ns)) {
                    self.snap
                        .add(&ck, &ix, Value::Int(i64::try_from(i).unwrap_or(i64::MAX)));
                }
                if self.typed(attr, REF, true, false).is_some() {
                    self.snap.add(key, attr, ValIn::Ref(crate::temp(&ck)));
                }
                self.walk(
                    obj,
                    &ck,
                    &format!("{}.{kind}", self.cfg.ns),
                    &did,
                    &format!("{p}/{i}"),
                );
            }
            return;
        }
        if items.iter().all(Json::is_object) {
            let child_ns = format!("{attr_ns}.{}", clean(field).0);
            for (i, item) in items.iter().enumerate() {
                let Some(obj) = item.as_object() else {
                    continue;
                };
                if let Some(target) = self.reference(obj, attr, field) {
                    if self.typed(attr, REF, true, false).is_some() {
                        self.snap.add(key, attr, ValIn::Ref(target));
                    }
                    continue;
                }
                let local = self
                    .cfg
                    .child_ids
                    .iter()
                    .find_map(|f| obj.get(*f).and_then(id_text));
                if local.is_none() && items.len() > 1 {
                    self.press(
                        "position-identity",
                        attr.to_string(),
                        "list items without their own id are identified by position: inserting one renames the rest",
                    );
                }
                let seg = local.unwrap_or_else(|| i.to_string());
                let ck = format!("{key}/{field}/{seg}");
                self.component(&ck, &format!("{path}/{field}/{seg}"));
                let index = Kw::new(&format!("{}/index", self.cfg.ns)).ok();
                if let Some(ix) = index {
                    self.snap
                        .add(&ck, &ix, Value::Int(i64::try_from(i).unwrap_or(i64::MAX)));
                }
                if self.typed(attr, REF, true, true).is_some() {
                    self.snap.add(key, attr, ValIn::Ref(crate::temp(&ck)));
                }
                self.walk(
                    obj,
                    &ck,
                    &child_ns,
                    &format!("{path}/{field}/{seg}"),
                    &format!("{p}/{i}"),
                );
            }
        } else if items.iter().all(|v| !v.is_object() && !v.is_array()) {
            if items.len() > 1 {
                self.press(
                    "order-lost",
                    attr.to_string(),
                    "a list of values is stored as a set: order and repeats are lost",
                );
            }
            for v in items {
                if let Some(val) = self.scalar(field, v)
                    && let Some(val) = self.typed(attr, val, true, false)
                {
                    self.snap.add(key, attr, val);
                }
            }
        } else {
            self.press(
                "shape",
                attr.to_string(),
                "a list mixing objects, lists and values; not stored",
            );
        }
    }

    fn component(&mut self, ck: &str, path: &str) {
        let path_attr = Kw::new(&format!("{}/path", self.cfg.ns)).ok();
        if let Some(pa) = path_attr {
            self.snap.ent(ck, &pa, Value::str(path));
        }
    }

    /// An embedded object that names another collection's entity becomes a ref to it. Its other fields are a
    /// copy of that entity as of this document; they are not asserted (the entity's own collection says what
    /// is true about it), and the copy is reported.
    fn reference(
        &mut self,
        obj: &Map<String, Json>,
        attr: &Kw,
        field: &str,
    ) -> Option<ownpurse_facts::EntRef> {
        let (kind, id_field) = ref_rule(self.cfg, field)?;
        let Some(id) = obj.get(id_field).and_then(id_text) else {
            self.press(
                "no-identity",
                attr.to_string(),
                format!("a {kind} reference without {id_field}; kept as a component"),
            );
            return None;
        };
        let others: Vec<&str> = obj
            .keys()
            .filter(|k| *k != id_field)
            .map(String::as_str)
            .collect();
        if !others.is_empty() {
            self.press(
                "embedded-copy",
                attr.to_string(),
                format!(
                    "carries a copy of {kind} fields ({}); only the reference is stored",
                    others.join(", ")
                ),
            );
        }
        let did = doc_id(self.source_id, kind, &id);
        Some(
            self.snap
                .refer(&format!("doc:{did}"), &Kw::lit("doc/id"), Value::str(&did)),
        )
    }

    fn install(&self) -> Vec<Json> {
        self.defs
            .iter()
            .filter(|(k, _)| self.db.attr(k).is_none())
            .map(|(k, (ty, many, component))| {
                let mut d = json!({"ident": k.as_str(), "type": ty.name()});
                if *many {
                    d["many"] = json!(true);
                }
                if *component {
                    d["component"] = json!(true);
                }
                if k.name() == "path" {
                    d["unique"] = json!("identity");
                    d["doc"] = json!(
                        "Where a component sits in its document: <doc id>/<field>/<position or id>"
                    );
                }
                d
            })
            .collect()
    }
}
