//! `ownpurse write plan | seal | export | approve-record | approve-import | approve-check` on a temp copy of
//! the demo world. No network: these commands read the local record and append approvals to it. The TTY path
//! of approve-record runs under a real pseudo-terminal; agent sessions and non-TTYs are refused.

use serde_json::{Value, json};
use std::io::{Read, Write};
use std::os::fd::{FromRawFd, OwnedFd};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn demo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/demo")
}

/// A fresh copy of the demo world, with the accountant gate on `hold` through 2025.
fn copy(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("xa-write-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    for f in ["record.sqlite", "record.sqlite-wal", "record.sqlite-shm"] {
        if demo().join(f).exists() {
            std::fs::copy(demo().join(f), d.join(f)).unwrap();
        }
    }
    let mut cfg: Value =
        serde_json::from_str(&std::fs::read_to_string(demo().join("ownpurse.json")).unwrap())
            .unwrap();
    cfg["branches"] = json!({"accountant_gate": [{"org": "hold", "through": "2025-12-31"}]});
    std::fs::write(d.join("ownpurse.json"), cfg.to_string()).unwrap();
    d
}

/// The command with a clean environment: no agent markers, no real config or token paths.
fn cmd(dir: &Path, args: &[&str]) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_ownpurse"));
    c.args(args)
        .current_dir(dir)
        .env_clear()
        .env("PATH", "/usr/bin:/bin")
        .env("OWNPURSE_CONFIG", dir.join("ownpurse.json"))
        .env("OWNPURSE_NOW", "2026-10-07T12:00Z")
        .env("HOME", dir);
    c
}

fn run(dir: &Path, args: &[&str]) -> (i32, String) {
    let out = cmd(dir, args).output().expect("run ownpurse");
    let mut text = String::from_utf8_lossy(&out.stdout).to_string();
    text.push_str(&String::from_utf8_lossy(&out.stderr));
    (out.status.code().unwrap_or(-1), text)
}

fn json_of(dir: &Path, args: &[&str]) -> (i32, Value) {
    let mut a = args.to_vec();
    a.push("--json");
    let out = cmd(dir, &a).output().expect("run ownpurse");
    let v = serde_json::from_slice(&out.stdout)
        .unwrap_or_else(|_| json!({"raw": String::from_utf8_lossy(&out.stdout)}));
    (out.status.code().unwrap_or(-1), v)
}

fn ops(dir: &Path, v: Value) -> PathBuf {
    let p = dir.join("fixes.json");
    std::fs::write(&p, v.to_string()).unwrap();
    p
}

fn journal() -> Value {
    json!({"id": "fixes-1", "title": "Demo fixes",
           "ops": [{"kind": "create_journal", "id": "op-1", "org": "design", "date": "2026-01-31",
                    "narration": "Reclass management charges", "cash_basis": true,
                    "lines": [{"account": "6420", "amount": "263.94"}, {"account": "2420", "amount": "-263.94"}]}]})
}

/// plan + seal; the package path and its hash.
fn sealed(dir: &Path) -> (PathBuf, String) {
    ops(dir, journal());
    let (code, v) = json_of(dir, &["write", "plan", "fixes.json"]);
    assert_eq!(code, 0, "{v}");
    let pkg = dir.join("fixes.package.json");
    assert_eq!(v["package"], "fixes.package.json");
    let (code, v) = json_of(dir, &["write", "seal", "fixes.package.json"]);
    assert_eq!(code, 0, "{v}");
    (pkg, v["hash"].as_str().unwrap().to_string())
}

