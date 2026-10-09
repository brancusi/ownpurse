//! Phase 8, the package side, on a temp copy of the demo record: the op builders A–D (requests, preconditions,
//! expected deltas, recovery), the freeze gate, duplicates, the seal, the review export, and the approvals
//! (TTY record, agent refusal, chat import from a transcript). No network anywhere.

use ownpurse_core::record::Record;
use ownpurse_core::write::approval::{self, TtyApproval};
use ownpurse_core::write::plan::{self, PlanConfig, PlanInput};
use ownpurse_core::write::{
    ApprovalStore, Body, JournalRef, MemApprovals, Op, Package, Precondition, Section, Via, export,
    fingerprint, seal,
};
use rusqlite::{Connection, params};
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

fn tenants() -> Vec<(String, String)> {
    let t: Value =
        serde_json::from_str(&std::fs::read_to_string(demo().join("tenants.json")).unwrap())
            .unwrap();
    let mut v: Vec<(String, String)> = t
        .as_object()
        .unwrap()
        .iter()
        .map(|(a, v)| (a.clone(), v.as_str().unwrap().to_string()))
        .collect();
    v.sort();
    v
}

fn tenant(org: &str) -> String {
    tenants().into_iter().find(|(a, _)| a == org).unwrap().1
}

/// A temp folder with a COPY of the demo record, plus a few documents the tests need (bank transactions with
/// line ids, a locked org). The fixture itself is never opened for writing.
fn world(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("xa-write-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    for ext in ["", "-wal", "-shm"] {
        let f = demo().join(format!("record.sqlite{ext}"));
        if f.exists() {
            std::fs::copy(&f, d.join(format!("record.sqlite{ext}"))).unwrap();
        }
    }
    let c = Connection::open(d.join("record.sqlite")).unwrap();
    let add = |tid: &str, entity: &str, id: &str, v: i64, payload: Value| {
        c.execute(
            "INSERT INTO versions(tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
             VALUES (?1, ?2, ?3, ?4, 'test', ?5, '2026-10-07T00:00:00Z', 'sync', ?6)",
            params![
                tid,
                entity,
                id,
                v,
                payload.get("Status").and_then(Value::as_str),
                payload.to_string()
            ],
        )
        .unwrap();
    };
    add(
        &tenant("lab"),
        "BankTransactions",
        "bt-900",
        1,
        json!({"BankAccount": {"AccountID": "l-card", "Code": "1060"}, "BankTransactionID": "bt-900",
               "Contact": {"ContactID": "c-9", "Name": "Panel Supplies Co"}, "DateString": "2026-02-10T00:00:00",
               "Date": "/Date(1770681600000+0000)/", "IsReconciled": false, "LineAmountTypes": "NoTax",
               "LineItems": [
                 {"LineItemID": "li-1", "AccountCode": "5510", "Description": "tasting cups", "LineAmount": 118.40, "TaxAmount": 0, "Quantity": 1, "UnitAmount": 118.40},
                 {"LineItemID": "li-2", "AccountCode": "5510", "Description": "spoons", "LineAmount": 30.50, "TaxAmount": 0, "Quantity": 1, "UnitAmount": 30.50}],
               "Reference": "INV-77", "Status": "AUTHORISED", "SubTotal": 148.90, "Total": 148.90, "TotalTax": 0,
               "Type": "SPEND", "UpdatedDateUTC": "/Date(1770681600000+0000)/", "CurrencyCode": "USD"}),
    );
    // a reconciled one (Xero refuses to edit those)
    add(
        &tenant("lab"),
        "BankTransactions",
        "bt-901",
        1,
        json!({"BankAccount": {"AccountID": "l-card", "Code": "1060"}, "BankTransactionID": "bt-901",
               "Contact": {"ContactID": "c-9", "Name": "Panel Supplies Co"}, "DateString": "2026-02-11T00:00:00",
               "IsReconciled": true, "LineAmountTypes": "NoTax", "Status": "AUTHORISED", "Type": "SPEND", "Total": 12.70,
               "LineItems": [{"LineItemID": "li-9", "AccountCode": "5510", "Description": "napkins", "LineAmount": 12.70}]}),
    );
    // roast locked through 2025-06-30
    let org = Record::open(d.join("record.sqlite"))
        .unwrap()
        .organisation(&tenant("roast"))
        .unwrap();
    let mut org = org;
    org["PeriodLockDate"] = json!("/Date(1751241600000+0000)/");
    add(&tenant("roast"), "Organisation", "o-roast", 99, org);
    d
}

