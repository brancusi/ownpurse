//! One live world: a temp copy of the demo fixture, private state/runtime folders (so the session registry and
//! sockets are the test's own), a TUI in a PTY, and a way to run the real CLI against it.

use super::pty::Pty;
use super::term::Term;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct Bins {
    pub cli: PathBuf,
    pub tui: PathBuf,
    /// A second TUI build with a newer version (`OWNPURSE_BUILD_VERSION`), for `ui upgrade`.
    pub bumped: Option<PathBuf>,
    /// A TUI build of the SAME version with another build id (`OWNPURSE_BUILD_ID=gfeed001`).
    pub same_version: Option<PathBuf>,
}

impl Bins {
    /// The built binaries in `dir` (e.g. `target/release`).
    pub fn in_dir(dir: &Path) -> Result<Bins, String> {
        let b = Bins {
            cli: dir.join("ownpurse"),
            tui: dir.join("ownpurse-tui"),
            bumped: None,
            same_version: None,
        };
        for p in [&b.cli, &b.tui] {
            if !p.is_file() {
                return Err(format!("{} is missing (build it first)", p.display()));
            }
        }
        Ok(b)
    }
}

pub struct World {
    pub root: PathBuf,
    pub bins: Bins,
    pub pty: Option<Pty>,
    pub cols: u16,
    pub rows: u16,
    /// Extra environment for every process of this world (e.g. the fake CLI's scenario).
    pub extra_env: Vec<(String, String)>,
}

/// What a CLI call returned.
#[derive(Debug, Clone)]
pub struct Out {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
    pub elapsed: Duration,
}

impl Out {
    pub fn json(&self) -> Value {
        serde_json::from_str(&self.stdout).unwrap_or(Value::Null)
    }
}

fn fixtures() -> PathBuf {
    crate::demo::fixtures_dir()
}

