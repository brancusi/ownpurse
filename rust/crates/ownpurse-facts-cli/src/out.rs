//! Output: compact text by default, JSON with `--json`.

use crate::load::Loaded;
use ownpurse_facts::{Db, Eid, Kw, TxReport, Value};
use rusqlite::Connection;
use rusqlite::types::ValueRef;
use serde_json::{Value as Json, json};
use std::collections::BTreeMap;

pub fn emit(as_json: bool, j: &Json, text: &str) {
    if as_json {
        println!("{j}");
    } else {
        println!("{text}");
    }
}

pub fn json_pretty(j: &Json) {
    println!("{}", serde_json::to_string_pretty(j).unwrap_or_default());
}

fn cell(v: ValueRef<'_>) -> Json {
    match v {
        ValueRef::Null => Json::Null,
        ValueRef::Integer(i) => json!(i),
        ValueRef::Real(f) => json!(f),
        ValueRef::Text(t) => json!(String::from_utf8_lossy(t)),
        ValueRef::Blob(b) => json!(format!("<{} bytes>", b.len())),
    }
}

fn text(j: &Json) -> String {
    match j {
        Json::Null => String::new(),
        Json::String(s) => s.clone(),
        other => other.to_string(),
    }
}

fn clip(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        return s.to_string();
    }
    let mut t: String = s.chars().take(n - 1).collect();
    t.push('…');
    t
}

/// Print rows as an aligned table (text) or an array of objects (JSON).
pub fn table(as_json: bool, cols: &[String], rows: &[Vec<Json>]) {
    if as_json {
        let objs: Vec<Json> = rows
            .iter()
            .map(|r| Json::Object(cols.iter().cloned().zip(r.iter().cloned()).collect()))
            .collect();
        println!("{}", json!({"rows": objs, "count": rows.len()}));
        return;
    }
    let cells: Vec<Vec<String>> = rows
        .iter()
        .map(|r| r.iter().map(|c| clip(&text(c), 64)).collect())
        .collect();
    let width = |i: usize| {
        cells
            .iter()
            .map(|r| r[i].chars().count())
            .chain(std::iter::once(cols[i].chars().count()))
            .max()
            .unwrap_or(0)
    };
    let widths: Vec<usize> = (0..cols.len()).map(width).collect();
    let line = |vals: Vec<String>| {
        vals.iter()
            .zip(&widths)
            .map(|(v, w)| format!("{v:<w$}"))
            .collect::<Vec<_>>()
            .join("  ")
            .trim_end()
            .to_string()
    };
    println!("{}", line(cols.to_vec()));
    for r in cells {
        println!("{}", line(r));
    }
    println!(
        "({} {})",
        rows.len(),
        if rows.len() == 1 { "row" } else { "rows" }
    );
}

