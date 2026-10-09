//! Golden parity with the frozen reference goldens, field by field and to the cent.
//!
//! Fixtures (committed): `rust/tests/fixtures/record.sqlite` + `golden-*.json` (regression goldens
//! written by `cargo run -p xtask -- demo fixtures`). The real record runs when `OWNPURSE_GOLDEN_REAL=<dir>` and
//! `OWNPURSE_GOLDEN_RECORD=<record.sqlite>` are set (local gate; never committed).

use ownpurse_core::{Ledger, Mode, Record, resolve};
use rust_decimal::Decimal;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

fn d(v: &Value) -> Decimal {
    Decimal::from_str(v.as_str().expect("decimal string")).expect("decimal")
}

fn check(record: &Record, golden_path: &Path) -> usize {
    let g: Value = serde_json::from_str(&std::fs::read_to_string(golden_path).unwrap()).unwrap();
    let tid = g["tenant_id"].as_str().unwrap();
    let as_of = g["as_of"].as_str().unwrap();
    let name = golden_path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();
    let l = Ledger::build(&ownpurse_core::view::View::main(record), tid).unwrap();
    let gp = g["postings"].as_array().unwrap();
    assert_eq!(l.postings.len(), gp.len(), "{name}: posting count");
    for (i, (p, e)) in l.postings.iter().zip(gp).enumerate() {
        let ctx = format!("{name} posting {i} ({} {})", p.source, p.source_id);
        assert_eq!(p.date, e["date"].as_str().unwrap(), "{ctx}: date");
        assert_eq!(p.source, e["source"].as_str().unwrap(), "{ctx}: source");
        assert_eq!(
            p.source_id,
            e["source_id"].as_str().unwrap(),
            "{ctx}: source_id"
        );
        assert_eq!(p.line as u64, e["line"].as_u64().unwrap(), "{ctx}: line");
        assert_eq!(
            p.account_id,
            e["account_id"].as_str().unwrap(),
            "{ctx}: account"
        );
        assert_eq!(p.amount, d(&e["amount"]), "{ctx}: amount");
        assert_eq!(
            p.description,
            e["description"].as_str().unwrap(),
            "{ctx}: description"
        );
        assert_eq!(p.contact, e["contact"].as_str().unwrap(), "{ctx}: contact");
        assert_eq!(
            p.reference,
            e["reference"].as_str().unwrap(),
            "{ctx}: reference"
        );
        assert_eq!(p.kind, e["kind"].as_str().unwrap(), "{ctx}: kind");
    }
    let gu = g["unbalanced"].as_array().unwrap();
    assert_eq!(l.unbalanced.len(), gu.len(), "{name}: unbalanced count");
    for (u, e) in l.unbalanced.iter().zip(gu) {
        assert_eq!(
            (u.source, u.id.as_str(), u.date.as_str()),
            (
                e["source"].as_str().unwrap(),
                e["id"].as_str().unwrap(),
                e["date"].as_str().unwrap()
            )
        );
        assert_eq!(u.imbalance, d(&e["imbalance"]), "{name}: imbalance");
    }
    let skipped: std::collections::BTreeMap<String, usize> = g["skipped"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_u64().unwrap() as usize))
        .collect();
    assert_eq!(l.skipped, skipped, "{name}: skipped");
    let fy = record.fy_end(tid).unwrap();
    for (mode_name, mode) in [("auto", Mode::Auto), ("history", Mode::History)] {
        let gb = &g["bases"][mode_name];
        let ctx = format!("{name} basis {mode_name}");
        let b = resolve(record, &l, tid, as_of, fy, mode).unwrap();
        let kind = match b.kind {
            ownpurse_core::BasisKind::XeroOpening => "xero-opening",
            _ => "history",
        };
        assert_eq!(kind, gb["kind"].as_str().unwrap(), "{ctx}: kind");
        assert_eq!(
            b.fy_start,
            gb["fy_start"].as_str().unwrap(),
            "{ctx}: fy_start"
        );
        assert_eq!(
            b.opening_date.as_deref(),
            gb["opening_date"].as_str(),
            "{ctx}: opening_date"
        );
        assert_eq!(
            b.matches_xero(),
            gb["matches_xero"].as_bool(),
            "{ctx}: matches_xero"
        );
        let gbal = gb["balances"].as_object().unwrap();
        assert_eq!(b.balances.len(), gbal.len(), "{ctx}: balance count");
        for (k, v) in gbal {
            assert_eq!(b.balances.get(k).copied(), Some(d(v)), "{ctx}: balance {k}");
        }
        let gd = gb["diffs"].as_array().unwrap();
        assert_eq!(b.diffs.len(), gd.len(), "{ctx}: diff count");
        for (x, e) in b.diffs.iter().zip(gd) {
            assert_eq!(
                x.account_id,
                e["account_id"].as_str().unwrap(),
                "{ctx}: diff account"
            );
            assert_eq!(
                (x.local, x.xero, x.difference),
                (d(&e["local"]), d(&e["xero"]), d(&e["difference"])),
                "{ctx}: diff {}",
                x.account_id
            );
        }
        for (aid, v) in gb["opening_for"].as_object().unwrap() {
            assert_eq!(
                b.opening_for(&l, aid, &b.fy_start),
                d(v),
                "{ctx}: opening_for {aid}"
            );
        }
    }
    gp.len()
}

fn run_dir(record_path: &Path, dir: &Path) -> (usize, usize) {
    let record = Record::open(record_path).unwrap();
    let calls = record.http_calls().unwrap();
    let mut files = 0;
    let mut postings = 0;
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with("golden-"))
        })
        .collect();
    paths.sort();
    for p in paths {
        postings += check(&record, &p);
        files += 1;
    }
    assert_eq!(
        record.http_calls().unwrap(),
        calls,
        "golden runs must never touch the network log"
    );
    (files, postings)
}

#[test]
fn fixture_goldens_match_to_the_cent() {
    let (files, postings) = run_dir(&fixtures().join("record.sqlite"), &fixtures());
    assert_eq!(files, 9);
    assert!(postings > 3000);
}

#[test]
fn fixture_reports_the_deliberate_difference() {
    let record = Record::open(fixtures().join("record.sqlite")).unwrap();
    let tenants: Value =
        serde_json::from_str(&std::fs::read_to_string(fixtures().join("tenants.json")).unwrap())
            .unwrap();
    let tid = tenants["one"].as_str().unwrap();
    let l = Ledger::build(&ownpurse_core::view::View::main(&record), tid).unwrap();
    let b = resolve(&record, &l, tid, "2025-12-31", (12, 31), Mode::Auto).unwrap();
    assert_eq!(b.matches_xero(), Some(false));
    assert_eq!(b.diffs.len(), 1);
    assert_eq!(b.diffs[0].difference, Decimal::new(-37, 2));
}

#[test]
fn real_record_goldens_when_configured() {
    let (Ok(dir), Ok(rec)) = (
        std::env::var("OWNPURSE_GOLDEN_REAL"),
        std::env::var("OWNPURSE_GOLDEN_RECORD"),
    ) else {
        eprintln!(
            "skipped: set OWNPURSE_GOLDEN_REAL and OWNPURSE_GOLDEN_RECORD to run the real-record goldens"
        );
        return;
    };
    let (files, postings) = run_dir(Path::new(&rec), Path::new(&dir));
    eprintln!("real record: {files} golden files, {postings} postings, all equal to the cent");
    assert!(files >= 1);
}
