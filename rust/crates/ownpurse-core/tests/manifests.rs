//! Manifests on the demo world (a temp copy; the fixture is never opened for writing): normalised JSON
//! Patch entries, the overlay and its effects, conflicts and posted entries, CHECK, PLAN into the sealed
//! write package, the stages, the guards, and the invariants (main never modified, undo-all = main,
//! determinism, conflicts never applied).

use ownpurse_core::Ledger;
use ownpurse_core::manifest::check::{self, Options, Severity};
use ownpurse_core::manifest::compile;
use ownpurse_core::manifest::ops::{self, Guard, JLine, Target};
use ownpurse_core::manifest::stage::{self, EntryStatus, Stage};
use ownpurse_core::manifest::{Refs, Store, effects, normalise::normalise};
use ownpurse_core::record::Record;
use ownpurse_core::view::{Ctx, View};
use ownpurse_core::write::plan::PlanConfig;
use rust_decimal::Decimal;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

/// A temp folder with a COPY of the demo record, plus a bank transaction with Xero line ids (lab).
fn tmp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("xam-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    for ext in ["", "-wal", "-shm"] {
        let f = demo().join(format!("record.sqlite{ext}"));
        if f.exists() {
            std::fs::copy(&f, d.join(format!("record.sqlite{ext}"))).unwrap();
        }
    }
    let lab = ctx().tenant("lab").unwrap().to_string();
    append(
        &d,
        &lab,
        "BankTransactions",
        "bt-900",
        json!({"BankAccount": {"AccountID": "l-card"}, "BankTransactionID": "bt-900",
               "Contact": {"Name": "Panel Supplies Co"}, "DateString": "2026-02-10T00:00:00",
               "IsReconciled": false, "LineAmountTypes": "NoTax", "Status": "AUTHORISED", "Type": "SPEND",
               "Total": 148.90,
               "LineItems": [
                 {"LineItemID": "li-1", "AccountCode": "5510", "Description": "tasting cups", "LineAmount": 118.40, "TaxAmount": 0},
                 {"LineItemID": "li-2", "AccountCode": "5510", "Description": "spoons", "LineAmount": 30.50, "TaxAmount": 0}]}),
    );
    d
}

/// A sync: a newer version of a document (appended to the temp copy).
fn append(dir: &Path, tid: &str, entity: &str, id: &str, payload: Value) {
    let c = rusqlite::Connection::open(dir.join("record.sqlite")).unwrap();
    let ver: i64 = c
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM versions WHERE tenant_id=?1 AND entity=?2 AND xero_id=?3",
            rusqlite::params![tid, entity, id],
            |r| r.get(0),
        )
        .unwrap();
    c.execute(
        "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
         VALUES (?1,?2,?3,?4,'x',?5,'2026-10-07T08:00:00Z','sync',?6)",
        rusqlite::params![
            tid,
            entity,
            id,
            ver + 1,
            payload.get("Status").and_then(Value::as_str),
            payload.to_string()
        ],
    )
    .unwrap();
}

