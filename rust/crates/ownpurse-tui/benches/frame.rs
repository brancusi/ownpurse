//! Frame budgets (spec §14) on the fixture record: one frame = `update` + draw + flush through the real
//! quiet backend (into a sink). `OWNPURSE_BENCH_GATE=1` (the local gate, `scripts/check.sh`) also
//! asserts each median ≤ 2× its budget: key → frame 8 ms, tab switch 8 ms, scope switch 16 ms,
//! startup → first frame 200 ms, as-of change 50 ms.

use criterion::Criterion;
use ownpurse_tui::app::{App, Msg};
use ownpurse_tui::keymap::Key;
use ownpurse_tui::model::Model;
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

fn runtime(app: App) -> Rt {
    let backend = Quiet::fixed(std::io::sink(), true, W, H);
    let terminal = Terminal::with_options(
        backend,
        TerminalOptions {
            viewport: Viewport::Fixed(Rect::new(0, 0, W, H)),
        },
    )
    .expect("terminal");
    Runtime::new(terminal, app)
}

fn startup() -> Rt {
    let source = snapshot::fixture_source();
    let now = snapshot::fixture_now();
    let model = Model::open(&source, now).expect("fixture record");
    let mut rt = runtime(App::new(model, source, theme::LEDGER_DARK, false, now));
    rt.frame().expect("frame");
    rt
}

fn press(rt: &mut Rt, k: &str) {
    rt.send(Msg::Key(Key::parse(k).expect("key").event()));
    rt.frame().expect("frame");
}

/// Alternate between as-of dates no cache has seen (each a fresh basis/IC computation).
fn as_of_change(rt: &mut Rt, i: usize) {
    let day = 1 + (i % 28);
    rt.app.ui.as_of = format!("2025-11-{day:02}");
    rt.app.invalidate();
    rt.frame().expect("frame");
}

/// Intercompany L1 opened cold: a new as-of on the Ledger (nothing cached for it), then `3` builds the
/// engine result (every pair) and draws one frame; then back to the Ledger.
fn ic_open(rt: &mut Rt, i: usize) -> Duration {
    if rt.app.ui.tab != ownpurse_tui::app::Tab::Ledger {
        press(rt, "2");
    }
    rt.app.ui.as_of = format!("2025-10-{:02}", 1 + (i % 28));
    rt.app.invalidate();
    rt.frame().expect("frame");
    let t = Instant::now();
    press(rt, "3");
    let d = t.elapsed();
    press(rt, "2");
    d
}

struct Case {
    name: &'static str,
    budget: Duration,
    /// Keys pressed once before timing (untimed).
    setup: &'static [&'static str],
    keys: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "key_to_frame",
        budget: Duration::from_millis(8),
        setup: &[],
        keys: &["j", "k"],
    },
    Case {
        name: "tab_switch",
        budget: Duration::from_millis(8),
        setup: &[],
        keys: &["2", "3", "4", "5", "1"],
    },
    Case {
        name: "scope_switch",
        budget: Duration::from_millis(16),
        setup: &[],
        keys: &["]"],
    },
    Case {
        name: "ledger_scope_switch",
        budget: Duration::from_millis(16),
        setup: &["2"],
        keys: &["]"],
    },
    Case {
        name: "drill_in_out",
        budget: Duration::from_millis(8),
        setup: &["2", "]"],
        keys: &["enter", "enter", "esc", "esc"],
    },
    Case {
        name: "ledger_basis_toggle",
        budget: Duration::from_millis(16),
        setup: &["2", "]"],
        keys: &["h"],
    },
    Case {
        name: "compare_open_close",
        budget: Duration::from_millis(30),
        setup: &["2"],
        keys: &["v", "esc"],
    },
    Case {
        name: "compare_drill_in_out",
        budget: Duration::from_millis(8),
        setup: &["2", "v"],
        keys: &["enter", "esc"],
    },
    Case {
        name: "ic_drill_in_out",
        budget: Duration::from_millis(8),
        // the second row: the first (unexplained, no one-sided items) has no item to drill into
        setup: &["3", "j"],
        keys: &["enter", "enter", "esc", "esc"],
    },
];

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

/// The gate: medians of timed frames, failing at 2× budget. Each timed press must draw exactly one frame.
/// Tab arrival (spec §14 tab switch, 8 ms): from Overview, press the tab's key; only that press is timed
/// (the way back to Overview isn't). Activity and System by name, so a regression shows which tab.
const ARRIVALS: &[(&str, &str)] = &[
    ("arrive_activity", "4"),
    ("arrive_system", "5"),
    ("arrive_activity_grouped", "4"),
];