#[test]
fn plan_seal_export() {
    let d = copy("seal");
    let (pkg, hash) = sealed(&d);
    assert_eq!(hash.len(), 64);
    let p: Value = serde_json::from_str(&std::fs::read_to_string(&pkg).unwrap()).unwrap();
    assert_eq!(p["seal"], hash);
    let pdf = d.join(format!("fixes-{}.pdf", &hash[..12]));
    let csv = d.join(format!("fixes-{}.csv", &hash[..12]));
    assert!(std::fs::read(&pdf).unwrap().starts_with(b"%PDF-1.4"));
    assert!(std::fs::read_to_string(&csv).unwrap().contains(&hash));
    // the human form prints the review with the hash; re-sealing an unchanged package keeps the hash
    let (code, text) = run(&d, &["write", "seal", "fixes.package.json"]);
    assert_eq!(code, 0);
    assert!(
        text.contains(&format!("sha256 {hash}")) && text.contains("changed: false"),
        "{text}"
    );
    // export into another folder
    std::fs::create_dir_all(d.join("out")).unwrap();
    let (code, _) = run(
        &d,
        &["write", "export", "fixes.package.json", "--dir", "out"],
    );
    assert_eq!(code, 0);
    assert!(
        d.join("out")
            .join(format!("fixes-{}.pdf", &hash[..12]))
            .exists()
    );
    // an edited package refuses to export (exit 3)
    let mut p = p;
    p["title"] = json!("edited");
    std::fs::write(&pkg, p.to_string()).unwrap();
    let (code, v) = json_of(&d, &["write", "export", "fixes.package.json"]);
    assert_eq!((code, v["error"].as_str()), (3, Some("hash_mismatch")));
}

#[test]
fn the_freeze_gate_refuses_at_plan() {
    let d = copy("gate");
    ops(
        &d,
        json!({"id": "g-1", "ops": [{"kind": "create_journal", "id": "op-1", "org": "hold", "date": "2025-12-31",
            "narration": "accrual", "cash_basis": false,
            "lines": [{"account": "1470", "amount": "100"}, {"account": "4610", "amount": "-100"}]}]}),
    );
    let (code, text) = run(&d, &["write", "plan", "fixes.json"]);
    assert_eq!(code, 2, "{text}");
    assert!(
        text.contains("freeze_gate") && text.contains("hold is frozen through 2025-12-31"),
        "{text}"
    );
    assert!(!d.join("fixes.package.json").exists());
}

#[test]
fn approve_record_refuses_without_a_tty_or_in_an_agent_session() {
    let d = copy("noTTY");
    let (_, hash) = sealed(&d);
    std::fs::write(d.join("resp.txt"), format!("approved {hash}")).unwrap();
    let args = [
        "write",
        "approve-record",
        "fixes.package.json",
        "--response-file",
        "resp.txt",
        "--i-am-the-owner",
    ];
    let (code, text) = run(&d, &args);
    assert_eq!(code, 3, "{text}");
    assert!(
        text.contains("agent_session") && text.contains("TTY"),
        "{text}"
    );
    let out = cmd(&d, &args).env("CLAUDECODE", "1").output().unwrap();
    let text = String::from_utf8_lossy(&out.stdout);
    assert_eq!(out.status.code(), Some(3));
    assert!(text.contains("CLAUDECODE"), "{text}");
    // nothing was recorded
    let (code, v) = json_of(&d, &["write", "approve-check", "fixes.package.json"]);
    assert_eq!((code, v["error"].as_str()), (3, Some("not_approved")));
}

