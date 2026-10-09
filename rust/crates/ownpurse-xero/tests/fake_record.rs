//! The TUI fake CLI's record helper (`ownpurse-fake-record`) appends what a real run writes, through the
//! production Writer, and `Writer::verify()` still proves the chain afterwards.

mod common;

use common::temp_dir;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The chain's entry count, proven by `Writer::verify`.
fn verified(record: &Path) -> i64 {
    ownpurse_xero::store::Writer::open(record)
        .unwrap()
        .verify()
        .unwrap()
        .0
}
const DESIGN: &str = "de519000-0000-4000-8000-0000000000c3";

fn copy_demo(name: &str) -> PathBuf {
    let d = temp_dir(name);
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo");
    for ext in ["", "-wal", "-shm"] {
        let f = src.join(format!("record.sqlite{ext}"));
        if f.exists() {
            std::fs::copy(&f, d.join(format!("record.sqlite{ext}"))).unwrap();
        }
    }
    d.join("record.sqlite")
}

fn run(record: &Path, scenario: &str) {
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse-fake-record"))
        .args([
            record.to_str().unwrap(),
            scenario,
            DESIGN,
            "2026-10-05T09:46:00Z",
        ])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn q(record: &Path, sql: &str) -> String {
    ownpurse_core::record::sqlite_ready(); // first SQLite init on one thread
    let c = rusqlite::Connection::open(record).unwrap();
    c.query_row(sql, [], |r| {
        r.get::<_, rusqlite::types::Value>(0)
            .map(|v| format!("{v:?}"))
    })
    .unwrap()
}

#[test]
fn scenarios_append_real_rows_and_the_chain_verifies() {
    // ok: 15 calls counting the reported figure down 938 → 923, 14 runs finishing at the pinned time, 3 versions
    let r = copy_demo("fake-ok");
    let before = q(&r, "SELECT COUNT(*) FROM http_log");
    run(&r, "ok");
    assert_eq!(before, "Integer(142)");
    assert_eq!(q(&r, "SELECT COUNT(*) FROM http_log"), "Integer(157)");
    assert_eq!(
        q(
            &r,
            &format!(
                "SELECT day_remaining FROM http_log WHERE tenant_id='{DESIGN}' ORDER BY id DESC LIMIT 1"
            )
        ),
        "Integer(923)"
    );
    assert_eq!(
        q(
            &r,
            "SELECT COUNT(*) FROM sync_runs WHERE finished_at='2026-10-05T09:46:00.000000Z' AND outcome='ok'"
        ),
        "Integer(14)"
    );
    assert_eq!(
        q(
            &r,
            "SELECT SUM(fetched) || '/' || SUM(new_versions) || '/' || SUM(calls) FROM sync_runs \
             WHERE finished_at='2026-10-05T09:46:00.000000Z'"
        ),
        "Text(\"25/3/15\")"
    );
    assert_eq!(
        q(
            &r,
            "SELECT COUNT(*) FROM versions WHERE observed_at='2026-10-05T09:46:00.000000Z'"
        ),
        "Integer(3)"
    );
    assert_eq!(verified(&r), 306);

    // limit: one 429 (day) reporting 0 left, and the failed run
    let r = copy_demo("fake-limit");
    run(&r, "limit");
    assert_eq!(
        q(
            &r,
            "SELECT status || '/' || day_remaining || '/' || problem FROM http_log ORDER BY id DESC LIMIT 1"
        ),
        "Text(\"429/0/day\")"
    );
    assert_eq!(
        q(
            &r,
            "SELECT detail FROM sync_runs WHERE outcome='error' ORDER BY rowid DESC LIMIT 1"
        ),
        "Text(\"{\\\"error\\\": \\\"BudgetExhausted\\\", \\\"message\\\": \\\"Xero daily call limit reached for this organisation.\\\"}\")"
    );
    assert_eq!(verified(&r), 276);

    // error: the 401, its retry, and the failed run
    let r = copy_demo("fake-error");
    run(&r, "error");
    assert_eq!(
        q(
            &r,
            "SELECT COUNT(*) FROM http_log WHERE status=401 AND day_remaining IS NULL"
        ),
        "Integer(2)"
    );
    assert_eq!(verified(&r), 277);
}

#[test]
fn refuses_a_record_outside_the_temp_folder() {
    let committed =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo/record.sqlite");
    let bytes = std::fs::read(&committed).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse-fake-record"))
        .args([
            committed.to_str().unwrap(),
            "ok",
            DESIGN,
            "2026-10-05T09:46:00Z",
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert_eq!(
        std::fs::read(&committed).unwrap(),
        bytes,
        "the committed fixture is untouched"
    );
}
