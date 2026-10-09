//! Classification budgets (spec §9) on a synthetic 5,000-line world built from the demo catalog
//! (`rust/tests/fixtures/demo/classify.toml`). `OWNPURSE_BENCH_GATE=1` (the local gate, `scripts/check.sh`) also
//! asserts each median ≤ 2× its budget: accept one line 5 ms, rule would-match 100 ms, bulk-accept preview
//! 50 ms, config reload after a hand edit 30 ms; and, measured apart (medians of 200 after a warm-up), an accept on
//! its own 5 ms and the full queue() + counts() right after a write 2 ms (the write's disk time swings with load).

use criterion::Criterion;
use ownpurse_core::classify::anchor::{DocType, Fingerprint, Line, description_hash};
use ownpurse_core::classify::config::ConfigFile;
use ownpurse_core::classify::store::Store;
use ownpurse_core::classify::view::{Filter, Sort, Workspace, WriteCtx};
use ownpurse_core::classify::{Actor, Anchor, BatchId};
use rust_decimal::Decimal;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const N: usize = 5_000;

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

const PAYEES: [&str; 8] = [
    "Hollowpine Packaging Works",
    "Cordillera Green Imports",
    "Emberline Thermal Services",
    "Brightwater Fuel Co-op",
    "Ridgeback Lab Glassware",
    "Larkspur Grocers Cooperative",
    "Corvid Registry Services",
    "Vellum & Quoin Type Foundry",
];
const WORDS: [&str; 8] = [
    "bag proofs for the Harvest Blend",
    "washed lot, 22 sacks",
    "afterburner catalyst service",
    "van fuel",
    "cupping bowls",
    "wholesale beans",
    "annual return filing",
    "wayfinding typeface licence",
];
const ORGS: [&str; 4] = ["roast", "hold", "design", "lab"];
const ACCTS: [&str; 6] = ["5310", "5020", "6140", "6520", "4010", "1630"];

fn lines() -> Vec<Line> {
    (0..N)
        .map(|i| {
            let org = ORGS[i % 4];
            let id = format!("bt-{i}");
            let cents = 1_013 + (i as i64 * 7_919) % 900_000;
            let amount = Decimal::new(if i % 5 == 0 { cents } else { -cents }, 2);
            let date = format!("2025-{:02}-{:02}", 1 + i % 12, 1 + i % 28);
            let description = format!("{} #{i}", WORDS[i % 8]);
            let acct = ACCTS[i % 6];
            Line {
                anchor: Anchor::line(org, DocType::BankTransaction, &id, "L1"),
                org: org.into(),
                tenant_id: "t".into(),
                doc_type: DocType::BankTransaction,
                doc_id: id.clone(),
                line_id: "L1".into(),
                date: date.clone(),
                payee: PAYEES[i % 8].into(),
                description: description.clone(),
                amount,
                account_code: acct.into(),
                account_name: String::new(),
                reconciled: Some(true),
                reference: String::new(),
                status: "AUTHORISED".into(),
                fingerprint: Fingerprint {
                    org: org.into(),
                    doc_id: id,
                    line_id: "L1".into(),
                    account: acct.into(),
                    amount: amount.to_string(),
                    date,
                    description_hash: description_hash(&description),
                },
                counterparty_org: None,
                bank_transfer: false,
            }
        })
        .collect()
}

