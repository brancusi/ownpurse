//! `write run / reconcile / verify / status` end to end against the loopback fake Xero, through the real
//! HTTP stack: the locks, live preconditions, the attempt log written before each send, read-back, the stop
//! rules, uncertain attempts settled by read-back (never re-sent blindly), batches, and the report deltas.

mod common;

use common::temp_dir;
use ownpurse_core::write::*;
use ownpurse_xero::TokenStore;
use ownpurse_xero::store::Writer;
use ownpurse_xero::write::fake::{FakeXero, Fault, State};
use ownpurse_xero::write::run::{self, Ctx, seal_of};
use ownpurse_xero::write::{RecordApprovals, UreqWire, WriteSettings, XeroWriter};
use serde_json::{Value, json};

const T: &str = "c0ffee00-0000-4000-8000-0000000000a1";
const NAME: &str = "Brindlewick Demo Co (US)";
const SCOPES: &str = "accounting.manualjournals accounting.banktransactions accounting.attachments";

fn num(s: &str) -> Value {
    Value::Number(serde_json::Number::from_string_unchecked(s.to_string()))
}

struct W {
    dir: std::path::PathBuf,
    fake: FakeXero,
    store: Writer,
    tokens: TokenStore,
}

fn world(name: &str, scopes: &str) -> W {
    let dir = temp_dir(name);
    std::fs::write(
        dir.join("tokens.json"),
        json!({"access_token": "AT", "refresh_token": "RT", "obtained_at": 4.0e9, "expires_in": 1800, "scope": scopes})
            .to_string(),
    )
    .unwrap();
    let mut s = State::default();
    s.add_tenant(
        T,
        NAME,
        &[
            ("1010", "Operating", "ASSET"),
            ("6310", "Cupping supplies", "EXPENSE"),
            ("6520", "Van fuel", "EXPENSE"),
            ("2700", "Owner loan", "LIABILITY"),
        ],
    );
    s.add_bank_spend(T, "bt-104", "1010", "6520", "238.61", true);
    W {
        fake: FakeXero::start(s).unwrap(),
        store: Writer::open(dir.join("record.sqlite")).unwrap(),
        tokens: TokenStore::new(dir.join("tokens.json"), "CLIENT"),
        dir,
    }
}

fn settings(w: &W) -> WriteSettings {
    WriteSettings {
        profile: "demo".into(),
        api_base: w.fake.api_base(),
        connections_url: w.fake.connections_url(),
        orgs: vec![("demo".into(), T.into())],
        forbid_tenants: vec![],
        require_tenant_name: Some(NAME.into()),
        allow_writes: true,
        loopback_only: true,
    }
}

fn delta(w: &W, code: &str, amount: &str) -> ExpectedDelta {
    ExpectedDelta {
        account_id: w.fake.with(|s| s.account_id(T, code)).unwrap(),
        code: code.into(),
        name: code.into(),
        section: if code.starts_with('6') {
            Section::Pl
        } else {
            Section::Bs
        },
        amount: amount.parse().unwrap(),
    }
}

