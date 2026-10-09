//! `doctor`: read-only setup checks (binaries on PATH, config, record, token file permissions).
//! Prints a fix for every problem and never a secret: of the token file only its metadata and its `scope`
//! field (the granted scope NAMES) are read; no token value is ever parsed into output. Exit 0 when nothing
//! fails (warnings allowed), 1 otherwise.

use crate::app::{CliError, Global, Res};
use crate::obj;
use crate::out::Out;
use ownpurse_core::config::{Env, load};
use ownpurse_core::record::Record;
use serde_json::{Map, Value};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Ok,
    Warn,
    Fail,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Ok => "ok",
            Status::Warn => "warn",
            Status::Fail => "fail",
        }
    }
}

#[derive(Debug)]
pub struct Check {
    pub name: &'static str,
    pub status: Status,
    pub detail: String,
    pub fix: String,
}

fn check(
    name: &'static str,
    status: Status,
    detail: impl Into<String>,
    fix: impl Into<String>,
) -> Check {
    Check {
        name,
        status,
        detail: detail.into(),
        fix: fix.into(),
    }
}

/// `~/…` for paths under home, so output never carries the user's home folder.
/// Also matches through symlinks (macOS `/var` → `/private/var`).
pub fn tilde(p: &Path, home: &Path) -> String {
    if home.as_os_str().is_empty() || home == Path::new("/") {
        return p.display().to_string();
    }
    let canon = |x: &Path| std::fs::canonicalize(x).unwrap_or_else(|_| x.to_path_buf());
    let homes = [home.to_path_buf(), canon(home)];
    let paths = [p.to_path_buf(), canon(p)];
    for pp in &paths {
        for h in &homes {
            if let Ok(rest) = pp.strip_prefix(h) {
                return format!("~/{}", rest.display());
            }
        }
    }
    p.display().to_string()
}

/// Replace the home folder (canonical form first, so `/private/var/…` never leaves a `/private` stub).
fn scrub_home(text: &str, home: &Path) -> String {
    if home.as_os_str().is_empty() || home == Path::new("/") {
        return text.to_string();
    }
    let canon = std::fs::canonicalize(home).unwrap_or_else(|_| home.to_path_buf());
    text.replace(&canon.display().to_string(), "~")
        .replace(&home.display().to_string(), "~")
}

fn on_path(env: &Env, name: &str) -> Option<PathBuf> {
    let path = env.var("PATH")?;
    std::env::split_paths(path)
        .map(|d| d.join(name))
        .find(|p| p.is_file())
}

#[cfg(unix)]
fn mode(p: &Path) -> Option<u32> {
    use std::os::unix::fs::PermissionsExt;
    std::fs::metadata(p)
        .ok()
        .map(|m| m.permissions().mode() & 0o777)
}

#[cfg(not(unix))]
fn mode(_: &Path) -> Option<u32> {
    None
}

/// All checks, in order. Pure over `env` apart from filesystem metadata and a read-only record open.
pub fn run_checks(env: &Env, profile: Option<&str>) -> Vec<Check> {
    let home = env.home.clone();
    let mut out = Vec::new();

    // 1. binaries on PATH
    let bin = env!("CARGO_PKG_VERSION");
    match on_path(env, "ownpurse") {
        Some(p) => out.push(check("ownpurse on PATH", Status::Ok, format!("{} ({bin})", tilde(&p, &home)), "")),
        None => out.push(check(
            "ownpurse on PATH",
            Status::Fail,
            "not found on PATH",
            "Add ~/.local/bin to PATH (e.g. in ~/.zshrc: export PATH=\"$HOME/.local/bin:$PATH\"), or rerun ./install.sh.",
        )),
    }
    match on_path(env, "ownpurse-tui") {
        Some(p) => out.push(check(
            "ownpurse-tui on PATH",
            Status::Ok,
            tilde(&p, &home),
            "",
        )),
        None => out.push(check(
            "ownpurse-tui on PATH",
            Status::Warn,
            "not found (`ownpurse tui` needs it)",
            "Rerun ./install.sh.",
        )),
    }

    // 2. config
    let config = match load(env, profile, None) {
        Ok(c) => {
            let where_ = c
                .path
                .as_deref()
                .map(|p| tilde(p, &home))
                .unwrap_or_else(|| "environment (OWNPURSE_CLIENT_ID)".into());
            out.push(check(
                "config",
                Status::Ok,
                format!(
                    "{where_} · profile {} · {} org(s)",
                    c.profile.name,
                    c.profile.orgs.len()
                ),
                "",
            ));
            if c.profile.orgs.is_empty() {
                out.push(check(
                    "org aliases",
                    Status::Warn,
                    "no orgs configured in this profile",
                    "Add \"orgs\": {\"roast\": \"<tenant id>\"} to the profile (see ownpurse.example.json).",
                ));
            }
            Some(c)
        }
        Err(e) => {
            out.push(check(
                "config",
                Status::Fail,
                scrub_home(&e.message, &home),
                "Create ownpurse.json in your project folder (copy ownpurse.example.json), or set OWNPURSE_CONFIG.",
            ));
            None
        }
    };

    if let Some(c) = &config {
        // 3. record
        let store = c.store();
        if !store.exists() {
            out.push(check(
                "record",
                Status::Warn,
                format!("{} does not exist yet", tilde(store, &home)),
                "Run the first sync: `ownpurse sync all`.",
            ));
        } else {
            match Record::open(store) {
                Ok(r) => {
                    let versions: i64 = r
                        .conn()
                        .query_row("SELECT COUNT(*) FROM versions", [], |x| x.get(0))
                        .unwrap_or(0);
                    out.push(check(
                        "record",
                        Status::Ok,
                        format!(
                            "{} · {versions} document versions (read-only)",
                            tilde(store, &home)
                        ),
                        "",
                    ));
                    // report snapshots dated beyond today + 1 year (an older build's sentinel): harmless now
                    // (plans and verify ignore them), but worth knowing about
                    let future: i64 = r
                        .conn()
                        .query_row(
                            "SELECT COUNT(*) FROM reports WHERE json_extract(params, '$.date') > date('now', '+1 year')",
                            [],
                            |x| x.get(0),
                        )
                        .unwrap_or(0);
                    if future > 0 {
                        out.push(check(
                            "report dates",
                            Status::Warn,
                            format!("{future} cached report(s) dated after today + 1 year (ignored by plan and verify)"),
                            "Nothing to do: the record is append-only and these rows are ignored.",
                        ));
                    }
                }
                Err(e) => out.push(check(
                    "record",
                    Status::Fail,
                    format!("{}: {e}", tilde(store, &home)),
                    "Check the file is a ownpurse record (`ownpurse verify` is coming in v0.9.x).",
                )),
            }
        }

        // 4. token file: metadata only, never opened
        let tok = &env.cwd.join(&c.profile.token_file); // a relative token_file is relative to the cwd
        match mode(tok) {
            None if !tok.exists() => out.push(check(
                "token file",
                Status::Warn,
                format!("{} not found (not logged in)", tilde(tok, &home)),
                "Log in: `ownpurse login` (opens the browser).",
            )),
            Some(m) if m & 0o077 != 0 => out.push(check(
                "token file",
                Status::Fail,
                format!("{} has mode {m:03o}: readable by others", tilde(tok, &home)),
                format!("chmod 600 {}", tilde(tok, &home)),
            )),
            Some(m) => out.push(check(
                "token file",
                Status::Ok,
                format!("{} · mode {m:03o} (no token value read)", tilde(tok, &home)),
                "",
            )),
            None => out.push(check(
                "token file",
                Status::Ok,
                format!("{} (no token value read)", tilde(tok, &home)),
                "",
            )),
        }
        // 5. the granted scope names (the token file's `scope` field only)
        if tok.exists() {
            scope_checks(&mut out, tok, c);
        }
    }
    out
}

