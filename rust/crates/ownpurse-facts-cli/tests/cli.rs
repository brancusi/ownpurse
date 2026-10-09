//! The binary end to end: create a log, load Xero's two pulls, correct something as a person, read it back.

use serde_json::Value as Json;
use std::path::{Path, PathBuf};
use std::process::Command;

fn samples() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples")
}

struct Log {
    _dir: tempfile::TempDir,
    path: PathBuf,
}

impl Log {
    fn new() -> Log {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("books.sqlite");
        Log { _dir: dir, path }
    }

    fn run(&self, args: &[&str]) -> (bool, String, String) {
        let out = Command::new(env!("CARGO_BIN_EXE_ownpurse-facts"))
            .arg("--log")
            .arg(&self.path)
            .args(["--at", "2026-04-02T10:00:00Z"])
            .args(args)
            .output()
            .unwrap();
        (
            out.status.success(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            String::from_utf8_lossy(&out.stderr).into_owned(),
        )
    }

    fn ok(&self, args: &[&str]) -> String {
        let (ok, out, err) = self.run(args);
        assert!(ok, "{args:?} failed: {err}");
        out
    }

    fn json(&self, args: &[&str]) -> Json {
        let mut a = vec!["--json"];
        a.extend_from_slice(args);
        serde_json::from_str(&self.ok(&a)).unwrap()
    }

    /// A manifest with just the Xero files, written next to the samples so its paths resolve.
    fn load_xero(&self) {
        let full: Json =
            serde_json::from_str(&std::fs::read_to_string(samples().join("ingest.json")).unwrap())
                .unwrap();
        let files: Vec<Json> = full["files"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["source"] == "xero:tallowmere")
            .map(|f| {
                let mut f = f.clone();
                let p = samples().join(f["file"].as_str().unwrap());
                f["file"] = Json::String(p.display().to_string());
                f
            })
            .collect();
        let m = serde_json::json!({"sources": full["sources"], "files": files});
        let path = self.path.with_extension("manifest.json");
        std::fs::write(&path, m.to_string()).unwrap();
        self.ok(&["load", path.to_str().unwrap()]);
    }
}

#[test]
fn init_load_verify() {
    let log = Log::new();
    log.ok(&["init"]);
    log.load_xero();
    let v = log.json(&["verify"]);
    assert_eq!(v["files"], 22, "{v}");
    let again = log.json(&[
        "load",
        log.path.with_extension("manifest.json").to_str().unwrap(),
    ]);
    for row in again["rows"].as_array().unwrap() {
        assert_eq!(row["tx"], "no change", "{row}");
    }
}

#[test]
fn a_person_corrects_a_fact_and_the_history_keeps_both() {
    let log = Log::new();
    log.ok(&["init"]);
    log.load_xero();
    let bt = "doc/id=xero:tallowmere/BankTransaction/c0ffee05-0000-4000-8000-000000000003";
    let e = log.json(&["entity", bt]);
    let id = e["db/id"].as_u64().unwrap();
    let tx = serde_json::json!({"ops": [
        {"note/target": id, "note/text": "Client dinner: three from Ostley Joinery, two of us."},
        ["add", id, "obs/description", "Client dinner, Ostley Joinery buyers"]
    ]});
    let file = log.path.with_extension("tx.json");
    std::fs::write(&file, tx.to_string()).unwrap();
    let dry = log.ok(&[
        "transact",
        file.to_str().unwrap(),
        "--actor",
        "person:owner",
        "--dry-run",
    ]);
    assert!(dry.contains("dry run"), "{dry}");
    log.ok(&[
        "transact",
        file.to_str().unwrap(),
        "--actor",
        "person:owner",
        "--reason",
        "what the dinner was",
    ]);
    let hist = log.json(&["history", bt]);
    let by_owner: Vec<&Json> = hist["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| r["by"] == "person:owner")
        .collect();
    assert_eq!(by_owner.len(), 1, "{hist}");
    assert_eq!(by_owner[0]["attr"], "obs/description");
    let notes = log.json(&[
        "sql",
        "SELECT value FROM facts_shown WHERE attr = 'note/text'",
    ]);
    assert_eq!(notes["count"], 1);
}

#[test]
fn sql_reads_and_never_writes() {
    let log = Log::new();
    log.ok(&["init"]);
    log.load_xero();
    let tb = log.json(&[
        "sql",
        "-f",
        samples()
            .join("queries/04-trial-balance.sql")
            .to_str()
            .unwrap(),
    ]);
    let meals = tb["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["account"] == "Meals and entertainment")
        .unwrap();
    assert_eq!(meals["balance"], "195.90");
    let (ok, _, err) = log.run(&["sql", "DELETE FROM datom"]);
    assert!(!ok && err.contains("readonly"), "{err}");
}

#[test]
fn as_of_reads_an_earlier_database() {
    let log = Log::new();
    log.ok(&["init"]);
    log.load_xero();
    let inv = "doc/id=xero:tallowmere/Invoice/c0ffee03-0000-4000-8000-000000000002";
    let now = log.json(&["pull", inv, r#"["obs/amount"]"#]);
    assert_eq!(now["obs/amount"], "1084.57");
    let first_pull_end = log.json(&[
        "sql",
        "SELECT max(tx) AS t FROM transactions WHERE tx < 455",
    ]);
    let t = first_pull_end["rows"][0]["t"].as_u64().unwrap().to_string();
    let then = log.json(&["--as-of", &t, "pull", inv, r#"["obs/amount"]"#]);
    assert_eq!(then["obs/amount"], "929.63");
}

#[test]
fn errors_say_what_to_do() {
    let log = Log::new();
    let (ok, _, err) = log.run(&["verify"]);
    assert!(!ok && err.contains("create one with `init`"), "{err}");
    log.ok(&["init"]);
    let (ok, _, err) = log.run(&[
        "ingest",
        "nope.json",
        "--projector",
        "nope",
        "--source",
        "x:y",
    ]);
    assert!(!ok && err.contains("projectors"), "{err}");
}