fn cfg() -> PlanConfig {
    PlanConfig {
        profile: "demo".into(),
        orgs: tenants(),
        gates: vec![("hold".into(), "2025-12-31".into())],
        history_notes: false,
    }
}

fn input(v: Value) -> PlanInput {
    serde_json::from_value(v).unwrap()
}

fn plan_in(dir: &Path, v: Value) -> Result<Package, ownpurse_core::write::WriteError> {
    let rec = Record::open(dir.join("record.sqlite")).unwrap();
    plan::plan(&rec, &cfg(), &input(v), dir, "2026-10-07T12:00:00Z")
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn journal_input() -> Value {
    json!({"id": "fixes-1", "title": "Demo fixes",
           "ops": [{"kind": "create_journal", "id": "op-1", "org": "design", "date": "2026-01-31",
                    "narration": "Reclass management charges", "cash_basis": true,
                    "lines": [{"account": "6420", "amount": "263.94", "description": "charge"},
                              {"account": "2420", "amount": "-263.94"}],
                    "decisions": [{"id": "18", "content": {"what": "reclass"}}]}]})
}

#[test]
fn op_a_builds_the_exact_journal_and_its_deltas() {
    let w = world("a");
    let p = plan_in(&w, journal_input()).unwrap();
    assert_eq!(p.ops.len(), 1);
    let o = &p.ops[0];
    let Op::CreateJournal(j) = &o.op else {
        panic!()
    };
    assert_eq!(j.line_amount_types, "NoTax");
    assert!(j.show_on_cash_basis_reports);
    assert!(j.narration.ends_with("XA:fixes-1:op-1"));
    let Body::Json { json: b } = &o.request.body else {
        panic!()
    };
    let mj = &b["ManualJournals"][0];
    assert_eq!(mj["Status"], "POSTED");
    assert_eq!(mj["ShowOnCashBasisReports"], true);
    assert_eq!(mj["LineAmountTypes"], "NoTax");
    assert_eq!(mj["JournalLines"][0]["LineAmount"].to_string(), "263.94");
    assert_eq!(o.request.path, "ManualJournals");
    assert!(ownpurse_core::write::allowed(
        o.request.method,
        &o.request.path
    ));
    // deltas: Dr 6420 (P&L) 263.94, Cr 2420 (BS) −263.94
    let m: Vec<(String, Section, Decimal)> = o
        .expected
        .iter()
        .map(|e| (e.code.clone(), e.section, e.amount))
        .collect();
    assert_eq!(
        m,
        vec![
            ("2420".into(), Section::Bs, d("-263.94")),
            ("6420".into(), Section::Pl, d("263.94"))
        ]
    );
    assert!(o.preconditions.iter().any(
        |p| matches!(p, Precondition::MarkerAbsent { marker } if marker == "XA:fixes-1:op-1")
    ));
    // recovery: the reversing journal
    let Some(Op::CreateJournal(rev)) = o.recovery.as_ref().and_then(|r| r.op.clone()) else {
        panic!()
    };
    assert_eq!(rev.lines[0].amount, d("-263.94"));
    assert!(rev.narration.contains("XA-REV:op-1"));
    assert_eq!(o.decision_refs[0].id, "18");
    assert!(o.gated.is_none());
}

#[test]
fn op_a_refusals_and_split() {
    let w = world("a2");
    let bad = |lines: Value| {
        let mut v = journal_input();
        v["ops"][0]["lines"] = lines;
        plan_in(&w, v).unwrap_err()
    };
    assert!(
        bad(json!([{"account": "6420", "amount": "1"}, {"account": "2420", "amount": "-2"}]))
            .message
            .contains("balance")
    );
    assert!(
        bad(json!([{"account": "9999", "amount": "1"}, {"account": "2420", "amount": "-1"}]))
            .message
            .contains("no account")
    );
    assert!(
        bad(json!([{"account": "1010", "amount": "1"}, {"account": "2420", "amount": "-1"}]))
            .message
            .contains("bank account")
    );
    assert!(
        bad(
            json!([{"account": "6420", "amount": "1.001"}, {"account": "2420", "amount": "-1.001"}])
        )
        .message
        .contains("2 decimals")
    );
    // cash_basis is required
    let mut v = journal_input();
    v["ops"][0].as_object_mut().unwrap().remove("cash_basis");
    assert!(serde_json::from_value::<PlanInput>(v).is_err());
    // a lock date refuses
    let mut v = journal_input();
    v["ops"][0]["org"] = json!("roast");
    v["ops"][0]["date"] = json!("2025-03-31");
    v["ops"][0]["lines"] =
        json!([{"account": "6290", "amount": "5"}, {"account": "2430", "amount": "-5"}]);
    let e = plan_in(&w, v).unwrap_err();
    assert_eq!(e.code, "precondition", "{e}");
    // split at a 3-line limit through 2420
    let mut v = journal_input();
    v["ops"][0]["lines"] = json!([
        {"account": "6420", "amount": "10"}, {"account": "6420", "amount": "20"},
        {"account": "6420", "amount": "30"}, {"account": "2420", "amount": "-60"}]);
    v["ops"][0]["max_lines"] = json!(3);
    assert!(
        plan_in(&w, v.clone())
            .unwrap_err()
            .message
            .contains("split_account")
    );
    v["ops"][0]["split_account"] = json!("2420");
    let p = plan_in(&w, v).unwrap();
    assert_eq!(p.ops.len(), 2);
    assert_eq!(p.ops[0].id, "op-1.1");
    let mut net = std::collections::BTreeMap::<String, Decimal>::new();
    for o in &p.ops {
        let Op::CreateJournal(j) = &o.op else {
            panic!()
        };
        assert!(j.lines.len() <= 3);
        assert!(j.lines.iter().map(|l| l.amount).sum::<Decimal>().is_zero());
        for e in &o.expected {
            *net.entry(e.code.clone()).or_default() += e.amount;
        }
        assert!(o.flags.iter().any(|f| f.starts_with("split ")));
    }
    assert_eq!(net["6420"], d("60"));
    assert_eq!(net["2420"], d("-60"));
}

#[test]
fn a_create_that_duplicates_an_existing_journal_is_refused() {
    let w = world("dup");
    let v = json!({"id": "dup-1", "ops": [{"kind": "create_journal", "id": "op-1", "org": "design",
        "date": "2025-04-30", "narration": "again", "cash_basis": true,
        "lines": [{"account": "6420", "amount": "2764.18"}, {"account": "2420", "amount": "-2764.18"}]}]});
    let e = plan_in(&w, v).unwrap_err();
    assert_eq!(e.code, "precondition");
    assert!(e.message.contains("mj-302"), "{e}");
}

#[test]
fn op_b_recodes_one_line_and_echoes_everything_else() {
    let w = world("b");
    let v = json!({"id": "rc-1", "ops": [{"kind": "recode_bank_line", "id": "op-1", "org": "lab",
        "bank_transaction": "bt-900", "line_item": "li-2", "account": "2440", "description": "spoons (advance)"}]});
    let p = plan_in(&w, v).unwrap();
    let o = &p.ops[0];
    let Op::RecodeBankLine(r) = &o.op else {
        panic!()
    };
    assert_eq!(
        (r.from_account_code.as_str(), r.to_account_code.as_deref()),
        ("5510", Some("2440"))
    );
    assert_eq!(o.request.path, "BankTransactions/bt-900");
    let Body::Json { json: b } = &o.request.body else {
        panic!()
    };
    let sent = &b["BankTransactions"][0];
    // only line li-2's AccountCode/Description differ from the record's document
    let rec = Record::open(w.join("record.sqlite")).unwrap();
    let before = rec
        .current_rows(&tenant("lab"), "BankTransactions")
        .unwrap()
        .into_iter()
        .find(|r| r.xero_id == "bt-900")
        .unwrap()
        .payload;
    let mut expect = before.clone();
    expect["LineItems"][1]["AccountCode"] = json!("2440");
    expect["LineItems"][1]["Description"] = json!("spoons (advance)");
    assert_eq!(sent, &expect);
    assert_eq!(sent["LineItems"][0], before["LineItems"][0]);
    // deltas: 5510 −30.50, 2440 +30.50
    let m: Vec<(String, Decimal)> = o
        .expected
        .iter()
        .map(|e| (e.code.clone(), e.amount))
        .collect();
    assert_eq!(
        m,
        vec![("2440".into(), d("30.50")), ("5510".into(), d("-30.50"))]
    );
    assert_eq!(
        o.expected_after.as_deref(),
        Some(fingerprint(&expect).as_str())
    );
    // a reconciled transaction is refused at plan (Xero refuses the edit), with the alternatives
    let v = json!({"id": "rc-3", "ops": [{"kind": "recode_bank_line", "id": "op-1", "org": "lab",
        "bank_transaction": "bt-901", "line_item": "li-9", "account": "2440"}]});
    let e = plan_in(&w, v).unwrap_err();
    assert_eq!(e.code, "reconciled_line", "{e}");
    assert!(
        e.help.iter().any(|h| h.contains("reclass journal")),
        "{e:?}"
    );
    // ... and the transaction's page in Xero, through the org switch (the browser path)
    assert!(
        e.help.iter().any(|h| h
            .contains("https://go.xero.com/organisationlogin/default.aspx?shortcode=")
            && h.contains("bankTransactionID%3Dbt-901")),
        "{e:?}"
    );
    assert!(
        matches!(&o.preconditions[0], Precondition::Target { id, version: Some(1), .. } if id == "bt-900")
    );
    let Some(Op::RecodeBankLine(inv)) = o.recovery.as_ref().and_then(|r| r.op.clone()) else {
        panic!()
    };
    assert_eq!(inv.to_account_code.as_deref(), Some("5510"));
    assert_eq!(inv.to_description.as_deref(), Some("spoons"));
    // a line without a LineItemID in the record is refused, with the re-sync help
    let v = json!({"id": "rc-2", "ops": [{"kind": "recode_bank_line", "id": "op-1", "org": "lab",
        "bank_transaction": "bt-401", "line_item": "x", "account": "2440"}]});
    let e = plan_in(&w, v).unwrap_err();
    assert!(e.message.contains("never chosen by position"), "{e}");
    assert!(e.help.iter().any(|h| h.contains("ownpurse sync lab")));
}

#[test]
fn op_c_voids_and_recovers_by_recreating() {
    let w = world("c");
    let v = json!({"id": "v-1", "ops": [{"kind": "void_journal", "id": "op-1", "org": "lab", "journal": "mj-401"}]});
    let p = plan_in(&w, v).unwrap();
    let o = &p.ops[0];
    assert!(matches!(&o.op, Op::VoidJournal { journal: JournalRef::Id(id) } if id == "mj-401"));
    let Body::Json { json: b } = &o.request.body else {
        panic!()
    };
    assert_eq!(
        b,
        &json!({"ManualJournals": [{"ManualJournalID": "mj-401", "Status": "VOIDED"}]})
    );
    let m: Vec<(String, Decimal)> = o
        .expected
        .iter()
        .map(|e| (e.code.clone(), e.amount))
        .collect();
    assert_eq!(
        m,
        vec![("2440".into(), d("-57.13")), ("4950".into(), d("57.13"))]
    );
    let Some(Op::CreateJournal(redo)) = o.recovery.as_ref().and_then(|r| r.op.clone()) else {
        panic!()
    };
    assert_eq!(redo.lines.len(), 2);
    assert_eq!(redo.date, "2025-11-28");
}

#[test]
fn op_d_attaches_by_content_hash() {
    let w = world("d");
    std::fs::write(w.join("support.csv"), "date,amount\n2026-01-31,263.94\n").unwrap();
    std::fs::write(w.join("notes.md"), "# no").unwrap();
    let mut v = journal_input();
    v["ops"].as_array_mut().unwrap().push(json!({"kind": "attach", "id": "op-2", "org": "design", "journal": "op:op-1", "file": "support.csv"}));
    let p = plan_in(&w, v.clone()).unwrap();
    let o = &p.ops[1];
    let Op::Attach(a) = &o.op else { panic!() };
    assert_eq!(a.journal, JournalRef::CreatedBy("op-1".into()));
    assert_eq!(a.mime, "text/csv");
    assert_eq!(
        o.request.path,
        "ManualJournals/{id}/Attachments/support.csv"
    );
    assert!(matches!(&o.request.body, Body::File { sha256, .. } if *sha256 == a.sha256));
    assert_eq!(o.depends_on, vec!["op-1".to_string()]);
    assert!(o.expected.is_empty());
    assert!(
        o.preconditions
            .iter()
            .any(|p| matches!(p, Precondition::AttachmentAbsent { .. }))
    );
    assert!(o.recovery.as_ref().unwrap().op.is_none());
    // Markdown is refused; an attach before its journal is refused
    v["ops"][1]["file"] = json!("notes.md");
    assert!(
        plan_in(&w, v.clone())
            .unwrap_err()
            .message
            .contains("only PDF, CSV or TXT")
    );
    v["ops"][1]["file"] = json!("support.csv");
    v["ops"][1]["journal"] = json!("op:op-9");
    assert!(plan_in(&w, v).is_err());
}

#[test]
fn the_freeze_gate_needs_the_accountant_reference() {
    let w = world("gate");
    let v = json!({"id": "g-1", "ops": [{"kind": "create_journal", "id": "op-1", "org": "hold", "date": "2025-12-31",
        "narration": "year-end fee accrual", "cash_basis": false,
        "lines": [{"account": "1470", "amount": "100"}, {"account": "4610", "amount": "-100"}]}]});
    let e = plan_in(&w, v.clone()).unwrap_err();
    assert_eq!(e.code, "freeze_gate");
    assert!(
        e.help
            .iter()
            .any(|h| h.starts_with("op-1: hold is frozen through 2025-12-31"))
    );
    let mut ok = v.clone();
    ok["accountant_decision"] = json!("TBD");
    assert_eq!(plan_in(&w, ok.clone()).unwrap_err().code, "freeze_gate");
    ok["accountant_decision"] = json!("AJE-2 (the accountant, 2026-10-01)");
    let p = plan_in(&w, ok).unwrap();
    assert!(p.ops[0].gated.is_some());
    assert!(
        p.checklist
            .iter()
            .any(|c| c.contains("set the 2025 lock date for hold"))
    );
    // prior period: any org, before the FY of the latest trial balance (2025)
    let v = json!({"id": "g-2", "ops": [{"kind": "void_journal", "id": "op-1", "org": "design", "journal": "mj-301"}]});
    let e = plan_in(&w, v).unwrap_err();
    assert_eq!(e.code, "freeze_gate");
    assert!(e.help.iter().any(|h| h.contains("prior period")));
    // an ungated org-year passes without a reference
    assert!(plan_in(&w, journal_input()).unwrap().ops[0].gated.is_none());
}

fn sealed(w: &Path) -> Package {
    seal::seal(plan_in(w, journal_input()).unwrap()).unwrap()
}

#[test]
fn the_seal_binds_every_byte_and_the_export_carries_it() {
    let w = world("seal");
    let p = sealed(&w);
    let h = seal::verify(&p).unwrap();
    assert_eq!(p.seal.as_deref(), Some(h.as_str()));
    assert_eq!(
        seal::hash(&p).unwrap(),
        h,
        "the hash ignores the seal field"
    );
    // a round trip through the file keeps the hash
    seal::save(&w.join("p.json"), &p).unwrap();
    assert_eq!(
        seal::verify(&seal::load(&w.join("p.json")).unwrap()).unwrap(),
        h
    );
    // any edit → hash_mismatch; unsealed → refused
    let mut q = p.clone();
    if let Op::CreateJournal(j) = &mut q.ops[0].op {
        j.lines[0].amount = d("263.95");
    }
    assert_eq!(seal::verify(&q).unwrap_err().code, "hash_mismatch");
    let mut u = p.clone();
    u.seal = None;
    assert_eq!(seal::verify(&u).unwrap_err().code, "not_sealed");
    assert!(export::csv(&q).is_err() && export::pdf(&u).is_err());
    // CSV and PDF carry the hash; deterministic
    let csv = export::csv(&p).unwrap();
    assert!(csv.lines().skip(1).all(|l| l.starts_with(&h)));
    assert!(csv.contains(",line,op-1,create_journal,design,2026-01-31,6420,,263.94,,charge,"));
    assert!(csv.contains(",expected,op-1,create_journal,design,2026-01-31,2420,Fees owed to Brackenridge,,263.94,balance sheet,"));
    let pdf = export::pdf(&p).unwrap();
    assert_eq!(pdf, export::pdf(&p).unwrap());
    let text = String::from_utf8_lossy(&pdf);
    assert!(text.starts_with("%PDF-1.4") && text.contains(&h));
}

#[test]
fn the_review_and_the_export_link_each_existing_target_to_its_xero_page() {
    let w = world("links");
    let v = json!({"id": "rc-links", "ops": [{"kind": "recode_bank_line", "id": "op-1", "org": "lab",
        "bank_transaction": "bt-900", "line_item": "li-2", "account": "2440"}]});
    let p = seal::seal(plan_in(&w, v).unwrap()).unwrap();
    let rec = Record::open(w.join("record.sqlite")).unwrap();
    let links = ownpurse_core::write::links(&rec, &p);
    let sc =
        ownpurse_core::deeplink::short_code(&rec, &tenant("lab")).expect("demo org short code");
    let url = ownpurse_core::deeplink::url_for(&sc, "BankTransactions", "", "bt-900").unwrap();
    assert!(url.starts_with("https://go.xero.com/organisationlogin/default.aspx?shortcode="));
    // CSV: a trailing xero_url column, filled on the op row only
    let csv = export::csv_with(&p, &links).unwrap();
    let head = csv.lines().next().unwrap();
    assert!(head.ends_with(",note,xero_url"), "{head}");
    let op_row = csv.lines().find(|l| l.contains(",op,op-1,")).unwrap();
    assert!(op_row.ends_with(&format!(",{url}")), "{op_row}");
    assert!(csv.lines().filter(|l| l.contains("go.xero.com")).count() == 1);
    // the review lines (PDF) say it under the operation
    let lines = export::lines_with(&p, &links).unwrap();
    assert!(
        lines.iter().any(|l| l == &format!("  open in Xero: {url}")),
        "{lines:?}"
    );
    // the sealed bytes never carry a link: no links in, none out, same hash
    assert!(!export::csv(&p).unwrap().contains("go.xero.com"));
    assert_eq!(seal::verify(&p).unwrap(), p.seal.clone().unwrap());
    // a journal another op creates has no page yet; once created, status links it
    let mut j = p.ops[0].clone();
    j.op = Op::VoidJournal {
        journal: JournalRef::CreatedBy("op-0".into()),
    };
    assert!(ownpurse_core::write::op_url(&j, &links, None).is_none());
    assert!(
        ownpurse_core::write::op_url(&j, &links, Some("mj-1"))
            .unwrap()
            .contains("%2FJournal%2FView.aspx%3FinvoiceID%3Dmj-1")
    );
    // no short code: no link at all (never a guessed one)
    assert!(ownpurse_core::write::op_url(&p.ops[0], &Default::default(), None).is_none());
}

fn tty<'a>(p: &'a Package, verbatim: &'a str, typed: &'a str) -> TtyApproval<'a> {
    TtyApproval {
        package: p,
        verbatim,
        typed,
        i_am_owner: true,
        tty: true,
        env_names: vec!["HOME".into(), "PATH".into()],
        scope: vec![],
        now: "2026-10-07T12:30:00Z".into(),
    }
}