pub fn sql(as_json: bool, conn: &Connection, q: &str) -> Result<(), String> {
    let mut st = conn.prepare(q).map_err(|e| format!("sql: {e}"))?;
    let cols: Vec<String> = st.column_names().iter().map(|c| (*c).to_string()).collect();
    let n = cols.len();
    let rows = st
        .query_map([], |r| {
            (0..n)
                .map(|i| r.get_ref(i).map(cell))
                .collect::<Result<Vec<_>, _>>()
        })
        .map_err(|e| format!("sql: {e}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| format!("sql: {e}"))?;
    table(as_json, &cols, &rows);
    Ok(())
}

pub fn loaded(as_json: bool, done: &[Loaded]) {
    let cols: Vec<String> = ["file", "projector", "tx", "+", "-", "attrs", "pressure"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    let rows: Vec<Vec<Json>> = done
        .iter()
        .map(|l| {
            let f = l
                .file
                .rsplit('/')
                .take(2)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("/");
            vec![
                json!(f),
                json!(l.projector),
                l.tx.map_or(json!("no change"), |t| json!(t)),
                json!(l.asserted),
                json!(l.retracted),
                json!(l.installed),
                json!(l.pressure.len()),
            ]
        })
        .collect();
    table(as_json, &cols, &rows);
    if !as_json {
        println!(
            "help[2]: ownpurse-facts pressure    ownpurse-facts sql \"SELECT * FROM observations\""
        );
    }
}

fn ident(db: &Db, a: Eid) -> String {
    db.schema()
        .ident(a)
        .map_or_else(|| a.to_string(), Kw::to_string)
}

pub fn report(as_json: bool, db: &Db, r: &TxReport, dry: bool) {
    let rows: Vec<Vec<Json>> = r
        .datoms
        .iter()
        .map(|d| {
            vec![
                json!(if d.added { "+" } else { "-" }),
                json!(d.e.0),
                json!(ident(db, d.a)),
                d.v.to_json(),
            ]
        })
        .collect();
    if !as_json {
        match r.tx {
            Some(t) if dry => println!("would write transaction {t} (dry run: nothing written)"),
            Some(t) => println!("transaction {t}"),
            None => println!("no change: the log already says this"),
        }
    }
    let cols: Vec<String> = ["op", "e", "attr", "value"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    table(as_json, &cols, &rows);
}

pub fn history(as_json: bool, db: &Db, e: Eid) {
    let get = |t: Eid, a: &'static str| db.get(t, &Kw::lit(a)).map(Value::text).unwrap_or_default();
    let rows: Vec<Vec<Json>> = db
        .history(e)
        .iter()
        .map(|d| {
            vec![
                json!(d.tx.0),
                json!(get(d.tx, "tx/instant")),
                json!(if d.added { "+" } else { "-" }),
                json!(ident(db, d.a)),
                d.v.to_json(),
                json!(get(d.tx, "tx/actor")),
            ]
        })
        .collect();
    let cols: Vec<String> = ["tx", "instant", "op", "attr", "value", "by"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    table(as_json, &cols, &rows);
}

pub fn schema(as_json: bool, db: &Db, prefix: Option<&str>) {
    let rows: Vec<Vec<Json>> = db
        .schema()
        .iter()
        .filter(|d| prefix.is_none_or(|p| d.ident.as_str().starts_with(p)))
        .map(|d| {
            let mut flags = Vec::new();
            if d.many {
                flags.push("many");
            }
            match d.unique {
                Some(ownpurse_facts::Unique::Identity) => flags.push("identity"),
                Some(ownpurse_facts::Unique::Value) => flags.push("unique"),
                None => {}
            }
            if d.component {
                flags.push("component");
            }
            vec![
                json!(d.ident.as_str()),
                json!(d.ty.name()),
                json!(flags.join(" ")),
                json!(d.doc.clone().unwrap_or_default()),
            ]
        })
        .collect();
    let cols: Vec<String> = ["attribute", "type", "flags", "doc"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    table(as_json, &cols, &rows);
}

pub fn pressure(as_json: bool, db: &Db, all: bool) {
    let Some(a) = db.attr(&Kw::lit("tx/pressure")) else {
        return;
    };
    let mut by: BTreeMap<String, Vec<(String, String, String)>> = BTreeMap::new();
    for (v, tx) in db.attr_values(a.id) {
        let line = v.text();
        let mut parts = line.splitn(3, " | ");
        let (k, at, detail) = (
            parts.next().unwrap_or(""),
            parts.next().unwrap_or(""),
            parts.next().unwrap_or(""),
        );
        let proj = db
            .get(tx, &Kw::lit("tx/projector"))
            .map(Value::text)
            .unwrap_or_default();
        by.entry(k.to_string())
            .or_default()
            .push((proj, at.to_string(), detail.to_string()));
    }
    let mut rows = Vec::new();
    for (k, items) in &by {
        let mut seen: BTreeMap<(String, String), (String, usize)> = BTreeMap::new();
        for (p, at, d) in items {
            let e = seen
                .entry((p.clone(), at.clone()))
                .or_insert((d.clone(), 0));
            e.1 += 1;
        }
        if all {
            for ((p, at), (d, n)) in seen {
                rows.push(vec![json!(k), json!(p), json!(at), json!(n), json!(d)]);
            }
        } else {
            let projs: std::collections::BTreeSet<&str> =
                items.iter().map(|(p, _, _)| p.as_str()).collect();
            let example = items
                .first()
                .map(|(_, at, d)| format!("{at}: {d}"))
                .unwrap_or_default();
            rows.push(vec![
                json!(k),
                json!(projs.into_iter().collect::<Vec<_>>().join(" ")),
                json!(""),
                json!(seen.len()),
                json!(example),
            ]);
        }
    }
    let cols: Vec<String> = ["kind", "projectors", "at", "places", "detail"]
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    table(as_json, &cols, &rows);
}
