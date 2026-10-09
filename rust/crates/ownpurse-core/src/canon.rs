//! Canonical JSON and JSON string escaping, shared by the CLI (output) and the API layer (content hashes and
//! the record's integrity chain must stay byte-identical across versions).

use serde_json::Value;
use std::fmt::Write;

/// A JSON string literal (`ascii`: escape everything outside ASCII).
pub fn json_str(out: &mut String, s: &str, ascii: bool) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c if ascii && (c as u32) > 0x7e => {
                let mut buf = [0u16; 2];
                for u in c.encode_utf16(&mut buf) {
                    let _ = write!(out, "\\u{u:04x}");
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Canonical JSON: sorted keys, compact separators, UTF-8 kept, numbers with their exact digits.
pub fn canonical(v: &Value) -> String {
    fn go(out: &mut String, v: &Value) {
        match v {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Number(n) => out.push_str(&py_number_text(&n.to_string())),
            Value::String(s) => json_str(out, s, false),
            Value::Array(a) => {
                out.push('[');
                for (i, x) in a.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    go(out, x);
                }
                out.push(']');
            }
            Value::Object(m) => {
                let mut keys: Vec<&String> = m.keys().collect();
                keys.sort();
                out.push('{');
                for (i, k) in keys.into_iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    json_str(out, k, false);
                    out.push(':');
                    go(out, &m[k]);
                }
                out.push('}');
            }
        }
    }
    let mut s = String::new();
    go(&mut s, v);
    s
}

/// A JSON number's text after decoding: integers stay integers, anything with a fraction or exponent
/// becomes an exact decimal whose text is used.
pub fn py_number_text(text: &str) -> String {
    if text.contains(['.', 'e', 'E']) {
        crate::money::parse(text)
            .map(|d| d.to_string())
            .unwrap_or_else(|_| text.to_string())
    } else {
        text.to_string()
    }
}
