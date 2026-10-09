//! axi.md-style output: compact TOON-like text by default, `--json` for
//! machines. Every response is optional scalar fields, optional tables (`name[count]{cols}:` rows) and a
//! `help[n]:` block of next steps. Errors use the same shape on stdout.
//!
//! JSON is written with a one-space indent, ASCII-escaped. Decimals
//! travel as JSON strings holding the decimal's exact text (scale kept).

pub use ownpurse_core::canon::canonical;
use ownpurse_core::canon::{json_str, py_number_text};
use rust_decimal::Decimal;
use serde_json::{Map, Value};
use std::fmt::Write as _;

/// A decimal as its exact text.
pub fn dec(d: Decimal) -> Value {
    Value::String(d.to_string())
}

pub fn opt_dec(d: Option<Decimal>) -> Value {
    d.map(dec).unwrap_or(Value::Null)
}

/// Build an ordered JSON object: `obj![("a", v), ...]`.
#[macro_export]
macro_rules! obj {
    ($(($k:expr, $v:expr)),* $(,)?) => {{
        #[allow(unused_mut)]
        let mut m = serde_json::Map::new();
        $( m.insert(($k).to_string(), serde_json::Value::from($v)); )*
        m
    }};
}

pub struct Table {
    pub name: String,
    pub cols: Vec<String>,
    pub rows: Vec<Map<String, Value>>,
    pub total: Option<usize>,
}

#[derive(Default)]
pub struct Out {
    pub as_json: bool,
    pub fields: Vec<(String, Value)>,
    pub tables: Vec<Table>,
    pub hints: Vec<String>,
}

impl Out {
    pub fn new(as_json: bool) -> Out {
        Out {
            as_json,
            ..Default::default()
        }
    }

    /// Set a field (a repeated key keeps its first position).
    pub fn field(&mut self, key: &str, value: impl Into<Value>) -> &mut Self {
        let value = value.into();
        if let Some(slot) = self.fields.iter_mut().find(|(k, _)| k == key) {
            slot.1 = value;
        } else {
            self.fields.push((key.to_string(), value));
        }
        self
    }

    pub fn table(
        &mut self,
        name: &str,
        cols: &[&str],
        rows: Vec<Map<String, Value>>,
        total: Option<usize>,
    ) -> &mut Self {
        self.tables.push(Table {
            name: name.into(),
            cols: cols.iter().map(|c| c.to_string()).collect(),
            rows,
            total,
        });
        self
    }

    pub fn table_cols(
        &mut self,
        name: &str,
        cols: Vec<String>,
        rows: Vec<Map<String, Value>>,
        total: Option<usize>,
    ) -> &mut Self {
        self.tables.push(Table {
            name: name.into(),
            cols,
            rows,
            total,
        });
        self
    }

    pub fn hint(&mut self, line: impl Into<String>) -> &mut Self {
        self.hints.push(line.into());
        self
    }

    pub fn render(&self) -> String {
        if self.as_json {
            let mut doc = Map::new();
            for (k, v) in &self.fields {
                doc.insert(k.clone(), v.clone());
            }
            for t in &self.tables {
                doc.insert(
                    t.name.clone(),
                    Value::Array(t.rows.iter().cloned().map(Value::Object).collect()),
                );
                doc.insert(
                    format!("{}_count", t.name),
                    Value::from(t.total.unwrap_or(t.rows.len())),
                );
            }
            if !self.hints.is_empty() {
                doc.insert(
                    "help".into(),
                    Value::Array(self.hints.iter().cloned().map(Value::String).collect()),
                );
            }
            let mut s = py_json(&Value::Object(doc));
            s.push('\n');
            return s;
        }
        let mut lines: Vec<String> = Vec::new();
        for (k, v) in &self.fields {
            match v {
                Value::Array(a) => lines.push(format!(
                    "{k}: {}",
                    if a.is_empty() {
                        "(none)".to_string()
                    } else {
                        a.iter().map(cell).collect::<Vec<_>>().join(", ")
                    }
                )),
                Value::Object(m) => lines.push(format!(
                    "{k}: {}",
                    m.iter()
                        .map(|(a, b)| format!("{a}={}", cell(b)))
                        .collect::<Vec<_>>()
                        .join(", ")
                )),
                other => lines.push(format!("{k}: {}", cell(other))),
            }
        }
        for t in &self.tables {
            let count = t.total.unwrap_or(t.rows.len());
            let shown = if t.rows.len() == count {
                count.to_string()
            } else {
                format!("{} of {count}", t.rows.len())
            };
            if t.rows.is_empty() {
                lines.push(format!("{}[0]: none", t.name));
                continue;
            }
            lines.push(format!("{}[{shown}]{{{}}}:", t.name, t.cols.join(",")));
            for r in &t.rows {
                let cells: Vec<String> = t
                    .cols
                    .iter()
                    .map(|c| cell(r.get(c).unwrap_or(&Value::Null)))
                    .collect();
                lines.push(format!("  {}", cells.join(",")));
            }
        }
        if !self.hints.is_empty() {
            lines.push(format!("help[{}]:", self.hints.len()));
            for h in &self.hints {
                lines.push(format!("  {h}"));
            }
        }
        let mut s = lines.join("\n");
        s.push('\n');
        s
    }

    pub fn emit(&self) {
        use std::io::Write;
        let mut so = std::io::stdout().lock();
        let _ = so.write_all(self.render().as_bytes());
        let _ = so.flush();
    }
}