fn journal_op(w: &W, pkg: &str, id: &str, amount: &str) -> Operation {
    let m = marker(pkg, id);
    let narration = format!("Owner-paid cupping supplies · {m}");
    let neg = format!("-{amount}");
    Operation {
        id: id.into(),
        org: "demo".into(),
        tenant_id: T.into(),
        op: Op::CreateJournal(Journal {
            date: "2025-03-31".into(),
            narration: narration.clone(),
            lines: vec![
                JournalLine {
                    account_code: "6310".into(),
                    description: "tins".into(),
                    amount: amount.parse().unwrap(),
                },
                JournalLine {
                    account_code: "2700".into(),
                    description: "tins".into(),
                    amount: neg.parse().unwrap(),
                },
            ],
            line_amount_types: "NoTax".into(),
            show_on_cash_basis_reports: true,
        }),
        request: WriteRequest {
            method: Method::Put,
            path: "ManualJournals".into(),
            body: Body::Json {
                json: json!({"ManualJournals": [{"Narration": narration, "Date": "2025-03-31", "Status": "POSTED",
                    "LineAmountTypes": "NoTax", "ShowOnCashBasisReports": true,
                    "JournalLines": [{"AccountCode": "6310", "Description": "tins", "LineAmount": num(amount)},
                                     {"AccountCode": "2700", "Description": "tins", "LineAmount": num(&neg)}]}]}),
            },
        },
        preconditions: vec![
            Precondition::MarkerAbsent { marker: m },
            Precondition::AccountsActive {
                codes: vec!["6310".into(), "2700".into()],
            },
            Precondition::PeriodOpen {
                date: "2025-03-31".into(),
            },
        ],
        expected_after: None,
        expected: vec![delta(w, "6310", amount), delta(w, "2700", &neg)],
        recovery: Some(Box::new(Recovery {
            note: "a reversing journal".into(),
            op: Some(Op::CreateJournal(Journal {
                date: "2025-03-31".into(),
                narration: format!("Reversal · {}", reversal_marker(id)),
                lines: vec![
                    JournalLine {
                        account_code: "6310".into(),
                        description: "tins".into(),
                        amount: neg.parse().unwrap(),
                    },
                    JournalLine {
                        account_code: "2700".into(),
                        description: "tins".into(),
                        amount: amount.parse().unwrap(),
                    },
                ],
                line_amount_types: "NoTax".into(),
                show_on_cash_basis_reports: true,
            })),
        })),
        depends_on: vec![],
        decision_refs: vec![],
        source_refs: vec![],
        gated: None,
        flags: vec![],
        history_note: None,
    }
}

fn attach_op(w: &W, after: &str) -> Operation {
    let bytes = b"date,payee,amount\n2025-03-04,Quillfeather Supplies,147.38\n";
    std::fs::write(w.dir.join("support.csv"), bytes).unwrap();
    let sha = sha256_hex(bytes);
    Operation {
        id: "op-02".into(),
        org: "demo".into(),
        tenant_id: T.into(),
        op: Op::Attach(Attachment {
            entity: "ManualJournals".into(),
            journal: JournalRef::CreatedBy(after.into()),
            file_name: "support.csv".into(),
            mime: "text/csv".into(),
            sha256: sha.clone(),
            bytes: bytes.len() as u64,
            path: "support.csv".into(),
        }),
        request: WriteRequest {
            method: Method::Put,
            path: "ManualJournals/{id}/Attachments/support.csv".into(),
            body: Body::File {
                sha256: sha,
                mime: "text/csv".into(),
            },
        },
        preconditions: vec![],
        expected_after: None,
        expected: vec![],
        recovery: None,
        depends_on: vec![after.into()],
        decision_refs: vec![],
        source_refs: vec![],
        gated: None,
        flags: vec![],
        history_note: None,
    }
}

fn recode_op(w: &W) -> Operation {
    let before = w.fake.with(|s| s.bank[T]["bt-104"].clone());
    let mut after = before.clone();
    after["LineItems"][0]["AccountCode"] = json!("6310");
    let mut body = after.clone();
    body.as_object_mut().unwrap().remove("UpdatedDateUTC");
    Operation {
        id: "op-03".into(),
        org: "demo".into(),
        tenant_id: T.into(),
        op: Op::RecodeBankLine(Recode {
            bank_transaction_id: "bt-104".into(),
            line_item_id: "bt-104-L1".into(),
            from_account_code: "6520".into(),
            from_description: "Sample tins".into(),
            to_account_code: Some("6310".into()),
            to_description: None,
        }),
        request: WriteRequest {
            method: Method::Post,
            path: "BankTransactions/bt-104".into(),
            body: Body::Json {
                json: json!({"BankTransactions": [body]}),
            },
        },
        preconditions: vec![Precondition::Target {
            entity: "BankTransactions".into(),
            id: "bt-104".into(),
            fingerprint: fingerprint(&before),
            version: None,
            status: "AUTHORISED".into(),
            before: Some(business_view(&before)),
        }],
        expected_after: Some(fingerprint(&after)),
        expected: vec![delta(w, "6310", "238.61"), delta(w, "6520", "-238.61")],
        recovery: Some(Box::new(Recovery {
            note: "the inverse recode".into(),
            op: Some(Op::RecodeBankLine(Recode {
                bank_transaction_id: "bt-104".into(),
                line_item_id: "bt-104-L1".into(),
                from_account_code: "6310".into(),
                from_description: "Sample tins".into(),
                to_account_code: Some("6520".into()),
                to_description: None,
            })),
        })),
        depends_on: vec![],
        decision_refs: vec![],
        source_refs: vec![],
        gated: None,
        flags: vec![],
        history_note: None,
    }
}