#[test]
fn approve_record_refuses_agents_and_binds_the_hash() {
    let w = world("ap");
    let p = sealed(&w);
    let h = seal::verify(&p).unwrap();
    let words = format!("Approved: package {} as reviewed.", &h[..12]);
    // agents and non-TTYs are refused
    let mut a = tty(&p, &words, &h[..12]);
    a.env_names.push("CLAUDECODE".into());
    assert_eq!(approval::record_tty(&a).unwrap_err().code, "agent_session");
    let mut a = tty(&p, &words, &h[..12]);
    a.tty = false;
    assert_eq!(approval::record_tty(&a).unwrap_err().code, "agent_session");
    // the response must quote the hash and approve; the typed confirmation must match
    assert_eq!(
        approval::record_tty(&tty(&p, "Approved.", &h[..12]))
            .unwrap_err()
            .code,
        "not_approved"
    );
    let no = format!("I do not approve {}", &h[..12]);
    assert_eq!(
        approval::record_tty(&tty(&p, &no, &h[..12]))
            .unwrap_err()
            .code,
        "not_approved"
    );
    assert_eq!(
        approval::record_tty(&tty(&p, &words, "0000000000000"))
            .unwrap_err()
            .code,
        "hash_mismatch"
    );
    let r = approval::record_tty(&tty(&p, &words, &h[..16])).unwrap();
    assert_eq!((r.via, r.hash.as_str()), (Via::Tty, h.as_str()));
    let mut store = MemApprovals::default();
    assert_eq!(
        approval::check(&store, &p).unwrap_err().code,
        "not_approved"
    );
    store.record(&r).unwrap();
    let found = approval::check(&store, &p).unwrap();
    assert!(approval::covers(&found, "op-1"));
    // an approval never carries over to an edited package
    let mut q = p.clone();
    q.title = "Demo fixes, edited".into();
    let q = seal::seal(q).unwrap();
    assert_eq!(
        approval::check(&store, &q).unwrap_err().code,
        "not_approved"
    );
    // a scope must name ops of the package
    let mut a = tty(&p, &words, &h[..12]);
    a.scope = vec!["op-9".into()];
    assert!(approval::record_tty(&a).is_err());
}

