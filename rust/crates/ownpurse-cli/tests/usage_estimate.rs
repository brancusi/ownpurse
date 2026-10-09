//! `usage`'s calls-left per org: a figure Xero reported today (`X-DayLimit-Remaining`) shows as is; the local
//! fallback (limit − calls today) is marked as an estimate: `≈ 1000 (estimated)` in text, and in `--json`
//! the number plus `"day_remaining_estimated": true`.
//! Runs on a temp copy of the CLI fixture; the committed record is never written.

use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

const ONE: &str = "11111111-1111-1111-1111-111111111111";

fn fixture_copy() -> PathBuf {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/cli");
    let d = std::env::temp_dir().join(format!("ownpurse-cli-usage-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    for f in ["ownpurse.json", "record.sqlite"] {
        std::fs::copy(src.join(f), d.join(f)).unwrap();
    }
    d
}

fn usage(dir: &Path, json: bool) -> String {
    let mut args = vec!["--offline", "usage", "--since", "2026-01-01"];
    if json {
        args.insert(0, "--json");
    }
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(&args)
        .env("OWNPURSE_CONFIG", dir.join("ownpurse.json"))
        .env_remove("OWNPURSE_PROFILE")
        .env_remove("OWNPURSE_STORE")
        .env("HOME", "/home/demo")
        .output()
        .unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

/// Now as the record's timestamp format (`%Y-%m-%dT%H:%M:%S.%fZ`).
fn now_iso() -> String {
    let d = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let secs = d.as_secs() as i64;
    let (y, m, day) = ownpurse_core::dates::civil_from_days(secs.div_euclid(86400));
    let rem = secs.rem_euclid(86400);
    format!(
        "{y:04}-{m:02}-{day:02}T{:02}:{:02}:{:02}.000000Z",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

fn quota(dir: &Path) -> Vec<Value> {
    let v: Value = serde_json::from_str(&usage(dir, true)).unwrap();
    v["quota"].as_array().unwrap().clone()
}

#[test]
fn usage_marks_the_fallback_figure_as_an_estimate() {
    let dir = fixture_copy();
    // nothing reported today: every org's figure is the estimate (the identity row has none)
    let q = quota(&dir);
    for row in &q[..3] {
        assert_eq!(row["day_remaining"], 1000, "{row}");
        assert_eq!(row["day_remaining_estimated"], true, "{row}");
        let keys: Vec<&String> = row.as_object().unwrap().keys().collect();
        let i = keys.iter().position(|k| *k == "day_remaining").unwrap();
        assert_eq!(
            keys[i + 1],
            "day_remaining_estimated",
            "right after the number"
        );
    }
    assert!(q[3].get("day_remaining_estimated").is_none(), "{}", q[3]);
    let text = usage(&dir, false);
    assert!(
        text.contains("\n  one,0,≈ 1000 (estimated),1000,"),
        "{text}"
    );

    // Xero reported 950 left for `one` today: that row shows the figure as is, the others stay estimated
    ownpurse_core::record::sqlite_ready(); // first SQLite init on one thread
    let c = rusqlite::Connection::open(dir.join("record.sqlite")).unwrap();
    c.execute(
        "INSERT INTO http_log (at, tenant_id, method, path, params, status, duration_ms, day_remaining, \
         min_remaining, problem, op_id) VALUES (?1, ?2, 'GET', 'Accounts', NULL, 200, 100, 950, 59, NULL, NULL)",
        (now_iso(), ONE),
    )
    .unwrap();
    drop(c);
    let q = quota(&dir);
    assert_eq!(q[0]["org"], "one");
    assert_eq!(q[0]["day_remaining"], 950);
    assert!(q[0].get("day_remaining_estimated").is_none(), "{}", q[0]);
    assert_eq!(q[1]["day_remaining_estimated"], true);
    let text = usage(&dir, false);
    assert!(text.contains("\n  one,1,950,1000,"), "{text}");
    assert!(text.contains("\n  hq,0,≈ 1000 (estimated),1000,"), "{text}");
}
