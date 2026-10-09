//! The network commands through the real binary, against a loopback fake Xero only: `sync`, `orgs`, `login`
//! (with a simulated browser), `--offline` refusals, `login --write` refused, and only the token URL ever
//! receives a non-GET.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

const ROAST: &str = "c0ffee00-0000-4000-8000-0000000000a1";

type Seen = Arc<Mutex<Vec<(String, String)>>>;
/// The fake's behaviour for `/Accounts`: `normal`, `throttle_once` (one 429 minute-limit first), `slow` (1.5 s).
type Mode = Arc<Mutex<&'static str>>;

fn fake() -> (String, Seen, Mode) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}", listener.local_addr().unwrap());
    let seen: Seen = Arc::new(Mutex::new(Vec::new()));
    let s2 = seen.clone();
    let mode: Mode = Arc::new(Mutex::new("normal"));
    let m2 = mode.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut first = String::new();
            if reader.read_line(&mut first).unwrap_or(0) == 0 {
                continue;
            }
            let mut len = 0usize;
            loop {
                let mut l = String::new();
                if reader.read_line(&mut l).unwrap_or(0) == 0 || l.trim().is_empty() {
                    break;
                }
                if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
                    len = v.trim().parse().unwrap_or(0);
                }
            }
            let mut body = vec![0u8; len];
            let _ = reader.read_exact(&mut body);
            let mut it = first.split_whitespace();
            let (method, target) = (
                it.next().unwrap_or("").to_string(),
                it.next().unwrap_or("").to_string(),
            );
            let path = target.split('?').next().unwrap_or("").to_string();
            s2.lock().unwrap().push((method.clone(), path.clone()));
            let mut status = "200 OK";
            let mut extra = String::new();
            if path == "/api.xro/2.0/Accounts" {
                let m = *m2.lock().unwrap();
                if m == "throttle_once" {
                    *m2.lock().unwrap() = "normal";
                    status = "429 Too Many Requests";
                    extra = "Retry-After: 0\r\nX-Rate-Limit-Problem: minute\r\n".into();
                } else if m == "slow" {
                    std::thread::sleep(std::time::Duration::from_millis(1500));
                }
            }
            let reply = match (method.as_str(), path.as_str()) {
                ("POST", "/connect/token") => {
                    json!({"access_token": "at-1", "refresh_token": "rt-1", "expires_in": 1800,
                           "scope": "openid accounting.settings.read"})
                }
                ("GET", "/connections") => json!([
                    {"tenantId": ROAST, "tenantName": "Tallowmere Coffee Roasting Co.", "tenantType": "ORGANISATION"}
                ]),
                ("GET", "/api.xro/2.0/Accounts") => json!({"Accounts": [
                    {"AccountID": "a-1", "Code": "1010", "Name": "Tallowmere Roastery Operating Account", "Status": "ACTIVE", "Type": "BANK",
                     "UpdatedDateUTC": "/Date(1735689600000+0000)/"}]}),
                _ => json!({}),
            }
            .to_string();
            let _ = stream.write_all(
                format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nX-DayLimit-Remaining: 950\r\n{extra}Connection: close\r\n\r\n{reply}",
                    reply.len()
                )
                .as_bytes(),
            );
        }
    });
    (base, seen, mode)
}

struct Sandbox {
    dir: PathBuf,
    base: String,
    seen: Seen,
    mode: Mode,
    port: u16,
}

impl Sandbox {
    fn new(name: &str, with_token: bool) -> Sandbox {
        let dir = std::env::temp_dir().join(format!("ownpurse-net-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let (base, seen, mode) = fake();
        let port = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let cfg = json!({"default_profile": "demo", "profiles": {"demo": {
            "client_id": "DEMO", "store": "record.sqlite", "token_file": dir.join("tokens.json"),
            "orgs": {"roast": ROAST},
            "xero": {"api_base": format!("{base}/api.xro/2.0"), "token_url": format!("{base}/connect/token"),
                     "connections_url": format!("{base}/connections"), "auth_url": "https://login.example.invalid/authorize",
                     "redirect_uri": format!("http://127.0.0.1:{port}/callback")}}}});
        std::fs::write(dir.join("ownpurse.json"), cfg.to_string()).unwrap();
        if with_token {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs_f64();
            let p = dir.join("tokens.json");
            std::fs::write(
                &p,
                json!({"access_token": "at-0", "refresh_token": "rt-0", "expires_in": 1800, "obtained_at": now,
                       "refreshed_at": now, "scope": "openid", "tenants": []})
                .to_string(),
            )
            .unwrap();
            std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).unwrap();
        }
        Sandbox {
            dir,
            base,
            seen,
            mode,
            port,
        }
    }

    fn cmd(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_ownpurse"));
        c.args(args)
            .current_dir(&self.dir)
            .env("OWNPURSE_CONFIG", self.dir.join("ownpurse.json"))
            .env_remove("OWNPURSE_PROFILE")
            .env_remove("OWNPURSE_STORE");
        c
    }

    fn run(&self, args: &[&str]) -> (i32, String) {
        let out = self.cmd(args).output().unwrap();
        (
            out.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&out.stdout).to_string(),
        )
    }