fn ctx() -> Ctx {
    let t: Value =
        serde_json::from_str(&std::fs::read_to_string(demo().join("tenants.json")).unwrap())
            .unwrap();
    let mut orgs: Vec<(String, String)> = t
        .as_object()
        .unwrap()
        .iter()
        .map(|(a, v)| (a.clone(), v.as_str().unwrap().to_string()))
        .collect();
    orgs.sort();
    Ctx {
        orgs,
        gate: vec![("hold".into(), "2025-12-31".into())],
    }
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn tb(view: &View, rec: &Record, org: &str) -> std::collections::BTreeMap<String, Decimal> {
    let c = ctx();
    let tid = c.tenant(org).unwrap();
    let l = Ledger::build(view, tid).unwrap();
    let fy = rec.fy_end(tid).unwrap();
    ownpurse_core::balances::resolve(rec, &l, tid, "2025-12-31", fy, ownpurse_core::Mode::Auto)
        .unwrap()
        .balances
}

fn code_bal(view: &View, rec: &Record, org: &str, code: &str) -> Decimal {
    let tid = ctx().tenant(org).unwrap().to_string();
    let l = Ledger::build(view, &tid).unwrap();
    let id = l.chart.by_code(code).unwrap().account_id.clone();
    tb(view, rec, org).get(&id).copied().unwrap_or_default()
}

struct World {
    dir: PathBuf,
    rec: Record,
    store: Store,
}

fn world(name: &str) -> World {
    let dir = tmp(name);
    let rec = Record::open(dir.join("record.sqlite")).unwrap();
    let store = Store::open(&dir.join("db/manifests.sqlite")).unwrap();
    World { dir, rec, store }
}

fn target(entity: &str, org: &str, id: Option<&str>) -> Target {
    Target {
        entity: entity.into(),
        org: org.into(),
        id: id.map(str::to_string),
    }
}

fn jl(account: &str, amount: &str) -> JLine {
    JLine {
        account: account.into(),
        description: String::new(),
        amount: d(amount),
    }
}

impl World {
    fn reopen(&mut self) {
        self.rec = Record::open(self.dir.join("record.sqlite")).unwrap();
    }
    fn create(&mut self, name: &str) {
        ops::create(&mut self.store, &self.rec, &ctx(), "human", name, "test").unwrap();
    }
    fn manifest(&self, name: &str) -> ownpurse_core::manifest::Manifest {
        self.store.manifest(name).unwrap()
    }
    fn view(&self, name: &str) -> View<'_> {
        View::open(&self.rec, &ctx(), Some(&self.manifest(name))).unwrap()
    }
    fn put(&mut self, name: &str, t: Target, after: Value) -> i64 {
        ops::add(
            &mut self.store,
            &self.rec,
            &ctx(),
            name,
            "human",
            &t,
            &after,
            "",
            &Refs::default(),
            None,
            &Guard::default(),
        )
        .unwrap()
    }
    fn journal(&mut self, name: &str, org: &str, date: &str, lines: &[(&str, &str)]) -> i64 {
        let ls: Vec<JLine> = lines.iter().map(|(a, amt)| jl(a, amt)).collect();
        let after = ops::journal_doc(date, "manifest test", true, &ls);
        self.put(name, target("ManualJournals", org, None), after)
    }
    fn doc(&self, name: &str, org: &str, entity: &str, id: &str) -> Value {
        ops::current(&self.view(name), &ctx(), &target(entity, org, Some(id)))
            .unwrap()
            .unwrap()
    }
    fn recode(&mut self, name: &str, org: &str, id: &str, line: &str, acct: &str) -> i64 {
        let doc = self.doc(name, org, "BankTransactions", id);
        let after = ops::recode_doc(&doc, line, Some(acct), None).unwrap();
        self.put(name, target("BankTransactions", org, Some(id)), after)
    }
    fn void(&mut self, name: &str, org: &str, entity: &str, id: &str) -> i64 {
        let doc = self.doc(name, org, entity, id);
        self.put(name, target(entity, org, Some(id)), ops::void_doc(&doc))
    }
    fn check(&self, name: &str) -> check::Report {
        let m = self.manifest(name);
        let diff = effects::diff(
            &self.rec,
            &ctx(),
            &m,
            "2025-12-31",
            None,
            &Default::default(),
        )
        .unwrap();
        check::check(&self.rec, &ctx(), &m, &Options::default(), Some(&diff)).unwrap()
    }
}

fn cfg(gates: bool) -> PlanConfig {
    PlanConfig {
        profile: "demo".into(),
        orgs: ctx().orgs.clone(),
        gates: if gates { ctx().gate.clone() } else { vec![] },
        history_notes: false,
    }
}

fn file_hash(p: &Path) -> Vec<u8> {
    use std::io::Read;
    let mut v = vec![];
    for ext in ["", "-wal"] {
        let f = PathBuf::from(format!("{}{ext}", p.display()));
        if let Ok(mut h) = std::fs::File::open(&f) {
            h.read_to_end(&mut v).unwrap();
        }
    }
    v
}

#[test]
fn a_new_journal_is_a_patch_from_nothing_and_clears_a_must_be_zero_balance() {
    let mut w = world("zero");
    let before = file_hash(&w.dir.join("record.sqlite"));
    w.create("payroll");
    assert_eq!(
        code_bal(&View::main(&w.rec), &w.rec, "design", "2250"),
        d("-1847.36")
    );
    let s = w.journal(
        "payroll",
        "design",
        "2025-12-31",
        &[("2250", "1847.36"), ("1014", "-1847.36")],
    );
    let m = w.manifest("payroll");
    let e = m.entry(s).unwrap();
    assert!(e.is_new() && e.id == format!("new:{s}"));
    let v = w.view("payroll");
    assert_eq!(code_bal(&v, &w.rec, "design", "2250"), Decimal::ZERO);
    assert_eq!(code_bal(&v, &w.rec, "design", "1014"), d("246.21"));
    let tid = ctx().tenant("design").unwrap().to_string();
    let l = Ledger::build(&v, &tid).unwrap();
    let marked: Vec<_> = l.postings.iter().filter(|p| p.proposed.is_some()).collect();
    assert_eq!(marked.len(), 2);
    assert!(
        marked
            .iter()
            .all(|p| p.proposed.as_ref().unwrap().glyph() == "+")
    );
    let mut zero = std::collections::BTreeMap::new();
    zero.insert("design".to_string(), vec!["2250".to_string()]);
    let diff = effects::diff(&w.rec, &ctx(), &m, "2025-12-31", None, &zero).unwrap();
    let z = diff
        .orgs
        .iter()
        .find(|o| o.org == "design")
        .unwrap()
        .zero
        .iter()
        .find(|a| a.code == "2250")
        .unwrap()
        .clone();
    assert_eq!((z.main, z.manifest), (d("-1847.36"), Decimal::ZERO));
    assert_eq!(diff.facts.journals, 1);
    drop(v);
    assert_eq!(
        file_hash(&w.dir.join("record.sqlite")),
        before,
        "main never modified"
    );
}

