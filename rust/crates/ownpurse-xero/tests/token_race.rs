//! The binding concurrency test: six refresher PROCESSES race on one token file against a loopback token
//! endpoint that rotates the refresh token on every use and rejects a reused one with 400 (as Xero does). Every
//! round must see exactly ONE refresh POST, every process must get the same access token, and the file must stay
//! valid JSON with mode 0600. Repeated 50 times (`OWNPURSE_RACE_ROUNDS` overrides). Mutation check: with the
//! flock in `TokenStore` removed, rounds see several refresh POSTs and rejected reuses, and this test fails.

use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
use std::os::unix::fs::PermissionsExt;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

#[derive(Default)]
struct Server {
    current_rt: String,
    issued: u64,
    posts: u64,
    rejected: u64,
}

/// A loopback token endpoint. Each request is answered after a short delay to widen the race window.
fn serve(state: Arc<Mutex<Server>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/connect/token", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let state = state.clone();
            std::thread::spawn(move || {
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut len = 0usize;
                let mut line = String::new();
                loop {
                    line.clear();
                    if reader.read_line(&mut line).unwrap_or(0) == 0 {
                        return;
                    }
                    let l = line.trim_end();
                    if l.is_empty() {
                        break;
                    }
                    if let Some(v) = l.to_ascii_lowercase().strip_prefix("content-length:") {
                        len = v.trim().parse().unwrap_or(0);
                    }
                }
                let mut body = vec![0u8; len];
                let _ = reader.read_exact(&mut body);
                let body = String::from_utf8_lossy(&body).to_string();
                let rt = body
                    .split('&')
                    .find_map(|kv| kv.strip_prefix("refresh_token="))
                    .unwrap_or("")
                    .to_string();
                std::thread::sleep(std::time::Duration::from_millis(30));
                let (status, reply) = {
                    let mut s = state.lock().unwrap();
                    s.posts += 1;
                    if rt == s.current_rt {
                        s.issued += 1;
                        s.current_rt = format!("rt-{}", s.issued + 1);
                        let reply = json!({"access_token": format!("at-{}", s.issued),
                            "refresh_token": s.current_rt, "expires_in": 1800, "token_type": "Bearer"});
                        (200, reply.to_string())
                    } else {
                        s.rejected += 1;
                        (400, r#"{"error":"invalid_grant"}"#.to_string())
                    }
                };
                let resp = format!(
                    "HTTP/1.1 {status} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{reply}",
                    if status == 200 { "OK" } else { "Bad Request" },
                    reply.len()
                );
                let _ = stream.write_all(resp.as_bytes());
            });
        }
    });
    url
}

#[test]
fn concurrent_refreshers_rotate_exactly_once_per_round() {
    let probe = env!("CARGO_BIN_EXE_ownpurse-token-probe");
    let rounds: u64 = std::env::var("OWNPURSE_RACE_ROUNDS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(50);
    let state = Arc::new(Mutex::new(Server {
        current_rt: "rt-1".into(),
        ..Default::default()
    }));
    let url = serve(state.clone());
    let dir = std::env::temp_dir().join(format!("ownpurse-race-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("tokens.json");

    for round in 0..rounds {
        // a stale token holding the server's current refresh token
        let (rt, posts_before) = {
            let s = state.lock().unwrap();
            (s.current_rt.clone(), s.posts)
        };
        let tok = json!({"access_token": "stale", "refresh_token": rt, "expires_in": 1800,
            "obtained_at": 1000.0, "refreshed_at": 1000.0, "scope": "openid", "tenants": [], "keep": round});
        std::fs::write(&path, serde_json::to_string_pretty(&tok).unwrap()).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();

        // six refresher processes, all started before any is awaited
        let mut kids = Vec::new();
        for i in 0..6 {
            let mut c = Command::new(probe);
            c.arg(&path)
                .arg("CLIENT")
                .arg(&url)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            kids.push((format!("rust-{i}"), c.spawn().unwrap()));
        }
        let mut tokens = Vec::new();
        for (who, k) in kids {
            let out = k.wait_with_output().unwrap();
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            assert!(
                out.status.success(),
                "round {round}: {who} failed: {text} {}",
                String::from_utf8_lossy(&out.stderr)
            );
            tokens.push((who, text));
        }

        let s = state.lock().unwrap();
        assert_eq!(
            s.posts - posts_before,
            1,
            "round {round}: exactly one refresh POST (got {})",
            s.posts - posts_before
        );
        let want = format!("at-{}", s.issued);
        assert!(
            tokens.iter().all(|(_, t)| *t == want),
            "round {round}: every process gets {want}: {tokens:?}"
        );
        let saved: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap())
            .unwrap_or_else(|e| panic!("round {round}: token file is not valid JSON: {e}"));
        assert_eq!(
            saved["refresh_token"],
            s.current_rt.as_str(),
            "round {round}"
        );
        assert_eq!(
            std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
            0o600,
            "round {round}: mode"
        );
    }
    let s = state.lock().unwrap();
    assert_eq!(s.issued, rounds);
    assert_eq!(
        s.rejected, 0,
        "no process ever reused a rotated refresh token"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