fn line(v: Value) -> String {
    v.to_string()
}

#[test]
fn chat_import_accepts_only_the_owners_typed_message() {
    let w = world("chat");
    let p = sealed(&w);
    let h = seal::verify(&p).unwrap();
    let session = "demo-session-1";
    let user = |uuid: &str, text: &str, extra: Value| {
        let mut e = json!({"type": "user", "uuid": uuid, "sessionId": session, "userType": "external",
            "isSidechain": false, "timestamp": "2026-10-07T12:40:00Z", "origin": {"kind": "human"},
            "message": {"role": "user", "content": text}});
        for (k, v) in extra.as_object().unwrap() {
            e[k] = v.clone();
        }
        line(e)
    };
    let dir = w.join("projects");
    std::fs::create_dir_all(dir.join("subagents")).unwrap();
    let file = dir.join(format!("{session}.jsonl"));
    let write = |lines: &[String]| std::fs::write(&file, lines.join("\n")).unwrap();
    let approve = format!("approved {h}");
    // relayed / injected / agent-side messages never count
    write(&[
        user(
            "u1",
            &approve,
            json!({"origin": {"kind": "task-notification"}}),
        ),
        user("u2", &approve, json!({"isSidechain": true})),
        user("u3", &approve, json!({"isMeta": true})),
        user("u4", &approve, json!({"sessionId": "another"})),
        user("u5", &approve, json!({"origin": null})),
        line(
            json!({"type": "user", "uuid": "u6", "sessionId": session, "userType": "external", "origin": {"kind": "human"},
            "message": {"role": "user", "content": [{"type": "tool_result", "content": approve}]}}),
        ),
        line(
            json!({"type": "assistant", "uuid": "u7", "sessionId": session, "message": {"role": "assistant", "content": approve}}),
        ),
        user("u8", &format!("approved {}", &h[..12]), json!({})),
    ]);
    let e = approval::record_chat(&p, &file, session, &[], "now").unwrap_err();
    assert_eq!(e.code, "not_approved");
    assert!(e.help.len() >= 5, "{:?}", e.help);
    // the file must be the session's own, never a subagent transcript
    assert!(approval::record_chat(&p, &dir.join("other.jsonl"), session, &[], "now").is_err());
    let sub = dir.join("subagents").join(format!("{session}.jsonl"));
    std::fs::write(&sub, user("u9", &approve, json!({}))).unwrap();
    assert_eq!(
        approval::record_chat(&p, &sub, session, &[], "now")
            .unwrap_err()
            .code,
        "usage"
    );
    // a typed message that negates is not an approval
    write(&[user("u10", &format!("do not approve {h} yet"), json!({}))]);
    assert_eq!(
        approval::record_chat(&p, &file, session, &[], "now")
            .unwrap_err()
            .code,
        "not_approved"
    );
    // the owner's typed message quoting the full hash is
    write(&[
        user(
            "u1",
            &approve,
            json!({"origin": {"kind": "task-notification"}}),
        ),
        user("u11", &format!("Yes, approved: {h}"), json!({})),
    ]);
    let r = approval::record_chat(&p, &file, session, &[], "2026-10-07T12:41:00Z").unwrap();
    assert_eq!(r.via, Via::Chat);
    assert!(r.where_.ends_with("&message=u11"));
    assert_eq!(r.at, "2026-10-07T12:40:00Z");
    assert!(r.verbatim.contains(&h));
}