#[test]
fn a_recode_and_a_void_through_the_generic_overlay() {
    let mut w = world("recode");
    w.create("tidy");
    let main = View::main(&w.rec);
    let (m6140, m5020, m6520) = (
        code_bal(&main, &w.rec, "roast", "6140"),
        code_bal(&main, &w.rec, "roast", "5020"),
        code_bal(&main, &w.rec, "roast", "6520"),
    );
    let r = w.recode("tidy", "roast", "bt-110", "L1", "5020");
    assert_eq!(
        w.manifest("tidy").entry(r).unwrap().paths(),
        vec!["/LineItems/n1/AccountCode".to_string()],
        "keyed paths, never array indices"
    );
    w.void("tidy", "roast", "BankTransactions", "bt-112");
    let v = w.view("tidy");
    assert_eq!(code_bal(&v, &w.rec, "roast", "6140"), m6140 - d("214.77"));
    assert_eq!(code_bal(&v, &w.rec, "roast", "5020"), m5020 + d("214.77"));
    assert_eq!(code_bal(&v, &w.rec, "roast", "6520"), m6520 - d("238.61"));
    let tid = ctx().tenant("roast").unwrap().to_string();
    let l = Ledger::build(&v, &tid).unwrap();
    assert!(l.ghosts.iter().all(|p| p.source_id == "bt-112") && !l.ghosts.is_empty());
    let marked: Vec<_> = l
        .postings
        .iter()
        .filter(|p| p.source_id == "bt-110" && p.proposed.is_some())
        .collect();
    assert_eq!(marked.len(), 1, "only the recoded line, not the bank side");
    let mk = marked[0].proposed.as_ref().unwrap();
    assert_eq!((mk.kind, mk.glyph()), ("change", "~"));
    assert_eq!(l.chart.get(mk.was.as_deref().unwrap()).code, "6140");
    // the check: bt-110 is reconciled, so its recode goes through the reclass journal (no line id needed);
    // a bank-transaction void is not writable
    let rep = w.check("tidy");
    assert!(
        !rep.issues
            .iter()
            .any(|i| i.code == "line_id" || i.code == "reconciled"),
        "{:?}",
        rep.issues
    );
    assert!(rep.issues.iter().any(|i| i.code == "not_writable"));
    assert_eq!(
        w.manifest("tidy").entry(r).unwrap().resolution.as_deref(),
        Some("reclass")
    );
}

#[test]
fn an_intercompany_bridge_goes_to_zero_with_the_manifest() {
    let mut w = world("ic");
    w.create("ic-fix");
    w.journal(
        "ic-fix",
        "roast",
        "2025-12-31",
        &[("6290", "263.94"), ("2430", "-263.94")],
    );
    let cfg: Value =
        serde_json::from_str(&std::fs::read_to_string(demo().join("ownpurse.json")).unwrap())
            .unwrap();
    let diff = effects::diff(
        &w.rec,
        &ctx(),
        &w.manifest("ic-fix"),
        "2025-12-31",
        Some(&cfg),
        &Default::default(),
    )
    .unwrap();
    let br = diff
        .bridges
        .iter()
        .find(|b| b.pair == "design-roast")
        .unwrap();
    assert_eq!(br.main, Some(d("263.94")));
    assert_eq!(br.manifest, Some(Decimal::ZERO), "{br:?}");
}

