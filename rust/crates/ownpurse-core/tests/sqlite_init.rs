//! Regression test for a SIGSEGV seen once in the agent tests: SQLite's first-time initialisation is not safe
//! to race on arm64 (weak memory ordering). Two threads entering `sqlite3_initialize` together can see the mutex
//! methods half-published and call a null `xMutexInit` (fault at address 0 in `sqlite3MutexInit`; confirmed as a
//! data race by ThreadSanitizer). rusqlite reaches it from every `Connection::open*`. `record::sqlite_ready()`
//! runs it once, behind a `std::sync::Once`, before any open.
//!
//! The race exists only at a process's FIRST initialisation, so this is its own test binary, and the regression
//! test runs the race in many fresh child processes: 32 threads each open the fixture record through
//! `Record::open` at the same instant. `raw_race` (ignored) is the same race without the fix, for the record.

use std::sync::{Arc, Barrier};

const CHILD: &str = "OWNPURSE_SQLITE_INIT_CHILD";

fn fixture() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/record.sqlite")
}

fn race(fixed: bool) {
    let n = 32;
    let gate = Arc::new(Barrier::new(n));
    let handles: Vec<_> = (0..n)
        .map(|_| {
            let gate = gate.clone();
            std::thread::spawn(move || {
                gate.wait();
                if fixed {
                    // the production path: Record::open runs sqlite_ready() first
                    ownpurse_core::record::Record::open(fixture()).unwrap();
                } else {
                    rusqlite::Connection::open_in_memory().unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
}

/// The child half: one fresh process, one first initialisation, 32 threads racing to it.
#[test]
#[ignore = "run by first_opens_race_safely_in_fresh_processes in a child process"]
fn child_first_open() {
    if std::env::var(CHILD).is_ok() {
        race(true);
    }
}

#[test]
fn first_opens_race_safely_in_fresh_processes() {
    let me = std::env::current_exe().unwrap();
    let runs: usize = std::env::var("OWNPURSE_SQLITE_INIT_RUNS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(25);
    for i in 0..runs {
        let out = std::process::Command::new(&me)
            .args([
                "--ignored",
                "--exact",
                "child_first_open",
                "--test-threads",
                "1",
            ])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "run {i}: {:?}\n{}",
            out.status,
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// `cargo test -p ownpurse-core --test sqlite_init raw_race -- --ignored`: the race without the fix (TSan
/// reports the data race in `sqlite3MutexInit`; a crash is rare).
#[test]
#[ignore = "reproduces the SQLite first-init race (unsafe by design)"]
fn raw_race() {
    race(false);
}
