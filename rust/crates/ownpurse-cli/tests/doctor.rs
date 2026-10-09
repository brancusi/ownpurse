//! `doctor` is read-only, prints a fix for every problem, never prints the home folder or token contents,
//! and exits 1 only when a check fails.

use std::path::{Path, PathBuf};
use std::process::Command;

const SECRET: &str = "never-print-this-token-value";

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

/// A temp project with the demo config and record, a fake HOME and a PATH holding the two binaries.
struct Sandbox {
    dir: PathBuf,
}

impl Sandbox {
    fn new(name: &str, token_mode: Option<u32>) -> Sandbox {
        let dir =
            std::env::temp_dir().join(format!("ownpurse-doctor-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("project/tokens")).unwrap();
        std::fs::create_dir_all(dir.join("bin")).unwrap();
        std::fs::copy(
            demo().join("ownpurse.json"),
            dir.join("project/ownpurse.json"),
        )
        .unwrap();
        std::fs::copy(
            demo().join("record.sqlite"),
            dir.join("project/record.sqlite"),
        )
        .unwrap();
        for b in ["ownpurse", "ownpurse-tui"] {
            std::fs::write(dir.join("bin").join(b), "#!/bin/sh\n").unwrap();
        }
        if let Some(m) = token_mode {
            let p = dir.join("project/tokens/demo.json");
            std::fs::write(&p, format!("{{\"access_token\": \"{SECRET}\"}}")).unwrap();
            set_mode(&p, m);
        }
        Sandbox { dir }
    }

    fn run(&self, cwd: &str, extra: &[&str]) -> (i32, String) {
        let mut args = vec!["doctor"];
        args.extend_from_slice(extra);
        let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
            .args(&args)
            .current_dir(self.dir.join(cwd))
            .env("HOME", &self.dir)
            .env("PATH", self.dir.join("bin"))
            .env_remove("OWNPURSE_CONFIG")
            .env_remove("OWNPURSE_PROFILE")
            .env_remove("OWNPURSE_STORE")
            .env_remove("OWNPURSE_CLIENT_ID")
            .env_remove("XDG_CONFIG_HOME")
            .env_remove("XDG_DATA_HOME")
            .output()
            .unwrap();
        let text = String::from_utf8(out.stdout).unwrap() + &String::from_utf8(out.stderr).unwrap();
        assert!(!text.contains(SECRET), "token contents leaked:\n{text}");
        assert!(
            !text.contains(&self.dir.display().to_string()),
            "home folder printed:\n{text}"
        );
        (out.status.code().unwrap_or(-1), text)
    }
}

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(unix)]
fn set_mode(p: &Path, m: u32) {
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(p, std::fs::Permissions::from_mode(m)).unwrap();
}

#[test]
fn all_good_exits_zero_and_never_prints_a_token_value() {
    let s = Sandbox::new("ok", Some(0o600));
    let (code, text) = s.run("project", &[]);
    assert_eq!(code, 0, "{text}");
    assert!(text.contains("result: ok"), "{text}");
    assert!(text.contains("mode 600 (no token value read)"), "{text}");
    assert!(text.contains("~/project/ownpurse.json"), "{text}");
    assert!(text.contains("profile demo · 4 org(s)"), "{text}");
}

#[test]
fn token_readable_by_others_fails_with_chmod_fix() {
    let s = Sandbox::new("mode", Some(0o644));
    let (code, text) = s.run("project", &[]);
    assert_eq!(code, 1, "{text}");
    assert!(text.contains("mode 644: readable by others"), "{text}");
    assert!(
        text.contains("chmod 600 ~/project/tokens/demo.json"),
        "{text}"
    );
}

#[test]
fn missing_token_and_record_warn_with_the_fix_commands() {
    let s = Sandbox::new("warn", None);
    std::fs::remove_file(s.dir.join("project/record.sqlite")).unwrap();
    let (code, text) = s.run("project", &[]);
    assert_eq!(code, 0, "warnings never fail: {text}");
    assert!(text.contains("result: ok with warnings"), "{text}");
    assert!(text.contains("`ownpurse login`"), "{text}");
    assert!(text.contains("`ownpurse sync all`"), "{text}");
    assert!(
        !s.dir.join("project/record.sqlite").exists(),
        "doctor never creates a record"
    );
}

#[test]
fn no_config_and_nothing_on_path_fail_with_fixes() {
    let s = Sandbox::new("bare", None);
    std::fs::remove_file(s.dir.join("bin/ownpurse")).unwrap();
    let (code, text) = s.run("bin", &["--json"]);
    assert_eq!(code, 1, "{text}");
    let first: serde_json::Value = serde_json::Deserializer::from_str(&text)
        .into_iter::<serde_json::Value>()
        .next()
        .unwrap()
        .unwrap();
    assert_eq!(first["result"], "fail");
    let checks = first["checks"].as_array().unwrap();
    let get = |n: &str| checks.iter().find(|c| c["check"] == n).unwrap().clone();
    assert_eq!(get("ownpurse on PATH")["status"], "fail");
    assert!(
        get("ownpurse on PATH")["fix"]
            .as_str()
            .unwrap()
            .contains(".local/bin")
    );
    assert_eq!(get("config")["status"], "fail");
    assert!(
        get("config")["fix"]
            .as_str()
            .unwrap()
            .contains("ownpurse.example.json")
    );
    assert!(
        get("config")["detail"]
            .as_str()
            .unwrap()
            .contains("in ~/bin and its parent folders"),
        "{text}"
    );
    assert!(
        checks
            .iter()
            .all(|c| c["status"] == "ok" || !c["fix"].as_str().unwrap().is_empty())
    );
}

#[test]
fn doctor_lists_granted_scope_names_never_token_values() {
    let sb = Sandbox::new("scopes", Some(0o600));
    let p = sb.dir.join("project/tokens/demo.json");
    std::fs::write(
        &p,
        format!(
            "{{\"access_token\": \"{SECRET}\", \"refresh_token\": \"{SECRET}\", \"id_token\": \"{SECRET}\", \
             \"scope\": \"openid offline_access accounting.settings.read accounting.manualjournals {SECRET}x\"}}"
        ),
    )
    .unwrap();
    set_mode(&p, 0o600);
    let (_, text) = sb.run("project", &["--json"]);
    let v: serde_json::Value = serde_json::from_str(&text[..text.rfind('}').unwrap() + 1]).unwrap();
    let rows = v["checks"].as_array().unwrap();
    let scopes = rows
        .iter()
        .find(|r| r["check"] == "scopes")
        .expect("a scopes check");
    assert_eq!(
        scopes["detail"],
        "read: openid offline_access accounting.settings.read · write: accounting.manualjournals"
    );
    // a profile that doesn't write has no write-scope check
    assert!(rows.iter().all(|r| r["check"] != "write scopes"));
    // with allow_writes: the missing write scopes are named, with the fix
    let cfg = sb.dir.join("project/ownpurse.json");
    let mut c: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&cfg).unwrap()).unwrap();
    c["profiles"]["demo"]["allow_writes"] = serde_json::json!(true);
    std::fs::write(&cfg, c.to_string()).unwrap();
    let (_, text) = sb.run("project", &["--json"]);
    let v: serde_json::Value = serde_json::from_str(&text[..text.rfind('}').unwrap() + 1]).unwrap();
    let w = v["checks"]
        .as_array()
        .unwrap()
        .iter()
        .find(|r| r["check"] == "write scopes")
        .expect("a write scopes check")
        .clone();
    assert_eq!(w["status"], "warn");
    assert_eq!(
        w["detail"],
        "allow_writes is set but not granted: accounting.banktransactions accounting.attachments"
    );
    assert!(
        w["fix"]
            .as_str()
            .unwrap()
            .contains("ownpurse login --write")
    );
}

