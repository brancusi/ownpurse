//! The binary end to end: create a log, load Xero's two pulls, correct something as a person, read it back.

mod common;

use common::*;

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

/// The whole month from every source: it loads, verifies, reloads as no change, and the link rules join the
/// witnesses of the dinner and both sides of the two supplier invoices.
#[test]
fn the_whole_month_from_every_source() {
    let log = Log::new();
    log.ok(&["init"]);
    let manifest = samples().join("ingest.json");
    let m = manifest.to_str().unwrap();
    log.ok(&["load", m]);
    let again = log.json(&["load", m]);
    for row in again["rows"].as_array().unwrap() {
        assert_eq!(row["tx"], "no change", "{row}");
    }
    log.ok(&["link"]);
    let relink = log.json(&["link"]);
    assert_eq!(relink["links"], 0, "rules are idempotent");
    let dinner = log.json(&[
        "sql",
        "-f",
        samples()
            .join("queries/09-what-backs-the-dinner.sql")
            .to_str()
            .unwrap(),
    ]);
    let sources: Vec<&str> = dinner["rows"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| r["source"].as_str())
        .collect();
    assert_eq!(
        dinner["count"], 5,
        "Xero, CSV, OFX, statement, receipt: {dinner}"
    );
    assert!(sources.contains(&"receipts:tallowmere") && sources.contains(&"xero:tallowmere"));
    let both = log.json(&[
        "sql",
        "-f",
        samples()
            .join("queries/10-both-sides-of-an-invoice.sql")
            .to_str()
            .unwrap(),
    ]);
    assert_eq!(both["count"], 2);
    let v = log.json(&["verify"]);
    assert_eq!(v["files"], 60);
}