#[test]
fn main_moving_is_a_conflict_and_main_reaching_after_is_posted() {
    let mut w = world("conflict");
    w.create("tidy");
    let r = w.recode("tidy", "lab", "bt-900", "li-2", "2440");
    let j = w.journal(
        "tidy",
        "lab",
        "2026-01-31",
        &[("5510", "10.01"), ("2440", "-10.01")],
    );
    let lab = ctx().tenant("lab").unwrap().to_string();
    let raw = w
        .rec
        .current_rows(&lab, "BankTransactions")
        .unwrap()
        .into_iter()
        .find(|r| r.xero_id == "bt-900")
        .unwrap()
        .payload;
    // a sync changes the amount: a conflict, never applied; the independent journal still applies
    let mut moved = raw.clone();
    moved["LineItems"][1]["LineAmount"] = json!(31.75);
    append(&w.dir, &lab, "BankTransactions", "bt-900", moved);
    w.reopen();
    {
        let v = w.view("tidy");
        let ov = v.overlay().unwrap();
        assert_eq!(ov.conflicts.len(), 1);
        assert_eq!(ov.conflicts[0].seq, r);
        assert!(ov.applied(j).is_some() && ov.applied(r).is_none());
    }
    assert!(
        w.check("tidy")
            .issues
            .iter()
            .any(|i| i.code == "stale" && i.seq == r)
    );
    // a later read-back equal to the entry's "after": posted, and the overlay is a no-op for it
    let mut after = raw.clone();
    after["LineItems"][1]["AccountCode"] = json!("2440");
    after["UpdatedDateUTC"] = json!("/Date(1791000000000+0000)/");
    append(&w.dir, &lab, "BankTransactions", "bt-900", after);
    w.reopen();
    let v = w.view("tidy");
    let ov = v.overlay().unwrap();
    assert!(ov.is_posted(r) && ov.conflicts.is_empty());
    let l = Ledger::build(&v, &lab).unwrap();
    assert!(
        l.postings
            .iter()
            .filter(|p| p.source_id == "bt-900")
            .all(|p| p.proposed.is_none())
    );
}

#[test]
fn undo_all_is_main_views_are_deterministic_and_one_entry_per_document() {
    let mut w = world("undo");
    w.create("all");
    let a = w.journal(
        "all",
        "design",
        "2025-12-31",
        &[("2250", "1847.36"), ("1014", "-1847.36")],
    );
    let b = w.recode("all", "lab", "bt-900", "li-1", "2440");
    // a second change to the same document replaces (combines with) the first
    let doc = w.doc("all", "lab", "BankTransactions", "bt-900");
    assert_eq!(doc["LineItems"]["li-1"]["AccountCode"], "2440");
    let after = ops::recode_doc(&doc, "li-2", None, Some("spoons (panel)")).unwrap();
    let c = w.put(
        "all",
        target("BankTransactions", "lab", Some("bt-900")),
        after,
    );
    let m = w.manifest("all");
    assert_eq!(m.entry(b).unwrap().replaced_by, Some(c));
    assert_eq!(m.entry(c).unwrap().paths().len(), 2);
    let v1: Vec<_> = ["lab", "design"]
        .iter()
        .map(|o| tb(&w.view("all"), &w.rec, o))
        .collect();
    let v2: Vec<_> = ["lab", "design"]
        .iter()
        .map(|o| tb(&w.view("all"), &w.rec, o))
        .collect();
    assert_eq!(v1, v2, "deterministic");
    for s in [a, c] {
        ops::undo(&mut w.store, "all", "human", s, true, &Guard::default()).unwrap();
    }
    let main = View::main(&w.rec);
    for o in ["roast", "hold", "design", "lab"] {
        let tid = ctx().tenant(o).unwrap().to_string();
        let lb = Ledger::build(&w.view("all"), &tid).unwrap();
        let lm = Ledger::build(&main, &tid).unwrap();
        assert_eq!(lb.postings, lm.postings, "{o}: undo-all = main");
        assert!(lb.ghosts.is_empty());
    }
    ops::undo(&mut w.store, "all", "human", a, false, &Guard::default()).unwrap();
    assert_eq!(
        code_bal(&w.view("all"), &w.rec, "design", "2250"),
        Decimal::ZERO
    );
}

