//! `Workspace::open` on a synthetic large record: the demo record plus 10,000 invented documents
//! (`classify::synth`), with the demo `classify.toml` and its store. Prints the stage breakdown;
//! `OWNPURSE_BENCH_GATE=1` asserts the median open ≤ 2 × 100 ms.

use ownpurse_core::Record;
use ownpurse_core::classify::view::{ClassifyPaths, Workspace};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

fn orgs() -> Vec<(String, String)> {
    let t: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(demo().join("tenants.json")).expect("tenants"),
    )
    .expect("json");
    let mut v: Vec<(String, String)> = t
        .as_object()
        .expect("object")
        .iter()
        .map(|(a, t)| (a.clone(), t.as_str().unwrap_or("").to_string()))
        .collect();
    v.sort();
    v
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

fn main() {
    let d = std::env::temp_dir().join(format!("xa-classify-open-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("db")).expect("tmp");
    let rec_path = d.join("record.sqlite");
    let added = ownpurse_core::classify::synth::big_record(
        &demo().join("record.sqlite"),
        &rec_path,
        10_000,
    )
    .expect("synthetic record");
    std::fs::copy(demo().join("classify.toml"), d.join("classify.toml")).expect("toml");
    std::fs::copy(
        demo().join("db/classify.sqlite"),
        d.join("db/classify.sqlite"),
    )
    .expect("store");
    let paths = ClassifyPaths {
        config: d.join("classify.toml"),
        store: d.join("db/classify.sqlite"),
    };
    let orgs = orgs();
    let mut opens = vec![];
    let mut lines = 0;
    for _ in 0..9 {
        let t = Instant::now();
        let rec = Record::open(&rec_path).expect("record");
        let w =
            Workspace::open(&paths, &ownpurse_core::view::View::main(&rec), &orgs).expect("open");
        opens.push(t.elapsed());
        lines = w.lines().len();
    }
    // breakdown
    let rec = Record::open(&rec_path).expect("record");
    let t = Instant::now();
    let l =
        ownpurse_core::classify::anchor::enumerate(&ownpurse_core::view::View::main(&rec), &orgs)
            .expect("enumerate");
    let enumerate = t.elapsed();
    let t = Instant::now();
    for (_, tid) in &orgs {
        let _ = ownpurse_core::view::View::main(&rec)
            .payloads(tid, "Accounts")
            .expect("accounts");
    }
    let accounts = t.elapsed();
    let t = Instant::now();
    let mut raw = 0usize;
    for (_, tid) in &orgs {
        for e in ["BankTransactions", "ManualJournals", "Invoices"] {
            let mut st = rec
                .conn()
                .prepare("SELECT payload FROM current WHERE tenant_id=?1 AND entity=?2")
                .expect("sql");
            let n: Vec<String> = st
                .query_map([tid.as_str(), e], |r| r.get(0))
                .expect("q")
                .collect::<Result<_, _>>()
                .expect("rows");
            raw += n.len();
        }
    }
    let sql = t.elapsed();
    println!("sql only (current view, {raw} docs): {sql:?}");
    let w = Workspace::open(&paths, &ownpurse_core::view::View::main(&rec), &orgs).expect("open");
    let t = Instant::now();
    let q = w.queue(
        &ownpurse_core::classify::view::Filter::default(),
        ownpurse_core::classify::view::Sort::StaleFirst,
    );
    let c = w.counts();
    let first_queue = t.elapsed();
    println!(
        "first queue()+counts(): {first_queue:?} ({} rows, {} open)",
        q.len(),
        c.open
    );
    let m = median(opens);
    println!(
        "classify_open: {} documents added ({added} lines; {lines} lines total): open median {m:?} \
         (enumerate {enumerate:?}, accounts read {accounts:?}, {} lines)",
        10_000,
        l.len()
    );
    let _ = std::fs::remove_dir_all(&d);
    if std::env::var("OWNPURSE_BENCH_GATE").is_ok_and(|v| v == "1") {
        assert!(
            m <= Duration::from_millis(200),
            "classify_open median {m:?} > 2 × 100 ms"
        );
    }
}
