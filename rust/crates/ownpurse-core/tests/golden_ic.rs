//! Intercompany parity with the frozen intercompany goldens, every field and key order, to the cent.
//!
//! Fixtures (committed): `ic-<as_of>.json`, regression goldens from `xtask demo fixtures` (the pairs and parties are in each golden). The real record runs when `OWNPURSE_GOLDEN_REAL=<dir>` and `OWNPURSE_GOLDEN_RECORD=<record>`
//! are set (local gate; never committed).

mod common;

use common::{assert_same, fixtures, goldens, read_json};
use ownpurse_core::Record;
use ownpurse_core::intercompany::{self as ic, IcContext};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Check one golden; returns (pairs, movement items, matched pairs).
fn check(record: &Record, path: &Path) -> (usize, usize, usize) {
    let g = read_json(path);
    let name = path.file_name().unwrap().to_string_lossy().to_string();
    let tenants: HashMap<String, String> = g["tenants"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
        .collect();
    let ctx = IcContext::new(
        ownpurse_core::view::View::main(record),
        tenants,
        &g["config"],
        g["as_of"].as_str().unwrap(),
    );
    let result = ic::build(&ctx, None).unwrap();
    assert_same(&format!("{name}:result"), &result, &g["result"]);
    assert_same(
        &format!("{name}:checks"),
        &ic::checks(&result),
        &g["checks"],
    );
    assert_same(
        &format!("{name}:envelope"),
        &ic::envelope(&result, "record.sqlite", "2026-10-05T00:00:00Z"),
        &g["envelope"],
    );
    let pairs = result["pairs"].as_array().unwrap().clone();
    for (mode, want) in g["sorted"].as_object().unwrap() {
        let mut v = pairs.clone();
        v.sort_by(ic::sort_cmp(mode));
        let got: Vec<Value> = v.iter().map(|p| p["id"].clone()).collect();
        assert_eq!(&Value::Array(got), want, "{name}: sort {mode}");
    }
    if let Some(first) = pairs.first() {
        let ids = vec![first["id"].as_str().unwrap().to_string()];
        assert_same(
            &format!("{name}:subset"),
            &ic::build(&ctx, Some(&ids)).unwrap(),
            &g["subset"],
        );
    }
    let items = pairs
        .iter()
        .map(|p| p["movement_items"].as_array().unwrap().len())
        .sum();
    let matched = pairs
        .iter()
        .map(|p| p["matched"].as_array().unwrap().len())
        .sum();
    (pairs.len(), items, matched)
}

fn run_dir(record_path: &Path, dir: &Path) -> (usize, usize, usize, usize) {
    let record = Record::open(record_path).unwrap();
    let calls = record.http_calls().unwrap();
    let mut tot = (0, 0, 0, 0);
    for p in goldens(dir, "ic-") {
        let (a, b, c) = check(&record, &p);
        tot = (tot.0 + 1, tot.1 + a, tot.2 + b, tot.3 + c);
    }
    assert_eq!(
        record.http_calls().unwrap(),
        calls,
        "never touches the network log"
    );
    tot
}

#[test]
fn fixture_ic_goldens_match() {
    let (files, pairs, items, matched) = run_dir(&fixtures().join("record.sqlite"), &fixtures());
    assert_eq!(files, 3);
    assert_eq!(pairs, 21);
    assert!(items > 5 && matched > 0, "{items} items, {matched} matched");
}

#[test]
fn fixture_statuses_are_exact() {
    let g = read_json(&fixtures().join("ic-2025-12-31.json"));
    let record = Record::open(fixtures().join("record.sqlite")).unwrap();
    let tenants = g["tenants"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
        .collect();
    let ctx = IcContext::new(
        ownpurse_core::view::View::main(&record),
        tenants,
        &g["config"],
        "2025-12-31",
    );
    let r = ic::build(&ctx, None).unwrap();
    let p = &r["pairs"][0];
    assert_eq!(
        (&p["a_net"], &p["b_party_net"], &p["difference"]),
        (&"1207.89".into(), &"-1003.26".into(), &"204.63".into())
    );
    assert_eq!(
        (&p["residual"], &p["status"]),
        (&"0.00".into(), &"bridged".into())
    );
    let statuses: Vec<&str> = r["pairs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["status"].as_str().unwrap())
        .collect();
    assert!(statuses.contains(&"unexplained") && statuses.contains(&"not supported"));
}

#[test]
fn real_record_ic_goldens_when_configured() {
    let (Ok(dir), Ok(rec)) = (
        std::env::var("OWNPURSE_GOLDEN_REAL"),
        std::env::var("OWNPURSE_GOLDEN_RECORD"),
    ) else {
        eprintln!("skipped: set OWNPURSE_GOLDEN_REAL and OWNPURSE_GOLDEN_RECORD");
        return;
    };
    let (files, pairs, items, matched) = run_dir(Path::new(&rec), Path::new(&dir));
    eprintln!(
        "real record ic: {files} golden files, {pairs} pairs, {items} movement items, {matched} matched pairs, \
         every field equal"
    );
    assert!(files >= 1);
}