#[test]
fn check_errors_and_warnings_then_plan_into_the_sealed_package() {
    let mut w = world("plan");
    std::fs::write(w.dir.join("support.csv"), "date,amount\n2026-01-31,10.01\n").unwrap();
    w.create("dec");
    // errors: an unknown account (with a suggestion), an unbalanced journal, an amount edit on a posted journal
    let bad = w.journal(
        "dec",
        "design",
        "2026-01-31",
        &[("6429", "5.00"), ("2420", "-4.00")],
    );
    let mj = w.doc("dec", "lab", "ManualJournals", "mj-401");
    let mut amt = mj.clone();
    amt["JournalLines"]["n1"]["LineAmount"] = json!("60.00");
    amt["JournalLines"]["n2"]["LineAmount"] = json!("-60.00");
    let edit = w.put("dec", target("ManualJournals", "lab", Some("mj-401")), amt);
    let rep = w.check("dec");
    let codes: Vec<(&str, i64)> = rep.issues.iter().map(|i| (i.code, i.seq)).collect();
    assert!(
        codes.contains(&("account", bad)) && codes.contains(&("unbalanced", bad)),
        "{codes:?}"
    );
    assert!(
        rep.issues
            .iter()
            .any(|i| i.code == "account" && i.message.contains("did you mean 6420"))
    );
    let nw = rep
        .issues
        .iter()
        .find(|i| i.code == "not_writable")
        .unwrap();
    assert_eq!(nw.seq, edit);
    assert!(nw.hint.contains("void-recreate"));
    assert!(nw.path.starts_with("/JournalLines/n"));
    let e =
        compile::compile(&w.rec, &ctx(), &cfg(true), &w.manifest("dec"), &rep, "now").unwrap_err();
    assert_eq!(e.code, "check");
    let m = w.manifest("dec");
    assert!(matches!(
        stage::status(&w.rec, &m, &rep, None).stage,
        Stage::CheckFailed { errors } if errors >= 3
    ));
    // fix: undo both; then a clean journal, a recode on a reconciled line, a void and an attachment
    for s in [bad, edit] {
        ops::undo(&mut w.store, "dec", "human", s, true, &Guard::default()).unwrap();
    }
    let j = w.journal(
        "dec",
        "design",
        "2026-01-31",
        &[("6420", "263.94"), ("2420", "-263.94")],
    );
    let r = w.recode("dec", "lab", "bt-900", "li-2", "2440");
    let v = w.void("dec", "lab", "ManualJournals", "mj-401");
    let att_doc = w.doc("dec", "design", "ManualJournals", "mj-303");
    let att = ops::attach_doc(&att_doc, &w.dir.join("support.csv")).unwrap();
    let a = w.put(
        "dec",
        target("ManualJournals", "design", Some("mj-303")),
        att,
    );
    let rep = w.check("dec");
    assert_eq!(rep.errors(), 0, "{:?}", rep.issues);
    let warns: Vec<(&str, i64)> = rep.issues.iter().map(|i| (i.code, i.seq)).collect();
    assert!(
        !warns.contains(&("reconciled", r)) && warns.contains(&("attachment", a)),
        "{warns:?}"
    );
    assert_eq!(
        stage::status(&w.rec, &w.manifest("dec"), &rep, None).stage,
        Stage::Review
    );
    assert_eq!(
        compile::compile(&w.rec, &ctx(), &cfg(true), &w.manifest("dec"), &rep, "now")
            .unwrap_err()
            .code,
        "review"
    );
    for i in rep.issues.clone() {
        ops::ack(
            &mut w.store,
            "dec",
            "human",
            i.seq,
            i.code,
            &i.path,
            true,
            &Guard::default(),
        )
        .unwrap();
    }
    let rep = w.check("dec");
    assert_eq!((rep.errors(), rep.unacked(), rep.acknowledged()), (0, 0, 1));
    assert_eq!(
        stage::status(&w.rec, &w.manifest("dec"), &rep, None).stage,
        Stage::Checked
    );
    let m = w.manifest("dec");
    let pkg =
        compile::compile(&w.rec, &ctx(), &cfg(true), &m, &rep, "2026-10-07T12:00:00Z").unwrap();
    assert_eq!(pkg.id, format!("dec-r{}", m.content_rev));
    assert_eq!(
        pkg.source.as_deref(),
        Some(format!("manifest:dec@r{}", m.content_rev).as_str())
    );
    let kinds: Vec<(String, &str)> = pkg
        .ops
        .iter()
        .map(|o| (o.id.clone(), o.op.kind()))
        .collect();
    assert_eq!(
        kinds,
        vec![
            (format!("e{j}"), "create_journal"),
            (format!("e{r}"), "recode_bank_line"),
            (format!("e{v}"), "void_journal"),
            (format!("e{a}"), "attach"),
        ]
    );
    let sealed = ownpurse_core::write::seal::seal(pkg).unwrap();
    let hash = sealed.seal.clone().unwrap();
    ops::planned(
        &mut w.store,
        "dec",
        "human",
        "db/packages/dec.package.json",
        &hash,
        m.content_rev,
    )
    .unwrap();
    let m = w.manifest("dec");
    let st = stage::status(&w.rec, &m, &rep, None);
    assert_eq!(st.stage, Stage::Planned);
    assert!(st.entries.iter().all(|(_, s)| *s == EntryStatus::Queued));
    let cmds = compile::commands("dec", &hash);
    assert_eq!(
        cmds.apply,
        format!("OWNPURSE_WRITE_RUN={hash} ownpurse manifest apply dec --package {hash}")
    );
    assert!(cmds.approve.contains(&hash));
    // a change after planning makes the plan stale (back to check)
    ops::undo(&mut w.store, "dec", "human", a, true, &Guard::default()).unwrap();
    let m = w.manifest("dec");
    let st = stage::status(&w.rec, &m, &w.check("dec"), None);
    assert!(st.plan_stale && st.stage != Stage::Planned);
}