#[test]
fn agent_markers_are_names_only() {
    let m = approval::agent_markers(["HOME", "CLAUDE_CODE_ENTRYPOINT", "OWNPURSE_AGENT", "PATH"]);
    assert_eq!(m, vec!["CLAUDE_CODE_ENTRYPOINT", "OWNPURSE_AGENT"]);
}

#[test]
fn one_package_creates_attaches_then_voids_the_same_journal() {
    let w = world("probe");
    std::fs::write(w.join("support.csv"), "date,amount\n2026-01-31,263.94\n").unwrap();
    let mut v = journal_input();
    let ops = v["ops"].as_array_mut().unwrap();
    ops.push(json!({"kind": "attach", "id": "op-2", "org": "design", "journal": "op:op-1", "file": "support.csv"}));
    ops.push(json!({"kind": "void_journal", "id": "op-3", "org": "design", "journal": "op:op-1"}));
    let p = plan_in(&w, v.clone()).unwrap();
    let o = &p.ops[2];
    assert!(matches!(&o.op, Op::VoidJournal { journal: JournalRef::CreatedBy(c) } if c == "op-1"));
    assert_eq!(o.request.path, "ManualJournals/{id}");
    assert_eq!(o.depends_on, vec!["op-1".to_string()]);
    let net: Decimal = p.ops[0]
        .expected
        .iter()
        .chain(&o.expected)
        .map(|e| e.amount.abs())
        .sum::<Decimal>()
        - p.ops[0]
            .expected
            .iter()
            .map(|e| e.amount.abs())
            .sum::<Decimal>()
            * Decimal::TWO;
    assert!(net.is_zero());
    for (a, b) in p.ops[0].expected.iter().zip(&o.expected) {
        assert_eq!((a.code.as_str(), a.amount), (b.code.as_str(), -b.amount));
    }
    // attach after the void, or a void of an unknown op, is refused
    let mut bad = v.clone();
    bad["ops"].as_array_mut().unwrap().swap(1, 2);
    assert!(
        plan_in(&w, bad)
            .unwrap_err()
            .message
            .contains("attach before the void")
    );
    let mut bad = v;
    bad["ops"][2]["journal"] = json!("op:op-7");
    assert!(plan_in(&w, bad).is_err());
}

