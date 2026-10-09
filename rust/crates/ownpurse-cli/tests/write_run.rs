//! The whole write path through the real CLI binary against the loopback fake Xero (never the network:
//! `OWNPURSE_WRITE_LOOPBACK_ONLY=1`): sync → probe → plan → seal → approval → run (dry and real) → verify →
//! status; probe-replay on a Demo Company only; recover with its own approval. The approval is appended by
//! the test through the record's ApprovalStore, standing in for the owner at his TTY (approve-record's own
//! TTY and agent refusals are tested in tests/write.rs).

use ownpurse_core::write::{AllOps, ApprovalRecord, ApprovalScope, ApprovalStore, Via};
use ownpurse_xero::store::Writer;
use ownpurse_xero::write::RecordApprovals;
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

fn world(name: &str, tenant_name: &str, require: Option<&str>) -> W {
    let dir = std::env::temp_dir().join(format!("xa-write-run-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let mut s = State::default();
    s.add_tenant(
        T,
        tenant_name,
        &[
            ("1010", "Operating", "BANK"),
            ("6310", "Cupping supplies", "EXPENSE"),
            ("6520", "Van fuel", "EXPENSE"),
            ("2700", "Owner loan", "LIABILITY"),
        ],
    );
    s.add_bank_spend(T, "bt-104", "1010", "6520", "238.61", false);
    let fake = FakeXero::start(s).unwrap();
    let mut profile = json!({"client_id": "DEMO0CLIENT0ID0000000000000000000", "store": "record.sqlite",
        "token_file": "tokens.json", "orgs": {"demo": T}, "allow_writes": true});
    if let Some(r) = require {
        profile["require_tenant_name"] = json!(r);
    }
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
               "scope": "accounting.transactions.read accounting.settings.read accounting.manualjournals accounting.banktransactions accounting.attachments"})
        .to_string(),
    )
    .unwrap();
    W { dir, fake }
}