    fn only_token_url_gets_non_get(&self) {
        for (m, p) in self.seen.lock().unwrap().iter() {
            if m != "GET" {
                assert_eq!((m.as_str(), p.as_str()), ("POST", "/connect/token"));
            }
        }
    }
}

fn first_doc(text: &str) -> Value {
    serde_json::Deserializer::from_str(text)
        .into_iter::<Value>()
        .next()
        .unwrap()
        .unwrap()
}

#[test]
fn sync_writes_the_record_and_reports() {
    let s = Sandbox::new("sync", true);
    let (code, text) = s.run(&["--json", "sync", "roast", "Accounts"]);
    assert_eq!(code, 0, "{text}");
    let d = first_doc(&text);
    assert_eq!(d["calls_used"], 1);
    assert_eq!(d["synced"][0]["org"], "roast");
    assert_eq!(d["synced"][0]["entity"], "Accounts");
    assert_eq!(d["synced"][0]["mode"], "full");
    assert_eq!(d["synced"][0]["new_versions"], 1);
    // the read commands see it at once
    let (code, text) = s.run(&["accounts", "roast"]);
    assert_eq!(code, 0, "{text}");
    assert!(
        text.contains("Tallowmere Roastery Operating Account"),
        "{text}"
    );
    // a second sync records nothing new
    let (_, text) = s.run(&["--json", "sync", "roast", "Accounts"]);
    assert_eq!(first_doc(&text)["synced"][0]["new_versions"], 0);
    s.only_token_url_gets_non_get();
    let _ = &s.base;
}

#[test]
fn offline_refuses_network_commands_without_calls() {
    let s = Sandbox::new("offline", true);
    for args in [
        vec!["--offline", "--json", "sync", "roast"],
        vec!["--offline", "--json", "login"],
    ] {
        let (code, text) = s.run(&args);
        assert_eq!(code, 1, "{args:?}: {text}");
        assert_eq!(first_doc(&text)["error"], "offline");
    }
    let (code, text) = s.run(&["--offline", "--json", "orgs"]);
    assert_eq!(code, 0, "{text}");
    assert!(s.seen.lock().unwrap().is_empty(), "no request at all");
    assert!(
        !s.dir.join("record.sqlite").exists(),
        "--offline never creates a record"
    );
}

#[test]
fn login_write_is_refused_without_allow_writes() {
    // a profile that doesn't allow writes: refused before any request or browser (Phase 8)
    let s = Sandbox::new("write", true);
    let (code, text) = s.run(&["--json", "login", "--write"]);
    assert_eq!(code, 2, "{text}");
    assert_eq!(first_doc(&text)["error"], "writes_disabled", "{text}");
    assert!(s.seen.lock().unwrap().is_empty());
}

#[test]
fn orgs_lists_connections_and_stores_them() {
    let s = Sandbox::new("orgs", true);
    let (code, text) = s.run(&["--json", "orgs"]);
    assert_eq!(code, 0, "{text}");
    assert_eq!(
        first_doc(&text)["orgs"][0]["name"],
        "Tallowmere Coffee Roasting Co."
    );
    let tok: Value =
        serde_json::from_str(&std::fs::read_to_string(s.dir.join("tokens.json")).unwrap()).unwrap();
    assert_eq!(tok["tenants"][0]["tenantId"], ROAST);
}

