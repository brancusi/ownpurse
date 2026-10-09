//! Explainer visualisations on the demo world (record.sqlite, as of 2025-12-31): every kind computed through
//! the same view, ledger, basis and intercompany engine as the books, to the cent.

use ownpurse_core::Record;
use ownpurse_core::intercompany::IcContext;
use ownpurse_core::viz::{self, Extra, FlowBasis, Model, Spec, Tone};
use rust_decimal::Decimal;
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

fn config() -> Value {
    serde_json::from_str(&std::fs::read_to_string(fixtures().join("demo/ownpurse.json")).unwrap())
        .unwrap()
}

fn tenants(cfg: &Value) -> HashMap<String, String> {
    cfg["profiles"]["demo"]["orgs"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap().to_string()))
        .collect()
}

fn d(s: &str) -> Decimal {
    Decimal::from_str(s).unwrap()
}

fn run(spec: &str) -> Model {
    let rec = Record::open(fixtures().join("demo/record.sqlite")).unwrap();
    let cfg = config();
    let ic = serde_json::json!({"parties": cfg["parties"], "intercompany": cfg["intercompany"]});
    let ctx = IcContext::new(
        ownpurse_core::view::View::main(&rec),
        tenants(&cfg),
        &ic,
        "2025-12-31",
    );
    let extra = Extra {
        today: "2026-10-05".into(),
        base_dir: Some(fixtures()),
        effects: None,
        orgs: ["roast", "hold", "design", "lab"]
            .iter()
            .map(|a| (a.to_string(), a.to_string()))
            .collect(),
    };
    viz::compute(&ctx, &Spec::parse(spec).unwrap(), &extra).unwrap()
}

#[test]
fn bridge_fully_explained() {
    let Model::Bridge(b) = run(r#"{"kind":"bridge","pair":"hold-design"}"#) else {
        panic!()
    };
    assert_eq!(b.status, "bridged");
    assert_eq!(b.start.amount, d("418.27"));
    assert_eq!(b.items.len(), 1);
    assert_eq!(b.items[0].amount, d("418.27"));
    assert_eq!(b.items[0].label, "Annual return filing for the parent");
    assert_eq!(b.items[0].sources[0].id, "bt-307");
    assert_eq!(b.items[0].id, "item/1");
    assert_eq!(b.residual, d("0.00"));
    assert_eq!(b.end.amount, Decimal::ZERO);
    assert_eq!(b.a_says, "hold's books: design owes hold 4,670.51");
    assert_eq!(b.b_says, "design's books: design owes hold 4,252.24");
}

#[test]
fn bridge_with_residual() {
    let Model::Bridge(b) = run(r#"{"kind":"bridge","pair":"design-roast"}"#) else {
        panic!()
    };
    assert_eq!(b.status, "unexplained");
    assert_eq!(b.start.amount, d("263.94"));
    assert!(b.items.is_empty());
    assert_eq!(b.residual, d("263.94"));
    assert_eq!(b.end.label, "Unexplained");
    // the waterfall closes: start − items = residual
    let explained: Decimal = b.items.iter().map(|i| i.amount).sum();
    assert_eq!(b.start.amount - explained, b.residual);
}