/// The granted scope NAMES, from the token file's `scope` field only (every other field is dropped unread).
pub fn granted_scope_names(tok: &Path) -> Option<Vec<String>> {
    let text = std::fs::read_to_string(tok).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;
    Some(
        v.get("scope")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .split_whitespace()
            // a scope name is lowercase letters, dots and underscores (`accounting.manualjournals`,
            // `offline_access`): anything else (a stray token value) is never echoed
            .filter(|x| {
                x.len() <= 64
                    && x.chars()
                        .all(|c| c.is_ascii_lowercase() || c == '.' || c == '_')
            })
            .map(str::to_string)
            .collect(),
    )
}

fn scope_checks(out: &mut Vec<Check>, tok: &Path, c: &ownpurse_core::config::Config) {
    let Some(names) = granted_scope_names(tok) else {
        out.push(check(
            "scopes",
            Status::Warn,
            "the token file's scope field can't be read",
            "Log in again: `ownpurse login`.",
        ));
        return;
    };
    let is_write = |n: &str| n.starts_with("accounting.") && !n.ends_with(".read");
    let read: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|n| !is_write(n))
        .collect();
    let write: Vec<&str> = names
        .iter()
        .map(String::as_str)
        .filter(|n| is_write(n))
        .collect();
    out.push(check(
        "scopes",
        Status::Ok,
        format!(
            "read: {} · write: {}",
            if read.is_empty() {
                "none".into()
            } else {
                read.join(" ")
            },
            if write.is_empty() {
                "none".into()
            } else {
                write.join(" ")
            }
        ),
        "",
    ));
    if c.profile.allow_writes {
        let want: Vec<String> = c
            .settings
            .get("xero.scopes.write")
            .and_then(Value::as_array)
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        let missing: Vec<&String> = want.iter().filter(|w| !names.contains(w)).collect();
        if missing.is_empty() {
            out.push(check(
                "write scopes",
                Status::Ok,
                "every write scope is granted (allow_writes)",
                "",
            ));
        } else {
            out.push(check(
                "write scopes",
                Status::Warn,
                format!(
                    "allow_writes is set but not granted: {}",
                    missing
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                "The owner grants them: `ownpurse login --write` (a browser consent).",
            ));
        }
    }
}

pub fn doctor(g: &Global) -> Res<()> {
    let env = Env::process();
    let checks = run_checks(&env, g.profile.as_deref());
    let fails = checks.iter().filter(|c| c.status == Status::Fail).count();
    let warns = checks.iter().filter(|c| c.status == Status::Warn).count();
    let rows: Vec<Map<String, Value>> = checks
        .iter()
        .map(|c| {
            obj![
                ("check", c.name),
                ("status", c.status.as_str()),
                ("detail", c.detail.as_str()),
                ("fix", c.fix.as_str()),
            ]
        })
        .collect();
    let mut out = Out::new(g.json);
    out.field("version", env!("CARGO_PKG_VERSION"));
    out.field(
        "result",
        if fails > 0 {
            "fail"
        } else if warns > 0 {
            "ok with warnings"
        } else {
            "ok"
        },
    );
    out.table("checks", &["check", "status", "detail", "fix"], rows, None);
    out.emit();
    if fails > 0 {
        return Err(CliError::runtime(
            "doctor",
            format!("{fails} check(s) failed; see the fix column."),
        ));
    }
    Ok(())
}