#[test]
fn login_with_a_simulated_browser() {
    let s = Sandbox::new("login", false);
    let mut child = s
        .cmd(&["--json", "login", "--no-browser", "--timeout", "20"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let mut buf = String::new();
    let url = loop {
        let mut line = String::new();
        assert!(
            out.read_line(&mut line).unwrap() > 0,
            "no URL printed: {buf}"
        );
        buf.push_str(&line);
        if let Some(i) = line.find("https://login.example.invalid/authorize?") {
            break line[i..].trim().trim_end_matches(['"', ',']).to_string();
        }
    };
    let scope = url
        .split("scope=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap();
    for sc in scope.split('+') {
        assert!(
            !(sc.starts_with("accounting.") && !sc.ends_with(".read")),
            "write scope {sc} requested: {url}"
        );
    }
    let state = url
        .split("state=")
        .nth(1)
        .unwrap()
        .split('&')
        .next()
        .unwrap()
        .to_string();
    let mut c = TcpStream::connect(("127.0.0.1", s.port)).unwrap();
    c.write_all(
        format!("GET /callback?code=C0DE&state={state} HTTP/1.1\r\nHost: x\r\n\r\n").as_bytes(),
    )
    .unwrap();
    let mut rest = String::new();
    out.read_to_string(&mut rest).unwrap();
    let status = child.wait().unwrap();
    assert!(status.success(), "{buf}{rest}");
    assert!(rest.contains("\"logged_in\""), "{rest}");
    let tok: Value =
        serde_json::from_str(&std::fs::read_to_string(s.dir.join("tokens.json")).unwrap()).unwrap();
    assert_eq!(tok["refresh_token"], "rt-1");
    assert_eq!(
        tok["tenants"][0]["tenantName"],
        "Tallowmere Coffee Roasting Co."
    );
    assert_eq!(
        std::fs::metadata(s.dir.join("tokens.json"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    s.only_token_url_gets_non_get();
    let _: &Path = &s.dir;
}

fn events(stderr: &str) -> Vec<Value> {
    stderr
        .lines()
        .map(|l| serde_json::from_str(l).unwrap_or_else(|e| panic!("not JSON: {l:?} ({e})")))
        .collect()
}

#[test]
fn progress_jsonl_golden_first_sync() {
    let s = Sandbox::new("progress", true);
    let out = s
        .cmd(&["--json", "sync", "roast", "Accounts", "--progress", "jsonl"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let ev = events(&String::from_utf8_lossy(&out.stderr));
    assert_eq!(
        ev,
        vec![
            json!({"ev": "start", "org": "roast", "entities": 1, "estimate": 1,
                   "estimate_label": "≈ 1+ call (first sync, 1+ per entity)"}),
            json!({"ev": "entity", "org": "roast", "entity": "Accounts", "page": null, "calls": 1, "day_remaining": 950}),
            json!({"ev": "done", "org": "roast", "calls": 1, "records": 1, "new_versions": 1}),
        ]
    );
    // stdout is still the normal envelope
    assert_eq!(
        first_doc(&String::from_utf8_lossy(&out.stdout))["calls_used"],
        1
    );
    // a second sync of the same entity: the record's estimate now applies
    let out = s
        .cmd(&["--json", "sync", "roast", "Accounts", "--progress", "jsonl"])
        .output()
        .unwrap();
    let ev = events(&String::from_utf8_lossy(&out.stderr));
    assert_eq!(ev[0]["estimate"], 1);
    assert_eq!(ev[0]["estimate_label"], "≈ 1 call");
    assert_eq!(ev[2]["new_versions"], 0);
}

#[test]
fn progress_reports_minute_limit_waits() {
    let s = Sandbox::new("wait", true);
    *s.mode.lock().unwrap() = "throttle_once";
    let out = s
        .cmd(&["--json", "sync", "roast", "Accounts", "--progress", "jsonl"])
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    let ev = events(&String::from_utf8_lossy(&out.stderr));
    let kinds: Vec<&str> = ev.iter().map(|e| e["ev"].as_str().unwrap()).collect();
    assert_eq!(kinds, ["start", "entity", "wait", "entity", "done"]);
    assert_eq!(
        ev[2],
        json!({"ev": "wait", "org": "roast", "reason": "minute_limit", "seconds": 1})
    );
    assert_eq!(ev[4]["calls"], 2, "both attempts count");
}

#[test]
fn without_progress_stderr_stays_empty() {
    let s = Sandbox::new("quiet", true);
    let out = s
        .cmd(&["--json", "sync", "roast", "Accounts"])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(
        out.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The reader (the TUI) goes away mid-run: the child must still finish and record its result.
#[test]
fn closed_pipes_never_kill_a_run() {
    let s = Sandbox::new("pipe", true);
    *s.mode.lock().unwrap() = "slow";
    let mut child = s
        .cmd(&["--json", "sync", "roast", "Accounts", "--progress", "jsonl"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    // read the first event, then close both pipes while the request is still in flight
    let mut err = BufReader::new(child.stderr.take().unwrap());
    let mut line = String::new();
    err.read_line(&mut line).unwrap();
    assert!(line.contains("\"start\""), "{line}");
    drop(err);
    drop(child.stdout.take());
    let status = child.wait().unwrap();
    assert!(status.success(), "the run completed: {status}");
    ownpurse_core::record::sqlite_ready(); // first SQLite init on one thread
    let c = rusqlite::Connection::open(s.dir.join("record.sqlite")).unwrap();
    let (outcome, fetched): (String, i64) = c
        .query_row("SELECT outcome, fetched FROM sync_runs", [], |r| {
            Ok((r.get(0)?, r.get(1)?))
        })
        .unwrap();
    assert_eq!((outcome.as_str(), fetched), ("ok", 1));
    let logged: i64 = c
        .query_row(
            "SELECT COUNT(*) FROM http_log WHERE status = 200",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(logged, 1);
}
