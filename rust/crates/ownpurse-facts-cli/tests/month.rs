//! The whole sample month, through the binary: every query in `samples/queries/` answers, the checks pass, replay is
//! deterministic, and the demo script still names what exists. The ignored tests are the open gaps from
//! docs/plans/FACTS-EDGES.md, written down as the behaviour phase 2 has to reach.

mod common;

use common::*;
use std::collections::BTreeMap;

/// Zero however it is written: `0`, `0.0`, `-0.00`.
fn is_zero(v: &Json) -> bool {
    let s = v.as_str().map_or_else(|| v.to_string(), str::to_string);
    s.trim_start_matches('-')
        .chars()
        .all(|c| c == '0' || c == '.')
}

fn text(v: &Json) -> String {
    v.as_str().map_or_else(|| v.to_string(), str::to_string)
}

#[test]
fn every_sample_query_answers() {
    let log = Log::month();
    let mut files: Vec<String> = std::fs::read_dir(samples().join("queries"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".sql"))
        .collect();
    files.sort();
    assert!(files.len() >= 12, "{files:?}");
    for f in &files {
        let rows = log.query(f);
        // 12 is a check: it lists entries that do not balance, so a pass is empty.
        if !f.starts_with("12-") {
            assert!(
                !rows.is_empty(),
                "{f} returned nothing: the query has rotted"
            );
        }
    }
}

#[test]
fn the_statement_reconciles_in_every_stream() {
    let log = Log::month();
    let rows = log.query("11-the-statement-reconciles.sql");
    let streams: Vec<String> = rows.iter().map(|r| text(&r["projector"])).collect();
    assert_eq!(streams, ["bank.csv@1", "bank.ofx@1", "extraction@1"]);
    for r in &rows {
        assert_eq!(r["opening"], "24310.55", "{r}");
        assert_eq!(r["closing"], "13413.22", "{r}");
        assert_eq!(r["moved"], "-10897.33", "{r}");
        assert_eq!(
            r["lines"], 13,
            "the lapsed pending hold is not money that moved: {r}"
        );
        assert!(is_zero(&r["difference"]), "{r}");
    }
}

#[test]
fn every_entry_balances_and_each_book_sums_to_zero() {
    let log = Log::month();
    assert!(log.query("12-every-entry-balances.sql").is_empty());
    let books = log.json(&[
        "sql",
        "SELECT book, dsum(quantity) AS total FROM postings WHERE status = 'entry.status/posted' GROUP BY book",
    ]);
    let rows = books["rows"].as_array().unwrap();
    assert_eq!(
        rows.len(),
        2,
        "the roastery's Xero and Quillfeather's QuickBooks: {books}"
    );
    for r in rows {
        assert!(is_zero(&r["total"]), "{r}");
    }
}

/// The check is only worth having if it fails: one extra posting on one entry, and 12 names that entry.
#[test]
fn the_balance_check_catches_an_entry_that_does_not_balance() {
    let log = Log::month();
    let ids = log.json(&[
        "sql",
        "SELECT en.e AS entry, a.value AS account, c.value AS commodity FROM facts en \
         JOIN facts p ON p.e = en.e AND p.attr = 'entry/posting' \
         JOIN facts a ON a.e = p.value AND a.attr = 'posting/account' \
         JOIN facts c ON c.e = p.value AND c.attr = 'posting/commodity' \
         WHERE en.attr = 'entry/id' AND en.value LIKE 'xero:%' LIMIT 1",
    ]);
    let r = &ids["rows"][0];
    let tx = serde_json::json!({"ops": [
        {"db/id": "extra", "posting/account": r["account"], "posting/quantity": "0.01",
         "posting/commodity": r["commodity"]},
        ["add", r["entry"], "entry/posting", "extra"]
    ]});
    let file = log.path.with_extension("tx.json");
    std::fs::write(&file, tx.to_string()).unwrap();
    log.ok(&[
        "transact",
        file.to_str().unwrap(),
        "--actor",
        "person:tester",
    ]);
    let off = log.query("12-every-entry-balances.sql");
    assert_eq!(off.len(), 1, "{off:?}");
    assert_eq!(off[0]["off_by"], "0.01");
}

#[test]
fn a_trial_balance_is_per_book() {
    let log = Log::month();
    let rows = log.query("04-trial-balance.sql");
    assert!(rows.iter().all(|r| r["book"].is_string()), "{rows:?}");
    let meals = rows
        .iter()
        .find(|r| r["book"] == "xero:tallowmere" && r["account"] == "Meals and entertainment")
        .unwrap();
    assert_eq!(meals["balance"], "195.90");
    let theirs = rows
        .iter()
        .filter(|r| r["book"] == "qbo:quillfeather")
        .count();
    assert!(theirs > 0, "the supplier's own book is there, kept apart");
}

