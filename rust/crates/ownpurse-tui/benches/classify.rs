//! Classify budgets (spec classify §9) on 5,000 open lines (the demo record's lines cycled into a core
//! `Workspace` over a temp copy of the demo classification, rules suggested): a Queue frame
//! with a filter applied (key → frame) 8 ms, accept one line (store write + view update + frame) 5 ms, the
//! bulk-accept preview 50 ms. `OWNPURSE_BENCH_GATE=1` asserts each median ≤ 2× its budget.

use criterion::Criterion;
use ownpurse_core::classify::config::ConfigFile;
use ownpurse_core::classify::store::Store;
use ownpurse_core::classify::{Actor, Anchor, BatchId, Filter, Workspace, WriteCtx};
use ownpurse_tui::app::{App, Msg};
use ownpurse_tui::classify::Data;
use ownpurse_tui::keymap::Key;
use ownpurse_tui::quiet::Quiet;
use ownpurse_tui::runtime::Runtime;
use ownpurse_tui::snapshot;
use ownpurse_tui::theme;
use ratatui::layout::Rect;
use ratatui::{Terminal, TerminalOptions, Viewport};
use std::hint::black_box;
use std::time::{Duration, Instant};

type Rt = Runtime<Quiet<std::io::Sink>>;

const W: u16 = 140;
const H: u16 = 40;
const LINES: usize = 5_000;

fn app() -> App {
    let mut a = snapshot::demo_app(theme::LEDGER_DARK, false);
    let record = ownpurse_core::Record::open(&a.source.record).expect("demo record");
    let base = ownpurse_core::classify::anchor::enumerate(
        &ownpurse_core::view::View::main(&record),
        &a.source.orgs,
    )
    .expect("lines");
    let lines: Vec<_> = (0..LINES)
        .map(|i| {
            let mut l = base[i % base.len()].clone();
            l.doc_id = format!("syn-{i}");
            l.fingerprint.doc_id = l.doc_id.clone();
            l.anchor = Anchor::line(&l.org, l.doc_type, &l.doc_id, &l.line_id);
            l
        })
        .collect();
    // a private copy (never the fixtures): a fresh store, the demo config
    let paths = snapshot::classify_copy(&snapshot::demo_dir());
    std::fs::remove_file(&paths.store).expect("fresh store");
    let config = ConfigFile::load(&paths.config).expect("config");
    let store = Store::open(&paths.store).expect("store");
    let mut w = Workspace::from_parts(config, store, lines, vec![]);
    w.suggest(
        &Filter::parse("status:all").expect("filter"),
        &WriteCtx::new(Actor::rule("engine"), BatchId::new("bench")),
    )
    .expect("suggest");
    a.classify = Some(Data::new(Box::new(w)));
    a
}

fn runtime() -> Rt {
    let backend = Quiet::fixed(std::io::sink(), true, W, H);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, W, H)),
        },
    )
    .expect("terminal");
    let mut rt = Runtime::new(terminal, app());
    rt.frame().expect("frame");
    // the Queue with a filter applied (`status:open` keeps every line)
    for k in "6 / s t a t u s : o p e n enter".split_whitespace() {
        press(&mut rt, k);
    }
    rt
}

fn press(rt: &mut Rt, k: &str) {
    rt.send(Msg::Key(Key::parse(k).expect("key").event()));
    rt.frame().expect("frame");
}

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

fn queue_frame(rt: &mut Rt, i: usize) {
    press(rt, if i.is_multiple_of(2) { "j" } else { "pagedown" });
}

fn accept(rt: &mut Rt) {
    press(rt, "enter");
}

fn bulk_preview(a: &App) -> usize {
    black_box(a.cl_bulk_preview().map_or(0, |p| p.anchors.len()))
}

fn gate() {
    let mut failures = Vec::new();
    let mut report = |name: &str, med: Duration, budget: Duration| {
        println!(
            "gate {name:<22} median {:>8} µs   budget {:>6} µs   fail at {:>6} µs",
            med.as_micros(),
            budget.as_micros(),
            (budget * 2).as_micros()
        );
        if med > budget * 2 {
            failures.push(format!(
                "{name}: {} µs > 2 × {} µs",
                med.as_micros(),
                budget.as_micros()
            ));
        }
    };
    let mut rt = runtime();
    assert!(
        rt.app.cl_rows().rows.len() >= LINES - 50,
        "5,000 open lines"
    );
    let times: Vec<Duration> = (0..200)
        .map(|i| {
            let t = Instant::now();
            queue_frame(&mut rt, i);
            t.elapsed()
        })
        .collect();
    report("queue_frame_5000", median(times), Duration::from_millis(8));
    let times: Vec<Duration> = (0..100)
        .map(|_| {
            let t = Instant::now();
            accept(&mut rt);
            t.elapsed()
        })
        .collect();
    report("accept_one_line", median(times), Duration::from_millis(5));
    let mut rt = runtime();
    press(&mut rt, "A");
    let times: Vec<Duration> = (0..30)
        .map(|_| {
            let t = Instant::now();
            bulk_preview(&rt.app);
            t.elapsed()
        })
        .collect();
    report(
        "bulk_preview_5000",
        median(times),
        Duration::from_millis(50),
    );
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
    {
        let mut rt = runtime();
        let mut i = 0;
        c.bench_function("queue_frame_5000", |b| {
            b.iter(|| {
                queue_frame(&mut rt, i);
                i += 1;
            })
        });
    }
    {
        let mut rt = runtime();
        c.bench_function("accept_one_line", |b| b.iter(|| accept(&mut rt)));
    }
    {
        let mut rt = runtime();
        press(&mut rt, "A");
        c.bench_function("bulk_preview_5000", |b| b.iter(|| bulk_preview(&rt.app)));
    }
    c.final_summary();
    if std::env::var("OWNPURSE_BENCH_GATE").is_ok_and(|v| v == "1") {
        gate();
    }
}