#[test]
fn every_entry_posted_is_applied_and_read_only() {
    let mut w = world("applied");
    w.create("v");
    let v = w.void("v", "lab", "ManualJournals", "mj-401");
    let rep = w.check("v");
    let m = w.manifest("v");
    let pkg = ownpurse_core::write::seal::seal(
        compile::compile(&w.rec, &ctx(), &cfg(false), &m, &rep, "now").unwrap(),
    )
    .unwrap();
    ops::planned(
        &mut w.store,
        "v",
        "human",
        "p.json",
        pkg.seal.as_deref().unwrap(),
        m.content_rev,
    )
    .unwrap();
    // the read-back: main now holds the voided journal
    let lab = ctx().tenant("lab").unwrap().to_string();
    let mut raw = w
        .rec
        .current_rows(&lab, "ManualJournals")
        .unwrap()
        .into_iter()
        .find(|r| r.xero_id == "mj-401")
        .unwrap()
        .payload;
    raw["Status"] = json!("VOIDED");
    raw["UpdatedDateUTC"] = json!("/Date(1791000000000+0000)/");
    append(&w.dir, &lab, "ManualJournals", "mj-401", raw);
    w.reopen();
    let m = w.manifest("v");
    let rep = w.check("v");
    let view = w.view("v");
    let ov = view.overlay().unwrap();
    assert!(ov.is_posted(v));
    let st = stage::status(&w.rec, &m, &rep, Some(ov));
    assert_eq!(st.stage, Stage::Applied);
    assert!(st.stage.read_only());
    assert_eq!(st.stage.short(1, 0), ("applied ✓".to_string(), false));
    assert_eq!(*st.of(v), EntryStatus::Posted);
}

#[test]
fn the_accountant_gate_is_an_error_until_referenced() {
    let mut w = world("gate");
    w.create("aje");
    let s = w.journal(
        "aje",
        "hold",
        "2025-12-31",
        &[("4610", "120.50"), ("1470", "-120.50")],
    );
    let o = w.journal(
        "aje",
        "lab",
        "2026-01-31",
        &[("5510", "1.01"), ("2440", "-1.01")],
    );
    {
        let v = w.view("aje");
        let ov = v.overlay().unwrap();
        assert!(
            ov.applied(s).unwrap().needs_accountant,
            "hold is gated through 2025-12-31"
        );
        assert!(!ov.applied(o).unwrap().needs_accountant);
    }
    assert!(
        w.check("aje")
            .issues
            .iter()
            .any(|i| i.code == "accountant" && i.seq == s)
    );
    ops::annotate(
        &mut w.store,
        "aje",
        "human",
        s,
        Some("AJE-2"),
        None,
        &Guard::default(),
    )
    .unwrap();
    assert!(!w.check("aje").issues.iter().any(|i| i.code == "accountant"));
}

#[test]
fn guards_refuse_and_the_store_is_chained() {
    let mut w = world("guards");
    w.create("g");
    let rev = w.manifest("g").head_rev;
    let t = target("ManualJournals", "roast", None);
    let after = ops::journal_doc(
        "2026-01-31",
        "x",
        true,
        &[jl("6290", "1.01"), jl("2430", "-1.01")],
    );
    let s = ops::add(
        &mut w.store,
        &w.rec,
        &ctx(),
        "g",
        "agent:bot",
        &t,
        &after,
        "why",
        &Refs::default(),
        None,
        &Guard {
            expect_rev: Some(rev),
            expect_manifest: Some("g".into()),
            overwrite: false,
        },
    )
    .unwrap();
    let stale = Guard {
        expect_rev: Some(rev),
        ..Default::default()
    };
    assert_eq!(
        ops::undo(&mut w.store, "g", "human", s, true, &stale)
            .unwrap_err()
            .code,
        "view_changed"
    );
    assert_eq!(
        ops::undo(&mut w.store, "g", "agent:bot", s, true, &Guard::default())
            .unwrap_err()
            .code,
        "forbidden"
    );
    ops::undo(
        &mut w.store,
        "g",
        "agent:bot",
        s,
        true,
        &Guard {
            overwrite: true,
            ..Default::default()
        },
    )
    .unwrap();
    // an entry that changes nothing is refused (never stored)
    let doc = w.doc("g", "lab", "ManualJournals", "mj-401");
    let r = ops::add(
        &mut w.store,
        &w.rec,
        &ctx(),
        "g",
        "human",
        &target("ManualJournals", "lab", Some("mj-401")),
        &doc,
        "",
        &Refs::default(),
        None,
        &Guard::default(),
    );
    assert_eq!(r.unwrap_err().code, "usage");
    ops::discard(&mut w.store, "g", "human", &Guard::default()).unwrap();
    assert_eq!(
        ops::undo(&mut w.store, "g", "human", s, false, &Guard::default())
            .unwrap_err()
            .code,
        "usage"
    );
    assert_eq!(w.store.verify().unwrap(), w.store.events().len());
    assert_eq!(
        normalise(&doc),
        doc,
        "normalising a normalised document is the identity"
    );
}

