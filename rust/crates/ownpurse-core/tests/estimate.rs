//! `estimate` against the frozen estimate golden on the demo fixture (and its variants) and the CLI fixture,
//! clock pinned (golden: `demo/estimate-2026-10-05.json`, a regression golden written by `cargo run -p xtask -- demo fixtures`).

use ownpurse_core::Record;
use ownpurse_core::estimate::{EstimateSettings, estimate};
use serde_json::Value;
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

const NOW: i64 = 1_791_193_560; // 2026-10-05T09:46:00Z

fn check(record: &Path, tenants: &[(&str, &str)], want: &Value) {
    let rec = Record::open(record).unwrap();
    for (alias, tid) in tenants {
        for (key, full) in [("estimate", false), ("full", true)] {
            let got = estimate(&rec, tid, None, full, EstimateSettings::default(), NOW).unwrap();
            let exp = want[*alias][key].as_i64();
            assert_eq!(got, exp, "{} {alias} {key}", record.display());
        }
    }
}

#[test]
fn matches_the_golden_on_demo_and_cli_fixtures() {
    assert_eq!(
        ownpurse_core::dates::days_from_civil(2026, 10, 5) * 86_400 + 9 * 3600 + 46 * 60,
        NOW
    );
    let demo = fixtures().join("demo");
    let golden: Value = serde_json::from_str(
        &std::fs::read_to_string(demo.join("estimate-2026-10-05.json")).unwrap(),
    )
    .unwrap();
    let tenants: Value =
        serde_json::from_str(&std::fs::read_to_string(demo.join("tenants.json")).unwrap()).unwrap();
    let t: Vec<(&str, &str)> = tenants
        .as_object()
        .unwrap()
        .iter()
        .map(|(a, v)| (a.as_str(), v.as_str().unwrap()))
        .collect();
    for (variant, want) in golden["variants"].as_object().unwrap() {
        let file = if variant == "fresh" {
            "record.sqlite".to_string()
        } else {
            format!("record-{variant}.sqlite")
        };
        check(&demo.join(file), &t, want);
    }
    let cli = [
        ("one", "11111111-1111-1111-1111-111111111111"),
        ("hq", "33333333-3333-3333-3333-333333333333"),
        ("co", "44444444-4444-4444-4444-444444444444"),
        ("eur", "55555555-5555-5555-5555-555555555555"),
    ];
    check(&fixtures().join("cli/record.sqlite"), &cli, &golden["cli"]);
    // never synced → None (keeps "calls: unknown")
    assert!(golden["variants"]["never"]["lab"]["estimate"].is_null());
}