/// Run `args` with stdin/stdout on a pseudo-terminal, typing `typed`; returns (exit, output).
fn run_pty(dir: &Path, args: &[&str], typed: &str) -> (i32, String) {
    let (mut master, mut slave) = (0, 0);
    // SAFETY: openpty fills two fds we own from here on.
    let rc = unsafe {
        libc::openpty(
            &mut master,
            &mut slave,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert_eq!(rc, 0, "openpty");
    // SAFETY: fresh fds from openpty, each wrapped once.
    let (master, slave) = unsafe { (OwnedFd::from_raw_fd(master), OwnedFd::from_raw_fd(slave)) };
    let mut child = cmd(dir, args)
        .stdin(Stdio::from(slave.try_clone().unwrap()))
        .stdout(Stdio::from(slave.try_clone().unwrap()))
        .stderr(Stdio::from(slave))
        .spawn()
        .unwrap();
    let mut m = std::fs::File::from(master);
    let mut r = m.try_clone().unwrap();
    // read while the child runs (a pty may drop buffered output once the last slave closes)
    let reader = std::thread::spawn(move || {
        let mut out = Vec::new();
        let mut buf = [0u8; 4096];
        while let Ok(n) = r.read(&mut buf) {
            if n == 0 {
                break;
            }
            out.extend_from_slice(&buf[..n]);
        }
        String::from_utf8_lossy(&out).to_string()
    });
    m.write_all(format!("{typed}\n").as_bytes()).unwrap();
    let status = child.wait().unwrap();
    drop(m);
    let out = reader.join().unwrap();
    (status.code().unwrap_or(-1), out)
}

#[test]
fn approve_record_at_a_terminal_then_check() {
    let d = copy("tty");
    let (_, hash) = sealed(&d);
    std::fs::write(
        d.join("resp.txt"),
        format!("Approved, package {}.\n", &hash[..12]),
    )
    .unwrap();
    let args = [
        "write",
        "approve-record",
        "fixes.package.json",
        "--response-file",
        "resp.txt",
        "--i-am-the-owner",
    ];
    // a mistyped confirmation records nothing
    let (code, out) = run_pty(&d, &args, "000000000000");
    assert_eq!(code, 3, "code {code} out {out:?}");
    assert!(out.contains("hash_mismatch"), "{out}");
    let (code, out) = run_pty(&d, &args, &hash[..16]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("Type the hash"), "{out}");
    let (code, v) = json_of(&d, &["write", "approve-check", "fixes.package.json"]);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["approved"], true);
    assert_eq!(v["approvals"][0]["via"], "tty");
    // an edited, re-sealed package is a new hash: not approved
    let pkg = d.join("fixes.package.json");
    let mut p: Value = serde_json::from_str(&std::fs::read_to_string(&pkg).unwrap()).unwrap();
    p["title"] = json!("Demo fixes v2");
    std::fs::write(&pkg, p.to_string()).unwrap();
    assert_eq!(run(&d, &["write", "seal", "fixes.package.json"]).0, 0);
    let (code, _) = run(&d, &["write", "approve-check", "fixes.package.json"]);
    assert_eq!(code, 3);
}

#[test]
fn approve_import_from_the_owners_transcript() {
    let d = copy("chat");
    let (_, hash) = sealed(&d);
    let session = "demo-session-2";
    let t = d.join(format!("{session}.jsonl"));
    let entry = |uuid: &str, origin: &str, text: &str| {
        json!({"type": "user", "uuid": uuid, "sessionId": session, "userType": "external", "isSidechain": false,
               "timestamp": "2026-10-07T12:05:00Z", "origin": {"kind": origin},
               "message": {"role": "user", "content": text}})
        .to_string()
    };
    // a relayed message (task notification) quoting the hash is not the owner's
    std::fs::write(
        &t,
        entry("u1", "task-notification", &format!("approved {hash}")),
    )
    .unwrap();
    let args = [
        "write",
        "approve-import",
        "fixes.package.json",
        "--transcript",
        t.to_str().unwrap(),
        "--session",
        session,
    ];
    let (code, text) = run(&d, &args);
    assert_eq!(code, 3, "{text}");
    assert!(text.contains("task-notification"), "{text}");
    std::fs::write(
        &t,
        [
            entry("u1", "task-notification", &format!("approved {hash}")),
            entry("u2", "human", &format!("I approve {hash}")),
        ]
        .join("\n"),
    )
    .unwrap();
    let (code, v) = json_of(&d, &args);
    assert_eq!(code, 0, "{v}");
    assert_eq!(v["via"], "chat");
    assert!(v["where"].as_str().unwrap().ends_with("&message=u2"));
    let (code, v) = json_of(&d, &["write", "approve-check", "fixes.package.json"]);
    assert_eq!((code, v["approvals"][0]["via"].as_str()), (0, Some("chat")));
}