#[test]
fn the_log_remembers_the_invoice_before_it_was_edited() {
    let log = Log::month();
    let rows = log.query("05-as-of.sql");
    let amount = rows.iter().find(|r| r["attr"] == "obs/amount").unwrap();
    assert_eq!(amount["value"], "929.63");
    let t = text(&amount["tx"]);
    let inv = "doc/id=xero:tallowmere/Invoice/c0ffee03-0000-4000-8000-000000000002";
    let then = log.json(&["--as-of", &t, "pull", inv, r#"["obs/amount"]"#]);
    assert_eq!(then["obs/amount"], "929.63");
    let now = log.json(&["pull", inv, r#"["obs/amount"]"#]);
    assert_eq!(now["obs/amount"], "1084.57");
}

#[test]
fn the_rules_find_every_kind_of_link() {
    let log = Log::month();
    let kinds = log.json(&["sql", "SELECT kind, count(*) AS n FROM links GROUP BY kind"]);
    let got: BTreeMap<String, u64> = kinds["rows"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| (text(&r["kind"]), r["n"].as_u64().unwrap()))
        .collect();
    let want: BTreeMap<String, u64> = [
        ("link.kind/counterpart", 2),
        ("link.kind/evidences", 8),
        ("link.kind/mirrors", 13),
        ("link.kind/same-event", 83),
        ("link.kind/settles", 6),
    ]
    .into_iter()
    .map(|(k, n)| (k.to_string(), n))
    .collect();
    assert_eq!(got, want);
}

/// The same files at the same clock give the same chain, byte for byte: the head hash is a fingerprint of the month.
#[test]
fn replay_is_deterministic() {
    let a = Log::month();
    let b = Log::month();
    let ha = a.json(&["verify"]);
    let hb = b.json(&["verify"]);
    assert_eq!(ha["head"], hb["head"]);
    assert_eq!(ha["transactions"], hb["transactions"]);
}

#[test]
fn load_shows_progress_file_by_file() {
    let log = Log::new();
    log.ok(&["init"]);
    let manifest = sample("ingest.json");
    let (ok, _, err) = log.run(&["load", "--progress", &manifest]);
    assert!(ok, "{err}");
    let lines: Vec<&str> = err.lines().collect();
    assert_eq!(lines.len(), 50, "{err}");
    assert!(lines[0].starts_with("[ 1/50] ") && lines[49].starts_with("[50/50] "));
    assert!(lines.iter().all(|l| l.contains(" tx ")), "{err}");
    let (_, _, again) = log.run(&["load", "--progress", &manifest]);
    assert!(again.lines().all(|l| l.ends_with("no change")), "{again}");
}

#[test]
fn the_demo_script_parses_and_names_what_exists() {
    let path = sample("demo.sh");
    let ok = std::process::Command::new("bash")
        .args(["-n", &path])
        .status()
        .unwrap()
        .success();
    assert!(ok, "bash -n {path}");
    let script = std::fs::read_to_string(&path).unwrap();
    for word in script.split_whitespace() {
        let word = word.trim_matches(|c: char| c == '"' || c == '\'');
        if word.ends_with(".sql") {
            let rel = word.rsplit('/').next().unwrap();
            assert!(
                samples().join("queries").join(rel).exists(),
                "demo.sh names {rel}"
            );
        }
    }
}

/// Open gap (FACTS-EDGES, change 1): a re-read of a snapshot source retracts what a person corrected, because the
/// diff cannot tell its own past assertions from anyone else's.
#[test]
#[ignore = "phase 2: snapshot ownership by projector"]
fn a_reread_keeps_a_persons_correction() {
    let log = Log::month();
    let receipt = log.json(&[
        "sql",
        "SELECT e FROM observations WHERE kind = 'obs.kind/receipt'",
    ]);
    let e = receipt["rows"][0]["e"].clone();
    let tx = serde_json::json!({"ops": [["add", e, "obs/counterparty", "The Copper Pot"]]});
    let file = log.path.with_extension("tx.json");
    std::fs::write(&file, tx.to_string()).unwrap();
    log.ok(&[
        "transact",
        file.to_str().unwrap(),
        "--actor",
        "person:owner",
    ]);
    // The extractor reads the photo again, a day later: the same extraction, read anew.
    let ext = samples().join("receipts/copper-pot-2026-03-12.extraction.json");
    let jpg = samples().join("receipts/copper-pot-2026-03-12.jpg");
    log.ok(&[
        "ingest",
        ext.to_str().unwrap(),
        "--projector",
        "extraction@1",
        "--source",
        "receipts:tallowmere",
        "--with",
        &format!("reads={}", jpg.display()),
        "--observed-at",
        "2026-04-03T09:00:00Z",
    ]);
    let now = log.json(&[
        "sql",
        &format!("SELECT value FROM facts WHERE e = {e} AND attr = 'obs/counterparty'"),
    ]);
    assert_eq!(now["rows"][0]["value"], "The Copper Pot", "{now}");
}

/// Open gap (FACTS-EDGES, change 8): lookups across files happen at ingest, so a file read before the one it
/// refers to resolves less. The same files in any order should give the same facts.
#[test]
#[ignore = "phase 2: cross-document lookups become rules"]
fn load_order_does_not_change_the_facts() {
    let forward = Log::month();
    let full: Json =
        serde_json::from_str(&std::fs::read_to_string(samples().join("ingest.json")).unwrap())
            .unwrap();
    let mut files: Vec<Json> = full["files"].as_array().unwrap().clone();
    files.reverse();
    for f in &mut files {
        let p = samples().join(f["file"].as_str().unwrap());
        f["file"] = Json::String(p.display().to_string());
        if let Some(with) = f["with"].as_object_mut() {
            for v in with.values_mut() {
                let p = samples().join(v.as_str().unwrap());
                *v = Json::String(p.display().to_string());
            }
        }
    }
    let backward = Log::new();
    backward.ok(&["init"]);
    let m = backward.path.with_extension("manifest.json");
    std::fs::write(
        &m,
        serde_json::json!({"sources": full["sources"], "files": files}).to_string(),
    )
    .unwrap();
    backward.ok(&["load", m.to_str().unwrap()]);
    backward.ok(&["link"]);
    let shape = "SELECT attr, count(*) AS n FROM facts WHERE attr LIKE 'obs/%' OR attr LIKE 'link/%' \
                 GROUP BY attr ORDER BY attr";
    assert_eq!(
        forward.json(&["sql", shape]),
        backward.json(&["sql", shape])
    );
}