#[test]
fn a_write_runs_evidence_and_a_sentinel_date_never_set_the_prior_period() {
    // exactly the row an older build left in the demo record: a write run's before-snapshot at 9999-12-31
    let w = world("sentinel");
    let lab = tenant("lab");
    let before = ownpurse_core::write::plan::prior_before(
        &Record::open(w.join("record.sqlite")).unwrap(),
        &lab,
    )
    .unwrap();
    let c = Connection::open(w.join("record.sqlite")).unwrap();
    c.execute(
        "INSERT INTO reports(tenant_id, report, params, fetched_at, content_hash, payload) VALUES (?1, 'TrialBalance', ?2, '2026-10-07T12:00:00Z', 'x', '{}')",
        params![lab, r#"{"date":"9999-12-31","phase":"before","xa_package":"dabc7522"}"#],
    )
    .unwrap();
    // and a sync-cached one with an implausible date
    c.execute(
        "INSERT INTO reports(tenant_id, report, params, fetched_at, content_hash, payload) VALUES (?1, 'TrialBalance', ?2, '2026-10-07T12:00:00Z', 'x', '{}')",
        params![lab, r#"{"date":"9999-12-31"}"#],
    )
    .unwrap();
    drop(c);
    let after = ownpurse_core::write::plan::prior_before(
        &Record::open(w.join("record.sqlite")).unwrap(),
        &lab,
    )
    .unwrap();
    assert_eq!(after, before, "the bad rows are harmless");
    assert!(after.as_deref().is_none_or(|d| d < "9999"), "{after:?}");
}