/// One timed arrival: (time, frames the timed press drew).
fn arrival(rt: &mut Rt, name: &str, key: &str) -> (Duration, u64) {
    press(rt, "1");
    let draws = rt.draws;
    let t = Instant::now();
    press(rt, key);
    let d = t.elapsed();
    let drew = rt.draws - draws;
    if name == "arrive_activity_grouped" && !rt.app.ui.activity.grouped {
        press(rt, "G");
    }
    (d, drew)
}

fn gate() {
    let mut failures = Vec::new();
    let report = |name: &str, med: Duration, budget: Duration| -> Option<String> {
        println!(
            "gate {name:<20} median {:>8} µs   budget {:>6} µs   fail at {:>6} µs",
            med.as_micros(),
            budget.as_micros(),
            (budget * 2).as_micros()
        );
        (med > budget * 2).then(|| {
            format!(
                "{name}: {} µs > 2 × {} µs",
                med.as_micros(),
                budget.as_micros()
            )
        })
    };
    let startup_times: Vec<Duration> = (0..15)
        .map(|_| {
            let t = Instant::now();
            black_box(startup());
            t.elapsed()
        })
        .collect();
    failures.extend(report(
        "startup_first_frame",
        median(startup_times),
        Duration::from_millis(200),
    ));
    for case in CASES {
        let mut rt = startup();
        for k in case.setup {
            press(&mut rt, k);
        }
        let mut times = Vec::new();
        for i in 0..200 {
            let k = case.keys[i % case.keys.len()];
            let draws = rt.draws;
            let t = Instant::now();
            press(&mut rt, k);
            times.push(t.elapsed());
            if rt.draws != draws + 1 {
                failures.push(format!(
                    "{}: {k} drew {} frames",
                    case.name,
                    rt.draws - draws
                ));
            }
        }
        failures.extend(report(case.name, median(times), case.budget));
    }
    // As-of change on the Overview: every org's two bases + the intercompany engine, then one frame.
    let mut rt = startup();
    let mut times = Vec::new();
    for i in 0..30 {
        let t = Instant::now();
        as_of_change(&mut rt, i);
        times.push(t.elapsed());
    }
    failures.extend(report(
        "as_of_change",
        median(times),
        Duration::from_millis(50),
    ));
    let mut rt = startup();
    let times: Vec<Duration> = (0..30).map(|i| ic_open(&mut rt, i)).collect();
    failures.extend(report("ic_open", median(times), Duration::from_millis(30)));
    for &(name, key) in ARRIVALS {
        let mut rt = startup();
        let mut times = Vec::new();
        for _ in 0..200 {
            let (d, drew) = arrival(&mut rt, name, key);
            times.push(d);
            if drew != 1 {
                failures.push(format!("{name}: arrival drew {drew} frames"));
            }
        }
        failures.extend(report(name, median(times), Duration::from_millis(8)));
    }
    assert!(
        failures.is_empty(),
        "frame budgets exceeded:\n{}",
        failures.join("\n")
    );
}

fn main() {
    let mut c = Criterion::default()
        .warm_up_time(Duration::from_millis(300))
        .measurement_time(Duration::from_secs(1))
        .sample_size(20)
        .configure_from_args();
    c.bench_function("startup_first_frame", |b| b.iter(|| black_box(startup())));
    for case in CASES {
        let mut rt = startup();
        for k in case.setup {
            press(&mut rt, k);
        }
        let mut i = 0;
        c.bench_function(case.name, |b| {
            b.iter(|| {
                press(&mut rt, case.keys[i % case.keys.len()]);
                i += 1;
            })
        });
    }
    {
        let mut rt = startup();
        let mut i = 0;
        c.bench_function("as_of_change", |b| {
            b.iter(|| {
                as_of_change(&mut rt, i);
                i += 1;
            })
        });
    }
    for &(name, key) in ARRIVALS {
        let mut rt = startup();
        c.bench_function(name, |b| {
            b.iter_custom(|n| (0..n).map(|_| arrival(&mut rt, name, key).0).sum())
        });
    }
    {
        let mut rt = startup();
        let mut i = 0;
        c.bench_function("ic_open_cold", |b| {
            b.iter(|| {
                black_box(ic_open(&mut rt, i));
                i += 1;
            })
        });
    }
    c.final_summary();
    if std::env::var("OWNPURSE_BENCH_GATE").is_ok_and(|v| v == "1") {
        gate();
    }
}
