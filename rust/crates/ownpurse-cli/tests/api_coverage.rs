//! API coverage for the 2025 cleanup (reads and writes beyond the first four operations), through the real
//! CLI binary against the loopback fake Xero only (`OWNPURSE_WRITE_LOOPBACK_ONLY=1`; never the network).

use ownpurse_xero::write::fake::{FakeXero, State};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::process::Command;

const T: &str = "c0ffee00-0000-4000-8000-0000000000a1";
const DEMO: &str = "Demo Company (US)";

struct W {
    dir: PathBuf,
    fake: FakeXero,
}

fn world(name: &str) -> W {
    let dir = std::env::temp_dir().join(format!("xa-api-cov-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = State::default();
    s.add_tenant(
        T,
        DEMO,
        &[
            ("1010", "Operating", "BANK"),
            ("1020", "Petty cash", "BANK"),
            ("6310", "Cupping supplies", "EXPENSE"),
            ("6520", "Van fuel", "EXPENSE"),
            ("2700", "Owner loan", "LIABILITY"),
        ],
    );
    s.add_bank_spend(T, "bt-104", "1010", "6520", "238.61", false);
    s.add_bank_spend(T, "bt-105", "1010", "6310", "61.09", true);
    s.add_contact(T, "ct-201", "Hollowbrook Fuel");
    s.add_contact(T, "ct-202", "Owner");
    let fake = FakeXero::start(s).unwrap();
    let profile = json!({"client_id": "DEMO0CLIENT0ID0000000000000000000", "store": "record.sqlite",
        "token_file": "tokens.json", "orgs": {"demo": T}, "allow_writes": true,
        "require_tenant_name": DEMO});
    std::fs::write(
        dir.join("ownpurse.json"),
        json!({"default_profile": "demo", "profiles": {"demo": profile},
               "xero": {"api_base": fake.api_base(), "connections_url": fake.connections_url(),
                        "token_url": fake.token_url()}})
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        dir.join("tokens.json"),
        json!({"access_token": "AT", "refresh_token": "RT", "obtained_at": 4.0e9, "expires_in": 1800,
               "scope": "accounting.transactions.read accounting.reports.read accounting.settings.read accounting.contacts.read accounting.attachments.read accounting.manualjournals accounting.banktransactions accounting.attachments accounting.contacts"})
        .to_string(),
    )
    .unwrap();
    let w = W { dir, fake };
    let (code, _, text) = cmd(&w.dir, &["sync", "demo"]);
    assert_eq!(code, 0, "sync: {text}");
    w
}

fn cmd(dir: &Path, args: &[&str]) -> (i32, Value, String) {
    let mut a = args.to_vec();
    a.push("--json");
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(&a)
        .current_dir(dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", dir)
        .env("OWNPURSE_CONFIG", dir.join("ownpurse.json"))
        .env("OWNPURSE_WRITE_LOOPBACK_ONLY", "1")
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    (out.status.code().unwrap_or(-1), v, text)
}

fn gets(w: &W, path: &str) -> usize {
    w.fake.with(|s| {
        s.log
            .iter()
            .filter(|l| l.0 == "GET" && l.1.contains(path))
            .count()
    })
}

// ── 1. report fetch ──────────────────────────────────────────────────────────────────────────────

#[test]
fn report_fetch_appends_xeros_report_and_parity_reads_it() {
    let w = world("report");
    let (code, v, text) = cmd(
        &w.dir,
        &[
            "report",
            "fetch",
            "demo",
            "trialbalance",
            "--date",
            "2026-10-31",
        ],
    );
    assert_eq!(code, 0, "{text}");
    let r = &v["reports"][0];
    assert_eq!(r["report"], "TrialBalance");
    assert_eq!(
        r["params"],
        r#"{"date":"2026-10-31","paymentsOnly":"false"}"#
    );
    assert_eq!(r["change"], "first");
    assert!(r["rows"].as_i64().unwrap() > 0, "{v}");
    assert_eq!(gets(&w, "Reports/TrialBalance"), 1);
    // the same numbers again: a new observation, marked unchanged
    let (_, v, _) = cmd(
        &w.dir,
        &["report", "fetch", "demo", "tb", "--date", "2026-10-31"],
    );
    assert_eq!(v["reports"][0]["change"], "unchanged", "{v}");
    // parity reads exactly that row (the params `report fetch` stores)
    let rec = ownpurse_core::Record::open(w.dir.join("record.sqlite")).unwrap();
    assert!(rec.tb_snapshot(T, "2026-10-31").unwrap().is_some());
    let (code, _, text) = cmd(&w.dir, &["parity", "demo", "--date", "2026-10-31"]);
    assert_eq!(code, 0, "{text}");
    // period reports, cash basis, the list
    let (code, v, text) = cmd(
        &w.dir,
        &[
            "report",
            "fetch",
            "demo",
            "pl",
            "--from",
            "2026-01-01",
            "--to",
            "2026-10-31",
            "--cash",
        ],
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(
        v["reports"][0]["params"],
        r#"{"fromDate":"2026-01-01","paymentsOnly":"true","toDate":"2026-10-31"}"#
    );
    let (code, _, text) = cmd(
        &w.dir,
        &[
            "report",
            "fetch",
            "demo",
            "balancesheet",
            "--date",
            "2026-10-31",
        ],
    );
    assert_eq!(code, 0, "{text}");
    let (code, _, text) = cmd(
        &w.dir,
        &[
            "report",
            "fetch",
            "demo",
            "banksummary",
            "--from",
            "2026-01-01",
            "--to",
            "2026-10-31",
        ],
    );
    assert_eq!(code, 0, "{text}");
    let (_, v, _) = cmd(&w.dir, &["report", "list", "demo"]);
    let list = v["reports"].as_array().unwrap();
    assert_eq!(list.len(), 5, "{v}");
    assert!(list.iter().all(|r| r["source"] == "fetch"));
    let (_, v, _) = cmd(&w.dir, &["report", "list", "demo", "pl"]);
    assert_eq!(v["reports"].as_array().unwrap().len(), 1);
}

#[test]
fn report_fetch_refuses_bad_input_before_any_call() {
    let w = world("report-bad");
    let before = w.fake.with(|s| s.log.len());
    for args in [
        vec!["report", "fetch", "demo", "cashflow"],
        vec!["report", "fetch", "demo", "tb", "--date", "2026-13-40"],
        vec!["report", "fetch", "demo", "banksummary", "--cash"],
    ] {
        let (code, v, text) = cmd(&w.dir, &args);
        assert_eq!(code, 2, "{args:?}: {text}");
        assert!(v["error"].is_string(), "{text}");
    }
    let (_, v, _) = cmd(&w.dir, &["--offline", "report", "fetch", "demo", "tb"]);
    assert_eq!(v["error"], "offline");
    assert_eq!(w.fake.with(|s| s.log.len()), before, "no call");
}

// ── 2. history: read, notes (op E) and the audit note of every landed write ──────────────────────

fn approve(dir: &Path, hash: &str) {
    use ownpurse_core::write::{AllOps, ApprovalRecord, ApprovalScope, ApprovalStore, Via};
    use ownpurse_xero::store::Writer;
    use ownpurse_xero::write::RecordApprovals;
    let store = Writer::open(dir.join("record.sqlite")).unwrap();
    RecordApprovals { store: &store }
        .record(&ApprovalRecord {
            hash: hash.into(),
            verbatim: format!("Approved, {}", &hash[..12]),
            via: Via::Tty,
            where_: "tty".into(),
            at: "2026-10-08T07:30:00Z".into(),
            scope: ApprovalScope::All(AllOps::All),
            recorded_at: "2026-10-08T07:30:01Z".into(),
        })
        .unwrap();
}

/// plan → seal → approve → run → verify; the package path and hash.
fn landed(w: &W, name: &str, input: Value) -> (String, String) {
    let f = w.dir.join(format!("{name}.ops.json"));
    std::fs::write(&f, input.to_string()).unwrap();
    let (code, v, text) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
    assert_eq!(code, 0, "plan: {text}");
    let pkg = v["package"].as_str().unwrap().to_string();
    let (code, v, text) = cmd(&w.dir, &["write", "seal", &pkg]);
    assert_eq!(code, 0, "seal: {text}");
    let hash = v["hash"].as_str().unwrap().to_string();
    approve(&w.dir, &hash);
    let (code, _, text) = cmd_env(
        &w.dir,
        &["write", "run", &pkg],
        &[("OWNPURSE_WRITE_RUN", &hash)],
    );
    assert_eq!(code, 0, "run: {text}");
    let (code, v, text) = cmd(&w.dir, &["write", "verify", &pkg]);
    assert_eq!(code, 0, "verify: {text}");
    assert_eq!(v["outcome"], "verified", "{v}");
    (pkg, hash)
}

fn cmd_env(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, Value, String) {
    let mut a = args.to_vec();
    a.push("--json");
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(&a)
        .current_dir(dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", dir)
        .env("OWNPURSE_CONFIG", dir.join("ownpurse.json"))
        .env("OWNPURSE_WRITE_LOOPBACK_ONLY", "1")
        .envs(env.iter().copied())
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    let v = serde_json::from_slice(&out.stdout).unwrap_or(Value::Null);
    (out.status.code().unwrap_or(-1), v, text)
}

#[test]
fn history_reads_into_the_record_and_every_landed_write_leaves_its_audit_note() {
    let w = world("history");
    // read: one call, into the record; --cached reads it back with zero calls
    let (code, v, text) = cmd(&w.dir, &["history", "demo", "BankTransactions", "bt-104"]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["history"].as_array().map(Vec::len), Some(0), "{v}");
    let calls = w.fake.with(|s| s.log.len());
    let (code, _, text) = cmd(
        &w.dir,
        &["history", "demo", "banktransactions", "bt-104", "--cached"],
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(w.fake.with(|s| s.log.len()), calls, "--cached: no call");
    let (code, v, _) = cmd(&w.dir, &["history", "demo", "Accounts", "x"]);
    assert_eq!((code, v["error"].as_str()), (2, Some("usage")), "{v}");
    // a journal (its audit note: the package turns notes on), an explicit note on it, and one on bt-104
    let (pkg, hash) = landed(
        &w,
        "notes",
        json!({"id": "notes-01", "title": "Owner-paid supplies, March", "history_notes": true, "ops": [
            {"kind": "create_journal", "id": "op-1", "org": "demo", "date": "2026-10-02",
             "narration": "Owner-paid supplies", "cash_basis": true,
             "lines": [{"account": "6310", "amount": "47.13"}, {"account": "2700", "amount": "-47.13"}]},
            {"kind": "add_note", "id": "op-2", "org": "demo", "entity": "ManualJournals",
             "document": "op:op-1", "details": "receipts in the March folder"},
            {"kind": "add_note", "id": "op-3", "org": "demo", "entity": "BankTransactions",
             "document": "bt-104", "details": "fuel for the market run"}]}),
    );
    let (_, st, _) = cmd(&w.dir, &["write", "status", &pkg]);
    let mj = st["ops"][0]["target_id"].as_str().unwrap().to_string();
    let notes = |k: &str| {
        w.fake.with(|s| {
            s.history
                .get(&(T.to_string(), k.to_string()))
                .cloned()
                .unwrap_or_default()
                .iter()
                .map(|h| h["Details"].as_str().unwrap_or("").to_string())
                .collect::<Vec<_>>()
        })
    };
    let on_mj = notes(&format!("ManualJournals/{mj}"));
    assert_eq!(on_mj.len(), 2, "{on_mj:?}");
    assert!(
        on_mj[0].starts_with(&format!(
            "ownpurse package {} op op-1: Owner-paid supplies, March - create a journal",
            &hash[..12]
        )),
        "the audit note: {on_mj:?}"
    );
    assert_eq!(on_mj[1], "receipts in the March folder");
    assert!(
        on_mj.iter().all(|n| n.is_ascii()),
        "notes are ASCII: {on_mj:?}"
    );
    // a note is itself the note: no audit note on top
    assert_eq!(
        notes("BankTransactions/bt-104"),
        ["fuel for the market run"]
    );
    // the record follows: the history read back by the run, zero calls
    let calls = w.fake.with(|s| s.log.len());
    let (_, v, _) = cmd(
        &w.dir,
        &["history", "demo", "ManualJournals", &mj, "--cached"],
    );
    assert_eq!(v["history"].as_array().map(Vec::len), Some(2), "{v}");
    assert_eq!(w.fake.with(|s| s.log.len()), calls);
    // the review says the note comes, and the notes are permanent
    let p: Value =
        serde_json::from_str(&std::fs::read_to_string(w.dir.join(&pkg)).unwrap()).unwrap();
    assert!(p["ops"][0]["history_note"].is_string());
    assert!(p["ops"][1]["history_note"].is_null());
    assert_eq!(p["ops"][1]["flags"], json!(["permanent history"]));
}

#[test]
fn audit_notes_are_off_for_a_demo_profile_unless_asked_and_a_long_note_is_refused() {
    let w = world("notes-off");
    let (pkg, _) = landed(
        &w,
        "plain",
        json!({"id": "plain-01", "title": "No notes", "ops": [
            {"kind": "create_journal", "id": "op-1", "org": "demo", "date": "2026-10-02",
             "narration": "Owner-paid supplies", "cash_basis": true,
             "lines": [{"account": "6310", "amount": "12.07"}, {"account": "2700", "amount": "-12.07"}]}]}),
    );
    let p: Value =
        serde_json::from_str(&std::fs::read_to_string(w.dir.join(&pkg)).unwrap()).unwrap();
    assert!(
        p["ops"][0]["history_note"].is_null(),
        "demo profile: off by default"
    );
    assert!(w.fake.with(|s| s.history.is_empty()));
    let f = w.dir.join("long.ops.json");
    std::fs::write(
        &f,
        json!({"id": "long-01", "ops": [{"kind": "add_note", "id": "op-1", "org": "demo",
            "entity": "BankTransactions", "document": "bt-104", "details": "x".repeat(251)}]})
        .to_string(),
    )
    .unwrap();
    let (code, v, _) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
    assert_eq!((code, v["error"].as_str()), (2, Some("usage")), "{v}");
}

#[test]
fn a_manifest_note_compiles_to_an_add_note_operation() {
    let w = world("mf-note");
    for args in [
        // the effects are dated by the latest cached Xero TB (item 1 fetches one)
        vec!["report", "fetch", "demo", "tb", "--date", "2026-10-31"],
        vec!["manifest", "new", "notes", "Notes for the accountant"],
        vec![
            "manifest",
            "note",
            "demo:bt:bt-104",
            "fuel for the market run",
            "--why",
            "context",
        ],
    ] {
        let (code, _, text) = cmd(&w.dir, &args);
        assert_eq!(code, 0, "{args:?}: {text}");
    }
    let (_, v, text) = cmd(&w.dir, &["manifest", "check"]);
    let issues = v.to_string();
    assert!(
        issues.contains("permanent: Xero keeps history notes"),
        "{text}"
    );
    assert_eq!(v["errors"], 0, "a note is never gated: {text}");
    let entry = v["issues"][0]["entry"].to_string();
    let (code, _, text) = cmd(
        &w.dir,
        &["manifest", "ack", &entry, "note", "--path", "/History"],
    );
    assert_eq!(code, 0, "{text}");
    let (code, v, text) = cmd(&w.dir, &["manifest", "plan", "--date", "2026-10-31"]);
    assert_eq!(code, 0, "{text}");
    let pkg = v["package"].as_str().unwrap();
    let p: Value = serde_json::from_str(
        &std::fs::read_to_string(w.dir.join(pkg))
            .unwrap_or_else(|_| std::fs::read_to_string(pkg).unwrap()),
    )
    .unwrap();
    let op = &p["ops"][0];
    assert_eq!(op["op"]["kind"], "add_note", "{p}");
    assert_eq!(op["op"]["details"], "fuel for the market run");
    assert_eq!(op["request"]["path"], "BankTransactions/bt-104/History");
    // too long: refused before it is recorded
    let (code, _, _) = cmd(
        &w.dir,
        &["manifest", "note", "demo:bt:bt-104", &"y".repeat(251)],
    );
    assert_eq!(code, 2);
}

// ── 3. attachments: list and download; attach to a bank transaction ──────────────────────────────

#[test]
fn a_receipt_attaches_to_a_bank_transaction_and_reads_back_byte_for_byte() {
    let w = world("attach-bt");
    let receipt = "date,vendor,amount\n2025-03-04,Hollowbrook Fuel,238.61\n";
    std::fs::write(w.dir.join("receipt-0304.csv"), receipt).unwrap();
    let (pkg, _) = landed(
        &w,
        "receipt",
        json!({"id": "receipt-01", "title": "Fuel receipt", "ops": [
            {"kind": "attach", "id": "op-1", "org": "demo", "entity": "BankTransactions",
             "journal": "bt-104", "file": "receipt-0304.csv"}]}),
    );
    let p: Value =
        serde_json::from_str(&std::fs::read_to_string(w.dir.join(&pkg)).unwrap()).unwrap();
    assert_eq!(
        p["ops"][0]["request"]["path"],
        "BankTransactions/bt-104/Attachments/receipt-0304.csv"
    );
    assert_eq!(p["ops"][0]["op"]["entity"], "BankTransactions");
    // list: one call, into the record; --cached: zero calls
    let (code, v, text) = cmd(
        &w.dir,
        &["attachments", "demo", "BankTransactions", "bt-104"],
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["attachments"][0]["file"], "receipt-0304.csv", "{v}");
    assert_eq!(v["attachments"][0]["bytes"], receipt.len());
    let calls = w.fake.with(|s| s.log.len());
    let (_, v, _) = cmd(
        &w.dir,
        &[
            "attachments",
            "demo",
            "banktransactions",
            "bt-104",
            "--cached",
        ],
    );
    assert_eq!(v["attachments"].as_array().map(Vec::len), Some(1), "{v}");
    assert_eq!(w.fake.with(|s| s.log.len()), calls);
    // download: the same bytes; never over an existing file
    let (code, v, text) = cmd(
        &w.dir,
        &[
            "attachments",
            "demo",
            "BankTransactions",
            "bt-104",
            "--get",
            "receipt-0304.csv",
            "--out",
            "copy.csv",
        ],
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["bytes"], receipt.len());
    assert_eq!(
        std::fs::read_to_string(w.dir.join("copy.csv")).unwrap(),
        receipt
    );
    let get = w.fake.with(|s| {
        s.log
            .iter()
            .find(|l| l.1.ends_with("/Attachments/receipt-0304.csv") && l.0 == "GET")
            .is_some()
    });
    assert!(get, "the file came from Xero");
    let (code, _, _) = cmd(
        &w.dir,
        &[
            "attachments",
            "demo",
            "BankTransactions",
            "bt-104",
            "--get",
            "receipt-0304.csv",
            "--out",
            "copy.csv",
        ],
    );
    assert_eq!(code, 2, "never overwrites");
    let (code, _, _) = cmd(
        &w.dir,
        &[
            "attachments",
            "demo",
            "BankTransactions",
            "bt-104",
            "--get",
            "nope.csv",
        ],
    );
    assert_eq!(code, 2);
    let (code, v, _) = cmd(&w.dir, &["attachments", "demo", "Accounts", "x"]);
    assert_eq!((code, v["error"].as_str()), (2, Some("usage")));
    // the same name again is refused live (AttachmentAbsent), nothing sent twice
    let f = w.dir.join("again.ops.json");
    std::fs::write(
        &f,
        json!({"id": "again-01", "ops": [{"kind": "attach", "id": "op-1", "org": "demo",
            "entity": "BankTransactions", "journal": "bt-104", "file": "receipt-0304.csv"}]})
        .to_string(),
    )
    .unwrap();
    let (code, v, text) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
    assert_eq!(code, 0, "{text}");
    let pkg = v["package"].as_str().unwrap().to_string();
    let (_, v, _) = cmd(&w.dir, &["write", "seal", &pkg]);
    let hash = v["hash"].as_str().unwrap().to_string();
    approve(&w.dir, &hash);
    let (code, _, text) = cmd_env(
        &w.dir,
        &["write", "run", &pkg],
        &[("OWNPURSE_WRITE_RUN", &hash)],
    );
    assert_eq!(code, 4, "{text}");
    assert!(text.contains("already attached"), "{text}");
}

// ── 4. create a bank transaction (cash, petty cash); void it as its recovery ─────────────────────

#[test]
fn a_petty_cash_spend_lands_once_and_its_recovery_voids_it() {
    let w = world("bank-txn");
    let input = json!({"id": "petty-01", "title": "Petty cash, March", "ops": [
        {"kind": "create_bank_transaction", "id": "op-1", "org": "demo", "type": "SPEND",
         "bank_account": "1020", "contact": "Hollowbrook Fuel", "date": "2026-10-03",
         "reference": "receipt 0412",
         "lines": [{"account": "6520", "amount": "23.47", "description": "fuel"},
                   {"account": "6310", "amount": "8.06", "description": "cups"}]}]});
    let (pkg, _) = landed(&w, "petty", input.clone());
    let p: Value =
        serde_json::from_str(&std::fs::read_to_string(w.dir.join(&pkg)).unwrap()).unwrap();
    let op = &p["ops"][0];
    assert_eq!(op["op"]["kind"], "create_bank_transaction");
    assert_eq!(
        op["op"]["contact"],
        json!({"id": "ct-201"}),
        "the name resolved to the id"
    );
    assert_eq!(
        op["request"]["body"]["json"]["BankTransactions"][0]["Contact"],
        json!({"ContactID": "ct-201"})
    );
    let deltas: Vec<(String, String)> = op["expected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["code"].as_str().unwrap().into(),
                d["amount"].as_str().unwrap().into(),
            )
        })
        .collect();
    assert_eq!(
        deltas,
        [
            ("1020".into(), "-31.53".into()),
            ("6310".into(), "8.06".into()),
            ("6520".into(), "23.47".into())
        ]
    );
    assert_eq!(op["recovery"]["op"]["kind"], "void_bank_transaction");
    let made: Vec<Value> = w.fake.with(|s| {
        s.bank[T]
            .values()
            .filter(|t| {
                t["Reference"]
                    .as_str()
                    .is_some_and(|r| r.contains("XA:petty-01:op-1"))
            })
            .cloned()
            .collect()
    });
    assert_eq!(made.len(), 1);
    assert_eq!(made[0]["Reference"], "receipt 0412 - XA:petty-01:op-1");
    assert_eq!(made[0]["Total"].to_string(), "31.53");
    // the same transaction again (another package): refused at plan, from the record
    let mut again = input.clone();
    again["id"] = json!("petty-02");
    let f = w.dir.join("again.ops.json");
    std::fs::write(&f, again.to_string()).unwrap();
    let (code, v, text) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
    assert_eq!(code, 4, "{text}");
    assert!(v["message"].as_str().unwrap().contains("duplicate"), "{v}");
    // the recovery: a void (Status DELETED), its own seal and approval
    let (code, v, text) = cmd(&w.dir, &["write", "recover", &pkg]);
    assert_eq!(code, 0, "{text}");
    let rec = v["recovery"].as_str().unwrap().to_string();
    let (code, v, text) = cmd(&w.dir, &["write", "seal", &rec]);
    assert_eq!(code, 0, "{text}");
    let rh = v["hash"].as_str().unwrap().to_string();
    approve(&w.dir, &rh);
    let (code, _, text) = cmd_env(
        &w.dir,
        &["write", "run", &rec],
        &[("OWNPURSE_WRITE_RUN", &rh)],
    );
    assert_eq!(code, 0, "{text}");
    let status = w.fake.with(|s| {
        s.bank[T]
            .values()
            .find(|t| {
                t["Reference"]
                    .as_str()
                    .is_some_and(|r| r.contains("XA:petty-01:op-1"))
            })
            .map(|t| t["Status"].clone())
    });
    assert_eq!(status, Some(json!("DELETED")));
    let (code, v, text) = cmd(&w.dir, &["write", "verify", &rec]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["outcome"], "verified");
}

#[test]
fn bank_transaction_creates_refuse_what_xero_would_get_wrong() {
    let w = world("bank-txn-bad");
    let plan = |name: &str, op: Value| {
        let f = w.dir.join(format!("{name}.ops.json"));
        std::fs::write(&f, json!({"id": name, "ops": [op]}).to_string()).unwrap();
        cmd(&w.dir, &["write", "plan", f.to_str().unwrap()])
    };
    let base = json!({"kind": "create_bank_transaction", "id": "op-1", "org": "demo", "type": "SPEND",
        "bank_account": "1020", "contact": "Hollowbrook Fuel", "date": "2026-10-03",
        "lines": [{"account": "6520", "amount": "4.10"}]});
    for (name, field, value, want) in [
        // a name Xero doesn't know would create a new contact: never sent
        ("b1", "contact", json!("Nobody Ltd"), "no contact"),
        ("b2", "bank_account", json!("6520"), "no bank account"),
        ("b3", "type", json!("TRANSFER"), "SPEND or RECEIVE"),
        (
            "b4",
            "lines",
            json!([{"account": "6520", "amount": "-4.10"}]),
            "positive",
        ),
        (
            "b5",
            "lines",
            json!([{"account": "1010", "amount": "4.10"}]),
            "bank account",
        ),
    ] {
        let mut op = base.clone();
        op[field] = value;
        let (code, v, text) = plan(name, op);
        assert_eq!(code, 2, "{name}: {text}");
        assert!(
            v["message"].as_str().unwrap_or("").contains(want),
            "{name}: {v}"
        );
    }
    // a reconciled transaction can't be voided through the API
    let (code, v, text) = plan(
        "b6",
        json!({"kind": "void_bank_transaction", "id": "op-1", "org": "demo", "bank_transaction": "bt-105"}),
    );
    assert_eq!(code, 2, "{text}");
    assert_eq!(v["error"], "reconciled_line");
}

#[test]
fn a_manifest_bank_transaction_compiles_to_a_create_and_a_void_to_status_deleted() {
    let w = world("mf-bank");
    std::fs::write(
        w.dir.join("cash.json"),
        json!({"Type": "SPEND", "Contact": {"Name": "Hollowbrook Fuel"}, "BankAccount": {"Code": "1020"},
               "Date": "2026-10-03", "Reference": "cash receipt 7", "Status": "AUTHORISED",
               "LineItems": [{"AccountCode": "6520", "LineAmount": "5.50", "Description": "fuel"}]})
        .to_string(),
    )
    .unwrap();
    std::fs::write(
        w.dir.join("void.json"),
        json!([{"op": "replace", "path": "/Status", "value": "DELETED"}]).to_string(),
    )
    .unwrap();
    for args in [
        vec!["report", "fetch", "demo", "tb", "--date", "2026-10-31"],
        vec!["manifest", "new", "cash", "Cash entries"],
        vec![
            "manifest",
            "add",
            "--entity",
            "BankTransactions",
            "--org",
            "demo",
            "--after",
            "cash.json",
        ],
        vec![
            "manifest",
            "add",
            "--entity",
            "BankTransactions",
            "--org",
            "demo",
            "--id",
            "bt-104",
            "--patch",
            "void.json",
        ],
    ] {
        let (code, _, text) = cmd(&w.dir, &args);
        assert_eq!(code, 0, "{args:?}: {text}");
    }
    // the void touches a 2025 document: the accountant's reference covers it
    let (_, c, check) = cmd(&w.dir, &["manifest", "check"]);
    let gated = c["issues"]
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["code"] == "accountant")
        .map(|i| i["entry"].to_string())
        .expect("the void is gated");
    let (code, _, text) = cmd(
        &w.dir,
        &[
            "manifest",
            "annotate",
            &gated,
            "--accountant",
            "AJE-2025-07",
        ],
    );
    assert_eq!(code, 0, "{text}");
    let (code, v, text) = cmd(&w.dir, &["manifest", "plan"]);
    assert_eq!(code, 0, "{text}\n{check}");
    let pkg = v["package"].as_str().unwrap();
    let p: Value = serde_json::from_str(
        &std::fs::read_to_string(w.dir.join(pkg))
            .unwrap_or_else(|_| std::fs::read_to_string(pkg).unwrap()),
    )
    .unwrap();
    let kinds: Vec<&str> = p["ops"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["op"]["kind"].as_str().unwrap())
        .collect();
    assert_eq!(
        kinds,
        ["create_bank_transaction", "void_bank_transaction"],
        "{p}"
    );
    assert_eq!(p["ops"][0]["op"]["contact"], json!({"id": "ct-201"}));
    assert_eq!(
        p["ops"][1]["request"]["body"]["json"]["BankTransactions"][0]["Status"],
        "DELETED"
    );
}

#[test]
fn the_new_probes_plan_on_a_demo_company() {
    let w = world("probes");
    for probe in ["history-note", "bank-transaction"] {
        let (code, v, text) = cmd(&w.dir, &["write", "probe", probe, "--out", "probes"]);
        assert_eq!(code, 0, "{probe}: {text}");
        let input = v["input"].as_str().unwrap().to_string();
        let (code, v, text) = cmd(&w.dir, &["write", "plan", &input]);
        assert_eq!(code, 0, "{probe}: {text}");
        let pkg = v["package"].as_str().unwrap();
        let p: Value = serde_json::from_str(
            &std::fs::read_to_string(w.dir.join(pkg))
                .unwrap_or_else(|_| std::fs::read_to_string(pkg).unwrap()),
        )
        .unwrap();
        match probe {
            "history-note" => {
                assert!(p["ops"][0]["history_note"].is_string(), "{p}");
                assert_eq!(p["ops"][1]["op"]["kind"], "add_note");
            }
            _ => {
                assert_eq!(p["ops"][0]["op"]["kind"], "create_bank_transaction");
                assert_eq!(
                    p["ops"][0]["recovery"]["op"]["kind"],
                    "void_bank_transaction"
                );
            }
        }
    }
}

#[test]
fn history_takes_an_id_prefix_through_the_record_and_shows_entities_decoded() {
    let w = world("history-prefix");
    let before = w.fake.with(|s| s.log.len());
    // a prefix that matches nothing, or several: refused, Xero never asked
    let (code, v, _) = cmd(&w.dir, &["history", "demo", "BankTransactions", "zz"]);
    assert_eq!((code, v["error"].as_str()), (2, Some("not_found")), "{v}");
    let (code, v, _) = cmd(&w.dir, &["history", "demo", "BankTransactions", "bt-10"]);
    assert_eq!((code, v["error"].as_str()), (2, Some("ambiguous")), "{v}");
    assert_eq!(
        w.fake.with(|s| s.log.len()),
        before,
        "no call with a partial id"
    );
    // a unique prefix resolves; Xero's HTML entities are shown decoded (any source)
    w.fake.with(|s| {
        s.history.insert(
            (T.to_string(), "BankTransactions/bt-104".into()),
            vec![ownpurse_xero::write::fake::State::history_record(
                "Note",
                "fuel &#183; market run &amp; back",
            )],
        );
    });
    let (code, v, text) = cmd(&w.dir, &["history", "demo", "BankTransactions", "bt-104"]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(
        v["history"][0]["details"], "fuel \u{b7} market run & back",
        "{v}"
    );
    let (code, v, text) = cmd(
        &w.dir,
        &["history", "demo", "BankTransactions", "bt-1", "--cached"],
    );
    assert_eq!(code, 2, "bt-1 is ambiguous: {text}");
    let _ = v;
    let (_, v, _) = cmd(
        &w.dir,
        &["history", "demo", "BankTransactions", "bt-104", "--cached"],
    );
    assert_eq!(v["history"][0]["details"], "fuel \u{b7} market run & back");
    // a journal by a unique prefix (the record knows it after a sync)
    let (code, v, text) = cmd(&w.dir, &["attachments", "demo", "BankTransactions", "bt-1"]);
    assert_eq!(
        (code, v["error"].as_str()),
        (2, Some("ambiguous")),
        "{text}"
    );
    // what this tool writes is ASCII: a non-ASCII note is folded at plan
    let f = w.dir.join("n.ops.json");
    std::fs::write(
        &f,
        json!({"id": "fold-01", "ops": [{"kind": "add_note", "id": "op-1", "org": "demo",
            "entity": "BankTransactions", "document": "bt-104", "details": "caf\u{e9} \u{b7} 6520 \u{2192} 6310"}]})
        .to_string(),
    )
    .unwrap();
    let (code, v, text) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
    assert_eq!(code, 0, "{text}");
    let p: Value = serde_json::from_str(
        &std::fs::read_to_string(w.dir.join(v["package"].as_str().unwrap())).unwrap(),
    )
    .unwrap();
    assert_eq!(p["ops"][0]["op"]["details"], "caf? - 6520 -> 6310");
}

// ── 5 & 6. bank transfers (reverse transfer as recovery) and contacts (exact-name guard) ─────────

#[test]
fn petty_cash_funding_is_a_transfer_whose_recovery_is_a_reverse_transfer() {
    let w = world("transfer");
    let input = json!({"id": "fund-01", "title": "Petty cash from the ATM", "ops": [
        {"kind": "create_bank_transfer", "id": "op-1", "org": "demo", "from": "1010", "to": "Petty cash",
         "amount": "200.00", "date": "2026-10-03", "reference": "ATM withdrawal"}]});
    let (pkg, _) = landed(&w, "fund", input.clone());
    let p: Value =
        serde_json::from_str(&std::fs::read_to_string(w.dir.join(&pkg)).unwrap()).unwrap();
    let op = &p["ops"][0];
    assert_eq!(op["op"]["kind"], "create_bank_transfer");
    assert_eq!(op["op"]["reference"], "ATM withdrawal - XA:fund-01:op-1");
    let deltas: Vec<(String, String)> = op["expected"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| {
            (
                d["code"].as_str().unwrap().into(),
                d["amount"].as_str().unwrap().into(),
            )
        })
        .collect();
    assert_eq!(
        deltas,
        [
            ("1010".into(), "-200.00".into()),
            ("1020".into(), "200.00".into())
        ]
    );
    assert!(
        op["flags"][0]
            .as_str()
            .unwrap()
            .starts_with("permanent history")
    );
    assert_eq!(op["recovery"]["op"]["kind"], "create_bank_transfer");
    assert_eq!(w.fake.with(|s| s.transfers[T].len()), 1);
    // again: refused at plan (the record holds it after the run's read-back)
    let mut again = input;
    again["id"] = json!("fund-02");
    let f = w.dir.join("again.ops.json");
    std::fs::write(&f, again.to_string()).unwrap();
    let (code, v, _) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
    assert_eq!(code, 4, "{v}");
    // the recovery: the reverse transfer, under its own marker and approval
    let (code, v, text) = cmd(&w.dir, &["write", "recover", &pkg]);
    assert_eq!(code, 0, "{text}");
    let rec = v["recovery"].as_str().unwrap().to_string();
    let (_, v, _) = cmd(&w.dir, &["write", "seal", &rec]);
    let rh = v["hash"].as_str().unwrap().to_string();
    approve(&w.dir, &rh);
    let (code, _, text) = cmd_env(
        &w.dir,
        &["write", "run", &rec],
        &[("OWNPURSE_WRITE_RUN", &rh)],
    );
    assert_eq!(code, 0, "{text}");
    let (code, v, text) = cmd(&w.dir, &["write", "verify", &rec]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["outcome"], "verified");
    let back = w.fake.with(|s| {
        s.transfers[T]
            .values()
            .find(|t| {
                t["Reference"]
                    .as_str()
                    .is_some_and(|r| r.contains("fund-01-recovery"))
            })
            .map(|t| {
                (
                    t["FromBankAccount"]["Code"].clone(),
                    t["ToBankAccount"]["Code"].clone(),
                    t["Amount"].to_string(),
                )
            })
    });
    assert_eq!(back, Some((json!("1020"), json!("1010"), "200.00".into())));
    // refusals at plan
    for (name, op, want) in [
        (
            "t1",
            json!({"kind": "create_bank_transfer", "id": "op-1", "org": "demo", "from": "1010", "to": "1010",
                      "amount": "5.00", "date": "2026-10-03"}),
            "two different",
        ),
        (
            "t2",
            json!({"kind": "create_bank_transfer", "id": "op-1", "org": "demo", "from": "1010", "to": "6520",
                      "amount": "5.00", "date": "2026-10-03"}),
            "no bank account",
        ),
        (
            "t3",
            json!({"kind": "create_bank_transfer", "id": "op-1", "org": "demo", "from": "1010", "to": "1020",
                      "amount": "0", "date": "2026-10-03"}),
            "positive",
        ),
    ] {
        let f = w.dir.join(format!("{name}.ops.json"));
        std::fs::write(&f, json!({"id": name, "ops": [op]}).to_string()).unwrap();
        let (code, v, _) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
        assert_eq!(code, 2, "{name}: {v}");
        assert!(
            v["message"].as_str().unwrap_or("").contains(want),
            "{name}: {v}"
        );
    }
}

#[test]
fn a_new_contact_then_a_cash_receipt_from_it_in_one_package_and_archive_as_recovery() {
    let w = world("contact");
    let (pkg, _) = landed(
        &w,
        "lodger",
        json!({"id": "lodger-01", "title": "Studio share, cash", "ops": [
            {"kind": "create_contact", "id": "op-1", "org": "demo", "name": "Fenwick Lodger", "email": "fenwick@example.test"},
            {"kind": "create_bank_transaction", "id": "op-2", "org": "demo", "type": "RECEIVE",
             "bank_account": "1020", "contact": "op:op-1", "date": "2026-10-03",
             "lines": [{"account": "6310", "amount": "137.25", "description": "studio share, October"}]}]}),
    );
    let made = w.fake.with(|s| {
        let c = s.contacts[T]
            .values()
            .find(|c| c["Name"] == "Fenwick Lodger")
            .cloned()
            .unwrap();
        let t = s.bank[T]
            .values()
            .find(|t| {
                t["Reference"]
                    .as_str()
                    .is_some_and(|r| r.contains("XA:lodger-01:op-2"))
            })
            .cloned()
            .unwrap();
        (c, t)
    });
    assert_eq!(made.0["EmailAddress"], "fenwick@example.test");
    assert_eq!(
        made.1["Contact"]["ContactID"], made.0["ContactID"],
        "the created contact's id was filled in"
    );
    assert_eq!(made.1["Type"], "RECEIVE");
    // the exact name again: refused at plan (record) — and live, if the record were stale
    let f = w.dir.join("dup.ops.json");
    std::fs::write(
        &f,
        json!({"id": "dup-01", "ops": [{"kind": "create_contact", "id": "op-1", "org": "demo", "name": "Hollowbrook Fuel"}]})
            .to_string(),
    )
    .unwrap();
    let (code, v, _) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
    assert_eq!(code, 2, "{v}");
    assert!(v["message"].as_str().unwrap().contains("exists"), "{v}");
    // recovery: archive the contact, void the receipt
    let (code, v, text) = cmd(&w.dir, &["write", "recover", &pkg]);
    assert_eq!(code, 0, "{text}");
    let kinds: Vec<String> = v["ops"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["kind"].as_str().unwrap().to_string())
        .collect();
    assert_eq!(kinds, ["void_bank_transaction", "archive_contact"], "{v}");
    let rec = v["recovery"].as_str().unwrap().to_string();
    let (_, v, _) = cmd(&w.dir, &["write", "seal", &rec]);
    let rh = v["hash"].as_str().unwrap().to_string();
    approve(&w.dir, &rh);
    let (code, _, text) = cmd_env(
        &w.dir,
        &["write", "run", &rec],
        &[("OWNPURSE_WRITE_RUN", &rh)],
    );
    assert_eq!(code, 0, "{text}");
    let status = w.fake.with(|s| {
        s.contacts[T]
            .values()
            .find(|c| c["Name"] == "Fenwick Lodger")
            .map(|c| c["ContactStatus"].clone())
    });
    assert_eq!(status, Some(json!("ARCHIVED")));
}

// ── live findings: Xero's search index lags a fresh create; free text goes out in ASCII ──────────

#[test]
fn a_fresh_create_reads_back_by_its_id_even_while_the_search_index_lags() {
    let w = world("lag");
    w.fake.with(|s| s.search_lag = 50);
    let (pkg, _) = landed(
        &w,
        "lag",
        json!({"id": "lag-01", "title": "Cash \u{b7} March", "ops": [
            {"kind": "create_bank_transaction", "id": "op-1", "org": "demo", "type": "SPEND",
             "bank_account": "1020", "contact": "ct-201", "date": "2026-10-03",
             "reference": "receipt \u{b7} 0412 \u{2192} fuel",
             "lines": [{"account": "6520", "amount": "3.33", "description": "fuel \u{b7} cans"}]},
            {"kind": "create_journal", "id": "op-2", "org": "demo", "date": "2026-10-03",
             "narration": "Owner \u{b7} paid", "cash_basis": true,
             "lines": [{"account": "6310", "amount": "1.11", "description": "caf\u{e9}"},
                       {"account": "2700", "amount": "-1.11"}]},
            {"kind": "create_bank_transfer", "id": "op-3", "org": "demo", "from": "1010", "to": "1020",
             "amount": "9.99", "date": "2026-10-03"}]}),
    );
    // what went out is ASCII (Xero would keep anything else as HTML entities)
    let sent = w.fake.with(|s| {
        s.log
            .iter()
            .filter(|l| l.0 == "PUT")
            .map(|l| l.3.clone())
            .collect::<Vec<_>>()
    });
    assert!(sent.len() >= 3);
    assert!(sent.iter().all(|b| b.is_ascii()), "{sent:?}");
    let p: Value =
        serde_json::from_str(&std::fs::read_to_string(w.dir.join(&pkg)).unwrap()).unwrap();
    assert_eq!(
        p["ops"][0]["op"]["reference"],
        "receipt - 0412 -> fuel - XA:lag-01:op-1"
    );
    assert_eq!(
        p["ops"][1]["op"]["narration"],
        "Owner - paid - XA:lag-01:op-2"
    );
}

#[test]
fn reconcile_retries_the_marker_search_and_names_the_query_it_sent() {
    use ownpurse_xero::write::fake::Fault;
    let backoff = [("OWNPURSE_SEARCH_BACKOFF_MS", "10,10,10")];
    for (lag, landed_want) in [(2u32, true), (40, false)] {
        let w = world(&format!("reconcile-lag-{lag}"));
        let f = w.dir.join("r.ops.json");
        std::fs::write(
            &f,
            json!({"id": "rec-01", "ops": [{"kind": "create_bank_transaction", "id": "op-1", "org": "demo",
                "type": "SPEND", "bank_account": "1020", "contact": "ct-201", "date": "2026-10-03",
                "lines": [{"account": "6520", "amount": "2.22"}]}]})
            .to_string(),
        )
        .unwrap();
        let (_, v, _) = cmd(&w.dir, &["write", "plan", f.to_str().unwrap()]);
        let pkg = v["package"].as_str().unwrap().to_string();
        let (_, v, _) = cmd(&w.dir, &["write", "seal", &pkg]);
        let hash = v["hash"].as_str().unwrap().to_string();
        approve(&w.dir, &hash);
        // the create lands but the answer is lost: uncertain
        w.fake.with(|s| {
            s.search_lag = 0;
            s.faults
                .push_back(("BankTransactions".into(), Fault::DropAfterApply));
        });
        let (code, _, text) = cmd_env(
            &w.dir,
            &["write", "run", &pkg],
            &[("OWNPURSE_WRITE_RUN", &hash)],
        );
        assert_ne!(code, 0, "{text}");
        // the search index hasn't caught up for `lag` searches
        w.fake.with(|s| {
            let ids: Vec<String> = s.bank[T]
                .keys()
                .filter(|k| k.starts_with("0000000c"))
                .cloned()
                .collect();
            for id in ids {
                s.fresh.borrow_mut().insert(id, lag);
            }
        });
        let (code, v, text) = cmd_env(&w.dir, &["write", "reconcile", &pkg, "op-1"], &backoff);
        if landed_want {
            assert_eq!(code, 0, "{text}");
            assert_eq!(v["landed"], true, "found after retrying: {v}");
        } else {
            assert!(
                text.contains("searched 4 times: GET BankTransactions?where=Reference.Contains"),
                "the evidence names the query: {text}"
            );
        }
    }
}

// ── 7. export for the accountant ─────────────────────────────────────────────────────────────────

#[test]
fn export_writes_tb_gl_and_xeros_bs_pl_as_csv_and_never_overwrites() {
    let w = world("export");
    for (kind, args) in [
        ("tb", vec!["--date", "2025-12-31", "--out", "tb.csv"]),
        (
            "gl",
            vec![
                "--from",
                "2025-01-01",
                "--to",
                "2025-12-31",
                "--out",
                "gl.csv",
            ],
        ),
    ] {
        let mut a = vec!["export", "demo", kind];
        a.extend(args);
        let (code, v, text) = cmd(&w.dir, &a);
        assert_eq!(code, 0, "{kind}: {text}");
        assert!(v["rows"].as_i64().unwrap() >= 1, "{v}");
    }
    let tb = std::fs::read_to_string(w.dir.join("tb.csv")).unwrap();
    assert!(
        tb.starts_with("code,account,class,debit,credit\r\n"),
        "{tb}"
    );
    assert!(tb.contains("6520,Van fuel,EXPENSE,238.61,"), "{tb}");
    assert!(tb.contains(",Total,,299.70,299.70\r\n"), "balanced: {tb}");
    let gl = std::fs::read_to_string(w.dir.join("gl.csv")).unwrap();
    assert!(gl.starts_with(
        "date,code,account,debit,credit,source,source_id,contact,reference,description\r\n"
    ));
    assert!(
        gl.contains("2025-03-04,6520,Van fuel,238.61,,BankTransactions,bt-104"),
        "{gl}"
    );
    // never over an existing file
    let (code, _, _) = cmd(
        &w.dir,
        &[
            "export",
            "demo",
            "tb",
            "--date",
            "2025-12-31",
            "--out",
            "tb.csv",
        ],
    );
    assert_eq!(code, 2);
    // bs / pl: Xero's reports as `report fetch` cached them, else a hint to fetch
    let (code, v, _) = cmd(
        &w.dir,
        &[
            "export",
            "demo",
            "bs",
            "--date",
            "2026-10-31",
            "--out",
            "bs.csv",
        ],
    );
    assert_eq!(code, 2, "{v}");
    assert!(
        v["help"][0]
            .as_str()
            .unwrap()
            .contains("report fetch demo balancesheet --date 2026-10-31")
    );
    cmd(
        &w.dir,
        &[
            "report",
            "fetch",
            "demo",
            "balancesheet",
            "--date",
            "2026-10-31",
        ],
    );
    let (code, _, text) = cmd(
        &w.dir,
        &[
            "export",
            "demo",
            "bs",
            "--date",
            "2026-10-31",
            "--out",
            "bs.csv",
        ],
    );
    assert_eq!(code, 0, "{text}");
    let bs = std::fs::read_to_string(w.dir.join("bs.csv")).unwrap();
    assert!(
        bs.contains("Balance Sheet\r\n") && bs.contains("Van fuel (6520)"),
        "{bs}"
    );
    let (code, v, _) = cmd(&w.dir, &["export", "demo", "tb", "--xlsx"]);
    assert_eq!(
        (code, v["error"].as_str()),
        (2, Some("not_available")),
        "{v}"
    );
}

#[test]
fn pl_and_bs_come_from_the_local_ledger_and_name_the_report_to_check_against() {
    let w = world("statements");
    let (code, v, text) = cmd(
        &w.dir,
        &[
            "bs",
            "demo",
            "--date",
            "2025-12-31",
            "--cash",
            "--full-history",
        ],
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["balanced"], json!(true), "{v}");
    assert_eq!(v["basis"], json!("cash"));
    assert_eq!(v["xero_report"], json!("none cached"));
    assert!(
        v["help"][0]
            .as_str()
            .unwrap()
            .contains("report fetch demo balancesheet --date 2025-12-31 --cash"),
        "{v}"
    );
    let (code, v, text) = cmd(
        &w.dir,
        &["pl", "demo", "--from", "2025-01-01", "--to", "2025-12-31"],
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["basis"], json!("accrual"));
    assert!(
        v["lines"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["account"] == "Van fuel"),
        "{v}"
    );
    // an opening named but not cached is a usage error that says how to fetch it
    let (code, v, _) = cmd(
        &w.dir,
        &[
            "bs",
            "demo",
            "--date",
            "2025-12-31",
            "--since",
            "2024-12-31",
        ],
    );
    assert_eq!(code, 2, "{v}");
}
