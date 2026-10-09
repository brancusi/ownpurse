//! A minimal JSON Schema check for the versioned reply schemas in docs/schema (the subset they use: type,
//! required, properties, additionalProperties: false, items, minItems).

use serde_json::Value;

fn type_ok(t: &str, v: &Value) -> bool {
    match t {
        "object" => v.is_object(),
        "array" => v.is_array(),
        "string" => v.is_string(),
        "integer" => v.as_i64().is_some() || v.as_u64().is_some(),
        "number" => v.is_number(),
        "boolean" => v.is_boolean(),
        "null" => v.is_null(),
        _ => true,
    }
}

/// Problems with `v` against `schema` (empty: valid).
pub fn check(schema: &Value, v: &Value, at: &str) -> Vec<String> {
    let mut out = vec![];
    if let Some(t) = schema.get("type") {
        let ok = match t {
            Value::String(t) => type_ok(t, v),
            Value::Array(ts) => ts.iter().filter_map(Value::as_str).any(|t| type_ok(t, v)),
            _ => true,
        };
        if !ok {
            out.push(format!("{at}: {} is not {t}", short(v)));
            return out;
        }
    }
    if let (Some(req), Some(m)) = (
        schema.get("required").and_then(Value::as_array),
        v.as_object(),
    ) {
        for k in req.iter().filter_map(Value::as_str) {
            if !m.contains_key(k) {
                out.push(format!("{at}: missing `{k}`"));
            }
        }
    }
    if let (Some(props), Some(m)) = (
        schema.get("properties").and_then(Value::as_object),
        v.as_object(),
    ) {
        for (k, x) in m {
            match props.get(k) {
                Some(s) => out.extend(check(s, x, &format!("{at}.{k}"))),
                None if schema.get("additionalProperties") == Some(&Value::Bool(false)) => {
                    out.push(format!("{at}: `{k}` is not in the schema"));
                }
                None => {}
            }
        }
    }
    if let (Some(items), Some(a)) = (schema.get("items"), v.as_array()) {
        for (i, x) in a.iter().enumerate() {
            out.extend(check(items, x, &format!("{at}[{i}]")));
        }
    }
    if let (Some(n), Some(a)) = (schema.get("minItems").and_then(Value::as_u64), v.as_array())
        && (a.len() as u64) < n
    {
        out.push(format!("{at}: fewer than {n} items"));
    }
    out
}

fn short(v: &Value) -> String {
    let s = v.to_string();
    if s.len() > 60 {
        format!("{}…", &s[..60])
    } else {
        s
    }
}

pub fn load(name: &str) -> Value {
    let p = crate::demo::fixtures_dir()
        .join("../../../docs/schema")
        .join(name);
    std::fs::read_to_string(&p)
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or(Value::Null)
}