/// One table cell: quoted when it holds a comma, a quote, a newline or edge spaces.
pub fn cell(v: &Value) -> String {
    let s = match v {
        Value::Null => return String::new(),
        Value::Bool(b) => return if *b { "true".into() } else { "false".into() },
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        other => py_str(other),
    };
    if s.contains([',', '"', '\n']) || s != s.trim() {
        format!("\"{}\"", s.replace('"', "\"\"").replace('\n', " "))
    } else {
        s
    }
}

/// The text form of a JSON-decoded value (`True`, `None`, dict/list reprs).
pub fn py_str(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => py_repr(other),
    }
}

/// The repr form of a JSON-decoded value.
pub fn py_repr(v: &Value) -> String {
    match v {
        Value::Null => "None".into(),
        Value::Bool(true) => "True".into(),
        Value::Bool(false) => "False".into(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => py_repr_str(s),
        Value::Array(a) => format!("[{}]", a.iter().map(py_repr).collect::<Vec<_>>().join(", ")),
        Value::Object(m) => format!(
            "{{{}}}",
            m.iter()
                .map(|(k, v)| format!("{}: {}", py_repr_str(k), py_repr(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

fn py_repr_str(s: &str) -> String {
    let q = if s.contains('\'') && !s.contains('"') {
        '"'
    } else {
        '\''
    };
    let mut out = String::with_capacity(s.len() + 2);
    out.push(q);
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == q => {
                out.push('\\');
                out.push(c);
            }
            c if (c as u32) < 0x20 || c as u32 == 0x7f => {
                let _ = write!(out, "\\x{:02x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push(q);
    out
}

/// JSON with a one-space indent (ASCII-escaped, `", "`/`": "` separators).
pub fn py_json(v: &Value) -> String {
    py_json_opts(v, true)
}

/// JSON with a one-space indent; `ascii` escapes everything outside ASCII.
pub fn py_json_opts(v: &Value, ascii: bool) -> String {
    fn go(out: &mut String, v: &Value, level: usize, ascii: bool) {
        match v {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Number(n) => out.push_str(&n.to_string()),
            Value::String(s) => json_str(out, s, ascii),
            Value::Array(a) => {
                if a.is_empty() {
                    out.push_str("[]");
                    return;
                }
                out.push('[');
                for (i, x) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push('\n');
                    out.push_str(&" ".repeat(level + 1));
                    go(out, x, level + 1, ascii);
                }
                out.push('\n');
                out.push_str(&" ".repeat(level));
                out.push(']');
            }
            Value::Object(m) => {
                if m.is_empty() {
                    out.push_str("{}");
                    return;
                }
                out.push('{');
                for (i, (k, x)) in m.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    out.push('\n');
                    out.push_str(&" ".repeat(level + 1));
                    json_str(out, k, ascii);
                    out.push_str(": ");
                    go(out, x, level + 1, ascii);
                }
                out.push('\n');
                out.push_str(&" ".repeat(level));
                out.push('}');
            }
        }
    }
    let mut s = String::new();
    go(&mut s, v, 0, ascii);
    s
}

/// A payload value as the JSON output carries it: non-integer
/// numbers become strings (Decimal), recursively.
pub fn py_loaded(v: &Value) -> Value {
    match v {
        Value::Number(n) => {
            let t = n.to_string();
            if t.contains(['.', 'e', 'E']) {
                Value::String(py_number_text(&t))
            } else {
                v.clone()
            }
        }
        Value::Array(a) => Value::Array(a.iter().map(py_loaded).collect()),
        Value::Object(m) => {
            Value::Object(m.iter().map(|(k, x)| (k.clone(), py_loaded(x))).collect())
        }
        other => other.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_layout_is_stable() {
        let v = json!({"a": 1, "b": [1, {"c": "é→"}], "e": [], "f": {}});
        assert_eq!(
            py_json(&v),
            "{\n \"a\": 1,\n \"b\": [\n  1,\n  {\n   \"c\": \"\\u00e9\\u2192\"\n  }\n ],\n \"e\": [],\n \"f\": {}\n}"
        );
        assert_eq!(py_json(&json!("😀")), "\"\\ud83d\\ude00\"");
    }

    #[test]
    fn canonical_sorts_and_keeps_utf8() {
        let v: Value =
            serde_json::from_str(r#"{"b": 1.50, "a": "é\n", "c": [true, null]}"#).unwrap();
        assert_eq!(canonical(&v), r#"{"a":"é\n","b":1.50,"c":[true,null]}"#);
    }

    #[test]
    fn cells_and_reprs() {
        assert_eq!(cell(&json!("a,b")), "\"a,b\"");
        assert_eq!(cell(&json!(" x")), "\" x\"");
        assert_eq!(cell(&json!("say \"hi\"")), "\"say \"\"hi\"\"\"");
        assert_eq!(cell(&json!(true)), "true");
        assert_eq!(cell(&Value::Null), "");
        assert_eq!(
            py_repr(&json!({"id": "x", "n": null, "l": ["it's"], "b": false})),
            "{'id': 'x', 'n': None, 'l': [\"it's\"], 'b': False}"
        );
        assert_eq!(py_repr(&json!("a\\b")), "'a\\\\b'");
    }

    #[test]
    fn text_render() {
        let mut o = Out::new(false);
        o.field("as_of", "2025-12-31")
            .field("list", json!([]))
            .field("totals", json!({"calls": 1}));
        o.table("t", &["a", "b"], vec![obj![("a", 1), ("b", "x")]], Some(3));
        o.table("e", &["a"], vec![], None);
        o.hint("next");
        assert_eq!(
            o.render(),
            "as_of: 2025-12-31\nlist: (none)\ntotals: calls=1\nt[1 of 3]{a,b}:\n  1,x\ne[0]: none\nhelp[1]:\n  next\n"
        );
    }
}