fn package(id: &str, ops: Vec<Operation>) -> Package {
    let mut p = Package {
        v: PACKAGE_V,
        id: id.into(),
        title: "Demo".into(),
        profile: "demo".into(),
        created_at: "2026-10-07T12:00:00Z".into(),
        accountant_decision: None,
        ops,
        excluded: vec![],
        checklist: vec![],
        supersedes: vec![],
        source: None,
        seal: None,
    };
    p.seal = Some(seal_of(&p));
    p
}

fn approve(w: &W, p: &Package) {
    let mut a = RecordApprovals { store: &w.store };
    a.record(&ApprovalRecord {
        hash: p.seal.clone().unwrap(),
        verbatim: format!("approved {}", &p.seal.as_ref().unwrap()[..12]),
        via: Via::Tty,
        where_: "tty".into(),
        at: "2026-10-07T12:01:00Z".into(),
        scope: ApprovalScope::All(AllOps::All),
        recorded_at: "2026-10-07T12:01:01Z".into(),
    })
    .unwrap();
}

/// Run `f` with a context on the world (the env lock set to the package hash unless `env` says otherwise).
fn with_ctx<T>(w: &W, p: &Package, env: Option<String>, f: impl FnOnce(&mut Ctx<'_>) -> T) -> T {
    let mut x = XeroWriter::new(
        settings(w),
        &w.store,
        &w.tokens,
        Box::new(UreqWire::default()),
    )
    .with_sleep(Box::new(|_| {}));
    let approvals = RecordApprovals { store: &w.store };
    let mut ctx = Ctx {
        transport: &mut x,
        store: &w.store,
        approvals: &approvals,
        profile: "demo".into(),
        allow_writes: true,
        run_env: env.or_else(|| p.seal.clone()),
        base_dir: w.dir.clone(),
        batch: BATCH_MAX,
        dry_run: false,
        require_tenant_name: Some(NAME.into()),
    };
    f(&mut ctx)
}

fn writes(w: &W, prefix: &str) -> usize {
    w.fake.with(|s| {
        s.log
            .iter()
            .filter(|l| l.0 != "GET" && l.1.contains(prefix))
            .count()
    })
}

#[test]
fn plan_seal_approve_run_verify_end_to_end() {
    let w = world("e2e", SCOPES);
    let p = package(
        "demo-01",
        vec![
            journal_op(&w, "demo-01", "op-01", "147.38"),
            attach_op(&w, "op-01"),
            recode_op(&w),
        ],
    );
    approve(&w, &p);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert!(rep.stopped.is_none(), "{:?}", rep.stopped);
    let outcomes: Vec<&str> = rep.ops.iter().map(|o| o.1.as_str()).collect();
    assert_eq!(outcomes, ["confirmed", "confirmed", "confirmed"]);
    // the log has sending BEFORE each outcome
    let st = run::status(&p, &w.store).unwrap();
    for op in st["ops"].as_array().unwrap() {
        let states: Vec<&str> = op["attempts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|a| a["state"].as_str().unwrap())
            .collect();
        assert_eq!(states, ["sending", "confirmed"], "{op}");
    }
    assert!(st["chain"]["entries"].as_i64().unwrap() > 0);
    // verify: targets read back, the trial balance moved exactly by the expected deltas
    let v = with_ctx(&w, &p, None, |c| run::verify(&p, c)).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    assert!(
        v["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x.as_str().unwrap().contains("no lock date"))
    );
    // the journal and the recoded transaction are in the record (read back by the run): recorded, no new
    // version now; an attachment adds no document version of its own
    for t in v["targets"].as_array().unwrap() {
        if t["op"] != "op-02" {
            assert_eq!(
                (t["recorded"].as_bool(), t["new_version"].as_bool()),
                (Some(true), Some(false)),
                "{t}"
            );
        }
    }
    // what Xero holds now is in the record (read-back versions, caused_by_op set): the journal and the
    // recoded transaction, chained; verify's re-read of unchanged documents adds nothing
    let rows: Vec<(String, String, String)> = {
        let mut st = w
            .store
            .conn()
            .prepare("SELECT entity, xero_id, caused_by_op FROM versions WHERE source = 'readback' ORDER BY id")
            .unwrap();
        st.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    let entities: Vec<&str> = rows.iter().map(|r| r.0.as_str()).collect();
    assert_eq!(
        entities,
        ["ManualJournals", "ManualJournals", "BankTransactions"],
        "{rows:?}"
    );
    assert!(
        rows.iter()
            .all(|r| r.2.ends_with(":op-01") || r.2.ends_with(":op-02") || r.2.ends_with(":op-03"))
    );
    assert_eq!(rows[2].1, "bt-104");
    let recoded: String = w
        .store
        .conn()
        .query_row(
            "SELECT json_extract(payload, '$.LineItems[0].AccountCode') FROM current WHERE xero_id = 'bt-104'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        recoded, "6310",
        "the record follows the recode without a sync"
    );
    // a second run sends nothing more
    let again = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert!(again.ops.is_empty());
    assert_eq!(writes(&w, "ManualJournals"), 2);
    let n: i64 = w
        .store
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM versions WHERE source = 'readback'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(n, 3, "verify re-read unchanged documents: no new versions");
    w.store.verify().unwrap();
}

#[test]
fn every_lock_is_required() {
    let w = world("locks", SCOPES);
    let p = package("demo-02", vec![journal_op(&w, "demo-02", "op-01", "88.17")]);
    // no approval yet
    let e = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap_err();
    assert_eq!((e.code.as_str(), e.exit), ("not_approved", 3));
    approve(&w, &p);
    // the one-run env var for another hash
    let e = with_ctx(&w, &p, Some("0".repeat(64)), |c| run::run(&p, c)).unwrap_err();
    assert_eq!((e.code.as_str(), e.exit), ("writes_disabled", 7));
    // the profile lock
    let e = with_ctx(&w, &p, None, |c| {
        c.allow_writes = false;
        run::run(&p, c)
    })
    .unwrap_err();
    assert_eq!(e.exit, 7);
    // a changed package (its seal no longer matches)
    let mut q = p.clone();
    q.title = "Demo (edited)".into();
    let e = with_ctx(&w, &q, p.seal.clone(), |c| run::run(&q, c)).unwrap_err();
    assert_eq!((e.code.as_str(), e.exit), ("hash_mismatch", 3));
    // a missing scope
    let w2 = world("locks-scope", "accounting.banktransactions");
    approve(&w2, &p);
    let e = with_ctx(&w2, &p, None, |c| run::run(&p, c)).unwrap_err();
    assert_eq!(e.exit, 7);
    assert_eq!(
        writes(&w, "ManualJournals") + writes(&w2, "ManualJournals"),
        0
    );
}

#[test]
fn a_stale_precondition_stops_before_sending() {
    let w = world("stale", SCOPES);
    let p = package("demo-03", vec![recode_op(&w)]);
    approve(&w, &p);
    // someone edits the line in Xero after planning
    w.fake.with(|s| {
        s.bank.get_mut(T).unwrap().get_mut("bt-104").unwrap()["LineItems"][0]["Description"] =
            json!("Edited")
    });
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    let e = rep.stopped.unwrap();
    assert_eq!((e.code.as_str(), e.exit), ("precondition_failed", 4));
    assert_eq!(writes(&w, "BankTransactions"), 0);
    // the error says what changed: a JSON Patch from the planned copy to the live document
    assert!(
        e.message
            .contains(r#"replace /LineItems/0/Description → "Edited""#),
        "{}",
        e.message
    );
    let detail = &rep.ops[0].2;
    assert_eq!(detail["diff"]["patch"][0]["op"], "replace", "{detail}");
    assert_eq!(
        detail["diff"]["patch"][0]["path"],
        "/LineItems/0/Description"
    );
    // and the attempt log keeps the live document as evidence
    let st = run::status(&p, &w.store).unwrap();
    assert_eq!(st["ops"][0]["attempts"][0]["state"], "rejected", "{st}");
    let ev: String = w
        .store
        .conn()
        .query_row(
            "SELECT detail FROM op_events ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(ev.contains("\"live\"") && ev.contains("Edited"), "{ev}");
}

#[test]
fn an_uncertain_attempt_is_reconciled_by_read_back_never_resent() {
    let w = world("uncertain", SCOPES);
    let p = package(
        "demo-04",
        vec![
            journal_op(&w, "demo-04", "op-01", "52.09"),
            attach_op(&w, "op-01"),
        ],
    );
    approve(&w, &p);
    w.fake.with(|s| {
        s.faults
            .push_back(("ManualJournals".into(), Fault::DropAfterApply))
    });
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert_eq!(rep.stopped.as_ref().map(|e| e.exit), Some(6));
    // nothing more is sent until reconciled
    let e = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap_err();
    assert_eq!((e.code.as_str(), e.exit), ("uncertain", 6));
    let r = with_ctx(&w, &p, None, |c| run::reconcile(&p, "op-01", c)).unwrap();
    assert_eq!(r["landed"], true);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert!(rep.stopped.is_none(), "{:?}", rep.stopped);
    assert_eq!(
        rep.ops.iter().map(|o| o.0.as_str()).collect::<Vec<_>>(),
        ["op-02"],
        "op-01 was not re-sent"
    );
    assert_eq!(
        writes(&w, "ManualJournals?")
            + w.fake.with(|s| s
                .log
                .iter()
                .filter(|l| l.0 == "PUT" && l.1.ends_with("/ManualJournals"))
                .count()),
        1
    );
}

#[test]
fn an_uncertain_attempt_that_did_not_land_is_resent_with_the_same_key() {
    let w = world("notlanded", SCOPES);
    let p = package("demo-05", vec![journal_op(&w, "demo-05", "op-01", "19.73")]);
    approve(&w, &p);
    w.fake.with(|s| {
        s.faults
            .push_back(("ManualJournals".into(), Fault::ServerErrorBeforeApply))
    });
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert_eq!(rep.stopped.as_ref().map(|e| e.exit), Some(6));
    let r = with_ctx(&w, &p, None, |c| run::reconcile(&p, "op-01", c)).unwrap();
    assert_eq!(r["landed"], false);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert!(rep.stopped.is_none(), "{:?}", rep.stopped);
    let keys: Vec<Option<String>> = w.fake.with(|s| {
        s.log
            .iter()
            .filter(|l| l.0 == "PUT")
            .map(|l| l.2.clone())
            .collect()
    });
    assert_eq!(keys.len(), 2);
    assert_eq!(keys[0], keys[1], "the same Idempotency-Key");
    let v = with_ctx(&w, &p, None, |c| run::verify(&p, c)).unwrap();
    assert_eq!(v["ok"], true);
}

#[test]
fn a_recode_that_comes_back_altered_is_a_discrepancy_and_a_refusal_is_rejected() {
    let w = world("altered", SCOPES);
    let p = package(
        "demo-06",
        vec![recode_op(&w), journal_op(&w, "demo-06", "op-04", "11.11")],
    );
    approve(&w, &p);
    w.fake
        .with(|s| s.alter_on_update = Some(("IsReconciled".into(), json!(false))));
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    let e = rep.stopped.unwrap();
    assert_eq!((e.code.as_str(), e.exit), ("discrepancy", 5));
    assert_eq!(writes(&w, "ManualJournals"), 0, "the batch stopped");
    // a reconciled line Xero refuses to edit
    let w = world("refused", SCOPES);
    w.fake.with(|s| s.refuse_reconciled_edit = true);
    let p = package("demo-07", vec![recode_op(&w)]);
    approve(&w, &p);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    let e = rep.stopped.unwrap();
    assert_eq!(e.code, "rejected");
    assert!(e.message.contains("reconciled"), "{}", e.message);
    let e = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap_err();
    assert_eq!(e.code, "rejected", "a refused package doesn't continue");
}

#[test]
fn at_most_ten_operations_per_run_and_a_dry_run_sends_nothing() {
    let w = world("batch", SCOPES);
    let ops: Vec<Operation> = (1..=12)
        .map(|i| {
            journal_op(
                &w,
                "demo-08",
                &format!("op-{i:02}"),
                &format!("{i}.{:02}", i * 7 % 100),
            )
        })
        .collect();
    let p = package("demo-08", ops);
    approve(&w, &p);
    let dry = with_ctx(&w, &p, Some("unset".into()), |c| {
        c.dry_run = true;
        run::run(&p, c)
    })
    .unwrap();
    assert_eq!(dry.ops.len(), 10);
    assert!(dry.ops.iter().all(|o| o.1 == "would_send"));
    assert_eq!(writes(&w, "ManualJournals"), 0);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert_eq!((rep.ops.len(), rep.remaining), (10, 2));
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert_eq!((rep.ops.len(), rep.remaining), (2, 0));
    let v = with_ctx(&w, &p, None, |c| run::verify(&p, c)).unwrap();
    assert_eq!(v["ok"], true, "{v}");
}

#[test]
fn recover_builds_a_package_that_runs_on_its_own_approval() {
    let w = world("recover", SCOPES);
    let p = package(
        "demo-09",
        vec![
            journal_op(&w, "demo-09", "op-01", "64.20"),
            attach_op(&w, "op-01"),
            recode_op(&w),
        ],
    );
    approve(&w, &p);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    assert!(rep.stopped.is_none());
    let mut rec = with_ctx(&w, &p, None, |c| run::recover(&p, None, c)).unwrap();
    assert_eq!(
        rec.ops.iter().map(|o| o.id.as_str()).collect::<Vec<_>>(),
        ["rec-op-03", "rec-op-01"]
    );
    assert_eq!(rec.excluded.len(), 1, "the attachment has no API recovery");
    rec.seal = Some(seal_of(&rec));
    // its own approval: the original's doesn't cover it
    let e = with_ctx(&w, &rec, None, |c| run::run(&rec, c)).unwrap_err();
    assert_eq!(e.code, "not_approved");
    approve(&w, &rec);
    let rep = with_ctx(&w, &rec, None, |c| run::run(&rec, c)).unwrap();
    assert!(rep.stopped.is_none(), "{:?}", rep.stopped);
    let v = with_ctx(&w, &rec, None, |c| run::verify(&rec, c)).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    // the line is back where it was
    assert_eq!(
        w.fake
            .with(|s| s.bank[T]["bt-104"]["LineItems"][0]["AccountCode"].clone()),
        "6520"
    );
}

#[test]
fn a_dependent_ops_date_is_never_a_sentinel() {
    // create → attach → void in one package: the void's journal date comes from the create
    let w = world("nosentinel", SCOPES);
    let mut void = attach_op(&w, "op-01");
    void.id = "op-03".into();
    void.op = Op::VoidJournal {
        journal: JournalRef::CreatedBy("op-01".into()),
    };
    void.request = ownpurse_core::write::ops::void_request("{id}");
    void.depends_on = vec!["op-01".into(), "op-02".into()];
    let create = journal_op(&w, "demo-10", "op-01", "33.17");
    // the void undoes the create's movement
    void.expected = create
        .expected
        .iter()
        .map(|d| ExpectedDelta {
            amount: -d.amount,
            ..d.clone()
        })
        .collect();
    let p = package("demo-10", vec![create, attach_op(&w, "op-01"), void]);
    approve(&w, &p);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    // create → attach → void in one package lands, the void's body carrying the created journal's id
    assert!(rep.stopped.is_none(), "{:?}", rep.stopped);
    assert_eq!(
        rep.ops.iter().map(|o| o.1.as_str()).collect::<Vec<_>>(),
        ["confirmed", "confirmed", "confirmed"]
    );
    let void_body: String = w.fake.with(|s| {
        s.log
            .iter()
            .rev()
            .find(|l| l.0 == "POST")
            .map(|l| l.3.clone())
            .unwrap()
    });
    assert!(
        !void_body.contains("{id}") && void_body.contains("0000000a-"),
        "{void_body}"
    );
    let v = with_ctx(&w, &p, None, |c| run::verify(&p, c)).unwrap();
    assert_eq!(v["ok"], true, "{v}");
    let dates: Vec<String> = {
        let mut st = w
            .store
            .conn()
            .prepare("SELECT json_extract(params, '$.date') FROM reports")
            .unwrap();
        st.query_map([], |r| r.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect()
    };
    assert!(!dates.is_empty());
    assert!(dates.iter().all(|d| !d.starts_with("9999")), "{dates:?}");
    let sent: Vec<String> = w.fake.with(|s| s.log.iter().map(|l| l.1.clone()).collect());
    assert!(sent.iter().all(|p| !p.contains("9999")), "{sent:?}");
}

#[test]
fn the_fingerprint_is_the_same_for_a_list_and_a_by_id_shape() {
    // as real Xero: the list (sync) and GET-by-id shapes of one bank transaction differ in contact details,
    // account names and ids, tax types and number formats; the business view doesn't
    let list = json!({"BankTransactionID": "bt-1", "Type": "SPEND", "Status": "AUTHORISED",
        "Contact": {"ContactID": "c-1", "Name": "Q"}, "BankAccount": {"AccountID": "a-1", "Code": "1010"},
        "DateString": "2025-03-04T00:00:00", "LineAmountTypes": "NoTax", "IsReconciled": true, "Total": 6,
        "LineItems": [{"LineItemID": "l1", "AccountCode": "652", "Description": "x", "LineAmount": 6.00}]});
    let by_id = json!({"BankTransactionID": "bt-1", "Type": "SPEND", "Status": "AUTHORISED",
        "Contact": {"ContactID": "c-1", "Name": "Q", "ContactStatus": "ACTIVE", "Addresses": [{"AddressType": "POBOX"}]},
        "BankAccount": {"AccountID": "a-1", "Code": "1010", "Name": "Operating"},
        "Date": "/Date(1741046400000+0000)/", "LineAmountTypes": "NoTax", "IsReconciled": true, "Total": "6.0000",
        "HasAttachments": false, "UpdatedDateUTC": "/Date(1)/",
        "LineItems": [{"LineItemID": "l1", "AccountCode": "652", "AccountID": "x-1", "TaxType": "NONE",
                       "Tracking": [], "Description": "x", "LineAmount": "6.0000", "TaxAmount": 0}]});
    assert_eq!(business_view(&list), business_view(&by_id));
    assert_eq!(fingerprint(&list), fingerprint(&by_id));
    let mut moved = by_id.clone();
    moved["LineItems"][0]["AccountCode"] = json!("6310");
    assert_ne!(fingerprint(&list), fingerprint(&moved));
}

#[test]
fn xeros_error_message_is_surfaced_and_a_placeholder_is_never_sent() {
    // a void whose body names a journal that isn't a GUID: Xero's top-level Message comes through
    let w = world("xeromsg", SCOPES);
    let mut op = attach_op(&w, "op-01");
    op.id = "op-09".into();
    op.op = Op::VoidJournal {
        journal: JournalRef::Id("not-a-guid".into()),
    };
    op.request = ownpurse_core::write::ops::void_request("not-a-guid");
    op.depends_on = vec![];
    let p = package("demo-11", vec![op]);
    approve(&w, &p);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    let e = rep.stopped.unwrap();
    assert_eq!(e.code, "rejected");
    assert!(
        e.message.contains("PostDataInvalidException (14): Error converting value \"not-a-guid\" to type 'System.Guid'"),
        "{}",
        e.message
    );
    assert!(
        rep.ops[0].2["errors"][0]
            .as_str()
            .unwrap()
            .starts_with("PostDataInvalidException (14)")
    );
    // a request whose journal isn't known: refused before anything is sent
    let w = world("unresolved", SCOPES);
    let mut op = attach_op(&w, "op-01");
    op.id = "op-08".into();
    // a reference the run can't resolve (as a hand-edited package might carry)
    op.op = Op::VoidJournal {
        journal: JournalRef::Id("x".into()),
    };
    op.request = ownpurse_core::write::ops::void_request("op:op-1");
    op.depends_on = vec![];
    let p = package("demo-12", vec![op]);
    approve(&w, &p);
    let rep = with_ctx(&w, &p, None, |c| run::run(&p, c)).unwrap();
    let e = rep.stopped.unwrap();
    assert_eq!(e.code, "precondition_failed", "{}", e.message);
    assert!(
        e.message.contains("still holds an `op:` reference"),
        "{}",
        e.message
    );
    assert_eq!(
        w.fake
            .with(|s| s.log.iter().filter(|l| l.0 == "POST").count()),
        0
    );
}