impl World {
    /// A fresh copy of the demo world in a temp folder (`name` keeps parallel worlds apart).
    pub fn new(name: &str, bins: Bins) -> Result<World, String> {
        // short path: Unix socket paths are limited to ~100 bytes
        let root = PathBuf::from(format!("/tmp/xad-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("w/db")).map_err(|e| e.to_string())?;
        std::fs::create_dir_all(root.join("home")).map_err(|e| e.to_string())?;
        let demo = fixtures().join("demo");
        for f in [
            "ownpurse.json",
            "record.sqlite",
            "classify.toml",
            "tenants.json",
        ] {
            std::fs::copy(demo.join(f), root.join("w").join(f)).map_err(|e| format!("{f}: {e}"))?;
        }
        std::fs::copy(
            demo.join("db/classify.sqlite"),
            root.join("w/db/classify.sqlite"),
        )
        .map_err(|e| e.to_string())?;
        Ok(World {
            root,
            bins,
            pty: None,
            cols: 100,
            rows: 30,
            extra_env: Vec::new(),
        })
    }

    /// Append one bank transaction with `n` lines to this world's record copy (before `start`), as a sync
    /// would: a new version row and its chain row (the record version is the chain's last seq).
    pub fn add_bulk_lines(&self, n: usize) -> Result<(), String> {
        let items: Vec<serde_json::Value> = (0..n)
            .map(|i| {
                // irregular cents: 3.17, 4.26, 5.35, …
                let cents = 317 + i as i64 * 109;
                serde_json::json!({
                    "AccountCode": "6310",
                    "Description": format!("Cupping sample tin {}", i + 1),
                    "LineAmount": serde_json::Number::from_string_unchecked(format!("{}.{:02}", cents / 100, cents % 100)),
                    "TaxAmount": 0
                })
            })
            .collect();
        let total: i64 = (0..n).map(|i| 317 + i as i64 * 109).sum();
        let payload = serde_json::json!({
            "BankAccount": {"AccountID": "r-op"},
            "BankTransactionID": "bt-bulk",
            "Contact": {"Name": "Brindlewick Tin Works"},
            "DateString": "2025-06-17T00:00:00",
            "IsReconciled": true,
            "LineAmountTypes": "NoTax",
            "LineItems": items,
            "Status": "AUTHORISED",
            "Total": serde_json::Number::from_string_unchecked(format!("{}.{:02}", total / 100, total % 100)),
            "Type": "SPEND"
        });
        let c = rusqlite::Connection::open(self.work().join("record.sqlite"))
            .map_err(|e| e.to_string())?;
        c.execute(
            "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
             SELECT tenant_id, 'BankTransactions', 'bt-bulk', 1, 'bulk', 'AUTHORISED', '2026-10-05T07:46:00Z', 'sync', ?1 \
             FROM versions WHERE xero_id = 'bt-104'",
            [payload.to_string()],
        )
        .map_err(|e| e.to_string())?;
        c.execute(
            "INSERT INTO chain (seq, at, tbl, row_id, row_hash, prev_hash, hash) \
             SELECT seq + 1, '2026-10-05T07:46:00Z', 'versions', (SELECT MAX(id) FROM versions), 'bulk', hash, 'bulk' \
             FROM chain ORDER BY seq DESC LIMIT 1",
            [],
        )
        .map_err(|e| e.to_string())?;
        Ok(())
    }

    /// A sync's pages, as fast as a writer can go: `pages` transactions, each appending a new version of
    /// `bt-104` (an irregular amount) and its chain row. Returns the record version after the last page.
    pub fn sync_pages(&self, pages: usize) -> Result<i64, String> {
        let c = rusqlite::Connection::open(self.work().join("record.sqlite"))
            .map_err(|e| e.to_string())?;
        for p in 0..pages {
            let tx = c.unchecked_transaction().map_err(|e| e.to_string())?;
            let cents = 427_631 + p as i64 * 1_337;
            tx.execute(
                "INSERT INTO versions (tenant_id, entity, xero_id, version, content_hash, status, observed_at, source, payload) \
                 SELECT tenant_id, entity, xero_id, version + 1, 'page' || ?1, status, '2026-10-05T09:48:00Z', 'sync', \
                        json_set(payload, '$.Total', ?2, '$.LineItems[0].LineAmount', ?2) \
                 FROM current WHERE entity = 'BankTransactions' AND xero_id = 'bt-104'",
                rusqlite::params![p as i64, format!("{}.{:02}", cents / 100, cents % 100).parse::<f64>().unwrap_or(0.0)],
            )
            .map_err(|e| e.to_string())?;
            tx.execute(
                "INSERT INTO chain (seq, at, tbl, row_id, row_hash, prev_hash, hash) \
                 SELECT seq + 1, '2026-10-05T09:48:00Z', 'versions', (SELECT MAX(id) FROM versions), 'page', hash, 'page' \
                 FROM chain ORDER BY seq DESC LIMIT 1",
                [],
            )
            .map_err(|e| e.to_string())?;
            tx.commit().map_err(|e| e.to_string())?;
        }
        c.query_row("SELECT MAX(seq) FROM chain", [], |r| r.get(0))
            .map_err(|e| e.to_string())
    }

    pub fn work(&self) -> PathBuf {
        self.root.join("w")
    }

    /// The environment every process of this world runs with (no inherited user state).
    pub fn env(&self) -> Vec<(String, String)> {
        let bin_dir = self
            .bins
            .cli
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        let mut v: Vec<(String, String)> = vec![
            ("HOME".into(), self.root.join("home").display().to_string()),
            (
                "XDG_STATE_HOME".into(),
                self.root.join("state").display().to_string(),
            ),
            (
                "XDG_RUNTIME_DIR".into(),
                self.root.join("run").display().to_string(),
            ),
            (
                "XDG_CONFIG_HOME".into(),
                self.root.join("home/.config").display().to_string(),
            ),
            (
                "OWNPURSE_CONFIG".into(),
                self.work().join("ownpurse.json").display().to_string(),
            ),
            ("OWNPURSE_NOW".into(), "2026-10-05T09:46Z".into()),
            ("OWNPURSE_SYNC".into(), "1".into()),
            ("TERM".into(), "xterm-256color".into()),
            ("LANG".into(), "en_US.UTF-8".into()),
            (
                "PATH".into(),
                format!("{}:/usr/bin:/bin", bin_dir.display()),
            ),
        ];
        v.extend(self.extra_env.iter().cloned());
        v
    }

    /// Start the TUI in a PTY of `cols`×`rows` with extra arguments; waits for its first frame and its
    /// session in the registry.
    pub fn start(&mut self, cols: u16, rows: u16, args: &[&str]) -> Result<(), String> {
        self.cols = cols;
        self.rows = rows;
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
        let pty = Pty::spawn(&self.bins.tui, &args, &self.env(), &self.work(), cols, rows)
            .map_err(|e| format!("spawn TUI: {e}"))?;
        self.pty = Some(pty);
        let start = Instant::now();
        loop {
            let ls = self.cli(&["--json", "ui", "ls"]);
            let n = ls.json()["sessions"].as_array().map_or(0, Vec::len);
            if n > 0 {
                break;
            }
            if start.elapsed() > Duration::from_secs(60) {
                let screen = self.screen().text();
                return Err(format!(
                    "the TUI never registered a session; screen:\n{screen}\nls: {}",
                    ls.stdout
                ));
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        self.settle();
        Ok(())
    }

    pub fn pty(&mut self) -> &mut Pty {
        self.pty.as_mut().expect("TUI started")
    }

    /// Wait for the terminal output to go quiet.
    pub fn settle(&self) {
        if let Some(p) = &self.pty {
            p.settle(Duration::from_millis(40), Duration::from_secs(10));
        }
    }

    pub fn screen(&self) -> Term {
        self.pty
            .as_ref()
            .map(Pty::snapshot)
            .unwrap_or_else(|| Term::new(1, 1))
    }

    /// Run the real CLI.
    pub fn cli(&self, args: &[&str]) -> Out {
        let start = Instant::now();
        let out = std::process::Command::new(&self.bins.cli)
            .args(args)
            .current_dir(self.work())
            .env_clear()
            .envs(self.env())
            .output();
        let elapsed = start.elapsed();
        match out {
            Ok(o) => Out {
                code: o.status.code().unwrap_or(-1),
                stdout: String::from_utf8_lossy(&o.stdout).into_owned(),
                stderr: String::from_utf8_lossy(&o.stderr).into_owned(),
                elapsed,
            },
            Err(e) => Out {
                code: -1,
                stdout: String::new(),
                stderr: e.to_string(),
                elapsed,
            },
        }
    }

    /// `ownpurse --json ui state` (plus extra args).
    pub fn state(&self, extra: &[&str]) -> Value {
        let mut a = vec!["--json", "ui", "state"];
        a.extend_from_slice(extra);
        self.cli(&a).json()
    }

    /// `ui send` with verbs; then waits for the frame.
    pub fn send(&self, args: &[&str]) -> Out {
        let mut a = vec!["--json", "ui", "send"];
        a.extend_from_slice(args);
        let o = self.cli(&a);
        self.settle();
        o
    }

    /// Type keys as the user (raw bytes), then wait for the frame.
    pub fn keys(&mut self, bytes: &[u8]) {
        let _ = self.pty().write(bytes);
        self.settle();
    }
}

impl Drop for World {
    fn drop(&mut self) {
        self.pty = None;
        if std::env::var_os("OWNPURSE_LIVE_KEEP").is_none() {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }
}
