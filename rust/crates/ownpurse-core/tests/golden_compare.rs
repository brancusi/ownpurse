//! Compare parity with the frozen compare goldens (`tb_compare` + `flatten` over the TUI's TB rows) and
//! `postings_compare`, every field and key order, to the cent.
//!
//! Fixtures (committed): `compare-<orgs>-<align>[-history].json` (regression goldens from `xtask demo fixtures`). The
//! real record runs when `OWNPURSE_GOLDEN_REAL=<dir>` and `OWNPURSE_GOLDEN_RECORD=<record>` are set.

mod common;

use common::{assert_same, fixtures, goldens, read_json};
use ownpurse_core::compare::{self as cmp, D0, RowStyle};
use ownpurse_core::{Basis, Chart, Ledger, Mode, Record, resolve};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Check one golden; returns (aligned rows, only rows, spine lines).
fn check(record: &Record, path: &Path) -> (usize, usize, usize) {
    let g = read_json(path);
    let name = path.file_name().unwrap().to_string_lossy().to_string();
    let orgs: Vec<String> = g["orgs"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o.as_str().unwrap().to_string())
        .collect();
    let as_of = g["as_of"].as_str().unwrap();
    let mode = if g["mode"] == "history" {
        Mode::History
    } else {
        Mode::Auto
    };
    let mut ledgers: HashMap<String, Ledger> = HashMap::new();
    let mut bases: HashMap<String, Basis> = HashMap::new();
    for o in &orgs {
        let tid = g["tenants"][o.as_str()].as_str().unwrap();
        let l = Ledger::build(&ownpurse_core::view::View::main(record), tid).unwrap();
        let b = resolve(record, &l, tid, as_of, record.fy_end(tid).unwrap(), mode).unwrap();
        ledgers.insert(o.clone(), l);
        bases.insert(o.clone(), b);
    }
    let tbs: HashMap<String, cmp::TbColumn> = orgs
        .iter()
        .map(|o| {
            (
                o.clone(),
                cmp::tb_rows(&ledgers[o], &bases[o], RowStyle::Tui),
            )
        })
        .collect();
    let charts: HashMap<String, &Chart> = orgs
        .iter()
        .map(|o| (o.clone(), &ledgers[o].chart))
        .collect();
    let result = cmp::tb_compare(
        &orgs,
        &tbs,
        &charts,
        g["align"].as_str().unwrap(),
        &g["account_map"],
    )
    .unwrap();
    assert_same(&format!("{name}:result"), &result, &g["result"]);
    let rows = cmp::flatten(&result).unwrap();
    assert_same(
        &format!("{name}:flatten"),
        &Value::Array(rows.clone()),
        &g["flatten"],
    );
    let since = bases[&orgs[0]].fy_start.clone();
    let lrefs: HashMap<String, &Ledger> = ledgers.iter().map(|(k, v)| (k.clone(), v)).collect();
    let mut spine = 0;
    for (i, want) in g["postings"].as_array().unwrap().iter().enumerate() {
        let row = rows
            .iter()
            .find(|r| r["key"] == want["key"])
            .expect("row by key");
        let ids: HashMap<String, Vec<String>> = row["member_ids"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(o, v)| {
                (
                    o.clone(),
                    v.as_array()
                        .unwrap()
                        .iter()
                        .map(|x| x.as_str().unwrap().to_string())
                        .collect(),
                )
            })
            .collect();
        let openings: HashMap<String, rust_decimal::Decimal> = orgs
            .iter()
            .map(|o| {
                let total = ids.get(o).into_iter().flatten().fold(D0, |acc, aid| {
                    ownpurse_core::money::add(acc, bases[o].opening_for(&ledgers[o], aid, &since))
                });
                (o.clone(), total)
            })
            .collect();
        let ov: serde_json::Map<String, Value> = orgs
            .iter()
            .map(|o| (o.clone(), cmp::dstr(openings[o])))
            .collect();
        assert_same(
            &format!("{name}:postings[{i}].openings"),
            &Value::Object(ov),
            &want["openings"],
        );
        let pc = cmp::postings_compare(&orgs, &lrefs, &ids, &since, as_of, &openings).unwrap();
        assert_same(
            &format!("{name}:postings[{i}].result"),
            &pc,
            &want["result"],
        );
        spine += pc["spine"].as_array().unwrap().len();
    }
    let aligned = rows.iter().filter(|r| r["section"] == "aligned").count();
    (aligned, rows.len() - aligned, spine)
}

fn run_dir(record_path: &Path, dir: &Path) -> (usize, usize, usize, usize) {
    let record = Record::open(record_path).unwrap();
    let calls = record.http_calls().unwrap();
    let mut tot = (0, 0, 0, 0);
    for p in goldens(dir, "compare-") {
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
fn fixture_compare_goldens_match() {
    let (files, aligned, only, spine) = run_dir(&fixtures().join("record.sqlite"), &fixtures());
    assert_eq!(files, 5);
    assert!(
        aligned > 5 && only > 5 && spine > 5,
        "{aligned} aligned, {only} only, {spine} spine"
    );
}

#[test]
fn real_record_compare_goldens_when_configured() {
    let (Ok(dir), Ok(rec)) = (
        std::env::var("OWNPURSE_GOLDEN_REAL"),
        std::env::var("OWNPURSE_GOLDEN_RECORD"),
    ) else {
        eprintln!("skipped: set OWNPURSE_GOLDEN_REAL and OWNPURSE_GOLDEN_RECORD");
        return;
    };
    let (files, aligned, only, spine) = run_dir(Path::new(&rec), Path::new(&dir));
    eprintln!(
        "real record compare: {files} golden files, {aligned} aligned rows, {only} only-in rows, {spine} \
         posting spine lines, every field equal"
    );
    assert!(files >= 1);
}