#[test]
fn documents_are_read_only_through_the_view() {
    // the single entry point: no `.main_payloads(` / `record.main_document(` outside record.rs / view.rs
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut bad = vec![];
    fn walk(d: &Path, out: &mut Vec<PathBuf>) {
        for e in std::fs::read_dir(d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out)
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p)
            }
        }
    }
    let mut files = vec![];
    walk(&src, &mut files);
    for f in files {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        if name == "record.rs" || name == "view.rs" {
            continue;
        }
        let t = std::fs::read_to_string(&f).unwrap();
        if t.contains(".main_payloads(") || t.contains("record.main_document(") {
            bad.push(f.display().to_string());
        }
    }
    assert!(bad.is_empty(), "documents read around the View: {bad:?}");
}

/// Xero refuses API edits of RECONCILED bank transactions (HTTP 400, verified on the Demo Company): such a
/// recode compiles to the org-month reclass journal (the default) or a browser checklist step; an entry with
/// no resolution is an error that offers both. bt-110 (roast, 2025-10-21) is reconciled in the demo world.
#[test]
fn reconciled_lines_go_through_a_reclass_journal_or_the_browser() {
    let mut w = world("reclass");
    w.create("rc");
    let r = w.recode("rc", "roast", "bt-110", "L1", "5020");
    assert_eq!(
        w.manifest("rc").entry(r).unwrap().resolution.as_deref(),
        Some("reclass"),
        "the default"
    );
    let rep = w.check("rc");
    assert_eq!(rep.errors(), 0, "{:?}", rep.issues);
    // the economic effect still shows on the line: 6140 → 5020
    assert_eq!(
        w.view("rc").overlay().unwrap().applied(r).unwrap().kind,
        "change"
    );
    let m = w.manifest("rc");
    let pkg = compile::compile(&w.rec, &ctx(), &cfg(false), &m, &rep, "now").unwrap();
    assert_eq!(pkg.ops.len(), 1);
    let op = &pkg.ops[0];
    assert_eq!(op.id, "reclass-roast-202510");
    let ownpurse_core::write::Op::CreateJournal(j) = &op.op else {
        panic!()
    };
    assert_eq!(j.date, "2025-10-21");
    let lines: Vec<(String, Decimal)> = j
        .lines
        .iter()
        .map(|l| (l.account_code.clone(), l.amount))
        .collect();
    assert!(
        lines.contains(&("5020".into(), d("214.77")))
            && lines.contains(&("6140".into(), d("-214.77"))),
        "{lines:?}"
    );
    assert!(j.narration.contains("bt-110#n1") && j.narration.contains("XA:"));
    // two reconciled lines in the same org-month share one reclass journal
    let s2 = w.recode("rc", "roast", "bt-112", "L1", "5020");
    let rep = w.check("rc");
    let pkg =
        compile::compile(&w.rec, &ctx(), &cfg(false), &w.manifest("rc"), &rep, "now").unwrap();
    assert_eq!(pkg.ops.len(), 1, "one reclass journal per org per month");
    let ownpurse_core::write::Op::CreateJournal(j) = &pkg.ops[0].op else {
        panic!()
    };
    assert_eq!(j.lines.len(), 4);
    // the browser path: no op, a checklist step
    ops::resolve(
        &mut w.store,
        "rc",
        "human",
        Some(s2),
        "browser",
        &Guard::default(),
    )
    .unwrap();
    let rep = w.check("rc");
    let pkg =
        compile::compile(&w.rec, &ctx(), &cfg(false), &w.manifest("rc"), &rep, "now").unwrap();
    assert_eq!(pkg.ops.len(), 1);
    assert!(
        pkg.checklist.iter().any(
            |c| c.contains("unreconcile bank transaction bt-112") && c.contains("re-reconcile")
        )
    );
    // an entry with no resolution (a raw patch on a reconciled line) is an error offering both
    let mut w2 = world("reclass2");
    w2.create("raw");
    let roast = ctx().tenant("roast").unwrap().to_string();
    let before = w2
        .rec
        .current_rows(&roast, "BankTransactions")
        .unwrap()
        .into_iter()
        .find(|r| r.xero_id == "bt-110")
        .unwrap()
        .payload;
    let mut after = before.clone();
    after["LineItems"][0]["AccountCode"] = json!("5020");
    let mut p = ownpurse_core::manifest::entry_payload(
        "BankTransactions",
        "roast",
        Some("bt-110"),
        Some(&before),
        &after,
        "",
        &Refs::default(),
        None,
    );
    p.as_object_mut().unwrap().remove("resolution");
    let s = w2
        .store
        .append("human", "raw", ownpurse_core::manifest::Kind::EntryAdd, p)
        .unwrap();
    let rep = w2.check("raw");
    let i = rep.issues.iter().find(|i| i.code == "reconciled").unwrap();
    assert_eq!((i.seq, i.severity), (s, Severity::Error));
    assert!(
        i.message.contains("Xero refuses edits")
            && i.hint.contains("reclass")
            && i.hint.contains("browser")
    );
    // once the reclass journal lands, the entry is posted (the bank transaction itself never changes)
    let pkg = {
        ops::resolve(
            &mut w2.store,
            "raw",
            "human",
            Some(s),
            "reclass",
            &Guard::default(),
        )
        .unwrap();
        let rep = w2.check("raw");
        let m = w2.manifest("raw");
        let pkg = ownpurse_core::write::seal::seal(
            compile::compile(&w2.rec, &ctx(), &cfg(false), &m, &rep, "now").unwrap(),
        )
        .unwrap();
        ops::planned(
            &mut w2.store,
            "raw",
            "human",
            "p.json",
            pkg.seal.as_deref().unwrap(),
            m.content_rev,
        )
        .unwrap();
        pkg
    };
    let ownpurse_core::write::Op::CreateJournal(j) = &pkg.ops[0].op else {
        panic!()
    };
    append(
        &w2.dir,
        &roast,
        "ManualJournals",
        "mj-rc-1",
        json!({"ManualJournalID": "mj-rc-1", "Status": "POSTED",
        "DateString": "2025-10-21T00:00:00", "Narration": j.narration, "LineAmountTypes": "NoTax",
        "JournalLines": [{"AccountCode": "5020", "LineAmount": 214.77}, {"AccountCode": "6140", "LineAmount": -214.77}]}),
    );
    w2.reopen();
    assert!(w2.view("raw").overlay().unwrap().is_posted(s));
}