#[test]
fn bridge_matches_and_unknown_pair() {
    let Model::Bridge(b) = run(r#"{"kind":"bridge","pair":"hold-roast"}"#) else {
        panic!()
    };
    assert_eq!((b.status.as_str(), b.residual), ("matches", Decimal::ZERO));
    let rec = Record::open(fixtures().join("demo/record.sqlite")).unwrap();
    let cfg = config();
    let ctx = IcContext::new(
        ownpurse_core::view::View::main(&rec),
        tenants(&cfg),
        &serde_json::json!({"intercompany": cfg["intercompany"]}),
        "2025-12-31",
    );
    let e = viz::compute(
        &ctx,
        &Spec::parse(r#"{"kind":"bridge","pair":"nope"}"#).unwrap(),
        &Extra::default(),
    )
    .unwrap_err();
    assert_eq!(e.code, "not_found");
}

#[test]
fn flow_owed_and_paid() {
    let Model::Flow(f) = run(r#"{"kind":"flow"}"#) else {
        panic!()
    };
    assert_eq!(f.basis, FlowBasis::Owed);
    assert_eq!(f.nodes.len(), 4);
    assert_eq!(f.nodes[0].id, "node/roast");
    let e = f.edges.iter().find(|e| e.pair == "hold-roast").unwrap();
    assert_eq!((e.from.as_str(), e.to.as_str()), ("roast", "hold"));
    assert_eq!(e.words, "roast owes hold 15,568.80");
    assert_eq!(e.note, None);
    assert_eq!(e.id, "edge/hold-roast");
    let e = f.edges.iter().find(|e| e.pair == "hold-design").unwrap();
    assert!(e.note.as_deref().unwrap().contains("a gap of 418.27"));
    let Model::Flow(p) = run(r#"{"kind":"flow","basis":"paid","orgs":["hold","roast"]}"#) else {
        panic!()
    };
    assert_eq!(p.nodes.len(), 2);
    assert!(p.edges.iter().all(|e| e.pair == "hold-roast"));
    let lent = p.edges.iter().find(|e| e.from == "hold").unwrap();
    assert!(lent.amount.unwrap() >= d("18437.62"));
    let repaid = p.edges.iter().find(|e| e.from == "roast").unwrap();
    assert!(repaid.amount.unwrap() >= d("3281.19"));
}

#[test]
fn t_account_balances() {
    let Model::TAccount(t) = run(r#"{"kind":"t_account","org":"hold","account":"1470","top":1}"#)
    else {
        panic!()
    };
    assert_eq!(
        (t.from.as_str(), t.to.as_str()),
        ("2025-01-01", "2025-12-31")
    );
    assert_eq!(t.code, "1470");
    assert_eq!(
        t.opening + t.debit_total - t.credit_total,
        t.closing,
        "opening + debits − credits = closing"
    );
    assert_eq!(t.closing, d("4670.51"), "the pair's a side at 2025-12-31");
    assert_eq!(t.debits.len(), 1);
    let shown: Decimal = t.debits.iter().map(|e| e.amount).sum::<Decimal>()
        + t.debits_more.as_ref().map_or(Decimal::ZERO, |m| m.amount);
    assert_eq!(shown, t.debit_total);
    assert_eq!(t.debit_count, 2);
    assert_eq!(t.debits_more.as_ref().unwrap().label, "1 more debit");
    // unknown account
    let rec = Record::open(fixtures().join("demo/record.sqlite")).unwrap();
    let cfg = config();
    let ctx = IcContext::new(
        ownpurse_core::view::View::main(&rec),
        tenants(&cfg),
        &serde_json::json!({}),
        "2025-12-31",
    );
    let e = viz::compute(
        &ctx,
        &Spec::parse(r#"{"kind":"t_account","org":"hold","account":"9999"}"#).unwrap(),
        &Extra::default(),
    )
    .unwrap_err();
    assert_eq!(e.code, "not_found");
}

#[test]
fn timeline_months() {
    let Model::Timeline(t) = run(r#"{"kind":"timeline","org":"hold","account":"1470"}"#) else {
        panic!()
    };
    assert_eq!(t.months.len(), 12);
    assert_eq!(t.months[0].id, "month/2025-01");
    assert_eq!(t.months[11].balance, d("4670.51"));
    let moved: Decimal = t.months.iter().map(|m| m.movement).sum();
    assert_eq!(t.opening + moved, t.months[11].balance);
    assert!(t.months.iter().any(|m| m.journals > 0), "fees are journals");
}

#[test]
fn checklist_demo_file() {
    let Model::Checklist(c) = run(r#"{"kind":"checklist","file":"viz/close-status-demo.json"}"#)
    else {
        panic!()
    };
    let titles: Vec<&str> = c.groups.iter().map(|g| g.title.as_str()).collect();
    assert_eq!(
        titles,
        [
            "Deadlines",
            "Waiting on Maren",
            "Waiting on Tobias",
            "Agent work",
            "Postings: posted",
            "Postings: prepared",
            "Postings: parked",
            "Postings: superseded"
        ]
    );
    let n: usize = c.groups.iter().map(|g| g.items.len()).sum();
    assert_eq!(n, 4 + 5 + 3 + 4, "every item kept");
    assert_eq!(c.groups[0].items[0].status, "10 days left");
    assert_eq!(c.groups[0].items[1].status, "filed 2026-09-12");
    assert_eq!(c.groups[0].items[1].tone, Tone::Done);
    assert_eq!(
        c.groups[0].items[2].status, "28 days left",
        "filed: false keeps the days"
    );
    assert_eq!(c.groups[0].items[3].status, "due 173 days ago");
    assert_eq!(c.groups[0].items[3].tone, Tone::Late);
    let w = &c.groups[2].items[1];
    assert_eq!(
        (w.status.as_str(), w.tone),
        ("waiting_on_docs", Tone::Unknown)
    );
    assert_eq!(c.also, ["crypto", "key_balances"]);
}

#[test]
fn deterministic_json() {
    let a = serde_json::to_string(&run(
        r#"{"kind":"t_account","org":"roast","account":"1010"}"#,
    ))
    .unwrap();
    let b = serde_json::to_string(&run(
        r#"{"kind":"t_account","org":"roast","account":"1010"}"#,
    ))
    .unwrap();
    assert_eq!(a, b);
    assert!(a.contains("\"kind\":\"t_account\""));
}

#[test]
fn before_after_from_a_manifest() {
    use ownpurse_core::manifest::ops::{self, Guard, JLine, Target};
    use ownpurse_core::manifest::{Refs, Store, effects};
    use ownpurse_core::view::Ctx;
    let dir = std::env::temp_dir().join(format!("xa-viz-ba-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for ext in ["", "-wal", "-shm"] {
        let f = fixtures().join(format!("demo/record.sqlite{ext}"));
        if f.exists() {
            std::fs::copy(&f, dir.join(format!("record.sqlite{ext}"))).unwrap();
        }
    }
    let rec = Record::open(dir.join("record.sqlite")).unwrap();
    let mut orgs: Vec<(String, String)> = tenants(&config()).into_iter().collect();
    orgs.sort();
    let ctx = Ctx { orgs, gate: vec![] };
    let mut store = Store::open(&dir.join("db/manifests.sqlite")).unwrap();
    ops::create(&mut store, &rec, &ctx, "human", "fees", "test").unwrap();
    let lines = [
        JLine {
            account: "1470".into(),
            description: String::new(),
            amount: d("100.13"),
        },
        JLine {
            account: "4610".into(),
            description: String::new(),
            amount: d("-100.13"),
        },
    ];
    let after = ops::journal_doc("2025-12-20", "fee true-up", true, &lines);
    let t = Target {
        entity: "ManualJournals".into(),
        org: "hold".into(),
        id: None,
    };
    ops::add(
        &mut store,
        &rec,
        &ctx,
        "fees",
        "human",
        &t,
        &after,
        "",
        &Refs::default(),
        None,
        &Guard::default(),
    )
    .unwrap();
    let m = store.manifest("fees").unwrap();
    let diff = effects::diff(&rec, &ctx, &m, "2025-12-31", None, &Default::default()).unwrap();
    let ba = viz::before_after(&diff, "hold").unwrap();
    assert_eq!(ba.manifest, "fees");
    let l = ba.lines.iter().find(|l| l.code == "1470").unwrap();
    assert_eq!(l.id, "line/1470");
    assert_eq!(l.delta, d("100.13"));
    assert_eq!(l.after - l.before, l.delta);
    let rev = ba.lines.iter().find(|l| l.code == "4610").unwrap();
    assert_eq!(rev.delta, d("-100.13"));
    let assets = ba.totals.iter().find(|t| t.id == "total/assets").unwrap();
    assert_eq!(assets.delta, d("100.13"));
    let net = ba.totals.iter().find(|t| t.id == "total/net").unwrap();
    assert_eq!((net.label.as_str(), net.delta), ("Net profit", d("100.13")));
    let _ = std::fs::remove_dir_all(&dir);
}