fn workspace() -> Workspace {
    let d = std::env::temp_dir().join(format!("xa-classify-bench-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(d.join("db")).expect("tmp");
    std::fs::copy(demo().join("classify.toml"), d.join("classify.toml")).expect("toml");
    let config = ConfigFile::load(&d.join("classify.toml")).expect("config");
    let store = Store::open(&d.join("db/classify.sqlite")).expect("store");
    let mut w = Workspace::from_parts(config, store, lines(), vec![]);
    w.suggest(
        &Filter::parse("status:all").expect("filter"),
        &WriteCtx::new(Actor::rule("engine"), BatchId::new("bench")),
    )
    .expect("suggest");
    w
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

fn report(name: &str, m: Duration, budget: Duration) -> Option<String> {
    println!("{name}: median {m:?} (budget {budget:?})");
    (m > budget * 2).then(|| format!("{name}: median {m:?} > 2 × {budget:?}"))
}

fn time<F: FnMut(usize)>(n: usize, mut f: F) -> Duration {
    median(
        (0..n)
            .map(|i| {
                let t = Instant::now();
                f(i);
                t.elapsed()
            })
            .collect(),
    )
}

/// Gate medians: this many timed runs, after `WARM_UP` untimed ones.
const GATE_RUNS: usize = 200;
const WARM_UP: usize = 30;

fn hand_edit(w: &mut Workspace, i: usize) {
    let p = w.config_file().path.clone();
    let mut t = std::fs::read_to_string(&p).expect("read");
    t.push_str(&format!("# hand edit {i}\n"));
    std::fs::write(&p, t).expect("write");
    black_box(w.reload_config());
}

fn gate() {
    let mut w = workspace();
    let human = WriteCtx::new(Actor::human(), BatchId::new("gate"));
    let anchors: Vec<Anchor> = w.lines().iter().map(|l| l.anchor.clone()).collect();
    let mut failures = vec![];
    let accept = time(200, |i| {
        black_box(w.accept(&anchors[i], &human).expect("accept"));
    });
    failures.extend(report("accept", accept, Duration::from_millis(5)));
    let rule = w.config().rule("r-hollowpine").expect("rule").clone();
    let wm = time(20, |_| {
        black_box(w.would_match(&rule));
    });
    failures.extend(report("would_match_5000", wm, Duration::from_millis(100)));
    let f = Filter::parse("status:all").expect("filter");
    let th: Decimal = "0.5".parse().expect("threshold");
    let bulk = time(20, |_| {
        black_box(w.bulk_preview(&f, th));
    });
    failures.extend(report("bulk_preview_5000", bulk, Duration::from_millis(50)));
    // accept (a store write) and the cached queue + counts are gated separately: the write's disk time
    // swings with machine load and must not hide (or fake) a change in the cached read path
    let all = Filter::parse("status:all").expect("filter");
    let mut k = 0;
    let warm = |w: &mut Workspace, k: &mut usize| {
        for _ in 0..WARM_UP {
            let _ = black_box(w.accept(&anchors[(*k + 300) % N], &human));
            *k += 1;
            black_box(w.queue(&all, Sort::AmountDesc));
            black_box(w.counts());
        }
    };
    warm(&mut w, &mut k);
    let acc = time(GATE_RUNS, |_| {
        let _ = black_box(w.accept(&anchors[(k + 300) % N], &human));
        k += 1;
    });
    failures.extend(report("accept_5000", acc, Duration::from_millis(5)));
    // each queue + counts read follows an accept (untimed), so it brings the cache up to a new revision
    let mut qc = Vec::with_capacity(GATE_RUNS);
    for _ in 0..GATE_RUNS {
        let _ = black_box(w.accept(&anchors[(k + 300) % N], &human));
        k += 1;
        let t = Instant::now();
        black_box(w.queue(&all, Sort::AmountDesc));
        black_box(w.counts());
        qc.push(t.elapsed());
    }
    failures.extend(report(
        "queue_counts_after_accept_5000",
        median(qc),
        Duration::from_millis(2),
    ));
    let reload = time(20, |i| hand_edit(&mut w, i));
    failures.extend(report("config_reload", reload, Duration::from_millis(30)));
    assert!(
        failures.is_empty(),
        "classify budgets exceeded:\n{}",
        failures.join("\n")
    );
}

fn main() {
    let mut c = Criterion::default()
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1))
        .sample_size(20)
        .configure_from_args();
    let mut w = workspace();
    let human = WriteCtx::new(Actor::human(), BatchId::new("bench"));
    let anchors: Vec<Anchor> = w.lines().iter().map(|l| l.anchor.clone()).collect();
    let mut i = 0;
    c.bench_function("classify_accept", |b| {
        b.iter(|| {
            let _ = black_box(w.accept(&anchors[i % N], &human));
            i += 1;
        })
    });
    let rule = w.config().rule("r-hollowpine").expect("rule").clone();
    c.bench_function("classify_would_match_5000", |b| {
        b.iter(|| black_box(w.would_match(&rule)))
    });
    let f = Filter::parse("status:all").expect("filter");
    let th: Decimal = "0.5".parse().expect("threshold");
    c.bench_function("classify_bulk_preview_5000", |b| {
        b.iter(|| black_box(w.bulk_preview(&f, th)))
    });
    let all = Filter::parse("status:all").expect("filter");
    c.bench_function("classify_queue_counts_5000", |b| {
        b.iter(|| {
            black_box(w.queue(&all, Sort::AmountDesc));
            black_box(w.counts())
        })
    });
    c.bench_function("classify_accept_then_queue_counts_5000", |b| {
        b.iter(|| {
            let _ = black_box(w.accept(&anchors[i % N], &human));
            i += 1;
            black_box(w.queue(&all, Sort::AmountDesc));
            black_box(w.counts())
        })
    });
    c.final_summary();
    drop(w);
    if std::env::var("OWNPURSE_BENCH_GATE").is_ok_and(|v| v == "1") {
        gate();
    }
}
