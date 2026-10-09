//! Shared helpers for the golden tests: fixture paths and a field-by-field JSON comparison that reports the
//! exact path of the first difference (key order included, so the CLI can emit byte-identical JSON).

#![allow(dead_code)]

use rust_decimal::Decimal;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

pub fn read_json(p: &Path) -> Value {
    serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap()
}

/// Golden files in `dir` whose name starts with `prefix`, sorted.
pub fn goldens(dir: &Path, prefix: &str) -> Vec<PathBuf> {
    let mut v: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(prefix))
        })
        .collect();
    v.sort();
    v
}

fn dec(s: &str) -> Option<Decimal> {
    Decimal::from_str(s).ok()
}

/// Compare every field. Strings must be identical; when they differ but both are decimals, the failure
/// names the numeric values (a value mismatch) or says the difference is the scale only. Numbers compare
/// as decimals; objects must have the same keys in the same order.
pub fn assert_same(path: &str, rust: &Value, golden: &Value) {
    match (rust, golden) {
        (Value::Object(a), Value::Object(b)) => {
            let ka: Vec<&String> = a.keys().collect();
            let kb: Vec<&String> = b.keys().collect();
            assert_eq!(ka, kb, "{path}: keys (or key order) differ");
            for (k, v) in a {
                assert_same(&format!("{path}.{k}"), v, &b[k]);
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            assert_eq!(a.len(), b.len(), "{path}: array length");
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                assert_same(&format!("{path}[{i}]"), x, y);
            }
        }
        (Value::String(a), Value::String(b)) => {
            if a != b {
                match (dec(a), dec(b)) {
                    (Some(x), Some(y)) if x == y => {
                        panic!("{path}: same value, different text (scale): got {a:?} golden {b:?}")
                    }
                    (Some(x), Some(y)) => panic!("{path}: value mismatch: got {x} golden {y}"),
                    _ => panic!("{path}: got {a:?} != golden {b:?}"),
                }
            }
        }
        (Value::Number(a), Value::Number(b)) => {
            assert_eq!(
                dec(&a.to_string()),
                dec(&b.to_string()),
                "{path}: number mismatch"
            );
        }
        _ => assert_eq!(rust, golden, "{path}"),
    }
}