fn cmd(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> (i32, Value, String) {
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

fn approve(dir: &Path, hash: &str) {
    let store = Writer::open(dir.join("record.sqlite")).unwrap();
    RecordApprovals { store: &store }
        .record(&ApprovalRecord {
            hash: hash.into(),
            verbatim: format!("Approved, {}", &hash[..12]),
            via: Via::Tty,
            where_: "tty".into(),
            at: "2026-10-07T12:30:00Z".into(),
            scope: ApprovalScope::All(AllOps::All),
            recorded_at: "2026-10-07T12:30:01Z".into(),
        })
        .unwrap();
}

/// probe → plan → seal: (package path, hash).
fn planned(w: &W, probe: &str, extra: &[&str]) -> (String, String) {
    let mut a = vec!["write", "probe", probe, "--out", "probes"];
    a.extend_from_slice(extra);
    let (code, v, text) = cmd(&w.dir, &a, &[]);
    assert_eq!(code, 0, "probe: {text}");
    let input = v["input"].as_str().unwrap().to_string();
    let (code, v, text) = cmd(&w.dir, &["write", "plan", &input], &[]);
    assert_eq!(code, 0, "plan: {text}");
    let pkg = v["package"].as_str().unwrap().to_string();
    let (code, v, text) = cmd(&w.dir, &["write", "seal", &pkg], &[]);
    assert_eq!(code, 0, "seal: {text}");
    let hash = v["hash"]
        .as_str()
        .or_else(|| v["seal"].as_str())
        .map(str::to_string)
        .unwrap_or_else(|| {
            let p: Value =
                serde_json::from_str(&std::fs::read_to_string(w.dir.join(&pkg)).unwrap()).unwrap();
            p["seal"].as_str().unwrap().to_string()
        });
    (pkg, hash)
}

fn sync(w: &W) {
    let (code, _, text) = cmd(&w.dir, &["sync", "demo"], &[]);
    assert_eq!(code, 0, "sync: {text}");
    // the connected tenants (names) into the token file, as `orgs` stores them
    let (code, _, text) = cmd(&w.dir, &["orgs"], &[]);
    assert!(
        code == 0 || text.contains("may only connect"),
        "orgs: {text}"
    );
}

#[test]
fn the_void_after_attachment_probe_end_to_end() {
    let w = world("void", DEMO, Some(DEMO));
    sync(&w);
    let (pkg, hash) = planned(&w, "void-after-attachment", &[]);
    // no approval: refused, nothing sent
    let (code, v, _) = cmd(
        &w.dir,
        &["write", "run", &pkg],
        &[("OWNPURSE_WRITE_RUN", &hash)],
    );
    assert_eq!(
        (code, v["error"].as_str()),
        (3, Some("not_approved")),
        "{v}"
    );
    approve(&w.dir, &hash);
    // the env lock is for this hash and this invocation
    let (code, _, _) = cmd(&w.dir, &["write", "run", &pkg], &[]);
    assert_eq!(code, 7);
    let (code, v, text) = cmd(&w.dir, &["write", "run", &pkg, "--dry-run"], &[]);
    assert_eq!(code, 0, "{text}");
    let dry: Vec<&str> = v["ops"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["outcome"].as_str().unwrap())
        .collect();
    assert_eq!(
        dry,
        ["would_send", "after_dependency", "after_dependency"],
        "{v}"
    );
    assert_eq!(
        w.fake
            .with(|s| s.log.iter().filter(|l| l.0 != "GET").count()),
        0
    );
    let (code, v, text) = cmd(
        &w.dir,
        &["write", "run", &pkg],
        &[("OWNPURSE_WRITE_RUN", &hash)],
    );
    assert_eq!(code, 0, "{text}");
    let outcomes: Vec<&str> = v["ops"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["outcome"].as_str().unwrap())
        .collect();
    assert_eq!(outcomes, ["confirmed", "confirmed", "confirmed"], "{v}");
    let (code, v, text) = cmd(&w.dir, &["write", "verify", &pkg], &[]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["ok"], true);
    let rec = ownpurse_core::Record::open(w.dir.join("record.sqlite")).unwrap();
    assert_eq!(
        ownpurse_core::write::view::last_verify(&rec, &hash).as_deref(),
        Some("verified")
    );
    let (code, v, _) = cmd(&w.dir, &["write", "status", &pkg], &[]);
    assert_eq!(code, 0);
    assert_eq!(v["sealed"], true);
    assert!(v["chain"]["entries"].as_i64().unwrap() > 0, "{v}");
}

#[test]
fn the_idempotency_probe_replays_on_a_demo_company_only() {
    let w = world("idem", DEMO, Some(DEMO));
    sync(&w);
    let (pkg, hash) = planned(&w, "idempotency", &[]);
    approve(&w.dir, &hash);
    let env = [("OWNPURSE_WRITE_RUN", hash.as_str())];
    let (code, _, text) = cmd(&w.dir, &["write", "run", &pkg], &env);
    assert_eq!(code, 0, "{text}");
    let (code, v, text) = cmd(&w.dir, &["write", "probe-replay", &pkg, "op-1"], &env);
    assert_eq!(code, 0, "{text}");
    assert_eq!(
        (v["marker_count"].as_u64(), v["duplicate"].as_bool()),
        (Some(1), Some(false)),
        "{v}"
    );
    let (code, v, text) = cmd(
        &w.dir,
        &["write", "probe-replay", &pkg, "op-1", "--changed"],
        &env,
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(
        v["http_status"], 400,
        "this fake refuses a changed body under a used key: {v}"
    );
    let (_, st, _) = cmd(&w.dir, &["write", "status", &pkg], &[]);
    let attempts: Vec<Value> = st["ops"][0]["attempts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a["attempt"].clone())
        .collect();
    assert!(
        attempts.contains(&json!("replay")) && attempts.contains(&json!("replay_changed")),
        "{st}"
    );
    assert!(st["ops"][0]["status"] == "landed", "{st}");
}

#[test]
fn probes_and_replays_refuse_anything_but_a_demo_company() {
    // a profile without require_tenant_name: the probe kit refuses before reading anything
    let w = world("notdemo", "Brindlewick Roastery", None);
    let (code, v, _) = cmd(&w.dir, &["write", "probe", "idempotency"], &[]);
    assert_eq!((code, v["error"].as_str()), (2, Some("not_a_demo")), "{v}");
    // a profile that requires a name, connected to a tenant that isn't a Demo Company
    let w = world(
        "notdemo2",
        "Brindlewick Roastery",
        Some("Brindlewick Roastery"),
    );
    let (code, v, _) = cmd(&w.dir, &["write", "probe", "idempotency"], &[]);
    assert_eq!((code, v["error"].as_str()), (2, Some("not_a_demo")), "{v}");
}

#[test]
fn the_recode_probe_and_its_recovery() {
    let w = world("recode", DEMO, Some(DEMO));
    sync(&w);
    let (pkg, hash) = planned(&w, "recode", &[]);
    approve(&w.dir, &hash);
    let (code, v, text) = cmd(
        &w.dir,
        &["write", "run", &pkg],
        &[("OWNPURSE_WRITE_RUN", &hash)],
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["ops"][0]["outcome"], "confirmed");
    let (code, v, text) = cmd(&w.dir, &["write", "verify", &pkg], &[]);
    assert_eq!(code, 0, "{text} {v}");
    // the recovery: its own seal and approval
    let (code, v, text) = cmd(&w.dir, &["write", "recover", &pkg], &[]);
    assert_eq!(code, 0, "{text}");
    let rec = v["recovery"].as_str().unwrap().to_string();
    let (code, v, text) = cmd(&w.dir, &["write", "seal", &rec], &[]);
    assert_eq!(code, 0, "{text}");
    let rh = v["hash"].as_str().map(str::to_string).unwrap_or_else(|| {
        let p: Value = serde_json::from_str(&std::fs::read_to_string(&rec).unwrap()).unwrap();
        p["seal"].as_str().unwrap().to_string()
    });
    let (code, _, _) = cmd(
        &w.dir,
        &["write", "run", &rec],
        &[("OWNPURSE_WRITE_RUN", &rh)],
    );
    assert_eq!(code, 3, "the original approval doesn't cover the recovery");
    approve(&w.dir, &rh);
    let (code, _, text) = cmd(
        &w.dir,
        &["write", "run", &rec],
        &[("OWNPURSE_WRITE_RUN", &rh)],
    );
    assert_eq!(code, 0, "{text}");
    let back = w
        .fake
        .with(|s| s.bank[T]["bt-104"]["LineItems"][0]["AccountCode"].clone());
    assert_eq!(back, "6520");
}

fn standing(w: &W) {
    std::fs::write(
        w.dir.join("standing-approval.txt"),
        "Play freely on the Demo Company; it can be reset.\n",
    )
    .unwrap();
}

#[test]
fn the_demo_standing_approval_lets_an_agent_run_a_demo_package() {
    let w = world("standing", DEMO, Some(DEMO));
    sync(&w);
    standing(&w);
    let (pkg, hash) = planned(&w, "idempotency", &[]);
    let (code, v, text) = cmd(
        &w.dir,
        &["write", "approve-demo", &pkg],
        &[("CLAUDECODE", "1")],
    );
    assert_eq!(code, 0, "{text}");
    assert_eq!(v["via"], "demo_standing");
    let (code, _, text) = cmd(
        &w.dir,
        &["write", "run", &pkg],
        &[("OWNPURSE_WRITE_RUN", &hash), ("CLAUDECODE", "1")],
    );
    assert_eq!(code, 0, "{text}");
    let (_, st, _) = cmd(&w.dir, &["write", "status", &pkg], &[]);
    assert_eq!(
        st["approvals"][0]["source"], "demo standing approval",
        "{st}"
    );
}

#[test]
fn the_standing_approval_never_applies_outside_a_demo_company() {
    // a real-looking profile, with the file present
    let w = world(
        "standing-real",
        "Brindlewick Roastery",
        Some("Brindlewick Roastery"),
    );
    sync(&w);
    standing(&w);
    std::fs::write(w.dir.join("ops.json"), json!({"id": "x1", "title": "x", "ops": [
        {"kind": "create_journal", "id": "op-1", "org": "demo", "date": "2026-10-07", "narration": "n",
         "cash_basis": true, "lines": [{"account": "6310", "amount": "0.01"}, {"account": "6520", "amount": "-0.01"}]}]}).to_string()).unwrap();
    let (code, v, text) = cmd(&w.dir, &["write", "plan", "ops.json"], &[]);
    assert_eq!(code, 0, "{text}");
    let pkg = v["package"].as_str().unwrap().to_string();
    let (code, _, text) = cmd(&w.dir, &["write", "seal", &pkg], &[]);
    assert_eq!(code, 0, "{text}");
    let (code, v, _) = cmd(&w.dir, &["write", "approve-demo", &pkg], &[]);
    assert_eq!((code, v["error"].as_str()), (2, Some("not_a_demo")), "{v}");
    // a demo profile whose connected tenant isn't the named Demo Company
    let w = world("standing-other", "Demo Company (UK)", Some(DEMO));
    sync(&w);
    standing(&w);
    let (code, v, text) = cmd(
        &w.dir,
        &["write", "probe", "idempotency", "--out", "probes"],
        &[],
    );
    assert_eq!(code, 0, "{text}");
    let (_, v2, _) = cmd(
        &w.dir,
        &["write", "plan", v["input"].as_str().unwrap()],
        &[],
    );
    let pkg = v2["package"].as_str().unwrap().to_string();
    cmd(&w.dir, &["write", "seal", &pkg], &[]);
    let (code, v, _) = cmd(&w.dir, &["write", "approve-demo", &pkg], &[]);
    assert_eq!((code, v["error"].as_str()), (2, Some("not_a_demo")), "{v}");
}

#[test]
fn the_seal_review_reads_as_lines_and_never_shows_a_signed_zero() {
    let w = world("review", DEMO, Some(DEMO));
    sync(&w);
    let (code, v, text) = cmd(
        &w.dir,
        &["write", "probe", "idempotency", "--out", "probes"],
        &[],
    );
    assert_eq!(code, 0, "{text}");
    let (_, v2, _) = cmd(
        &w.dir,
        &["write", "plan", v["input"].as_str().unwrap()],
        &[],
    );
    let pkg = v2["package"].as_str().unwrap().to_string();
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(["write", "seal", &pkg])
        .current_dir(&w.dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", &w.dir)
        .env("OWNPURSE_CONFIG", w.dir.join("ownpurse.json"))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("\nreview:\n  "), "{text}");
    assert!(text.contains("profit effect 0.00"), "{text}");
    assert!(!text.contains("-0.00"), "{text}");
}

#[test]
fn text_output_has_no_debug_dumps() {
    let w = world("text", DEMO, Some(DEMO));
    sync(&w);
    let (pkg, hash) = planned(&w, "void-after-attachment", &[]);
    approve(&w.dir, &hash);
    for args in [
        vec!["write", "run", pkg.as_str(), "--dry-run"],
        vec!["write", "run", pkg.as_str()],
        vec!["write", "status", pkg.as_str()],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
            .args(&args)
            .current_dir(&w.dir)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("HOME", &w.dir)
            .env("OWNPURSE_CONFIG", w.dir.join("ownpurse.json"))
            .env("OWNPURSE_WRITE_LOOPBACK_ONLY", "1")
            .env("OWNPURSE_WRITE_RUN", &hash)
            .output()
            .unwrap();
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(out.status.success(), "{args:?}: {text}");
        for bad in ["{'", "': ", "True", "False", "None"] {
            assert!(!text.contains(bad), "{args:?} shows `{bad}`:\n{text}");
        }
    }
}

/// Strict JSON (serde_json rejects raw control characters in strings, as Python's json.loads does).
fn strict(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Value {
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
    let text = String::from_utf8(out.stdout).expect("UTF-8");
    assert!(
        !text
            .trim_end_matches('\n')
            .chars()
            .any(|c| (c as u32) < 0x20 && c != '\n' && c != ' '),
        "{args:?}: a raw control character"
    );
    serde_json::from_str::<Value>(&text)
        .unwrap_or_else(|e| panic!("{args:?}: not strict JSON ({e}):\n{text}"))
}

#[test]
fn every_write_command_answers_strict_json() {
    let w = world("strict", DEMO, Some(DEMO));
    sync(&w);
    standing(&w);
    let v = strict(
        &w.dir,
        &["write", "probe", "void-after-attachment", "--out", "probes"],
        &[],
    );
    let input = v["input"].as_str().unwrap().to_string();
    let v = strict(&w.dir, &["write", "plan", &input], &[]);
    let pkg = v["package"].as_str().unwrap().to_string();
    let v = strict(&w.dir, &["write", "seal", &pkg], &[]);
    assert!(
        v["review"].as_str().unwrap().contains('\n'),
        "the review is one string with its lines"
    );
    let hash = v["hash"].as_str().unwrap().to_string();
    let env = [("OWNPURSE_WRITE_RUN", hash.as_str())];
    strict(&w.dir, &["write", "export", &pkg], &[]);
    strict(&w.dir, &["write", "approve-check", &pkg], &[]);
    strict(&w.dir, &["write", "approve-demo", &pkg], &[]);
    strict(&w.dir, &["write", "approve-check", &pkg], &[]);
    strict(&w.dir, &["write", "run", &pkg, "--dry-run"], &[]);
    strict(&w.dir, &["write", "run", &pkg], &env);
    strict(&w.dir, &["write", "verify", &pkg], &[]);
    strict(&w.dir, &["write", "status", &pkg], &[]);
    let v = strict(&w.dir, &["write", "recover", &pkg], &[]);
    let rec = v["recovery"].as_str().unwrap().to_string();
    strict(&w.dir, &["write", "seal", &rec], &[]);
    // refusals too
    strict(&w.dir, &["write", "run", &rec], &[]);
    strict(&w.dir, &["write", "reconcile", &pkg, "op-1"], &[]);
    strict(&w.dir, &["write", "probe", "nonesuch"], &[]);
}

#[test]
fn verify_of_a_rejected_package_says_so() {
    let w = world("verifyrej", DEMO, Some(DEMO));
    sync(&w);
    standing(&w);
    let (pkg, hash) = planned(&w, "idempotency", &[]);
    cmd(&w.dir, &["write", "approve-demo", &pkg], &[]);
    // the journal's account goes inactive in Xero after planning: the run's precondition refuses it
    w.fake.with(|s| {
        for a in s.accounts.values_mut().flatten() {
            a["Status"] = json!("ARCHIVED");
        }
    });
    let (code, _, _) = cmd(
        &w.dir,
        &["write", "run", &pkg],
        &[("OWNPURSE_WRITE_RUN", &hash)],
    );
    assert_eq!(code, 4);
    let (code, v, text) = cmd(&w.dir, &["write", "verify", &pkg], &[]);
    assert_eq!(code, 4, "{text}");
    assert_eq!(
        (v["error"].as_str(), v["ok"].as_bool()),
        (Some("rejected"), Some(false)),
        "{v}"
    );
    assert_eq!(v["targets"][0]["status"], "rejected");
    assert!(
        v["message"]
            .as_str()
            .unwrap()
            .contains("op-1: accounts not unique and active"),
        "{v}"
    );
    assert!(
        !text.contains("Rejected(") && !text.contains("Object {"),
        "{text}"
    );
    // the record keeps what verify concluded (System › Writes reads it)
    let rec = ownpurse_core::Record::open(w.dir.join("record.sqlite")).unwrap();
    assert_eq!(
        ownpurse_core::write::view::last_verify(&rec, &hash).as_deref(),
        Some("rejected")
    );
}

#[test]
fn probe_prints_its_next_commands_as_a_numbered_list_with_approve_demo() {
    let w = world("probenext", DEMO, Some(DEMO));
    sync(&w);
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(["write", "probe", "idempotency", "--out", "probes"])
        .current_dir(&w.dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("HOME", &w.dir)
        .env("OWNPURSE_CONFIG", w.dir.join("ownpurse.json"))
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(text.contains("next:\n  1. ownpurse write plan "), "{text}");
    assert!(text.contains(". ownpurse write approve-demo "), "{text}");
    assert!(
        !text.contains("approve-record") && !text.contains("[\""),
        "{text}"
    );
    // and a reconciled recode isn't a probe any more (Xero refuses it)
    let (code, v, _) = cmd(&w.dir, &["write", "probe", "recode-reconciled"], &[]);
    assert_eq!(code, 2, "{v}");
}

#[test]
fn open_in_xero_links_status_txns_and_open_never_call_xero() {
    let w = world("links", DEMO, Some(DEMO));
    sync(&w);
    let calls = w.fake.with(|s| s.log.len());
    // `open`: from the record's ShortCode, no request at all
    let (code, v, text) = cmd(&w.dir, &["open", "demo", "BankTransactions", "bt-104"], &[]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(
        v["url"],
        "https://go.xero.com/organisationlogin/default.aspx?shortcode=%21Fk3dQ&redirecturl=%2FBank%2FViewTransaction.aspx%3FbankTransactionID%3Dbt-104"
    );
    assert_eq!(v["launched"], false);
    let (code, v, _) = cmd(&w.dir, &["open", "demo", "Contacts", "c-1"], &[]);
    assert_eq!((code, v["error"].as_str()), (2, Some("usage")), "{v}");
    let (code, v, _) = cmd(&w.dir, &["open", "demo", "BankRec", "acct-1010"], &[]);
    assert_eq!(code, 0);
    assert!(
        v["url"]
            .as_str()
            .unwrap()
            .ends_with("%2FBankRec%2FBankRec.aspx%3FaccountId%3Dacct-1010")
    );
    // txns / show --json carry it
    let (_, v, _) = cmd(&w.dir, &["txns", "demo"], &[]);
    let t = &v["bank_transactions"][0];
    assert!(
        t["url"]
            .as_str()
            .unwrap()
            .contains("bankTransactionID%3Dbt-104"),
        "{v}"
    );
    let (_, v, _) = cmd(&w.dir, &["show", "demo", "BankTransactions", "bt-104"], &[]);
    assert!(v["url"].as_str().unwrap().contains("%21Fk3dQ"), "{v}");
    assert_eq!(
        w.fake.with(|s| s.log.len()),
        calls,
        "no Xero call from links"
    );
    // write status: the target before the run, the created journal after it
    let (pkg, hash) = planned(&w, "void-after-attachment", &[]);
    let (_, st, _) = cmd(&w.dir, &["write", "status", &pkg], &[]);
    assert_eq!(st["ops"][0]["url"], Value::Null, "not created yet: {st}");
    approve(&w.dir, &hash);
    let (code, _, text) = cmd(
        &w.dir,
        &["write", "run", &pkg],
        &[("OWNPURSE_WRITE_RUN", &hash)],
    );
    assert_eq!(code, 0, "{text}");
    let (_, st, _) = cmd(&w.dir, &["write", "status", &pkg], &[]);
    let id = st["ops"][0]["target_id"].as_str().unwrap().to_string();
    assert!(
        st["ops"][0]["url"]
            .as_str()
            .unwrap()
            .ends_with(&format!("%2FJournal%2FView.aspx%3FinvoiceID%3D{id}")),
        "{st}"
    );
}
