//! `ownpurse viz`: the explainer models on the demo world, text and JSON, zero calls.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(args)
        .env("OWNPURSE_CONFIG", demo().join("ownpurse.json"))
        .env_remove("OWNPURSE_PROFILE")
        .env_remove("OWNPURSE_STORE")
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_DATA_HOME")
        .env("HOME", "/home/demo")
        .output()
        .expect("run ownpurse");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).unwrap(),
    )
}

#[test]
fn bridge_text_and_json() {
    let (code, out) = run(&["viz", "bridge", "hold-design", "--as-of", "2025-12-31"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Why hold and design disagree by 418.27, item by item"));
    assert!(out.contains("Annual return filing for the parent"));
    let (code, out) = run(&[
        "--json",
        "viz",
        "bridge",
        "design-roast",
        "--as-of",
        "2025-12-31",
    ]);
    assert_eq!(code, 0);
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["model"]["residual"], "263.94");
    assert_eq!(v["spec"]["pair"], "design-roast");
    assert_eq!(v["parts"][0], "start");
}

#[test]
fn t_account_flow_and_the_spec_flag() {
    let (code, out) = run(&[
        "--json",
        "viz",
        "t_account",
        "hold",
        "1470",
        "--as-of",
        "2025-12-31",
    ]);
    assert_eq!(code, 0, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["model"]["closing"], "4670.51");
    let (code, out) = run(&[
        "--json",
        "viz",
        "flow",
        "--spec",
        r#"{"kind":"flow","as_of":"2025-12-31","orgs":["hold","roast"]}"#,
    ]);
    assert_eq!(code, 0, "{out}");
    let v: Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["model"]["edges"][0]["words"], "roast owes hold 15,568.80");
}

#[test]
fn errors_are_usage() {
    let (code, _) = run(&["viz", "bridge", "nope", "--as-of", "2025-12-31"]);
    assert_eq!(code, 2);
    let (code, _) = run(&["viz", "t_account", "hold"]);
    assert_eq!(code, 2);
    let (code, _) = run(&["viz", "flow", "--spec", "{"]);
    assert_eq!(code, 2);
}