/// Editing a posted journal's lines needs an explicit `void-recreate` (never a default): void the original,
/// create the corrected journal; posted once the original is voided and the replacement is in the record.
#[test]
fn a_posted_journal_edit_resolves_by_void_and_recreate() {
    let mut w = world("vr");
    w.create("vr");
    let mut doc = w.doc("vr", "lab", "ManualJournals", "mj-401");
    doc["JournalLines"]["n1"]["LineAmount"] = json!("60.00");
    doc["JournalLines"]["n2"]["LineAmount"] = json!("-60.00");
    let s = w.put("vr", target("ManualJournals", "lab", Some("mj-401")), doc);
    assert!(
        w.manifest("vr").entry(s).unwrap().resolution.is_none(),
        "not a default"
    );
    let rep = w.check("vr");
    assert!(
        rep.issues
            .iter()
            .any(|i| i.code == "not_writable" && i.hint.contains("void-recreate"))
    );
    assert_eq!(
        ops::resolve(
            &mut w.store,
            "vr",
            "human",
            None,
            "void-recreate",
            &Guard::default()
        )
        .unwrap_err()
        .code,
        "usage",
        "never a manifest default"
    );
    ops::resolve(
        &mut w.store,
        "vr",
        "human",
        Some(s),
        "void-recreate",
        &Guard::default(),
    )
    .unwrap();
    let rep = w.check("vr");
    assert_eq!(rep.errors(), 0, "{:?}", rep.issues);
    let m = w.manifest("vr");
    let pkg = compile::compile(&w.rec, &ctx(), &cfg(false), &m, &rep, "now").unwrap();
    let kinds: Vec<(String, &str)> = pkg
        .ops
        .iter()
        .map(|o| (o.id.clone(), o.op.kind()))
        .collect();
    assert_eq!(
        kinds,
        vec![
            (format!("e{s}-void"), "void_journal"),
            (format!("e{s}"), "create_journal")
        ]
    );
    assert_eq!(pkg.ops[1].depends_on, vec![format!("e{s}-void")]);
    let ownpurse_core::write::Op::CreateJournal(j) = &pkg.ops[1].op else {
        panic!()
    };
    assert!(j.narration.contains("replaces mj-401"));
    assert_eq!(j.lines[0].amount, d("60.00"));
}