#[test]
fn login_write_is_refused_for_a_profile_without_allow_writes() {
    let sb = Sandbox::new("loginwrite", Some(0o600));
    let out = Command::new(env!("CARGO_BIN_EXE_ownpurse"))
        .args(["login", "--write", "--no-browser", "--json"])
        .current_dir(sb.dir.join("project"))
        .env("HOME", &sb.dir)
        .env("PATH", sb.dir.join("bin"))
        .env_remove("OWNPURSE_CONFIG")
        .output()
        .unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(2), "{text}");
    assert!(
        text.contains("writes_disabled") && !text.contains("http"),
        "refused before any URL: {text}"
    );
}

#[test]
fn doctor_warns_about_report_dates_far_in_the_future() {
    let sb = Sandbox::new("futuredates", Some(0o600));
    let c = rusqlite::Connection::open(sb.dir.join("project/record.sqlite")).unwrap();
    c.execute(
        "INSERT INTO reports(tenant_id, report, params, fetched_at, content_hash, payload) VALUES ('t', 'TrialBalance', ?1, '2026-10-07T12:00:00Z', 'x', '{}')",
        [r#"{"date":"9999-12-31","phase":"before","xa_package":"dabc7522"}"#],
    )
    .unwrap();
    drop(c);
    let (_, text) = sb.run("project", &[]);
    assert!(
        text.contains("report dates,warn,1 cached report(s) dated after today + 1 year"),
        "{text}"
    );
}
